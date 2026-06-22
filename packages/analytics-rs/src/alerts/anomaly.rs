use super::AnomalySensitivity;
use super::storage::StorageError;

pub struct AnomalyDetector;

impl AnomalyDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect if a value is anomalous based on historical data using z-score analysis
    pub fn detect_anomaly(
        &self,
        _metric: &str,
        value: f64,
        sensitivity: AnomalySensitivity,
        historical_data: &[f64],
    ) -> Result<bool, StorageError> {
        if historical_data.is_empty() {
            return Ok(false); // No baseline data, cannot detect anomaly
        }

        if historical_data.len() < 10 {
            return Ok(false); // Insufficient data for reliable detection
        }

        let (mean, std_dev) = self.calculate_statistics(historical_data);
        
        if std_dev == 0.0 {
            return Ok(false); // No variance, cannot detect anomaly
        }

        let z_score = (value - mean) / std_dev;
        let threshold = self.sensitivity_to_threshold(sensitivity);

        Ok(z_score.abs() > threshold)
    }

    /// Calculate mean and standard deviation of historical data
    fn calculate_statistics(&self, data: &[f64]) -> (f64, f64) {
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        
        let variance = data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / n;
        
        let std_dev = variance.sqrt();
        
        (mean, std_dev)
    }

    /// Convert sensitivity level to z-score threshold
    fn sensitivity_to_threshold(&self, sensitivity: AnomalySensitivity) -> f64 {
        match sensitivity {
            AnomalySensitivity::Low => 3.0,    // 3 standard deviations
            AnomalySensitivity::Medium => 2.5, // 2.5 standard deviations
            AnomalySensitivity::High => 2.0,   // 2 standard deviations
        }
    }

    /// Get the z-score for a value (useful for debugging/monitoring)
    pub fn get_z_score(&self, value: f64, historical_data: &[f64]) -> Option<f64> {
        if historical_data.is_empty() || historical_data.len() < 10 {
            return None;
        }

        let (mean, std_dev) = self.calculate_statistics(historical_data);
        
        if std_dev == 0.0 {
            return None;
        }

        Some((value - mean) / std_dev)
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_anomaly_high_sensitivity() {
        let detector = AnomalyDetector::new();
        let historical_data = vec![10.0, 11.0, 10.5, 9.8, 10.2, 10.7, 9.9, 10.1, 10.3, 10.0];
        
        // Normal value within range
        let result = detector.detect_anomaly("test", 10.5, AnomalySensitivity::High, &historical_data).unwrap();
        assert!(!result);
        
        // Anomalous value far from mean
        let result = detector.detect_anomaly("test", 20.0, AnomalySensitivity::High, &historical_data).unwrap();
        assert!(result);
    }

    #[test]
    fn test_detect_anomaly_low_sensitivity() {
        let detector = AnomalyDetector::new();
        let historical_data = vec![10.0, 11.0, 10.5, 9.8, 10.2, 10.7, 9.9, 10.1, 10.3, 10.0];
        
        // Value within normal range at low sensitivity (close to mean)
        let result = detector.detect_anomaly("test", 10.5, AnomalySensitivity::Low, &historical_data).unwrap();
        assert!(!result);
        
        // Extreme value still anomalous even at low sensitivity
        let result = detector.detect_anomaly("test", 25.0, AnomalySensitivity::Low, &historical_data).unwrap();
        assert!(result);
    }

    #[test]
    fn test_insufficient_data() {
        let detector = AnomalyDetector::new();
        let historical_data = vec![10.0, 11.0, 10.5]; // Less than 10 data points
        
        let result = detector.detect_anomaly("test", 20.0, AnomalySensitivity::High, &historical_data).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_empty_historical_data() {
        let detector = AnomalyDetector::new();
        let historical_data: Vec<f64> = vec![];
        
        let result = detector.detect_anomaly("test", 20.0, AnomalySensitivity::High, &historical_data).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_zero_variance() {
        let detector = AnomalyDetector::new();
        let historical_data = vec![10.0; 20]; // All same values
        
        let result = detector.detect_anomaly("test", 20.0, AnomalySensitivity::High, &historical_data).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_get_z_score() {
        let detector = AnomalyDetector::new();
        let historical_data = vec![10.0, 11.0, 10.5, 9.8, 10.2, 10.7, 9.9, 10.1, 10.3, 10.0];
        
        let z_score = detector.get_z_score(10.5, &historical_data);
        assert!(z_score.is_some());
        assert!(z_score.unwrap().abs() < 1.0); // Should be close to mean
        
        let z_score = detector.get_z_score(20.0, &historical_data);
        assert!(z_score.is_some());
        assert!(z_score.unwrap().abs() > 2.0); // Should be far from mean
    }

    #[test]
    fn test_get_z_score_insufficient_data() {
        let detector = AnomalyDetector::new();
        let historical_data = vec![10.0, 11.0, 10.5];
        
        let z_score = detector.get_z_score(20.0, &historical_data);
        assert!(z_score.is_none());
    }
}