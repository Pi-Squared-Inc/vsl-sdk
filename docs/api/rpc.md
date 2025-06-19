# JSON-RPC API Documentation

## List of Endpoints

- [`vsl_submitClaim`](#vsl_submitclaim)
- [`vsl_settleClaim`](#vsl_settleclaim)
- [`vsl_listSettledClaimsForReceiver`](#vsl_listsettledclaimsforreceiver)
- [`vsl_listSubmittedClaimsForReceiver`](#vsl_listsubmittedclaimsforreceiver)
- [`vsl_listSettledClaimsForSender`](#vsl_listsettledclaimsforsender)
- [`vsl_listSubmittedClaimsForSender`](#vsl_listsubmittedclaimsforsender)
- [`vsl_getSettledClaimById`](#vsl_getsettledclaimbyid)
- [`vsl_pay`](#vsl_pay)
- [`vsl_getAccount`](#vsl_getaccount)
- [`vsl_getBalance`](#vsl_getbalance)
- [`vsl_getAssetBalance`](#vsl_getassetbalance)
- [`vsl_getAssetBalances`](#vsl_getassetbalances)
- [`vsl_createAsset`](#vsl_createasset)
- [`vsl_transferAsset`](#vsl_transferasset)
- [`vsl_getAssetById`](#vsl_getassetbyid)
- [`vsl_getAccountState`](#vsl_getaccountstate)
- [`vsl_setAccountState`](#vsl_setaccountstate)
- [`vsl_getAccountNonce`](#vsl_getaccountnonce)
- [`vsl_getHealth`](#vsl_gethealth)
- [`vsl_subscribeToSettledClaimsForReceiver`](#vsl_subscribetosettledclaimsforreceiver)
- [`vsl_subscribeToSubmittedClaimsForReceiver`](#vsl_subscribetosubmittedclaimsforreceiver)

---

## `vsl_submitClaim`

Submits a request-for-verification claim.

- Records and indexes the signed [SubmittedClaim](#submittedclaim). Does not settle any claim.
- The `fee` amount will be transfered from the claim sender to each of the verifiers in the `to` field.
- The validation fee will also be deducted from the sender's account.

- Input: a signed [SubmittedClaim](#submittedclaim) containing the claim to be verified. The message is signed by the sender (which must match the `from` address).
- Returns: the claim ID of the submitted claim as a `String`.

Will fail if:

- Signature invalid or signer address does not match the `from` field
- `fee` / `from` / `nonce` / `to` not valid
- `quorum` is larger then the length of `to`
- sender balance cannot cover validation + verification fees

**Parameters**:

- `claim`: Signed< [SubmittedClaim](#submittedclaim) >

**Returns**:

String

---

## `vsl_settleClaim`

Submits a verified claim for settlement.

- The validation fee will be deducted from the sender's account.
- Claim will be in a pending state until quorum is reached
- When/If quorum is reached, the claim will be recorded as settled
  together with the addresses of all verifiers which have requested settlement so far

- Input: a signed [SettleClaimMessage](#settleclaimmessage) signed by the sender.
- Returns: the claim ID of the claim to be settled as a `String`.

Will fail if:

- Signature invalid or signer address does not match the `from` field
- `from` / `nonce` not valid
- sender balance cannot cover validation fee
- Claim
  - was already settled
  - was not found among submitted claims
  - has expired
- Verifier
  - not among those specified in submitted claim
  - has already verified the claim

**Parameters**:

- `settled_claim`: Signed< [SettleClaimMessage](#settleclaimmessage) >

**Returns**:

String

---

## `vsl_listSettledClaimsForReceiver`

Yields (recent) settled claims for a receiver.

- Input: the (Ethereum-style) address for which settled claims are tracked (optional).
- Input: a [Timestamp](#timestamp) (`since`)
- Returns: the list of most recent timestamped and signed [SettledVerifiedClaim](#settledverifiedclaim)s recorded since the given timestamp (limited at 64 entries).

Will fail if:

- `address` not valid

**Parameters**:

- `address`: Option< String >
- `since`: [Timestamp](#timestamp)

**Returns**:

Vec< [Timestamped](#timestamped)< Signed< [SettledVerifiedClaim](#settledverifiedclaim) > > >

---

## `vsl_listSubmittedClaimsForReceiver`

Yields (recent) claim verification requests for a receiver.

- Input: the (Ethereum-style) address for which claims requests are tracked.
- Input: a [Timestamp](#timestamp) (`since`)
- Returns: the list of most recent timestamped and signed [SubmittedClaim](#submittedclaim)s recorded since the given timestamp (limited at 64 entries).

Will fail if:

- `address` not valid

**Parameters**:

- `address`: String
- `since`: [Timestamp](#timestamp)

**Returns**:

Vec< [Timestamped](#timestamped)< Signed< [SubmittedClaim](#submittedclaim) > > >

---

## `vsl_listSettledClaimsForSender`

Yields (recent) settled claims from an address.

- Input: the (Ethereum-style) address that submitted the claims for settlement.
- Input: a [Timestamp](#timestamp) (`since`).
- Returns: the list of most recent timestamped and signed [SettledVerifiedClaim](#settledverifiedclaim)s recorded since the given timestamp (limited at 64 entries).

Will fail if:

- `address` not valid

**Parameters**:

- `address`: String
- `since`: [Timestamp](#timestamp)

**Returns**:

Vec< [Timestamped](#timestamped)< Signed< [SettledVerifiedClaim](#settledverifiedclaim) > > >

---

## `vsl_listSubmittedClaimsForSender`

Yields (recent) claim verification requests from an address.

- Input: the (Ethereum-style) address that submitted the claims for verification.
- Input: a [Timestamp](#timestamp) (`since`)
- Returns: the list of most recent timestamped and signed [SubmittedClaim](#submittedclaim)s recorded since the given timestamp (limited at 64 entries).

Will fail if:

- `address` not valid

**Parameters**:

- `address`: Option< String >
- `since`: [Timestamp](#timestamp)

**Returns**:

Vec< [Timestamped](#timestamped)< Signed< [SubmittedClaim](#submittedclaim) > > >

---

## `vsl_getSettledClaimById`

Retrieves a settled claim by its unique claim ID.

- Input: a claim ID, which is the Keccak256 hash of the claim creator, creation nonce, and claim string.
- Returns: the timestamped and signed [SettledVerifiedClaim](#settledverifiedclaim) claim corresponding to the given claim ID.

Will fail if:

- claim is not found among the settled claims

**Parameters**:

- `claim_id`: String

**Returns**:

[Timestamped](#timestamped)< Signed< [SettledVerifiedClaim](#settledverifiedclaim) > >

---

## `vsl_pay`

Submits a VSL payment from one account to another.

- The `amount` will be transfered from `from` to `to`.
- The validation fee will also be deducted from the sender's account.
- A [SettledVerifiedClaim](#settledverifiedclaim) will be recorded containing the json-serialized [PayMessage](#paymessage) as its claim

- Input: a signed [PayMessage](#paymessage) specifying the payment parameters, such as the sender, receiver, and the amount. The message is singed by the sender (which must match the `from` address).
- Returns: the settled payment claim ID as a `String`.

Will fail if:

- Signature invalid or signer address does not match the `from` field
- `from` / `to` / `nonce` / `amount` not valid
- sender balance cannot cover the specified `amount` and the validation fee

**Parameters**:

- `payment`: Signed< [PayMessage](#paymessage) >

**Returns**:

String

---

## `vsl_getAccount`

Retrieves information about a specific account.

Currently not implemented

- Input: the (Ethereum-style) address of the account to query.
- Returns: a JSON string representing the account's metadata.

**Parameters**:

- `account_id`: String

**Returns**:

String

---

## `vsl_getBalance`

Retrieves the native token balance of a given account.

- Input: the (Ethereum-style) address of the account to query.
- Returns: the balance as a base-10 string of the u128 value

Will fail if:

- `account_id` not valid

**Parameters**:

- `account_id`: String

**Returns**:

String

---

## `vsl_getAssetBalance`

Retrieves the balance of a specific asset held by an account.

- Input: the (Ethereum-style) address of the account to query.
- Input: the asset ID(hex-encoded 256 bit) to query.
- Returns: the asset balance as a base-10 string of the u128 value

Will fail if:

- `account_id` / `asset_id` not valid

**Parameters**:

- `account_id`: String
- `asset_id`: String

**Returns**:

String

---

## `vsl_getAssetBalances`

Retrieves the balances of all assets held by an account.

- Input: the (Ethereum-style) address of the account to query.
- Returns: a map of asset IDs to balances as a base-10 string of the u128 value

Will fail if:

- `account_id` not valid

**Parameters**:

- `account_id`: String

**Returns**:

std::collections::HashMap< String,String >

---

## `vsl_createAsset`

Creates a new asset on the network.

- A [SettledVerifiedClaim](#settledverifiedclaim) will be recorded containing the json-serialized [CreateAssetMessage](#createassetmessage) as its claim

- Input: a signed [CreateAssetMessage](#createassetmessage) defining the asset properties.
- Returns: A [CreateAssetResult](#createassetresult) containing the asset ID of the newly created asset and the settled create asset claim ID

Will fail if:

- Signature invalid or signer address does not match the `account_id` field
- `account_id` / `total_supply` / `nonce` not valid
- `decimals` is larger than `18`
- sender balance cannot cover validation fee

**Parameters**:

- `asset_data`: Signed< [CreateAssetMessage](#createassetmessage) >

**Returns**:

[CreateAssetResult](#createassetresult)

---

## `vsl_transferAsset`

Transfers a specific asset from one account to another.

- A [SettledVerifiedClaim](#settledverifiedclaim) will be recorded containing the json-serialized [TransferAssetMessage](#transferassetmessage) as its claim

- Input: a signed [TransferAssetMessage](#transferassetmessage) with the transfer details.
- Returns: the settled asset transfer claim ID as a `String`

Will fail if:

- Signature invalid or signer address does not match the `from` field
- `from` / `to` / `nonce` / `amount` / `asset_id` not valid
- sender balance cannot cover validation fee
- sender asset balance cannot cover `amount`

**Parameters**:

- `transfer_asset`: Signed< [TransferAssetMessage](#transferassetmessage) >

**Returns**:

String

---

## `vsl_getAssetById`

Retrieves creation metadata for a given asset by its ID.

- Input: the asset ID to query.
- Returns: The original [CreateAssetMessage](#createassetmessage) used for setting up this asset
  or `None` if no asset with that id was created.

Will fail if:
- `asset_id` not valid

**Parameters**:

- `asset_id`: String

**Returns**:

Option< [CreateAssetMessage](#createassetmessage) >

---

## `vsl_getAccountState`

Returns the account's current state, or null if unset.
The state is a 256-bit hash formatted as a hex string starting 0x

- Input: the (Ethereum-style) address of the account to query.

Will fail if:

- `account_id` not valid

**Parameters**:

- `account_id`: String

**Returns**:

Option< String >

---

## `vsl_setAccountState`

Sets the account's current state.
The state is a 256-bit hash formatted as a hex string starting with 0x

- Input: a signed [SetStateMessage](#setstatemessage)
- Returns: a settled claim ID for the set state transaction.

Will fail if:

- Signature invalid or signer address does not match the `from` field
- `from` / `nonce` / `state` not valid
- sender balance cannot cover validation fee

**Parameters**:

- `state`: Signed< [SetStateMessage](#setstatemessage) >

**Returns**:

String

---

## `vsl_getAccountNonce`

Returns the account's current nonce

- Input: the (Ethereum-style) address of the account to query.

Will fail if:

- `account_id` not valid

**Parameters**:

- `account_id`: String

**Returns**:

u64

---

## `vsl_getHealth`

Checks if the server is up and ready.

- Returns: "ok" if the server is healthy.

**Returns**:

String

---

## `vsl_subscribeToSettledClaimsForReceiver`

[Subscription](https://geth.ethereum.org/docs/rpc/pubsub) to the settled claims for a receiver

- Input: the (Ethereum-style) address of the account for which settled claims are tracked (optional)
- yields: a stream of timestamped signed [SettledVerifiedClaim](#settledverifiedclaim)s for the given account (or for all accounts)

**Parameters**:

- `address`: Option< String >

**Returns**:

SubscriptionResult

---

## `vsl_subscribeToSubmittedClaimsForReceiver`

[Subscription](https://geth.ethereum.org/docs/rpc/pubsub) to the claim verification requests for a receiver

- Input: the (Ethereum-style) address of the account for which settled claims are tracked
- yields: a stream of timestamped signed [SubmittedClaim](#submittedclaim)s for the given address

**Parameters**:

- `address`: String

**Returns**:

SubscriptionResult

---

## SubmittedClaim

An (unsigned) vls_submitClaim request for claim-verification

**JSON Schema**: [SubmittedClaim](SubmittedClaim.json)

### Fields:

- **claim** (string): the claim to be verified (VSL does not care about how its encoded)

- **claim_type** (string): the claim type (could be any string)

- **expires** ([Timestamp](timestamp))

- **fee** (string): the fee for verification (u128 formatted as hex string).

- **from** (string)

- **nonce** (string): the client nonce (64 bit unsigned integer)

- **proof** (string): the proof of the claim (VSL does not care about how its encoded)

- **quorum** (integer)

- **to** (array< string >): the list of (Ethereum-style) addresses of accounts which can verify this claim

## VerifiedClaim

Representation of a verified claim

**JSON Schema**: [VerifiedClaim](VerifiedClaim.json)

### Fields:

- **claim** (string): the original claim which was verified and now settled

- **claim_id** (string): the id (hex-encoded 256 bit hash) of the claim (useful for retrieving the full data of the claim)

- **claim_owner** (string): the (Ethereum-style) address of the client which produced this claim

- **claim_type** (string): the claim type

## SettleClaimMessage

An (unsigned) vls_settleClaim request made by a verifier having verified the claim

**JSON Schema**: [SettleClaimMessage](SettleClaimMessage.json)

### Fields:

- **from** (string): The (Ethereum-style) address of the verifier requesting claim settlement

- **nonce** (string): The nonce (64 bit unsigned integer) of the verifier requesting claim settlement

- **target_claim_id** (string): The id (hex-encoded 256 bit hash) of the claim for which claim settlement is requested

## SettledVerifiedClaim

A settled (verified) claim

**JSON Schema**: [SettledVerifiedClaim](SettledVerifiedClaim.json)

### Fields:

- **verified_claim** ([VerifiedClaim](verifiedclaim)): the claim which was verified

- **verifiers** (array< string >): the (Ethereum-style) addresses of the verifiers which have verified the claim and are part of the quorum

## PayMessage

An (unsigned) vsl_pay request (in VSL tokens)

**JSON Schema**: [PayMessage](PayMessage.json)

### Fields:

- **amount** (string): The amount to be transfered (u128 formatted as hex string)

- **from** (string): The (Ethereum-style) address of the account requesting the transfer

- **nonce** (string): The nonce (64 bit unsigned integer) of the account creating the asset

- **to** (string): The (Ethereum-style) address of the account receiving the payment

## CreateAssetMessage

An (unsigned) vsl_createAsset request

**JSON Schema**: [CreateAssetMessage](CreateAssetMessage.json)

### Fields:

- **account_id** (string): The (Ethereum-style) address of the account creating the asset

- **decimals** (integer): Number of decimals

- **nonce** (string): The nonce (64 bit unsigned integer) of the account creating the asset

- **ticker_symbol** (string): Ticker symbol to be used for the new asset

- **total_supply** (string): The amount to initialize the new asset with (u128 formatted as hex string)

## CreateAssetResult

The return object of a `vsl_createAsset` request

**JSON Schema**: [CreateAssetResult](CreateAssetResult.json)

### Fields:

- **asset_id** (string): The ID (hex-encoded 256 bit hash) of the asset

- **claim_id** (string): Settled claim ID for the create asset command  (hex-encoded 256 bit hash)

## TransferAssetMessage

An (unsigned) vsl_transferAsset request

**JSON Schema**: [TransferAssetMessage](TransferAssetMessage.json)

### Fields:

- **amount** (string): The amount (of asset) to be transfered (u128 formatted as hex string)

- **asset_id** (string): The id (hex-encoded 256 bit hash) of the asset (returned when asset was created)

- **from** (string): The (Ethereum-style) address of the account transfering the asset

- **nonce** (string): The nonce (64 bit unsigned integer) of the account transfering the asset

- **to** (string): The (Ethereum-style) address of the account receiving the asset

## SetStateMessage

An (unsigned) vsl_setState request

**JSON Schema**: [SetStateMessage](SetStateMessage.json)

### Fields:

- **from** (string): The (Ethereum-style) address of the account requesting its state to be changed

- **nonce** (string): The nonce (64 bit unsigned integer) of the account requesting its state to be changed

- **state** (string): The new state (hex-encoded 256 bit hash)

## Timestamp

Records the time elapsed from the [UNIX_EPOCH]

**JSON Schema**: [Timestamp](Timestamp.json)

### Fields:

- **nanos** (integer): the _remaining fraction_ of a second, expressed in nano-seconds

- **seconds** (integer): Time elapsed from the [UNIX_EPOCH] (in seconds, truncated)
