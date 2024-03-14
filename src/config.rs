use crate::{client, request::Request, store, strategy};
use bytes::Bytes;
use hyper::header::{HeaderName, HeaderValue};
use hyper::{HeaderMap, Method, Uri};
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, error::Error, str::FromStr, time::Duration};

/// Main configuration struct containing all other configuration settings for each module.
#[derive(serde::Deserialize, Debug)]
pub struct Config {
    /// Specifies how the HTTP client behaves, particularly concerning request timeouts and
    /// connection pool behavior.
    pub client: Option<client::Config>,
    /// Determines how the performance scores of monitored web services are calculated.
    /// Uses a Default Strategy if none is specified.
    #[serde(default)]
    pub strategy: strategy::Config,
    /// Specifies the mechanism for storing and retrieving monitoring data.
    /// Defaults to the Default Store if not provided.
    #[serde(default)]
    pub store: store::Config,
    /// Can be set, if there's the need to provide an interval for the `run` method from config.
    #[serde(deserialize_with = "deserialize_opt_duration")]
    #[serde(default)]
    pub interval: Option<Duration>,
    /// List of web service requests to monitor.
    pub requests: Vec<Request>,
}

impl Config {
    /// Constructs a `Config` object from a YAML file.
    ///
    /// # Arguments
    /// * `path` - A string slice that holds the path to the config YAML file.
    ///
    /// # Returns
    /// `Config` on success or a `Box<dyn Error>` error caused due to parsing or reading the file.
    pub fn from_file(path: &str) -> Result<Config, Box<dyn Error>> {
        // Read the configuration file into a string.
        let config_str = std::fs::read_to_string(path)?;

        // Deserialize the YAML string into a `Config` object.
        let config = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }
}

/// Deserializes body from a `String` into `Bytes`.
///
/// # Arguments
/// * `deserializer` - A deserializer that implements the `Deserializer` trait.
///
/// # Returns
/// The body `Bytes` on success or a deserialization error on failure.
pub(crate) fn deserialize_body<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let value = serde_json::to_vec(&value).map_err(serde::de::Error::custom)?;

    Ok(Bytes::from(value))
}

/// Deserializes a string into an `Option<Duration>`.
///
/// # Arguments
/// * `deserializer` - A deserializer that implements the `Deserializer` trait.
///
/// # Returns
/// An optional Duration on success or a deserialization error on failure.
pub(crate) fn deserialize_opt_duration<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    match s {
        Some(s) => {
            let duration = humantime::parse_duration(&s).map_err(serde::de::Error::custom)?;
            Ok(Some(duration))
        }
        None => Ok(None),
    }
}

/// Deserialize an HTTP method from a string.
/// Ensures that the provided method is valid and supported.
///
/// ## Arguments
/// * `deserializer`: D - The deserializer used for the HTTP method.
///
/// ## Returns
/// A `Result` that is either a `Method` on success or a deserialization `Error` on failure.
pub(crate) fn deserialize_method<'de, D>(deserializer: D) -> Result<Method, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<Method>().map_err(serde::de::Error::custom)
}
/// Deserialize a URI from a string.
/// Validates the URI and ensures proper formatting.
///
/// ## Arguments
/// * `deserializer`: D - The deserializer used to parse the URL.
///
/// ## Returns
/// A `Result` that is either a `Uri` on success or a deserialization `Error` on failure.
pub(crate) fn deserialize_uri<'de, D>(deserializer: D) -> Result<Uri, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Uri::from_str(&s).map_err(serde::de::Error::custom)
}

/// Deserialize HTTP headers from a HashMap.
/// Converts each key-value pair into a valid HTTP header.
///
/// ## Arguments
/// * `deserializer`: D - The deserializer used for the headers.
///
/// ## Returns
/// A `Result` that is either a `HeaderMap` on success or a deserialization `Error` on failure.
pub(crate) fn deserialize_headers<'de, D>(deserializer: D) -> Result<HeaderMap, D::Error>
where
    D: Deserializer<'de>,
{
    let map: Option<HashMap<String, String>> = Deserialize::deserialize(deserializer)?;
    let headers = map.map_or_else(HeaderMap::new, |m| {
        m.into_iter().fold(HeaderMap::new(), |mut acc, (k, v)| {
            let key = HeaderName::from_str(&k).expect("invalid header name");
            let value = HeaderValue::from_str(&v).expect("invalid header value");
            acc.insert(key, value);
            acc
        })
    });
    Ok(headers)
}
