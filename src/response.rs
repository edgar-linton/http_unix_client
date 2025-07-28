use bytes::Bytes;
#[cfg(feature = "cookies")]
use cookie::Cookie;
use encoding_rs::{Encoding, UTF_8};
#[cfg(feature = "cookies")]
use http::header::SET_COOKIE;
use http::{HeaderMap, Version};
use http_body::Body;
use http_body_util::BodyExt;
use hyper::{body::Incoming, ext::ReasonPhrase};
use hyper_util::client::legacy::connect::HttpInfo;
use mime::Mime;
#[cfg(feature = "json")]
use serde::de::DeserializeOwned;
use std::net::SocketAddr;

use crate::error::StatusError;
use crate::{Result, StatusCode, UnixUrl};

/// A Response to a submitted `Request`.
#[derive(Debug)]
pub struct Response {
    response: http::Response<Incoming>,
    url: Box<UnixUrl>,
}

impl Response {
    pub(super) fn new(response: http::Response<Incoming>, url: UnixUrl) -> Self {
        Self {
            response,
            url: Box::new(url),
        }
    }

    /// Get the `StatusCode` of this `Response`.
    pub fn status(&self) -> StatusCode {
        self.response.status()
    }

    /// Get the HTTP `Version` of this `Response`.
    #[inline]
    pub fn version(&self) -> Version {
        self.response.version()
    }

    /// Get the `Headers` of this `Response`.
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        self.response.headers()
    }

    /// Get a mutable reference to the `Headers` of this `Response`.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        self.response.headers_mut()
    }

    /// Get the content length of the response, if it is known.
    ///
    /// This value does not directly represents the value of the `Content-Length`
    /// header, but rather the size of the response's body. To read the header's
    /// value, please use the [`Response::headers`] method instead.
    ///
    /// Reasons it may not be known:
    ///
    /// - The response does not include a body (e.g. it responds to a `HEAD`
    ///   request).
    /// - The response is gzipped and automatically decoded (thus changing the
    ///   actual decoded length).
    pub fn content_length(&self) -> Option<u64> {
        self.response.body().size_hint().exact()
    }

    /// Retrieve the cookies contained in the response.
    ///
    /// Note that invalid 'Set-Cookie' headers will be ignored.
    ///
    /// # Optional
    ///
    /// This requires the optional `cookies` feature to be enabled.
    #[cfg(feature = "cookies")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cookies")))]
    pub fn cookies<'a>(&'a self) -> impl Iterator<Item = cookie::Cookie<'a>> + 'a {
        self.response
            .headers()
            .get_all(SET_COOKIE)
            .into_iter()
            .filter_map(|v| v.to_str().ok())
            .filter_map(|v| v.parse::<Cookie>().ok())
    }

    /// Get the final `Url` of this `Response`.
    #[inline]
    pub fn url(&self) -> &UnixUrl {
        &self.url
    }

    /// Get the remote address used to get this `Response`.
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.response
            .extensions()
            .get::<HttpInfo>()
            .map(|info| info.remote_addr())
    }

    /// Returns a reference to the associated extensions.
    pub fn extensions(&self) -> &http::Extensions {
        self.response.extensions()
    }

    /// Returns a mutable reference to the associated extensions.
    pub fn extensions_mut(&mut self) -> &mut http::Extensions {
        self.response.extensions_mut()
    }

    /// Get the full response text.
    ///
    /// This method decodes the response body with BOM sniffing
    /// and with malformed sequences replaced with the
    /// [`char::REPLACEMENT_CHARACTER`].
    /// Encoding is determined from the `charset` parameter of `Content-Type` header,
    /// and defaults to `utf-8` if not presented.
    ///
    /// Note that the BOM is stripped from the returned String.
    ///
    /// # Note
    ///
    /// If the `charset` feature is disabled the method will only attempt to decode the
    /// response as UTF-8, regardless of the given `Content-Type`
    ///
    /// # Example
    ///
    /// ```
    /// # use http_unix_client::Error;
    ///
    /// # async fn run() -> Result<(), Error> {
    /// let content = http_unix_client::get("/tmp/my.socket", "/range/26")
    ///     .await?
    ///     .text()
    ///     .await?;
    ///
    /// println!("text: {content:?}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn text(self) -> crate::Result<String> {
        #[cfg(feature = "charset")]
        {
            self.text_with_charset("utf-8").await
        }

        #[cfg(not(feature = "charset"))]
        {
            let full = self.bytes().await?;
            let text = String::from_utf8_lossy(&full);
            Ok(text.into_owned())
        }
    }

    /// Get the full response text given a specific encoding.
    ///
    /// This method decodes the response body with BOM sniffing
    /// and with malformed sequences replaced with the [`char::REPLACEMENT_CHARACTER`].
    /// You can provide a default encoding for decoding the raw message, while the
    /// `charset` parameter of `Content-Type` header is still prioritized. For more information
    /// about the possible encoding name, please go to [`encoding_rs`] docs.
    ///
    /// Note that the BOM is stripped from the returned String.
    ///
    /// [`encoding_rs`]: https://docs.rs/encoding_rs/0.8/encoding_rs/#relationship-with-windows-code-pages
    ///
    /// # Optional
    ///
    /// This requires the optional `encoding_rs` feature enabled.
    ///
    /// # Example
    ///
    /// ```
    /// # use http_unix_client::Error;
    ///
    /// # async fn run() -> Result<(), Error> {
    /// let content = http_unix_client::get("/tmp/my.socket", "/range/26")
    ///     .await?
    ///     .text_with_charset("utf-8")
    ///     .await?;
    ///
    /// println!("text: {content:?}");
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "charset")]
    #[cfg_attr(docsrs, doc(cfg(feature = "charset")))]
    pub async fn text_with_charset(self, default_encoding: &str) -> crate::Result<String> {
        let content_type = self
            .headers()
            .get(crate::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<Mime>().ok());
        let encoding_name = content_type
            .as_ref()
            .and_then(|mime| mime.get_param("charset").map(|charset| charset.as_str()))
            .unwrap_or(default_encoding);
        let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(UTF_8);

        let full = self.bytes().await?;

        let (text, _, _) = encoding.decode(&full);
        Ok(text.into_owned())
    }

    /// Try to deserialize the response body as JSON.
    ///
    /// # Optional
    ///
    /// This requires the optional `json` feature enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http_unix_client::Error;
    /// # use serde::Deserialize;
    /// #
    /// // This `derive` requires the `serde` dependency.
    /// #[derive(Deserialize)]
    /// struct Ip {
    ///     origin: String,
    /// }
    ///
    /// # async fn run() -> Result<(), Error> {
    /// let ip = http_unix_client::get("/tmp/my.socket", "/ip")
    ///     .await?
    ///     .json::<Ip>()
    ///     .await?;
    ///
    /// println!("ip: {}", ip.origin);
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() { }
    /// ```
    ///
    /// # Errors
    ///
    /// This method fails whenever the response body is not in JSON format,
    /// or it cannot be properly deserialized to target type `T`. For more
    /// details please see [`serde_json::from_reader`].
    ///
    /// [`serde_json::from_reader`]: https://docs.serde.rs/serde_json/fn.from_reader.html
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub async fn json<T: DeserializeOwned>(self) -> crate::Result<T> {
        let full = self.bytes().await?;
        let json = serde_json::from_slice(&full)?;
        Ok(json)
    }

    /// Get the full response body as `Bytes`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http_unix_client::Error;
    ///
    /// # async fn run() -> Result<(), Error> {
    /// let bytes = http_unix_client::get("/tmp/my.socket", "/pi")
    ///     .await?
    ///     .bytes()
    ///     .await?;
    ///
    /// println!("bytes: {bytes:?}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bytes(self) -> crate::Result<Bytes> {
        let test = self.response.into_body().collect().await?;
        Ok(test.to_bytes())
    }

    /// Stream a chunk of the response body.
    ///
    /// When the response body has been exhausted, this will return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use http_unix_client::Error;
    ///
    /// # async fn run() -> Result<(), Error> {
    /// let mut res = http_unix_client::get("/tmp/my.socket", "/").await?;
    ///
    /// while let Some(chunk) = res.chunk().await? {
    ///     println!("Chunk: {chunk:?}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chunk(&mut self) -> Result<Option<Bytes>> {
        if let Some(res) = self.response.body_mut().frame().await {
            let frame = res?;
            if let Ok(chunk) = frame.into_data() {
                return Ok(Some(chunk));
            }
        }
        Ok(None)
    }

    /// Turn a response into an error if the server returned an error.
    ///
    /// # Example
    ///
    /// ```
    /// # use http_unix_client::Response;
    /// fn on_response(res: Response) {
    ///     match res.error_for_status() {
    ///         Ok(_res) => (),
    ///         Err(err) => {
    ///             // asserting a 400 as an example
    ///             // it could be any status between 400...599
    ///             assert_eq!(
    ///                 err.status(),
    ///                 Some(http_unix_client::StatusCode::BAD_REQUEST)
    ///             );
    ///         }
    ///     }
    /// }
    /// # fn main() {}
    /// ```
    pub fn error_for_status(self) -> crate::Result<Self> {
        let status = self.response.status();
        if status.is_client_error() || status.is_server_error() {
            let reason = self.response.extensions().get::<ReasonPhrase>().cloned();
            Err(StatusError::new(status, reason).into())
        } else {
            Ok(self)
        }
    }

    /// Turn a reference to a response into an error if the server returned an error.
    ///
    /// # Example
    ///
    /// ```
    /// # use http_unix_client::Response;
    /// fn on_response(res: &Response) {
    ///     match res.error_for_status_ref() {
    ///         Ok(_res) => (),
    ///         Err(err) => {
    ///             // asserting a 400 as an example
    ///             // it could be any status between 400...599
    ///             assert_eq!(
    ///                 err.status(),
    ///                 Some(http_unix_client::StatusCode::BAD_REQUEST)
    ///             );
    ///         }
    ///     }
    /// }
    /// # fn main() {}
    /// ```
    pub fn error_for_status_ref(&self) -> Result<&Self> {
        let status = self.response.status();
        if status.is_client_error() || status.is_server_error() {
            let reason = self.response.extensions().get::<ReasonPhrase>().cloned();
            Err(StatusError::new(status, reason).into())
        } else {
            Ok(self)
        }
    }
}
