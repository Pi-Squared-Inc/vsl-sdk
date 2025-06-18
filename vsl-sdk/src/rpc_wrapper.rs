use std::collections::HashMap;
use std::str::FromStr;

use alloy::consensus::Signed;
use alloy::hex::{FromHex as _, FromHexError};
use alloy::signers::Error as SignError;
use alloy::signers::k256::SecretKey;
use alloy::signers::local::PrivateKeySigner;
use jsonrpsee::core::client::{ClientT, Error as RpcError, Subscription, SubscriptionClientT};
use jsonrpsee::rpc_params;
use jsonrpsee::ws_client::WsClient;

use crate::helpers::IntoSigned;
use crate::rpc_messages::{
    AccountStateHash, CreateAssetMessage, IdentifiableClaim as _, PayMessage, SetStateMessage,
    SettleClaimMessage, SettledVerifiedClaim, SubmittedClaim, Timestamped, TransferAssetMessage,
};
use crate::{Address, Amount, ParseAmountError, Timestamp, B256};

/// Metadata about an asset
pub struct AssetData {
    /// The address of the account creating the asset
    pub account_id: Address,
    /// The nonce of the account creating the asset
    pub nonce: u64,
    /// Ticker symbol to be used for the new asset
    pub ticker_symbol: String,
    /// Number of decimals
    pub decimals: u8,
    /// The amount used to initialize the asset
    pub total_supply: Amount,
}

impl TryFrom<CreateAssetMessage> for AssetData {
    type Error = RpcWrapperError;

    fn try_from(create_asset_message: CreateAssetMessage) -> Result<Self, Self::Error> {
        let CreateAssetMessage {
            account_id,
            nonce,
            ticker_symbol,
            decimals,
            total_supply,
        } = create_asset_message;
        let Ok(account_id) = Address::from_str(&account_id) else {
            return Err(RpcWrapperError::ParseError(
                "Cannot parse Address".to_string(),
            ));
        };
        let Ok(nonce) = u64::from_str_radix(&nonce, 10) else {
            return Err(RpcWrapperError::ParseError(
                "Cannot parse nonce".to_string(),
            ));
        };
        let total_supply = Amount::from_hex_str_with_decimals(&total_supply, decimals)?;
        Ok(Self {
            account_id,
            nonce,
            ticker_symbol,
            decimals,
            total_supply,
        })
    }
}

impl Into<CreateAssetMessage> for AssetData {

    fn into(self) -> CreateAssetMessage {
        CreateAssetMessage {
            account_id: self.account_id.to_string(),
            nonce: self.nonce.to_string(),
            ticker_symbol: self.ticker_symbol,
            decimals: self.decimals,
            total_supply: self.total_supply.to_hex_string(),
        }
    }
}

#[derive(Debug)]
pub enum RpcWrapperError {
    RpcError(RpcError),
    FromHexError(FromHexError),
    SignError(SignError),
    AmountError(ParseAmountError),
    ParseError(String),
    NonExistentAsset,
    InvalidAmount,
}

impl From<RpcError> for RpcWrapperError {
    fn from(value: RpcError) -> Self {
        Self::RpcError(value)
    }
}

impl From<FromHexError> for RpcWrapperError {
    fn from(value: FromHexError) -> Self {
        Self::FromHexError(value)
    }
}

impl From<SignError> for RpcWrapperError {
    fn from(value: SignError) -> Self {
        Self::SignError(value)
    }
}

impl From<ParseAmountError> for RpcWrapperError {
    fn from(value: ParseAmountError) -> Self {
        Self::AmountError(value)
    }
}

pub type RpcWrapperResult<T> = Result<T, RpcWrapperError>;

pub struct RpcWrapper<T> {
    key: PrivateKeySigner,
    address: Address,
    nonce: u64,
    rpc_client: T,
}

impl<T> RpcWrapper<T>
where
    T: ClientT + Clone,
{
    pub async fn from_signer(
        signer: PrivateKeySigner,
        nonce: Option<u64>,
        rpc_client: &T,
    ) -> RpcWrapperResult<Self> {
        let nonce = nonce.unwrap_or(get_account_nonce(rpc_client, &signer.address()).await?);
        Ok(Self {
            address: signer.address(),
            key: signer,
            nonce,
            rpc_client: rpc_client.clone(),
        })
    }

    pub async fn from_private_key_str(
        private_key_str: &str,
        nonce: Option<u64>,
        rpc_client: &T,
    ) -> RpcWrapperResult<Self> {
        let bytes = <[u8; 32]>::from_hex(private_key_str).expect("Could not extract private key");
        Self::from_private_key_bytes(bytes, nonce, rpc_client).await
    }

    pub async fn from_private_key_bytes(
        bytes: [u8; 32],
        nonce: Option<u64>,
        rpc_client: &T,
    ) -> RpcWrapperResult<Self> {
        let secret_key = SecretKey::from_bytes(&bytes.into()).expect("could not parse private key");
        Self::from_secret_key(secret_key, nonce, rpc_client).await
    }

    pub async fn from_secret_key(
        secret_key: SecretKey,
        nonce: Option<u64>,
        rpc_client: &T,
    ) -> RpcWrapperResult<Self> {
        let signer = PrivateKeySigner::from(secret_key);
        Self::from_signer(signer, nonce, rpc_client).await
    }

    pub fn inc_nonce(&mut self) {
        self.nonce += 1;
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn rpc_client(&self) -> &T {
        &self.rpc_client
    }

    pub fn sign<Signable: IntoSigned>(
        &self,
        message: Signable,
    ) -> alloy::signers::Result<Signed<Signable>> {
        message.into_signed(&self.key)
    }

    pub fn claim_id(&self, claim: &str) -> B256 {
        SubmittedClaim::claim_id_hash(&self.address.to_string(), &self.nonce.to_string(), claim)
    }

    pub async fn submit_claim(
        &mut self,
        // the claim to be verified
        claim: String,
        // the claim type
        claim_type: String,
        // the proof of the claim
        proof: String,
        // the list of verifiers to receive this claim
        to: Vec<&Address>,
        // the minimum quorum of signatures
        quorum: u16,
        // the time after which the claim is dropped if not enough verifications are received
        expires: Timestamp,
        // the total fee for verification and claim validation
        fee: Amount,
    ) -> RpcWrapperResult<B256> {
        let submitted_claim = SubmittedClaim {
            claim,
            claim_type,
            proof,
            nonce: self.nonce().to_string(),
            to: to.iter().map(ToString::to_string).collect(),
            quorum,
            from: self.address().to_string(),
            expires,
            fee: fee.to_hex_string(),
        };
        let claim = self.sign(submitted_claim)?;
        let response: String = self
            .rpc_client
            .request("vsl_submitClaim", rpc_params![claim])
            .await?;
        self.inc_nonce();
        Ok(B256::from_str(&response)?)
    }

    pub async fn settle_claim(
        &mut self,
        // The id of the claim for which claim settlement is requested
        claim_id: &B256,
    ) -> RpcWrapperResult<()> {
        let settle_claim_message = SettleClaimMessage {
            from: self.address.to_string(),
            nonce: self.nonce().to_string(),
            target_claim_id: claim_id.to_string(),
        };
        let claim = self.sign(settle_claim_message)?;
        let _: String = self
            .rpc_client
            .request("vsl_settleClaim", rpc_params![claim])
            .await?;
        self.inc_nonce();
        Ok(())
    }

    /// Submits a VSL payment from one account to another.
    ///
    /// - The `amount` will be transfered from current object to `to`.
    /// - The validation fee will also be deducted from the sender's account.
    /// - A [SettledVerifiedClaim] will be recorded containing the json-serialized [PayMessage] as its claim
    ///
    /// - Input:
    ///   * `to`     - address of the account receiving the payment
    ///   * `amount` - amount to be transfered
    ///
    /// - Returns: the settled payment claim ID
    ///
    /// Will fail if:
    ///
    /// - sender balance cannot cover the specified `amount` and the validation fee
    pub async fn pay(&mut self, to: &Address, amount: &Amount) -> RpcWrapperResult<B256> {
        let pay_message = PayMessage {
            from: self.address().to_string(),
            nonce: self.nonce().to_string(),
            to: to.to_string(),
            amount: amount.to_hex_string(),
        };
        let signed_claim = self.sign(pay_message)?;
        let response: String = self
            .rpc_client
            .request("vsl_pay", rpc_params![signed_claim])
            .await?;
        self.inc_nonce();
        Ok(B256::from_str(&response)?)
    }

    /// Creates a new asset on the network.
    ///
    /// - A [SettledVerifiedClaim] will be recorded containing the json-serialized [CreateAssetMessage] as its claim
    ///
    /// - Input:
    ///   *
    /// - Returns: the asset ID of the newly created asset.
    ///
    /// Will fail if:
    ///
    /// - sender balance cannot cover validation fee
    /// - `total` uses more decimals than allowed by `decimals`
    pub async fn create_asset(
        &mut self,
        ticker_symbol: &str,
        decimals: u8,
        total_supply: &Amount,
    ) -> RpcWrapperResult<String> {
        let create_asset_message =
            self.create_asset_message(ticker_symbol, decimals, total_supply);
        let signed_claim = self.sign(create_asset_message)?;
        let response: String = self
            .rpc_client
            .request("vsl_createAsset", rpc_params![signed_claim])
            .await?;
        self.inc_nonce();
        Ok(response)
    }

    pub fn create_asset_message(
        &mut self,
        ticker_symbol: &str,
        decimals: u8,
        total_supply: &Amount,
    ) -> CreateAssetMessage {
        AssetData {
            account_id: self.address,
            nonce: self.nonce,
            ticker_symbol: ticker_symbol.to_string(),
            decimals,
            total_supply: *total_supply,
        }
        .into()
    }

    /// Transfers a specific asset from one account to another.
    ///
    /// - A [SettledVerifiedClaim] will be recorded containing the json-serialized [TransferAssetMessage] as its claim
    ///
    /// - Input:
    ///   * `asset_id`  - the id of the asset
    ///   * `to`     - address of the account receiving the asset
    ///   * `amount` - amount (of asset) to be transfered
    ///
    /// - Returns: the settled payment claim ID
    ///
    /// Will fail if:
    ///
    /// - sender balance cannot cover validation fee
    /// - sender asset balance cannot cover `amount`
    /// - `amount` uses more decimals than allowed by the asset metadata
    pub async fn transfer_asset(
        &mut self,
        asset_id: &str,
        to: &Address,
        amount: &Amount,
    ) -> RpcWrapperResult<B256> {
        let Some(asset_data) = self.get_asset_by_id(asset_id).await? else {
            return Err(RpcWrapperError::NonExistentAsset);
        };
        if amount.decimals != asset_data.decimals {
            return Err(RpcWrapperError::InvalidAmount);
        }
        let transfer_asset_message =
            self.transfer_asset_message(asset_id, to, amount)?;
        let signed_claim = self.sign(transfer_asset_message)?;
        let response: String = self
            .rpc_client
            .request("vsl_transferAsset", rpc_params![signed_claim])
            .await?;
        self.inc_nonce();
        Ok(B256::from_str(&response)?)
    }

    pub fn transfer_asset_message(
        &self,
        asset_id: &str,
        to: &Address,
        amount: &Amount,
    ) -> RpcWrapperResult<TransferAssetMessage> {
        Ok(TransferAssetMessage {
            from: self.address().to_string(),
            nonce: self.nonce().to_string(),
            to: to.to_string(),
            amount: amount.to_hex_string(),
            asset_id: asset_id.to_string(),
        })
    }

    /// Sets the account's current state.
    ///
    /// - Input:
    ///   * state - a 256-bit hash
    /// - Returns: a settled claim ID for the set state transaction.
    ///
    /// Will fail if:
    ///
    /// - sender balance cannot cover validation fee    
    pub async fn set_account_state(&mut self, state: &AccountStateHash) -> RpcWrapperResult<B256> {
        let set_state_message = SetStateMessage {
            from: self.address().to_string(),
            nonce: self.nonce().to_string(),
            state: state.to_string(),
        };
        let signed_claim = self.sign(set_state_message)?;
        let response: String = self
            .rpc_client
            .request("vsl_setAccountState", rpc_params![signed_claim])
            .await?;
        self.inc_nonce();
        Ok(B256::from_str(&response)?)
    }

    /// Retrieves a settled claim by its unique claim ID.
    ///
    /// - Input: a claim ID, which is the Keccak256 hash of the claim creator, creation nonce, and claim string.
    /// - Returns: the timestamped and signed [SettledVerifiedClaim] claim corresponding to the given claim ID.
    ///
    /// Will fail if:
    ///
    /// - claim is not found among the settled claims
    pub async fn get_settled_claim_by_id(
        &self,
        // the Keccak256 hash of the claim creator, creation nonce, and claim string.
        claim_id: &B256,
    ) -> RpcWrapperResult<Timestamped<Signed<SettledVerifiedClaim>>> {
        get_settled_claim_by_id(&self.rpc_client, claim_id).await
    }

    /// Retrieves the native token balance of the wrapped account.
    pub async fn get_balance(&self) -> RpcWrapperResult<Amount> {
        get_balance(self.rpc_client(), self.address()).await
    }

    /// Retrieves the balance of a specific asset held by the wrapped account.
    ///
    /// - Input: the asset ID to query.
    /// - Returns: the asset balance
    pub async fn get_asset_balance(&self, asset_id: &str) -> RpcWrapperResult<Amount> {
        get_asset_balance(self.rpc_client(), self.address(), asset_id).await
    }

    /// Retrieves the balances of all assets held by the wrapped account.
    ///
    /// - Returns: a map of asset IDs to balances
    pub async fn get_asset_balances(&self) -> RpcWrapperResult<HashMap<String, Amount>> {
        get_asset_balances(self.rpc_client(), self.address()).await
    }

    /// Retrieves creation metadata for a given asset by its ID.
    ///
    /// - Input: the asset ID to query.
    /// - Returns: An [AssetData] containing information about the asset,
    ///   or `None` if no asset with that id was created.
    pub async fn get_asset_by_id(&self, asset_id: &str) -> RpcWrapperResult<Option<AssetData>> {
        get_asset_by_id(self.rpc_client(), asset_id).await
    }

    /// Returns the wrapped account's current state, or `None` if unset.
    /// The state is a 256-bit hash
    pub async fn get_account_state(&self) -> RpcWrapperResult<Option<AccountStateHash>> {
        get_account_state(self.rpc_client(), self.address()).await
    }

    /// Yields (recent) settled claims which were originally submitted for verification by the wrapped account.
    ///
    /// - Input: a [Timestamp] (`since`)
    /// - Returns: the list of timestamped and signed [SettledVerifiedClaim]s recorded since the given timestamp.
    pub async fn list_settled_claims_for_receiver(
        &self,
        since: &Timestamp,
    ) -> RpcWrapperResult<Vec<Timestamped<Signed<SettledVerifiedClaim>>>> {
        list_settled_claims_for_receiver(self.rpc_client(), Some(self.address()), since).await
    }

    /// Yields (recent) claim verification requests listing the wrapped account as a verifier.
    ///
    /// - Input: a [Timestamp] (`since`)
    /// - Returns: the list of timestamped and signed [SubmittedClaim]s recorded since the given timestamp.
    pub async fn list_submitted_claims_for_receiver(
        &self,
        since: &Timestamp,
    ) -> RpcWrapperResult<Vec<Timestamped<Signed<SubmittedClaim>>>> {
        list_submitted_claims_for_receiver(&self.rpc_client, &self.address, since).await
    }

    /// Yields (recent) claims settled by the wrapped account as a verifier.
    ///
    /// - Input: a [Timestamp] (`since`).
    /// - Returns: the list of timestamped and signed [SettledVerifiedClaim]s recorded since the given timestamp.
    pub async fn list_settled_claims_for_sender(
        &self,
        since: &Timestamp,
    ) -> RpcWrapperResult<Vec<Timestamped<Signed<SettledVerifiedClaim>>>> {
        list_settled_claims_for_sender(&self.rpc_client, &self.address, since).await
    }

    /// Yields (recent) claim verification requests from the wrapped account.
    ///
    /// - Input: a [Timestamp] (`since`)
    /// - Returns: the list of timestamped and signed [SubmittedClaim]s recorded since the given timestamp.
    pub async fn list_submitted_claims_for_sender(
        &self,
        since: &Timestamp,
    ) -> RpcWrapperResult<Vec<Timestamped<Signed<SubmittedClaim>>>> {
        list_submitted_claims_for_sender(&self.rpc_client, Some(&self.address), since).await
    }
}

/// Retrieves a settled claim by its unique claim ID.
///
/// - Input: a claim ID, which is the Keccak256 hash of the claim creator, creation nonce, and claim string.
/// - Returns: the timestamped and signed [SettledVerifiedClaim] claim corresponding to the given claim ID.
///
/// Will fail if:
///
/// - claim is not found among the settled claims
pub async fn get_settled_claim_by_id<T: ClientT>(
    rpc_client: &T,
    // the Keccak256 hash of the claim creator, creation nonce, and claim string.
    claim_id: &B256,
) -> RpcWrapperResult<Timestamped<Signed<SettledVerifiedClaim>>> {
    let response = rpc_client
        .request("vsl_getSettledClaimById", rpc_params![claim_id.to_string()])
        .await?;
    Ok(response)
}

/// Yields (recent) settled claims for a receiver.
///
/// - Input: the address for which settled claims are tracked (use `None` for all claims).
/// - Input: a [Timestamp] (`since`)
/// - Returns: the list of timestamped and signed [SettledVerifiedClaim]s recorded since the given timestamp.
pub async fn list_settled_claims_for_receiver<T: ClientT>(
    rpc_client: &T,
    address: Option<&Address>,
    since: &Timestamp,
) -> RpcWrapperResult<Vec<Timestamped<Signed<SettledVerifiedClaim>>>> {
    let response = rpc_client
        .request(
            "vsl_listSettledClaimsForReceiver",
            rpc_params![address.map(|x| x.to_string()), since],
        )
        .await?;
    Ok(response)
}

/// Yields (recent) claim verification requests for a receiver.
///
/// - Input: the address for which claims requests are tracked.
/// - Input: a [Timestamp] (`since`)
/// - Returns: the list of timestamped and signed [SubmittedClaim]s recorded since the given timestamp.
pub async fn list_submitted_claims_for_receiver<T: ClientT>(
    rpc_client: &T,
    address: &Address,
    since: &Timestamp,
) -> RpcWrapperResult<Vec<Timestamped<Signed<SubmittedClaim>>>> {
    let response = rpc_client
        .request(
            "vsl_listSubmittedClaimsForReceiver",
            rpc_params![address.to_string(), since],
        )
        .await?;
    Ok(response)
}

/// Yields (recent) settled claims from an address.
///
/// - Input: the address that submitted the claims for settlement.
/// - Input: a [Timestamp] (`since`).
/// - Returns: the list of timestamped and signed [SettledVerifiedClaim]s recorded since the given timestamp.
pub async fn list_settled_claims_for_sender<T: ClientT>(
    rpc_client: &T,
    address: &Address,
    since: &Timestamp,
) -> RpcWrapperResult<Vec<Timestamped<Signed<SettledVerifiedClaim>>>> {
    let response = rpc_client
        .request(
            "vsl_listSettledClaimsForSender",
            rpc_params![address.to_string(), since],
        )
        .await?;
    Ok(response)
}

/// Yields (recent) claim verification requests from an address.
///
/// - Input: the address that submitted the claims for verification.
/// - Input: a [Timestamp] (`since`)
/// - Returns: the list of timestamped and signed [SubmittedClaim]s recorded since the given timestamp.
pub async fn list_submitted_claims_for_sender<T: ClientT>(
    rpc_client: &T,
    // the address that submitted the claims for verification.
    address: Option<&Address>,
    since: &Timestamp,
) -> RpcWrapperResult<Vec<Timestamped<Signed<SubmittedClaim>>>> {
    let response = rpc_client
        .request(
            "vsl_listSubmittedClaimsForSender",
            rpc_params![address.map(|x| x.to_string()), since],
        )
        .await?;
    Ok(response)
}

/// Retrieves the native token balance of a given account.
///
/// - Input: the account address to query.
/// - Returns: the balance
pub async fn get_balance<T: ClientT>(
    rpc_client: &T,
    address: &Address,
) -> RpcWrapperResult<Amount> {
    let response: String = rpc_client
        .request("vsl_getBalance", rpc_params![address.to_string()])
        .await?;
    Ok(Amount::from_hex_str(&response)?)
}

/// Retrieves the balance of a specific asset held by an account.
///
/// - Input: the account address.
/// - Input: the asset ID to query.
/// - Returns: the asset balance
pub async fn get_asset_balance<T: ClientT>(
    rpc_client: &T,
    account_id: &Address,
    asset_id: &str,
) -> RpcWrapperResult<Amount> {
    let Some(asset_data) = get_asset_by_id(rpc_client, asset_id).await? else {
        return Err(RpcWrapperError::NonExistentAsset);
    };
    let response: String = rpc_client
        .request(
            "vsl_getAssetBalance",
            rpc_params![account_id.to_string(), asset_id],
        )
        .await?;
    Ok(Amount::from_hex_str_with_decimals(&response, asset_data.decimals)?)
}

/// Retrieves the balances of all assets held by an account.
///
/// - Input: the account address to query.
/// - Returns: a map of asset IDs to balances
pub async fn get_asset_balances<T: ClientT>(
    rpc_client: &T,
    account_id: &Address,
) -> RpcWrapperResult<HashMap<String, Amount>> {
    let response: HashMap<String, String> = rpc_client
        .request("vsl_getAssetBalances", rpc_params![account_id.to_string()])
        .await?;
    let mut result = HashMap::with_capacity(response.len());
    for (asset_id, amount) in response {
        let Some(asset_data) = get_asset_by_id(rpc_client, &asset_id).await? else {
            return Err(RpcWrapperError::NonExistentAsset);
        };
        result.insert(asset_id, Amount::from_hex_str_with_decimals(&amount, asset_data.decimals)?);
    }
    Ok(result)
}

/// Retrieves creation metadata for a given asset by its ID.
///
/// - Input: the asset ID to query.
/// - Returns: An [AssetData] containing information about the asset,
///   or `None` if no asset with that id was created.
pub async fn get_asset_by_id<T: ClientT>(
    rpc_client: &T,
    asset_id: &str,
) -> RpcWrapperResult<Option<AssetData>> {
    let response: Option<CreateAssetMessage> = rpc_client
        .request("vsl_getAssetById", rpc_params![asset_id])
        .await?;
    let Some(response) = response else {
        return Ok(None);
    };
    Ok(Some(AssetData::try_from(response)?))
}

/// Returns the account's current state, or `None` if unset.
/// The state is a 256-bit hash
pub async fn get_account_state<T: ClientT>(
    rpc_client: &T,
    account_id: &Address,
) -> RpcWrapperResult<Option<AccountStateHash>> {
    let response: Option<String> = rpc_client
        .request("vsl_getAccountState", rpc_params![account_id.to_string()])
        .await?;
    let Some(response) = response else {
        return Ok(None);
    };
    Ok(Some(AccountStateHash::from_str(&response)?))
}

/// Returns the account's current nonce
///
/// - Input: the account address
pub async fn get_account_nonce<T: ClientT>(
    rpc_client: &T,
    account_id: &Address,
) -> RpcWrapperResult<u64> {
    let response = rpc_client
        .request("vsl_getAccountNonce", rpc_params![account_id.to_string()])
        .await?;
    Ok(response)
}

/// Checks if the server is up and ready.
///
/// - Returns: "ok" if the server is healthy.
pub async fn get_health<T: ClientT>(rpc_client: &T) -> RpcWrapperResult<()> {
    let response: String = rpc_client.request("vsl_getHealth", rpc_params!()).await?;
    assert_eq!("ok", response.to_lowercase());
    Ok(())
}

/// [Subscribe](https://geth.ethereum.org/docs/rpc/pubsub) to the claim verification requests for a receiver
///
/// - input: the address for which claim requests are tracked
/// - yields: a stream of timestamped signed [SubmittedClaim]s for the given address
pub async fn subscribe_to_submitted_claims_for_receiver(
    ws_client: &WsClient,
    address: &Address,
) -> RpcWrapperResult<Subscription<Timestamped<Signed<SubmittedClaim>>>> {
    Ok(ws_client
        .subscribe(
            "vsl_subscribeToSubmittedClaimsForReceiver",
            rpc_params![address.to_string()],
            "vsl_unsubscribeFromSubmittedClaimsForReceiver",
        )
        .await?)
}

/// [Subscribe](https://geth.ethereum.org/docs/rpc/pubsub) to the settled claims for a receiver
///
/// - input: the address for which settled claims are tracked (use `None` for all claims)
/// - yields: a stream of timestamped signed [SettledVerifiedClaim]s for the given address
pub async fn subscribe_to_settled_claims_for_receiver(
    ws_client: &WsClient,
    address: Option<&Address>,
) -> RpcWrapperResult<Subscription<Timestamped<Signed<SettledVerifiedClaim>>>> {
    Ok(ws_client
        .subscribe(
            "vsl_subscribeToSettledClaimsForReceiver",
            rpc_params![address.map(|x| x.to_string())],
            "vsl_unsubscribeFromSettledClaimsForReceiver",
        )
        .await?)
}
