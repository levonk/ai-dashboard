# Token Estimation Methodology

This document explains the token estimation methodology used in the AI Dashboard analytics system for different AI models and input types.

## Overview

Token estimation is critical for:
- Cost calculation and budgeting
- Resource planning and capacity management
- Analytics and usage tracking
- Performance optimization

The system uses multiple estimation methods depending on the model and input type, with varying levels of accuracy.

## Estimation Methods

### 1. Character-Based Estimation

**Accuracy:** ~85% confidence  
**Use Case:** Text and chat models when exact token count is unavailable

**Formula:**
```
estimated_tokens = ceil(character_count / char_to_token_ratio)
```

**Default Ratios:**
- Most models: 4 characters per token
- Custom models: Configurable via registry

**Example:**
```rust
let estimator = TextTokenEstimator::new();
let estimate = estimator.estimate_by_chars("Hello, world!", "gpt-4");
// Result: ~4 tokens (13 chars / 4 = 3.25 → 4)
```

### 2. Word-Based Estimation

**Accuracy:** ~75% confidence  
**Use Case:** Fallback when character-based estimation is unavailable

**Formula:**
```
estimated_tokens = ceil(word_count / word_to_token_ratio)
```

**Default Ratio:** 0.75 words per token

**Example:**
```rust
let estimate = estimator.estimate_by_words("Hello world test", "gpt-4");
// Result: ~4 tokens (3 words / 0.75 = 4)
```

### 3. Model-Specific Estimation

**Accuracy:** ~80-95% confidence  
**Use Case:** When provider-specific token counting is available

**Formula:** Varies by provider and model type

**Anthropic Claude:**
- Text: 4 characters per token
- Images: Base 765 tokens + scaling by resolution
- Adjustments: +10% for Claude 3 vs GPT-4

**OpenAI GPT:**
- Text: 4 characters per token
- Images: Base 765 tokens + scaling by resolution
- Audio: 32 tokens per second (GPT-4o)
- Video: 162 tokens per second (GPT-4o, 1080p)

**Google Gemini:**
- Text: 4 characters per token
- Images: Base 765 tokens + scaling by resolution
- Video: 130 tokens per second (Gemini Ultra)

### 4. Exact Token Count

**Accuracy:** 100% confidence  
**Use Case:** When actual API response includes token count

**Method:** Use actual token counts from API responses when available

## Input Type Estimation

### Text/Chat

**Method:** Character-based or word-based  
**Confidence:** 85% (character), 75% (word)  
**Overhead:** +3 tokens per message for chat formatting

### Images

**Method:** Model-specific based on resolution and detail level

**Detail Levels:**
- **Low:** ~85 tokens (512x512 or smaller)
- **Medium:** ~170 tokens (up to 1024x1024)
- **High:** 765 base + scaling factor

**Scaling Formula:**
```
scaling = ceil((width * height) / (512 * 512))
total_tokens = 765 + scaling
```

**Example:**
- 512x512: 765 + 1 = 766 tokens
- 1024x1024: 765 + 4 = 769 tokens
- 2048x2048: 765 + 16 = 781 tokens

### Audio

**Method:** Duration-based estimation

**Formula:**
```
tokens = duration_seconds * tokens_per_second
```

**Rates by Model:**
- GPT-4o Audio: 32 tokens/second
- Whisper: 25 tokens/second
- Default: 30 tokens/second

**Example:**
```rust
let audio = AudioMetadata {
    duration_seconds: 10.0,
    format: AudioFormat::MP3,
    sample_rate: 44100,
    channels: 2,
};
let estimate = estimator.estimate_audio_tokens(&audio, "gpt-4o-audio-preview");
// Result: 320 tokens (10 * 32)
```

### Video

**Method:** Combined audio + visual estimation

**Formula:**
```
audio_tokens = duration_seconds * 32
visual_tokens = duration_seconds * visual_rate * resolution_factor
total_tokens = audio_tokens + visual_tokens
```

**Resolution Factor:**
```
resolution_factor = (width * height) / (1920 * 1080)
```

**Visual Rates:**
- GPT-4o Video: 130 tokens/second at 1080p
- Gemini Ultra: 130 tokens/second at 1080p

**Example:**
```rust
let video = VideoMetadata {
    duration_seconds: 10.0,
    format: VideoFormat::MP4,
    resolution: (1920, 1080),
    frame_rate: 30.0,
};
let estimate = estimator.estimate_video_tokens(&video, "gpt-4o-video-preview");
// Result: ~1620 tokens (10 * (32 + 130))
```

## Model Registry

The model registry maintains configuration for all supported models, including:

- Character-to-token ratios
- Word-to-token ratios
- Supported input types
- Base token costs for different media types
- Provider-specific adjustments

### Adding New Models

```rust
let custom_config = ModelConfig {
    model_id: "custom-model".to_string(),
    provider: "custom".to_string(),
    supported_input_types: vec![InputType::Text, InputType::Image],
    char_to_token_ratio: 4.5,
    word_to_token_ratio: 0.8,
    image_base_tokens: 800,
    audio_tokens_per_second: 0.0,
    video_tokens_per_second: 0.0,
    estimation_method: EstimationMethod::ModelSpecific,
};

registry.register_model(custom_config);
```

## Accuracy and Limitations

### Accuracy Levels

| Method | Confidence | Best For |
|--------|-----------|----------|
| Exact API count | 100% | Production cost calculation |
| Model-specific | 80-95% | Planning and analytics |
| Character-based | 85% | General text estimation |
| Word-based | 75% | Quick estimates |

### Known Limitations

1. **Language Variations:** Character-to-token ratios vary by language (e.g., Chinese uses fewer characters per token than English)

2. **Special Tokens:** System prompts, formatting, and special tokens may not be accurately estimated

3. **Model Updates:** New model versions may change tokenization schemes

4. **Regional Differences:** Some models have region-specific token counting

5. **Content Complexity:** Code, mathematical notation, and specialized content may have different tokenization

### Mitigation Strategies

1. **Regular Calibration:** Compare estimates against actual usage and adjust ratios
2. **Confidence Tracking:** All estimates include confidence scores
3. **Fallback Methods:** Use multiple estimation methods when possible
4. **Monitoring:** Track estimation accuracy vs actual usage in production

## Performance Considerations

### Hashing Performance

- **Algorithm:** SHA-256
- **Speed:** ~100MB/s on modern hardware
- **Collision Rate:** < 0.001% with SHA-256

### Estimation Overhead

- **Text estimation:** < 1μs per request
- **Image estimation:** < 10μs per request
- **Audio/Video estimation:** < 5μs per request

### Optimization Tips

1. Cache hash results for identical content
2. Use character-based estimation for high-volume scenarios
3. Batch estimation for multiple items
4. Pre-register frequently used models

## Testing

The test suite includes:

- Unit tests for each estimation method
- Integration tests for model-specific accuracy
- Performance benchmarks
- Edge case coverage (empty content, very large inputs, etc.)

Run tests with:
```bash
cargo test
cargo test --test utils_integration_test
```

## References

- [Anthropic Token Counting](https://docs.anthropic.com/claude/docs/token-counting)
- [OpenAI Token Counting](https://platform.openai.com/tokenizer)
- [Google Gemini Token Counting](https://ai.google.dev/gemini-api/docs/tokens)

## Future Improvements

1. **Machine Learning:** Train ML models to predict token counts more accurately
2. **Real-time Calibration:** Automatically adjust ratios based on actual usage
3. **Language-Specific Ratios:** Maintain different ratios for different languages
4. **API Integration:** Direct integration with provider token counting APIs when available
