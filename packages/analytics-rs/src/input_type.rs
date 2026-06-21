use crate::tokens::InputType;
use std::path::Path;

/// Input type detection and classification
/// 
/// This module provides utilities to detect and classify different content types
/// based on file extensions, MIME types, or content analysis.

/// Input type detector
pub struct InputTypeDetector;

impl InputTypeDetector {
    /// Detect input type from file extension
    pub fn from_file_extension(file_path: &str) -> InputType {
        let path = Path::new(file_path);
        
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => Self::extension_to_input_type(ext),
            None => InputType::Text, // Default to text if no extension
        }
    }
    
    /// Detect input type from MIME type
    pub fn from_mime_type(mime_type: &str) -> InputType {
        match mime_type {
            // Text types
            mime if mime.starts_with("text/") => InputType::Text,
            "application/json" | "application/xml" | "application/yaml" => InputType::Text,
            
            // Image types
            mime if mime.starts_with("image/") => InputType::Image,
            
            // Audio types
            mime if mime.starts_with("audio/") => InputType::Audio,
            
            // Video types
            mime if mime.starts_with("video/") => InputType::Video,
            
            // Default
            _ => InputType::Text,
        }
    }
    
    /// Detect input type from content analysis
    pub fn from_content(content: &str) -> InputType {
        // Simple heuristics for content detection
        if Self::looks_like_image_data(content) {
            InputType::Image
        } else if Self::looks_like_audio_data(content) {
            InputType::Audio
        } else if Self::looks_like_video_data(content) {
            InputType::Video
        } else if Self::looks_like_chat_conversation(content) {
            InputType::Chat
        } else {
            InputType::Text
        }
    }
    
    /// Detect input type from multiple sources (prioritized)
    pub fn detect(file_path: Option<&str>, mime_type: Option<&str>, content: Option<&str>) -> InputType {
        // Priority: MIME type > file extension > content analysis
        if let Some(mime) = mime_type {
            return Self::from_mime_type(mime);
        }
        
        if let Some(path) = file_path {
            return Self::from_file_extension(path);
        }
        
        if let Some(cont) = content {
            return Self::from_content(cont);
        }
        
        InputType::Text // Default fallback
    }
    
    /// Convert file extension to input type
    fn extension_to_input_type(extension: &str) -> InputType {
        let ext_lower = extension.to_lowercase();
        
        match ext_lower.as_str() {
            // Image extensions
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" | "tiff" => InputType::Image,
            
            // Audio extensions
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "wma" | "opus" => InputType::Audio,
            
            // Video extensions
            "mp4" | "avi" | "mov" | "wmv" | "flv" | "webm" | "mkv" | "m4v" => InputType::Video,
            
            // Text/document extensions
            "txt" | "md" | "json" | "xml" | "yaml" | "yml" | "csv" | "html" | "css" | "js" | "ts" | "py" | "rs" => InputType::Text,
            
            // Default to text
            _ => InputType::Text,
        }
    }
    
    /// Check if content looks like image data
    fn looks_like_image_data(content: &str) -> bool {
        // Check for base64 image data patterns
        content.starts_with("data:image/") || 
        content.contains("base64,") ||
        content.len() > 1000 && content.chars().filter(|c| c.is_alphanumeric()).count() > content.len() / 2
    }
    
    /// Check if content looks like audio data
    fn looks_like_audio_data(content: &str) -> bool {
        content.starts_with("data:audio/") || 
        content.contains("audio/")
    }
    
    /// Check if content looks like video data
    fn looks_like_video_data(content: &str) -> bool {
        content.starts_with("data:video/") || 
        content.contains("video/")
    }
    
    /// Check if content looks like a chat conversation
    fn looks_like_chat_conversation(content: &str) -> bool {
        let content_lower = content.to_lowercase();
        content_lower.contains("user:") || 
        content_lower.contains("assistant:") ||
        content_lower.contains("system:") ||
        content_lower.contains("role:") ||
        (content_lower.contains("message") && content_lower.contains("content"))
    }
    
    /// Check if an input type is supported for a specific model
    pub fn is_supported_for_model(input_type: &InputType, model: &str) -> bool {
        // This would typically query the token registry
        // For now, we'll use basic heuristics
        match input_type {
            InputType::Text | InputType::Chat => true, // Most models support text
            
            InputType::Image => {
                model.contains("vision") || 
                model.contains("claude-3") || 
                model.contains("gpt-4o") ||
                model.contains("gemini")
            }
            
            InputType::Audio => {
                model.contains("audio") || 
                model.contains("gpt-4o") ||
                model.contains("whisper")
            }
            
            InputType::Video => {
                model.contains("video") || 
                model.contains("gpt-4o") ||
                model.contains("gemini")
            }
            
            InputType::Multimodal => true, // Assume multimodal models support it
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_file_extension_image() {
        assert_eq!(
            InputTypeDetector::from_file_extension("test.png"),
            InputType::Image
        );
        assert_eq!(
            InputTypeDetector::from_file_extension("photo.jpg"),
            InputType::Image
        );
    }
    
    #[test]
    fn test_from_file_extension_audio() {
        assert_eq!(
            InputTypeDetector::from_file_extension("sound.mp3"),
            InputType::Audio
        );
        assert_eq!(
            InputTypeDetector::from_file_extension("audio.wav"),
            InputType::Audio
        );
    }
    
    #[test]
    fn test_from_file_extension_video() {
        assert_eq!(
            InputTypeDetector::from_file_extension("movie.mp4"),
            InputType::Video
        );
        assert_eq!(
            InputTypeDetector::from_file_extension("clip.webm"),
            InputType::Video
        );
    }
    
    #[test]
    fn test_from_file_extension_text() {
        assert_eq!(
            InputTypeDetector::from_file_extension("document.txt"),
            InputType::Text
        );
        assert_eq!(
            InputTypeDetector::from_file_extension("code.rs"),
            InputType::Text
        );
    }
    
    #[test]
    fn test_from_mime_type() {
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
        assert_eq!(
            InputTypeDetector::from_mime_type("text/plain"),
            InputType::Text
        );
    }
    
    #[test]
    fn test_from_content_chat() {
        let chat_content = "user: Hello\nassistant: Hi there!";
        assert_eq!(
            InputTypeDetector::from_content(chat_content),
            InputType::Chat
        );
    }
    
    #[test]
    fn test_from_content_image_data() {
        let image_data = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA";
        assert_eq!(
            InputTypeDetector::from_content(image_data),
            InputType::Image
        );
    }
    
    #[test]
    fn test_detect_with_priority() {
        // MIME type should take priority
        assert_eq!(
            InputTypeDetector::detect(
                Some("file.txt"),
                Some("image/png"),
                Some("text content")
            ),
            InputType::Image
        );
        
        // File extension should be used if no MIME type
        assert_eq!(
            InputTypeDetector::detect(
                Some("file.png"),
                None,
                Some("text content")
            ),
            InputType::Image
        );
        
        // Content analysis should be fallback
        assert_eq!(
            InputTypeDetector::detect(
                None,
                None,
                Some("user: hello")
            ),
            InputType::Chat
        );
    }
    
    #[test]
    fn test_is_supported_for_model() {
        assert!(InputTypeDetector::is_supported_for_model(&InputType::Text, "gpt-4"));
        assert!(InputTypeDetector::is_supported_for_model(&InputType::Image, "gpt-4-vision-preview"));
        assert!(InputTypeDetector::is_supported_for_model(&InputType::Image, "claude-3-opus"));
        assert!(!InputTypeDetector::is_supported_for_model(&InputType::Image, "gpt-3.5-turbo"));
    }
    
    #[test]
    fn test_case_insensitive_extension() {
        assert_eq!(
            InputTypeDetector::from_file_extension("test.PNG"),
            InputType::Image
        );
        assert_eq!(
            InputTypeDetector::from_file_extension("test.MP3"),
            InputType::Audio
        );
    }
}
