use isup::{Config, Service};
use std::sync::{atomic::Ordering::SeqCst, Arc};
use warp::Filter;

// Local port to run the server
const PORT: u16 = 8080;

// Define the response for the route handler
#[derive(serde::Serialize)]
struct Response {
    // The best scoring URL
    url: Option<String>,
    // The timestamp of the last update
    updated_at: u64,
}

// Implement a new method for the Response struct
impl Response {
    pub fn new(url: Option<String>, updated_at: u64) -> Self {
        Self { updated_at, url }
    }
}

// src/routes.rs
//
// Define the route handler
async fn best_url(service: Arc<Service>) -> Result<impl warp::Reply, warp::Rejection> {
    // Additionally, the response can be cached in a light data structure (in-memory, ...)
    let url = service.best_url().await.unwrap_or(None);
    let updated_at = service.updated_at.load(SeqCst);

    Ok(warp::reply::json(&Response::new(url, updated_at)))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // > Load the configuration from a file
    let config = Config::from_file("examples/server/config.yml")?;

    // > Extract the interval from the configuration
    let interval = config.interval.expect("interval is required");

    // > Create a new IsUp instance wrapped in an Arc
    // This allows us to share the instance across threads
    // and create cheap clones of the instance when required
    let service = Arc::new(Service::from_config(config)?);

    // > Spawn a background task to update scores
    // This creates a new task will run indefinitely in the background,
    // updating the scores at the interval specified in the configuration
    service.clone().run(interval).await;

    // > Create a Service instance to pass to the route handler
    let warp_service = warp::any().map(move || service.clone());
    // > Define the GET / route
    let route = warp::get().and(warp_service).and_then(best_url);

    // Print the server address
    println!("initialized service @ http://localhost:{PORT}");
    // > Start the server
    warp::serve(route).run(([127, 0, 0, 1], PORT)).await;

    Ok(())
}
