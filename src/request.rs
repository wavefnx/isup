use crate::config::{deserialize_body, deserialize_headers, deserialize_method, deserialize_uri};
use bytes::Bytes;
use http_body_util::Full;
use hyper::{HeaderMap, Method, Uri};

/// Represents an HTTP request with customizable elements like URL, method, body, and headers.
/// This struct is designed for ease of creation, deserialization and modification of HTTP request components.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Request {
    /// The URL of the request, stored as a `Uri`.
    /// It is deserialized using a custom deserializer to handle different URI formats.
    #[serde(deserialize_with = "deserialize_uri")]
    pub url: Uri,
    /// The HTTP method (e.g., GET, POST) for the request.
    /// Custom deserialization is used to convert string representations into `Method` types.
    #[serde(deserialize_with = "deserialize_method")]
    pub method: Method,
    /// The body of the request, represented as `Bytes`.
    /// A custom deserializer is used, and it defaults to an empty body if not provided.
    #[serde(deserialize_with = "deserialize_body", default = "Bytes::new")]
    pub body: Bytes,
    /// A collection of HTTP headers as a `HeaderMap`.
    /// These are deserialized using a custom function to correctly handle header formatting.
    #[serde(deserialize_with = "deserialize_headers", default = "HeaderMap::new")]
    pub headers: HeaderMap,
}

impl Request {
    /// Creates a new `Request` instance with specified method and URL.
    ///
    /// # Arguments
    /// * `method`: A string slice representing the HTTP method.
    /// * `url`: A string slice representing the URL of the request.
    ///
    /// # Panics
    /// Panics if the method or URL cannot be parsed.
    pub fn new<I: Into<String>>(method: I, url: I) -> Self {
        Self {
            url: url.into().parse().expect("Invalid URL"),
            method: method.into().parse().expect("Invalid method"),
            body: Bytes::new(),
            headers: HeaderMap::new(),
        }
    }

    /// Sets the body of the request.
    ///
    /// # Arguments
    /// * `body`: The new body of the request, provided as any type that can convert into `Bytes`.
    ///
    /// # Returns
    /// The updated `Request` instance with the new body.
    pub fn set_body<I: Into<Bytes>>(mut self, body: I) -> Self {
        self.body = body.into();
        self
    }

    /// Sets the headers of the request.
    ///
    /// # Arguments
    /// * `headers`: A `HeaderMap` representing the new headers of the request.
    ///
    /// # Returns
    /// The updated `Request` instance with the new headers.
    pub fn set_headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }
}

impl From<Request> for hyper::Request<Full<Bytes>> {
    /// Converts a `Request` instance into a `hyper::Request` object.
    /// This allows the `Request` to be used directly with the Hyper library.
    ///
    /// # Arguments
    /// * `request`: The `Request` instance to convert.
    ///
    /// # Returns
    /// A `hyper::Request` object built from the provided `Request` instance.
    fn from(request: Request) -> hyper::Request<Full<Bytes>> {
        let mut builder = hyper::Request::builder();

        *builder.headers_mut().expect("failed to acquire builder headers") = request.headers;

        builder.method(request.method).uri(request.url).body(Full::new(request.body)).expect("failed to build request")
    }
}
