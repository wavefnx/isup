#[cfg(test)]
mod strategy_tests {
    use std::time::Duration;

    use isup::{
        strategy::{Strategy, WeightedLog},
        Score,
    };

    #[test]
    fn it_calculates_weighted_log() {
        // Create a new weighted log strategy
        let strategy = WeightedLog::new(0.5, 10.0);
        // Define the previous duration and the new mock durations
        let previous_duration = Duration::from_millis(500);
        let new_duration = Duration::from_millis(300);

        // Create a new score with the previous duration
        let score = Score::new(0.0, 0.0, previous_duration);

        // Calculate the weighted log
        let weighted = strategy.calculate(score, new_duration, 200);
        assert_eq!(weighted.response_avg, Duration::from_millis(400));
        assert_eq!(weighted.reliability, 0.001);
        assert_eq!(weighted.score, 0.000949647);

        // Pass the previous weighted score to the strategy immitating a second measurement
        let weighted = strategy.calculate(weighted, new_duration, 200);
        assert_eq!(weighted.response_avg, Duration::from_millis(350));
        assert_eq!(weighted.reliability, 0.002);
        assert_eq!(weighted.score, 0.001898393);
    }
}
