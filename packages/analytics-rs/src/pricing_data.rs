use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingDataUpdate {
    pub provider: String,
    pub model: String,
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
    pub input_type_factors: HashMap<String, f64>,
    pub version: String,
    pub effective_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingDataValidation {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn validate_pricing_data(update: &PricingDataUpdate) -> PricingDataValidation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Validate prices are non-negative
    if update.input_price_per_1k < 0.0 {
        errors.push("Input price per 1k tokens cannot be negative".to_string());
    }
    if update.output_price_per_1k < 0.0 {
        errors.push("Output price per 1k tokens cannot be negative".to_string());
    }

    // Validate prices are reasonable (not extremely high)
    if update.input_price_per_1k > 1000.0 {
        warnings.push("Input price per 1k tokens is unusually high".to_string());
    }
    if update.output_price_per_1k > 1000.0 {
        warnings.push("Output price per 1k tokens is unusually high".to_string());
    }

    // Validate input type factors
    for (input_type, factor) in &update.input_type_factors {
        if *factor < 0.0 {
            errors.push(format!("Input type factor for {} cannot be negative", input_type));
        }
        if *factor > 100.0 {
            warnings.push(format!("Input type factor for {} is unusually high", input_type));
        }
    }

    // Validate version format
    if update.version.is_empty() {
        errors.push("Version cannot be empty".to_string());
    }

    // Validate effective date format
    if update.effective_date.is_empty() {
        errors.push("Effective date cannot be empty".to_string());
    }

    PricingDataValidation {
        is_valid: errors.is_empty(),
        errors,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_pricing_data_valid() {
        let mut input_type_factors = HashMap::new();
        input_type_factors.insert("text".to_string(), 1.0);
        input_type_factors.insert("image".to_string(), 1.5);

        let update = PricingDataUpdate {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet".to_string(),
            input_price_per_1k: 3.0,
            output_price_per_1k: 15.0,
            input_type_factors,
            version: "2024-01".to_string(),
            effective_date: "2024-01-01".to_string(),
        };

        let validation = validate_pricing_data(&update);
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn test_validate_pricing_data_negative_price() {
        let mut input_type_factors = HashMap::new();
        input_type_factors.insert("text".to_string(), 1.0);

        let update = PricingDataUpdate {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet".to_string(),
            input_price_per_1k: -3.0,
            output_price_per_1k: 15.0,
            input_type_factors,
            version: "2024-01".to_string(),
            effective_date: "2024-01-01".to_string(),
        };

        let validation = validate_pricing_data(&update);
        assert!(!validation.is_valid);
        assert!(!validation.errors.is_empty());
    }

    #[test]
    fn test_validate_pricing_data_high_price_warning() {
        let mut input_type_factors = HashMap::new();
        input_type_factors.insert("text".to_string(), 1.0);

        let update = PricingDataUpdate {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet".to_string(),
            input_price_per_1k: 1500.0,
            output_price_per_1k: 15.0,
            input_type_factors,
            version: "2024-01".to_string(),
            effective_date: "2024-01-01".to_string(),
        };

        let validation = validate_pricing_data(&update);
        assert!(validation.is_valid);
        assert!(!validation.warnings.is_empty());
    }
}
