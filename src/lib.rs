#![forbid(unsafe_code)]
#![warn(clippy::all, unreachable_pub)]
#![deny(unused_must_use, rust_2018_idioms)]

mod score;
pub use score::Score;

mod config;
pub use config::Config;

mod client;
pub use client::Client;

mod request;
pub use request::Request;

/// The `store` module provides the necessary implementations for data storage and retrieval within the application.
/// It defines the `Store` trait and various implementations of this trait to handle the storage of monitoring data,
/// such as scores and metrics, potentially using different backend technologies (in-memory storage, redis, ...).
pub mod store;
use store::Store;

/// The `strategy` module contains logic for score calculation and assessment strategies.
/// It defines the `Strategy` trait, along with various implementations that dictate how to calculate and update
/// the performance scores of monitored services based on response times, error rates, and other significant metrics.
pub mod strategy;
use strategy::Strategy;

use bytes::Bytes;
use futures::future::join_all;
use http_body_util::Full;
use hyper::Uri;
use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{str::FromStr, time::Duration};

/// The `Service` struct is the main component of the application, responsible for
/// orchestrattion, monitoring and performance calculation.
/// At the same time, the library provides all the necessary components to build
/// a custom monitoring service by utilizing the available traits and structs.
pub struct Service {
    /// The HTTP client used for executing the requests. It handles the network
    /// communication and ensures requests are properly sent and responses received.
    client: Client,
    /// The strategy used for calculating the scores of the endpoints. It takes into
    /// account various metrics and updates the evaluation of the endpoints.
    strategy: Box<dyn Strategy + Sync + Send + 'static>,
    /// The store mechanism for the scores. It allows for storing, updating,
    /// and retrieving the scores of monitored endpoints.
    pub store: Box<dyn Store + Sync + Send + 'static>,
    /// List of HTTP requests to be monitored. Each request corresponds to a
    /// web endpoint whose availability and performance is to be ranked.
    pub requests: Vec<hyper::Request<Full<Bytes>>>,
    /// Unix timestamp of last time the scores were updated.
    pub updated_at: AtomicU64,
}

impl Default for Service {
    /// Creates a new `Service` instance with default settings.
    fn default() -> Self {
        let client = Client::default();
        let strategy = strategy::WeightedLog::default();
        let store = store::Memory::default();
        Self::new(strategy, store, client, vec![])
    }
}

impl Service {
    /// Constructs a new `Service`.
    ///
    /// # Arguments
    /// * `client`: A `Client` instance for making HTTP requests.
    /// * `interval`: Duration between consecutive monitoring cycles.
    /// * `strategy`: Implementation of the scoring strategy.
    /// * `store`: Implementation of the Store trait.
    /// * `requests`: List of web endpoints to monitor.
    ///
    /// # Returns
    /// A new instance of `Service`.
    pub fn new(
        strategy: impl Strategy + Sync + Send + 'static,
        store: impl Store + Sync + Send + 'static,
        client: Client,
        requests: Vec<Request>,
    ) -> Self {
        Self {
            // Convert each `Request` into a `hyper::Request` for the HTTP client.
            requests: requests.into_iter().map(|request| request.into()).collect(),
            client,
            store: Box::new(store),
            strategy: Box::new(strategy),
            updated_at: AtomicU64::new(0),
        }
    }

    /// Initializes a `Service` instance based on provided configuration.
    ///
    /// # Arguments
    /// * `config`: Configuration settings for the service.
    ///
    /// # Returns
    /// A result that, on success, contains an initialized `Service` instance.
    ///
    /// # Errors
    /// Returns an error if the configuration is invalid or incomplete.
    pub fn from_config(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        //  Create store from the configuration
        let store = store::from_config(config.store);
        // Create strategy from the configuration
        let strategy = strategy::from_config(config.strategy);
        // Initialize a new HTTP client; without timeout set from the configuration
        // and `pool_idle_timeout` set to 60 seconds. That determines how long an idle
        // connection is kept open before being closed.

        let client = match config.client {
            Some(config) => Client::new(config.request_timeout, config.pool_idle_timeout),
            None => Client::new(config.interval, None),
        };

        // Create `HyperRequest` instances from the configuration's `Request` instances
        let requests = config.requests.into_iter().map(|request| request.into()).collect();

        Ok(Self { requests, client, store, strategy, updated_at: AtomicU64::new(0) })
    }

    /// Retrieves the URL with the best score asynchronously.
    ///
    /// # Returns
    /// A future resolving to an `Option<String>` containing the best URL or an error.
    ///
    /// # Errors
    /// Returns an error if the process of retrieving the best URL fails.
    pub async fn best_url(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        self.store.best_url().await
    }

    /// Spawns a background task to periodically update scores of endpoints.
    ///
    /// # Arguments
    /// * `interval`: Duration between each scoring update.
    ///
    /// This function runs indefinitely, updating endpoint scores based on the specified interval.
    pub async fn run(self: std::sync::Arc<Self>, interval: Duration) {
        tokio::spawn(async move {
            loop {
                // Update scores for all services
                self.update().await.expect("failed to update scores");
                // Wait for the specified interval before the next update
                tokio::time::sleep(interval).await;
            }
        });
    }

    /// Retrieves a list of all monitored URLs.
    ///
    /// # Returns
    /// A vector of strings, each representing a monitored URL.
    pub fn urls(&self) -> Vec<String> {
        self.requests.iter().map(|r| r.uri().to_string()).collect()
    }

    /// Adds a new request to the list of monitored endpoints.
    ///
    /// # Arguments
    /// * `request`: The request to be added for monitoring.
    pub fn insert_request(&mut self, request: Request) {
        self.requests.push(request.into());
    }

    /// Removes a request from the list of monitored endpoints.
    ///
    /// # Arguments
    /// * `url`: The URL of the request to be removed.
    ///
    /// # Returns
    /// A result indicating the success of the operation.
    ///
    /// # Errors
    /// Returns an error if the URL is invalid or cannot be parsed.
    pub fn remove_request(&mut self, url: &str) -> Result<(), Box<dyn Error>> {
        let url = Uri::from_str(url)?.to_string();
        self.requests.retain(|r| r.uri().to_string() != url);
        Ok(())
    }

    /// Sets a new store for storing and retrieving scores.
    ///
    /// # Arguments
    /// * `store`: The new store to be used.
    ///
    /// # Returns
    /// The updated `Service` instance with the new store.
    pub fn use_store<T: Store + Sync + Send + 'static>(mut self, store: T) -> Self {
        self.store = Box::new(store);
        self
    }

    /// Sets a new strategy for score calculation.
    ///
    /// # Arguments
    /// * `strategy`: The new strategy to be used for score calculation.
    ///
    /// # Returns
    /// The updated `Service` instance with the new strategy.
    pub fn use_strategy<T: Strategy + Sync + Send + 'static>(mut self, strategy: T) -> Self {
        self.strategy = Box::new(strategy);
        self
    }

    /// Updates the scores for all tracked services.
    ///
    /// This function performs HTTP requests concurrently for each service, updating their
    /// scores based on the response time and HTTP status code. It leverages the provided
    /// strategy for score calculation and updates the store with new scores.
    pub async fn update(&self) -> Result<(), Box<dyn Error>> {
        // Concurrently send requests to all endpoints and handle their responses
        join_all(self.requests.iter().map(|r| self.process_request(r))).await;

        // Update the timestamp of the last update
        let unix = SystemTime::now().duration_since(UNIX_EPOCH)?;
        self.updated_at.store(unix.as_secs(), SeqCst);
        Ok(())
    }

    /// Handles a single request, updating the score for its corresponding service.
    ///
    /// # Arguments
    /// * `request` - A reference to the hyper::Request object to be sent.
    ///
    /// This function sends the HTTP request, measures the response time, calculates the
    /// new score based on the strategy, and updates the score in store.
    async fn process_request(&self, request: &hyper::Request<Full<Bytes>>) {
        let url = request.uri().to_string();

        let start = tokio::time::Instant::now();
        let response = self.client.request(request.clone()).await;
        let elapsed = start.elapsed();

        let status = response.map(|r| r.status().as_u16()).unwrap_or(0);

        // Calculate and update score based on response
        self.update_score(url, elapsed, status).await;
    }

    /// Calculates and updates the score for a given URL.
    ///
    /// # Arguments
    /// * `url` - The URL of the service.
    /// * `elapsed` - The elapsed time of the request.
    /// * `status` - The HTTP status code received in the response.
    ///
    /// This function calculates the new score based on the elapsed time and status code,
    /// then updates it in the store.
    async fn update_score(&self, url: String, elapsed: Duration, status: u16) {
        let score = match self.store.get(&url).await {
            Ok(Some(score)) => self.strategy.calculate(score, elapsed, status),
            _ => self.strategy.calculate(Score::default(), elapsed, status),
        };

        self.store.set(url, score).await.expect("failed to set score");
    }
}
