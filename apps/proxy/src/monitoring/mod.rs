//! Hardware monitoring module
//! 
//! This module provides a unified interface for hardware monitoring including
//! GPU, CPU, memory, temperature, and power metrics collection.

pub mod gpu;
pub mod cpu;
pub mod memory;
pub mod temperature;
pub mod power;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub use gpu::{GpuMonitor, GpuMetrics};
pub use cpu::{CpuMonitor, SystemCpuMetrics, CpuMetrics};
pub use memory::{MemoryMonitor, MemoryMetrics, SystemMemoryMetrics, GpuMemoryMetrics};
pub use temperature::{TemperatureMonitor, SystemTemperatureMetrics, TemperatureMetrics, TemperatureComponentType};
pub use power::{PowerMonitor, SystemPowerMetrics, PowerMetrics, PowerComponentType};

/// Combined hardware metrics from all monitoring sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareMetrics {
    /// GPU metrics
    pub gpu: Vec<GpuMetrics>,
    /// CPU metrics
    pub cpu: SystemCpuMetrics,
    /// Memory metrics
    pub memory: MemoryMetrics,
    /// Temperature metrics
    pub temperature: SystemTemperatureMetrics,
    /// Power metrics
    pub power: SystemPowerMetrics,
    /// Timestamp of metrics collection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Hardware monitoring service that aggregates all monitoring modules
pub struct HardwareMonitor {
    gpu_monitor: Arc<GpuMonitor>,
    cpu_monitor: Arc<CpuMonitor>,
    memory_monitor: Arc<MemoryMonitor>,
    temperature_monitor: Arc<TemperatureMonitor>,
    power_monitor: Arc<PowerMonitor>,
}

impl HardwareMonitor {
    /// Initialize the hardware monitor with all monitoring modules
    pub fn new() -> Result<Self> {
        info!("Initializing hardware monitor");
        
        let gpu_monitor = Arc::new(GpuMonitor::new()?);
        let cpu_monitor = Arc::new(CpuMonitor::new()?);
        let memory_monitor = Arc::new(MemoryMonitor::new()?);
        let temperature_monitor = Arc::new(TemperatureMonitor::new());
        let power_monitor = Arc::new(PowerMonitor::new());
        
        info!("Hardware monitor initialized successfully");
        
        Ok(HardwareMonitor {
            gpu_monitor,
            cpu_monitor,
            memory_monitor,
            temperature_monitor,
            power_monitor,
        })
    }

    /// Initialize the hardware monitor with custom temperature and power thresholds
    pub fn with_thresholds(
        cpu_warning_temp: f32,
        cpu_critical_temp: f32,
        gpu_warning_temp: f32,
        gpu_critical_temp: f32,
        gpu_warning_power: f32,
        gpu_critical_power: f32,
        base_system_power: f32,
    ) -> Result<Self> {
        info!("Initializing hardware monitor with custom thresholds");
        
        let gpu_monitor = Arc::new(GpuMonitor::new()?);
        let cpu_monitor = Arc::new(CpuMonitor::new()?);
        let memory_monitor = Arc::new(MemoryMonitor::new()?);
        let temperature_monitor = Arc::new(TemperatureMonitor::with_thresholds(
            cpu_warning_temp,
            cpu_critical_temp,
            gpu_warning_temp,
            gpu_critical_temp,
        ));
        let power_monitor = Arc::new(PowerMonitor::with_thresholds(
            gpu_warning_power,
            gpu_critical_power,
            base_system_power,
        ));
        
        info!("Hardware monitor initialized with custom thresholds");
        
        Ok(HardwareMonitor {
            gpu_monitor,
            cpu_monitor,
            memory_monitor,
            temperature_monitor,
            power_monitor,
        })
    }

    /// Check if hardware monitoring is available
    pub fn is_available(&self) -> bool {
        self.cpu_monitor.is_available() && self.memory_monitor.is_available()
    }

    /// Check if GPU monitoring is available
    pub fn is_gpu_available(&self) -> bool {
        self.gpu_monitor.is_available()
    }

    /// Get the number of GPU devices
    pub fn gpu_device_count(&self) -> u32 {
        self.gpu_monitor.device_count()
    }

    /// Get the number of CPU cores
    pub fn cpu_physical_cores(&self) -> usize {
        self.cpu_monitor.physical_cores()
    }

    /// Get the number of logical CPU cores
    pub fn cpu_logical_cores(&self) -> usize {
        self.cpu_monitor.logical_cores()
    }

    /// Collect all hardware metrics
    pub fn collect_metrics(&self) -> Result<HardwareMetrics> {
        debug!("Collecting hardware metrics");
        
        // Collect GPU metrics
        let gpu = self.gpu_monitor.collect_metrics()
            .unwrap_or_else(|e| {
                warn!("Failed to collect GPU metrics: {}", e);
                vec![]
            });
        
        // Collect CPU metrics
        let cpu = self.cpu_monitor.collect_metrics()
            .context("Failed to collect CPU metrics")?;
        
        // Collect memory metrics
        let memory = self.memory_monitor.collect_metrics(gpu.clone())
            .context("Failed to collect memory metrics")?;
        
        // Collect temperature metrics
        let temperature = self.temperature_monitor.collect_metrics(gpu.clone(), &cpu);
        
        // Collect power metrics
        let power = self.power_monitor.collect_metrics(gpu.clone(), &cpu);
        
        let timestamp = chrono::Utc::now();
        
        debug!("Hardware metrics collection completed");
        
        Ok(HardwareMetrics {
            gpu,
            cpu,
            memory,
            temperature,
            power,
            timestamp,
        })
    }

    /// Collect only GPU metrics
    pub fn collect_gpu_metrics(&self) -> Result<Vec<GpuMetrics>> {
        self.gpu_monitor.collect_metrics()
    }

    /// Collect only CPU metrics
    pub fn collect_cpu_metrics(&self) -> Result<SystemCpuMetrics> {
        self.cpu_monitor.collect_metrics()
    }

    /// Collect only memory metrics
    pub fn collect_memory_metrics(&self) -> Result<MemoryMetrics> {
        let gpu = self.gpu_monitor.collect_metrics()
            .unwrap_or_else(|e| {
                warn!("Failed to collect GPU metrics for memory: {}", e);
                vec![]
            });
        self.memory_monitor.collect_metrics(gpu)
    }

    /// Collect only temperature metrics
    pub fn collect_temperature_metrics(&self) -> Result<SystemTemperatureMetrics> {
        let gpu = self.gpu_monitor.collect_metrics()
            .unwrap_or_else(|e| {
                warn!("Failed to collect GPU metrics for temperature: {}", e);
                vec![]
            });
        let cpu = self.cpu_monitor.collect_metrics()?;
        Ok(self.temperature_monitor.collect_metrics(gpu, &cpu))
    }

    /// Collect only power metrics
    pub fn collect_power_metrics(&self) -> Result<SystemPowerMetrics> {
        let gpu = self.gpu_monitor.collect_metrics()
            .unwrap_or_else(|e| {
                warn!("Failed to collect GPU metrics for power: {}", e);
                vec![]
            });
        let cpu = self.cpu_monitor.collect_metrics()?;
        Ok(self.power_monitor.collect_metrics(gpu, &cpu))
    }

    /// Get GPU monitor reference
    pub fn gpu_monitor(&self) -> &GpuMonitor {
        &self.gpu_monitor
    }

    /// Get CPU monitor reference
    pub fn cpu_monitor(&self) -> &CpuMonitor {
        &self.cpu_monitor
    }

    /// Get memory monitor reference
    pub fn memory_monitor(&self) -> &MemoryMonitor {
        &self.memory_monitor
    }

    /// Get temperature monitor reference
    pub fn temperature_monitor(&self) -> &TemperatureMonitor {
        &self.temperature_monitor
    }

    /// Get power monitor reference
    pub fn power_monitor(&self) -> &PowerMonitor {
        &self.power_monitor
    }
}

impl Default for HardwareMonitor {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            error!("Failed to create hardware monitor: {}", e);
            // Create a degraded monitor with basic functionality
            HardwareMonitor {
                gpu_monitor: Arc::new(GpuMonitor::default()),
                cpu_monitor: Arc::new(CpuMonitor::default()),
                memory_monitor: Arc::new(MemoryMonitor::default()),
                temperature_monitor: Arc::new(TemperatureMonitor::default()),
                power_monitor: Arc::new(PowerMonitor::default()),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_monitor_initialization() {
        let monitor = HardwareMonitor::new();
        assert!(monitor.is_ok());
        
        let monitor = monitor.unwrap();
        assert!(monitor.is_available());
    }

    #[test]
    fn test_collect_metrics() {
        let monitor = HardwareMonitor::new().unwrap();
        
        let metrics = monitor.collect_metrics();
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        assert!(!metrics.cpu.cores.is_empty());
        assert!(metrics.memory.system.total_memory_mb > 0);
        assert!(metrics.timestamp > chrono::Utc::now() - chrono::Duration::seconds(10));
    }

    #[test]
    fn test_collect_gpu_metrics() {
        let monitor = HardwareMonitor::new().unwrap();
        
        let metrics = monitor.collect_gpu_metrics();
        // May fail if no GPU available, but shouldn't panic
        let _ = metrics;
    }

    #[test]
    fn test_collect_cpu_metrics() {
        let monitor = HardwareMonitor::new().unwrap();
        
        let metrics = monitor.collect_cpu_metrics();
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        assert!(metrics.physical_cores > 0);
        assert!(metrics.logical_cores > 0);
    }

    #[test]
    fn test_collect_memory_metrics() {
        let monitor = HardwareMonitor::new().unwrap();
        
        let metrics = monitor.collect_memory_metrics();
        assert!(metrics.is_ok());
        
        let metrics = metrics.unwrap();
        assert!(metrics.system.total_memory_mb > 0);
    }

    #[test]
    fn test_with_thresholds() {
        let monitor = HardwareMonitor::with_thresholds(
            80.0, 90.0,  // CPU temps
            85.0, 95.0,  // GPU temps
            250.0, 350.0, // GPU power
            100.0,       // Base system power
        );
        assert!(monitor.is_ok());
        
        let monitor = monitor.unwrap();
        assert!(monitor.is_available());
    }
}
