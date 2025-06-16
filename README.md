# VSL-Core documentation and Rust SDK

## Documentation

Contained in the `docs` directory, the documentation provides:

- a documentation for the VSL RPC server
  - description of all endpoints in [`docs/api/rpc.md`](docs/api/rpc.md)
    - this is automatically generated from the rust api [`vsl-sdk/src/rpc_service.rs`](vsl-sdk/src/rpc_service.rs)
  - JSON schemas for the structures used by the RPC endpoints in [`docs/api/*.json*`](docs/api)
    - also are automatically generated from the rust api [`vsl-sdk/src/rpc_messages.rs`](vsl-sdk/src/rpc_messages.rs)
- A document describing the [Expected interaction with VSL](docs/flow.md)
- A document describing the [signing process for RPC messages](docs/signing.md)
- A document describing the [fee schedule](docs/fee-schedule.md)

## The VSL SDK for Rust

If one is interested in developing applications for VSL using Rust, the `vsl-sdk`
is a library providing

- definitions for the structures used by the RPC server messages
  and infrastructure to easily sign them and/or verify their signatures
  ([`vsl-sdk/src/rpc_messages.rs`](vsl-sdk/src/rpc_messages.rs))

- the `jsonrpsee` definition of the VSL RPC server ([`vsl-sdk/src/rpc_service.rs`](vsl-sdk/src/rpc_service.rs))

- A higher-level interface for interacting with the RPC server
   ([`vsl-sdk/src/rpc_wrapper.rs`](vsl-sdk/src/rpc_wrapper.rs)) which
  - Acts as wrapper to RPC calls on behalf of an owned account
    - builds and signs messages
    - encodes and decodes aguments to/from strings

- [An example Faucet application](vsl-sdk/examples/faucet/README.md) developed using the vsl-sdk
