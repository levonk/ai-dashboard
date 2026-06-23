//! Temperature metrics collection module
//! 
//! This module provides temperature monitoring capabilities for GPU and CPU.
//! It aggregates temperature data from GPU and CPU monitoring modules.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Temperature metrics for a specific component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureMetrics {
    /// Component name (e.g., "GPU-0", "CPU-Core-0")
    pub component_name: String,
    /// Component type
    pub component_type: TemperatureComponentType,
    /// Temperature in Celsius
    pub temperature_celsius: f32,
    /// Whether temperature is in normal range
    pub is_normal: bool,
    /// Temperature threshold for warning
    pub warning_threshold_celsius: f32,
    /// Temperature threshold for critical
    pub critical_threshold_celsius: f32,
}

/// Component type for temperature monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemperatureComponentType {
    /// GPU component
    Gpu,
    /// CPU component
    Cpu,
    /// Other component
    Other(String),
}

/// Combined temperature metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTemperatureMetrics {
    /// All temperature readings
    pub temperatures: Vec<TemperatureMetrics>,
    /// Average CPU temperature
    pub average_cpu_temperature: Option<f32>,
    /// Maximum CPU temperature
    pub max_cpu_temperature: Option<f32>,
    /// Average GPU temperature
    pub average_gpu_temperature: Option<f32>,
    /// Maximum GPU temperature
    pub max_gpu_temperature: Option<f32>,
    /// Components with critical temperatures
    pub critical_components: Vec<String>,
    /// Components with warning temperatures
    pub warning_components: Vec<String>,
}

/// Temperature monitoring service
pub struct TemperatureMonitor {
    /// Default warning threshold for CPU (Celsius)
    cpu_warning_threshold: f32,
    /// Default critical threshold for CPU (Celsius)
    cpu_critical_threshold: f32,
    /// Default warning threshold for GPU (Celsius)
    gpu_warning_threshold: f32,
    /// Default critical threshold for GPU (Celsius)
    gpu_critical_threshold: f32,
}

impl TemperatureMonitor {
    /// Initialize the temperature monitor with default thresholds
    pub fn new() -> Self {
        info!("Temperature monitor initialized with default thresholds");
        
        TemperatureMonitor {
            // CPU: warning at 80°C, critical at 90°C
            cpu_warning_threshold: 80.0,
            cpu_critical_threshold: 90.0,
            // GPU: warning at 85°C, critical at 95°C
            gpu_warning_threshold: 85.0,
            gpu_critical_threshold: 95.0,
        }
    }

    /// Initialize the temperature monitor with custom thresholds
    pub fn with_thresholds(
        cpu_warning: f32,
        cpu_critical: f32,
        gpu_warning: f32,
        gpu_critical: f32,
    ) -> Self {
        info!("Temperature monitor initialized with custom thresholds");
        
        TemperatureMonitor {
            cpu_warning_threshold: cpu_warning,
            cpu_critical_threshold: cpu_critical,
            gpu_warning_threshold: gpu_warning,
            gpu_critical_threshold: gpu_critical,
        }
    }

    /// Check if temperature monitoring is available
    pub fn is_available(&self) -> bool {
        true // Temperature monitoring is always available through aggregation
    }

    /// Collect temperature metrics from GPU data
    pub fn collect_gpu_temperatures(
        &self,
        gpu_metrics: Vec<super::gpu::GpuMetrics>,
    ) -> Vec<TemperatureMetrics> {
        gpu_metrics.into_iter()
            .filter_map(|gpu| {
                gpu.temperature_celsius.map(|temp| {
                    let component_name = format!("GPU-{}", gpu.device_index);
                    let is_normal = temp < self.gpu_warning_threshold;
                    
                    TemperatureMetrics {
                        component_name: component_name.clone(),
                        component_type: TemperatureComponentType::Gpu,
                        temperature_celsius: temp as f32,
                        is_normal,
                        warning_threshold_celsius: self.gpu_warning_threshold,
                        critical_threshold_celsius: self.gpu_critical_threshold,
                    }
                })
            })
            .collect()
    }

    /// Collect temperature metrics from CPU data
    /// Note: sysinfo doesn't provide per-core temperature, so we use a placeholder
    /// In production, this would use platform-specific temperature sensors
    pub fn collect_cpu_temperatures(
        &self,
        cpu_metrics: &super::cpu::SystemCpuMetrics,
    ) -> Vec<TemperatureMetrics> {
        // Since sysinfo doesn't provide CPU temperature, we'll create a placeholder
        // In production, this would use platform-specific APIs like lm-sensors on Linux
        // For now, we'll return empty vector and log a warning
        debug!("CPU temperature collection not fully implemented - requires platform-specific sensors");
        
        vec![]
    }

    /// Collect all temperature metrics
    pub fn collect_metrics(
        &self,
        gpu_metrics: Vec<super::gpu::GpuMetrics>,
        cpu_metrics: &super::cpu::SystemCpuMetrics,
    ) -> SystemTemperatureMetrics {
        let gpu_temps = self.collect_gpu_temperatures(gpu_metrics);
        let cpu_temps = self.collect_cpu_temperatures(cpu_metrics);
        
        let temperatures = [gpu_temps, cpu_temps].concat();
        
        // Calculate statistics
        let gpu_temp_values: Vec<f32> = temperatures.iter()
            .filter(|t| matches!(t.component_type, TemperatureComponentType::Gpu))
            .map(|t| t.temperature_celsius)
            .collect();
        
        let cpu_temp_values: Vec<f32> = temperatures.iter()
            .filter(|t| matches!(t.component_type, TemperatureComponentType::Cpu))
            .map(|t| t.temperature_celsius)
            .collect();
        
        let average_gpu_temperature = if gpu_temp_values.is_empty() {
            None
        } else {
            Some(gpu_temp_values.iter().sum::<f32>() / gpu_temp_values.len() as f32)
        };
        
        let max_gpu_temperature = gpu_temp_values.iter().cloned().max();
        
        let average_cpu_temperature = if cpu_temp_values.is_empty() {
            None
        } else {
            Some(cpu_temp_values.iter().sum::<f32>() / cpu_temp_values.len() as f32)
        };
        
        let max_cpu_temperature = cpu_temp_values.iter().cloned().max();
        
        // Identify critical and warning components
        let critical_components: Vec<String> = temperatures.iter()
            .filter(|t| t.temperature_celsius >= t.critical_threshold_celsius)
            .map(|t| t.component_name.clone())
            .collect();
        
        let warning_components: Vec<String> = temperatures.iter()
            .filter(|t| t.temperature_celsius >= t.warning_threshold_celsius && t.temperature_celsius < t.critical_threshold_celsius)
            .map(|t| t.component_name.clone())
            .collect();
        
        SystemTemperatureMetrics {
            temperatures,
            average_cpu_temperature,
            max_cpu_temperature,
            average_gpu_temperature,
            max_gpu_temperature,
            critical_components,
            warning_components,
        }
    }
}

impl Default for TemperatureMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_monitor_initialization() {
        let monitor = TemperatureMonitor::new();
        assert!(monitor.is_available());
    }

    #[test]
    fn test_collect_gpu_temperatures() {
        let monitor = TemperatureMonitor::new();
        
        let gpu_metrics = vec![
            super::gpu::GpuMetrics {
                device_index: 0,
                device_name: "Test GPU".to_string(),
                utilization_percent: Some(50.0),
                memory_used_mb: Some(4096),
                memory_total_mb: Some(8192),
                memory_utilization_percent: Some(50.0),
                temperature_celsius: Some(65),
                power_watts: Some(150.0),
                fan_speed_percent: Some(70),
                clock_speed_mhz: Some(1800),
                is_under_load: true,
            }
        ];
        
        let temps = monitor.collect_gpu_temperatures(gpu_metrics);
        assert_eq!(temps.len(), 1);
        
        let temp = &temps[0];
        assert_eq!(temp.component_name, "GPU-0");
        assert_eq!(temp.temperature_celsius, 65.0);
        assert!(temp.is_normal);
    }

    #[test]
    fn test_collect_gpu_temperatures_critical() {
        let monitor = TemperatureMonitor::new();
        
        let gpu_metrics = vec![
            super::gpu::GpuMetrics {
                device_index: 0,
                device_name: "Test GPU".to_string(),
                utilization_percent: Some(50.0),
                memory_used_mb: Some(4096),
                memory_total_mb: Some(8192),
                memory_utilization_percent: Some(50.0),
                temperature_celsius: Some(100), // Above critical threshold
                power_watts: Some(150.0),
                fan_speed_percent: Some(70),
                clock_speed_mhz: Some(1800),
                is_under_load: true,
            }
        ];
        
        let temps = monitor.collect_gpu_temperatures(gpu_metrics);
        assert_eq!(temps.len(), 1);
        
        let temp = &temps[0];
        assert!(!temp.is_normal);
        assert!(temp.temperature_celsius >= temp.critical_threshold_celsius);
    }

    #[test]
    fn test_collect_metrics() {
        let monitor = TemperatureMonitor::new();
        
        let gpu_metrics = vec![
            super::gpu::GpuMetrics {
                device_index: 0,
                device_name: "Test GPU".to_string(),
                utilization_percent: Some(50.0),
                memory_used_mb: Some(4096),
                memory_total_mb: Some(8192),
                memory_utilization_percent: Some(50.0),
                temperature_celsius: Some(65),
                power_watts: Some(150.0),
                fan_speed_percent: Some(70),
                clock_speed_mhz: Some(1800),
                is_under_load: true,
            }
        ];
        
        let cpu_metrics = super::cpu::SystemCpuMetrics {
            total_utilization_percent: 50.0,
            physical_cores: 4,
            logical_cores: 8,
            vendor_id: "Test".to_string(),
            brand: "Test CPU".to_string(),
            cores: vec![],
            load_average: None,
        };
        
        let metrics = monitor.collect_metrics(gpu_metrics, &cpu_metrics);
        assert_eq!(metrics.temperatures.len(), 1);
        assert_eq!(metrics.average_gpu_temperature, Some(65.0));
        assert_eq!(metrics.max_gpu_temperature, Some(65.0));
        assert!(metrics.critical_components.is_empty());
        assert!(metrics.warning_components.is_empty());
    }
}
