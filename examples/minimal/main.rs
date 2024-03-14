use isup::{Config, Service};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // > Load the configuration from a file
    let config = Config::from_file("examples/minimal/config.yml")?;

    // Even though loading from a config file is possible, another way to utilize a `Service`
    // would be through the `Service::default()` method, and then `service.insert_request(Request)`
    //
    // > Initialize a Default Service and use the requests defined in the config file
    let service = Service::from_config(config)?;

    // ...rest of the code

    // > Update the service
    service.update().await?;

    // > Retrieve the best scoring URL
    println!(">> {:?}", service.best_url().await?);

    Ok(())
}
