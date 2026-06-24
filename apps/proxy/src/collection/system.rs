//! System resource metrics collection module
//! 
//! This module provides system-level metrics collection including GPU/CPU utilization,
//! memory usage, temperatures, power consumption, and clock speeds. It builds on the
//! hardware monitoring modules to provide a unified interface for system resource metrics.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

use crate::monitoring::{
    HardwareMonitor, GpuMetrics, SystemCpuMetrics, 
    MemoryMetrics,
    SystemTemperatureMetrics,
    SystemPowerMetrics
};

/// System resource metrics collected at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// GPU utilization metrics
    pub gpu_utilization: Vec<GpuUtilizationMetrics>,
    /// CPU utilization metrics
    pub cpu_utilization: CpuUtilizationMetrics,
    /// Memory usage metrics
    pub memory_usage: MemoryUsageMetrics,
    /// Temperature metrics
    pub temperature: TemperatureUsageMetrics,
    /// Power consumption metrics
    pub power_consumption: PowerUsageMetrics,
    /// Clock speed metrics
    pub clock_speeds: ClockSpeedMetrics,
    /// Sample rate in Hz
    pub sample_rate_hz: f32,
    /// Timestamp of metrics collection
    pub timestamp: DateTime<Utc>,
}

/// GPU utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuUtilizationMetrics {
    /// GPU device index
    pub device_index: u32,
    /// GPU device name
    pub device_name: String,
    /// GPU utilization percentage (0-100)
    pub utilization_percent: Option<f32>,
    /// Whether the GPU is currently under load
    pub is_under_load: bool,
}

/// CPU utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUtilizationMetrics {
    /// Total CPU utilization percentage (0-100)
    pub total_utilization_percent: f32,
    /// Number of physical CPU cores
    pub physical_cores: usize,
    /// Number of logical CPU cores
    pub logical_cores: usize,
    /// Individual core utilization
    pub core_utilization: Vec<CoreUtilization>,
    /// System load average (1, 5, 15 minutes)
    pub load_average: Option<(f64, f64, f64)>,
}

/// Individual core utilization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreUtilization {
    /// Core index
    pub core_index: usize,
    /// Core name
    pub core_name: String,
    /// Utilization percentage (0-100)
    pub utilization_percent: f32,
    /// Whether the core is under load
    pub is_under_load: bool,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageMetrics {
    /// GPU memory usage
    pub gpu_memory: Vec<GpuMemoryUsage>,
    /// CPU memory usage
    pub cpu_memory: CpuMemoryUsage,
    /// Unified memory usage (if available)
    pub unified_memory: Option<UnifiedMemoryUsage>,
}

/// GPU memory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMemoryUsage {
    /// GPU device index
    pub device_index: u32,
    /// GPU device name
    pub device_name: String,
    /// Memory used in MB
    pub memory_used_mb: Option<u64>,
    /// Memory total in MB
    pub memory_total_mb: Option<u64>,
    /// Memory utilization percentage (0-100)
    pub memory_utilization_percent: Option<f32>,
}

/// CPU memory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMemoryUsage {
    /// Memory used in MB
    pub memory_used_mb: u64,
    /// Memory total in MB
    pub memory_total_mb: u64,
    /// Memory utilization percentage (0-100)
    pub memory_utilization_percent: f32,
    /// Available memory in MB
    pub memory_available_mb: u64,
}

/// Unified memory usage (for systems with shared GPU/CPU memory)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMemoryUsage {
    /// Unified memory used in MB
    pub memory_used_mb: u64,
    /// Unified memory total in MB
    pub memory_total_mb: u64,
    /// Unified memory utilization percentage (0-100)
    pub memory_utilization_percent: f32,
}

/// Temperature usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureUsageMetrics {
    /// GPU temperatures
    pub gpu_temperatures: Vec<GpuTemperature>,
    /// CPU temperatures
    pub cpu_temperatures: Vec<CpuTemperature>,
}

/// GPU temperature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuTemperature {
    /// GPU device index
    pub device_index: u32,
    /// GPU device name
    pub device_name: String,
    /// Temperature in Celsius
    pub temperature_celsius: Option<u32>,
}

/// CPU temperature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuTemperature {
    /// CPU core index
    pub core_index: usize,
    /// Core name
    pub core_name: String,
    /// Temperature in Celsius (if available)
    pub temperature_celsius: Option<u32>,
}

/// Power consumption metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerUsageMetrics {
    /// GPU power consumption
    pub gpu_power: Vec<GpuPower>,
    /// Estimated system power consumption
    pub system_power_watts: Option<f32>,
}

/// GPU power consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuPower {
    /// GPU device index
    pub device_index: u32,
    /// GPU device name
    pub device_name: String,
    /// Power consumption in Watts
    pub power_watts: Option<f32>,
}

/// Clock speed metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockSpeedMetrics {
    /// GPU clock speeds
    pub gpu_clock_speeds: Vec<GpuClockSpeed>,
    /// CPU clock speeds
    pub cpu_clock_speeds: Vec<CpuClockSpeed>,
}

/// GPU clock speed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuClockSpeed {
    /// GPU device index
    pub device_index: u32,
    /// GPU device name
    pub device_name: String,
    /// Clock speed in MHz
    pub clock_speed_mhz: Option<u32>,
}

/// CPU clock speed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuClockSpeed {
    /// CPU core index
    pub core_index: usize,
    /// Core name
    pub core_name: String,
    /// Clock speed in MHz
    pub clock_speed_mhz: Option<u64>,
}

/// System metrics collector
pub struct SystemMetricsCollector {
    hardware_monitor: Arc<HardwareMonitor>,
    sample_rate_hz: f32,
}

impl SystemMetricsCollector {
    /// Initialize the system metrics collector
    pub fn new(hardware_monitor: Arc<HardwareMonitor>) -> Self {
        info!("Initializing system metrics collector");
        
        SystemMetricsCollector {
            hardware_monitor,
            sample_rate_hz: 1.0, // Default 1 Hz (1 second interval)
        }
    }

    /// Initialize with custom sample rate
    pub fn with_sample_rate(hardware_monitor: Arc<HardwareMonitor>, sample_rate_hz: f32) -> Self {
        info!("Initializing system metrics collector with sample rate: {} Hz", sample_rate_hz);
        
        SystemMetricsCollector {
            hardware_monitor,
            sample_rate_hz,
        }
    }

    /// Get the current sample rate
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate_hz
    }

    /// Set the sample rate
    pub fn set_sample_rate(&mut self, sample_rate_hz: f32) {
        info!("Updating sample rate from {} Hz to {} Hz", self.sample_rate_hz, sample_rate_hz);
        self.sample_rate_hz = sample_rate_hz;
    }

    /// Collect system metrics
    pub fn collect_metrics(&self) -> Result<SystemMetrics> {
        debug!("Collecting system metrics");
        
        let hardware_metrics = self.hardware_monitor.collect_metrics()
            .context("Failed to collect hardware metrics")?;
        
        // Transform hardware metrics into system metrics
        let gpu_utilization = self.extract_gpu_utilization(&hardware_metrics.gpu);
        let cpu_utilization = self.extract_cpu_utilization(&hardware_metrics.cpu);
        let memory_usage = self.extract_memory_usage(&hardware_metrics.memory);
        let temperature = self.extract_temperature(&hardware_metrics.gpu, &hardware_metrics.cpu, &hardware_metrics.temperature);
        let power_consumption = self.extract_power(&hardware_metrics.gpu, &hardware_metrics.power);
        let clock_speeds = self.extract_clock_speeds(&hardware_metrics.gpu, &hardware_metrics.cpu);
        
        let timestamp = hardware_metrics.timestamp;
        
        debug!("System metrics collection completed");
        
        Ok(SystemMetrics {
            gpu_utilization,
            cpu_utilization,
            memory_usage,
            temperature,
            power_consumption,
            clock_speeds,
            sample_rate_hz: self.sample_rate_hz,
            timestamp,
        })
    }

    /// Extract GPU utilization metrics
    fn extract_gpu_utilization(&self, gpu_metrics: &[GpuMetrics]) -> Vec<GpuUtilizationMetrics> {
        gpu_metrics.iter().map(|gpu| GpuUtilizationMetrics {
            device_index: gpu.device_index,
            device_name: gpu.device_name.clone(),
            utilization_percent: gpu.utilization_percent,
            is_under_load: gpu.is_under_load,
        }).collect()
    }

    /// Extract CPU utilization metrics
    fn extract_cpu_utilization(&self, cpu_metrics: &SystemCpuMetrics) -> CpuUtilizationMetrics {
        let core_utilization: Vec<CoreUtilization> = cpu_metrics.cores.iter().map(|core| CoreUtilization {
            core_index: core.core_index,
            core_name: core.core_name.clone(),
            utilization_percent: core.utilization_percent,
            is_under_load: core.is_under_load,
        }).collect();

        CpuUtilizationMetrics {
            total_utilization_percent: cpu_metrics.total_utilization_percent,
            physical_cores: cpu_metrics.physical_cores,
            logical_cores: cpu_metrics.logical_cores,
            core_utilization,
            load_average: cpu_metrics.load_average,
        }
    }

    /// Extract memory usage metrics
    fn extract_memory_usage(&self, memory_metrics: &MemoryMetrics) -> MemoryUsageMetrics {
        let gpu_memory: Vec<GpuMemoryUsage> = memory_metrics.gpu_memory.iter().map(|mem| GpuMemoryUsage {
            device_index: mem.device_index,
            device_name: mem.device_name.clone(),
            memory_used_mb: mem.memory_used_mb,
            memory_total_mb: mem.memory_total_mb,
            memory_utilization_percent: mem.memory_utilization_percent,
        }).collect();

        let cpu_memory = CpuMemoryUsage {
            memory_used_mb: memory_metrics.system_memory.used_memory_mb,
            memory_total_mb: memory_metrics.system_memory.total_memory_mb,
            memory_utilization_percent: memory_metrics.system_memory.utilization_percent,
            memory_available_mb: memory_metrics.system_memory.available_memory_mb,
        };

        let unified_memory = memory_metrics.unified_memory.as_ref().map(|mem| UnifiedMemoryUsage {
            memory_used_mb: mem.memory_used_mb,
            memory_total_mb: mem.memory_total_mb,
            memory_utilization_percent: mem.memory_utilization_percent,
        });

        MemoryUsageMetrics {
            gpu_memory,
            cpu_memory,
            unified_memory,
        }
    }

    /// Extract temperature metrics
    fn extract_temperature(
        &self, 
        gpu_metrics: &[GpuMetrics], 
        cpu_metrics: &SystemCpuMetrics,
        temp_metrics: &SystemTemperatureMetrics
    ) -> TemperatureUsageMetrics {
        let gpu_temperatures: Vec<GpuTemperature> = gpu_metrics.iter().map(|gpu| GpuTemperature {
            device_index: gpu.device_index,
            device_name: gpu.device_name.clone(),
            temperature_celsius: gpu.temperature_celsius,
        }).collect();

        let cpu_temperatures: Vec<CpuTemperature> = cpu_metrics.cores.iter().enumerate().map(|(index, core)| {
            // Find corresponding temperature if available
            let temperature_celsius = temp_metrics.cpu_temperatures
                .iter()
                .find(|t| t.component_index == index)
                .and_then(|t| Some(t.temperature_celsius));

            CpuTemperature {
                core_index: index,
                core_name: core.core_name.clone(),
                temperature_celsius,
            }
        }).collect();

        TemperatureUsageMetrics {
            gpu_temperatures,
            cpu_temperatures,
        }
    }

    /// Extract power consumption metrics
    fn extract_power(&self, gpu_metrics: &[GpuMetrics], power_metrics: &SystemPowerMetrics) -> PowerUsageMetrics {
        let gpu_power: Vec<GpuPower> = gpu_metrics.iter().map(|gpu| GpuPower {
            device_index: gpu.device_index,
            device_name: gpu.device_name.clone(),
            power_watts: gpu.power_watts,
        }).collect();

        PowerUsageMetrics {
            gpu_power,
            system_power_watts: power_metrics.total_system_power_watts,
        }
    }

    /// Extract clock speed metrics
    fn extract_clock_speeds(&self, gpu_metrics: &[GpuMetrics], cpu_metrics: &SystemCpuMetrics) -> ClockSpeedMetrics {
        let gpu_clock_speeds: Vec<GpuClockSpeed> = gpu_metrics.iter().map(|gpu| GpuClockSpeed {
            device_index: gpu.device_index,
            device_name: gpu.device_name.clone(),
            clock_speed_mhz: gpu.clock_speed_mhz,
        }).collect();

        let cpu_clock_speeds: Vec<CpuClockSpeed> = cpu_metrics.cores.iter().map(|core| CpuClockSpeed {
            core_index: core.core_index,
            core_name: core.core_name.clone(),
            clock_speed_mhz: core.frequency_mhz,
        }).collect();

        ClockSpeedMetrics {
            gpu_clock_speeds,
            cpu_clock_speeds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_metrics_collector_initialization() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = SystemMetricsCollector::new(hardware_monitor);
        
        assert_eq!(collector.sample_rate(), 1.0);
    }

    #[test]
    fn test_system_metrics_collector_with_sample_rate() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = SystemMetricsCollector::with_sample_rate(hardware_monitor, 2.0);
        
        assert_eq!(collector.sample_rate(), 2.0);
    }

    #[test]
    fn test_set_sample_rate() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let mut collector = SystemMetricsCollector::new(hardware_monitor);
        
        collector.set_sample_rate(5.0);
        assert_eq!(collector.sample_rate(), 5.0);
    }

    #[test]
    fn test_collect_metrics() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = SystemMetricsCollector::new(hardware_monitor);
        
        let metrics = collector.collect_metrics();
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        assert!(metrics.timestamp > chrono::Utc::now() - chrono::Duration::seconds(10));
        assert_eq!(metrics.sample_rate_hz, 1.0);
    }

    #[test]
    fn test_gpu_utilization_extraction() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = SystemMetricsCollector::new(hardware_monitor);
        
        let hardware_metrics = hardware_monitor.collect_metrics().unwrap();
        let gpu_utilization = collector.extract_gpu_utilization(&hardware_metrics.gpu);
        
        // Verify extraction preserves data
        for (i, util) in gpu_utilization.iter().enumerate() {
            assert_eq!(util.device_index, hardware_metrics.gpu[i].device_index);
            assert_eq!(util.device_name, hardware_metrics.gpu[i].device_name);
            assert_eq!(util.utilization_percent, hardware_metrics.gpu[i].utilization_percent);
            assert_eq!(util.is_under_load, hardware_metrics.gpu[i].is_under_load);
        }
    }

    #[test]
    fn test_cpu_utilization_extraction() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = SystemMetricsCollector::new(hardware_monitor);
        
        let hardware_metrics = hardware_monitor.collect_metrics().unwrap();
        let cpu_utilization = collector.extract_cpu_utilization(&hardware_metrics.cpu);
        
        assert_eq!(cpu_utilization.total_utilization_percent, hardware_metrics.cpu.total_utilization_percent);
        assert_eq!(cpu_utilization.physical_cores, hardware_metrics.cpu.physical_cores);
        assert_eq!(cpu_utilization.logical_cores, hardware_metrics.cpu.logical_cores);
        assert_eq!(cpu_utilization.core_utilization.len(), hardware_metrics.cpu.cores.len());
    }

    #[test]
    fn test_memory_usage_extraction() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = SystemMetricsCollector::new(hardware_monitor);
        
        let hardware_metrics = hardware_monitor.collect_metrics().unwrap();
        let memory_usage = collector.extract_memory_usage(&hardware_metrics.memory);
        
        assert_eq!(memory_usage.gpu_memory.len(), hardware_metrics.memory.gpu_memory.len());
        assert_eq!(memory_usage.cpu_memory.memory_used_mb, hardware_metrics.memory.system_memory.used_memory_mb);
        assert_eq!(memory_usage.cpu_memory.memory_total_mb, hardware_metrics.memory.system_memory.total_memory_mb);
    }

    #[test]
    fn test_temperature_extraction() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = SystemMetricsCollector::new(hardware_monitor);
        
        let hardware_metrics = hardware_monitor.collect_metrics().unwrap();
        let temperature = collector.extract_temperature(&hardware_metrics.gpu, &hardware_metrics.cpu, &hardware_metrics.temperature);
        
        assert_eq!(temperature.gpu_temperatures.len(), hardware_metrics.gpu.len());
        assert_eq!(temperature.cpu_temperatures.len(), hardware_metrics.cpu.cores.len());
    }

    #[test]
    fn test_power_extraction() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = SystemMetricsCollector::new(hardware_monitor);
        
        let hardware_metrics = hardware_monitor.collect_metrics().unwrap();
        let power = collector.extract_power(&hardware_metrics.gpu, &hardware_metrics.power);
        
        assert_eq!(power.gpu_power.len(), hardware_metrics.gpu.len());
    }

    #[test]
    fn test_clock_speeds_extraction() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = SystemMetricsCollector::new(hardware_monitor);
        
        let hardware_metrics = hardware_monitor.collect_metrics().unwrap();
        let clock_speeds = collector.extract_clock_speeds(&hardware_metrics.gpu, &hardware_metrics.cpu);
        
        assert_eq!(clock_speeds.gpu_clock_speeds.len(), hardware_metrics.gpu.len());
        assert_eq!(clock_speeds.cpu_clock_speeds.len(), hardware_metrics.cpu.cores.len());
    }
}