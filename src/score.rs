use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Represents a scoring system for evaluating the performance of a web service.
/// It incorporates various metrics such as response time and reliability
/// to produce a comprehensive performance score.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Score {
    /// The average response time of the service.
    /// This value plays a key role in determining the service's responsiveness and efficiency.
    pub response_avg: Duration,
    /// The calculated score reflecting the overall performance and reliability of the service.
    /// A higher score indicates better performance and reliability.
    pub score: f32,
    /// A measure of the service's reliability, typically based on its success rate of responses.
    /// It is a factor in the overall performance score, with higher reliability leading to a higher score.
    pub reliability: f32,
}

impl Score {
    /// Creates a new `Score` instance with specified initial values.
    ///
    /// # Arguments
    /// * `score`: A floating-point number representing the initial performance score.
    /// * `reliability`: A floating-point number representing the initial reliability measure.
    /// * `response_avg`: A `Duration` representing the initial average response time.
    /// * `status`: A `u16` representing the most recent HTTP status code received.
    ///
    /// # Returns
    /// A new `Score` instance with the provided values.
    pub fn new(score: f32, reliability: f32, response_avg: Duration) -> Self {
        Self { response_avg, score, reliability }
    }
}
