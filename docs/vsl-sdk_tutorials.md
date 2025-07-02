# VSL SDK Quickstart Tutorials (Rust)

The VSL SDK provides a convenient Rust interface for interacting with a VSL node over JSON-RPC and WebSocket. Below are six tutorials demonstrating common workflows. Each section includes:

1. **Conceptual overview** — What the code does and why.
2. **Imports & setup** — Crate dependencies and initialization.
3. **Step-by-step code** — Rust code with inline comments.
4. **Expected output** — Sample console output.



## Tutorial 1: Account Creation, Funding & Transfer

**Overview:** This tutorial shows how to instantiate the `Client`, create two accounts, fund them via the testnet faucet, and transfer tokens between them.

### Imports & Initialization

```rust
use dotenv::dotenv;                     // Load .env into environment
use std::env;                           // Read environment variables
use vsl_sdk::{Client, FaucetParams, TransferParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load API key and RPC URL from .env
    dotenv().ok();
    let api_key = env::var("VSL_API_KEY")?;
    let rpc_url = env::var("VSL_RPC_URL")?;

    // Build the VSL client with HTTP transport
    let client = Client::builder()
        .api_key(api_key)
        .rpc_url(rpc_url)
        .build()?;
```

### Creating Accounts

```rust
    // Create two fresh accounts (keypairs managed by the SDK)
    let account_a = client.accounts().create().await?;
    let account_b = client.accounts().create().await?;
    println!("Account A address: {}", account_a.address);
    println!("Account B address: {}", account_b.address);
```

### Funding Accounts (Testnet Only)

```rust
    // Use the built-in faucet to top up each account with 1,000 tokens
    client.faucet().fund(FaucetParams::new(&account_a.address, 1000)).await?;
    client.faucet().fund(FaucetParams::new(&account_b.address, 1000)).await?;
    println!("Both accounts funded with 1,000 tokens");
```

### Transferring Tokens

```rust
    // Transfer 100 tokens from Account A to Account B
    let tx = client.transactions()
        .transfer(
            TransferParams::builder()
                .from(&account_a.address)
                .to(&account_b.address)
                .amount(100)
                .build()? )
        .await?;
    println!("Transfer transaction hash: {}", tx.hash);
    Ok(())
}
```

**Expected Output:**

```
Account A address: 0x...
Account B address: 0x...
Both accounts funded with 1,000 tokens
Transfer transaction hash: 0x...
```

---

## Tutorial 2: Custom Asset Creation & Transfer

**Overview:** Demonstrates issuing a new asset (token) and sending it to another account.

### Imports & Setup

```rust
use dotenv::dotenv;
use std::env;
use vsl_sdk::{Client, AssetParams, AssetTransferParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let client = Client::builder()
        .api_key(env::var("VSL_API_KEY")?)
        .rpc_url(env::var("VSL_RPC_URL")?)
        .build()?;
```

### Creating & Funding the Creator

```rust
    // Account that will issue the new token
    let creator = client.accounts().create().await?;
    client.faucet().fund(FaucetParams::new(&creator.address, 1000)).await?;
    println!("Creator account: {}", creator.address);
```

### Issuing a Token

```rust
    // Define asset parameters: name, symbol, supply, decimals
    let asset = client.assets().create(
        AssetParams::new(&creator.address, "MyToken", "MTK", 10_000, 2)
    ).await?;
    println!("Issued asset ID: {}", asset.id);
```

### Transferring the Asset

```rust
    // Create a receiver and send 500 MTK
    let receiver = client.accounts().create().await?;
    let tx = client.transactions().transfer_asset(
        AssetTransferParams::new(&creator.address, &receiver.address, asset.id, 500)
    ).await?;
    println!("Asset transfer hash: {}", tx.hash);
    Ok(())
}
```

**Expected Output:**

```
Creator account: 0x...
Issued asset ID: 123
Asset transfer hash: 0x...
```

---

## Tutorial 3: Subscribing to VSL Events

**Overview:** Shows how to listen for real-time events (e.g., new blocks) over WebSocket.

### Imports & Initialization

```rust
use dotenv::dotenv;
use std::env;
use futures::StreamExt;               // For processing the async stream
use vsl_sdk::{Client, SubscriptionParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let client = Client::builder()
        .api_key(env::var("VSL_API_KEY")?)
        .rpc_url(env::var("VSL_RPC_URL")?)
        .ws_url(env::var("VSL_WS_URL")?)
        .build()?;
```

### Subscribing & Processing Events

```rust
    // Subscribe to "blocks" topic to receive new block notifications
    let mut stream = client.subscriptions()
        .subscribe(SubscriptionParams::new("blocks"))
        .await?;

    println!("Waiting for new blocks...");
    while let Some(event) = stream.next().await {
        match event {
            Ok(block) => println!("New block: {:?}", block),
            Err(err) => eprintln!("Subscription error: {}", err),
        }
    }
    Ok(())
}
```

---

## Tutorial 4: Adding a Verifiable Claim (Identity Module)

**Overview:** Adds a custom claim (e.g., KYC or email) to an account via the Identity module.

```rust
use dotenv::dotenv;
use std::env;
use vsl_sdk::{Client, IdentityClaimParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let client = Client::builder()
        .api_key(env::var("VSL_API_KEY")?)
        .rpc_url(env::var("VSL_RPC_URL")?)
        .build()?;

    // Create and fund the account that issues the claim
    let account = client.accounts().create().await?;
    client.faucet().fund(FaucetParams::new(&account.address, 1000)).await?;

    // Build claim parameters
    let params = IdentityClaimParams::builder()
        .subject(&account.address)   // Target account
        .claim_type("Email")        // Claim category
        .value("user@example.com")  // Claim data
        .issuer(&account.address)     // Claim issuer
        .build()?;

    // Submit the claim
    let claim = client.identity().add_claim(params).await?;
    println!("Added claim ID: {}", claim.id);
    Ok(())
}
```

---

## Tutorial 5: Retrieving & Verifying Claims

**Overview:** Fetches all claims for an account and verifies each signature.

```rust
use dotenv::dotenv;
use std::env;
use vsl_sdk::{Client, IdentityVerifyParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let client = Client::builder()
        .api_key(env::var("VSL_API_KEY")?)
        .rpc_url(env::var("VSL_RPC_URL")?)
        .build()?;

    let subject = "0xAb12...Cd34"; // Replace with actual address
    let claims = client.identity().get_claims(subject).await?;
    println!("{} claims found for {}", claims.len(), subject);

    for claim in claims {
        println!("Claim {}: {}={}, issuer={}"
            , claim.id, claim.claim_type, claim.value, claim.issuer);
        let valid = client.identity()
            .verify_claim(IdentityVerifyParams::new(claim.id))
            .await?;
        println!("  Signature valid: {}", valid);
    }
    Ok(())
}
```

---

## Tutorial 6: Deploying & Invoking a Smart Contract (VM Module)

**Overview:** Compiles, deploys, and interacts with a Solidity contract using the VM module.

```rust
use dotenv::dotenv;
use std::env;
use std::fs;
use vsl_sdk::{Client, VmDeployParams, VmCallParams};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let client = Client::builder()
        .api_key(env::var("VSL_API_KEY")?)
        .rpc_url(env::var("VSL_RPC_URL")?)
        .build()?;

    // 1. Create deployer account and fund it
    let deployer = client.accounts().create().await?;
    client.faucet().fund(FaucetParams::new(&deployer.address, 1000)).await?;

    // 2. Read Solidity source
    let source = fs::read_to_string("./contracts/Counter.sol")?;

    // 3. Deploy contract
    let deploy = VmDeployParams::builder()
        .from(&deployer.address)
        .source_code(&source)
        .constructor_args(vec!["0".into()])
        .build()?;
    let deployed = client.vm().deploy(deploy).await?;
    println!("Deployed at: {}", deployed.contract_address);

    // 4. Call increment()
    let inc = VmCallParams::builder()
        .from(&deployer.address)
        .to(&deployed.contract_address)
        .method("increment")
        .args(vec![])
        .build()?;
    let receipt = client.vm().call(inc).await?;
    println!("increment TX: {}", receipt.tx_hash);

    // 5. Query counter value
    let query = VmCallParams::builder()
        .to(&deployed.contract_address)
        .method("getCount")
        .args(vec![])
        .build()?;
    let result = client.vm().call(query).await?;
    println!("Counter value: {}", result.return_value);
    Ok(())
}
```

---

