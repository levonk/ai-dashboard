use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use toon_format::{encode, decode, EncodeOptions, DecodeOptions};

/// ToonFormat encoder for efficient token-optimized data serialization
pub struct ToonFormatEncoder;

/// ToonFormat decoder for parsing token-optimized data
pub struct ToonFormatDecoder;

impl ToonFormatEncoder {
    /// Encode a JSON value to ToonFormat for token-efficient transmission
    pub fn encode_json(value: &Value) -> Result<String> {
        encode(value, &EncodeOptions::default())
            .context("Failed to serialize ToonFormat")
    }

    /// Encode a serializable struct to ToonFormat
    pub fn encode<T: Serialize>(data: &T) -> Result<String> {
        encode(data, &EncodeOptions::default())
            .context("Failed to serialize ToonFormat")
    }

    /// Encode multiple records for bulk export with compression
    pub fn encode_bulk<T: Serialize>(records: &Vec<T>) -> Result<String> {
        encode(records, &EncodeOptions::default())
            .context("Failed to serialize bulk records to ToonFormat")
    }
}

impl ToonFormatDecoder {
    /// Decode ToonFormat string to JSON value
    pub fn decode_to_json(toon_str: &str) -> Result<Value> {
        decode::<Value>(toon_str, &DecodeOptions::default())
            .context("Failed to parse ToonFormat")
    }

    /// Decode ToonFormat string to a deserializable struct
    pub fn decode<T: for<'de> Deserialize<'de>>(toon_str: &str) -> Result<T> {
        decode(toon_str, &DecodeOptions::default())
            .context("Failed to deserialize from ToonFormat")
    }

    /// Decode bulk records from ToonFormat
    pub fn decode_bulk<T: for<'de> Deserialize<'de>>(toon_str: &str) -> Result<Vec<T>> {
        decode(toon_str, &DecodeOptions::default())
            .context("Failed to deserialize bulk records from ToonFormat")
    }
}

/// Calculate compression ratio between JSON and ToonFormat
pub fn calculate_compression_ratio(json_size: usize, toon_size: usize) -> f64 {
    if json_size == 0 {
        return 0.0;
    }
    (json_size as f64 - toon_size as f64) / json_size as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_encode_decode_simple() {
        let data = json!({
            "name": "test",
            "value": 42,
            "active": true
        });

        let encoded = ToonFormatEncoder::encode_json(&data).unwrap();
        let decoded = ToonFormatDecoder::decode_to_json(&encoded).unwrap();
        
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_encode_decode_nested() {
        let data = json!({
            "user": {
                "name": "Alice",
                "metrics": {
                    "requests": 100,
                    "errors": 5
                }
            },
            "tags": ["production", "api"]
        });

        let encoded = ToonFormatEncoder::encode_json(&data).unwrap();
        let decoded = ToonFormatDecoder::decode_to_json(&encoded).unwrap();
        
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_bulk_encode_decode() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Record {
            id: u32,
            name: String,
        }

        let records = vec![
            Record { id: 1, name: "First".to_string() },
            Record { id: 2, name: "Second".to_string() },
        ];

        let encoded = ToonFormatEncoder::encode_bulk(&records).unwrap();
        let decoded: Vec<Record> = ToonFormatDecoder::decode_bulk(&encoded).unwrap();
        
        assert_eq!(records, decoded);
    }

    #[test]
    fn test_compression_ratio() {
        let data = json!({
            "field1": "value1",
            "field2": "value2",
            "field3": "value3",
            "nested": {
                "a": 1,
                "b": 2,
                "c": 3
            }
        });

        let json_str = serde_json::to_string(&data).unwrap();
        let toon_str = ToonFormatEncoder::encode_json(&data).unwrap();
        
        let ratio = calculate_compression_ratio(json_str.len(), toon_str.len());
        assert!(ratio >= 0.0);
    }
}
