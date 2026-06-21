use std::time::{Instant, Duration};

/// High-precision timing utilities for performance metrics
/// 
/// This module provides microsecond-precision timing utilities for measuring
/// request latency, processing time, and other performance metrics.

/// High-precision timer
#[derive(Debug, Clone)]
pub struct Timer {
    start_time: Option<Instant>,
    elapsed: Duration,
}

impl Timer {
    /// Create a new timer in stopped state
    pub fn new() -> Self {
        Self {
            start_time: None,
            elapsed: Duration::ZERO,
        }
    }
    
    /// Create and start a new timer
    pub fn start() -> Self {
        Self {
            start_time: Some(Instant::now()),
            elapsed: Duration::ZERO,
        }
    }
    
    /// Start the timer
    pub fn start_timer(&mut self) {
        self.start_time = Some(Instant::now());
        self.elapsed = Duration::ZERO;
    }
    
    /// Stop the timer and return elapsed time
    pub fn stop(&mut self) -> Duration {
        if let Some(start) = self.start_time {
            self.elapsed += start.elapsed();
            self.start_time = None;
        }
        self.elapsed
    }
    
    /// Get elapsed time without stopping the timer
    pub fn elapsed(&self) -> Duration {
        if let Some(start) = self.start_time {
            self.elapsed + start.elapsed()
        } else {
            self.elapsed
        }
    }
    
    /// Get elapsed time in microseconds
    pub fn elapsed_micros(&self) -> u128 {
        self.elapsed().as_micros()
    }
    
    /// Get elapsed time in milliseconds
    pub fn elapsed_millis(&self) -> u128 {
        self.elapsed().as_millis()
    }
    
    /// Get elapsed time in seconds (as f64)
    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }
    
    /// Reset the timer
    pub fn reset(&mut self) {
        self.start_time = None;
        self.elapsed = Duration::ZERO;
    }
    
    /// Check if the timer is currently running
    pub fn is_running(&self) -> bool {
        self.start_time.is_some()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// Measure execution time of a function
/// 
/// # Arguments
/// * `f` - Function to measure
/// 
/// # Returns
/// Tuple of (function result, execution time)
pub fn measure<F, R>(f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// Measure execution time of a function and return microseconds
pub fn measure_micros<F, R>(f: F) -> (R, u128)
where
    F: FnOnce() -> R,
{
    let (result, duration) = measure(f);
    (result, duration.as_micros())
}

/// Measure execution time of a function and return milliseconds
pub fn measure_millis<F, R>(f: F) -> (R, u128)
where
    F: FnOnce() -> R,
{
    let (result, duration) = measure(f);
    (result, duration.as_millis())
}

/// Performance statistics for timing measurements
#[derive(Debug, Clone)]
pub struct TimingStats {
    pub count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
}

impl TimingStats {
    pub fn new() -> Self {
        Self {
            count: 0,
            total_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
        }
    }
    
    /// Record a timing measurement
    pub fn record(&mut self, duration: Duration) {
        self.count += 1;
        self.total_duration += duration;
        self.min_duration = self.min_duration.min(duration);
        self.max_duration = self.max_duration.max(duration);
        if self.count > 0 {
            self.avg_duration = self.total_duration / self.count as u32;
        }
    }
    
    /// Get average duration in microseconds
    pub fn avg_micros(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.avg_duration.as_micros() as f64
        }
    }
    
    /// Get average duration in milliseconds
    pub fn avg_millis(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.avg_duration.as_millis() as f64
        }
    }
    
    /// Get percentiles (requires collecting all samples)
    /// This is a simplified version - for production, use proper percentile calculation
    pub fn percentile(&self, _p: f64) -> Duration {
        // Simplified: return average for now
        // In production, you'd collect all samples and calculate proper percentiles
        self.avg_duration
    }
    
    /// Reset statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for TimingStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped timer that automatically stops when dropped
pub struct ScopedTimer {
    timer: Timer,
    on_drop: Option<Box<dyn FnMut(Duration) + Send>>,
}

impl ScopedTimer {
    /// Create a new scoped timer with a callback
    pub fn new<F>(on_drop: F) -> Self
    where
        F: FnMut(Duration) + Send + 'static,
    {
        Self {
            timer: Timer::start(),
            on_drop: Some(Box::new(on_drop)),
        }
    }
    
    /// Get elapsed time without stopping
    pub fn elapsed(&self) -> Duration {
        self.timer.elapsed()
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let elapsed = self.timer.stop();
        if let Some(mut callback) = self.on_drop.take() {
            callback(elapsed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_timer_basic() {
        let mut timer = Timer::new();
        assert!(!timer.is_running());
        
        timer.start_timer();
        assert!(timer.is_running());
        
        thread::sleep(Duration::from_millis(10));
        
        let elapsed = timer.stop();
        assert!(!timer.is_running());
        assert!(elapsed.as_millis() >= 10);
    }
    
    #[test]
    fn test_timer_elapsed() {
        let timer = Timer::start();
        thread::sleep(Duration::from_millis(5));
        
        let elapsed = timer.elapsed();
        assert!(elapsed.as_millis() >= 5);
        assert!(timer.is_running()); // Should still be running
    }
    
    #[test]
    fn test_timer_elapsed_micros() {
        let timer = Timer::start();
        thread::sleep(Duration::from_millis(1));
        
        let micros = timer.elapsed_micros();
        assert!(micros >= 1000);
    }
    
    #[test]
    fn test_timer_reset() {
        let mut timer = Timer::start();
        thread::sleep(Duration::from_millis(5));
        
        timer.reset();
        assert!(!timer.is_running());
        assert_eq!(timer.elapsed(), Duration::ZERO);
    }
    
    #[test]
    fn test_measure() {
        let (result, duration) = measure(|| {
            thread::sleep(Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        assert!(duration.as_millis() >= 10);
    }
    
    #[test]
    fn test_measure_micros() {
        let (result, micros) = measure_micros(|| {
            thread::sleep(Duration::from_millis(1));
            "test"
        });
        
        assert_eq!(result, "test");
        assert!(micros >= 1000);
    }
    
    #[test]
    fn test_timing_stats() {
        let mut stats = TimingStats::new();
        
        stats.record(Duration::from_millis(10));
        stats.record(Duration::from_millis(20));
        stats.record(Duration::from_millis(30));
        
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min_duration, Duration::from_millis(10));
        assert_eq!(stats.max_duration, Duration::from_millis(30));
        assert_eq!(stats.avg_duration, Duration::from_millis(20));
    }
    
    #[test]
    fn test_timing_stats_reset() {
        let mut stats = TimingStats::new();
        stats.record(Duration::from_millis(10));
        
        stats.reset();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.total_duration, Duration::ZERO);
    }
    
    #[test]
    fn test_scoped_timer() {
        use std::sync::{Arc, Mutex};
        
        let captured_duration = Arc::new(Mutex::new(None));
        
        {
            let duration_ref = captured_duration.clone();
            let _timer = ScopedTimer::new(move |duration| {
                *duration_ref.lock().unwrap() = Some(duration);
            });
            thread::sleep(Duration::from_millis(5));
        } // Timer drops here
        
        let duration = captured_duration.lock().unwrap();
        assert!(duration.is_some());
        assert!(duration.unwrap().as_millis() >= 5);
    }
    
    #[test]
    fn test_timer_start_factory() {
        let timer = Timer::start();
        assert!(timer.is_running());
        
        thread::sleep(Duration::from_millis(1));
        assert!(timer.elapsed().as_millis() >= 1);
    }
}
