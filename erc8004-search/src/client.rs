//! HTTP client for the ERC-8004 Semantic Search Service.
//!
//! [`SearchClient`] wraps `reqwest` with automatic x402 payment handling
//! via [`r402_http::client::X402Client`] middleware.
//!
//! The SDK defaults to the QNTX-hosted endpoint ([`DEFAULT_BASE_URL`]),
//! so callers can start querying with zero configuration. Override via
//! [`SearchClientBuilder::base_url`] when self-hosting.

use reqwest::Client;
use reqwest_middleware::ClientWithMiddleware;
use url::Url;

use crate::error::{Error, Result};
use crate::types::{CapabilitiesResponse, HealthResponse, SearchRequest, SearchResponse};

/// Built-in default base URL for the QNTX-hosted ERC-8004 search service.
///
/// Used by [`SearchClient::new`] and [`SearchClient::builder`] unless
/// overridden with [`SearchClientBuilder::base_url`].
pub const DEFAULT_BASE_URL: &str = "https://search.qntx.fun";

/// HTTP client for the ERC-8004 Semantic Search Service.
///
/// Handles request construction, JSON serialization, error mapping,
/// and optional x402 payment middleware.
///
/// # Construction
///
/// | Method                      | Use case                                  |
/// |-----------------------------|-------------------------------------------|
/// | [`SearchClient::new`]       | Quick start, no payment, default endpoint |
/// | [`SearchClient::builder`]   | Payment, custom URL, timeouts, etc.       |
///
/// # Example
///
/// ```rust,ignore
/// use erc8004_search::{SearchClient, SearchRequest, Filters};
/// use serde_json::json;
///
/// // Zero-config — uses the built-in QNTX endpoint.
/// let client = SearchClient::new();
/// let resp = client.search("DeFi lending").await?;
///
/// // Fully configured via builder.
/// let req = SearchRequest::new("MCP tool server")
///     .limit(5)
///     .min_score(0.5)
///     .filters(Filters::new().eq("chainId", json!(8453)));
/// let resp = client.execute(req).await?;
/// ```
#[derive(Debug, Clone)]
pub struct SearchClient {
    http: ClientWithMiddleware,
    base_url: Url,
}

impl SearchClient {
    /// Create a client targeting the built-in QNTX endpoint
    /// ([`DEFAULT_BASE_URL`]) without x402 payment middleware.
    ///
    /// This is the fastest way to start querying.
    /// For payment or custom endpoints, use [`SearchClient::builder`].
    #[must_use]
    pub fn new() -> Self {
        SearchClientBuilder::default()
            .build()
            .expect("built-in DEFAULT_BASE_URL is always valid")
    }

    /// Create a [`SearchClientBuilder`] pre-configured with the built-in
    /// QNTX endpoint ([`DEFAULT_BASE_URL`]).
    ///
    /// Use this to attach x402 signers, set timeouts, or override the
    /// base URL via [`SearchClientBuilder::base_url`].
    #[must_use]
    pub fn builder() -> SearchClientBuilder {
        SearchClientBuilder::default()
    }

    /// `GET /api/v1/health` -- Check service health.
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP transport failure or unexpected response.
    pub async fn health(&self) -> Result<HealthResponse> {
        let url = self.url("/api/v1/health");
        tracing::debug!(%url, "GET health");

        let resp = self.http.get(url).send().await?;
        // Health returns 200 or 503, both have valid JSON bodies.
        Ok(resp.json().await?)
    }

    /// `GET /api/v1/capabilities` -- Discover service capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP transport failure or unexpected response.
    pub async fn capabilities(&self) -> Result<CapabilitiesResponse> {
        let url = self.url("/api/v1/capabilities");
        tracing::debug!(%url, "GET capabilities");

        let resp = self.http.get(url).send().await?;
        self.json_or_error(resp).await
    }

    /// `POST /api/v1/search` -- Semantic search with a query string.
    ///
    /// Convenience method that creates a [`SearchRequest`] with defaults.
    /// For full control, use [`execute`](Self::execute).
    ///
    /// # Errors
    ///
    /// Returns an error on validation, payment, transport, or server failure.
    pub async fn search(&self, query: impl Into<String>) -> Result<SearchResponse> {
        self.execute(SearchRequest::new(query)).await
    }

    /// `POST /api/v1/search` -- Execute a fully-configured search request.
    ///
    /// The x402 payment middleware (if configured) automatically intercepts
    /// `402 Payment Required` responses, signs a payment, and retries.
    ///
    /// # Errors
    ///
    /// Returns an error on validation, payment, transport, or server failure.
    pub async fn execute(&self, request: SearchRequest) -> Result<SearchResponse> {
        let url = self.url("/api/v1/search");
        tracing::debug!(%url, query = %request.query, limit = request.limit, "POST search");

        let resp = self.http.post(url).json(&request).send().await?;

        // If we still get 402 after middleware, payment was not handled.
        if resp.status() == reqwest::StatusCode::PAYMENT_REQUIRED {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::PaymentRequired(body));
        }

        self.json_or_error(resp).await
    }

    /// Fetch all pages of results for a query, collecting into a single vec.
    ///
    /// Iterates using cursor-based pagination until `hasMore` is `false`
    /// or `max_pages` is reached.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered during pagination.
    pub async fn search_all(
        &self,
        query: impl Into<String>,
        max_pages: usize,
    ) -> Result<Vec<crate::types::SearchResultItem>> {
        let query = query.into();
        let mut all_results = Vec::new();
        let mut cursor: Option<String> = None;

        for _ in 0..max_pages {
            let mut req = SearchRequest::new(query.clone());
            if let Some(c) = cursor.take() {
                req = req.cursor(c);
            }

            let resp = self.execute(req).await?;
            let has_more = resp.pagination.as_ref().is_some_and(|p| p.has_more);
            cursor = resp.pagination.as_ref().and_then(|p| p.next_cursor.clone());

            all_results.extend(resp.results);

            if !has_more || cursor.is_none() {
                break;
            }
        }

        Ok(all_results)
    }

    /// Construct the full URL for an API path.
    fn url(&self, path: &str) -> String {
        let base = self.base_url.as_str().trim_end_matches('/');
        format!("{base}{path}")
    }

    /// Parse a JSON response body, mapping non-2xx status to [`Error::Api`].
    async fn json_or_error<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T> {
        if resp.status().is_success() {
            return Ok(resp.json().await?);
        }

        // Try to parse the structured error body.
        let status = resp.status().as_u16();
        let api_err = resp
            .json::<crate::types::ErrorResponse>()
            .await
            .map_or_else(
                |_| Error::Api {
                    status,
                    message: "unexpected error response".into(),
                    code: "UNKNOWN".into(),
                    request_id: String::new(),
                },
                Error::from_error_response,
            );
        Err(api_err)
    }
}

/// Builder for constructing a [`SearchClient`] with x402 payment and custom
/// HTTP settings.
///
/// Defaults to [`DEFAULT_BASE_URL`]. Override with [`base_url`](Self::base_url).
///
/// # Example
///
/// ```rust,ignore
/// use erc8004_search::SearchClient;
/// use alloy_signer_local::PrivateKeySigner;
///
/// let signer: PrivateKeySigner = "0x...".parse()?;
///
/// // Default endpoint + payment.
/// let client = SearchClient::builder()
///     .evm_signer(signer)
///     .build()?;
///
/// // Custom endpoint + payment + timeout.
/// let client = SearchClient::builder()
///     .base_url("https://custom.example.com")
///     .evm_signer(signer)
///     .timeout(std::time::Duration::from_secs(60))
///     .build()?;
/// ```
#[allow(missing_debug_implementations)]
pub struct SearchClientBuilder {
    base_url: String,
    reqwest_builder: reqwest::ClientBuilder,
    x402: r402_http::client::X402Client<r402::scheme::FirstMatch>,
    has_payment: bool,
}

impl Default for SearchClientBuilder {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_owned(),
            reqwest_builder: Client::builder()
                .pool_max_idle_per_host(4)
                .tcp_keepalive(std::time::Duration::from_secs(30)),
            x402: r402_http::client::X402Client::new(),
            has_payment: false,
        }
    }
}

impl SearchClientBuilder {
    /// Override the base URL (default: [`DEFAULT_BASE_URL`]).
    ///
    /// Only needed when running your own ERC-8004 service instance.
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Register an EVM signer for automatic x402 payment on EIP-155 chains.
    ///
    /// The signer's private key is used to sign ERC-3009 / Permit2
    /// payment authorizations when the service returns `402 Payment Required`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use alloy_signer_local::PrivateKeySigner;
    ///
    /// let signer: PrivateKeySigner = "0x...".parse()?;
    /// let client = SearchClient::builder()
    ///     .evm_signer(signer)
    ///     .build()?;
    /// ```
    #[cfg(feature = "evm")]
    #[must_use]
    pub fn evm_signer(
        mut self,
        signer: impl r402_evm::exact::client::SignerLike + Clone + 'static,
    ) -> Self {
        self.x402 = self.x402.register(r402_evm::Eip155ExactClient::new(signer));
        self.has_payment = true;
        self
    }

    /// Register an EVM signer wrapped in `Arc` (for shared ownership).
    #[cfg(feature = "evm")]
    #[must_use]
    pub fn evm_signer_arc(
        mut self,
        signer: std::sync::Arc<impl r402_evm::exact::client::SignerLike + 'static>,
    ) -> Self {
        self.x402 = self.x402.register(r402_evm::Eip155ExactClient::new(signer));
        self.has_payment = true;
        self
    }

    /// Register a raw x402 scheme client for custom payment schemes.
    ///
    /// Use this for Solana or other non-EVM payment schemes.
    #[must_use]
    pub fn register_scheme<S>(mut self, scheme: S) -> Self
    where
        S: r402::scheme::SchemeClient + 'static,
    {
        self.x402 = self.x402.register(scheme);
        self.has_payment = true;
        self
    }

    /// Set the HTTP request timeout.
    #[must_use]
    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.reqwest_builder = self.reqwest_builder.timeout(timeout);
        self
    }

    /// Set a custom `User-Agent` header.
    #[must_use]
    pub fn user_agent(mut self, ua: impl AsRef<str>) -> Self {
        self.reqwest_builder = self.reqwest_builder.user_agent(ua.as_ref());
        self
    }

    /// Build the [`SearchClient`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Config`] if the base URL is invalid or the HTTP
    /// client cannot be constructed.
    pub fn build(self) -> Result<SearchClient> {
        let base_url = Url::parse(&self.base_url)
            .map_err(|e| Error::Config(format!("invalid base URL '{}': {e}", self.base_url)))?;

        let reqwest_client = self
            .reqwest_builder
            .build()
            .map_err(|e| Error::Config(format!("failed to build HTTP client: {e}")))?;

        let http = if self.has_payment {
            self.x402.wrap(reqwest_client)
        } else {
            reqwest_middleware::ClientBuilder::new(reqwest_client).build()
        };

        Ok(SearchClient { http, base_url })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uses_default_url() {
        let client = SearchClient::new();
        assert_eq!(
            client.base_url.as_str().trim_end_matches('/'),
            DEFAULT_BASE_URL
        );
    }

    #[test]
    fn builder_uses_default_url() {
        let client = SearchClient::builder().build().expect("valid");
        assert_eq!(
            client.base_url.as_str().trim_end_matches('/'),
            DEFAULT_BASE_URL
        );
    }

    #[test]
    fn builder_base_url_override() {
        let client = SearchClient::builder()
            .base_url("http://127.0.0.1:9090")
            .build()
            .expect("valid");
        assert_eq!(
            client.url("/api/v1/search"),
            "http://127.0.0.1:9090/api/v1/search"
        );
    }

    #[test]
    fn builder_rejects_invalid_url() {
        let result = SearchClient::builder()
            .base_url("not a url ://")
            .build();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Config(_)));
    }

    #[test]
    fn url_construction() {
        let client = SearchClient::builder()
            .base_url("http://localhost:8080")
            .build()
            .expect("valid url");
        assert_eq!(
            client.url("/api/v1/search"),
            "http://localhost:8080/api/v1/search"
        );
    }

    #[test]
    fn url_strips_trailing_slash() {
        let client = SearchClient::builder()
            .base_url("http://localhost:8080/")
            .build()
            .expect("valid url");
        assert_eq!(
            client.url("/api/v1/health"),
            "http://localhost:8080/api/v1/health"
        );
    }
}
