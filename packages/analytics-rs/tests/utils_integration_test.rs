//! Integration tests for utility modules
//! 
//! This test suite validates the integration of hashing, token estimation,
//! timing utilities, and input type detection with model-specific test cases.

use analytics_rs::{
    hashing::{content_hash, structured_hash, correlation_id, HashCollisionMonitor},
    tokens::{TextTokenEstimator, InputType, EstimationMethod, ChatMessage},
    image_tokens::{ImageTokenEstimator, ImageMetadata, ImageFormat, ImageDetail},
    media_tokens::{AudioTokenEstimator, VideoTokenEstimator, AudioMetadata, VideoMetadata, AudioFormat, VideoFormat},
    token_registry::{TokenRegistry, ModelConfig},
    timing::{Timer, measure, measure_micros, TimingStats},
    input_type::InputTypeDetector,
};
use serde::Serialize;

#[test]
fn test_hashing_integration() {
    // Test content hashing consistency
    let content = "Test request content";
    let hash1 = content_hash(content);
    let hash2 = content_hash(content);
    assert_eq!(hash1, hash2);
    
    // Test structured hashing
    #[derive(Serialize)]
    struct Request {
        model: String,
        prompt: String,
    }
    
    let req = Request {
        model: "claude-3-opus".to_string(),
        prompt: "Hello".to_string(),
    };
    
    let hash = structured_hash(&req).unwrap();
    assert_eq!(hash.len(), 64);
    
    // Test correlation ID generation
    let components = vec!["claude-3-opus", "test prompt", "temp:0.7"];
    let corr_id = correlation_id(&components);
    assert_eq!(corr_id.len(), 64);
}

#[test]
fn test_token_estimation_integration() {
    let estimator = TextTokenEstimator::new();
    
    // Test Anthropic Claude models
    let claude_estimate = estimator.estimate_by_chars("Hello, world!", "claude-3-opus");
    assert_eq!(claude_estimate.model, "claude-3-opus");
    assert!(claude_estimate.estimated_tokens > 0);
    
    // Test OpenAI GPT models
    let gpt_estimate = estimator.estimate_by_chars("Hello, world!", "gpt-4");
    assert_eq!(gpt_estimate.model, "gpt-4");
    assert!(gpt_estimate.estimated_tokens > 0);
    
    // Test Google Gemini models
    let gemini_estimate = estimator.estimate_by_chars("Hello, world!", "gemini-pro");
    assert_eq!(gemini_estimate.model, "gemini-pro");
    assert!(gemini_estimate.estimated_tokens > 0);
    
    // Test chat estimation
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Hello!".to_string(),
        },
    ];
    
    let chat_estimate = estimator.estimate_chat_tokens(&messages, "claude-3-opus");
    assert_eq!(chat_estimate.input_type, InputType::Chat);
    assert!(chat_estimate.estimated_tokens > claude_estimate.estimated_tokens);
}

#[test]
fn test_image_token_estimation_integration() {
    let estimator = ImageTokenEstimator::new();
    
    // Test different detail levels
    let low_detail = ImageMetadata {
        width: 512,
        height: 512,
        format: ImageFormat::PNG,
        detail_level: ImageDetail::Low,
    };
    
    let low_estimate = estimator.estimate_image_tokens(&low_detail, "gpt-4-vision-preview");
    assert_eq!(low_estimate.estimated_tokens, 85);
    
    let medium_detail = ImageMetadata {
        width: 512,
        height: 512,
        format: ImageFormat::JPEG,
        detail_level: ImageDetail::Medium,
    };
    
    let medium_estimate = estimator.estimate_image_tokens(&medium_detail, "gpt-4-vision-preview");
    assert_eq!(medium_estimate.estimated_tokens, 170);
    
    // Test model-specific adjustments
    let claude_estimate = estimator.estimate_image_tokens(&medium_detail, "claude-3-opus");
    assert!(claude_estimate.estimated_tokens != medium_estimate.estimated_tokens);
}

#[test]
fn test_audio_token_estimation_integration() {
    let estimator = AudioTokenEstimator::new();
    
    // Test GPT-4o Audio
    let audio_metadata = AudioMetadata {
        duration_seconds: 10.0,
        format: AudioFormat::MP3,
        sample_rate: 44100,
        channels: 2,
    };
    
    let gpt_audio_estimate = estimator.estimate_audio_tokens(&audio_metadata, "gpt-4o-audio-preview");
    assert_eq!(gpt_audio_estimate.estimated_tokens, 320); // 10 * 32
    
    // Test Whisper
    let whisper_estimate = estimator.estimate_audio_tokens(&audio_metadata, "whisper-1");
    assert_eq!(whisper_estimate.estimated_tokens, 250); // 10 * 25
}

#[test]
fn test_video_token_estimation_integration() {
    let estimator = VideoTokenEstimator::new();
    
    let video_metadata = VideoMetadata {
        duration_seconds: 10.0,
        format: VideoFormat::MP4,
        resolution: (1920, 1080),
        frame_rate: 30.0,
    };
    
    let estimate = estimator.estimate_video_tokens(&video_metadata, "gpt-4o-video-preview");
    assert!(estimate.estimated_tokens > 0);
    assert_eq!(estimate.input_type, InputType::Video);
}

#[test]
fn test_token_registry_integration() {
    let registry = TokenRegistry::new();
    
    // Test model registration and retrieval
    assert!(registry.has_model("claude-3-opus"));
    assert!(registry.has_model("gpt-4"));
    assert!(registry.has_model("gemini-pro"));
    
    // Test provider filtering
    let anthropic_models = registry.get_models_by_provider("anthropic");
    assert!(!anthropic_models.is_empty());
    
    // Test input type filtering
    let image_models = registry.get_models_by_input_type(&InputType::Image);
    assert!(!image_models.is_empty());
    
    // Test custom model registration
    let custom_config = ModelConfig {
        model_id: "custom-model".to_string(),
        provider: "custom".to_string(),
        supported_input_types: vec![InputType::Text],
        char_to_token_ratio: 5.0,
        word_to_token_ratio: 0.8,
        image_base_tokens: 0,
        audio_tokens_per_second: 0.0,
        video_tokens_per_second: 0.0,
        estimation_method: EstimationMethod::CharacterBased,
    };
    
    let mut mutable_registry = TokenRegistry::new();
    mutable_registry.register_model(custom_config);
    assert!(mutable_registry.has_model("custom-model"));
}

#[test]
fn test_timing_utilities_integration() {
    // Test Timer
    let mut timer = Timer::start();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let elapsed = timer.stop();
    assert!(elapsed.as_millis() >= 10);
    
    // Test measure function
    let (result, duration) = measure(|| {
        std::thread::sleep(std::time::Duration::from_millis(5));
        42
    });
    assert_eq!(result, 42);
    assert!(duration.as_millis() >= 5);
    
    // Test measure_micros
    let (result, micros) = measure_micros(|| {
        std::thread::sleep(std::time::Duration::from_millis(1));
        "test"
    });
    assert_eq!(result, "test");
    assert!(micros >= 1000);
    
    // Test TimingStats
    let mut stats = TimingStats::new();
    stats.record(std::time::Duration::from_millis(10));
    stats.record(std::time::Duration::from_millis(20));
    stats.record(std::time::Duration::from_millis(30));
    
    assert_eq!(stats.count, 3);
    assert_eq!(stats.min_duration, std::time::Duration::from_millis(10));
    assert_eq!(stats.max_duration, std::time::Duration::from_millis(30));
}

#[test]
fn test_input_type_detection_integration() {
    // Test file extension detection
    assert_eq!(
        InputTypeDetector::from_file_extension("test.png"),
        InputType::Image
    );
    assert_eq!(
        InputTypeDetector::from_file_extension("test.mp3"),
        InputType::Audio
    );
    assert_eq!(
        InputTypeDetector::from_file_extension("test.mp4"),
        InputType::Video
    );
    assert_eq!(
        InputTypeDetector::from_file_extension("test.txt"),
        InputType::Text
    );
    
    // Test MIME type detection
    assert_eq!(
        InputTypeDetector::from_mime_type("image/png"),
        InputType::Image
    );
    assert_eq!(
        InputTypeDetector::from_mime_type("audio/mpeg"),
        InputType::Audio
    );
    assert_eq!(
        InputTypeDetector::from_mime_type("video/mp4"),
        InputType::Video
    );
    
    // Test content detection
    let chat_content = "user: Hello\nassistant: Hi there!";
    assert_eq!(
        InputTypeDetector::from_content(chat_content),
        InputType::Chat
    );
    
    // Test priority-based detection
    assert_eq!(
        InputTypeDetector::detect(
            Some("file.txt"),
            Some("image/png"),
            Some("text content")
        ),
        InputType::Image // MIME type takes priority
    );
    
    // Test model support checking
    assert!(InputTypeDetector::is_supported_for_model(&InputType::Text, "gpt-4"));
    assert!(InputTypeDetector::is_supported_for_model(&InputType::Image, "gpt-4-vision-preview"));
    assert!(!InputTypeDetector::is_supported_for_model(&InputType::Image, "gpt-3.5-turbo"));
}

#[test]
fn test_model_specific_token_accuracy() {
    let estimator = TextTokenEstimator::new();
    
    // Test that different models have appropriate token ratios
    let text = "The quick brown fox jumps over the lazy dog.";
    
    let claude_tokens = estimator.estimate_by_chars(text, "claude-3-opus").estimated_tokens;
    let gpt_tokens = estimator.estimate_by_chars(text, "gpt-4").estimated_tokens;
    let gemini_tokens = estimator.estimate_by_chars(text, "gemini-pro").estimated_tokens;
    
    // All should be similar since they use the same ratio
    assert_eq!(claude_tokens, gpt_tokens);
    assert_eq!(gpt_tokens, gemini_tokens);
}

#[test]
fn test_hash_collision_monitoring() {
    let mut monitor = HashCollisionMonitor::new();
    
    // Record some hashes
    for i in 0..100 {
        let content = format!("content_{}", i);
        monitor.record_hash(&content_hash(&content));
    }
    
    assert_eq!(monitor.total_hashes(), 100);
    assert_eq!(monitor.collision_rate(), 0.0);
}

#[test]
fn test_multimodal_token_estimation() {
    // Test that GPT-4o supports all input types
    let registry = TokenRegistry::new();
    let gpt4o_config = registry.get_model_config("gpt-4o").unwrap();
    
    assert!(gpt4o_config.supported_input_types.contains(&InputType::Text));
    assert!(gpt4o_config.supported_input_types.contains(&InputType::Image));
    assert!(gpt4o_config.supported_input_types.contains(&InputType::Audio));
    assert!(gpt4o_config.supported_input_types.contains(&InputType::Video));
}

#[test]
fn test_batch_estimation() {
    let text_estimator = TextTokenEstimator::new();
    let image_estimator = ImageTokenEstimator::new();
    let audio_estimator = AudioTokenEstimator::new();
    
    // Test batch text estimation
    let texts = vec!["Hello", "World", "Test"];
    let total: u32 = texts.iter()
        .map(|text| text_estimator.estimate_by_chars(text, "gpt-4").estimated_tokens)
        .sum();
    assert!(total > 0);
    
    // Test batch image estimation
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
    
    let batch_estimate = image_estimator.estimate_batch_tokens(&images, "gpt-4-vision-preview");
    assert_eq!(batch_estimate.estimated_tokens, 85 + 170);
    
    // Test batch audio estimation
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
    
    let audio_batch = audio_estimator.estimate_batch_tokens(&audio_files, "gpt-4o-audio-preview");
    assert_eq!(audio_batch.estimated_tokens, 480); // (5 + 10) * 32
}

#[test]
fn test_end_to_end_workflow() {
    // Simulate a complete analytics workflow
    
    // 1. Hash request content
    let request_content = "What is the capital of France?";
    let request_hash = content_hash(request_content);
    assert_eq!(request_hash.len(), 64);
    
    // 2. Detect input type
    let input_type = InputTypeDetector::from_content(request_content);
    assert_eq!(input_type, InputType::Text);
    
    // 3. Estimate tokens
    let estimator = TextTokenEstimator::new();
    let token_estimate = estimator.estimate_by_chars(request_content, "claude-3-opus");
    assert!(token_estimate.estimated_tokens > 0);
    
    // 4. Measure processing time
    let (result, duration) = measure(|| {
        // Simulate processing
        std::thread::sleep(std::time::Duration::from_millis(1));
        token_estimate.estimated_tokens
    });
    assert!(result > 0);
    assert!(duration.as_millis() >= 1);
    
    // 5. Record in timing stats
    let mut stats = TimingStats::new();
    stats.record(duration);
    assert_eq!(stats.count, 1);
}
