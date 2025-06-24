# VSL Request Tutorial

In this tutorial, we will walk through the process of making a request to the VSL service using the `RpcWrapper` available in the [VSL SDK](https://github.com/Pi-Squared-Inc/vsl-sdk/blob/main/vsl-sdk/src/rpc_wrapper.rs).

## Creating an account with a private key

Any Ethereum private key should work. The private key is a random 32-byte number, so you can generate it with a random number generator, e.g.:

```bash
pip install eth_account
python3 -c '
from eth_account import Account
import secrets

private_key = "0x" + secrets.token_hex(32)
print("Private key:", private_key)
acct = Account.from_key(private_key)
print("Address:", acct.address)
'
```

The output should look like this:
```
Private key: 0xc3bbe3b26edabb0be0298244517a6f97538be4f7df3ed1b3508f2804f9932019
Address: 0x75c51B0770646902999e55D86c5F399FaF6AbDc7
```

In this tutorial, we will use the private key and address above. Note that, in general, the private key should be kept... well... private.

You can also use the [vsl-cli](https://github.com/Pi-Squared-Inc/vsl-cli) tool, with the following commands:

```bash
vsl-cli account:create my-account
vsl-cli account:export my-account
```

## Setting up VSL

In order to keep things self-contained, this tutorial will explain how to start and use your own VSL validator instance. However, you can also use a public instance of VSL (currently, there is one available at `34.94.236.105`, and it uses the same ports as the private VSL instance).

Either clone vsl-sdk, or copy the [docker-compose.public.yml](https://github.com/Pi-Squared-Inc/vsl-sdk/blob/main/docker-compose.public.yml) file in an empty directory.

In the same directory as the `docker-compose.public.yml` file, create a `.env`
file containing the private key of the master account, e.g.

```bash
VSL_MASTER_ADDR=0x75c51B0770646902999e55D86c5F399FaF6AbDc7
```

Initialize and load VSL, see the [quick_start.md](quick_start.md) file for details:

```bash
docker compose -f docker-compose.public.yml pull
docker compose -f docker-compose.public.yml up
```

Then, in a different terminal, test that the service started properly:

```bash
curl -X POST http://localhost:44444 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "vsl_getHealth",
    "params": {}
  }'
```

You should receive this:

```json
{"jsonrpc":"2.0","id":1,"result":"ok"}
```

## Creating the project

First, you need to create a Rust project and add vsl-sdk as a dependency:

```bash
cargo new send_claim --bin
cd send_claim
cargo add vsl-sdk --git "https://github.com/Pi-Squared-Inc/vsl-sdk"
```

If the `add` command fails with `no authentication methods succeeded` you can try to fix the issue with:

```bash
eval `ssh-agent -s`
ssh-add
```

## Initialization

In this tutorial, we will use async features. We will use tokyo to handle this.

```bash
cargo add tokio@1.45.1
```

Next, let us make the `main` function async. Let us open`src/main.rs` and replace its contents with the following:

```rs
#[tokio::main(flavor = "current_thread")]
pub async fn main() {
}
```

In order to make RPC requests, we need to create a JSON RPC HTTP client. For that, we need to add a dependency on jsonrpsee:

```bash
cargo add jsonrpsee@0.25.1 --features client
```

Next, open the `src/main.rs` file and add the following import:

```rs
use jsonrpsee::http_client::HttpClientBuilder;
```

and, in the main function, create the http client:

```rs
    let http_url = "http://127.0.0.1:4000/".to_string();
    let http_client = HttpClientBuilder::new()
        .build(http_url)
        .expect("Could not connect");
```

The vsl-sdk package provides a `RpcWrapper` object which makes requests on behalf of an account. For simplicity, we will use the master account we used when starting the VSL server, but any account that has enough tokens to pay for the requests will work. See the [fee-schedule.md](fee-schedule.md) file for the endpoints which require payments.

Let us import the RpcWrapper:

```rs
use vsl_sdk::rpc_wrapper::RpcWrapper;
```

At the end of the main function, let us create the account wrapper we will use for making RPC requests:

```rs
    // Usually it's a bad idea to hardcode private keys, so in a real project
    // you should read it from a secure source.
    let private_key = "0xc3bbe3b26edabb0be0298244517a6f97538be4f7df3ed1b3508f2804f9932019".to_string();
    let mut account =
        RpcWrapper::from_private_key_str(&private_key, None, &http_client).await;
```

## The request

At this point, we should be ready to submit a claim. First, we need the following imports:

```rs
use std::str::FromStr;
use vsl_sdk::Address;
use vsl_sdk::Amount;
use vsl_sdk::Timestamp;
```

Next, we need the claim data. The `claim`, `claim_type` and `proof` fields are opaque to the VSL validator. They should make sens to the client making the request (you) and to the verifiers specified when submitting the claim.

```rs
    let claim = "The claim".to_string();
    let claim_type = "The claim type".to_string();
    let proof = "A proof".to_string();
```

Each claim must be verified by at least one verifier. Since verifiers are outside of the scope of this tutorial, we will provide a dummy address here.

```rs
    let verifier_address_str = "0x1234567890abcdef1234567890abcdef12345678".to_string();
    let verifier_address = Address::from_str(&&verifier_address_str).expect("Invalid address");
```

In general, we can have multiple verifiers, and we must specify how many of them must verify our claim before the VSL validator will allow it to be settled.

```rs
    let quorum = 1_u16;
```

The verifiers are paid for their work, so the client must specify the amount used for this. The client and the verifiers determine between themselves what constitutes an acceptable payment. Note that if there is more than one verifier, the fee is split between all of them.

```rs
    let fee = Amount::from_attos(1);
```

Each claim must specify an expiration time. If the clam was not settled before the expiration time, it will be deleted.

```rs
    let now = Timestamp::now();
    let expires = Timestamp::from_seconds(now.seconds() + 60 * 60); // 1 hour from now
```

We can finally make the request:

```rs
    let response = account.submit_claim(
            claim,
            claim_type,
            proof,
            vec![&verifier_address],
            quorum,
            expires,
            fee
    ).await.expect("Error while submitting the claim");
    println!("Claim submitted successfully: {:?}", response);
```

After running the code with `cargo run`, the output should look something like this:

```bash
Claim submitted successfully: 0x0049ab87ca4ed85fc11cd32f2b5bb4285717d9dd4acfc8c9090cfef78193f3b7
```

If we look at the VSL Explorer, [http://127.0.0.1:4000/](http://127.0.0.1:4000/), we should see a claim with the same ID as above, with a `Pending` status.

For reference, here is the full tutorial code:

main.rs:
```rs
use jsonrpsee::http_client::HttpClientBuilder;
use std::str::FromStr;
use vsl_sdk::{Address, Amount, Timestamp, rpc_wrapper::RpcWrapper};

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    // Replace with your actual VSL server address.
    let http_url = "http://127.0.0.1:44444/".to_string();
    let http_client = HttpClientBuilder::new()
        .build(http_url)
        .expect("Could not connect");

    // Usually it's a bad idea to hardcode private keys, so in a real project
    // you should read it from a secure source.
    let private_key =
        "0xc3bbe3b26edabb0be0298244517a6f97538be4f7df3ed1b3508f2804f9932019".to_string();
    let mut account = RpcWrapper::from_private_key_str(&private_key, None, &http_client)
        .await
        .expect("Failed to create an account wrapper.");

    let claim = "The claim".to_string();
    let claim_type = "The claim type".to_string();
    let proof = "A proof".to_string();
    let verifier_address_str = "0x1234567890abcdef1234567890abcdef12345678".to_string();
    let verifier_address = Address::from_str(&&verifier_address_str).expect("Invalid address");
    let quorum = 1_u16;
    let fee = Amount::from_attos(1);
    let now = Timestamp::now();
    let expires = Timestamp::from_seconds(now.seconds() + 60 * 60); // 1 hour from now
    let response = account
        .submit_claim(
            claim,
            claim_type,
            proof,
            vec![&verifier_address],
            quorum,
            expires,
            fee,
        )
        .await
        .expect("Error while submitting the claim");
    println!("Claim submitted successfully: {:?}", response);
}
```

Cargo.toml:
```toml
[package]
name = "send_claim"
version = "0.1.0"
edition = "2024"

[dependencies]
jsonrpsee = { version = "0.25.1", features = ["client"] }
tokio = "1.45.1"
vsl-sdk = { git = "https://github.com/Pi-Squared-Inc/vsl-sdk", version = "0.1.0" }
```