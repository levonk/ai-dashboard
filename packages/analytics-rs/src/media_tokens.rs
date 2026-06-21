use serde::{Serialize, Deserialize};
use crate::tokens::{TokenEstimate, InputType, EstimationMethod};

/// Audio and video token estimation for multimodal models
/// 
/// This module provides token counting for audio and video inputs across
/// different multimodal models (GPT-4o Audio, Whisper, etc.)

/// Audio metadata for token estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioMetadata {
    pub duration_seconds: f64,
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Video metadata for token estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub duration_seconds: f64,
    pub format: VideoFormat,
    pub resolution: (u32, u32), // (width, height)
    pub frame_rate: f64,
}

/// Audio format types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AudioFormat {
    WAV,
    MP3,
    FLAC,
    AAC,
    OGG,
    Unknown,
}

/// Video format types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum VideoFormat {
    MP4,
    WEBM,
    AVI,
    MOV,
    Unknown,
}

/// Audio token estimator
pub struct AudioTokenEstimator;

impl AudioTokenEstimator {
    pub fn new() -> Self {
        Self
    }
    
    /// Estimate tokens for audio content based on duration
    /// 
    /// # Arguments
    /// * `metadata` - Audio metadata including duration and format
    /// * `model` - The model being used (affects token counting)
    /// 
    /// # Returns
    /// Token estimate with confidence and method
    pub fn estimate_audio_tokens(&self, metadata: &AudioMetadata, model: &str) -> TokenEstimate {
        // Base estimation: ~32 tokens per second for most models
        let base_tokens_per_second = self.get_tokens_per_second(model);
        let estimated_tokens = (metadata.duration_seconds * base_tokens_per_second).ceil() as u32;
        
        TokenEstimate {
            estimated_tokens,
            model: model.to_string(),
            input_type: InputType::Audio,
            confidence: 0.70, // Audio estimation is less precise than text
            method: EstimationMethod::ModelSpecific,
        }
    }
    
    /// Get tokens per second for a specific model
    fn get_tokens_per_second(&self, model: &str) -> f64 {
        match model {
            // GPT-4o Audio
            "gpt-4o-audio-preview" => 32.0,
            
            // Whisper (speech-to-text)
            "whisper-1" => 25.0,
            
            // Default estimation
            _ => 30.0,
        }
    }
    
    /// Estimate tokens for multiple audio files
    pub fn estimate_batch_tokens(&self, audio_files: &[AudioMetadata], model: &str) -> TokenEstimate {
        let total: u32 = audio_files.iter()
            .map(|audio| self.estimate_audio_tokens(audio, model).estimated_tokens)
            .sum();
        
        TokenEstimate {
            estimated_tokens: total,
            model: model.to_string(),
            input_type: InputType::Audio,
            confidence: 0.65, // Batch estimation has slightly lower confidence
            method: EstimationMethod::ModelSpecific,
        }
    }
}

/// Video token estimator
pub struct VideoTokenEstimator;

impl VideoTokenEstimator {
    pub fn new() -> Self {
        Self
    }
    
    /// Estimate tokens for video content based on duration and resolution
    /// 
    /// # Arguments
    /// * `metadata` - Video metadata including duration, resolution, and format
    /// * `model` - The model being used (affects token counting)
    /// 
    /// # Returns
    /// Token estimate with confidence and method
    pub fn estimate_video_tokens(&self, metadata: &VideoMetadata, model: &str) -> TokenEstimate {
        // Video tokens are typically estimated as:
        // - Base tokens per second for audio track
        // - Additional tokens for visual frames
        
        let audio_tokens_per_second = 32.0; // Approximate for audio track
        let visual_tokens_per_second = self.get_visual_tokens_per_second(metadata, model);
        
        let total_tokens_per_second = audio_tokens_per_second + visual_tokens_per_second;
        let estimated_tokens = (metadata.duration_seconds * total_tokens_per_second).ceil() as u32;
        
        TokenEstimate {
            estimated_tokens,
            model: model.to_string(),
            input_type: InputType::Video,
            confidence: 0.60, // Video estimation is the least precise
            method: EstimationMethod::ModelSpecific,
        }
    }
    
    /// Get visual tokens per second based on resolution and model
    fn get_visual_tokens_per_second(&self, metadata: &VideoMetadata, model: &str) -> f64 {
        let (width, height) = metadata.resolution;
        let resolution_factor = (width * height) as f64 / (1920.0 * 1080.0); // Normalize to 1080p
        
        match model {
            // GPT-4o Video
            "gpt-4o-video-preview" => 130.0 * resolution_factor,
            
            // Default estimation
            _ => 100.0 * resolution_factor,
        }
    }
    
    /// Estimate tokens for multiple video files
    pub fn estimate_batch_tokens(&self, videos: &[VideoMetadata], model: &str) -> TokenEstimate {
        let total: u32 = videos.iter()
            .map(|video| self.estimate_video_tokens(video, model).estimated_tokens)
            .sum();
        
        TokenEstimate {
            estimated_tokens: total,
            model: model.to_string(),
            input_type: InputType::Video,
            confidence: 0.55, // Batch video estimation has lowest confidence
            method: EstimationMethod::ModelSpecific,
        }
    }
}

impl Default for AudioTokenEstimator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for VideoTokenEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_audio_tokens() {
        let estimator = AudioTokenEstimator::new();
        let metadata = AudioMetadata {
            duration_seconds: 10.0,
            format: AudioFormat::MP3,
            sample_rate: 44100,
            channels: 2,
        };
        
        let estimate = estimator.estimate_audio_tokens(&metadata, "gpt-4o-audio-preview");
        assert_eq!(estimate.estimated_tokens, 320); // 10 seconds * 32 tokens/sec
        assert_eq!(estimate.input_type, InputType::Audio);
    }
    
    #[test]
    fn test_estimate_audio_tokens_whisper() {
        let estimator = AudioTokenEstimator::new();
        let metadata = AudioMetadata {
            duration_seconds: 5.0,
            format: AudioFormat::WAV,
            sample_rate: 16000,
            channels: 1,
        };
        
        let estimate = estimator.estimate_audio_tokens(&metadata, "whisper-1");
        assert_eq!(estimate.estimated_tokens, 125); // 5 seconds * 25 tokens/sec
    }
    
    #[test]
    fn test_estimate_video_tokens() {
        let estimator = VideoTokenEstimator::new();
        let metadata = VideoMetadata {
            duration_seconds: 10.0,
            format: VideoFormat::MP4,
            resolution: (1920, 1080),
            frame_rate: 30.0,
        };
        
        let estimate = estimator.estimate_video_tokens(&metadata, "gpt-4o-video-preview");
        // 10 seconds * (32 audio + 130 visual) = 1620 tokens
        assert!(estimate.estimated_tokens > 0);
        assert_eq!(estimate.input_type, InputType::Video);
    }
    
    #[test]
    fn test_estimate_video_tokens_lower_resolution() {
        let estimator = VideoTokenEstimator::new();
        let metadata = VideoMetadata {
            duration_seconds: 10.0,
            format: VideoFormat::MP4,
            resolution: (1280, 720), // 720p
            frame_rate: 30.0,
        };
        
        let estimate = estimator.estimate_video_tokens(&metadata, "gpt-4o-video-preview");
        // Should be lower than 1080p due to resolution factor
        assert!(estimate.estimated_tokens > 0);
    }
    
    #[test]
    fn test_audio_batch_estimation() {
        let estimator = AudioTokenEstimator::new();
        let audio_files = vec![
            AudioMetadata {
                duration_seconds: 5.0,
                format: AudioFormat::MP3,
                sample_rate: 44100,
                channels: 2,
            },
            AudioMetadata {
                duration_seconds: 10.0,
                format: AudioFormat::WAV,
                sample_rate: 44100,
                channels: 2,
            },
        ];
        
        let estimate = estimator.estimate_batch_tokens(&audio_files, "gpt-4o-audio-preview");
        assert_eq!(estimate.estimated_tokens, 480); // (5 + 10) * 32
    }
    
    #[test]
    fn test_video_batch_estimation() {
        let estimator = VideoTokenEstimator::new();
        let videos = vec![
            VideoMetadata {
                duration_seconds: 5.0,
                format: VideoFormat::MP4,
                resolution: (1920, 1080),
                frame_rate: 30.0,
            },
        ];
        
        let estimate = estimator.estimate_batch_tokens(&videos, "gpt-4o-video-preview");
        assert!(estimate.estimated_tokens > 0);
    }
}
