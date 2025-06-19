# VSL-Core documentation and Rust SDK

VSL (Verifiable Settlement Layer) is a decentralized protocol for validating and settling verifiable claims. This repository includes:

- Documentation for understanding and interacting with the system
- A Rust SDK for developers

## Running an Instance of VSL Locally

Follow the instructions given [here](docs/quick_start.md) to run local instances of VSL and the explorer.

## Documentation

The `docs` directory contains detailed information about the VSL system, including:

- Documentation of the VSL RPC server
  - [RPC API Reference](docs/api/rpc.md): Description of all RPC endpoints.
    - Automatically generated from the rust api [`vsl-sdk/src/rpc_service.rs`](vsl-sdk/src/rpc_service.rs)
  - [JSON Schemas](docs/api): Schemas for the structures used by the RPC endpoints in [`docs/api/*.json*`](docs/api)
    - Automatically generated from the rust api [`vsl-sdk/src/rpc_messages.rs`](vsl-sdk/src/rpc_messages.rs)
- [Expected interaction with VSL](docs/flow.md): Describes expected interactions with VSL.
- [Message Signing Guide](docs/signing.md): Describes the signing process for RPC messages.
- [Fee Schedule](docs/fee-schedule.md): Lists fees associated with different operations.

## VSL SDK for Rust

If you're interested in developing applications for VSL using Rust, the `vsl-sdk`
is a library providing

- Definitions for the structures used by the RPC server messages
  and infrastructure to easily sign them and/or verify their signatures
  ([`vsl-sdk/src/rpc_messages.rs`](vsl-sdk/src/rpc_messages.rs))

- The `jsonrpsee` definition of the VSL RPC server ([`vsl-sdk/src/rpc_service.rs`](vsl-sdk/src/rpc_service.rs))

- A higher-level interface for interacting with the RPC server
   ([`vsl-sdk/src/rpc_wrapper.rs`](vsl-sdk/src/rpc_wrapper.rs)) which
  - Acts as wrapper to RPC calls on behalf of an owned account
    - Builds and signs messages
    - Encodes and decodes aguments to/from strings

- [An example Faucet application](vsl-sdk/examples/faucet/README.md) developed using the vsl-sdk
