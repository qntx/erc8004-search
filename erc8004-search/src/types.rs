//! Request and response types for the ERC-8004 Semantic Search API.
//!
//! All types use camelCase JSON serialization to match the
//! [ERC-8004 Semantic Search Standard v1](https://github.com/qntx/erc8004-search-service/blob/main/docs/SEMANTIC_SEARCH_STANDARD_V1.md).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `POST /api/v1/search` request body.
///
/// # Example
///
/// ```
/// use erc8004_search::SearchRequest;
///
/// let req = SearchRequest::new("DeFi lending protocol")
///     .limit(5)
///     .min_score(0.3)
///     .include_metadata(true);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    /// Natural-language search query.
    pub query: String,

    /// Maximum number of results (default: 10, max: 100).
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Offset for pagination (0-indexed).
    #[serde(default)]
    pub offset: usize,

    /// Cursor for cursor-based pagination (takes precedence over `offset`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    /// Structured filter criteria.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<Filters>,

    /// Minimum similarity score threshold (0.0–1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_score: Option<f32>,

    /// Include full metadata in results (default: true).
    #[serde(default = "default_include_metadata")]
    pub include_metadata: bool,

    /// Wallet addresses to **exclude** from reputation score re-aggregation.
    /// Mutually exclusive with `reputation_include_wallets`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reputation_exclude_wallets: Option<Vec<String>>,

    /// Wallet addresses to **exclusively include** in reputation score
    /// re-aggregation. Mutually exclusive with `reputation_exclude_wallets`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reputation_include_wallets: Option<Vec<String>>,
}

impl SearchRequest {
    /// Create a new search request with the given query.
    #[must_use]
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            limit: default_limit(),
            offset: 0,
            cursor: None,
            filters: None,
            min_score: None,
            include_metadata: true,
            reputation_exclude_wallets: None,
            reputation_include_wallets: None,
        }
    }

    /// Set the maximum number of results.
    #[must_use]
    pub const fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set the offset for pagination.
    #[must_use]
    pub const fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Set a cursor for cursor-based pagination.
    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Set structured filter criteria.
    #[must_use]
    pub fn filters(mut self, filters: Filters) -> Self {
        self.filters = Some(filters);
        self
    }

    /// Set the minimum similarity score threshold (0.0–1.0).
    #[must_use]
    pub const fn min_score(mut self, min_score: f32) -> Self {
        self.min_score = Some(min_score);
        self
    }

    /// Control whether full metadata is included in results.
    #[must_use]
    pub const fn include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Set wallet addresses to exclude from reputation re-aggregation.
    #[must_use]
    pub fn exclude_wallets(mut self, wallets: Vec<String>) -> Self {
        self.reputation_exclude_wallets = Some(wallets);
        self.reputation_include_wallets = None;
        self
    }

    /// Set wallet addresses to exclusively include in reputation re-aggregation.
    #[must_use]
    pub fn include_wallets(mut self, wallets: Vec<String>) -> Self {
        self.reputation_include_wallets = Some(wallets);
        self.reputation_exclude_wallets = None;
        self
    }
}

/// Structured filter object (ERC-8004 standard).
///
/// All conditions are AND-ed together across and within operators.
///
/// # Example
///
/// ```
/// use erc8004_search::Filters;
/// use serde_json::json;
///
/// let filters = Filters::new()
///     .eq("chainId", json!(8453))
///     .eq("active", json!(true))
///     .r#in("serviceName", vec![json!("MCP"), json!("A2A")]);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    /// Exact-match conditions: `{ "field": value }`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub equals: HashMap<String, Value>,

    /// Match-any conditions: `{ "field": [v1, v2] }`.
    #[serde(default, rename = "in", skip_serializing_if = "HashMap::is_empty")]
    pub in_: HashMap<String, Vec<Value>>,

    /// Exclude conditions: `{ "field": [v1, v2] }`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub not_in: HashMap<String, Vec<Value>>,

    /// Fields that must exist (not null).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exists: Vec<String>,

    /// Fields that must not exist.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub not_exists: Vec<String>,
}

impl Filters {
    /// Create an empty filter set.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an exact-match condition.
    #[must_use]
    pub fn eq(mut self, field: impl Into<String>, value: Value) -> Self {
        self.equals.insert(field.into(), value);
        self
    }

    /// Add a match-any condition (OR logic within the field).
    #[must_use]
    pub fn r#in(mut self, field: impl Into<String>, values: Vec<Value>) -> Self {
        self.in_.insert(field.into(), values);
        self
    }

    /// Add an exclusion condition.
    #[must_use]
    pub fn not_in(mut self, field: impl Into<String>, values: Vec<Value>) -> Self {
        self.not_in.insert(field.into(), values);
        self
    }

    /// Require a field to exist (not null).
    #[must_use]
    pub fn exists(mut self, field: impl Into<String>) -> Self {
        self.exists.push(field.into());
        self
    }

    /// Require a field to not exist.
    #[must_use]
    pub fn not_exists(mut self, field: impl Into<String>) -> Self {
        self.not_exists.push(field.into());
        self
    }

    /// Total number of filter conditions.
    #[must_use]
    pub fn count(&self) -> usize {
        self.equals.len()
            + self.in_.len()
            + self.not_in.len()
            + self.exists.len()
            + self.not_exists.len()
    }

    /// Returns `true` if no filter conditions are set.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

/// `POST /api/v1/search` response body (ERC-8004 standard).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    /// Echo of the search query.
    pub query: String,

    /// Search results, ordered by relevance score (highest first).
    pub results: Vec<SearchResultItem>,

    /// Pagination metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,

    /// Unique request identifier (matches `X-Request-ID` header).
    pub request_id: String,

    /// ISO 8601 timestamp.
    pub timestamp: String,

    /// Provider metadata (name and version).
    pub provider: ProviderInfo,
}

/// A single search result (ERC-8004 standard).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultItem {
    /// 1-indexed relevance rank.
    pub rank: usize,

    /// Agent identifier in `"{chainId}:{tokenId}"` format.
    pub agent_id: String,

    /// EIP-155 chain ID.
    pub chain_id: i64,

    /// Agent name.
    pub name: String,

    /// Agent description.
    pub description: String,

    /// Similarity score (0.0–1.0, higher = more similar).
    pub score: f32,

    /// Rich metadata (omitted when `includeMetadata` is false).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ResultMetadata>,

    /// Explanations for why this result matched.
    #[serde(default)]
    pub match_reasons: Vec<String>,
}

/// Extended metadata for a search result (ERC-8004 aligned).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultMetadata {
    /// URI resolving to the agent registration file.
    #[serde(rename = "agentURI", default, skip_serializing_if = "Option::is_none")]
    pub agent_uri: Option<String>,

    /// Agent image/avatar URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,

    /// Whether the agent is currently active.
    #[serde(default)]
    pub active: bool,

    /// x402 payment support.
    #[serde(rename = "x402Support", default)]
    pub x402_support: bool,

    /// Supported trust models.
    #[serde(default)]
    pub supported_trust: Value,

    /// Full services array (preserves ERC-8004 extensibility).
    #[serde(default)]
    pub services: Value,

    /// Multi-chain registration entries.
    #[serde(default)]
    pub registrations: Value,

    /// Primary service endpoint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,

    /// Provider-computed reputation score.
    #[serde(default)]
    pub reputation_score: f32,

    /// Number of on-chain feedback submissions contributing to the score.
    #[serde(default)]
    pub feedback_count: i64,

    /// Per-wallet feedback breakdown.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub feedback_details: Vec<Value>,

    /// Registration block timestamp (Unix epoch seconds).
    #[serde(default)]
    pub created_at: i64,

    /// ISO 8601 timestamp of last index update.
    #[serde(default)]
    pub updated_at: String,
}

/// Pagination metadata (ERC-8004 standard).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    /// Whether more results are available.
    pub has_more: bool,

    /// Cursor for fetching the next page.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,

    /// Number of results per page.
    pub limit: usize,

    /// Current offset.
    pub offset: usize,
}

/// Service provider info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    /// Provider name.
    pub name: String,
    /// Provider version.
    pub version: String,
}

/// `GET /api/v1/health` response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    /// `"ok"`, `"degraded"`, or `"down"`.
    pub status: String,

    /// ISO 8601 timestamp.
    pub timestamp: String,

    /// Service version.
    pub version: String,

    /// Subsystem health indicators.
    pub services: ServiceHealth,

    /// Seconds since the service started.
    #[serde(default)]
    pub uptime: u64,
}

/// Subsystem health indicators.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceHealth {
    /// Embedding engine status (`"ok"` or `"error"`).
    pub embedding: String,
    /// Vector store status (`"ok"` or `"error"`).
    pub vector_store: String,
}

/// `GET /api/v1/capabilities` response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesResponse {
    /// Service version.
    pub version: String,

    /// API limits.
    pub limits: ApiLimits,

    /// Supported filter field names.
    pub supported_filters: Vec<String>,

    /// Supported filter operators.
    pub supported_operators: Vec<String>,

    /// Feature flags.
    pub features: ApiFeatures,
}

/// API limits advertised via capabilities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiLimits {
    /// Maximum query string length.
    pub max_query_length: usize,
    /// Maximum `limit` value.
    pub max_limit: usize,
    /// Maximum number of filter conditions.
    pub max_filters: usize,
    /// Maximum request body size in bytes.
    pub max_request_size: usize,
}

/// Feature flags (ERC-8004 standard).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiFeatures {
    /// Whether offset-based pagination is supported.
    pub pagination: bool,
    /// Whether cursor-based pagination is supported.
    pub cursor_pagination: bool,
    /// Whether metadata filtering is supported.
    pub metadata_filtering: bool,
    /// Whether `minScore` threshold is supported.
    pub score_threshold: bool,
}

/// Structured error response from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// Human-readable error message.
    pub error: String,

    /// Machine-readable error code.
    pub code: String,

    /// HTTP status code.
    pub status: u16,

    /// Request ID for tracing.
    #[serde(default)]
    pub request_id: String,

    /// ISO 8601 timestamp.
    #[serde(default)]
    pub timestamp: String,
}

const fn default_limit() -> usize {
    10
}

const fn default_include_metadata() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn search_request_builder() {
        let req = SearchRequest::new("DeFi agent")
            .limit(5)
            .min_score(0.3)
            .include_metadata(false);
        assert_eq!(req.query, "DeFi agent");
        assert_eq!(req.limit, 5);
        assert_eq!(req.min_score, Some(0.3));
        assert!(!req.include_metadata);
    }

    #[test]
    fn search_request_serializes_camel_case() {
        let req = SearchRequest::new("test query").limit(3);
        let v = serde_json::to_value(&req).expect("serialize");
        assert_eq!(v["query"], "test query");
        assert_eq!(v["limit"], 3);
        assert!(v.get("includeMetadata").is_some());
        assert!(v.get("include_metadata").is_none());
    }

    #[test]
    fn filters_builder() {
        let f = Filters::new()
            .eq("chainId", json!(8453))
            .eq("active", json!(true))
            .r#in("serviceName", vec![json!("MCP"), json!("A2A")])
            .not_in("chainId", vec![json!(1)])
            .exists("image")
            .not_exists("deprecated");
        assert_eq!(f.count(), 6);
        assert!(!f.is_empty());
    }

    #[test]
    fn filters_serialize_roundtrip() {
        let original = Filters::new()
            .eq("active", json!(true))
            .r#in("chainId", vec![json!(8453), json!(84532)]);
        let json_str = serde_json::to_string(&original).expect("serialize");
        let parsed: Filters = serde_json::from_str(&json_str).expect("deserialize");
        assert_eq!(parsed.equals.len(), 1);
        assert_eq!(parsed.in_.len(), 1);
    }

    #[test]
    fn search_response_deserializes_from_spec_json() {
        let json = json!({
            "query": "test",
            "results": [{
                "rank": 1,
                "agentId": "8453:123",
                "chainId": 8453,
                "name": "TestAgent",
                "description": "A test agent",
                "score": 0.95,
                "metadata": {
                    "agentURI": "ipfs://QmTest",
                    "active": true,
                    "x402Support": true,
                    "supportedTrust": ["reputation"],
                    "services": [{"name": "MCP", "endpoint": "https://mcp.test/"}],
                    "registrations": [{"agentId": 123}],
                    "reputationScore": 0.85,
                    "feedbackCount": 12,
                    "createdAt": 1_704_067_200,
                    "updatedAt": "2025-12-01T00:00:00Z"
                },
                "matchReasons": []
            }],
            "pagination": {
                "hasMore": true,
                "nextCursor": "1",
                "limit": 10,
                "offset": 0
            },
            "requestId": "abc-123",
            "timestamp": "2025-12-01T00:00:00Z",
            "provider": {
                "name": "erc8004-search-service",
                "version": "0.4.0"
            }
        });

        let resp: SearchResponse = serde_json::from_value(json).expect("deserialize");
        assert_eq!(resp.results.len(), 1);
        assert_eq!(resp.results[0].name, "TestAgent");
        assert!(resp.results[0].metadata.is_some());
        let meta = resp.results[0].metadata.as_ref().expect("metadata");
        assert_eq!(meta.agent_uri.as_deref(), Some("ipfs://QmTest"));
        assert!(meta.x402_support);
        assert!(resp.pagination.is_some());
        let pg = resp.pagination.as_ref().expect("pagination");
        assert!(pg.has_more);
        assert_eq!(pg.next_cursor.as_deref(), Some("1"));
    }

    #[test]
    fn health_response_deserializes() {
        let json = json!({
            "status": "ok",
            "timestamp": "2025-12-01T00:00:00Z",
            "version": "0.4.0",
            "services": {
                "embedding": "ok",
                "vectorStore": "ok"
            },
            "uptime": 3600
        });
        let h: HealthResponse = serde_json::from_value(json).expect("deserialize");
        assert_eq!(h.status, "ok");
        assert_eq!(h.services.vector_store, "ok");
    }

    #[test]
    fn error_response_deserializes() {
        let json = json!({
            "error": "query cannot be empty",
            "code": "VALIDATION_ERROR",
            "status": 400,
            "requestId": "abc",
            "timestamp": "2025-12-01T00:00:00Z"
        });
        let e: ErrorResponse = serde_json::from_value(json).expect("deserialize");
        assert_eq!(e.code, "VALIDATION_ERROR");
        assert_eq!(e.status, 400);
    }

    #[test]
    fn empty_filters_not_serialized() {
        let req = SearchRequest::new("test");
        let v = serde_json::to_value(&req).expect("serialize");
        assert!(
            v.get("filters").is_none(),
            "empty filters should be omitted"
        );
        assert!(v.get("cursor").is_none(), "None cursor should be omitted");
        assert!(
            v.get("minScore").is_none(),
            "None minScore should be omitted"
        );
    }

    #[test]
    fn exclude_wallets_clears_include() {
        let req = SearchRequest::new("test")
            .include_wallets(vec!["0xabc".into()])
            .exclude_wallets(vec!["0xdef".into()]);
        assert!(req.reputation_include_wallets.is_none());
        assert!(req.reputation_exclude_wallets.is_some());
    }
}
