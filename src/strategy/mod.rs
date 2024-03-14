use crate::score::Score;
use std::time::Duration;

mod weighted_log;
pub use weighted_log::WeightedLog;

/// Defines the configuration options for different scoring strategies.
///
/// The `Config` enum allows the selection of different scoring strategies through configuration.
/// Currently, it supports the `WeightedLog` strategy, which can be expanded to include more strategies in the future.
#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum Config {
    /// Configuration for the Weighted Logarithmic strategy.
    /// It is designed to provide a score based on weighted response times.
    WeightedLog(weighted_log::WeightedLog),
}

impl Default for Config {
    /// Provides a default configuration, which currently is the `WeightedLog` strategy.
    fn default() -> Self {
        Config::WeightedLog(weighted_log::WeightedLog::default())
    }
}

/// Creates a scoring strategy instance from the given configuration.
///
/// This function is responsible for interpreting the configuration and initializing
/// the appropriate scoring strategy based on it.
///
/// # Arguments
/// * `config` - The configuration for the scoring strategy.
///
/// # Returns
/// A boxed instance of a scoring strategy, implementing the `Strategy` trait.
pub fn from_config(config: Config) -> Box<dyn Strategy + Sync + Send + 'static> {
    match config {
        // Constructs a `WeightedLog` strategy based on the provided configuration.
        Config::WeightedLog(config) => Box::new(config),
    }
}

/// Trait defining the strategy for score calculation.
pub trait Strategy {
    /// Calculates a new `Score` based on the previous score, new response time, and the HTTP status code.
    ///
    /// # Arguments
    /// * `score`: The current score before this calculation.
    /// * `new_response`: The most recent response time to be factored into the score.
    /// * `status_code`: The HTTP status code of the new response, which affects score calculation.
    ///
    /// # Returns
    /// A new `Score` instance representing the updated score after applying the strategy.
    fn calculate(&self, score: Score, new_response: Duration, status_code: u16) -> Score;
}
