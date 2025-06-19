# VSL-Core documentation and Rust SDK

## Documentation

Contained in the `docs` directory, the documentation provides:

- An instruction to run VSL locally in [`docs/quick_start.md`](docs/quick_start.md)
- A documentation for the VSL RPC server
  - Description of all endpoints in [`docs/api/rpc.md`](docs/api/rpc.md)
    - This is automatically generated from the rust api [`vsl-sdk/src/rpc_service.rs`](vsl-sdk/src/rpc_service.rs)
  - JSON schemas for the structures used by the RPC endpoints in [`docs/api/*.json*`](docs/api)
    - Automatically generated from the rust api [`vsl-sdk/src/rpc_messages.rs`](vsl-sdk/src/rpc_messages.rs)
- A document describing the [Expected interaction with VSL](docs/flow.md)
- A document describing the [signing process for RPC messages](docs/signing.md)
- A document describing the [fee schedule](docs/fee-schedule.md)

## The VSL SDK for Rust

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
