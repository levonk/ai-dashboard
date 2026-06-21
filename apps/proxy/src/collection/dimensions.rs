use anyhow::Result;
use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Multi-dimensional attribute collection
/// Collects and categorizes attributes across different dimensions for rich analytics
pub struct DimensionCollector {
    dimensions: HashMap<String, Dimension>,
}

impl DimensionCollector {
    /// Create a new dimension collector
    pub fn new() -> Self {
        Self {
            dimensions: HashMap::new(),
        }
    }

    /// Add a dimension to track
    pub fn add_dimension(&mut self, dimension: Dimension) {
        self.dimensions.insert(dimension.name.clone(), dimension);
    }

    /// Record a value for a dimension
    pub fn record_dimension_value(&mut self, dimension_name: &str, value: String) -> Result<()> {
        if let Some(dimension) = self.dimensions.get_mut(dimension_name) {
            dimension.record_value(value);
            Ok(())
        } else {
            anyhow::bail!("Dimension '{}' not found", dimension_name);
        }
    }

    /// Get dimension by name
    pub fn get_dimension(&self, name: &str) -> Option<&Dimension> {
        self.dimensions.get(name)
    }

    /// Get all dimension names
    pub fn get_dimension_names(&self) -> Vec<String> {
        self.dimensions.keys().cloned().collect()
    }

    /// Get dimension statistics
    pub fn get_dimension_stats(&self, dimension_name: &str) -> Option<DimensionStats> {
        self.dimensions.get(dimension_name).map(|d| d.get_stats())
    }

    /// Collect all dimensions into a structured format
    pub fn collect_dimensions(&self) -> HashMap<String, DimensionData> {
        self.dimensions
            .iter()
            .map(|(name, dimension)| {
                (name.clone(), dimension.collect_data())
            })
            .collect()
    }

    /// Initialize default dimensions
    pub fn initialize_default_dimensions(&mut self) {
        // Geographic dimension
        self.add_dimension(Dimension::new(
            "geography".to_string(),
            DimensionType::Categorical,
            vec!["us".to_string(), "eu".to_string(), "asia".to_string(), "other".to_string()],
        ));

        // Organization dimension
        self.add_dimension(Dimension::new(
            "organization".to_string(),
            DimensionType::Categorical,
            vec![],
        ));

        // User segment dimension
        self.add_dimension(Dimension::new(
            "user_segment".to_string(),
            DimensionType::Categorical,
            vec!["free".to_string(), "pro".to_string(), "enterprise".to_string()],
        ));

        // Environment dimension
        self.add_dimension(Dimension::new(
            "environment".to_string(),
            DimensionType::Categorical,
            vec!["development".to_string(), "staging".to_string(), "production".to_string()],
        ));

        // Time dimension
        self.add_dimension(Dimension::new(
            "time_of_day".to_string(),
            DimensionType::Temporal,
            vec![],
        ));

        // Request size dimension
        self.add_dimension(Dimension::new(
            "request_size".to_string(),
            DimensionType::Numerical,
            vec![],
        ));
    }
}

impl Default for DimensionCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Dimension types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DimensionType {
    Categorical,
    Numerical,
    Temporal,
    Boolean,
}

/// A single dimension for tracking attributes
#[derive(Debug, Clone)]
pub struct Dimension {
    pub name: String,
    pub dimension_type: DimensionType,
    pub allowed_values: Vec<String>,
    pub value_counts: HashMap<String, u64>,
    pub total_count: u64,
}

impl Dimension {
    /// Create a new dimension
    pub fn new(name: String, dimension_type: DimensionType, allowed_values: Vec<String>) -> Self {
        Self {
            name,
            dimension_type,
            allowed_values,
            value_counts: HashMap::new(),
            total_count: 0,
        }
    }

    /// Record a value for this dimension
    pub fn record_value(&mut self, value: String) {
        // Validate against allowed values if specified
        if !self.allowed_values.is_empty() && !self.allowed_values.contains(&value) {
            return; // Skip invalid values
        }

        *self.value_counts.entry(value).or_insert(0) += 1;
        self.total_count += 1;
    }

    /// Get statistics for this dimension
    pub fn get_stats(&self) -> DimensionStats {
        let most_common = self.value_counts
            .iter()
            .max_by(|a, b| a.1.cmp(b.1))
            .map(|(k, v)| (k.clone(), *v));

        DimensionStats {
            name: self.name.clone(),
            dimension_type: self.dimension_type.clone(),
            total_count: self.total_count,
            unique_values: self.value_counts.len() as u64,
            most_common,
        }
    }

    /// Collect dimension data for export
    pub fn collect_data(&self) -> DimensionData {
        DimensionData {
            name: self.name.clone(),
            dimension_type: self.dimension_type.clone(),
            value_distribution: self.value_counts.clone(),
            total_count: self.total_count,
        }
    }

    /// Get value distribution as percentages
    pub fn get_distribution_percentages(&self) -> HashMap<String, f64> {
        if self.total_count == 0 {
            return HashMap::new();
        }

        self.value_counts
            .iter()
            .map(|(k, v)| (k.clone(), (*v as f64 / self.total_count as f64) * 100.0))
            .collect()
    }
}

/// Dimension statistics
#[derive(Debug, Clone)]
pub struct DimensionStats {
    pub name: String,
    pub dimension_type: DimensionType,
    pub total_count: u64,
    pub unique_values: u64,
    pub most_common: Option<(String, u64)>,
}

/// Dimension data for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionData {
    pub name: String,
    pub dimension_type: DimensionType,
    pub value_distribution: HashMap<String, u64>,
    pub total_count: u64,
}

/// Multi-dimensional attribute record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeRecord {
    pub request_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub dimensions: HashMap<String, String>,
}

impl AttributeRecord {
    /// Create a new attribute record
    pub fn new(request_id: String) -> Self {
        Self {
            request_id,
            timestamp: chrono::Utc::now(),
            dimensions: HashMap::new(),
        }
    }

    /// Add a dimension value
    pub fn with_dimension(mut self, name: String, value: String) -> Self {
        self.dimensions.insert(name, value);
        self
    }

    /// Get a dimension value
    pub fn get_dimension(&self, name: &str) -> Option<&String> {
        self.dimensions.get(name)
    }

    /// Check if has a specific dimension
    pub fn has_dimension(&self, name: &str) -> bool {
        self.dimensions.contains_key(name)
    }
}

/// Dimension extractor for pulling dimensions from requests
pub struct DimensionExtractor;

impl DimensionExtractor {
    /// Extract geographic dimension from request
    pub fn extract_geography(headers: &HashMap<String, String>) -> Option<String> {
        headers.get("x-geo-region").cloned()
            .or_else(|| headers.get("cf-ipcountry").cloned())
    }

    /// Extract organization dimension from request
    pub fn extract_organization(headers: &HashMap<String, String>) -> Option<String> {
        headers.get("x-org-id").cloned()
    }

    /// Extract user segment from request
    pub fn extract_user_segment(headers: &HashMap<String, String>) -> Option<String> {
        headers.get("x-user-segment").cloned()
    }

    /// Extract environment from request
    pub fn extract_environment(context: &HashMap<String, String>) -> String {
        context.get("environment")
            .cloned()
            .unwrap_or_else(|| "production".to_string())
    }

    /// Extract time of day dimension
    pub fn extract_time_of_day() -> String {
        let hour = chrono::Utc::now().hour();
        
        match hour {
            0..=5 => "night".to_string(),
            6..=11 => "morning".to_string(),
            12..=17 => "afternoon".to_string(),
            18..=23 => "evening".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Extract request size dimension
    pub fn extract_request_size(request: &serde_json::Value) -> String {
        let content = request.to_string();
        let size = content.len();
        
        match size {
            0..=1000 => "small".to_string(),
            1001..=10000 => "medium".to_string(),
            10001..=100000 => "large".to_string(),
            _ => "extra_large".to_string(),
        }
    }

    /// Extract all dimensions from a request
    pub fn extract_all_dimensions(
        headers: &HashMap<String, String>,
        context: &HashMap<String, String>,
        request: &serde_json::Value,
    ) -> HashMap<String, String> {
        let mut dimensions = HashMap::new();

        if let Some(geo) = Self::extract_geography(headers) {
            dimensions.insert("geography".to_string(), geo);
        }

        if let Some(org) = Self::extract_organization(headers) {
            dimensions.insert("organization".to_string(), org);
        }

        if let Some(segment) = Self::extract_user_segment(headers) {
            dimensions.insert("user_segment".to_string(), segment);
        }

        dimensions.insert("environment".to_string(), Self::extract_environment(context));
        dimensions.insert("time_of_day".to_string(), Self::extract_time_of_day());
        dimensions.insert("request_size".to_string(), Self::extract_request_size(request));

        dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_collector_creation() {
        let collector = DimensionCollector::new();
        assert_eq!(collector.get_dimension_names().len(), 0);
    }

    #[test]
    fn test_add_dimension() {
        let mut collector = DimensionCollector::new();
        let dimension = Dimension::new(
            "test".to_string(),
            DimensionType::Categorical,
            vec!["value1".to_string()],
        );
        
        collector.add_dimension(dimension);
        assert_eq!(collector.get_dimension_names().len(), 1);
    }

    #[test]
    fn test_record_dimension_value() {
        let mut collector = DimensionCollector::new();
        collector.add_dimension(Dimension::new(
            "test".to_string(),
            DimensionType::Categorical,
            vec!["value1".to_string(), "value2".to_string()],
        ));
        
        collector.record_dimension_value("test", "value1".to_string()).unwrap();
        
        let stats = collector.get_dimension_stats("test").unwrap();
        assert_eq!(stats.total_count, 1);
    }

    #[test]
    fn test_record_invalid_value() {
        let mut collector = DimensionCollector::new();
        collector.add_dimension(Dimension::new(
            "test".to_string(),
            DimensionType::Categorical,
            vec!["value1".to_string()], // Only value1 is allowed
        ));
        
        collector.record_dimension_value("test", "value2".to_string()).unwrap();
        
        let stats = collector.get_dimension_stats("test").unwrap();
        assert_eq!(stats.total_count, 0); // Invalid value should be skipped
    }

    #[test]
    fn test_initialize_default_dimensions() {
        let mut collector = DimensionCollector::new();
        collector.initialize_default_dimensions();
        
        assert!(collector.get_dimension_names().contains(&"geography".to_string()));
        assert!(collector.get_dimension_names().contains(&"organization".to_string()));
        assert!(collector.get_dimension_names().contains(&"user_segment".to_string()));
    }

    #[test]
    fn test_dimension_stats() {
        let mut dimension = Dimension::new(
            "test".to_string(),
            DimensionType::Categorical,
            vec![],
        );
        
        dimension.record_value("value1".to_string());
        dimension.record_value("value1".to_string());
        dimension.record_value("value2".to_string());
        
        let stats = dimension.get_stats();
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.unique_values, 2);
    }

    #[test]
    fn test_get_distribution_percentages() {
        let mut dimension = Dimension::new(
            "test".to_string(),
            DimensionType::Categorical,
            vec![],
        );
        
        dimension.record_value("value1".to_string());
        dimension.record_value("value1".to_string());
        dimension.record_value("value2".to_string());
        
        let distribution = dimension.get_distribution_percentages();
        assert_eq!(distribution.get("value1"), Some(&66.66666666666666));
        assert_eq!(distribution.get("value2"), Some(&33.33333333333333));
    }

    #[test]
    fn test_attribute_record() {
        let record = AttributeRecord::new("req-123".to_string())
            .with_dimension("geo".to_string(), "us".to_string())
            .with_dimension("segment".to_string(), "pro".to_string());
        
        assert_eq!(record.get_dimension("geo"), Some(&"us".to_string()));
        assert!(record.has_dimension("segment"));
    }

    #[test]
    fn test_extract_geography() {
        let mut headers = HashMap::new();
        headers.insert("x-geo-region".to_string(), "us".to_string());
        
        let geo = DimensionExtractor::extract_geography(&headers);
        assert_eq!(geo, Some("us".to_string()));
    }

    #[test]
    fn test_extract_environment() {
        let mut context = HashMap::new();
        context.insert("environment".to_string(), "staging".to_string());
        
        let env = DimensionExtractor::extract_environment(&context);
        assert_eq!(env, "staging");
    }

    #[test]
    fn test_extract_time_of_day() {
        let time = DimensionExtractor::extract_time_of_day();
        assert!["night", "morning", "afternoon", "evening"].contains(&time.as_str());
    }

    #[test]
    fn test_extract_request_size() {
        let small_request = serde_json::json!({"prompt": "Hi"});
        let large_request = serde_json::json!({"prompt": "A".repeat(20000)});
        
        let small_size = DimensionExtractor::extract_request_size(&small_request);
        let large_size = DimensionExtractor::extract_request_size(&large_request);
        
        assert_eq!(small_size, "small");
        assert_eq!(large_size, "extra_large");
    }

    #[test]
    fn test_extract_all_dimensions() {
        let mut headers = HashMap::new();
        headers.insert("x-geo-region".to_string(), "us".to_string());
        
        let mut context = HashMap::new();
        context.insert("environment".to_string(), "production".to_string());
        
        let request = serde_json::json!({"prompt": "Hello"});
        
        let dimensions = DimensionExtractor::extract_all_dimensions(&headers, &context, &request);
        
        assert_eq!(dimensions.get("geography"), Some(&"us".to_string()));
        assert_eq!(dimensions.get("environment"), Some(&"production".to_string()));
        assert!(dimensions.contains_key("time_of_day"));
    }
}