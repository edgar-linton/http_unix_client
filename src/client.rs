use http_body_util::Full;
use hyper::body::Bytes;
use hyper_util::client::legacy::Client as HyperClient;
use hyperlocal::{UnixClientExt, UnixConnector};
use std::path::Path;

use crate::{Error, Method, Request, RequestBuilder, Response, UnixUrl};

/// An asynchronous `Client` to make Requests over Unix socket with.
#[derive(Debug, Clone)]
pub struct Client {
    inner: HyperClient<UnixConnector, Full<Bytes>>,
}

impl Client {
    /// Creates a new [`Client`] instance for making HTTP requests over Unix sockets.
    pub fn new() -> Self {
        Self {
            inner: HyperClient::unix(),
        }
    }

    /// Start building a Request with the Method and Url.
    ///
    /// Returns a RequestBuilder, which will allow setting headers and the request body before sending.
    ///
    /// # Errors
    ///
    /// This method fails whenever the supplied socket and path cannot parsed to a [`UnixUrl`].
    pub fn request<P>(&self, method: Method, socket: P, path: &str) -> RequestBuilder
    where
        P: AsRef<Path>,
    {
        let req = UnixUrl::new(socket, path)
            .map(|url| Request::new(method, url))
            .map_err(Error::from);

        RequestBuilder::new(self.clone(), req)
    }

    /// Creates a new HTTP GET request for the given socket and path.
    pub fn get<P>(&self, socket: P, path: &str) -> RequestBuilder
    where
        P: AsRef<Path>,
    {
        self.request(Method::GET, socket, path)
    }

    /// Creates a new HTTP POST request for the given socket and path.
    pub fn post<P>(&self, socket: P, path: &str) -> RequestBuilder
    where
        P: AsRef<Path>,
    {
        self.request(Method::POST, socket, path)
    }

    /// Creates a new HTTP PUT request for the given socket and path.
    pub fn put<P>(&self, socket: P, path: &str) -> RequestBuilder
    where
        P: AsRef<Path>,
    {
        self.request(Method::PUT, socket, path)
    }

    /// Creates a new HTTP PATCH request for the given socket and path.
    pub fn patch<P>(&self, socket: P, path: &str) -> RequestBuilder
    where
        P: AsRef<Path>,
    {
        self.request(Method::PATCH, socket, path)
    }

    /// Creates a new HTTP DELETE request for the given socket and path.
    pub fn delete<P>(&self, socket: P, path: &str) -> RequestBuilder
    where
        P: AsRef<Path>,
    {
        self.request(Method::DELETE, socket, path)
    }

    /// Creates a new HTTP HEAD request for the given socket and path.
    pub fn head<P>(&self, socket: P, path: &str) -> RequestBuilder
    where
        P: AsRef<Path>,
    {
        self.request(Method::HEAD, socket, path)
    }

    /// Executes a [`Request`].
    ///
    /// A `Request` can be built manually with `Request::new()` or obtained
    /// from a RequestBuilder with `RequestBuilder::build()`.
    ///
    /// You should prefer to use the `RequestBuilder` and
    /// `RequestBuilder::send()`.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending request,
    pub async fn execute(&self, request: Request) -> Result<Response, Error> {
        let (method, url, headers, body, version, extensions) = request.pieces();
        let body = match body {
            Some(body) => Full::new(body.bytes().clone()),
            None => Full::new(Bytes::new()),
        };
        let mut builder = http::Request::builder()
            .method(method)
            .uri(url.clone())
            .version(version);
        if let Some(builder_headers) = builder.headers_mut() {
            builder_headers.extend(headers);
        }
        if let Some(builder_extensions) = builder.extensions_mut() {
            builder_extensions.extend(extensions);
        }
        let req = builder.body(body)?;
        let resp = self.inner.request(req).await?;

        Ok(Response::new(resp, url))
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
