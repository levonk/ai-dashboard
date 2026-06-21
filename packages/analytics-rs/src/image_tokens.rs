use serde::{Serialize, Deserialize};
use crate::tokens::{TokenEstimate, InputType, EstimationMethod};

/// Image token estimation for vision models
/// 
/// This module provides token counting for image inputs across different
/// vision-capable models (GPT-4 Vision, Claude 3, Gemini Pro Vision, etc.)

/// Image metadata for token estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub detail_level: ImageDetail,
}

/// Image format types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImageFormat {
    PNG,
    JPEG,
    WEBP,
    GIF,
    Unknown,
}

/// Image detail level for token estimation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImageDetail {
    Low,    // ~85 tokens
    Medium, // ~170 tokens
    High,   // ~765 tokens (base) + scaling
    Auto,   // Model decides
}

/// Image token estimator
pub struct ImageTokenEstimator;

impl ImageTokenEstimator {
    pub fn new() -> Self {
        Self
    }
    
    /// Estimate tokens for an image based on dimensions and detail level
    /// 
    /// # Arguments
    /// * `metadata` - Image metadata including dimensions and format
    /// * `model` - The model being used (affects token counting)
    /// 
    /// # Returns
    /// Token estimate with confidence and method
    pub fn estimate_image_tokens(&self, metadata: &ImageMetadata, model: &str) -> TokenEstimate {
        let base = match metadata.detail_level {
            ImageDetail::Low => 85,
            ImageDetail::Medium => 170,
            ImageDetail::High => self.calculate_high_detail_tokens(metadata),
            ImageDetail::Auto => 170, // Default to medium for auto
        };
        
        // Model-specific adjustments
        let adjusted = self.apply_model_adjustments(base, model);
        
        TokenEstimate {
            estimated_tokens: adjusted,
            model: model.to_string(),
            input_type: InputType::Image,
            confidence: 0.80, // Image estimation is reasonably accurate
            method: EstimationMethod::ModelSpecific,
        }
    }
    
    /// Calculate tokens for high-detail images based on dimensions
    /// 
    /// High detail uses a base of 765 tokens plus scaling based on image size.
    /// The formula is: base + (width * height / scale_factor)
    fn calculate_high_detail_tokens(&self, metadata: &ImageMetadata) -> u32 {
        let base = 765;
        let scale_factor = 512.0 * 512.0; // 512x512 reference
        let area = metadata.width as f64 * metadata.height as f64;
        let scaling = (area / scale_factor).ceil() as u32;
        
        base + scaling
    }
    
    /// Apply model-specific adjustments to token count
    fn apply_model_adjustments(&self, base_tokens: u32, model: &str) -> u32 {
        match model {
            // GPT-4 Vision uses the standard calculation
            "gpt-4-vision-preview" | "gpt-4o" => base_tokens,
            
            // Claude 3 has slightly different token counting
            "claude-3-opus" | "claude-3-sonnet" | "claude-3-haiku" => {
                // Claude 3 uses a similar but not identical scheme
                // Adjust based on known differences
                (base_tokens as f64 * 1.1).ceil() as u32
            }
            
            // Gemini Pro Vision
            "gemini-pro-vision" => {
                // Gemini has its own token counting scheme
                (base_tokens as f64 * 0.9).ceil() as u32
            }
            
            // Default to base calculation
            _ => base_tokens,
        }
    }
    
    /// Estimate tokens for multiple images (e.g., in a batch)
    pub fn estimate_batch_tokens(&self, images: &[ImageMetadata], model: &str) -> TokenEstimate {
        let total: u32 = images.iter()
            .map(|img| self.estimate_image_tokens(img, model).estimated_tokens)
            .sum();
        
        TokenEstimate {
            estimated_tokens: total,
            model: model.to_string(),
            input_type: InputType::Image,
            confidence: 0.75, // Batch estimation has slightly lower confidence
            method: EstimationMethod::ModelSpecific,
        }
    }
    
    /// Get recommended detail level for an image based on size
    pub fn recommend_detail_level(&self, metadata: &ImageMetadata) -> ImageDetail {
        let area = metadata.width * metadata.height;
        
        if area < 512 * 512 {
            ImageDetail::Low
        } else if area < 1024 * 1024 {
            ImageDetail::Medium
        } else {
            ImageDetail::High
        }
    }
}

impl Default for ImageTokenEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_low_detail() {
        let estimator = ImageTokenEstimator::new();
        let metadata = ImageMetadata {
            width: 512,
            height: 512,
            format: ImageFormat::PNG,
            detail_level: ImageDetail::Low,
        };
        
        let estimate = estimator.estimate_image_tokens(&metadata, "gpt-4-vision-preview");
        assert_eq!(estimate.estimated_tokens, 85);
        assert_eq!(estimate.input_type, InputType::Image);
    }
    
    #[test]
    fn test_estimate_medium_detail() {
        let estimator = ImageTokenEstimator::new();
        let metadata = ImageMetadata {
            width: 512,
            height: 512,
            format: ImageFormat::JPEG,
            detail_level: ImageDetail::Medium,
        };
        
        let estimate = estimator.estimate_image_tokens(&metadata, "gpt-4-vision-preview");
        assert_eq!(estimate.estimated_tokens, 170);
    }
    
    #[test]
    fn test_estimate_high_detail() {
        let estimator = ImageTokenEstimator::new();
        let metadata = ImageMetadata {
            width: 1024,
            height: 1024,
            format: ImageFormat::PNG,
            detail_level: ImageDetail::High,
        };
        
        let estimate = estimator.estimate_image_tokens(&metadata, "gpt-4-vision-preview");
        // Base 765 + scaling for 1024x1024 (4x 512x512)
        assert!(estimate.estimated_tokens > 765);
    }
    
    #[test]
    fn test_model_adjustments() {
        let estimator = ImageTokenEstimator::new();
        let metadata = ImageMetadata {
            width: 512,
            height: 512,
            format: ImageFormat::PNG,
            detail_level: ImageDetail::Medium,
        };
        
        let gpt_estimate = estimator.estimate_image_tokens(&metadata, "gpt-4-vision-preview");
        let claude_estimate = estimator.estimate_image_tokens(&metadata, "claude-3-opus");
        
        // Claude should have slightly higher token count due to adjustment
        assert!(claude_estimate.estimated_tokens > gpt_estimate.estimated_tokens);
    }
    
    #[test]
    fn test_batch_estimation() {
        let estimator = ImageTokenEstimator::new();
        let images = vec![
            ImageMetadata {
                width: 512,
                height: 512,
                format: ImageFormat::PNG,
                detail_level: ImageDetail::Low,
            },
            ImageMetadata {
                width: 512,
                height: 512,
                format: ImageFormat::JPEG,
                detail_level: ImageDetail::Medium,
            },
        ];
        
        let estimate = estimator.estimate_batch_tokens(&images, "gpt-4-vision-preview");
        assert_eq!(estimate.estimated_tokens, 85 + 170);
    }
    
    #[test]
    fn test_recommend_detail_level() {
        let estimator = ImageTokenEstimator::new();
        
        let small = ImageMetadata {
            width: 256,
            height: 256,
            format: ImageFormat::PNG,
            detail_level: ImageDetail::Auto,
        };
        
        let medium = ImageMetadata {
            width: 768,
            height: 768,
            format: ImageFormat::PNG,
            detail_level: ImageDetail::Auto,
        };
        
        let large = ImageMetadata {
            width: 2048,
            height: 2048,
            format: ImageFormat::PNG,
            detail_level: ImageDetail::Auto,
        };
        
        assert_eq!(estimator.recommend_detail_level(&small), ImageDetail::Low);
        assert_eq!(estimator.recommend_detail_level(&medium), ImageDetail::Medium);
        assert_eq!(estimator.recommend_detail_level(&large), ImageDetail::High);
    }
}
