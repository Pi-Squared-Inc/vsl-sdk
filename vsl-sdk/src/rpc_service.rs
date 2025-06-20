use alloy::consensus::Signed;
use jsonrpsee::core::{RpcResult, SubscriptionResult};
use jsonrpsee::proc_macros::rpc;

use crate::Timestamp;
use crate::rpc_messages::{
    CreateAssetMessage, CreateAssetResult, PayMessage, SetStateMessage, SettleClaimMessage, SettledClaimData, SettledVerifiedClaim, SubmittedClaim, SubmittedClaimData, Timestamped, TransferAssetMessage
};

#[rpc(server, client)]
pub trait ClaimRpc {
    /// Submits a request-for-verification claim.
    ///
    /// - Records and indexes the signed [SubmittedClaim]. Does not settle any claim.
    /// - The `fee` amount will be transfered from the claim sender to each of the verifiers in the `to` field.
    /// - The validation fee will also be deducted from the sender's account.
    ///
    /// - Input: a signed [SubmittedClaim] containing the claim to be verified. The message is signed by the sender (which must match the `from` address).
    /// - Returns: the claim ID of the submitted claim as a `String`.
    ///
    /// Will fail if:
    ///
    /// - Signature invalid or signer address does not match the `from` field
    /// - `fee` / `from` / `nonce` / `to` not valid
    /// - `quorum` is larger then the length of `to`
    /// - sender balance cannot cover validation + verification fees
    #[method(name = "vsl_submitClaim", param_kind = map)]
    async fn submit_claim(&self, claim: Signed<SubmittedClaim>) -> RpcResult<String>;

    /// Submits a verified claim for settlement.
    ///
    /// - The validation fee will be deducted from the sender's account.
    /// - Claim will be in a pending state until quorum is reached
    /// - When/If quorum is reached, the claim will be recorded as settled
    ///   together with the addresses of all verifiers which have requested settlement so far
    ///
    /// - Input: a signed [SettleClaimMessage] signed by the sender.
    /// - Returns: the claim ID of the claim to be settled as a `String`.
    ///
    /// Will fail if:
    ///
    /// - Signature invalid or signer address does not match the `from` field
    /// - `from` / `nonce` not valid
    /// - sender balance cannot cover validation fee
    /// - Claim
    ///   - was already settled
    ///   - was not found among submitted claims
    ///   - has expired
    /// - Verifier
    ///   - not among those specified in submitted claim
    ///   - has already verified the claim
    #[method(name = "vsl_settleClaim", param_kind = map)]
    async fn settle_claim(&self, settled_claim: Signed<SettleClaimMessage>) -> RpcResult<String>;

    /// Yields (recent) settled claims metadata
    ///
    /// - Input: a [Timestamp] (`since`)
    /// - Returns: a list containing metadata for the most recent settled claims recorded since the given timestamp (limited at 64 entries).
    #[method(name = "vsl_listSettledClaimsMetadata", param_kind = map)]
    async fn list_settled_claims_metadata(
        &self,
        since: Timestamp,
    ) -> RpcResult<Vec<Timestamped<SettledClaimData>>>;

    /// Yields (recent) submitted claims metadata
    ///
    /// - Input: a [Timestamp] (`since`)
    /// - Returns: a list containing metadata for the most recent submitted claims recorded since the given timestamp (limited at 64 entries).
    #[method(name = "vsl_listSubmittedClaimsMetadata", param_kind = map)]
    async fn list_submitted_claims_metadata(
        &self,
        since: Timestamp,
    ) -> RpcResult<Vec<Timestamped<SubmittedClaimData>>>;



    /// Yields (recent) settled claims for a receiver.
    ///
    /// - Input: the (Ethereum-style) address for which settled claims are tracked (optional).
    /// - Input: a [Timestamp] (`since`)
    /// - Returns: the list of most recent timestamped and signed [SettledVerifiedClaim]s recorded since the given timestamp (limited at 64 entries).
    ///
    /// Will fail if:
    ///
    /// - `address` not valid
    #[method(name = "vsl_listSettledClaimsForReceiver", param_kind = map)]
    async fn list_settled_claims_for_receiver(
        &self,
        address: Option<String>,
        since: Timestamp,
    ) -> RpcResult<Vec<Timestamped<Signed<SettledVerifiedClaim>>>>;

    /// Yields (recent) claim verification requests for a receiver.
    ///
    /// - Input: the (Ethereum-style) address for which claims requests are tracked.
    /// - Input: a [Timestamp] (`since`)
    /// - Returns: the list of most recent timestamped and signed [SubmittedClaim]s recorded since the given timestamp (limited at 64 entries).
    ///
    /// Will fail if:
    ///
    /// - `address` not valid
    #[method(name = "vsl_listSubmittedClaimsForReceiver", param_kind = map)]
    async fn list_submitted_claims_for_receiver(
        &self,
        address: String,
        since: Timestamp,
    ) -> RpcResult<Vec<Timestamped<Signed<SubmittedClaim>>>>;

    /// Yields (recent) settled claims from an address.
    ///
    /// - Input: the (Ethereum-style) address that submitted the claims for settlement.
    /// - Input: a [Timestamp] (`since`).
    /// - Returns: the list of most recent timestamped and signed [SettledVerifiedClaim]s recorded since the given timestamp (limited at 64 entries).
    ///
    /// Will fail if:
    ///
    /// - `address` not valid
    #[method(name = "vsl_listSettledClaimsForSender", param_kind = map)]
    async fn list_settled_claims_for_sender(
        &self,
        address: String,
        since: Timestamp,
    ) -> RpcResult<Vec<Timestamped<Signed<SettledVerifiedClaim>>>>;

    /// Yields (recent) claim verification requests from an address.
    ///
    /// - Input: the (Ethereum-style) address that submitted the claims for verification.
    /// - Input: a [Timestamp] (`since`)
    /// - Returns: the list of most recent timestamped and signed [SubmittedClaim]s recorded since the given timestamp (limited at 64 entries).
    ///
    /// Will fail if:
    ///
    /// - `address` not valid
    #[method(name = "vsl_listSubmittedClaimsForSender", param_kind = map)]
    async fn list_submitted_claims_for_sender(
        &self,
        address: Option<String>,
        since: Timestamp,
    ) -> RpcResult<Vec<Timestamped<Signed<SubmittedClaim>>>>;

    /// Retrieves the claim data contained in the submitted claim with the given ID.
    ///
    /// - Input: a claim ID, which is the Keccak256 hash of the claim creator, creation nonce, and claim string.
    /// - Returns: the contents of the `claim` field from the corresponding [SubmittedClaim].
    ///
    /// Will fail if:
    ///
    /// - no claim with given ID is not found among the submitted claims
    #[method(name = "vsl_getClaimDataById", param_kind = map)]
    async fn get_claim_data_by_id(
        &self,
        claim_id: String,
    ) -> RpcResult<String>;

    /// Retrieves the proof contained in the submitted claim with the given ID.
    ///
    /// - Input: a claim ID, which is the Keccak256 hash of the claim creator, creation nonce, and claim string.
    /// - Returns: the contents of the `proof` field from the corresponding [SubmittedClaim].
    ///
    /// Will fail if:
    ///
    /// - no claim with given ID is not found among the submitted claims
    #[method(name = "vsl_getProofById", param_kind = map)]
    async fn get_proof_by_id(
        &self,
        claim_id: String,
    ) -> RpcResult<String>;

    /// Retrieves a settled claim by its unique claim ID.
    ///
    /// - Input: a claim ID, which is the Keccak256 hash of the claim creator, creation nonce, and claim string.
    /// - Returns: the timestamped and signed [SettledVerifiedClaim] claim corresponding to the given claim ID.
    ///
    /// Will fail if:
    ///
    /// - claim is not found among the settled claims
    #[method(name = "vsl_getSettledClaimById", param_kind = map)]
    async fn get_settled_claim_by_id(
        &self,
        claim_id: String,
    ) -> RpcResult<Timestamped<Signed<SettledVerifiedClaim>>>;

    /// Submits a VSL payment from one account to another.
    ///
    /// - The `amount` will be transfered from `from` to `to`.
    /// - The validation fee will also be deducted from the sender's account.
    /// - A [SettledVerifiedClaim] will be recorded containing the json-serialized [PayMessage] as its claim
    ///
    /// - Input: a signed [PayMessage] specifying the payment parameters, such as the sender, receiver, and the amount. The message is singed by the sender (which must match the `from` address).
    /// - Returns: the settled payment claim ID as a `String`.
    ///
    /// Will fail if:
    ///
    /// - Signature invalid or signer address does not match the `from` field
    /// - `from` / `to` / `nonce` / `amount` not valid
    /// - sender balance cannot cover the specified `amount` and the validation fee
    #[method(name = "vsl_pay", param_kind = map)]
    async fn pay(&self, payment: Signed<PayMessage>) -> RpcResult<String>;

    /// Retrieves information about a specific account.
    ///
    /// Currently not implemented
    ///
    /// - Input: the (Ethereum-style) address of the account to query.
    /// - Returns: a JSON string representing the account's metadata.
    #[method(name = "vsl_getAccount", param_kind = map)]
    async fn get_account(&self, account_id: String) -> RpcResult<String>;

    /// Retrieves the native token balance of a given account.
    ///
    /// - Input: the (Ethereum-style) address of the account to query.
    /// - Returns: the balance as a base-10 string of the u128 value
    ///
    /// Will fail if:
    ///
    /// - `account_id` not valid
    #[method(name = "vsl_getBalance", param_kind = map)]
    async fn get_balance(&self, account_id: String) -> RpcResult<String>;

    /// Retrieves the balance of a specific asset held by an account.
    ///
    /// - Input: the (Ethereum-style) address of the account to query.
    /// - Input: the asset ID(hex-encoded 256 bit) to query.
    /// - Returns: the asset balance as a base-10 string of the u128 value
    ///
    /// Will fail if:
    ///
    /// - `account_id` / `asset_id` not valid
    #[method(name = "vsl_getAssetBalance", param_kind = map)]
    async fn get_asset_balance(&self, account_id: String, asset_id: String) -> RpcResult<String>;

    /// Retrieves the balances of all assets held by an account.
    ///
    /// - Input: the (Ethereum-style) address of the account to query.
    /// - Returns: a map of asset IDs to balances as a base-10 string of the u128 value
    ///
    /// Will fail if:
    ///
    /// - `account_id` not valid
    #[method(name = "vsl_getAssetBalances", param_kind = map)]
    async fn get_asset_balances(
        &self,
        account_id: String,
    ) -> RpcResult<std::collections::HashMap<String, String>>;

    /// Creates a new asset on the network.
    ///
    /// - A [SettledVerifiedClaim] will be recorded containing the json-serialized [CreateAssetMessage] as its claim
    ///
    /// - Input: a signed [CreateAssetMessage] defining the asset properties.
    /// - Returns: A [CreateAssetResult] containing the asset ID of the newly created asset and the settled create asset claim ID
    ///
    /// Will fail if:
    ///
    /// - Signature invalid or signer address does not match the `account_id` field
    /// - `account_id` / `total_supply` / `nonce` not valid
    /// - `decimals` is larger than `18`
    /// - sender balance cannot cover validation fee
    #[method(name = "vsl_createAsset", param_kind = map)]
    async fn create_asset(
        &self,
        asset_data: Signed<CreateAssetMessage>,
    ) -> RpcResult<CreateAssetResult>;

    /// Transfers a specific asset from one account to another.
    ///
    /// - A [SettledVerifiedClaim] will be recorded containing the json-serialized [TransferAssetMessage] as its claim
    ///
    /// - Input: a signed [TransferAssetMessage] with the transfer details.
    /// - Returns: the settled asset transfer claim ID as a `String`
    ///
    /// Will fail if:
    ///
    /// - Signature invalid or signer address does not match the `from` field
    /// - `from` / `to` / `nonce` / `amount` / `asset_id` not valid
    /// - sender balance cannot cover validation fee
    /// - sender asset balance cannot cover `amount`
    #[method(name = "vsl_transferAsset", param_kind = map)]
    async fn transfer_asset(
        &self,
        transfer_asset: Signed<TransferAssetMessage>,
    ) -> RpcResult<String>;

    /// Retrieves creation metadata for a given asset by its ID.
    ///
    /// - Input: the asset ID to query.
    /// - Returns: The original [CreateAssetMessage] used for setting up this asset
    ///   or `None` if no asset with that id was created.
    ///
    /// Will fail if:
    /// - `asset_id` not valid
    #[method(name = "vsl_getAssetById", param_kind = map)]
    async fn get_asset_by_id(&self, asset_id: String) -> RpcResult<Option<CreateAssetMessage>>;

    /// Returns the account's current state, or null if unset.
    /// The state is a 256-bit hash formatted as a hex string starting 0x
    ///
    /// - Input: the (Ethereum-style) address of the account to query.
    ///
    /// Will fail if:
    ///
    /// - `account_id` not valid
    #[method(name = "vsl_getAccountState", param_kind = array)]
    async fn get_state(&self, account_id: String) -> RpcResult<Option<String>>;

    /// Sets the account's current state.
    /// The state is a 256-bit hash formatted as a hex string starting with 0x
    ///
    /// - Input: a signed [SetStateMessage]
    /// - Returns: a settled claim ID for the set state transaction.
    ///
    /// Will fail if:
    ///
    /// - Signature invalid or signer address does not match the `from` field
    /// - `from` / `nonce` / `state` not valid
    /// - sender balance cannot cover validation fee
    #[method(name = "vsl_setAccountState", param_kind = array)]
    async fn set_state(&self, state: Signed<SetStateMessage>) -> RpcResult<String>;

    /// Returns the account's current nonce
    ///
    /// - Input: the (Ethereum-style) address of the account to query.
    ///
    /// Will fail if:
    ///
    /// - `account_id` not valid
    #[method(name = "vsl_getAccountNonce", param_kind = array)]
    async fn get_nonce(&self, account_id: String) -> RpcResult<u64>;

    /// Checks if the server is up and ready.
    ///
    /// - Returns: "ok" if the server is healthy.
    #[method(name = "vsl_getHealth", param_kind = map)]
    async fn get_health(&self) -> RpcResult<String>;

    /// [Subscription](https://geth.ethereum.org/docs/rpc/pubsub) to the settled claims for a receiver
    ///
    /// - Input: the (Ethereum-style) address of the account for which settled claims are tracked (optional)
    /// - yields: a stream of timestamped signed [SettledVerifiedClaim]s for the given account (or for all accounts)
    #[subscription(
        name = "vsl_subscribeToSettledClaimsForReceiver",
        unsubscribe = "vsl_unsubscribeFromSettledClaimsForReceiver",
        param_kind = map,
        item = Timestamped<Signed<SettledVerifiedClaim>>)]
    async fn subscribe_to_settled_claims_for_receiver(
        &self,
        address: Option<String>,
    ) -> SubscriptionResult;

    /// [Subscription](https://geth.ethereum.org/docs/rpc/pubsub) to the claim verification requests for a receiver
    ///
    /// - Input: the (Ethereum-style) address of the account for which settled claims are tracked
    /// - yields: a stream of timestamped signed [SubmittedClaim]s for the given address
    #[subscription(
        name = "vsl_subscribeToSubmittedClaimsForReceiver",
        unsubscribe = "vsl_unsubscribeFromSubmittedClaimsForReceiver",
        param_kind = map,
        item = Timestamped<Signed<SubmittedClaim>>)]
    async fn subscribe_to_submitted_claims_for_receiver(
        &self,
        address: String,
    ) -> SubscriptionResult;
}
