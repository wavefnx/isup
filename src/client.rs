use crate::config::deserialize_opt_duration;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{body::Incoming, Request, Response};
use hyper_tls::HttpsConnector;
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client as HyperClient},
    rt::TokioExecutor,
};
use std::{error::Error, time::Duration};

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
/// Client configuration
///
/// The `request_timeout` field is used to define the maximum time a request can take before it's considered failed, impacting it's score.
/// The `pool_idle_timeout` field is used to define the maximum time a connection can be idle before it's closed.
///
/// If the interval is set but the client configuration is not:
/// - the `request_timeout` will default to the interval value
/// - the `pool_idle_timeout` will default to the underlying hyper client's default value (90s)
///
/// If no interval and no client configuration is set:
/// - the `request_timeout` will default to the underlying hyper client's default value (never)
/// - the `pool_idle_timeout` will default to the underlying hyper client's default value (90s)
pub struct Config {
    #[serde(deserialize_with = "deserialize_opt_duration")]
    pub request_timeout: Option<std::time::Duration>,
    #[serde(deserialize_with = "deserialize_opt_duration")]
    pub pool_idle_timeout: Option<std::time::Duration>,
}

/// A client for making HTTP requests, built on top of Hyper and Hyper-TLS for HTTPS support.
pub struct Client {
    /// The inner HyperClient, which handles the actual HTTP requests.
    inner: HyperClient<HttpsConnector<HttpConnector>, Full<Bytes>>,
    /// The maximum amount of time to wait for a request to complete.
    request_timeout: Option<Duration>,
}

impl Default for Client {
    /// Create a new default instance of `Client` with a 2 second request timeout and a 60 second pool idle timeout.
    fn default() -> Self {
        Self {
            inner: HyperClient::builder(TokioExecutor::new())
                .pool_idle_timeout(Duration::from_secs(60))
                .build(HttpsConnector::new()),
            request_timeout: Some(Duration::from_secs(2)),
        }
    }
}

impl Client {
    /// Creates a new instance of `Client` with custom timeout settings.
    ///
    /// # Arguments
    /// * `request_timeout`: Duration to wait before timing out a request.
    /// * `pool_idle_timeout`: Duration before an idle connection in the pool is closed.
    pub fn new(request_timeout: Option<Duration>, pool_idle_timeout: Option<Duration>) -> Self {
        Self {
            request_timeout,
            inner: HyperClient::builder(TokioExecutor::new())
                .pool_idle_timeout(pool_idle_timeout)
                .build(HttpsConnector::new()),
        }
    }

    pub fn from_config(config: Config) -> Self {
        Self::new(config.request_timeout, config.pool_idle_timeout)
    }

    /// Updates the request timeout for the client.
    ///
    /// # Arguments
    /// * `timeout`: New timeout duration to set.
    ///
    /// # Returns
    /// The updated `Client` instance.
    pub fn set_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.request_timeout = timeout;
        self
    }
}

impl Client {
    /// Sends an HTTP request and awaits the response.
    ///
    /// # Arguments
    /// * `req`: The hyper::Request object to send.
    ///
    /// # Returns
    /// A `Result` which, on success, contains the `Response<Incoming>`. On failure, it returns an error.
    ///
    /// This method uses `tokio::time::timeout` to apply the configured request timeout.
    pub async fn request(&self, req: Request<Full<Bytes>>) -> Result<Response<Incoming>, Box<dyn Error>> {
        match self.request_timeout {
            Some(timeout) => {
                let response = tokio::time::timeout(timeout, self.inner.request(req)).await?;
                Ok(response?)
            }
            None => {
                let response = self.inner.request(req).await?;
                Ok(response)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_client_new() {
        let request_timeout = Some(Duration::from_secs(5));
        let pool_idle_timeout = Some(Duration::from_secs(30));

        let client = Client::new(request_timeout, pool_idle_timeout);

        assert_eq!(client.request_timeout, request_timeout);
    }
}
