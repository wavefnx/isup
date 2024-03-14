use super::Strategy;
use crate::score::Score;
use std::time::Duration;

/// A struct for creating a score utilizing HTTP response metrics and a weighted response average.
/// It returns a natural logarithmic score based on the weighted average, response reliability,
/// and HTTP status codes.
#[derive(Debug, serde::Deserialize)]
pub struct WeightedLog {
    /// The weight given to new responses. A value closer to 1.0 gives
    /// more weight to newer responses, whereas a value closer to 0.0
    /// prioritizes historical data.
    pub weight: f32,
    /// A factor that determines the amount of effort a service will require
    // to recover back to it's current score after a failure.
    pub effort: f32,
}

impl Default for WeightedLog {
    /// Provides default values for the `WeightLog` struct.
    fn default() -> Self {
        Self { weight: 0.5, effort: 10.0 }
    }
}

impl WeightedLog {
    /// Represents the weight given to HTTP responses indicating no errors.
    /// A high weight reflects a successful operation or response, such as HTTP status codes
    /// in the range 100-399 (informational, successful, redirection).
    const STATUS_NO_ERROR: f32 = 1.0;
    /// Weight assigned to recoverable error statuses, typically client errors that
    /// may be temporary and have a chance of recovery in subsequent attempts.
    /// This includes specific status codes like 408 (Request Timeout) or 429 (Too Many Requests),
    /// which suggest possible resolution on retrying after some time.
    const STATUS_RECOVERABLE: f32 = 0.7;
    /// Represents the weight for server-side errors (HTTP status codes 500-599).
    /// These errors are generally more critical than recoverable errors but may still be resolved
    /// without requiring client-side changes, hence a moderate weight.
    const STATUS_SERVER_ERROR: f32 = 0.5;
    /// Weight for undefined or unclassified status codes, which fall outside the standard HTTP
    /// status code range (100-599). These indicate unusual or unexpected responses, thus
    /// assigned a lower weight reflecting higher uncertainty and potential issues.
    const STATUS_UNDEFINED: f32 = 0.3;
    /// Weight for non-recoverable client errors (HTTP status codes 400-499 excluding recoverable ones).
    /// These represent significant issues on the client-side and are likely to have the most impact on
    /// the service score, as they often require client-side intervention to resolve.
    const STATUS_NON_RECOVERABLE: f32 = 0.2;
    /// A constant factor used in the calculation of the reliability score.
    /// The reliability factor determines the magnitude of adjustment to the reliability score
    /// based on the outcome of each HTTP request.
    const RELIABILITY_FACTOR: f32 = 0.001;

    /// Constructs a new `WeightLog` instance with specified weight and effort values.
    pub fn new(weight: f32, effort: f32) -> Self {
        Self { weight, effort }
    }

    /// Determines the status weight based on the HTTP status code.
    ///
    /// ## Arguments
    /// * `status`: u16 - The HTTP status code.
    ///
    /// ## Returns
    /// The weight associated with the given status code, influencing the overall score.
    pub(crate) fn get_status_weight(&self, status: u16) -> f32 {
        match status {
            // Apply higher weight for successful, informational, and redirect responses.
            100..=399 => Self::STATUS_NO_ERROR,
            // Apply moderate weight for specific recoverable client errors.
            408 | 429 => Self::STATUS_RECOVERABLE,
            // Apply lower weight for non-recoverable client errors.
            400..=499 => Self::STATUS_NON_RECOVERABLE,
            // Apply moderate weight for server errors.
            500..=599 => Self::STATUS_SERVER_ERROR,
            // Apply lowest weight for undefined or unclassified statuses.
            _ => Self::STATUS_UNDEFINED,
        }
    }

    /// Adjusts the reliability score based on the status code.
    ///
    /// ## Arguments
    /// * `reliability`: f32 - The current reliability score.
    /// * `status_code`: u16 - The HTTP status code.
    ///
    /// ## Returns
    /// The adjusted reliability score after considering the outcome of the operation.
    pub(crate) fn adjust_reliability(&self, reliability: f32, status_code: u16) -> f32 {
        let increment = match status_code {
            // Increase reliability for successful operations.
            200..=299 => Self::RELIABILITY_FACTOR,
            // Keep reliability neutral for info or redirect responses.
            100..=199 | 300..=399 => 0.0,
            // Decrease reliability for failures.
            _ => -(self.effort * Self::RELIABILITY_FACTOR),
        };

        // Ensure the reliability score stays within the bounds of 0.0 to 1.0.
        (reliability + increment).clamp(0.0, 1.0)
    }

    /// Updates the `response` by calculating a weighted average of the existing
    /// (historical) response time and a new response time. This method is designed to
    /// balance recent response time data against historical data, ensuring that the
    /// `response` reflects recent changes while maintaining a degree of stability.
    ///
    /// ## Arguments
    /// * `current`: Duration - The current average response time.
    /// * `new`: Duration - The latest response time measurement.
    ///
    /// ## Returns
    /// The updated average response time as a `Duration`.
    pub(crate) fn weighted_response_average(&self, current: Duration, new: Duration) -> Duration {
        // Weight for the historical response time.
        let weight_historical = 1.0 - self.weight;
        // Calculating weighted historical response time.
        let weighted_historical_response = weight_historical * current.as_nanos() as f32;
        // Calculating weighted new response time.
        let weighted_new_response = self.weight * new.as_nanos() as f32;
        // Compute the weighted average of the historical and new response times.
        let average_response = weighted_historical_response + weighted_new_response;
        // Update the response time to the new weighted average.
        Duration::from_nanos(average_response as u64)
    }
    /// Calculates the total score using a logarithmic function. The score combines
    /// the effects of reliability, status weight, and response time into a single measure.
    ///
    /// ## Arguments
    /// * `reliability`: f32 - The current reliability score.
    /// * `status_weight`: f32 - The weight assigned based on the HTTP status code.
    /// * `response`: Duration - The current response time.
    ///
    /// ## Returns
    /// The calculated logarithmic score as a floating-point number.
    pub(crate) fn calculate_logarithmic_score(&self, reliability: f32, status_weight: f32, response: Duration) -> f32 {
        // Influence of response time on score, adjusted by status weight.
        let response_influence = 0.1 + (0.5 - status_weight).abs() * 0.15;
        // Calculate the response time factor.
        let response_factor = 1.0 / (1.0 + response.as_secs_f32() * response_influence);
        // Base score combining reliability, status weight, and response time factor.
        let base_score = reliability * status_weight * response_factor + 1.0;
        // Apply logarithm to base score for final total score.
        base_score.ln()
    }
}

impl Strategy for WeightedLog {
    /// Implementation of `calculate` for `WeightLog`.
    ///
    /// It uses a weighted log approach to update the score based on the new response and status code.
    ///
    /// # Arguments
    /// * `score`: The current score before this calculation.
    /// * `new_response`: The new response time, to be integrated into the score.
    /// * `status_code`: The HTTP status code of the new response.
    ///
    /// # Returns
    /// A new `Score` instance representing the updated score.
    fn calculate(&self, score: Score, new_response: Duration, status_code: u16) -> Score {
        // Determine the weight associated with the given status code.
        let status_weight = self.get_status_weight(status_code);
        // Calculate the weighted average of the response time.
        let response = self.weighted_response_average(score.response_avg, new_response);
        // Adjust the reliability based on the status code.
        let reliability = self.adjust_reliability(score.reliability, status_code);
        // Calculate the new score using the updated parameters.
        let score = self.calculate_logarithmic_score(reliability, status_weight, new_response);
        // Return a new Score instance with the updated values.
        Score::new(score, reliability, response)
    }
}
