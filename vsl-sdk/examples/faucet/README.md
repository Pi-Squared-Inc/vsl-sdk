# Faucet example

## Overview

This app provides a means for clients to request and (pending verification) be granted funds from a faucet.

It is also built as an example of claim verification and settlement.

### Basic workflow

- client submits a claim with "claim" field being the amount requested and listing the [faucet verifier](faucet_verifier.rs) as (one of the) verifier(s)
  - if it's the first time requesting funds, it also sends as "proof" the id of a settled claim representing a payment to itself from the "master account" (the account we use to initialize whitelisted accounts with funds).
- the faucet verifier listens for submitted claims addresses to itself and checks whether the client is entitled to receive funds from the faucet, that is
  - if it has previously received funds from the faucet
    - the verifier checks that enough time has elapsed since the last request (the verifier maintains a database with timestamps of the latest requests per account); or
  - if it is its first time requesting funds, the verifier checks the validity of the "proof"
- if the above conditions are met, the verifier settles the claim
- the [faucet](faucet.rs) listens for claims settled by the faucet verifier and just fulfills  the requests

## Quickstart

Form this directory, use
```bash
cargo run --example faucet-verifier
```
to start the faucet verifier, and

```bash
cargo run --example faucet
```
to start the faucet itself.

## Configuration

The two executables rely on YAML configuration file for initialization. Alter these to suite your needs.

### Sample `faucet_verifier.yml`

```yml
private_key: 79eb4ee7c5061998e04cae9859485b51ba37e1865d11454c404991eea58acabf
```

The private key of the faucet verifier. Used for signing (settlement) requests.

```yml
vsl_server_addr: 127.0.0.1:44444
```

The address (and port) of the RPC server node.

```yml
master_account_address: 0x1010101010101010101010101010101010101010
```

The address of the "master" account used to grant initial funds to accounts.

```yml
validator_address: 0x1010101010101010101010101010101010101010
```

The address of the validator (which signs claims settled on the VSL node)

```yml
max_amount: 100
```

The maximum amount (in atto-tokens) which can be requested from the faucet

```yml
min_waiting_time: 1440
```

The minimum time a client needs to wait between successful faucet requests.

```yml
db_path: faucet.db
```

The path to the file storing the database with timestamps of the latest
succesful faucet requests for each client.

### Sample `faucet.yml`

```yml
private_key: 79eb4ee7c5061998e04cae9859485b51ba37e1865d11454c404991eea58acabf
```

The private key of the faucet. Used for signing (payment) requests.

```yml
validator_address:  0x1010101010101010101010101010101010101010
```

Same as above

```yml
verifier_address:  0x1010101010101010101010101010101010101010
```

The accepted faucet verifier

```yml
vsl_server_addr: 127.0.0.1:44444
```

```yml
max_amount: 100
```

## More information

For full information please run the examples with the `--help` option.

```bash
cargo run --example faucet-verifier -- --help
cargo run --example faucet -- --help
```
