//! Data compression utilities for time-series storage
//!
//! This module provides compression algorithms for time-series data to reduce
//! storage requirements while maintaining query performance.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compression algorithm type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// Simple delta encoding
    Delta,
    /// Gorilla compression (optimized for time-series)
    Gorilla,
    /// Snappy compression
    Snappy,
}

impl Default for CompressionAlgorithm {
    fn default() -> Self {
        Self::Gorilla
    }
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Compression algorithm to use
    pub algorithm: CompressionAlgorithm,
    /// Minimum compression ratio to apply compression (0.0 - 1.0)
    pub min_compression_ratio: f64,
    /// Maximum size for compression (bytes)
    pub max_size_for_compression: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::default(),
            min_compression_ratio: 0.5, // Require at least 50% compression
            max_size_for_compression: 1024, // Only compress data > 1KB
        }
    }
}

/// Compression result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    /// Original size in bytes
    pub original_size: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
    /// Compression ratio (compressed_size / original_size)
    pub compression_ratio: f64,
    /// Algorithm used
    pub algorithm: CompressionAlgorithm,
    /// Compressed data
    pub data: Vec<u8>,
}

/// Time-series compressor
pub struct TimeSeriesCompressor {
    config: CompressionConfig,
}

impl TimeSeriesCompressor {
    /// Create a new time-series compressor with default configuration
    pub fn new() -> Self {
        Self {
            config: CompressionConfig::default(),
        }
    }

    /// Create a new time-series compressor with custom configuration
    pub fn with_config(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// Compress time-series data
    pub fn compress(&self, data: &[u8]) -> Result<CompressionResult> {
        let original_size = data.len();

        // Skip compression if data is too small
        if original_size < self.config.max_size_for_compression {
            return Ok(CompressionResult {
                original_size,
                compressed_size: original_size,
                compression_ratio: 1.0,
                algorithm: CompressionAlgorithm::None,
                data: data.to_vec(),
            });
        }

        // Apply compression based on algorithm
        let (compressed_data, algorithm) = match self.config.algorithm {
            CompressionAlgorithm::None => (data.to_vec(), CompressionAlgorithm::None),
            CompressionAlgorithm::Delta => self.compress_delta(data)?,
            CompressionAlgorithm::Gorilla => self.compress_gorilla(data)?,
            CompressionAlgorithm::Snappy => self.compress_snappy(data)?,
        };

        let compressed_size = compressed_data.len();
        let compression_ratio = compressed_size as f64 / original_size as f64;

        // Check if compression meets minimum ratio requirement
        if compression_ratio > self.config.min_compression_ratio {
            // Compression not effective enough, return original
            Ok(CompressionResult {
                original_size,
                compressed_size: original_size,
                compression_ratio: 1.0,
                algorithm: CompressionAlgorithm::None,
                data: data.to_vec(),
            })
        } else {
            Ok(CompressionResult {
                original_size,
                compressed_size,
                compression_ratio,
                algorithm,
                data: compressed_data,
            })
        }
    }

    /// Decompress time-series data
    pub fn decompress(&self, data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>> {
        match algorithm {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            CompressionAlgorithm::Delta => self.decompress_delta(data),
            CompressionAlgorithm::Gorilla => self.decompress_gorilla(data),
            CompressionAlgorithm::Snappy => self.decompress_snappy(data),
        }
    }

    /// Delta encoding compression
    fn compress_delta(&self, data: &[u8]) -> Result<(Vec<u8>, CompressionAlgorithm)> {
        if data.is_empty() {
            return Ok((Vec::new(), CompressionAlgorithm::Delta));
        }

        let mut compressed = Vec::with_capacity(data.len());
        compressed.push(data[0]);

        for i in 1..data.len() {
            let delta = data[i].wrapping_sub(data[i - 1]);
            compressed.push(delta);
        }

        Ok((compressed, CompressionAlgorithm::Delta))
    }

    /// Delta encoding decompression
    fn decompress_delta(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut decompressed = Vec::with_capacity(data.len());
        decompressed.push(data[0]);

        for i in 1..data.len() {
            let value = data[i].wrapping_add(decompressed[i - 1]);
            decompressed.push(value);
        }

        Ok(decompressed)
    }

    /// Gorilla compression (simplified version for demonstration)
    fn compress_gorilla(&self, data: &[u8]) -> Result<(Vec<u8>, CompressionAlgorithm)> {
        // Simplified Gorilla compression - in production, use a proper implementation
        // This is a placeholder that applies delta encoding as a fallback
        self.compress_delta(data)
    }

    /// Gorilla decompression (simplified version for demonstration)
    fn decompress_gorilla(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Simplified Gorilla decompression - in production, use a proper implementation
        self.decompress_delta(data)
    }

    /// Snappy compression (placeholder - would use snap crate in production)
    fn compress_snappy(&self, data: &[u8]) -> Result<(Vec<u8>, CompressionAlgorithm)> {
        // Placeholder for Snappy compression
        // In production, use: let mut encoder = snap::raw::Encoder::new();
        // encoder.compress_vec(data)
        Ok((data.to_vec(), CompressionAlgorithm::Snappy))
    }

    /// Snappy decompression (placeholder - would use snap crate in production)
    fn decompress_snappy(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Placeholder for Snappy decompression
        // In production, use: let mut decoder = snap::raw::Decoder::new();
        // decoder.decompress_vec(data)
        Ok(data.to_vec())
    }

    /// Estimate compression ratio without actually compressing
    pub fn estimate_compression_ratio(&self, data: &[u8]) -> f64 {
        if data.len() < self.config.max_size_for_compression {
            return 1.0;
        }

        // Simple heuristic: estimate based on data patterns
        let unique_bytes = data.iter().collect::<std::collections::HashSet<_>>().len();
        let repetition_ratio = 1.0 - (unique_bytes as f64 / data.len() as f64);

        // Higher repetition ratio = better compression
        let estimated_ratio = 1.0 - (repetition_ratio * 0.7); // Max 70% compression
        estimated_ratio.max(0.3) // Minimum 30% of original size
    }
}

/// Compression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// Total data compressed (bytes)
    pub total_compressed: usize,
    /// Total original data size (bytes)
    pub total_original: usize,
    /// Overall compression ratio
    pub overall_ratio: f64,
    /// Number of compressions performed
    pub compression_count: u64,
    /// Algorithm usage statistics
    pub algorithm_usage: HashMap<CompressionAlgorithm, u64>,
}

impl Default for CompressionStats {
    fn default() -> Self {
        Self {
            total_compressed: 0,
            total_original: 0,
            overall_ratio: 1.0,
            compression_count: 0,
            algorithm_usage: HashMap::new(),
        }
    }
}

impl CompressionStats {
    /// Update statistics with a new compression result
    pub fn update(&mut self, result: &CompressionResult) {
        self.total_original += result.original_size;
        self.total_compressed += result.compressed_size;
        self.compression_count += 1;

        if self.total_original > 0 {
            self.overall_ratio = self.total_compressed as f64 / self.total_original as f64;
        }

        *self.algorithm_usage.entry(result.algorithm).or_insert(0) += 1;
    }

    /// Get storage savings in bytes
    pub fn storage_savings(&self) -> usize {
        self.total_original.saturating_sub(self.total_compressed)
    }

    /// Get storage savings percentage
    pub fn storage_savings_percentage(&self) -> f64 {
        if self.total_original == 0 {
            0.0
        } else {
            (self.storage_savings() as f64 / self.total_original as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert_eq!(config.algorithm, CompressionAlgorithm::Gorilla);
        assert_eq!(config.min_compression_ratio, 0.5);
        assert_eq!(config.max_size_for_compression, 1024);
    }

    #[test]
    fn test_delta_compression() {
        let compressor = TimeSeriesCompressor::new();
        let data = vec![10u8, 11, 12, 13, 14, 15];

        let result = compressor.compress(&data).unwrap();
        assert_eq!(result.algorithm, CompressionAlgorithm::Delta);
        assert!(result.compressed_size <= data.len());

        let decompressed = compressor.decompress(&result.data, result.algorithm).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compression_skip_small_data() {
        let compressor = TimeSeriesCompressor::new();
        let data = vec![1u8, 2, 3]; // Small data

        let result = compressor.compress(&data).unwrap();
        assert_eq!(result.algorithm, CompressionAlgorithm::None);
        assert_eq!(result.compressed_size, data.len());
    }

    #[test]
    fn test_compression_stats() {
        let mut stats = CompressionStats::default();
        let result = CompressionResult {
            original_size: 1000,
            compressed_size: 500,
            compression_ratio: 0.5,
            algorithm: CompressionAlgorithm::Delta,
            data: vec![],
        };

        stats.update(&result);
        assert_eq!(stats.total_original, 1000);
        assert_eq!(stats.total_compressed, 500);
        assert_eq!(stats.overall_ratio, 0.5);
        assert_eq!(stats.storage_savings(), 500);
        assert_eq!(stats.storage_savings_percentage(), 50.0);
    }

    #[test]
    fn test_estimate_compression_ratio() {
        let compressor = TimeSeriesCompressor::new();
        
        // Highly repetitive data
        let repetitive_data = vec![10u8; 2000];
        let ratio = compressor.estimate_compression_ratio(&repetitive_data);
        assert!(ratio < 1.0);

        // Small data
        let small_data = vec![1u8, 2, 3];
        let ratio = compressor.estimate_compression_ratio(&small_data);
        assert_eq!(ratio, 1.0);
    }

    #[test]
    fn test_compression_result_serialization() {
        let result = CompressionResult {
            original_size: 1000,
            compressed_size: 500,
            compression_ratio: 0.5,
            algorithm: CompressionAlgorithm::Delta,
            data: vec![1, 2, 3],
        };

        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: CompressionResult = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.original_size, result.original_size);
        assert_eq!(deserialized.compression_ratio, result.compression_ratio);
    }

    #[test]
    fn test_compression_algorithm_equality() {
        assert_eq!(CompressionAlgorithm::Delta, CompressionAlgorithm::Delta);
        assert_ne!(CompressionAlgorithm::Delta, CompressionAlgorithm::Gorilla);
    }

    #[test]
    fn test_compression_config_custom() {
        let config = CompressionConfig {
            algorithm: CompressionAlgorithm::Snappy,
            min_compression_ratio: 0.3,
            max_size_for_compression: 2048,
        };

        assert_eq!(config.algorithm, CompressionAlgorithm::Snappy);
        assert_eq!(config.min_compression_ratio, 0.3);
        assert_eq!(config.max_size_for_compression, 2048);
    }

    #[test]
    fn test_compression_with_custom_config() {
        let config = CompressionConfig {
            algorithm: CompressionAlgorithm::None,
            min_compression_ratio: 0.0,
            max_size_for_compression: 0,
        };

        let compressor = TimeSeriesCompressor::with_config(config);
        let data = vec![1u8, 2, 3, 4, 5];

        let result = compressor.compress(&data).unwrap();
        assert_eq!(result.algorithm, CompressionAlgorithm::None);
    }

    #[test]
    fn test_compression_stats_multiple_updates() {
        let mut stats = CompressionStats::default();

        let result1 = CompressionResult {
            original_size: 1000,
            compressed_size: 500,
            compression_ratio: 0.5,
            algorithm: CompressionAlgorithm::Delta,
            data: vec![],
        };

        let result2 = CompressionResult {
            original_size: 2000,
            compressed_size: 800,
            compression_ratio: 0.4,
            algorithm: CompressionAlgorithm::Gorilla,
            data: vec![],
        };

        stats.update(&result1);
        stats.update(&result2);

        assert_eq!(stats.total_original, 3000);
        assert_eq!(stats.total_compressed, 1300);
        assert_eq!(stats.compression_count, 2);
        assert_eq!(stats.storage_savings(), 1700);
    }
}
