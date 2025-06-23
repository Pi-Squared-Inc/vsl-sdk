use alloy::consensus::Signed;
use alloy::consensus::transaction::{RlpEcdsaDecodableTx, RlpEcdsaEncodableTx};
use alloy::eips::Typed2718;
use alloy::primitives::{Address, B256, Keccak256, keccak256, wrap_fixed_bytes};
use alloy_rlp::{Decodable, Encodable, RlpDecodable, RlpEncodable};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::helpers::{HasSender, IntoSigned};
use crate::{Amount, AssetId, Timestamp, impl_rlp_ecdsa_glue};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Some data with an identifier and an associated timestamp
pub struct Timestamped<T> {
    /// Usually a claim identifier (hex-encoded 256 bit hash)
    pub id: B256,
    /// The data being timestamped
    pub data: T,
    /// The Timestamp itself
    pub timestamp: Timestamp,
}

impl<T> Timestamped<T> {
    pub fn new(id: B256, timestamp: Timestamp, data: T) -> Self {
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

#[derive(Debug, Clone, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
/// An (unsigned) vls_submitClaim request for claim-verification
pub struct SubmittedClaim {
    /// the claim to be verified (VSL does not care about how its encoded)
    pub claim: String,
    /// the claim type (could be any string)
    pub claim_type: String,
    /// the proof of the claim (VSL does not care about how its encoded)
    pub proof: String,
    /// the client nonce (64 bit unsigned integer)
    pub nonce: u64,
    /// the list of (Ethereum-style) addresses of accounts which can verify this claim
    pub to: Vec<Address>,
    //the minimum quorum of signatures
    pub quorum: u16,
    // the (Ethereum-style) address of the client account requesting verification
    pub from: Address,
    // the time after which the claim is dropped if not enough verifications are received
    pub expires: Timestamp,
    /// the fee for verification (u128 formatted as hex string).
    pub fee: Amount,
}

impl HasSender for SubmittedClaim {
    fn sender(&self) -> Option<Address> {
        Some(self.from)
    }
}

impl_rlp_ecdsa_glue!(SubmittedClaim);

#[derive(Debug, Clone, Serialize, Deserialize, RlpDecodable, RlpEncodable)]
/// An (unsigned) vls_settleClaim request made by a verifier having verified the claim
pub struct SettleClaimMessage {
    /// The (Ethereum-style) address of the verifier requesting claim settlement
    pub from: Address,
    /// The nonce (64 bit unsigned integer) of the verifier requesting claim settlement
    pub nonce: u64,
    /// The id (hex-encoded 256 bit hash) of the claim for which claim settlement is requested
    pub target_claim_id: B256,
}

impl HasSender for SettleClaimMessage {
    fn sender(&self) -> Option<Address> {
        Some(self.from)
    }
}

impl_rlp_ecdsa_glue!(SettleClaimMessage);

#[derive(Debug, Clone, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
/// Representation of a verified claim
pub struct VerifiedClaim {
    /// the original claim which was verified and now settled
    pub claim: String,
    /// the id (hex-encoded 256 bit hash) of the claim (useful for retrieving the full data of the claim)
    pub claim_id: B256,
    /// the claim type
    pub claim_type: String,
    /// the (Ethereum-style) address of the client which produced this claim
    pub claim_owner: Address,
}

impl HasSender for VerifiedClaim {}

/// Metadata for a settled (verified) claim
#[derive(Debug, Clone, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct SettledClaimData {
    /// the claim type
    pub claim_type: String,
    /// the (Ethereum-style) address of the client which produced this claim
    pub claim_owner: Address,
    /// the (Ethereum-style) addresses of the verifiers which have verified the claim and are part of the quorum
    pub verifiers: Vec<Address>,
}

impl From<SettledVerifiedClaim> for SettledClaimData {
    fn from(value: SettledVerifiedClaim) -> Self {
        Self {
            claim_type: value.verified_claim.claim_type,
            claim_owner: value.verified_claim.claim_owner,
            verifiers: value.verifiers,
        }
    }
}

impl From<&SettledVerifiedClaim> for SettledClaimData {
    fn from(value: &SettledVerifiedClaim) -> Self {
        Self {
            claim_type: value.verified_claim.claim_type.clone(),
            claim_owner: value.verified_claim.claim_owner.clone(),
            verifiers: value.verifiers.clone(),
        }
    }
}

/// Metadata for a submitted claim
#[derive(Debug, Clone, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct SubmittedClaimData {
    /// the claim type (could be any string)
    pub claim_type: String,
    /// the client nonce (64 bit unsigned integer)
    pub nonce: u64,
    /// the list of (Ethereum-style) addresses of accounts which can verify this claim
    pub to: Vec<Address>,
    //the minimum quorum of signatures
    pub quorum: u16,
    // the (Ethereum-style) address of the client account requesting verification
    pub from: Address,
    // the time after which the claim is dropped if not enough verifications are received
    pub expires: Timestamp,
    /// the fee for verification (u128 formatted as hex string).
    pub fee: Amount,
}

impl From<SubmittedClaim> for SubmittedClaimData {
    fn from(value: SubmittedClaim) -> Self {
        Self {
            claim_type: value.claim_type,
            nonce: value.nonce,
            to: value.to,
            quorum: value.quorum,
            from: value.from,
            expires: value.expires,
            fee: value.fee,
        }
    }
}

impl From<&SubmittedClaim> for SubmittedClaimData {
    fn from(value: &SubmittedClaim) -> Self {
        Self {
            claim_type: value.claim_type.clone(),
            nonce: value.nonce.clone(),
            to: value.to.clone(),
            quorum: value.quorum,
            from: value.from.clone(),
            expires: value.expires,
            fee: value.fee.clone(),
        }
    }
}

impl<'a, T1, T2> From<&'a Timestamped<Signed<T1>>> for Timestamped<T2>
where
    T2: From<&'a T1>,
{
    fn from(ts: &'a Timestamped<Signed<T1>>) -> Self {
        Timestamped {
            id: ts.id.clone(),
            data: T2::from(ts.data.tx()),
            timestamp: ts.timestamp,
        }
    }
}

/// A settled (verified) claim
#[derive(Debug, Clone, Serialize, Deserialize, RlpEncodable, RlpDecodable)]
pub struct SettledVerifiedClaim {
    /// the claim which was verified
    pub verified_claim: VerifiedClaim,
    /// the (Ethereum-style) addresses of the verifiers which have verified the claim and are part of the quorum
    pub verifiers: Vec<Address>,
}

impl HasSender for SettledVerifiedClaim {
    fn sender(&self) -> Option<Address> {
        self.verified_claim.sender()
    }
}

impl_rlp_ecdsa_glue!(SettledVerifiedClaim);

#[derive(Debug, Clone, Serialize, Deserialize, RlpDecodable, RlpEncodable, PartialEq, Eq)]
/// An (unsigned) vsl_pay request (in VSL tokens)
pub struct PayMessage {
    /// The (Ethereum-style) address of the account requesting the transfer
    pub from: Address,
    /// The (Ethereum-style) address of the account receiving the payment
    pub to: Address,
    /// The amount to be transfered (u128 formatted as hex string)
    pub amount: Amount,
    /// The nonce (64 bit unsigned integer) of the account creating the asset
    pub nonce: u64,
}

impl HasSender for PayMessage {
    fn sender(&self) -> Option<Address> {
        Some(self.from)
    }
}

impl_rlp_ecdsa_glue!(PayMessage);

#[derive(Debug, Clone, Serialize, Deserialize, RlpDecodable, RlpEncodable, PartialEq, Eq)]
/// An (unsigned) vsl_createAsset request
pub struct CreateAssetMessage {
    /// The (Ethereum-style) address of the account creating the asset
    pub account_id: Address,
    /// The nonce (64 bit unsigned integer) of the account creating the asset
    pub nonce: u64,
    /// Ticker symbol to be used for the new asset
    pub ticker_symbol: String,
    /// Number of decimals
    pub decimals: u8,
    /// The amount to initialize the new asset with (u128 formatted as hex string)
    pub total_supply: Amount,
}

impl HasSender for CreateAssetMessage {
    fn sender(&self) -> Option<Address> {
        Some(self.account_id)
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

#[derive(Debug, Clone, Serialize, Deserialize, RlpEncodable, RlpDecodable, PartialEq, Eq)]
/// An (unsigned) vsl_transferAsset request
pub struct TransferAssetMessage {
    /// The (Ethereum-style) address of the account transfering the asset
    pub from: Address,
    /// The nonce (64 bit unsigned integer) of the account transfering the asset
    pub nonce: u64,
    /// The id (hex-encoded 256 bit hash) of the asset (returned when asset was created)
    pub asset_id: AssetId,
    /// The (Ethereum-style) address of the account receiving the asset
    pub to: Address,
    /// The amount (of asset) to be transfered (u128 formatted as hex string)
    pub amount: Amount,
}

impl HasSender for TransferAssetMessage {
    fn sender(&self) -> Option<Address> {
        Some(self.from)
    }
}

impl_rlp_ecdsa_glue!(TransferAssetMessage);

#[derive(Debug, Clone, Serialize, Deserialize, RlpEncodable, RlpDecodable, PartialEq, Eq)]
/// An (unsigned) vsl_setState request
pub struct SetStateMessage {
    /// The (Ethereum-style) address of the account requesting its state to be changed
    pub from: Address,
    /// The nonce (64 bit unsigned integer) of the account requesting its state to be changed
    pub nonce: u64,
    /// The new state (hex-encoded 256 bit hash)
    pub state: AccountStateHash,
}

impl HasSender for SetStateMessage {
    fn sender(&self) -> Option<Address> {
        Some(self.from)
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    fn claim_nonce(&self) -> u64;
    fn claim_owner(&self) -> &Address;
    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(self.claim_owner(), self.claim_nonce(), self.claim_str())
    }

    fn claim_id_hash(owner: &Address, nonce: u64, claim: &str) -> B256 {
        let mut hasher = Keccak256::new();
        hasher.update(owner);
        hasher.update(nonce.to_be_bytes());
        hasher.update(claim);
        hasher.finalize()
    }
}

impl IdentifiableClaim for SubmittedClaim {
    fn claim_str(&self) -> &str {
        &self.claim
    }

    fn claim_nonce(&self) -> u64 {
        self.nonce
    }

    fn claim_owner(&self) -> &Address {
        &self.from
    }
}

impl IdentifiableClaim for VerifiedClaim {
    fn claim_str(&self) -> &str {
        &self.claim
    }

    fn claim_nonce(&self) -> u64 {
        unimplemented!()
    }

    fn claim_owner(&self) -> &Address {
        &self.claim_owner
    }

    fn claim_id(&self) -> B256 {
        self.claim_id
    }
}

impl IdentifiableClaim for PayMessage {
    fn claim_str(&self) -> &str {
        todo!()
    }

    fn claim_nonce(&self) -> u64 {
        self.nonce
    }

    fn claim_owner(&self) -> &Address {
        &self.from
    }

    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(
            self.claim_owner(),
            self.claim_nonce(),
            &serde_json::to_string(&ValidatorVerifiedClaim::from(self)).unwrap(),
        )
    }
}

impl IdentifiableClaim for CreateAssetMessage {
    fn claim_str(&self) -> &str {
        todo!()
    }

    fn claim_nonce(&self) -> u64 {
        self.nonce
    }

    fn claim_owner(&self) -> &Address {
        &self.account_id
    }

    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(
            self.claim_owner(),
            self.claim_nonce(),
            &serde_json::to_string(&ValidatorVerifiedClaim::from(self)).unwrap(),
        )
    }
}

impl IdentifiableClaim for TransferAssetMessage {
    fn claim_str(&self) -> &str {
        todo!()
    }

    fn claim_nonce(&self) -> u64 {
        self.nonce
    }

    fn claim_owner(&self) -> &Address {
        &self.from
    }

    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(
            self.claim_owner(),
            self.claim_nonce(),
            &serde_json::to_string(&ValidatorVerifiedClaim::from(self)).unwrap(),
        )
    }
}
