//! Power metrics collection module
//! 
//! This module provides power monitoring capabilities for GPU and system power consumption.
//! It aggregates power data from GPU monitoring and provides system-wide power metrics.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Power metrics for a specific component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerMetrics {
    /// Component name (e.g., "GPU-0", "System")
    pub component_name: String,
    /// Component type
    pub component_type: PowerComponentType,
    /// Power consumption in Watts
    pub power_watts: f32,
    /// Whether power consumption is in normal range
    pub is_normal: bool,
    /// Power threshold for warning (Watts)
    pub warning_threshold_watts: f32,
    /// Power threshold for critical (Watts)
    pub critical_threshold_watts: f32,
}

/// Component type for power monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PowerComponentType {
    /// GPU component
    Gpu,
    /// System component
    System,
    /// Other component
    Other(String),
}

/// Combined power metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPowerMetrics {
    /// All power readings
    pub power_readings: Vec<PowerMetrics>,
    /// Total GPU power consumption
    pub total_gpu_power: f32,
    /// Average GPU power consumption
    pub average_gpu_power: Option<f32>,
    /// Maximum GPU power consumption
    pub max_gpu_power: Option<f32>,
    /// Estimated total system power (Watts)
    pub estimated_total_system_power: f32,
    /// Components with critical power consumption
    pub critical_components: Vec<String>,
    /// Components with warning power consumption
    pub warning_components: Vec<String>,
}

/// Power monitoring service
pub struct PowerMonitor {
    /// Default warning threshold for GPU (Watts)
    gpu_warning_threshold: f32,
    /// Default critical threshold for GPU (Watts)
    gpu_critical_threshold: f32,
    /// Base system power consumption estimate (Watts)
    /// This is the power consumed by the system without GPU load
    base_system_power: f32,
}

impl PowerMonitor {
    /// Initialize the power monitor with default thresholds
    pub fn new() -> Self {
        info!("Power monitor initialized with default thresholds");
        
        PowerMonitor {
            // GPU: warning at 250W, critical at 350W (typical for high-end GPUs)
            gpu_warning_threshold: 250.0,
            gpu_critical_threshold: 350.0,
            // Base system power estimate (CPU, motherboard, etc.)
            base_system_power: 100.0,
        }
    }

    /// Initialize the power monitor with custom thresholds
    pub fn with_thresholds(
        gpu_warning: f32,
        gpu_critical: f32,
        base_system_power: f32,
    ) -> Self {
        info!("Power monitor initialized with custom thresholds");
        
        PowerMonitor {
            gpu_warning_threshold: gpu_warning,
            gpu_critical_threshold: gpu_critical,
            base_system_power,
        }
    }

    /// Check if power monitoring is available
    pub fn is_available(&self) -> bool {
        true // Power monitoring is always available through aggregation
    }

    /// Collect power metrics from GPU data
    pub fn collect_gpu_power(
        &self,
        gpu_metrics: Vec<super::gpu::GpuMetrics>,
    ) -> Vec<PowerMetrics> {
        gpu_metrics.into_iter()
            .filter_map(|gpu| {
                gpu.power_watts.map(|power| {
                    let component_name = format!("GPU-{}", gpu.device_index);
                    let is_normal = power < self.gpu_warning_threshold;
                    
                    PowerMetrics {
                        component_name: component_name.clone(),
                        component_type: PowerComponentType::Gpu,
                        power_watts: power,
                        is_normal,
                        warning_threshold_watts: self.gpu_warning_threshold,
                        critical_threshold_watts: self.gpu_critical_threshold,
                    }
                })
            })
            .collect()
    }

    /// Estimate system power consumption
    /// This is a rough estimate based on CPU utilization and base system power
    pub fn estimate_system_power(&self, cpu_metrics: &super::cpu::SystemCpuMetrics) -> f32 {
        // Estimate CPU power based on utilization
        // Assume max CPU power of 150W for high-end CPUs
        let cpu_power_estimate = (cpu_metrics.total_utilization_percent / 100.0) * 150.0;
        
        // Total system power = base + CPU power
        self.base_system_power + cpu_power_estimate
    }

    /// Collect all power metrics
    pub fn collect_metrics(
        &self,
        gpu_metrics: Vec<super::gpu::GpuMetrics>,
        cpu_metrics: &super::cpu::SystemCpuMetrics,
    ) -> SystemPowerMetrics {
        let gpu_power = self.collect_gpu_power(gpu_metrics);
        
        // Calculate GPU power statistics
        let gpu_power_values: Vec<f32> = gpu_power.iter()
            .map(|p| p.power_watts)
            .collect();
        
        let total_gpu_power = gpu_power_values.iter().sum();
        let average_gpu_power = if gpu_power_values.is_empty() {
            None
        } else {
            Some(total_gpu_power / gpu_power_values.len() as f32)
        };
        let max_gpu_power = gpu_power_values.iter().cloned().max();
        
        // Estimate total system power
        let estimated_system_power = self.estimate_system_power(cpu_metrics);
        let estimated_total_system_power = estimated_system_power + total_gpu_power;
        
        // Create system power reading
        let system_power = PowerMetrics {
            component_name: "System".to_string(),
            component_type: PowerComponentType::System,
            power_watts: estimated_system_power,
            is_normal: true, // System power is typically within normal range
            warning_threshold_watts: 500.0, // High threshold for total system
            critical_threshold_watts: 700.0, // Very high threshold for total system
        };
        
        let power_readings = [gpu_power, vec![system_power]].concat();
        
        // Identify critical and warning components
        let critical_components: Vec<String> = power_readings.iter()
            .filter(|p| p.power_watts >= p.critical_threshold_watts)
            .map(|p| p.component_name.clone())
            .collect();
        
        let warning_components: Vec<String> = power_readings.iter()
            .filter(|p| p.power_watts >= p.warning_threshold_watts && p.power_watts < p.critical_threshold_watts)
            .map(|p| p.component_name.clone())
            .collect();
        
        SystemPowerMetrics {
            power_readings,
            total_gpu_power,
            average_gpu_power,
            max_gpu_power,
            estimated_total_system_power,
            critical_components,
            warning_components,
        }
    }
}

impl Default for PowerMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_monitor_initialization() {
        let monitor = PowerMonitor::new();
        assert!(monitor.is_available());
    }

    #[test]
    fn test_collect_gpu_power() {
        let monitor = PowerMonitor::new();
        
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
        
        let power = monitor.collect_gpu_power(gpu_metrics);
        assert_eq!(power.len(), 1);
        
        let p = &power[0];
        assert_eq!(p.component_name, "GPU-0");
        assert_eq!(p.power_watts, 150.0);
        assert!(p.is_normal);
    }

    #[test]
    fn test_collect_gpu_power_critical() {
        let monitor = PowerMonitor::new();
        
        let gpu_metrics = vec![
            super::gpu::GpuMetrics {
                device_index: 0,
                device_name: "Test GPU".to_string(),
                utilization_percent: Some(50.0),
                memory_used_mb: Some(4096),
                memory_total_mb: Some(8192),
                memory_utilization_percent: Some(50.0),
                temperature_celsius: Some(65),
                power_watts: Some(400.0), // Above critical threshold
                fan_speed_percent: Some(70),
                clock_speed_mhz: Some(1800),
                is_under_load: true,
            }
        ];
        
        let power = monitor.collect_gpu_power(gpu_metrics);
        assert_eq!(power.len(), 1);
        
        let p = &power[0];
        assert!(!p.is_normal);
        assert!(p.power_watts >= p.critical_threshold_watts);
    }

    #[test]
    fn test_estimate_system_power() {
        let monitor = PowerMonitor::new();
        
        let cpu_metrics = super::cpu::SystemCpuMetrics {
            total_utilization_percent: 50.0,
            physical_cores: 4,
            logical_cores: 8,
            vendor_id: "Test".to_string(),
            brand: "Test CPU".to_string(),
            cores: vec![],
            load_average: None,
        };
        
        let power = monitor.estimate_system_power(&cpu_metrics);
        // Base 100W + 50% of 150W = 175W
        assert!((power - 175.0).abs() < 1.0);
    }

    #[test]
    fn test_collect_metrics() {
        let monitor = PowerMonitor::new();
        
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
        assert_eq!(metrics.power_readings.len(), 2); // 1 GPU + 1 System
        assert_eq!(metrics.total_gpu_power, 150.0);
        assert_eq!(metrics.average_gpu_power, Some(150.0));
        assert_eq!(metrics.max_gpu_power, Some(150.0));
        assert!(metrics.estimated_total_system_power > 150.0);
        assert!(metrics.critical_components.is_empty());
        assert!(metrics.warning_components.is_empty());
    }
}
