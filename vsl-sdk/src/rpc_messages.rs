use alloy::consensus::Signed;
use alloy::consensus::transaction::{RlpEcdsaDecodableTx, RlpEcdsaEncodableTx};
use alloy::eips::Typed2718;
use alloy::primitives::{Address, B256, Keccak256, keccak256, wrap_fixed_bytes};
use alloy_rlp::{BufMut, Decodable, Encodable, RlpDecodable, RlpEncodable};
use schemars::schema::{Metadata, Schema, SchemaObject, SingleOrVec};
use schemars::{JsonSchema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::helpers::{HasSender, IntoSigned};
use crate::{Timestamp, impl_rlp_ecdsa_glue};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct VslAddress {
    pub address: Address,
}

impl VslAddress {
    pub fn new(address: Address) -> Self {
        Self { address }
    }
}

impl Decodable for VslAddress {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let address = Address::decode(buf)?;
        Ok(VslAddress::new(address))
    }
}

impl Encodable for VslAddress {
    fn encode(&self, out: &mut dyn BufMut) {
        self.address.encode(out);
    }
}

impl FromStr for VslAddress {
    type Err = <Address as FromStr>::Err;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Address::from_str(s).map(VslAddress::new)
    }
}

impl fmt::Display for VslAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.address.fmt(f)
    }
}

impl JsonSchema for VslAddress {
    fn schema_name() -> String {
        "VslAddress".to_string()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Owned("vsl_sdk::VslAddress".to_string())
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        Schema::Object(SchemaObject {
            metadata: Some(Box::new(Metadata {
                id: None,
                title: Some("VslAddress".to_string()),
                description: Some("An Ethereum address.".to_string()),
                default: None,
                deprecated: false,
                read_only: false,
                write_only: false,
                examples: vec![serde_json::json!(
                    "0x75c51B0770646902999e55D86c5F399FaF6AbDc7"
                )],
            })),
            instance_type: Some(SingleOrVec::Single(Box::new(
                schemars::schema::InstanceType::String,
            ))),
            format: None,
            enum_values: None,
            const_value: None,
            subschemas: None,
            number: None,
            string: Some(Box::new(schemars::schema::StringValidation {
                min_length: Some(42),
                max_length: Some(42),
                pattern: Some("^0x[a-fA-F0-9]{40}$".to_string()),
            })),
            array: None,
            object: None,
            reference: None,
            extensions: schemars::Map::new(),
        })
    }
}

impl From<Address> for VslAddress {
    fn from(address: Address) -> Self {
        VslAddress { address }
    }
}

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
    pub to: Vec<VslAddress>,
    //the minimum quorum of signatures
    pub quorum: u16,
    // the (Ethereum-style) address of the client account requesting verification
    pub from: VslAddress,
    // the time after which the claim is dropped if not enough verifications are received
    pub expires: Timestamp,
    /// the fee for verification (u128 formatted as hex string).
    pub fee: String,
}

impl HasSender for SubmittedClaim {
    fn sender(&self) -> Option<Address> {
        Some(self.from.address)
    }
}

impl_rlp_ecdsa_glue!(SubmittedClaim);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RlpDecodable, RlpEncodable)]
/// An (unsigned) vls_settleClaim request made by a verifier having verified the claim
pub struct SettleClaimMessage {
    /// The (Ethereum-style) address of the verifier requesting claim settlement
    pub from: VslAddress,
    /// The nonce (64 bit unsigned integer) of the verifier requesting claim settlement
    pub nonce: String,
    /// The id (hex-encoded 256 bit hash) of the claim for which claim settlement is requested
    pub target_claim_id: String,
}

impl HasSender for SettleClaimMessage {
    fn sender(&self) -> Option<Address> {
        Some(self.from.address)
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
    pub claim_owner: VslAddress,
}

impl HasSender for VerifiedClaim {}

/// Metadata for a settled (verified) claim
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RlpEncodable, RlpDecodable)]
pub struct SettledClaimData {
    /// the claim type
    pub claim_type: String,
    /// the (Ethereum-style) address of the client which produced this claim
    pub claim_owner: VslAddress,
    /// the (Ethereum-style) addresses of the verifiers which have verified the claim and are part of the quorum
    pub verifiers: Vec<VslAddress>,
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RlpEncodable, RlpDecodable)]
pub struct SubmittedClaimData {
    /// the claim type (could be any string)
    pub claim_type: String,
    /// the client nonce (64 bit unsigned integer)
    pub nonce: String,
    /// the list of (Ethereum-style) addresses of accounts which can verify this claim
    pub to: Vec<VslAddress>,
    //the minimum quorum of signatures
    pub quorum: u16,
    // the (Ethereum-style) address of the client account requesting verification
    pub from: VslAddress,
    // the time after which the claim is dropped if not enough verifications are received
    pub expires: Timestamp,
    /// the fee for verification (u128 formatted as hex string).
    pub fee: String,
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, RlpEncodable, RlpDecodable)]
pub struct SettledVerifiedClaim {
    /// the claim which was verified
    pub verified_claim: VerifiedClaim,
    /// the (Ethereum-style) addresses of the verifiers which have verified the claim and are part of the quorum
    pub verifiers: Vec<VslAddress>,
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
    pub from: VslAddress,
    /// The (Ethereum-style) address of the account receiving the payment
    pub to: VslAddress,
    /// The amount to be transfered (u128 formatted as hex string)
    pub amount: String,
    /// The nonce (64 bit unsigned integer) of the account creating the asset
    pub nonce: String,
}

impl HasSender for PayMessage {
    fn sender(&self) -> Option<Address> {
        Some(self.from.address)
    }
}

impl_rlp_ecdsa_glue!(PayMessage);

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, RlpDecodable, RlpEncodable, PartialEq, Eq,
)]
/// An (unsigned) vsl_createAsset request
pub struct CreateAssetMessage {
    /// The (Ethereum-style) address of the account creating the asset
    pub account_id: VslAddress,
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
        Some(self.account_id.address)
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
    pub from: VslAddress,
    /// The nonce (64 bit unsigned integer) of the account transfering the asset
    pub nonce: String,
    /// The id (hex-encoded 256 bit hash) of the asset (returned when asset was created)
    pub asset_id: String,
    /// The (Ethereum-style) address of the account receiving the asset
    pub to: VslAddress,
    /// The amount (of asset) to be transfered (u128 formatted as hex string)
    pub amount: String,
}

impl HasSender for TransferAssetMessage {
    fn sender(&self) -> Option<Address> {
        Some(self.from.address)
    }
}

impl_rlp_ecdsa_glue!(TransferAssetMessage);

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, RlpEncodable, RlpDecodable, PartialEq, Eq,
)]
/// An (unsigned) vsl_setState request
pub struct SetStateMessage {
    /// The (Ethereum-style) address of the account requesting its state to be changed
    pub from: VslAddress,
    /// The nonce (64 bit unsigned integer) of the account requesting its state to be changed
    pub nonce: String,
    /// The new state (hex-encoded 256 bit hash)
    pub state: String,
}

impl HasSender for SetStateMessage {
    fn sender(&self) -> Option<Address> {
        Some(self.from.address)
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
    fn claim_owner(&self) -> &Address;
    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(self.claim_owner(), self.claim_nonce_str(), self.claim_str())
    }

    fn claim_id_hash(owner: &Address, nonce: &str, claim: &str) -> B256 {
        let mut hasher = Keccak256::new();
        hasher.update(owner.to_string().to_lowercase());
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

    fn claim_owner(&self) -> &Address {
        &self.from.address
    }
}

impl IdentifiableClaim for VerifiedClaim {
    fn claim_str(&self) -> &str {
        &self.claim
    }

    fn claim_nonce_str(&self) -> &str {
        unimplemented!()
    }

    fn claim_owner(&self) -> &Address {
        &self.claim_owner.address
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

    fn claim_owner(&self) -> &Address {
        &self.from.address
    }

    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(
            self.claim_owner(),
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

    fn claim_owner(&self) -> &Address {
        &self.account_id.address
    }

    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(
            self.claim_owner(),
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

    fn claim_owner(&self) -> &Address {
        &self.from.address
    }

    fn claim_id(&self) -> B256 {
        Self::claim_id_hash(
            &self.claim_owner(),
            self.claim_nonce_str(),
            &serde_json::to_string(&ValidatorVerifiedClaim::from(self)).unwrap(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Collected public data about an account
pub struct AccountData {
    /// the account nonce (64 bit unsigned integer)
    pub nonce: u64,
    /// the account native token balance (u128 formatted as hex string).
    pub balance: String,
    /// The balances of all assets held by the account.
    pub asset_balances: HashMap<String, String>,
    /// the current state of the account (a 256-bit hash), or `None` if unset.
    pub state: Option<String>,
}
