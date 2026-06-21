use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error tracking and classification
/// Collects, classifies, and analyzes errors from AI requests
pub struct ErrorCollector {
    error_counts: HashMap<String, u64>,
    error_history: Vec<ErrorRecord>,
}

impl ErrorCollector {
    /// Create a new error collector
    pub fn new() -> Self {
        Self {
            error_counts: HashMap::new(),
            error_history: Vec::new(),
        }
    }

    /// Record an error
    pub fn record_error(&mut self, error: ErrorRecord) {
        let error_type = error.error_type.clone();
        *self.error_counts.entry(error_type).or_insert(0) += 1;
        self.error_history.push(error);
    }

    /// Get error count by type
    pub fn get_error_count(&self, error_type: &str) -> u64 {
        *self.error_counts.get(error_type).unwrap_or(&0)
    }

    /// Get total error count
    pub fn total_errors(&self) -> u64 {
        self.error_counts.values().sum()
    }

    /// Get error rate (errors / total requests)
    pub fn error_rate(&self, total_requests: u64) -> f64 {
        if total_requests > 0 {
            self.total_errors() as f64 / total_requests as f64
        } else {
            0.0
        }
    }

    /// Get most common error types
    pub fn get_common_errors(&self, limit: usize) -> Vec<(String, u64)> {
        let mut errors: Vec<(String, u64)> = self.error_counts
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        
        errors.sort_by(|a, b| b.1.cmp(&a.1));
        errors.truncate(limit);
        errors
    }

    /// Clear error history
    pub fn clear_history(&mut self) {
        self.error_history.clear();
    }

    /// Reset error counts
    pub fn reset_counts(&mut self) {
        self.error_counts.clear();
    }
}

impl Default for ErrorCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Error classification types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ErrorType {
    /// Authentication/authorization errors
    Authentication,
    /// Rate limiting errors
    RateLimit,
    /// Network connectivity errors
    Network,
    /// Provider API errors
    Provider,
    /// Validation errors
    Validation,
    /// Timeout errors
    Timeout,
    /// Internal server errors
    Internal,
    /// Unknown errors
    Unknown,
}

impl ErrorType {
    /// Classify an error from error message and status code
    pub fn classify(message: &str, status_code: Option<u16>) -> Self {
        let lower_message = message.to_lowercase();
        
        // Check for authentication errors
        if lower_message.contains("unauthorized") || 
           lower_message.contains("authentication") ||
           lower_message.contains("forbidden") ||
           status_code == Some(401) || status_code == Some(403) {
            return ErrorType::Authentication;
        }
        
        // Check for rate limiting
        if lower_message.contains("rate limit") || 
           lower_message.contains("too many requests") ||
           status_code == Some(429) {
            return ErrorType::RateLimit;
        }
        
        // Check for network errors
        if lower_message.contains("network") || 
           lower_message.contains("connection") ||
           lower_message.contains("timeout") ||
           lower_message.contains("dns") {
            return ErrorType::Network;
        }
        
        // Check for provider errors
        if lower_message.contains("provider") || 
           lower_message.contains("api") ||
           lower_message.contains("service unavailable") ||
           status_code == Some(502) || status_code == Some(503) {
            return ErrorType::Provider;
        }
        
        // Check for validation errors
        if lower_message.contains("validation") || 
           lower_message.contains("invalid") ||
           lower_message.contains("bad request") ||
           status_code == Some(400) {
            return ErrorType::Validation;
        }
        
        // Check for timeout errors
        if lower_message.contains("timeout") || status_code == Some(408) {
            return ErrorType::Timeout;
        }
        
        // Check for internal errors
        if lower_message.contains("internal") || status_code == Some(500) {
            return ErrorType::Internal;
        }
        
        ErrorType::Unknown
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorType::Authentication => "authentication",
            ErrorType::RateLimit => "rate_limit",
            ErrorType::Network => "network",
            ErrorType::Provider => "provider",
            ErrorType::Validation => "validation",
            ErrorType::Timeout => "timeout",
            ErrorType::Internal => "internal",
            ErrorType::Unknown => "unknown",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s {
            "authentication" => ErrorType::Authentication,
            "rate_limit" => ErrorType::RateLimit,
            "network" => ErrorType::Network,
            "provider" => ErrorType::Provider,
            "validation" => ErrorType::Validation,
            "timeout" => ErrorType::Timeout,
            "internal" => ErrorType::Internal,
            _ => ErrorType::Unknown,
        }
    }
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ErrorSeverity {
    /// Determine severity from error type and context
    pub fn from_error_type(error_type: &ErrorType, is_recurring: bool) -> Self {
        match error_type {
            ErrorType::Authentication => ErrorSeverity::High,
            ErrorType::RateLimit => ErrorSeverity::Medium,
            ErrorType::Network => if is_recurring { ErrorSeverity::High } else { ErrorSeverity::Medium },
            ErrorType::Provider => ErrorSeverity::High,
            ErrorType::Validation => ErrorSeverity::Low,
            ErrorType::Timeout => ErrorSeverity::Medium,
            ErrorType::Internal => ErrorSeverity::Critical,
            ErrorType::Unknown => if is_recurring { ErrorSeverity::Medium } else { ErrorSeverity::Low },
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorSeverity::Low => "low",
            ErrorSeverity::Medium => "medium",
            ErrorSeverity::High => "high",
            ErrorSeverity::Critical => "critical",
        }
    }
}

/// Error record for tracking individual errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    pub error_id: String,
    pub error_type: String,
    pub severity: String,
    pub message: String,
    pub status_code: Option<u16>,
    pub timestamp: DateTime<Utc>,
    pub request_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub context: HashMap<String, String>,
}

impl ErrorRecord {
    /// Create a new error record
    pub fn new(
        message: String,
        status_code: Option<u16>,
        request_id: Option<String>,
        provider: Option<String>,
        model: Option<String>,
    ) -> Self {
        let error_type = ErrorType::classify(&message, status_code);
        let severity = ErrorSeverity::from_error_type(&error_type, false);
        
        Self {
            error_id: uuid::Uuid::new_v4().to_string(),
            error_type: error_type.as_str().to_string(),
            severity: severity.as_str().to_string(),
            message,
            status_code,
            timestamp: Utc::now(),
            request_id,
            provider,
            model,
            context: HashMap::new(),
        }
    }

    /// Add context information
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }

    /// Check if error is recurring (based on type and message similarity)
    pub fn is_recurring(&self, other: &ErrorRecord) -> bool {
        self.error_type == other.error_type && 
        self.message == other.message
    }
}

/// Error statistics for reporting
#[derive(Debug, Clone)]
pub struct ErrorStats {
    pub total_errors: u64,
    pub by_type: HashMap<String, u64>,
    pub by_severity: HashMap<String, u64>,
    pub error_rate: f64,
    pub most_common: Vec<(String, u64)>,
}

impl ErrorStats {
    /// Calculate error statistics from collector
    pub fn from_collector(collector: &ErrorCollector, total_requests: u64) -> Self {
        let mut by_severity = HashMap::new();
        
        for record in &collector.error_history {
            *by_severity.entry(record.severity.clone()).or_insert(0) += 1;
        }
        
        Self {
            total_errors: collector.total_errors(),
            by_type: collector.error_counts.clone(),
            by_severity,
            error_rate: collector.error_rate(total_requests),
            most_common: collector.get_common_errors(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_collector_creation() {
        let collector = ErrorCollector::new();
        assert_eq!(collector.total_errors(), 0);
    }

    #[test]
    fn test_record_error() {
        let mut collector = ErrorCollector::new();
        let error = ErrorRecord::new(
            "Test error".to_string(),
            Some(500),
            Some("req-123".to_string()),
            None,
            None,
        );
        
        collector.record_error(error);
        assert_eq!(collector.total_errors(), 1);
    }

    #[test]
    fn test_error_count_by_type() {
        let mut collector = ErrorCollector::new();
        let error = ErrorRecord::new(
            "Unauthorized access".to_string(),
            Some(401),
            None,
            None,
            None,
        );
        
        collector.record_error(error);
        assert_eq!(collector.get_error_count("authentication"), 1);
    }

    #[test]
    fn test_error_rate() {
        let mut collector = ErrorCollector::new();
        let error = ErrorRecord::new(
            "Test error".to_string(),
            Some(500),
            None,
            None,
            None,
        );
        
        collector.record_error(error);
        assert_eq!(collector.error_rate(10), 0.1);
    }

    #[test]
    fn test_error_type_classification() {
        assert_eq!(
            ErrorType::classify("Unauthorized access", Some(401)),
            ErrorType::Authentication
        );
        assert_eq!(
            ErrorType::classify("Rate limit exceeded", Some(429)),
            ErrorType::RateLimit
        );
        assert_eq!(
            ErrorType::classify("Network error", None),
            ErrorType::Network
        );
    }

    #[test]
    fn test_error_severity() {
        let severity = ErrorSeverity::from_error_type(&ErrorType::Authentication, false);
        assert_eq!(severity, ErrorSeverity::High);
        
        let severity = ErrorSeverity::from_error_type(&ErrorType::Internal, false);
        assert_eq!(severity, ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_record_creation() {
        let record = ErrorRecord::new(
            "Test error".to_string(),
            Some(500),
            Some("req-123".to_string()),
            Some("anthropic".to_string()),
            Some("claude-3-opus".to_string()),
        );
        
        assert_eq!(record.message, "Test error");
        assert_eq!(record.status_code, Some(500));
        assert_eq!(record.provider, Some("anthropic".to_string()));
    }

    #[test]
    fn test_error_record_with_context() {
        let record = ErrorRecord::new(
            "Test error".to_string(),
            None,
            None,
            None,
            None,
        ).with_context("key".to_string(), "value".to_string());
        
        assert!(record.context.contains_key("key"));
    }

    #[test]
    fn test_is_recurring() {
        let record1 = ErrorRecord::new(
            "Test error".to_string(),
            Some(500),
            None,
            None,
            None,
        );
        
        let record2 = ErrorRecord::new(
            "Test error".to_string(),
            Some(500),
            None,
            None,
            None,
        );
        
        assert!(record1.is_recurring(&record2));
    }

    #[test]
    fn test_get_common_errors() {
        let mut collector = ErrorCollector::new();
        
        for _ in 0..3 {
            let error = ErrorRecord::new(
                "Authentication error".to_string(),
                Some(401),
                None,
                None,
                None,
            );
            collector.record_error(error);
        }
        
        for _ in 0..2 {
            let error = ErrorRecord::new(
                "Network error".to_string(),
                None,
                None,
                None,
                None,
            );
            collector.record_error(error);
        }
        
        let common = collector.get_common_errors(2);
        assert_eq!(common.len(), 2);
        assert_eq!(common[0].1, 3); // Most common
    }

    #[test]
    fn test_error_stats() {
        let mut collector = ErrorCollector::new();
        
        let error = ErrorRecord::new(
            "Test error".to_string(),
            Some(500),
            None,
            None,
            None,
        );
        collector.record_error(error);
        
        let stats = ErrorStats::from_collector(&collector, 10);
        assert_eq!(stats.total_errors, 1);
        assert_eq!(stats.error_rate, 0.1);
    }
}