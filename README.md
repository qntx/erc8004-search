<div align="center">

# erc8004-search

**Rust SDK for the [ERC-8004 Semantic Search](https://github.com/qntx/8004)**

[![Crates.io](https://img.shields.io/crates/v/erc8004-search.svg)](https://crates.io/crates/erc8004-search)
[![Documentation](https://docs.rs/erc8004-search/badge.svg)](https://docs.rs/erc8004-search)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

Discover on-chain AI agents through semantic search — with built-in [x402](https://www.x402.org/) micropayment support.

</div>

---

## Highlights

- **Built-in endpoint** — Defaults to the QNTX-hosted service (`https://search.qntx.fun`) with x402 micropayments.
- **x402 payment middleware** — Automatic EVM (EIP-155) and Solana payment signing via [`r402`](https://github.com/qntx/r402).
- **Type-safe filters** — `Protocol`, `TrustModel`, and `WalletFilter` enums prevent typos and provide IDE autocompletion.
- **Flexible payment control** — Pluggable `PaymentSelector` (`FirstMatch`, `PreferChain`, `MaxAmount`) and `PaymentPolicy` support.
- **Production-ready** — Connection pooling, configurable timeouts, structured error handling, and `tracing` instrumentation.

## Installation

```sh
cargo add erc8004-search
```

## Quick Start

The QNTX-hosted service uses x402 micropayments, so an EVM signer is required:

```rust
use erc8004_search::SearchClient;
use alloy_signer_local::PrivateKeySigner;

#[tokio::main]
async fn main() -> erc8004_search::Result<()> {
    let signer: PrivateKeySigner = std::env::var("PRIVATE_KEY")?.parse()?;
    let client = SearchClient::builder()
        .evm_signer(signer)
        .build()?;

    let response = client.search("DeFi lending agent").await?;
    for item in &response.results {
        println!("#{} {} — score {:.3}", item.rank, item.name, item.score);
    }
    Ok(())
}
```

For self-hosted endpoints, chain `.base_url("https://...")` on the builder.

## Filtered Search

Use type-safe enums to narrow results by on-chain metadata:

```rust
use erc8004_search::{Filters, Protocol, TrustModel, SearchRequest};

let request = SearchRequest::new("MCP tool server")
    .limit(5)
    .min_score(0.5)
    .filters(
        Filters::new()
            .chain_id(8453)
            .active(true)
            .x402_support(true)
            .protocols([Protocol::Mcp, Protocol::A2a])
            .trust_models([TrustModel::Reputation])
    );
```

### Available Filter Methods

| Method                   | Field            | Description                         |
|--------------------------|------------------|-------------------------------------|
| `.chain_id(i64)`         | `chainId`        | Exact chain ID                      |
| `.chain_id_in([...])`    | `chainId`        | Match any chain ID                  |
| `.active(bool)`          | `active`         | Agent active status                 |
| `.x402_support(bool)`    | `x402Support`    | x402 payment support                |
| `.protocols([...])`      | `serviceName`    | Service protocols (`Protocol` enum) |
| `.trust_models([...])`   | `supportedTrust` | Trust models (`TrustModel` enum)    |
| `.agent_id(str)`         | `agentId`        | Exact agent ID                      |
| `.name_eq(str)`          | `name`           | Exact agent name                    |

Low-level `.eq()` / `.r#in()` / `.not_in()` / `.exists()` / `.not_exists()` methods are available for custom fields.

### Protocol Enum

`Mcp` · `A2a` · `Oasf` · `Ens` · `Did` · `Web` · `Email`

### TrustModel Enum

`Reputation` · `CryptoEconomic` · `TeeAttestation`

## Reputation Wallet Filter

Control which wallet feedback is included in reputation score calculation:

```rust
use erc8004_search::{SearchRequest, WalletFilter};

let req = SearchRequest::new("DeFi agent")
    .wallet_filter(WalletFilter::Exclude(vec!["0xdead...".into()]));

let req = SearchRequest::new("DeFi agent")
    .wallet_filter(WalletFilter::Include(vec!["0xbeef...".into()]));
```

## Payment Configuration

### Custom Payment Selector

```rust
use erc8004_search::{SearchClient, PreferChain};
use r402::chain::ChainIdPattern;

let client = SearchClient::builder()
    .evm_signer(signer)
    .payment_selector(PreferChain::new([ChainIdPattern::exact(8453)]))
    .build()?;
```

### Available Selectors

| Selector      | Description                              |
|---------------|------------------------------------------|
| `FirstMatch`  | First compatible scheme (default)        |
| `PreferChain` | Prefer specific chains in priority order |
| `MaxAmount`   | Reject payments above a ceiling          |

## Cursor Pagination

```rust
let all_results = client.search_all("blockchain agent", 10).await?;
println!("Total results: {}", all_results.len());
```

## Health & Capabilities

```rust
let health = client.health().await?;
println!("Status: {}", health.status);

let caps = client.capabilities().await?;
println!("Max query length: {}", caps.limits.max_query_length);
```

## Custom HTTP Settings

```rust
use std::time::Duration;

let client = SearchClient::builder()
    .evm_signer(signer)
    .timeout(Duration::from_secs(30))
    .user_agent("my-app/1.0")
    .build()?;
```

## Feature Flags

| Feature  | Default | Description                                 |
|----------|---------|---------------------------------------------|
| `evm`    | **yes** | EVM chain payment support via `r402-evm`    |
| `solana` | no      | Solana chain payment support via `r402-svm` |

Enable Solana support:

```toml
[dependencies]
erc8004-search = { version = "0.3", features = ["evm", "solana"] }
```

## Examples

```sh
# Basic search
PRIVATE_KEY="0x..." cargo run --example search

# Filtered search with pagination
PRIVATE_KEY="0x..." cargo run --example search_filters
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.
