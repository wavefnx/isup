    
<div align="center">

![isup-logo](https://github.com/wavefnx/isup/assets/157986149/23bf68dd-f161-44b7-8c84-b7f14aacf828)
</div>

<div align="center"> 
    
[Overview](#Overview) | [Features](#Features) | [Disclaimer](#Disclaimer)  | [Examples](#Examples)  | [Tests](#Tests) | [Installation](#Installation) | [Usage](#Usage) | [License](#License)
</div>


<div align="center">
    
[![CI](https://img.shields.io/github/actions/workflow/status/wavefnx/isup/ci.yml?style=flat-square&label=CI&labelColor=%23343940&color=%2340C057)](https://github.com/wavefnx/isup/actions/workflows/ci.yml)
[![MPL-2.0](https://img.shields.io/github/license/wavefnx/isup?style=flat-square&color=blue&label=)](LICENSE)
</div>

## Overview
The `isup` library _(is up?)_ provides a set of modules that allow users to build customizable endpoint ranking systems.
- **Distributed Networks**: Utilize `isup` to identify the most reliable and responsive endpoints in a network by utilizing a pre-defined `Strategy` or creating a completely custom one. This is essential when the fastest request gains a competitive edge.
- **Micro-services**: Embed the library inside your code and find the best endpoint to connect upon initialization or create a single-entry endpoint for your services to connect to.
- **Monitoring**: Observe a group of services and detect potential downtime or performance issues.

There are many more ways you can utilize `isup`. Get started either by creating a completely custom solution or by taking advantage of the `Service` provided with the library to quickly set things up.

## Features
- **Custom Strategies**: The `Strategy` trait allows for custom algorithms to be built and produce scores in order to rank your endpoints.
- **Storage Flexibility**:  Choose between multiple options, including `Memory` and `Redis` or implement your own custom solution using the `Store` trait.
- **Lightweight Client**: The `Client` module is using `Hyper` under the hood, for maximum speed and minimal overhead. That allows for precise measurements and minimal memory usage.

## Disclaimer
This library is in early development stages and subject to potential breaking changes, despite its modular design.
Backward compatibility is not guaranteed and the package is intentionally not published on crates.io until and if there's an `alpha` release in the future.

Contributions are welcome. Users are encouraged to submit pull requests, fork, or alter the code in accordance with the terms outlined in the [LICENSE](LICENSE).

## Examples 
The [examples](examples) folder contains a set of comprehensive, well-commented although still non-exaustive ways to utilize the library.
> [!NOTE]
> The memory usage of the examples `(441mb)` doens't reflect the optimised production builds `(10mb)`.

To execute them, run the command below in the root of the repository:

```sh
# e.g. To run the `server` example:
cargo run --example server
```

- [**Server**](examples/server/main.rs): Start a light-weight service that provides a `GET` endpoint, runs a background task and updates scores at the interval specified in the [config](examples/server/config.yml) file.
- [**Manual**](examples/manual.rs):  Showcases a setup with no need for a configuration. This can be the case when all the inputs of the `Service` are well-known and programtically defined. It initiates a blocking loop and prints the best scoring url.
- [**Minimal**](examples/minimal/main.rs): The simplest way to get started. It performs a one-shot update and prints the url with the highest score before it exits. That can be useful when there's the need to connect at random intervals or only once.
- [**Runtime**](examples/runtime.rs): Presents a way to add or remove servers on runtime in order for them to be monitored and scored.

Additionally, while not an executable example by itself, the [Service](src/lib.rs) struct provided with the library shows how the underlying components can be utilized to create your a custom solution.

## Tests
To run the implemented tests, execute the following command at the root of the repository:  
```rust
cargo test
```

## Installation
```toml
[dependencies]
isup = { git = "https://github.com/wavefnx/isup" }
```

## Usage
To get started with a simple setup, you can create a [config](config.example.yml) file in the root of the repository and follow the code below. For more details and ways to use the library, make sure to check out the [examples](#Examples) section.
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the configuration from file
    let config = isup::Config::from_file("isup.config.yml")?;
    
    // Initialize the Service from config
    let service = isup::Service::from_config(config)?;
    
    // Update the scores
    service.update().await?;

    // Retrieve the best scoring URL
    if let Some(url) = service.best_url().await? {
        println!("best: {url}",);
    }

    Ok(())
}
```

## License
This library is released under the terms of the [Mozilla Public License](https://www.mozilla.org/en-US/MPL/) version 2.0. See [LICENSE](LICENSE).
