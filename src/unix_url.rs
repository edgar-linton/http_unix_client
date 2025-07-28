use http::Uri;
use std::fmt;
use std::path::Path;
use std::str::Split;
use url::{Origin, PathSegmentsMut, UrlQuery};
use url::{ParseError, Url, form_urlencoded::Serializer};

/// A wrapper around `Url` representing a URL over a UNIX domain socket.
///
/// `UnixUrl` encapsulates a UNIX socket path and a relative URL path into a
/// single `unix://`-scheme `Url`, where the socket path is hex-encoded into the host portion.
/// This is particularly useful for HTTP clients that support UNIX socket connections.
///
/// # Example
///
/// ```
/// use http_unix_client::UnixUrl;
///
/// let socket_path = "/tmp/my.socket";
/// let url_path = "/api/v1/status";
/// let unix_url = UnixUrl::new(socket_path, url_path).unwrap();
///
/// assert_eq!(
///     unix_url.to_string(),
///     "unix://2f746d702f6d792e736f636b6574/api/v1/status");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct UnixUrl {
    inner: Url,
}

impl UnixUrl {
    /// Creates a new `UnixUrl` from a UNIX socket path and a relative URL path.
    ///
    /// The socket path is hex-encoded into the `host` component of a `unix://` URL,
    /// while the given path becomes the URL's path/query/fragment.
    ///
    /// # Arguments
    ///
    /// * `socket` - Path to the UNIX domain socket
    /// * `path` - A relative URL path, such as `/api/v1`, `foo/bar?debug=true`, etc.
    ///
    /// # Errors
    ///
    /// Returns a `ParseError` if the final URL is not syntactically valid.
    pub fn new<P>(socket: P, path: &str) -> Result<Self, ParseError>
    where
        P: AsRef<Path>,
    {
        let encoded_socket = hex::encode(socket.as_ref().to_string_lossy().as_bytes());

        let normalized_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}",)
        };

        let url_string = format!("unix://{encoded_socket}{normalized_path}");
        let url = Url::parse(&url_string)?;

        Ok(Self { inner: url })
    }

    /// Returns the full URL as a string.
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    /// Returns a reference to the inner `Url`.
    pub fn as_url(&self) -> &Url {
        &self.inner
    }

    /// Returns the fragment of the URL, if any.
    pub fn fragment(&self) -> Option<&str> {
        self.inner.fragment()
    }

    /// Consumes this instance and returns the inner `Url`.
    pub fn into_inner(self) -> Url {
        self.inner
    }

    /// Returns the URL's origin (scheme + host + port).
    pub fn origin(&self) -> Origin {
        self.inner.origin()
    }

    /// Returns the path of the URL.
    pub fn path(&self) -> &str {
        self.inner.path()
    }

    /// Returns an iterator over the path segments.
    pub fn path_segments(&self) -> Option<Split<'_, char>> {
        self.inner.path_segments()
    }

    /// Returns a mutable interface to modify path segments.
    pub fn path_segments_mut(&mut self) -> Option<PathSegmentsMut<'_>> {
        self.inner.path_segments_mut().ok()
    }

    /// Returns the query string of the URL, if any.
    pub fn query(&self) -> Option<&str> {
        self.inner.query()
    }

    /// Returns a query pair serializer for mutating query parameters.
    pub fn query_pairs_mut(&mut self) -> Serializer<'_, UrlQuery<'_>> {
        self.inner.query_pairs_mut()
    }

    /// Sets the path of the URL.
    pub fn set_path(&mut self, path: &str) {
        self.inner.set_path(path);
    }

    /// Sets the query string of the URL.
    pub fn set_query(&mut self, query: Option<&str>) {
        self.inner.set_query(query);
    }
}

impl fmt::Display for UnixUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<UnixUrl> for Url {
    fn from(value: UnixUrl) -> Self {
        value.inner
    }
}

impl TryFrom<UnixUrl> for Uri {
    type Error = http::Error;

    fn try_from(value: UnixUrl) -> Result<Self, Self::Error> {
        let uri = value.inner.as_str().parse()?;

        Ok(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_to_string() {
        let url = UnixUrl::new("/tmp/my.socket", "/").unwrap();
        assert_eq!(url.as_str(), "unix://2f746d702f6d792e736f636b6574/");
    }

    #[test]
    fn test_path_normalization() {
        let url = UnixUrl::new("/tmp/my.socket", "v1/status").unwrap();
        assert_eq!(url.path(), "/v1/status");
    }

    #[test]
    fn test_query_and_fragment() {
        let url = UnixUrl::new("/tmp/my.socket", "/hello/world?debug=true#frag").unwrap();
        assert_eq!(url.query(), Some("debug=true"));
        assert_eq!(url.fragment(), Some("frag"));
    }

    #[test]
    fn test_query_mutation() {
        let mut url = UnixUrl::new("/tmp/my.socket", "/foo").unwrap();
        url.set_query(Some("x=1"));
        assert_eq!(url.query(), Some("x=1"));

        let mut qp = url.query_pairs_mut();
        qp.append_pair("y", "2");
        drop(qp);

        assert!(url.query().unwrap().contains("x=1"));
        assert!(url.query().unwrap().contains("y=2"));
    }
}
