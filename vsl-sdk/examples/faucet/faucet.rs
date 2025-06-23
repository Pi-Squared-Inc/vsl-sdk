use std::{collections::HashSet, path::PathBuf, time::Duration};

use clap::Parser;
use config::Config;
use jsonrpsee::http_client::HttpClientBuilder;
use serde::Deserialize;
use tokio::time::sleep;
use vsl_sdk::{
    Address, Amount, IntoSigned, Timestamp,
    rpc_wrapper::{self, RpcWrapper, RpcWrapperResult, format_amount, parse_amount},
};

const LOOP_INTERVAL: u64 = 5; // seconds

/// Example Faucet for the VSL devnet
///
/// All of the verification logic is handled by the `faucet_verifier`
///
/// The faucet relies on a trusted verifier to check that the client's request
/// meets the requirements to receive funds from the faucet.
///
/// Hence, it listens to claims settled by the verifier and, trusting it,
/// for each such claim it simply transfers the requested funds to the client.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[arg(default_value = "faucet.yml")]
    config_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Settings {
    /// The private key of the faucet
    private_key: String,
    /// The address of a validator signing settled claims
    validator_address: Address,
    /// The address of the verifier authorizing faucet usage
    verifier_address: Address,
    /// The address of the VSL server in the form IP:PORT
    vsl_server_addr: String,
    /// The maximum amount allowed to be requested (*10-18)
    max_amount: u64,
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

    let max_amount = Amount::from_tokens(settings.max_amount as u128);

    // Initialize an Http client for regular RPC communication
    let http_url = format!("http://{}", settings.vsl_server_addr);
    let http_client = HttpClientBuilder::new()
        .build(http_url)
        .expect("Could not connect");

    let mut max_ts = Timestamp::from_seconds(0);
    let mut claims = HashSet::new();
    let mut account =
        RpcWrapper::from_private_key_str(&settings.private_key, None, &http_client).await?;
    loop {
        let response = rpc_wrapper::list_settled_claims_for_sender(
            &http_client,
            &settings.verifier_address,
            &max_ts,
        )
        .await
        .expect("Expected a response");
        for ts_claim in response {
            max_ts = max_ts.max(ts_claim.timestamp.tick());

            let settled_claim = ts_claim.data.tx();
            // (In)sanity checks
            // check  that the signature corresponds to the enclosed message
            let Ok(signer) = settled_claim.recover_address(ts_claim.data.signature()) else {
                eprintln!("Error while checking signature");
                continue;
            };
            // check that the message is signed by the validator
            if signer != settings.validator_address {
                eprintln!("Settled claim is not signed by the recognized validator");
                continue;
            }
            // check that the verifier has indeed verified the claim
            if settled_claim
                .verifiers
                .iter()
                .find(|a| a == &&settings.verifier_address)
                .is_none()
            {
                eprintln!("Recognized verifier not among the verifiers settling the claim");
                continue;
            }
            // check
            let faucet_client = &settled_claim.verified_claim.claim_owner;
            let amount = &settled_claim.verified_claim.claim;
            let Ok(amount) = parse_amount(amount) else {
                eprintln!("Cannot parse the requested amount: {}", amount);
                continue;
            };
            let claim_hash = &settled_claim.verified_claim.claim_id;
            if !claims.insert(claim_hash.clone()) {
                eprintln!("Claim with hash {} already present! Skipping.", claim_hash);
                continue;
            }
            if amount > max_amount {
                eprintln!("Requested amount exceeds max allowed amount");
                continue;
            }
            let Ok(response) = account.pay(&faucet_client, &amount).await else {
                eprintln!(
                    "Error while transfering amount '{}' to client '{}'",
                    format_amount(amount),
                    faucet_client
                );
                continue;
            };
            eprintln!(
                "Payed {} to {} (transaction id: {})",
                format_amount(amount),
                faucet_client,
                response
            );
            sleep(Duration::from_secs(LOOP_INTERVAL)).await;
        }
    }
}
