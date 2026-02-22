//! Basic semantic search with automatic x402 wallet payment.
//!
//! ```sh
//! PRIVATE_KEY="0x..." cargo run --example search
//! PRIVATE_KEY="0x..." SEARCH_URL="https://search.example.com" cargo run --example search
//! PRIVATE_KEY="0x..." QUERY="AI agent on Base" cargo run --example search
//! ```

#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::sync::Arc;

use erc8004_search::SearchClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = std::env::var("PRIVATE_KEY").expect("set PRIVATE_KEY env var");
    let search_url = std::env::var("SEARCH_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let query = std::env::var("QUERY").unwrap_or_else(|_| "DeFi lending protocol on Base".into());

    let signer: alloy_signer_local::PrivateKeySigner = private_key.parse()?;
    eprintln!(
        "wallet:  {}",
        r402_evm::exact::client::SignerLike::address(&signer)
    );
    eprintln!("service: {search_url}");
    eprintln!("query:   {query}\n");

    let client = SearchClient::builder(&search_url)
        .evm_signer_arc(Arc::new(signer))
        .build()?;

    let resp = client.search(&query).await?;

    eprintln!(
        "{} results  (request {})\n",
        resp.results.len(),
        resp.request_id
    );

    for r in &resp.results {
        eprintln!(
            "  #{:<2} {:.3}  {}  (chain {})",
            r.rank, r.score, r.name, r.chain_id
        );
        let desc = r.description.trim();
        if !desc.is_empty() {
            eprintln!("          {}", truncate(desc, 72));
        }
        if let Some(meta) = &r.metadata {
            if let Some(uri) = &meta.agent_uri
                && !uri.starts_with("data:")
            {
                eprintln!("          uri: {uri}");
            }
            if let Some(svcs) = meta.services.as_array() {
                let names: Vec<&str> = svcs.iter().filter_map(|s| s["name"].as_str()).collect();
                if !names.is_empty() {
                    eprintln!("          services: {}", names.join(", "));
                }
            }
        }
    }

    Ok(())
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
