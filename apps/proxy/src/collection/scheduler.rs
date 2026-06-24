//! Periodic metrics collection scheduler
//! 
//! This module provides a scheduler for periodic collection of system metrics.
//! It supports configurable intervals and handles collection errors gracefully.

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, MissedTickBehavior};
use tracing::{debug, error, info, warn};

use crate::collection::system::SystemMetricsCollector;

/// Metrics collection scheduler
pub struct MetricsScheduler {
    collector: Arc<SystemMetricsCollector>,
    interval_seconds: u64,
    is_running: Arc<RwLock<bool>>,
    last_collection: Arc<RwLock<Option<DateTime<Utc>>>>,
    collection_count: Arc<RwLock<u64>>,
    error_count: Arc<RwLock<u64>>,
}

impl MetricsScheduler {
    /// Create a new metrics scheduler
    pub fn new(collector: Arc<SystemMetricsCollector>, interval_seconds: u64) -> Self {
        info!("Creating metrics scheduler with interval: {}s", interval_seconds);
        
        MetricsScheduler {
            collector,
            interval_seconds,
            is_running: Arc::new(RwLock::new(false)),
            last_collection: Arc::new(RwLock::new(None)),
            collection_count: Arc::new(RwLock::new(0)),
            error_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            warn!("Scheduler is already running");
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        info!("Starting metrics scheduler with {}s interval", self.interval_seconds);

        let collector = Arc::clone(&self.collector);
        let is_running = Arc::clone(&self.is_running);
        let last_collection = Arc::clone(&self.last_collection);
        let collection_count = Arc::clone(&self.collection_count);
        let error_count = Arc::clone(&self.error_count);
        let interval_duration = Duration::from_secs(self.interval_seconds);

        tokio::spawn(async move {
            let mut ticker = interval(interval_duration);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

            info!("Metrics scheduler started");

            loop {
                ticker.tick().await;

                // Check if still running
                {
                    let running = is_running.read().await;
                    if !*running {
                        info!("Metrics scheduler stopped");
                        break;
                    }
                }

                // Collect metrics
                match collector.collect_metrics() {
                    Ok(metrics) => {
                        debug!("Successfully collected metrics at {}", metrics.timestamp);
                        
                        // Update last collection time
                        {
                            let mut last = last_collection.write().await;
                            *last = Some(metrics.timestamp);
                        }
                        
                        // Increment collection count
                        {
                            let mut count = collection_count.write().await;
                            *count += 1;
                        }
                        
                        // Here you would typically store the metrics or send them to a processor
                        // For now, we just log them
                        debug!("Collected system metrics: {} GPUs, {} CPU cores, sample rate: {} Hz",
                               metrics.gpu_utilization.len(),
                               metrics.cpu_utilization.logical_cores,
                               metrics.sample_rate_hz);
                    }
                    Err(e) => {
                        error!("Failed to collect metrics: {}", e);
                        
                        // Increment error count
                        {
                            let mut count = error_count.write().await;
                            *count += 1;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping metrics scheduler");
        
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        
        info!("Metrics scheduler stopped");
        Ok(())
    }

    /// Check if the scheduler is running
    pub async fn is_running(&self) -> bool {
        let is_running = self.is_running.read().await;
        *is_running
    }

    /// Get the last collection time
    pub async fn last_collection(&self) -> Option<DateTime<Utc>> {
        let last = self.last_collection.read().await;
        *last
    }

    /// Get the collection count
    pub async fn collection_count(&self) -> u64 {
        let count = self.collection_count.read().await;
        *count
    }

    /// Get the error count
    pub async fn error_count(&self) -> u64 {
        let count = self.error_count.read().await;
        *count
    }

    /// Get scheduler statistics
    pub async fn statistics(&self) -> SchedulerStatistics {
        let is_running = self.is_running.read().await;
        let last_collection = self.last_collection.read().await;
        let collection_count = self.collection_count.read().await;
        let error_count = self.error_count.read().await;

        SchedulerStatistics {
            is_running: *is_running,
            interval_seconds: self.interval_seconds,
            last_collection: *last_collection,
            collection_count: *collection_count,
            error_count: *error_count,
            success_rate: if *collection_count + *error_count > 0 {
                (*collection_count as f64) / ((*collection_count + *error_count) as f64)
            } else {
                1.0
            },
        }
    }

    /// Update the collection interval
    pub async fn update_interval(&mut self, interval_seconds: u64) -> Result<()> {
        info!("Updating scheduler interval from {}s to {}s", self.interval_seconds, interval_seconds);
        
        // Stop the scheduler if running
        if self.is_running().await {
            self.stop().await?;
        }
        
        self.interval_seconds = interval_seconds;
        
        // Restart if it was running
        // Note: This requires the caller to restart the scheduler
        // We don't auto-restart to give the caller control
        
        Ok(())
    }

    /// Get the current interval
    pub fn interval(&self) -> u64 {
        self.interval_seconds
    }
}

/// Scheduler statistics
#[derive(Debug, Clone)]
pub struct SchedulerStatistics {
    /// Whether the scheduler is running
    pub is_running: bool,
    /// Collection interval in seconds
    pub interval_seconds: u64,
    /// Last collection time
    pub last_collection: Option<DateTime<Utc>>,
    /// Total number of successful collections
    pub collection_count: u64,
    /// Total number of failed collections
    pub error_count: u64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::HardwareMonitor;

    #[tokio::test]
    async fn test_scheduler_initialization() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = Arc::new(SystemMetricsCollector::new(hardware_monitor));
        let scheduler = MetricsScheduler::new(collector, 1);
        
        assert_eq!(scheduler.interval(), 1);
        assert!(!scheduler.is_running().await);
    }

    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = Arc::new(SystemMetricsCollector::new(hardware_monitor));
        let scheduler = MetricsScheduler::new(collector, 1);
        
        // Start scheduler
        scheduler.start().await.unwrap();
        assert!(scheduler.is_running().await);
        
        // Stop scheduler
        scheduler.stop().await.unwrap();
        assert!(!scheduler.is_running().await);
    }

    #[tokio::test]
    async fn test_scheduler_statistics() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = Arc::new(SystemMetricsCollector::new(hardware_monitor));
        let scheduler = MetricsScheduler::new(collector, 1);
        
        let stats = scheduler.statistics().await;
        assert!(!stats.is_running);
        assert_eq!(stats.interval_seconds, 1);
        assert_eq!(stats.collection_count, 0);
        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_scheduler_update_interval() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = Arc::new(SystemMetricsCollector::new(hardware_monitor));
        let mut scheduler = MetricsScheduler::new(collector, 1);
        
        scheduler.update_interval(5).await.unwrap();
        assert_eq!(scheduler.interval(), 5);
    }

    #[tokio::test]
    async fn test_scheduler_collection() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = Arc::new(SystemMetricsCollector::new(hardware_monitor));
        let scheduler = MetricsScheduler::new(collector, 1);
        
        // Start scheduler
        scheduler.start().await.unwrap();
        
        // Wait for at least one collection
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Check that collection occurred
        let stats = scheduler.statistics().await;
        assert!(stats.collection_count > 0 || stats.error_count > 0);
        
        // Stop scheduler
        scheduler.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_scheduler_double_start() {
        let hardware_monitor = Arc::new(HardwareMonitor::new().unwrap());
        let collector = Arc::new(SystemMetricsCollector::new(hardware_monitor));
        let scheduler = MetricsScheduler::new(collector, 1);
        
        // Start scheduler twice
        scheduler.start().await.unwrap();
        scheduler.start().await.unwrap();
        
        assert!(scheduler.is_running().await);
        
        scheduler.stop().await.unwrap();
    }
}