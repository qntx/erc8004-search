//! # erc8004-search
//!
//! Rust SDK for the [ERC-8004 Semantic Search Service](https://github.com/qntx/erc8004-search-service).
//!
//! Provides a typed, ergonomic client for querying on-chain AI agent registrations
//! via semantic search, with built-in [x402](https://www.x402.org/) payment support.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use erc8004_search::SearchClient;
//! use alloy_signer_local::PrivateKeySigner;
//!
//! let signer: PrivateKeySigner = "0x...".parse()?;
//! let client = SearchClient::builder()
//!     .evm_signer(signer)
//!     .build()?;
//!
//! let response = client.search("DeFi lending agent").await?;
//! for result in &response.results {
//!     println!("#{} {} (score: {:.3})", result.rank, result.name, result.score);
//! }
//! ```
//!
//! ## Custom Endpoint
//!
//! ```rust,ignore
//! use erc8004_search::SearchClient;
//! use alloy_signer_local::PrivateKeySigner;
//!
//! let signer: PrivateKeySigner = "0x...".parse()?;
//! let client = SearchClient::builder()
//!     .base_url("https://custom.example.com")
//!     .evm_signer(signer)
//!     .build()?;
//! ```
//!
//! ## Feature Flags
//!
//! - **`evm`** *(default)* — EVM chain payment support via `r402-evm`
//! - **`solana`** — Solana chain payment support via `r402-svm`

mod client;
mod error;
mod types;

pub use client::{DEFAULT_BASE_URL, SearchClient, SearchClientBuilder};
pub use error::{Error, Result};
pub use types::{
    ApiFeatures, ApiLimits, CapabilitiesResponse, ErrorResponse, Filters, HealthResponse,
    PaginationMeta, Protocol, ProviderInfo, ResultMetadata, SearchRequest, SearchResponse,
    SearchResultItem, ServiceHealth, TrustModel, WalletFilter,
};

// Re-export x402 payment types for convenience.
pub use r402::scheme::{FirstMatch, MaxAmount, PaymentSelector, PreferChain};
#[cfg(feature = "evm")]
pub use r402_evm::Eip155ExactClient;
pub use r402_http::client::X402Client;
