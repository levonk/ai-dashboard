//! GPU metrics collection module using NVML
//! 
//! This module provides GPU monitoring capabilities using the NVIDIA Management Library (NVML)
//! through the nvml-wrapper crate. It collects utilization, memory, temperature, and power metrics.

use anyhow::{Context, Result};
use nvml_wrapper::{Nvml, Device};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

/// GPU metrics collected from NVIDIA GPUs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    /// GPU device index
    pub device_index: u32,
    /// GPU device name
    pub device_name: String,
    /// GPU utilization percentage (0-100)
    pub utilization_percent: Option<f32>,
    /// GPU memory used in MB
    pub memory_used_mb: Option<u64>,
    /// GPU memory total in MB
    pub memory_total_mb: Option<u64>,
    /// GPU memory utilization percentage (0-100)
    pub memory_utilization_percent: Option<f32>,
    /// GPU temperature in Celsius
    pub temperature_celsius: Option<u32>,
    /// GPU power consumption in Watts
    pub power_watts: Option<f32>,
    /// GPU fan speed percentage (0-100)
    pub fan_speed_percent: Option<u32>,
    /// GPU clock speed in MHz
    pub clock_speed_mhz: Option<u32>,
    /// Whether the GPU is currently under load
    pub is_under_load: bool,
}

/// GPU monitoring service
pub struct GpuMonitor {
    nvml: Option<Arc<Nvml>>,
    device_count: u32,
    last_error: Arc<Mutex<Option<String>>>,
}

impl GpuMonitor {
    /// Initialize the GPU monitor
    pub fn new() -> Result<Self> {
        match Nvml::init() {
            Ok(nvml) => {
                let device_count = nvml.device_count().unwrap_or(0);
                info!("GPU monitor initialized with {} device(s)", device_count);
                
                Ok(GpuMonitor {
                    nvml: Some(Arc::new(nvml)),
                    device_count,
                    last_error: Arc::new(Mutex::new(None)),
                })
            }
            Err(e) => {
                let error_msg = format!("Failed to initialize NVML: {}", e);
                warn!("{}", error_msg);
                Ok(GpuMonitor {
                    nvml: None,
                    device_count: 0,
                    last_error: Arc::new(Mutex::new(Some(error_msg))),
                })
            }
        }
    }

    /// Check if GPU monitoring is available
    pub fn is_available(&self) -> bool {
        self.nvml.is_some()
    }

    /// Get the number of GPU devices
    pub fn device_count(&self) -> u32 {
        self.device_count
    }

    /// Get the last error if any
    pub fn last_error(&self) -> Option<String> {
        self.last_error.lock().unwrap().clone()
    }

    /// Collect metrics from all GPU devices
    pub fn collect_metrics(&self) -> Result<Vec<GpuMetrics>> {
        let nvml = self.nvml.as_ref().context("NVML not initialized")?;
        
        let mut metrics = Vec::new();
        
        for i in 0..self.device_count {
            match self.collect_device_metrics(nvml, i) {
                Ok(device_metrics) => {
                    debug!("Collected metrics for GPU device {}", i);
                    metrics.push(device_metrics);
                }
                Err(e) => {
                    error!("Failed to collect metrics for GPU device {}: {}", i, e);
                    // Continue collecting from other devices even if one fails
                }
            }
        }
        
        Ok(metrics)
    }

    /// Collect metrics from a specific GPU device
    fn collect_device_metrics(&self, nvml: &Nvml, device_index: u32) -> Result<GpuMetrics> {
        let device = nvml.device_by_index(device_index)
            .context(format!("Failed to get GPU device {}", device_index))?;

        let device_name = device.name()
            .unwrap_or_else(|_| format!("GPU-{}", device_index));

        // Collect utilization
        let utilization = device.utilization_rates()
            .ok();
        
        let utilization_percent = utilization.as_ref()
            .map(|u| u.gpu as f32);

        // Collect memory information
        let memory_info = device.memory_info()
            .ok();
        
        let memory_used_mb = memory_info.as_ref()
            .map(|m| m.used / 1024 / 1024);
        
        let memory_total_mb = memory_info.as_ref()
            .map(|m| m.total / 1024 / 1024);
        
        let memory_utilization_percent = memory_info.as_ref()
            .and_then(|m| {
                if m.total > 0 {
                    Some((m.used as f32 / m.total as f32) * 100.0)
                } else {
                    None
                }
            });

        // Collect temperature
        let temperature_celsius = device.temperature(nvml_wrapper::TemperatureSensor::Gpu)
            .ok()
            .map(|t| t as u32);

        // Collect power usage
        let power_watts = device.power_usage()
            .ok()
            .map(|p| p as f32 / 1000.0); // Convert milliwatts to watts

        // Collect fan speed
        let fan_speed_percent = device.fan_speed(0)
            .ok()
            .map(|f| f as u32);

        // Collect clock speed
        let clock_speed_mhz = device.clock_info(nvml_wrapper::Clock::Graphics)
            .ok()
            .map(|c| c as u32);

        // Determine if GPU is under load (utilization > 10%)
        let is_under_load = utilization_percent.map_or(false, |u| u > 10.0);

        Ok(GpuMetrics {
            device_index,
            device_name,
            utilization_percent,
            memory_used_mb,
            memory_total_mb,
            memory_utilization_percent,
            temperature_celsius,
            power_watts,
            fan_speed_percent,
            clock_speed_mhz,
            is_under_load,
        })
    }

    /// Collect metrics from a specific GPU device by index
    pub fn collect_device_metrics_by_index(&self, device_index: u32) -> Result<GpuMetrics> {
        let nvml = self.nvml.as_ref().context("NVML not initialized")?;
        
        if device_index >= self.device_count {
            anyhow::bail!("Device index {} out of range (total devices: {})", device_index, self.device_count);
        }
        
        self.collect_device_metrics(nvml, device_index)
    }
}

impl Default for GpuMonitor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            error!("Failed to create GPU monitor: {}", e);
            GpuMonitor {
                nvml: None,
                device_count: 0,
                last_error: Arc::new(Mutex::new(Some(e.to_string()))),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_monitor_initialization() {
        let monitor = GpuMonitor::new();
        assert!(monitor.is_ok());
        
        let monitor = monitor.unwrap();
        // Monitor may or may not be available depending on system
        // Just ensure it doesn't panic
        let _ = monitor.is_available();
    }

    #[test]
    fn test_collect_metrics() {
        let monitor = GpuMonitor::new().unwrap();
        
        if !monitor.is_available() {
            // Skip test if no GPU available
            return;
        }
        
        let metrics = monitor.collect_metrics();
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        // May have 0 or more GPUs
        for metric in metrics {
            assert!(metric.device_index < monitor.device_count());
            assert!(!metric.device_name.is_empty());
        }
    }

    #[test]
    fn test_collect_device_metrics_by_index() {
        let monitor = GpuMonitor::new().unwrap();
        
        if !monitor.is_available() || monitor.device_count() == 0 {
            // Skip test if no GPU available
            return;
        }
        
        let metrics = monitor.collect_device_metrics_by_index(0);
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.device_index, 0);
        assert!(!metrics.device_name.is_empty());
    }

    #[test]
    fn test_invalid_device_index() {
        let monitor = GpuMonitor::new().unwrap();
        
        if !monitor.is_available() {
            // Skip test if no GPU available
            return;
        }
        
        let metrics = monitor.collect_device_metrics_by_index(999);
        assert!(metrics.is_err());
    }
}
