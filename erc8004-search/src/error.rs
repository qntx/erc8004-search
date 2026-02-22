//! Error types for the ERC-8004 Search SDK.
//!
//! [`Error`] covers all failure modes a caller may encounter:
//! HTTP transport, x402 payment, API-level validation, and
//! response deserialization.

use crate::types::ErrorResponse;

/// Convenience alias used throughout the SDK.
pub type Result<T> = std::result::Result<T, Error>;

/// All errors that can occur when using the SDK.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The service returned a structured API error (4xx / 5xx).
    #[error("API error ({status}): {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Human-readable error message.
        message: String,
        /// Machine-readable error code (e.g. `VALIDATION_ERROR`).
        code: String,
        /// Request ID for tracing.
        request_id: String,
    },

    /// The service returned `402 Payment Required` but x402
    /// payment middleware was unable to handle it automatically.
    #[error("payment required: {0}")]
    PaymentRequired(String),

    /// HTTP transport error (connection, timeout, DNS, etc.).
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest_middleware::Error),

    /// HTTP transport error from raw reqwest (e.g. `.json()` deserialization).
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// Failed to deserialize a response body.
    #[error("deserialization error: {0}")]
    Deserialization(#[from] serde_json::Error),

    /// Invalid configuration (e.g. malformed base URL).
    #[error("invalid configuration: {0}")]
    Config(String),
}

impl Error {
    /// Create an [`Error::Api`] from a parsed [`ErrorResponse`].
    pub(crate) fn from_error_response(resp: ErrorResponse) -> Self {
        Self::Api {
            status: resp.status,
            message: resp.error,
            code: resp.code,
            request_id: resp.request_id,
        }
    }

    /// Returns `true` if this is a validation error (HTTP 400).
    #[must_use]
    pub fn is_validation(&self) -> bool {
        matches!(self, Self::Api { code, .. } if code == "VALIDATION_ERROR")
    }

    /// Returns `true` if this is a payment-required error (HTTP 402).
    #[must_use]
    pub const fn is_payment_required(&self) -> bool {
        matches!(self, Self::PaymentRequired(_))
    }

    /// Returns `true` if this is a rate-limit error (HTTP 429).
    #[must_use]
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Self::Api { code, .. } if code == "RATE_LIMIT_EXCEEDED")
    }

    /// Returns the request ID if available.
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::Api { request_id, .. } if !request_id.is_empty() => Some(request_id),
            _ => None,
        }
    }
}
