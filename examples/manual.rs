use isup::{store::Memory, strategy::WeightedLog, Client, Request, Service};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // > Initialize a default client  (2 second request timeout and 60 second pool idle timeout)
    let client = Client::default();
    // Additionally, you can create a client with custom settings
    // let client = Client::new(request_timeout, pool_idle_timeout);
    //
    // > Initialize a default strategy
    let strategy = WeightedLog::default();
    // For a strategy with custom settings:
    // let strategy = WeightedLog::new(weight, effort);
    //
    // > Initialize a default store
    let store = Memory::default();
    // Any store that implements the `Store` trait can be used as long as their feature is enabled.
    //
    // When in production, always load sensitive data in a secure manner, that can be done programmatically
    // using the `store::Redis::from_url(REDIS_ENV_URL)` or by getting the value from a secure secret storage.
    //
    // > Create a list of requests to monitor
    // Headers can be set with the method `set_headers(HeaderMap)`
    let requests: Vec<Request> = vec![
        // Additionally, in case the request needs a body, use the method `set_body(Into<Bytes>)`
        Request::new("POST", "https://www.rust-lang.org").set_body("🦀"),
        Request::new("POST", "https://example.com/"),
    ];

    // > Construct the new `Service` instance with our settings
    let service = Service::new(strategy, store, client, requests);

    // > Set the interval for the blocking loop
    let interval = Duration::from_millis(5000);

    println!("initializing service, press Ctrl+C to exit");
    println!();
    println!("urls: {:?}", service.urls());
    loop {
        // Update scores for all services
        service.update().await?;
        // Retrieve the best scoring URL
        println!(">> {:?}", service.best_url().await?);
        // Wait for the specified interval before the next update
        sleep(interval).await;
    }
}
