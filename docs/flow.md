# Expected interaction with VSL

## The client submits the claim to be verified

- using the [`vsl_submitClaim`](rpc.md#vsl-submitclaim) endpoint
  - Creates a [`SubmittedClaim`](SubmittedClaim.json) message
  - signs the message with its private key

## VSL handles the submitted claim

Upon receiving a `vsl_submitClaim` request:

- 1 (atto-)token is transfered from the sender to the VSL validator as processing fee
- the `fee` is transfered from the sender to the verifiers
- the `SubmittedClaim` signed message is timestamped and deposited as-is in the VSL DB.
- the message is also stored into several indices for faster retrieval:
  - by the sender
  - by each of the intended receivers (taken from the verifiers field addresses)

## Verifiers watch the submitted claims DB for new submissions

- using the [`vsl_listSubmittedClaimsForReceiver`](rpc.md#vsl_listsubmittedclaimsforreceiver) endpoint
  - using their own address as the receiver

## Verifiers verify a claim and make a request for settlement

- using the [`vsl_settleClaim`](rpc.md#vsl-settleclaim) endpoint
  - prepare a [`VerifiedClaim`](VerifiedClaim.json) message
  - signs the message with its private key

## VSL settles verified claims

Upon receiving a `vsl_settleClaim` request:

- 1 (atto-)token is transfered from the sender to the VSL validator as processing fee
- the message is split into its signature and the contained `VerifiedClaim`
- VSL checks that:
  - there exist a submitted claim corresponding to this verified claim
  - that claim is not expired
  - that claim specifies the sender of the verified claim message as a verifier
  - if any of the above fails, the request is denied with a specific error
- the validator creates a [`SettledVerifiedClaim`](SettledVerifiedClaim.json) containing
  - the `VerifiedClaim` object
  - a vector containing the string representation of the signature
- the thus obtained message is signed with the validator's private key and stored in the VSL DB
  - similarly indexed by the sender and the originator of the claim as the intended receiver

## Clients watch for settled claims and process them

- using the [`vsl_listSettledClaimsForReceiver`](rpc.md#vsl_listsettledclaimsforreceiver) endpoint
  - they can check that the message is signed by the validator
  - they can also double check that the stored signature corresponds to the intended verifier.
