//! Minimal semantic search with x402 payment.
//!
//! ```sh
//! PRIVATE_KEY="0x..." cargo run --example search
//! PRIVATE_KEY="0x..." QUERY="AI agent on Base" cargo run --example search
//! PRIVATE_KEY="0x..." SEARCH_URL="https://custom.host" cargo run --example search
//! ```

#![allow(clippy::print_stdout, clippy::print_stderr)]

use erc8004_search::SearchClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: alloy_signer_local::PrivateKeySigner =
        std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY").parse()?;

    let client = SearchClient::builder()
        .evm_signer(signer)
        .build()?;

    let resp = client.search("DeFi lending").await?;

    println!("{} results (request {})\n", resp.results.len(), resp.request_id);
    for r in &resp.results {
        println!("  #{:<2} {:.3}  {}  — {}", r.rank, r.score, r.name, r.description);
    }

    Ok(())
}
