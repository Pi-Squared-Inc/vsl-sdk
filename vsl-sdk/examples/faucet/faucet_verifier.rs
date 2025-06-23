use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use config::Config;
use jsonrpsee::{http_client::HttpClientBuilder, ws_client::WsClientBuilder};
use serde::Deserialize;
use sled::{Db, IVec};
use vsl_sdk::{
    Address, Amount, B256, HasSender, IntoSigned, Timestamp,
    rpc_messages::{IdentifiableClaim, PayMessage, SubmittedClaim},
    rpc_wrapper::{self, RpcWrapper, RpcWrapperResult},
};

/// Example Faucet verifier for the VSL devnet
///
/// A client desiring to request funds from the faucet should send a
/// verification claim request to the VSL service/validator where:
///
/// - the `claim` field should contain the amount requested
///
/// - the `to` list should contain the address of the faucet verifier
///
/// - the `quorum` should be `1`
///
/// - the `fee` a non zero amount (can be `1`)
///
/// - if this is the first time that the client requests funds from the faucet,
///   then the client needs to submit a proof that it is 'whitelisted'.
///   We accept as proof the id of a settled claim proving that the client has
///   received funds from the master account (e.g., at initialization time).
///
/// Note: the client needs to have enough funds to cover the `fee` and
/// the validation fee.
///
/// The faucet verifier listens to claim requests listing it as a verifier.
/// For each such claim, it will ensure that:
///
/// - enough time has passed since the last request for funds from the client
///
/// - the amount requested does not exceed the allowed maximum per request
///
/// If verification passes, the faucet verifier will
///
/// - record the current timestamp for this client
///
/// - settle the claim
///
/// Note: if verification fails (for any reason) there will be no notification
/// to the client. Hence the client should make sure it does not submit a
/// new request too early.
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// Path to the configuration file
    #[arg(default_value = "faucet_verifier.yml")]
    config_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Settings {
    /// Private key of the faucet validator
    private_key: String,
    /// The address of the master account to prove whitelisting
    master_account_address: Address,
    /// The address of a validator signing settled claims
    validator_address: Address,
    /// The address of the VSL server in the form IP:PORT
    vsl_server_addr: String,
    /// The maximum amount allowed to be requested (*10-18)
    max_amount: u64,
    /// Minimum time (in seconds) from the previous request
    min_waiting_time: u64,
    /// Path to the database persisting the last request time for each client
    db_path: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> RpcWrapperResult<()> {
    let args = Args::parse();
    let settings: Settings = Config::builder()
        .add_source(config::File::from(args.config_path))
        .build()
        .expect("Config error")
        .try_deserialize()
        .expect("Config error");
    let db: Db = sled::open(settings.db_path).expect("cannot open database");
    let max_amount = Amount::from_vsl_tokens(settings.max_amount as u128);

    // Initialize an Http client for regular RPC communication
    let http_url = format!("http://{}", settings.vsl_server_addr);
    let http_client = HttpClientBuilder::new()
        .build(http_url)
        .expect("Could not connect");

    // Initialize an Web Socket client for RPC subscriptions
    let ws_url = format!("ws://{}", settings.vsl_server_addr);
    let ws_client = WsClientBuilder::new()
        .build(ws_url)
        .await
        .expect("Could not connect");

    // Initialize the RPC communication wrapped for the faucet
    let mut account =
        RpcWrapper::from_private_key_str(&settings.private_key, None, &http_client).await?;

    // Subscribe to receive submitted claims listing the faucet as a verifier
    let mut submitted_claims_subscription =
        rpc_wrapper::subscribe_to_submitted_claims_for_receiver(&ws_client, account.address())
            .await?;

    loop {
        let Some(request) = submitted_claims_subscription.next().await else {
            panic!("The subscription has been terminated (channel full or dropped)");
        };
        // (In)sanity checks
        // check that response can be deserialized
        let request = match request {
            Ok(request) => request,
            Err(e) => {
                eprintln!("Failed to deserialize: {}", e);
                continue;
            }
        };
        // check that the signature matches the message and the sender is the signer
        let Some(request) = SubmittedClaim::check_and_strip_signature(request.data) else {
            eprintln!("Failed to validate request");
            continue;
        };
        // Check that there is exactly one verifier
        if request.to.len() != 1 {
            eprintln!("Expected a single verifier");
            continue;
        }
        // Check that the verifier address can be parsed
        let Ok(verifier_addr) = Address::from_str(&request.to[0]) else {
            eprintln!("Invalid verifier address");
            continue;
        };
        // Check that the verifier is our account
        if &verifier_addr != account.address() {
            eprintln!("mentioned verifier address different from faucet address");
            continue;
        }
        let Some(client) = request.sender() else {
            eprintln!("Invalid client address");
            continue;
        };
        let Ok(amount) = Amount::from_hex_str(&request.claim) else {
            eprintln!("Cannot parse the requested amount: {}", request.claim);
            continue;
        };
        if amount > max_amount {
            eprintln!("Amount requested larger than maximum allowed amount");
            continue;
        }
        match db.get(client.as_slice()) {
            Ok(Some(old_seconds)) => {
                // check that timestamp is ok
                let Ok((old_seconds, _)) =
                    bincode::decode_from_slice::<u64, _>(&old_seconds, bincode::config::standard())
                else {
                    panic!("Cannot decode saved timestamp");
                };
                if old_seconds + settings.min_waiting_time > Timestamp::now().seconds() {
                    eprintln!("Request came too early. ignoring");
                    continue;
                }
            }
            _ => {
                // assume there is no entry in the db; check proof
                let Ok(claim_id) = B256::from_str(&request.proof) else {
                    eprintln!("Cannot parse proof into a claim id");
                    continue;
                };
                // retrieve claim by id
                let Ok(proof_claim) = account.get_settled_claim_by_id(&claim_id).await else {
                    eprintln!("Cannot locate proof claim");
                    continue;
                };
                let settled_claim = proof_claim.data.tx();
                // check  that the signature corresponds to the enclosed message
                let Ok(signer) = settled_claim.recover_address(proof_claim.data.signature()) else {
                    eprintln!("Error while checking signature for proof claim");
                    continue;
                };
                // check that the message is signed by the validator
                if signer != settings.validator_address {
                    eprintln!("proof claim is not signed by the recognized validator");
                    continue;
                }
                // expect a pay message as the claim
                let Ok(pay_message) =
                    serde_json::from_str::<PayMessage>(&settled_claim.verified_claim.claim)
                else {
                    eprintln!("Cannot decode a proof pay message");
                    continue;
                };
                // decode pay message sender address
                let Some(sender) = pay_message.sender() else {
                    eprintln!("Cannot decode proof pay message sender address");
                    continue;
                };
                // expect payment was made from the master account
                if sender != settings.master_account_address {
                    eprintln!("Expected proof payment was made from the master account");
                    continue;
                }
                // decode pay message receiver address
                let Ok(receiver) = Address::from_str(&pay_message.to) else {
                    eprintln!("Cannot decode proof pay message receiver address");
                    continue;
                };
                // expect payment was made to the client
                if receiver != client {
                    eprintln!("Expected proof payment was made to the client");
                    continue;
                }
            }
        }
        // Checks have passed; Claim has been verified

        // record new timestamp
        let Ok(encoded) =
            bincode::encode_to_vec(Timestamp::now().seconds(), bincode::config::standard())
        else {
            eprintln!("Could not encode timestamp seconds to vector");
            continue;
        };
        let Ok(_) = db.insert(client.as_slice(), IVec::from(encoded)) else {
            eprintln!("Error persisting the new client timestamp");
            continue;
        };

        let _ = db.flush();

        // settle the claim
        let _ = account
            .settle_claim(&request.claim_id())
            .await
            .map_err(|e| eprintln!("Error while requesting claim settlement: {:#?}", e));
    }
}
