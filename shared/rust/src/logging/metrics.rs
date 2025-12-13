//! Performance metrics collection and reporting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

lazy_static::lazy_static! {
    static ref METRICS: RwLock<MetricsRegistry> = RwLock::new(MetricsRegistry::new());
}

/// Performance metrics for various operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Operation counts
    pub counters: HashMap<String, u64>,
    /// Operation timing statistics
    pub timings: HashMap<String, TimingStats>,
    /// Gauge values (current state)
    pub gauges: HashMap<String, f64>,
}

/// Timing statistics for an operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingStats {
    pub count: u64,
    pub total_ms: u64,
    pub min_ms: u64,
    pub max_ms: u64,
    pub avg_ms: f64,
}

impl Default for TimingStats {
    fn default() -> Self {
        Self {
            count: 0,
            total_ms: 0,
            min_ms: u64::MAX,
            max_ms: 0,
            avg_ms: 0.0,
        }
    }
}

/// Internal metrics registry.
struct MetricsRegistry {
    counters: HashMap<String, AtomicU64>,
    timings: HashMap<String, RwLock<TimingAccumulator>>,
    gauges: HashMap<String, RwLock<f64>>,
}

struct TimingAccumulator {
    count: u64,
    total_ns: u64,
    min_ns: u64,
    max_ns: u64,
}

impl MetricsRegistry {
    fn new() -> Self {
        Self {
            counters: HashMap::new(),
            timings: HashMap::new(),
            gauges: HashMap::new(),
        }
    }
}

/// Increment a counter metric.
pub fn increment_counter(name: &str) {
    increment_counter_by(name, 1);
}

/// Increment a counter by a specific amount.
pub fn increment_counter_by(name: &str, amount: u64) {
    let registry = METRICS.read().unwrap();
    if let Some(counter) = registry.counters.get(name) {
        counter.fetch_add(amount, Ordering::Relaxed);
    } else {
        drop(registry);
        let mut registry = METRICS.write().unwrap();
        registry
            .counters
            .entry(name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(amount, Ordering::Relaxed);
    }
}

/// Record a timing measurement.
pub fn record_timing(name: &str, duration: Duration) {
    let ns = duration.as_nanos() as u64;
    let registry = METRICS.read().unwrap();

    if let Some(timing) = registry.timings.get(name) {
        let mut acc = timing.write().unwrap();
        acc.count += 1;
        acc.total_ns += ns;
        acc.min_ns = acc.min_ns.min(ns);
        acc.max_ns = acc.max_ns.max(ns);
    } else {
        drop(registry);
        let mut registry = METRICS.write().unwrap();
        registry.timings.entry(name.to_string()).or_insert_with(|| {
            RwLock::new(TimingAccumulator {
                count: 1,
                total_ns: ns,
                min_ns: ns,
                max_ns: ns,
            })
        });
    }
}

/// Set a gauge value.
pub fn set_gauge(name: &str, value: f64) {
    let registry = METRICS.read().unwrap();

    if let Some(gauge) = registry.gauges.get(name) {
        *gauge.write().unwrap() = value;
    } else {
        drop(registry);
        let mut registry = METRICS.write().unwrap();
        registry
            .gauges
            .entry(name.to_string())
            .or_insert_with(|| RwLock::new(0.0));
        if let Some(gauge) = registry.gauges.get(name) {
            *gauge.write().unwrap() = value;
        }
    }
}

/// Get current performance metrics snapshot.
pub fn get_metrics() -> PerformanceMetrics {
    let registry = METRICS.read().unwrap();
    let mut metrics = PerformanceMetrics::default();

    // Counters
    for (name, counter) in &registry.counters {
        metrics
            .counters
            .insert(name.clone(), counter.load(Ordering::Relaxed));
    }

    // Timings
    for (name, timing) in &registry.timings {
        let acc = timing.read().unwrap();
        if acc.count > 0 {
            metrics.timings.insert(
                name.clone(),
                TimingStats {
                    count: acc.count,
                    total_ms: acc.total_ns / 1_000_000,
                    min_ms: acc.min_ns / 1_000_000,
                    max_ms: acc.max_ns / 1_000_000,
                    avg_ms: (acc.total_ns as f64 / acc.count as f64) / 1_000_000.0,
                },
            );
        }
    }

    // Gauges
    for (name, gauge) in &registry.gauges {
        metrics.gauges.insert(name.clone(), *gauge.read().unwrap());
    }

    metrics
}

/// Reset all metrics.
pub fn reset_metrics() {
    let mut registry = METRICS.write().unwrap();
    registry.counters.clear();
    registry.timings.clear();
    registry.gauges.clear();
}

/// Timer guard for automatic timing measurement.
pub struct Timer {
    name: String,
    start: Instant,
}

impl Timer {
    /// Start a new timer.
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        record_timing(&self.name, self.start.elapsed());
    }
}

/// Macro for timing a block of code.
#[macro_export]
macro_rules! time_operation {
    ($name:expr, $block:expr) => {{
        let _timer = $crate::logging::metrics::Timer::start($name);
        $block
    }};
}
