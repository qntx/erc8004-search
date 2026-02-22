# erc8004-search

Rust SDK for the [ERC-8004 Semantic Search Service](https://github.com/qntx/erc8004-search-service).

Provides a typed, ergonomic client for querying on-chain AI agent registrations
via semantic search, with built-in [x402](https://www.x402.org/) payment support.

## Quick Start

```rust
use erc8004_search::SearchClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SearchClient::new("https://search.example.com")?;
    let response = client.search("DeFi lending agent").await?;

    for result in &response.results {
        println!("#{} {} (score: {:.3})", result.rank, result.name, result.score);
    }
    Ok(())
}
```

## x402 Payment

When the service requires payment, attach an EVM signer:

```rust
use erc8004_search::SearchClient;
use alloy_signer_local::PrivateKeySigner;
use std::sync::Arc;

let signer: PrivateKeySigner = "0x...".parse()?;
let client = SearchClient::builder("https://search.example.com")
    .evm_signer_arc(Arc::new(signer))
    .build()?;
```

## Filtered Search

```rust
use erc8004_search::{SearchClient, SearchRequest, Filters};
use serde_json::json;

let req = SearchRequest::new("MCP tool server")
    .limit(5)
    .min_score(0.5)
    .filters(
        Filters::new()
            .eq("chainId", json!(8453))
            .eq("active", json!(true))
            .r#in("serviceName", vec![json!("MCP"), json!("A2A")])
    );
let resp = client.execute(req).await?;
```

## Cursor Pagination

```rust
let results = client.search_all("blockchain agent", 10).await?;
// Collects up to 10 pages of results automatically.
```

## Feature Flags

| Feature  | Default | Description                          |
|----------|---------|--------------------------------------|
| `evm`    | Yes     | EVM chain payment via `r402-evm`     |
| `solana` | No      | Solana chain payment via `r402-svm`  |

## Examples

```sh
# Basic search with x402 payment
PRIVATE_KEY="0x..." cargo run --example search

# Filtered search with pagination
PRIVATE_KEY="0x..." cargo run --example search_filters

# Custom service URL
PRIVATE_KEY="0x..." SEARCH_URL="https://search.example.com" cargo run --example search
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT License](LICENSE-MIT) at your option.
