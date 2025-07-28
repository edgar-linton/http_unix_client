use bytes::Bytes;

/// Represents the body of an HTTP request.
#[derive(Debug, Clone)]
pub struct Body {
    /// The underlying byte buffer of the request body.
    inner: Bytes,
}

impl Body {
    /// Returns a reference to the raw bytes of the HTTP body.
    #[inline]
    pub fn bytes(&self) -> &Bytes {
        &self.inner
    }
}

impl From<Bytes> for Body {
    #[inline]
    fn from(value: Bytes) -> Self {
        Self { inner: value }
    }
}

impl From<Vec<u8>> for Body {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl From<&'static [u8]> for Body {
    #[inline]
    fn from(value: &'static [u8]) -> Self {
        Self {
            inner: Bytes::from_static(value),
        }
    }
}

impl From<String> for Body {
    #[inline]
    fn from(value: String) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl From<&'static str> for Body {
    #[inline]
    fn from(value: &'static str) -> Self {
        Self {
            inner: value.into(),
        }
    }
}
