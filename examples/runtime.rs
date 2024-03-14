use isup::{Request, Service};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // > Initialize a Default Service
    let mut service = Service::default();
    // with empty requests
    assert_eq!(service.urls(), Vec::<String>::new());

    // > Insert a new request on runtime, after initialization
    let request = Request::new("GET", "https://www.rust-lang.org");
    service.insert_request(request);

    // > Get and print all requests for demonstrational purposes
    let all = service.urls();
    assert_eq!(all.len(), 1);

    // > Update the scores
    service.update().await?;

    // > Retrieve the best scoring URL
    let best = service.best_url().await?;
    println!(">> {:?}", best);

    // > Remove the request
    service.remove_request("https://www.rust-lang.org")?;
    // and verify that the request was removed
    assert_eq!(service.urls(), Vec::<String>::new());

    Ok(())
}
