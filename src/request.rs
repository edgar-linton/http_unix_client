use base64::{Engine, prelude::BASE64_STANDARD};
use core::fmt;
use serde::Serialize;

use crate::{
    Body, Client, Extensions, Method, Response, Result, UnixUrl, Version,
    error::BuilderError,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue},
};

/// A request which can be executed with `Client::execute()`.
#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    url: UnixUrl,
    headers: HeaderMap,
    body: Option<Body>,
    version: Version,
    extensions: Extensions,
}

impl Request {
    /// Constructs a new request.
    #[inline]
    pub(super) fn new(method: Method, url: UnixUrl) -> Self {
        Request {
            method,
            url,
            headers: HeaderMap::new(),
            body: None,
            version: Version::default(),
            extensions: Extensions::new(),
        }
    }

    /// Get the method.
    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get a mutable reference to the method.
    #[inline]
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }

    /// Get the url.
    #[inline]
    pub fn url(&self) -> &UnixUrl {
        &self.url
    }

    /// Get a mutable reference to the url.
    #[inline]
    pub fn url_mut(&mut self) -> &mut UnixUrl {
        &mut self.url
    }

    /// Get the headers.
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a mutable reference to the headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Get the body.
    #[inline]
    pub fn body(&self) -> Option<&Body> {
        self.body.as_ref()
    }

    /// Get a mutable reference to the body.
    #[inline]
    pub fn body_mut(&mut self) -> &mut Option<Body> {
        &mut self.body
    }

    /// Get the extensions.
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    /// Get a mutable reference to the extensions.
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    /// Get the http version.
    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get a mutable reference to the http version.
    #[inline]
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.version
    }

    pub(super) fn pieces(
        self,
    ) -> (
        Method,
        UnixUrl,
        HeaderMap,
        Option<Body>,
        Version,
        Extensions,
    ) {
        (
            self.method,
            self.url,
            self.headers,
            self.body,
            self.version,
            self.extensions,
        )
    }
}

/// A builder to construct the properties of a `Request`.
///
/// To construct a `RequestBuilder`, refer to the `Client` documentation.
#[must_use = "RequestBuilder does nothing until you 'send' it"]
#[derive(Debug)]
pub struct RequestBuilder {
    client: Client,
    request: Result<Request>,
}

impl RequestBuilder {
    pub(super) fn new(client: Client, request: Result<Request>) -> Self {
        Self { client, request }
    }

    /// Assemble a builder starting from an existing [`Client`] and a [`Request`].
    pub fn from_parts(client: Client, request: Request) -> RequestBuilder {
        RequestBuilder {
            client,
            request: crate::Result::Ok(request),
        }
    }

    /// Adds a [`Header`][crate] to the request.
    pub fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.header_sensitive(key, value, false)
    }

    /// Add a `Header` to this Request with ability to define if `header_value` is sensitive.
    fn header_sensitive<K, V>(mut self, key: K, value: V, sensitive: bool) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        if let Ok(ref mut req) = self.request {
            match <HeaderName as TryFrom<K>>::try_from(key) {
                Ok(key) => match <HeaderValue as TryFrom<V>>::try_from(value) {
                    Ok(mut value) => {
                        if sensitive {
                            value.set_sensitive(true);
                        }
                        req.headers_mut().append(key, value);
                    }
                    Err(err) => self.request = Err(BuilderError::Http(err.into()).into()),
                },
                Err(err) => {
                    self.request = Err(BuilderError::Http(err.into()).into());
                }
            };
        }
        self
    }

    /// Add a set of Headers to the existing ones on this Request.
    ///
    /// The headers will be merged in to any already set.
    pub fn headers<M>(mut self, headers: M) -> Self
    where
        HeaderMap: TryFrom<M>,
        <HeaderMap as TryFrom<M>>::Error: Into<http::Error>,
    {
        if let Ok(ref mut req) = self.request {
            match <HeaderMap as TryFrom<M>>::try_from(headers) {
                Ok(headers) => {
                    let mut prev: Option<HeaderName> = None;
                    for (key, value) in headers {
                        match key {
                            Some(key) => {
                                req.headers_mut().insert(&key, value);
                                prev = Some(key);
                            }
                            None => {
                                if let Some(ref key) = prev {
                                    req.headers_mut().append(key, value);
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    self.request = Err(BuilderError::Http(err.into()).into());
                }
            }
        }
        self
    }

    /// Enable HTTP basic authentication.
    ///
    /// ```
    /// # use http_unix_client::{Client, Error};
    ///
    /// # async fn run() -> Result<(), Error> {
    /// let client = Client::new();
    /// let resp = client.delete("/tmp/my.socket", "/delete")
    ///     .basic_auth("admin", Some("good password"))
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn basic_auth<U, P>(self, username: U, password: Option<P>) -> Self
    where
        U: fmt::Display,
        P: fmt::Display,
    {
        let decode = match password {
            Some(password) => format!("{username}:{password}"),
            None => username.to_string(),
        };
        let encode = BASE64_STANDARD.encode(decode.as_bytes());
        self.header_sensitive(AUTHORIZATION, format!("Basic {encode}"), true)
    }

    /// Enable HTTP bearer authentication.
    pub fn bearer_auth<T>(self, token: T) -> Self
    where
        T: fmt::Display,
    {
        let token = format!("Bearer {token}");
        self.header_sensitive(AUTHORIZATION, token, true)
    }

    /// Sets the request body.
    pub fn body<T>(mut self, body: T) -> Self
    where
        T: Into<Body>,
    {
        if let Ok(ref mut req) = self.request {
            *req.body_mut() = Some(body.into());
        }
        self
    }

    /// Serializes the given value as query parameters and appends them to the URI.
    pub fn query<T>(mut self, query: &T) -> Self
    where
        T: Serialize + ?Sized,
    {
        let mut error = None;
        if let Ok(ref mut req) = self.request {
            let url = req.url_mut();
            let mut pairs = url.query_pairs_mut();
            let serializer = serde_urlencoded::Serializer::new(&mut pairs);

            if let Err(err) = query.serialize(serializer) {
                error = Some(err);
            }
        }

        if let Ok(ref mut req) = self.request {
            if let Some("") = req.url().query() {
                req.url_mut().set_query(None);
            }
        }
        if let Some(err) = error {
            self.request = Err(BuilderError::SerializeUrl(err).into());
        }

        self
    }

    /// Sets the HTTP version for the request (e.g., `HTTP/2`).
    pub fn version(mut self, version: Version) -> Self {
        if let Ok(ref mut req) = self.request {
            *req.version_mut() = version;
        }
        self
    }

    /// Serializes the given value as a URL-encoded form body and sets the appropriate `Content-Type` header.
    pub fn form<T>(mut self, form: &T) -> Self
    where
        T: Serialize + ?Sized,
    {
        if let Ok(ref mut req) = self.request {
            match serde_urlencoded::to_string(form) {
                Ok(body) => {
                    req.headers_mut().insert(
                        CONTENT_TYPE,
                        HeaderValue::from_static("application/x-www-form-urlencoded"),
                    );
                    *req.body_mut() = Some(body.into());
                }
                Err(err) => self.request = Err(BuilderError::SerializeUrl(err).into()),
            }
        }

        self
    }

    /// Send a JSON body.
    ///
    /// # Optional
    ///
    /// This requires the optional `json` feature enabled.
    ///
    /// # Errors
    ///
    /// Serialization can fail if `T`'s implementation of `Serialize` decides to
    /// fail, or if `T` contains a map with non-string keys.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn json<T>(mut self, json: &T) -> Self
    where
        T: Serialize + ?Sized,
    {
        if let Ok(ref mut req) = self.request {
            match serde_json::to_vec(json) {
                Ok(body) => {
                    req.headers_mut()
                        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                    *req.body_mut() = Some(body.into());
                }
                Err(err) => self.request = Err(BuilderError::SerializeJson(err).into()),
            }
        }

        self
    }

    /// Returns the underlying request, consuming the builder.
    ///
    /// # Errors
    /// Returns an error if the request is invalid.
    pub fn build(self) -> Result<Request> {
        self.request
    }

    /// Build a `Request`, which can be inspected, modified and executed with
    /// `Client::execute()`.
    ///
    /// This is similar to [`RequestBuilder::build()`], but also returns the
    /// embedded [`Client`].
    pub fn build_split(self) -> (Client, crate::Result<Request>) {
        (self.client, self.request)
    }

    /// Constructs the Request and sends it to the target URL, returning a
    /// future Response.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending request,
    /// redirect loop was detected or redirect limit was exhausted.
    ///
    /// # Example
    ///
    /// ```
    /// # use http_unix_client::Error;
    /// #
    /// # async fn run() -> Result<(), Error> {
    /// let response = http_unix_client::Client::new()
    ///     .get("/tmp.my.socket", "/get")
    ///     .send()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(self) -> Result<Response> {
        let request = self.request?;
        let response = self.client.execute(request).await?;

        Ok(response)
    }

    /// Attempt to clone the RequestBuilder.
    ///
    /// `None` is returned if the RequestBuilder can not be cloned,
    ///
    /// # Examples
    ///
    /// ```
    /// # use http_unix_client::Error;
    /// #
    /// # fn run() -> Result<(), Error> {
    /// let client = http_unix_client::Client::new();
    /// let builder = client.post("/tmp/my.socket", "/post")
    ///     .body("from a &str!");
    /// let clone = builder.try_clone();
    /// assert!(clone.is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_clone(&self) -> Option<RequestBuilder> {
        self.request.as_ref().ok().map(|req| RequestBuilder {
            client: self.client.clone(),
            request: Ok(req.clone()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Method;
    use serde::Serialize;
    use std::collections::BTreeMap;

    #[test]
    fn add_query_append() {
        let client = Client::new();
        let builder = client.get("/tmp/my.socket", "/");

        let builder = builder.query(&[("foo", "bar")]);
        let builder = builder.query(&[("qux", 3)]);

        let req = builder.build().expect("request is invalid");
        assert_eq!(req.url().query(), Some("foo=bar&qux=3"));
    }

    #[test]
    fn add_query_append_same() {
        let client = Client::new();
        let builder = client.get("/tmp/my.socket", "/");

        let builder = builder.query(&[("foo", "a"), ("foo", "b")]);

        let req = builder.build().expect("request is valid");
        assert_eq!(req.url().query(), Some("foo=a&foo=b"));
    }

    #[test]
    fn add_query_struct() {
        #[derive(Serialize)]
        struct Params {
            foo: String,
            qux: i32,
        }

        let client = Client::new();
        let builder = client.get("/tmp/my.socket", "/");

        let params = Params {
            foo: "bar".into(),
            qux: 3,
        };

        let builder = builder.query(&params);

        let req = builder.build().expect("request is invalid");
        assert_eq!(req.url().query(), Some("foo=bar&qux=3"));
    }

    #[test]
    fn add_query_map() {
        let mut params = BTreeMap::new();
        params.insert("foo", "bar");
        params.insert("qux", "three");

        let client = Client::new();
        let builder = client.get("/tmp/my.socket", "/");

        let builder = builder.query(&params);

        let req = builder.build().expect("request is invalid");
        assert_eq!(req.url().query(), Some("foo=bar&qux=three"));
    }

    #[test]
    fn test_replace_headers() {
        use http::HeaderMap;

        let mut headers = HeaderMap::new();
        headers.insert("foo", "bar".parse().unwrap());
        headers.append("foo", "baz".parse().unwrap());

        let client = Client::new();
        let req = client
            .get("/tmp/my.socket", "/")
            .header("im-a", "keeper")
            .header("foo", "pop me")
            .headers(headers)
            .build()
            .expect("request build");

        assert_eq!(req.headers()["im-a"], "keeper");

        let foo = req.headers().get_all("foo").iter().collect::<Vec<_>>();
        assert_eq!(foo.len(), 2);
        assert_eq!(foo[0], "bar");
        assert_eq!(foo[1], "baz");
    }

    #[test]
    fn normalize_empty_query() {
        let client = Client::new();
        let empty_query: &[(&str, &str)] = &[];

        let req = client
            .get("/tmp/my.socket", "/")
            .query(empty_query)
            .build()
            .expect("request build");

        assert_eq!(req.url().query(), None);
        assert_eq!(req.url().as_str(), "unix://2f746d702f6d792e736f636b6574/");
    }

    #[test]
    fn try_clone_reusable() {
        let client = Client::new();
        let builder = client
            .post("/tmp/my.socket", "/post")
            .header("foo", "bar")
            .body("from a &str!");
        let req = builder
            .try_clone()
            .expect("clone successful")
            .build()
            .expect("request is valid");
        assert_eq!(
            req.url().as_str(),
            "unix://2f746d702f6d792e736f636b6574/post"
        );
        assert_eq!(req.method(), Method::POST);
        assert_eq!(req.headers()["foo"], "bar");
    }

    #[test]
    fn try_clone_no_body() {
        let client = Client::new();
        let builder = client.get("/tmp/my.socket", "/get");

        let req = builder
            .try_clone()
            .expect("clone successful")
            .build()
            .expect("request is valid");
        assert_eq!(
            req.url().as_str(),
            "unix://2f746d702f6d792e736f636b6574/get"
        );
        assert_eq!(req.method(), Method::GET);
        assert!(req.body().is_none());
    }

    #[test]
    fn test_basic_auth_sensitive_header() {
        let client = Client::new();

        let req = client
            .get("/tmp/my.socket", "/")
            .basic_auth("Aladdin", Some("open sesame"))
            .build()
            .expect("request build");

        assert_eq!(req.url().as_str(), "unix://2f746d702f6d792e736f636b6574/");
        assert_eq!(
            req.headers()["authorization"],
            "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="
        );
        assert!(req.headers()["authorization"].is_sensitive());
    }

    #[test]
    fn test_bearer_auth_sensitive_header() {
        let client = Client::new();

        let req = client
            .get("/tmp/my.socket", "/")
            .bearer_auth("Hold my bear")
            .build()
            .expect("request build");

        assert_eq!(req.url().as_str(), "unix://2f746d702f6d792e736f636b6574/");
        assert_eq!(req.headers()["authorization"], "Bearer Hold my bear");
        assert!(req.headers()["authorization"].is_sensitive());
    }

    #[test]
    fn test_explicit_sensitive_header() {
        let client = Client::new();

        let mut header = http::HeaderValue::from_static("in plain sight");
        header.set_sensitive(true);

        let req = client
            .get("/tmp/my.socket", "/")
            .header("hiding", header)
            .build()
            .expect("request build");

        assert_eq!(req.url().as_str(), "unix://2f746d702f6d792e736f636b6574/");
        assert_eq!(req.headers()["hiding"], "in plain sight");
        assert!(req.headers()["hiding"].is_sensitive());
    }

    #[test]
    fn builder_split_reassemble() {
        let builder = {
            let client = Client::new();
            client.get("/tmp/my.socket", "/")
        };
        let (client, inner) = builder.build_split();
        let request = inner.unwrap();
        let builder = RequestBuilder::from_parts(client, request);
        builder.build().unwrap();
    }
}
