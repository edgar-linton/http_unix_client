//! Error types for the unix-http-client crate.
//!
//! This module defines the [`Error`] enum, a unified error type used throughout the client.  
//! All errors produced by this crate will be returned as a variant of [`Error`],  
//! making error handling simple and consistent.

use std::fmt;

use http::StatusCode;
use hyper::ext::ReasonPhrase;
use thiserror::Error;

/// A unified error type for all operations in this crate.
///
/// Most methods return a [`Result<T, Error>`]. This enum wraps errors from
/// underlying libraries and provides an additional variant for HTTP status-based errors.
#[derive(Debug, Error)]
#[error(transparent)]
pub enum Error {
    /// An parse error from the `url` crate.
    UrlParseError(#[from] url::ParseError),
    /// BuilderError
    #[error("Builder error: {0}")]
    BuilderError(#[from] BuilderError),
    /// An error from the `http` crate.
    HttpError(#[from] http::Error),
    /// An error from the `hyper` crate.
    HyperError(#[from] hyper::Error),
    /// An error from serializing or deserializing JSON (available when the `json` feature is enabled).
    #[cfg(feature = "json")]
    #[error("Error decoding response body.")]
    Decode(#[from] serde_json::Error),
    /// An error when constructing a URI.
    InvalidUriParts(#[from] http::uri::InvalidUriParts),
    /// An error from the legacy hyper client utility.
    ClientError(#[from] hyper_util::client::legacy::Error),
    /// Returned when the server responds with an error status code.]
    StatusError(#[from] StatusError),
}

impl Error {
    /// Returns true if the error is from a type Builder.
    pub fn is_builder(&self) -> bool {
        matches!(self, Self::BuilderError(..))
    }

    /// Returns true if the error is from `Response::error_for_status`.
    pub fn is_status(&self) -> bool {
        matches!(self, Self::StatusError { .. })
    }

    /// Returns true if the error is related to connect
    pub fn is_connect(&self) -> bool {
        matches!(self, Self::ClientError(err) if err.is_connect())
    }

    /// Returns the status code, if the error was generated from a response.
    pub fn status(&self) -> Option<StatusCode> {
        match self {
            Self::StatusError(err) => Some(err.code),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum BuilderError {
    /// An parse error from the `url` crate.
    UrlParse(#[from] url::ParseError),
    /// An error from the `http` crate.
    Http(#[from] http::Error),
    /// An error from serializing URL query parameters.
    SerializeUrl(#[from] serde_urlencoded::ser::Error),
    /// An error from serializing or deserializing JSON (available when the `json` feature is enabled).
    #[cfg(feature = "json")]
    SerializeJson(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct StatusError {
    /// The HTTP status code.
    code: StatusCode,
    /// An optional reason phrase for the error.
    reason: Option<ReasonPhrase>,
}

impl StatusError {
    pub(crate) fn new(code: StatusCode, reason: Option<ReasonPhrase>) -> Self {
        Self { code, reason }
    }
}

impl fmt::Display for StatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code;
        let prefix = if self.code.is_client_error() {
            "HTTP status client error"
        } else {
            "HTTP status server error"
        };
        match self
            .reason
            .as_ref()
            .and_then(|r| std::str::from_utf8(r.as_bytes()).ok())
        {
            Some(reason) => {
                write!(f, "{prefix} ({code} {reason})")
            }
            None => write!(f, "{prefix} ({code})"),
        }
    }
}

impl std::error::Error for StatusError {}

/// A result type alias for this crate.
pub type Result<T> = std::result::Result<T, Error>;
