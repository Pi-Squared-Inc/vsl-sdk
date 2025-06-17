# Faucet example

## Overview

This app provides a means for clients to request and (pending verification) be granted funds from a faucet.

It is also built as an example of claim verification and settlement.

### Basic workflow

- A client submits a claim with the "claim" field being the amount requested and listing the [faucet verifier](faucet_verifier.rs) as (one of the) verifier(s)
  - If it's the first time requesting funds, it also sends as "proof" the ID of a settled claim representing a payment to itself from the "master account" (the account we use to initialize whitelisted accounts with funds).
- The faucet verifier listens for submitted claims addresses to itself and checks whether the client is entitled to receive funds from the faucet, that is
  - If it has previously received funds from the faucet
    - The verifier checks that enough time has elapsed since the last request (the verifier maintains a database with timestamps of the latest requests per account); or
  - If it is its first time requesting funds, the verifier checks the validity of the "proof"
- If the above conditions are met, the verifier settles the claim
- The [faucet](faucet.rs) listens for claims settled by the faucet verifier and just fulfills  the requests

## Quickstart

From this directory, the following commands will get you started quickly:
To start the faucet verifier, run the command below:

```bash
cargo run --example faucet-verifier
```

To start the faucet, run the command below:
```bash
cargo run --example faucet
```

## Configuration

The two executables rely on YAML configuration file for initialization. Alter these to suite your needs.

### Sample `faucet_verifier.yml`

```yml
private_key: 79eb4ee7c5061998e04cae9859485b51ba37e1865d11454c404991eea58acabf
vsl_server_addr: 127.0.0.1:44444
master_account_address: 0x1010101010101010101010101010101010101010
validator_address: 0x1010101010101010101010101010101010101010
max_amount: 100
min_waiting_time: 1440
db_path: faucet.db
```

| Key                      | Description                                                               |
| ------------------------ | ------------------------------------------------------------------------- |
| `private_key`            | Private key of the faucet verifier, used for signing settlement requests. |
| `vsl_server_addr`        | Address and port of the VSL RPC server node.                              |
| `master_account_address` | Address of the "master" account used to initialize client accounts.       |
| `validator_address`      | Address of the validator node that signs settled claims.                  |
| `max_amount`             | Maximum token amount (in atto-tokens) that can be requested.              |
| `min_waiting_time`       | Minimum wait time (in minutes) between successful requests.               |
| `db_path`                | Path to the local database storing request timestamps per account.        |

### Sample `faucet.yml`

```yml
private_key: 79eb4ee7c5061998e04cae9859485b51ba37e1865d11454c404991eea58acabf
validator_address: 0x1010101010101010101010101010101010101010
verifier_address: 0x1010101010101010101010101010101010101010
vsl_server_addr: 127.0.0.1:44444
max_amount: 100
```

| Key                 | Description                                                              |
| ------------------- | ------------------------------------------------------------------------ |
| `private_key`       | Private key of the faucet, used for signing payment requests.            |
| `validator_address` | Address of the validator node that signs settled claims.                 |
| `verifier_address`  | Address of the accepted faucet verifier.                                 |
| `vsl_server_addr`   | Address and port of the VSL RPC server node.                             |
| `max_amount`        | Maximum token amount (in atto-tokens) that can be fulfilled per request. |

For full list of options, please run the examples with the `--help` option.

```bash
cargo run --example faucet-verifier -- --help
cargo run --example faucet -- --help
```
