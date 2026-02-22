//! Filtered search and cursor pagination with x402 payment.
//!
//! ```sh
//! PRIVATE_KEY="0x..." cargo run --example search_filters
//! ```

#![allow(clippy::print_stdout, clippy::print_stderr)]

use erc8004_search::{Filters, SearchClient, SearchRequest};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let signer: alloy_signer_local::PrivateKeySigner =
        std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY").parse()?;

    let client = SearchClient::builder().evm_signer(signer).build()?;

    // Filtered search: chain + active + service type
    let req = SearchRequest::new("MCP tool server")
        .limit(3)
        .min_score(0.3)
        .filters(
            Filters::new()
                .eq("chainId", json!(8453))
                .eq("active", json!(true))
                .r#in("serviceName", vec![json!("MCP"), json!("REST")]),
        );

    let resp = client.execute(req).await?;
    println!("filtered ({} results)", resp.results.len());
    for r in &resp.results {
        println!("  #{:<2} {:.3}  {}", r.rank, r.score, r.name);
    }

    // Cursor pagination: collect up to 3 pages
    let all = client.search_all("blockchain agent", 3).await?;
    println!("paginated ({} total)", all.len());
    for r in &all {
        println!("  #{:<2} {:.3}  {}", r.rank, r.score, r.name);
    }

    Ok(())
}
