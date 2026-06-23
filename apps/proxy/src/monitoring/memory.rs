//! Memory metrics collection module using sysinfo
//! 
//! This module provides memory monitoring capabilities using the sysinfo crate.
//! It collects GPU, CPU, and unified memory usage metrics.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

/// Memory metrics for GPU memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMemoryMetrics {
    /// GPU device index
    pub device_index: u32,
    /// GPU device name
    pub device_name: String,
    /// Memory used in MB
    pub memory_used_mb: u64,
    /// Memory total in MB
    pub memory_total_mb: u64,
    /// Memory free in MB
    pub memory_free_mb: u64,
    /// Memory utilization percentage (0-100)
    pub utilization_percent: f32,
}

/// Memory metrics for system memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMemoryMetrics {
    /// Total system memory in MB
    pub total_memory_mb: u64,
    /// Used system memory in MB
    pub used_memory_mb: u64,
    /// Free system memory in MB
    pub free_memory_mb: u64,
    /// Available system memory in MB
    pub available_memory_mb: u64,
    /// Memory utilization percentage (0-100)
    pub utilization_percent: f32,
    /// Total swap space in MB
    pub total_swap_mb: u64,
    /// Used swap space in MB
    pub used_swap_mb: u64,
    /// Free swap space in MB
    pub free_swap_mb: u64,
    /// Swap utilization percentage (0-100)
    pub swap_utilization_percent: f32,
}

/// Combined memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// System memory metrics
    pub system: SystemMemoryMetrics,
    /// GPU memory metrics (if available)
    pub gpu_memory: Vec<GpuMemoryMetrics>,
}

/// Memory monitoring service
pub struct MemoryMonitor {
    system: Arc<Mutex<System>>,
}

impl MemoryMonitor {
    /// Initialize the memory monitor
    pub fn new() -> Result<Self> {
        let system = System::new_all();
        
        info!("Memory monitor initialized");
        
        Ok(MemoryMonitor {
            system: Arc::new(Mutex::new(system)),
        })
    }

    /// Check if memory monitoring is available
    pub fn is_available(&self) -> bool {
        true // sysinfo is always available on supported platforms
    }

    /// Collect system memory metrics
    pub fn collect_system_metrics(&self) -> Result<SystemMemoryMetrics> {
        let mut system = self.system.lock()
            .context("Failed to acquire system lock")?;
        
        // Refresh memory information
        system.refresh_memory();
        
        let total_memory_kb = system.total_memory();
        let available_memory_kb = system.available_memory();
        let used_memory_kb = total_memory_kb.saturating_sub(available_memory_kb);
        let free_memory_kb = available_memory_kb;
        
        let total_memory_mb = total_memory_kb / 1024;
        let used_memory_mb = used_memory_kb / 1024;
        let free_memory_mb = free_memory_kb / 1024;
        let available_memory_mb = available_memory_kb / 1024;
        
        let utilization_percent = if total_memory_mb > 0 {
            (used_memory_mb as f32 / total_memory_mb as f32) * 100.0
        } else {
            0.0
        };
        
        let total_swap_kb = system.total_swap();
        let free_swap_kb = system.free_swap();
        let used_swap_kb = total_swap_kb.saturating_sub(free_swap_kb);
        
        let total_swap_mb = total_swap_kb / 1024;
        let used_swap_mb = used_swap_kb / 1024;
        let free_swap_mb = free_swap_kb / 1024;
        
        let swap_utilization_percent = if total_swap_mb > 0 {
            (used_swap_mb as f32 / total_swap_mb as f32) * 100.0
        } else {
            0.0
        };
        
        Ok(SystemMemoryMetrics {
            total_memory_mb,
            used_memory_mb,
            free_memory_mb,
            available_memory_mb,
            utilization_percent,
            total_swap_mb,
            used_swap_mb,
            free_swap_mb,
            swap_utilization_percent,
        })
    }

    /// Collect GPU memory metrics (requires GPU monitor)
    pub fn collect_gpu_metrics(&self, gpu_metrics: Vec<super::gpu::GpuMetrics>) -> Vec<GpuMemoryMetrics> {
        gpu_metrics.into_iter()
            .map(|gpu| {
                let memory_used_mb = gpu.memory_used_mb.unwrap_or(0);
                let memory_total_mb = gpu.memory_total_mb.unwrap_or(0);
                let memory_free_mb = memory_total_mb.saturating_sub(memory_used_mb);
                
                let utilization_percent = if memory_total_mb > 0 {
                    (memory_used_mb as f32 / memory_total_mb as f32) * 100.0
                } else {
                    0.0
                };
                
                GpuMemoryMetrics {
                    device_index: gpu.device_index,
                    device_name: gpu.device_name,
                    memory_used_mb,
                    memory_total_mb,
                    memory_free_mb,
                    utilization_percent,
                }
            })
            .collect()
    }

    /// Collect all memory metrics
    pub fn collect_metrics(&self, gpu_metrics: Vec<super::gpu::GpuMetrics>) -> Result<MemoryMetrics> {
        let system = self.collect_system_metrics()?;
        let gpu_memory = self.collect_gpu_metrics(gpu_metrics);
        
        Ok(MemoryMetrics {
            system,
            gpu_memory,
        })
    }
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::error!("Failed to create memory monitor: {}", e);
            MemoryMonitor {
                system: Arc::new(Mutex::new(System::new_all())),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_monitor_initialization() {
        let monitor = MemoryMonitor::new();
        assert!(monitor.is_ok());
        
        let monitor = monitor.unwrap();
        assert!(monitor.is_available());
    }

    #[test]
    fn test_collect_system_metrics() {
        let monitor = MemoryMonitor::new().unwrap();
        
        let metrics = monitor.collect_system_metrics();
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        assert!(metrics.total_memory_mb > 0);
        assert!(metrics.utilization_percent >= 0.0);
        assert!(metrics.utilization_percent <= 100.0);
    }

    #[test]
    fn test_collect_gpu_metrics() {
        let monitor = MemoryMonitor::new().unwrap();
        
        // Create mock GPU metrics
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
        
        let metrics = monitor.collect_gpu_metrics(gpu_metrics);
        assert_eq!(metrics.len(), 1);
        
        let gpu_memory = &metrics[0];
        assert_eq!(gpu_memory.device_index, 0);
        assert_eq!(gpu_memory.memory_used_mb, 4096);
        assert_eq!(gpu_memory.memory_total_mb, 8192);
        assert_eq!(gpu_memory.memory_free_mb, 4096);
        assert_eq!(gpu_memory.utilization_percent, 50.0);
    }

    #[test]
    fn test_collect_metrics() {
        let monitor = MemoryMonitor::new().unwrap();
        
        let metrics = monitor.collect_metrics(vec![]);
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        assert!(metrics.system.total_memory_mb > 0);
        assert!(metrics.gpu_memory.is_empty());
    }
}
