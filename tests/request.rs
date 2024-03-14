#[cfg(test)]
mod request_tests {
    use bytes::Bytes;
    use hyper::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
    use isup::Request;

    #[test]
    fn it_creates_a_new() {
        // Define the method and URL
        let method = "GET";
        let url = "http://example.com/";
        // Create a new request
        let request = Request::new(method, url);
        // Verify that the request was created
        assert_eq!(&request.url.to_string(), url);
        // Verify that the method was set
        assert_eq!(request.method, method);
        // Verify that the body is empty
        assert!(request.body.is_empty());
        // Verify that the headers are empty
        assert!(request.headers.is_empty());
    }

    #[test]
    #[should_panic]
    fn it_fails_comparing_url_without_trailing_slash() {
        // Define the method and URL
        let method = "GET";
        let url = "http://example.com";
        // Create a new request
        let request = Request::new(method, url);

        // The `Uri` crate will append a trailing slash if not provided
        // when using the `to_string()` method.
        // "http://example.com/" != "http://example.com"
        assert_eq!(&request.url.to_string(), url);
    }

    #[test]
    fn it_sets_body() {
        // Create a new request
        let mut request = Request::new("POST", "http://example.com/");
        // Create a new body
        let body = Bytes::from("Hello, Rust");
        // Set the body of the request
        request = request.set_body(body.clone());
        // Verify that the body was set
        assert_eq!(request.body, body);
    }

    #[test]
    fn it_sets_headers() {
        // Create a new request
        let mut request = Request::new("POST", "http://example.com/");
        // Create a new header map
        let mut headers = HeaderMap::new();
        // Insert a new header into the map
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Set the headers of the request
        request = request.set_headers(headers.clone());
        // Verify that the headers were set
        assert_eq!(request.headers, headers);
    }
}
