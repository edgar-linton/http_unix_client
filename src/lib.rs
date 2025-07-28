#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(test, deny(warnings))]

//! # http-unix-client
//!
//! An HTTP client for interacting with HTTP servers over Unix sockets.
//! The crate mimics the architecture of the [reqwest](https://docs.rs/reqwest/latest/reqwest/) crate.
//! The [`Client`] is asynchronous (requiring Tokio).
//!
//! ## Supported Platforms
//!
//! This crate is only supported on Unix-like systems (Linux, macOS, BSD, etc.) because it relies on Unix domain sockets.
//!
//! ## Examples
//!
//! ### Making a GET request
//!
//! For a single request, you can use the [`get`] shortcut method.
//!
//! ```rust
//! # use http_unix_client::{Client, Error, get};
//! #
//! # async fn run() -> Result<(), Error> {
//! let body = get("/tmp/my.socket", "/health")
//!     .await?
//!     .text()
//!     .await?;
//!
//! println!("body = {body:?}");
//! #   Ok(())
//! # }
//! ```
//!
//! **NOTE**: If you plan to perform multiple requests, it is best to create a
//! [`Client`] and reuse it, taking advantage of keep-alive connection
//! pooling.
//!
//! ## Making POST requests (or setting request bodies)
//!
//! There are several ways you can set the body of a request. The basic one is
//! by using the `body()` method of a [`RequestBuilder`]. This lets you set the
//! exact raw bytes of what the body should be. It accepts various types,
//! including `String` and `Vec<u8>`. If you wish to pass a custom
//! type, you can use the `reqwest::Body` constructors.
//!
//! ```rust
//! # use http_unix_client::{Client, Error};
//!
//! # async fn run() -> Result<(), Error> {
//! let client = Client::new();
//! let res = client.post("/tmp/my.socket", "/health")
//!     .body("the exact body that is sent")
//!     .send()
//!     .await?;
//! #   Ok(())
//! # }
//! ```
//!
//! ### Forms
//!
//! It's very common to want to send form data in a request body. This can be
//! done with any type that can be serialized into form data.
//!
//! This can be an array of tuples, or a `HashMap`, or a custom type that
//! implements [`Serialize`][serde].
//!
//! ```no_run
//! # use http_unix_client::{Client, Error};
//! #
//! # async fn run() -> Result<(), Error> {
//! // This will POST a body of `foo=bar&baz=quux`
//! let params = [("foo", "bar"), ("baz", "quux")];
//! let client = Client::new();
//! let res = client.post("/tmp/my.socket", "/health")
//!     .form(&params)
//!     .send()
//!     .await?;
//!     Ok(())
//! }
//! ```
//!
//! ### JSON
//!
//! There is also a `json` method helper on the [`RequestBuilder`] that works in
//! a similar fashion the `form` method. It can take any value that can be
//! serialized into JSON. The feature `json` is required.
//!
//! ```rust
//! # use http_unix_client::{Client, Error};
//! # use std::collections::HashMap;
//! #
//! # #[cfg(feature = "json")]
//! # async fn run() -> Result<(), Error> {
//! // This will POST a body of `{"lang":"rust","body":"json"}`
//! let mut map = HashMap::new();
//! map.insert("lang", "rust");
//! map.insert("body", "json");
//! let client = Client::new();
//! let res = client.post("/tmp/my.socket", "/health")
//!     .json(&map)
//!     .send()
//!     .await?;
//! #   Ok(())
//! # }
//! ```

mod body;
mod client;
mod error;
mod request;
mod response;
mod unix_url;

pub use body::Body;
pub use client::Client;
#[cfg(feature = "cookies")]
pub use cookie::Cookie;
pub use error::{Error, Result};
pub use http::{Extensions, Method, StatusCode, Uri, Version, header};
pub use request::{Request, RequestBuilder};
pub use response::Response;
pub use unix_url::UnixUrl;
pub use url::Url;

/// Shortcut method to quickly make a `GET` request.
///
/// See also the methods on the [`Response`]
/// type.
///
/// **NOTE**: This function creates a new internal `Client` on each call,
/// and so should not be used if making many requests. Create a
/// [`Client`] instead.
///
/// # Examples
///
/// ```rust
/// # use http_unix_client::Error;
///
/// # async fn run() -> Result<(), Error> {
/// let body = http_unix_client::get("/tmp/my.socket", "/").await?
///     .text().await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// This function fails if:
///
/// - supplied `path` cannot be parsed to an url
/// - there was an error while sending request
pub async fn get<P>(socket: P, path: &str) -> crate::Result<Response>
where
    P: AsRef<std::path::Path>,
{
    Client::new().get(socket, path).send().await
}
