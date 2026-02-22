//! Filtered semantic search with automatic x402 wallet payment.
//!
//! Demonstrates ERC-8004 filter operators (`equals`, `in`, `notIn`)
//! and cursor-based pagination with x402 payment.
//!
//! ```sh
//! PRIVATE_KEY="0x..." cargo run --example search_filters
//! PRIVATE_KEY="0x..." SEARCH_URL="https://search.example.com" cargo run --example search_filters
//! ```

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::sync::Arc;

use erc8004_search::{Filters, SearchClient, SearchRequest};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = std::env::var("PRIVATE_KEY").expect("set PRIVATE_KEY env var");
    let search_url = std::env::var("SEARCH_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into());

    let signer: alloy_signer_local::PrivateKeySigner = private_key.parse()?;
    eprintln!(
        "wallet:  {}",
        r402_evm::exact::client::SignerLike::address(&signer)
    );
    eprintln!("service: {search_url}\n");

    let client = SearchClient::builder(&search_url)
        .evm_signer_arc(Arc::new(signer))
        .build()?;

    // --- equals: filter by chain ---
    section("equals { chainId: 8453 }");
    let req = SearchRequest::new("MCP tool server")
        .limit(3)
        .filters(Filters::new().eq("chainId", json!(8453)));
    print_response(&client.execute(req).await?);

    // --- in: multiple service names ---
    section("in { serviceName: [MCP, REST] }");
    let req = SearchRequest::new("AI assistant agent")
        .limit(3)
        .filters(Filters::new().r#in("serviceName", vec![json!("MCP"), json!("REST")]));
    print_response(&client.execute(req).await?);

    // --- equals: boolean fields ---
    section("equals { active: true, x402Support: true }");
    let req = SearchRequest::new("DeFi trading bot").limit(3).filters(
        Filters::new()
            .eq("active", json!(true))
            .eq("x402Support", json!(true)),
    );
    print_response(&client.execute(req).await?);

    // --- notIn: exclude chains ---
    section("notIn { chainId: [1] }");
    let req = SearchRequest::new("oracle data feed")
        .limit(3)
        .filters(Filters::new().not_in("chainId", vec![json!(1)]));
    print_response(&client.execute(req).await?);

    // --- cursor-based pagination ---
    section("pagination (limit=2, two pages)");
    let resp = client
        .execute(SearchRequest::new("blockchain agent").limit(2))
        .await?;
    print_response(&resp);

    if let Some(pg) = &resp.pagination
        && let Some(cursor) = &pg.next_cursor
    {
        eprintln!("  -> fetching page 2 (cursor={cursor})");
        let req = SearchRequest::new("blockchain agent")
            .limit(2)
            .cursor(cursor);
        print_response(&client.execute(req).await?);
    }

    Ok(())
}

fn section(title: &str) {
    eprintln!("\n--- {title} ---");
}

fn print_response(resp: &erc8004_search::SearchResponse) {
    eprintln!("  {} results", resp.results.len());
    for r in &resp.results {
        let desc = truncate(r.description.trim(), 60);
        eprintln!(
            "  #{:<2} {:.3}  {}  (chain {})  {desc}",
            r.rank, r.score, r.name, r.chain_id
        );
    }
    if let Some(pg) = &resp.pagination
        && let Some(cursor) = &pg.next_cursor
    {
        eprintln!("  next_cursor: {cursor}");
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_owned()
    } else {
        let end = s
            .char_indices()
            .nth(max.saturating_sub(3))
            .map_or(s.len(), |(i, _)| i);
        format!("{}...", &s[..end])
    }
}
