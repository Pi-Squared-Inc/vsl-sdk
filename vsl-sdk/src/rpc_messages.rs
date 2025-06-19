use alloy::consensus::transaction::{RlpEcdsaDecodableTx, RlpEcdsaEncodableTx};
use alloy::eips::Typed2718;
use alloy::primitives::{Address, B256, Keccak256, keccak256, wrap_fixed_bytes};
use alloy_rlp::{Decodable, Encodable, RlpDecodable, RlpEncodable};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr as _;

use crate::helpers::{HasSender, IntoSigned};
use crate::{Timestamp, impl_rlp_ecdsa_glue};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Some data with an identifier and an associated timestamp
pub struct Timestamped<T> {
    /// Usually a claim identifier (hex-encoded 256 bit hash)
    pub id: String,
    /// The data being timestamped
    pub data: T,
    /// The Timestamp itself
    pub timestamp: Timestamp,
}

impl<T> Timestamped<T> {
    pub fn new(id: String, timestamp: Timestamp, data: T) -> Self {
        Self {
            id,
            data,
            timestamp,
        }
    }
}

impl<T> HasSender for Timestamped<T>
where
    T: HasSender,
{
    fn sender(&self) -> Option<Address> {
        self.data.sender()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RlpEncodable, RlpDecodable)]
/// An (unsigned) vls_submitClaim request for claim-verification
pub struct SubmittedClaim {
    /// the claim to be verified (VSL does not care about how its encoded)
    pub claim: String,
    /// the claim type (could be any string)
    pub claim_type: String,
    /// the proof of the claim (VSL does not care about how its encoded)
    pub proof: String,
    /// the client nonce (64 bit unsigned integer)
    pub nonce: String,
    /// the list of (Ethereum-style) addresses of accounts which can verify this claim
    pub to: Vec<String>,
    //the minimum quorum of signatures
    pub quorum: u16,
    // the (Ethereum-style) address of the client account requesting verification
    pub from: String,
    // the time after which the claim is dropped if not enough verifications are received
    pub expires: Timestamp,
    /// the fee for verification (u128 formatted as hex string).
    pub fee: String,
}

impl HasSender for SubmittedClaim {
    fn sender(&self) -> Option<Address> {
        Address::from_str(&self.from).ok()
    }
}

impl_rlp_ecdsa_glue!(SubmittedClaim);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RlpDecodable, RlpEncodable)]
/// An (unsigned) vls_settleClaim request made by a verifier having verified the claim
pub struct SettleClaimMessage {
    /// The (Ethereum-style) address of the verifier requesting claim settlement
    pub from: String,
    /// The nonce (64 bit unsigned integer) of the verifier requesting claim settlement
    pub nonce: String,
    /// The id (hex-encoded 256 bit hash) of the claim for which claim settlement is requested
    pub target_claim_id: String,
}

impl HasSender for SettleClaimMessage {
    fn sender(&self) -> Option<Address> {
        let Ok(addr) = self.from.parse() else {
            return None;
        };
        Some(addr)
    }
}

impl_rlp_ecdsa_glue!(SettleClaimMessage);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RlpEncodable, RlpDecodable)]
/// Representation of a verified claim
pub struct VerifiedClaim {
    /// the original claim which was verified and now settled
    pub claim: String,
    /// the id (hex-encoded 256 bit hash) of the claim (useful for retrieving the full data of the claim)
    pub claim_id: String,
    /// the claim type
    pub claim_type: String,
    /// the (Ethereum-style) address of the client which produced this claim
    pub claim_owner: String,
}

impl HasSender for VerifiedClaim {}

/// A settled (verified) claim
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RlpEncodable, RlpDecodable)]
pub struct SettledVerifiedClaim {
    /// the claim which was verified
    pub verified_claim: VerifiedClaim,
    /// the (Ethereum-style) addresses of the verifiers which have verified the claim and are part of the quorum
    pub verifiers: Vec<String>,
}

impl HasSender for SettledVerifiedClaim {
    fn sender(&self) -> Option<Address> {
        self.verified_claim.sender()
    }
}

impl_rlp_ecdsa_glue!(SettledVerifiedClaim);

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, RlpDecodable, RlpEncodable, PartialEq, Eq,
)]
/// An (unsigned) vsl_pay request (in VSL tokens)
pub struct PayMessage {
    /// The (Ethereum-style) address of the account requesting the transfer
    pub from: String,
    /// The (Ethereum-style) address of the account receiving the payment
    pub to: String,
    /// The amount to be transfered (u128 formatted as hex string)
    pub amount: String,
    /// The nonce (64 bit unsigned integer) of the account creating the asset
    pub nonce: String,
}

impl HasSender for PayMessage {
    fn sender(&self) -> Option<Address> {
        Address::from_str(&self.from).ok()
    }
}

impl_rlp_ecdsa_glue!(PayMessage);

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, RlpDecodable, RlpEncodable, PartialEq, Eq,
)]
/// An (unsigned) vsl_createAsset request
pub struct CreateAssetMessage {
    /// The (Ethereum-style) address of the account creating the asset
    pub account_id: String,
    /// The nonce (64 bit unsigned integer) of the account creating the asset
    pub nonce: String,
    /// Ticker symbol to be used for the new asset
    pub ticker_symbol: String,
    /// Number of decimals
    pub decimals: u8,
    /// The amount to initialize the new asset with (u128 formatted as hex string)
    pub total_supply: String,
}

impl HasSender for CreateAssetMessage {
    fn sender(&self) -> Option<Address> {
        Address::from_str(&self.account_id).ok()
    }
}

impl_rlp_ecdsa_glue!(CreateAssetMessage);

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, RlpDecodable, RlpEncodable, PartialEq, Eq,
)]
/// The return object of a `vsl_createAsset` request
pub struct CreateAssetResult {
    /// The ID (hex-encoded 256 bit hash) of the asset
    pub asset_id: String,
    /// Settled claim ID for the create asset command  (hex-encoded 256 bit hash)
    pub claim_id: String,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, RlpEncodable, RlpDecodable, PartialEq, Eq,
)]
/// An (unsigned) vsl_transferAsset request
pub struct TransferAssetMessage {
    /// The (Ethereum-style) address of the account transfering the asset
    pub from: String,
    /// The nonce (64 bit unsigned integer) of the account transfering the asset
    pub nonce: String,
    /// The id (hex-encoded 256 bit hash) of the asset (returned when asset was created)
    pub asset_id: String,
    /// The (Ethereum-style) address of the account receiving the asset
    pub to: String,
    /// The amount (of asset) to be transfered (u128 formatted as hex string)
    pub amount: String,
}

impl HasSender for TransferAssetMessage {
    fn sender(&self) -> Option<Address> {
        Address::from_str(&self.from).ok()
    }
}

impl_rlp_ecdsa_glue!(TransferAssetMessage);

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, RlpEncodable, RlpDecodable, PartialEq, Eq,
)]
/// An (unsigned) vsl_setState request
pub struct SetStateMessage {
    /// The (Ethereum-style) address of the account requesting its state to be changed
    pub from: String,
    /// The nonce (64 bit unsigned integer) of the account requesting its state to be changed
    pub nonce: String,
    /// The new state (hex-encoded 256 bit hash)
    pub state: String,
}

impl HasSender for SetStateMessage {
    fn sender(&self) -> Option<Address> {
        Address::from_str(&self.from).ok()
    }
}

impl_rlp_ecdsa_glue!(SetStateMessage);

wrap_fixed_bytes! {
    /// Account State is a 256-bit hash.
    pub struct AccountStateHash<32>;
}
impl AccountStateHash {
    pub fn hash(data: &[u8]) -> Self {
        keccak256(data).into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ValidatorVerifiedClaim {
    Payment(PayMessage),
    AssetCreation(CreateAssetMessage),
    AssetTransfer(TransferAssetMessage),
    SetState(SetStateMessage),
}

impl ValidatorVerifiedClaim {
    pub fn kind(&self) -> &str {
        use ValidatorVerifiedClaim::*;
        match self {
            Payment(_) => "Payment",
            AssetCreation(_) => "AssetCreation",
            AssetTransfer(_) => "AssetTransfer",
            SetState(_) => "SetState",
        }
    }
}

impl<T: Clone + Into<ValidatorVerifiedClaim>> From<&T> for ValidatorVerifiedClaim {
    fn from(value: &T) -> Self {
        value.clone().into()
    }
}

impl From<PayMessage> for ValidatorVerifiedClaim {
    fn from(value: PayMessage) -> Self {
        ValidatorVerifiedClaim::Payment(value)
    }
}

impl From<CreateAssetMessage> for ValidatorVerifiedClaim {
    fn from(value: CreateAssetMessage) -> Self {
        ValidatorVerifiedClaim::AssetCreation(value)
    }
}

impl From<TransferAssetMessage> for ValidatorVerifiedClaim {
    fn from(value: TransferAssetMessage) -> Self {
        ValidatorVerifiedClaim::AssetTransfer(value)
    }
}

impl From<SetStateMessage> for ValidatorVerifiedClaim {
    fn from(value: SetStateMessage) -> Self {
        ValidatorVerifiedClaim::SetState(value)
    }
}

pub trait IdentifiableClaim {
    fn claim_str(&self) -> &str;
    fn claim_nonce_str(&self) -> &str;
    fn claim_owner_str(&self) -> &str;
    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(
            self.claim_owner_str(),
            self.claim_nonce_str(),
            self.claim_str(),
        )
    }

    fn claim_id_hash(owner: &str, nonce: &str, claim: &str) -> B256 {
        let mut hasher = Keccak256::new();
        hasher.update(owner);
        hasher.update(nonce);
        hasher.update(claim);
        hasher.finalize()
    }
}

impl IdentifiableClaim for SubmittedClaim {
    fn claim_str(&self) -> &str {
        &self.claim
    }

    fn claim_nonce_str(&self) -> &str {
        &self.nonce
    }

    fn claim_owner_str(&self) -> &str {
        &self.from
    }
}

impl IdentifiableClaim for VerifiedClaim {
    fn claim_str(&self) -> &str {
        &self.claim
    }

    fn claim_nonce_str(&self) -> &str {
        unimplemented!()
    }

    fn claim_owner_str(&self) -> &str {
        &self.claim_owner
    }

    fn claim_id(&self) -> B256 {
        B256::from_str(&self.claim_id.clone()).unwrap()
    }
}

impl IdentifiableClaim for PayMessage {
    fn claim_str(&self) -> &str {
        todo!()
    }

    fn claim_nonce_str(&self) -> &str {
        &self.nonce
    }

    fn claim_owner_str(&self) -> &str {
        &self.from
    }

    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(
            self.claim_owner_str(),
            self.claim_nonce_str(),
            &serde_json::to_string(&ValidatorVerifiedClaim::from(self)).unwrap(),
        )
    }
}

impl IdentifiableClaim for CreateAssetMessage {
    fn claim_str(&self) -> &str {
        todo!()
    }

    fn claim_nonce_str(&self) -> &str {
        &self.nonce
    }

    fn claim_owner_str(&self) -> &str {
        &self.account_id
    }

    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(
            self.claim_owner_str(),
            self.claim_nonce_str(),
            &serde_json::to_string(&ValidatorVerifiedClaim::from(self)).unwrap(),
        )
    }
}

impl IdentifiableClaim for TransferAssetMessage {
    fn claim_str(&self) -> &str {
        todo!()
    }

    fn claim_nonce_str(&self) -> &str {
        &self.nonce
    }

    fn claim_owner_str(&self) -> &str {
        &self.from
    }

    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(
            self.claim_owner_str(),
            self.claim_nonce_str(),
            &serde_json::to_string(&ValidatorVerifiedClaim::from(self)).unwrap(),
        )
    }
}
