//! CPU metrics collection module using sysinfo
//! 
//! This module provides CPU monitoring capabilities using the sysinfo crate.
//! It collects utilization, memory, temperature, and other CPU-related metrics.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt, CpuExt};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

/// CPU metrics collected from the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// CPU core index
    pub core_index: usize,
    /// CPU core name
    pub core_name: String,
    /// CPU utilization percentage (0-100)
    pub utilization_percent: f32,
    /// CPU frequency in MHz
    pub frequency_mhz: Option<u64>,
    /// CPU vendor ID
    pub vendor_id: Option<String>,
    /// CPU brand
    pub brand: Option<String>,
    /// Whether the CPU is currently under load
    pub is_under_load: bool,
}

/// Aggregate CPU metrics for the entire system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCpuMetrics {
    /// Total CPU utilization percentage (0-100)
    pub total_utilization_percent: f32,
    /// Number of physical CPU cores
    pub physical_cores: usize,
    /// Number of logical CPU cores (threads)
    pub logical_cores: usize,
    /// CPU vendor ID
    pub vendor_id: String,
    /// CPU brand
    pub brand: String,
    /// Individual core metrics
    pub cores: Vec<CpuMetrics>,
    /// System load average (1, 5, 15 minutes)
    pub load_average: Option<(f64, f64, f64)>,
}

/// CPU monitoring service
pub struct CpuMonitor {
    system: Arc<Mutex<System>>,
    physical_cores: usize,
    logical_cores: usize,
}

impl CpuMonitor {
    /// Initialize the CPU monitor
    pub fn new() -> Result<Self> {
        let mut system = System::new_all();
        
        // Get CPU information
        let physical_cores = system.physical_core_count()
            .context("Failed to get physical core count")?;
        
        let logical_cores = system.cpus().len();
        
        info!("CPU monitor initialized: {} physical cores, {} logical cores", 
              physical_cores, logical_cores);
        
        Ok(CpuMonitor {
            system: Arc::new(Mutex::new(system)),
            physical_cores,
            logical_cores,
        })
    }

    /// Check if CPU monitoring is available
    pub fn is_available(&self) -> bool {
        true // sysinfo is always available on supported platforms
    }

    /// Get the number of physical CPU cores
    pub fn physical_cores(&self) -> usize {
        self.physical_cores
    }

    /// Get the number of logical CPU cores
    pub fn logical_cores(&self) -> usize {
        self.logical_cores
    }

    /// Collect metrics from all CPU cores
    pub fn collect_metrics(&self) -> Result<SystemCpuMetrics> {
        let mut system = self.system.lock()
            .context("Failed to acquire system lock")?;
        
        // Refresh CPU information
        system.refresh_cpu();
        
        // Get overall CPU usage
        let total_utilization_percent = system.global_cpu_usage();
        
        // Get CPU vendor and brand
        let vendor_id = system.cpus()
            .first()
            .and_then(|cpu| cpu.vendor_id())
            .unwrap_or("Unknown")
            .to_string();
        
        let brand = system.cpus()
            .first()
            .and_then(|cpu| cpu.brand())
            .unwrap_or("Unknown")
            .to_string();
        
        // Collect individual core metrics
        let mut cores = Vec::new();
        for (index, cpu) in system.cpus().iter().enumerate() {
            let utilization_percent = cpu.cpu_usage();
            let frequency_mhz = cpu.frequency();
            let core_name = format!("Core-{}", index);
            
            // Determine if core is under load (utilization > 10%)
            let is_under_load = utilization_percent > 10.0;
            
            cores.push(CpuMetrics {
                core_index: index,
                core_name,
                utilization_percent,
                frequency_mhz: Some(frequency_mhz),
                vendor_id: Some(cpu.vendor_id().unwrap_or("Unknown").to_string()),
                brand: Some(cpu.brand().unwrap_or("Unknown").to_string()),
                is_under_load,
            });
        }
        
        // Get load average (Unix-like systems only)
        let load_average = System::load_average();
        let load_average = if load_average.one == 0.0 && load_average.five == 0.0 && load_average.fifteen == 0.0 {
            None
        } else {
            Some((load_average.one, load_average.five, load_average.fifteen))
        };
        
        Ok(SystemCpuMetrics {
            total_utilization_percent,
            physical_cores: self.physical_cores,
            logical_cores: self.logical_cores,
            vendor_id,
            brand,
            cores,
            load_average,
        })
    }

    /// Collect metrics from a specific CPU core by index
    pub fn collect_core_metrics(&self, core_index: usize) -> Result<CpuMetrics> {
        let mut system = self.system.lock()
            .context("Failed to acquire system lock")?;
        
        // Refresh CPU information
        system.refresh_cpu();
        
        let cpus = system.cpus();
        
        if core_index >= cpus.len() {
            anyhow::bail!("Core index {} out of range (total cores: {})", 
                         core_index, cpus.len());
        }
        
        let cpu = &cpus[core_index];
        let utilization_percent = cpu.cpu_usage();
        let frequency_mhz = cpu.frequency();
        let core_name = format!("Core-{}", core_index);
        
        // Determine if core is under load (utilization > 10%)
        let is_under_load = utilization_percent > 10.0;
        
        Ok(CpuMetrics {
            core_index,
            core_name,
            utilization_percent,
            frequency_mhz: Some(frequency_mhz),
            vendor_id: Some(cpu.vendor_id().unwrap_or("Unknown").to_string()),
            brand: Some(cpu.brand().unwrap_or("Unknown").to_string()),
            is_under_load,
        })
    }
}

impl Default for CpuMonitor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            error!("Failed to create CPU monitor: {}", e);
            CpuMonitor {
                system: Arc::new(Mutex::new(System::new_all())),
                physical_cores: 1,
                logical_cores: 1,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_monitor_initialization() {
        let monitor = CpuMonitor::new();
        assert!(monitor.is_ok());
        
        let monitor = monitor.unwrap();
        assert!(monitor.is_available());
        assert!(monitor.physical_cores() > 0);
        assert!(monitor.logical_cores() > 0);
    }

    #[test]
    fn test_collect_metrics() {
        let monitor = CpuMonitor::new().unwrap();
        
        let metrics = monitor.collect_metrics();
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        assert!(metrics.physical_cores > 0);
        assert!(metrics.logical_cores > 0);
        assert!(!metrics.vendor_id.is_empty());
        assert!(!metrics.brand.is_empty());
        assert_eq!(metrics.cores.len(), metrics.logical_cores);
        
        // Check that utilization is in valid range
        assert!(metrics.total_utilization_percent >= 0.0);
        assert!(metrics.total_utilization_percent <= 100.0);
    }

    #[test]
    fn test_collect_core_metrics() {
        let monitor = CpuMonitor::new().unwrap();
        
        let metrics = monitor.collect_core_metrics(0);
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.core_index, 0);
        assert!(!metrics.core_name.is_empty());
        
        // Check that utilization is in valid range
        assert!(metrics.utilization_percent >= 0.0);
        assert!(metrics.utilization_percent <= 100.0);
    }

    #[test]
    fn test_invalid_core_index() {
        let monitor = CpuMonitor::new().unwrap();
        
        let metrics = monitor.collect_core_metrics(999);
        assert!(metrics.is_err());
    }
}
