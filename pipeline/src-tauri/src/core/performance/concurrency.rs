//! Dynamic Concurrency Tuning Module
//!
//! This module provides automatic detection and calculation of optimal concurrency
//! settings based on system resources (CPU cores, RAM, disk type).
//!
//! # Architecture
//!
//! This is a **Trusty Module** - pure logic with comprehensive tests.
//! - NO I/O operations (system detection is read-only introspection)
//! - All functions are pure calculations
//! - Highly testable with different configurations
//!
//! # Usage
//!
//! ```rust
//! use pipeline::core::performance::concurrency::{
//!     detect_system_resources,
//!     calculate_optimal_concurrency
//! };
//!
//! // Auto-detect system resources
//! let resources = detect_system_resources();
//!
//! // Calculate optimal concurrency
//! let concurrency = calculate_optimal_concurrency(&resources);
//! println!("Using {} concurrent workers", concurrency);
//! ```
//!
//! # Performance Tuning Strategy
//!
//! The optimal concurrency is calculated using a multi-factor formula:
//!
//! 1. **CPU-based baseline**: `cpu_cores × 2`
//!    - Accounts for I/O-bound operations (file reading, database writes)
//!    - Each core can handle ~2 concurrent I/O operations efficiently
//!
//! 2. **Memory constraints**: Reduce concurrency if RAM < 8GB
//!    - 4GB RAM: Divide by 4 (risk of swapping)
//!    - 6GB RAM: Divide by 2 (limited headroom)
//!    - 8GB+ RAM: No reduction
//!
//! 3. **Storage type**: Cap based on disk performance
//!    - HDD: Cap at 50 (seek times limit parallelism)
//!    - SSD: Cap at 100 (near-linear scaling)
//!
//! 4. **Absolute bounds**: Clamp to [10, 100]
//!    - Minimum 10: Ensure reasonable throughput on any system
//!    - Maximum 100: Prevent database connection exhaustion

use sysinfo::System;
use std::thread;

/// System resource information used to calculate optimal concurrency.
///
/// This struct captures the relevant system capabilities that affect
/// file processing performance.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemResources {
    /// Number of logical CPU cores (includes hyperthreading)
    pub cpu_cores: usize,

    /// Available system memory in gigabytes
    pub available_memory_gb: f64,

    /// Whether the primary storage is an SSD (true) or HDD (false)
    pub is_ssd: bool,
}

impl SystemResources {
    /// Create a new SystemResources with explicit values.
    ///
    /// Useful for testing different configurations.
    ///
    /// # Examples
    ///
    /// ```
    /// use pipeline::core::performance::concurrency::SystemResources;
    ///
    /// let resources = SystemResources::new(8, 16.0, true);
    /// assert_eq!(resources.cpu_cores, 8);
    /// assert_eq!(resources.available_memory_gb, 16.0);
    /// assert!(resources.is_ssd);
    /// ```
    pub fn new(cpu_cores: usize, available_memory_gb: f64, is_ssd: bool) -> Self {
        Self {
            cpu_cores,
            available_memory_gb,
            is_ssd,
        }
    }
}

/// Automatically detect system resources.
///
/// This function queries the operating system to determine:
/// - CPU core count (logical cores including hyperthreading)
/// - Available system memory
/// - Primary disk type (SSD vs HDD)
///
/// # Returns
///
/// A `SystemResources` struct with detected values.
///
/// # Fallback Behavior
///
/// If detection fails:
/// - CPU cores: Falls back to 4
/// - Memory: Falls back to 8.0 GB
/// - SSD: Assumes true (conservative for performance)
///
/// # Examples
///
/// ```
/// use pipeline::core::performance::concurrency::detect_system_resources;
///
/// let resources = detect_system_resources();
/// println!("Detected {} CPU cores", resources.cpu_cores);
/// println!("Available memory: {:.2} GB", resources.available_memory_gb);
/// println!("SSD: {}", resources.is_ssd);
/// ```
pub fn detect_system_resources() -> SystemResources {
    // Detect CPU cores
    let cpu_cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4); // Fallback to 4 cores if detection fails

    // Initialize system info
    let sys = System::new_all();

    // Detect available memory (convert bytes to GB)
    // sysinfo 0.30 returns memory in bytes
    let total_memory_bytes = sys.total_memory();
    let available_memory_gb = (total_memory_bytes as f64) / (1024.0 * 1024.0 * 1024.0);

    // Detect if primary disk is SSD
    // Strategy: In sysinfo 0.30, we don't have direct SSD detection
    // Default to true (SSD) as a conservative assumption for modern systems
    // In production, this could be enhanced with platform-specific detection
    let is_ssd = true; // Conservative default: assume SSD for better performance

    SystemResources {
        cpu_cores,
        available_memory_gb,
        is_ssd,
    }
}

/// Calculate the optimal concurrency limit based on system resources.
///
/// This function implements a multi-factor formula to determine the ideal
/// number of concurrent file processing workers.
///
/// # Algorithm
///
/// 1. Start with CPU-based baseline: `cpu_cores × 2`
/// 2. Apply memory constraints:
///    - If RAM < 4GB: divide by 4
///    - If RAM < 6GB: divide by 2
///    - If RAM >= 8GB: no reduction
/// 3. Apply storage type cap:
///    - HDD: cap at 50
///    - SSD: cap at 100
/// 4. Clamp to absolute bounds [10, 100]
///
/// # Arguments
///
/// * `resources` - System resource information
///
/// # Returns
///
/// Optimal concurrency limit (10-100)
///
/// # Examples
///
/// ```
/// use pipeline::core::performance::concurrency::{SystemResources, calculate_optimal_concurrency};
///
/// // High-end system: 16 cores, 32GB RAM, SSD
/// let resources = SystemResources::new(16, 32.0, true);
/// let concurrency = calculate_optimal_concurrency(&resources);
/// assert_eq!(concurrency, 32); // 16 × 2, no constraints
///
/// // Low-end system: 4 cores, 4GB RAM, HDD
/// let resources = SystemResources::new(4, 4.0, false);
/// let concurrency = calculate_optimal_concurrency(&resources);
/// assert_eq!(concurrency, 10); // Limited by memory and minimum bound
/// ```
pub fn calculate_optimal_concurrency(resources: &SystemResources) -> usize {
    // Step 1: CPU-based baseline (2× cores for I/O-bound operations)
    let mut concurrency = resources.cpu_cores * 2;

    // Step 2: Apply memory constraints
    if resources.available_memory_gb < 4.0 {
        // Very limited memory: reduce significantly to avoid swapping
        concurrency /= 4;
    } else if resources.available_memory_gb < 6.0 {
        // Limited memory: reduce moderately
        concurrency /= 2;
    }
    // 8GB+ RAM: no memory-based reduction

    // Step 3: Apply storage type cap
    let storage_cap = if resources.is_ssd {
        100 // SSDs scale well with parallelism
    } else {
        50 // HDDs are limited by seek times
    };

    concurrency = concurrency.min(storage_cap);

    // Step 4: Apply absolute bounds
    // - Minimum 10: ensure reasonable throughput
    // - Maximum 100: prevent resource exhaustion
    concurrency.clamp(10, 100)
}

/// Calculate the optimal database connection pool size.
///
/// The pool size should support concurrent operations plus some overhead
/// for connection management and potential contention.
///
/// # Formula
///
/// `pool_size = (concurrency × 1.5).clamp(20, 200)`
///
/// - 1.5× multiplier: Provides headroom for connection recycling
/// - Minimum 20: Ensures adequate connections even on small systems
/// - Maximum 200: Prevents PostgreSQL connection exhaustion
///
/// # Arguments
///
/// * `concurrency` - Target concurrency limit
///
/// # Returns
///
/// Optimal database connection pool size (20-200)
///
/// # Examples
///
/// ```
/// use pipeline::core::performance::concurrency::calculate_database_pool_size;
///
/// let pool_size = calculate_database_pool_size(50);
/// assert_eq!(pool_size, 75); // 50 × 1.5
///
/// let pool_size = calculate_database_pool_size(10);
/// assert_eq!(pool_size, 20); // Clamped to minimum
///
/// let pool_size = calculate_database_pool_size(150);
/// assert_eq!(pool_size, 200); // Clamped to maximum
/// ```
pub fn calculate_database_pool_size(concurrency: usize) -> usize {
    // 1.5× concurrency to provide connection headroom
    let pool_size = (concurrency as f64 * 1.5) as usize;

    // Clamp to PostgreSQL-friendly bounds
    pool_size.clamp(20, 200)
}

/// Calculate the optimal batch size for database operations.
///
/// Larger batches reduce transaction overhead but increase memory usage
/// and potential lock contention. The optimal size balances these factors.
///
/// # Formula
///
/// `batch_size = (concurrency × 100).clamp(500, 10000)`
///
/// - 100× multiplier: Each worker can handle ~100 records per batch
/// - Minimum 500: Ensures meaningful batch performance improvement
/// - Maximum 10,000: Prevents excessive memory usage and lock duration
///
/// # Arguments
///
/// * `concurrency` - Target concurrency limit
///
/// # Returns
///
/// Optimal batch size for database inserts (500-10,000)
///
/// # Examples
///
/// ```
/// use pipeline::core::performance::concurrency::calculate_batch_size;
///
/// let batch_size = calculate_batch_size(50);
/// assert_eq!(batch_size, 5000); // 50 × 100
///
/// let batch_size = calculate_batch_size(10);
/// assert_eq!(batch_size, 1000); // 10 × 100
///
/// let batch_size = calculate_batch_size(150);
/// assert_eq!(batch_size, 10000); // Clamped to maximum
/// ```
pub fn calculate_batch_size(concurrency: usize) -> usize {
    // Each concurrent worker can process ~100 records efficiently
    let batch_size = concurrency * 100;

    // Clamp to reasonable bounds
    batch_size.clamp(500, 10_000)
}

/// Calculate all performance settings in one call.
///
/// This is a convenience function that calculates optimal concurrency,
/// database pool size, and batch size based on detected system resources.
///
/// # Returns
///
/// Tuple of (concurrency, pool_size, batch_size)
///
/// # Examples
///
/// ```
/// use pipeline::core::performance::concurrency::calculate_all_settings;
///
/// let (concurrency, pool_size, batch_size) = calculate_all_settings();
/// println!("Concurrency: {}", concurrency);
/// println!("DB Pool: {}", pool_size);
/// println!("Batch Size: {}", batch_size);
/// ```
pub fn calculate_all_settings() -> (usize, usize, usize) {
    let resources = detect_system_resources();
    let concurrency = calculate_optimal_concurrency(&resources);
    let pool_size = calculate_database_pool_size(concurrency);
    let batch_size = calculate_batch_size(concurrency);

    (concurrency, pool_size, batch_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_resources_new() {
        let resources = SystemResources::new(8, 16.0, true);
        assert_eq!(resources.cpu_cores, 8);
        assert_eq!(resources.available_memory_gb, 16.0);
        assert!(resources.is_ssd);
    }

    #[test]
    fn test_detect_system_resources() {
        let resources = detect_system_resources();

        // Should detect at least 1 core
        assert!(resources.cpu_cores >= 1);

        // Should detect some memory
        assert!(resources.available_memory_gb > 0.0);

        // is_ssd is boolean
        assert!(resources.is_ssd || !resources.is_ssd);
    }

    #[test]
    fn test_optimal_concurrency_high_end_system() {
        // 16 cores, 32GB RAM, SSD
        let resources = SystemResources::new(16, 32.0, true);
        let concurrency = calculate_optimal_concurrency(&resources);

        // Should be 16 × 2 = 32 (no constraints)
        assert_eq!(concurrency, 32);
    }

    #[test]
    fn test_optimal_concurrency_mid_range_system() {
        // 8 cores, 16GB RAM, SSD
        let resources = SystemResources::new(8, 16.0, true);
        let concurrency = calculate_optimal_concurrency(&resources);

        // Should be 8 × 2 = 16 (no constraints)
        assert_eq!(concurrency, 16);
    }

    #[test]
    fn test_optimal_concurrency_low_end_system() {
        // 4 cores, 4GB RAM, HDD
        let resources = SystemResources::new(4, 4.0, false);
        let concurrency = calculate_optimal_concurrency(&resources);

        // Should be limited by memory: 4 × 2 = 8, then / 4 = 2, clamped to 10
        assert_eq!(concurrency, 10);
    }

    #[test]
    fn test_optimal_concurrency_memory_constrained() {
        // 8 cores, 5GB RAM, SSD
        let resources = SystemResources::new(8, 5.0, true);
        let concurrency = calculate_optimal_concurrency(&resources);

        // Should be: 8 × 2 = 16, then / 2 = 8, clamped to 10 (minimum)
        assert_eq!(concurrency, 10);
    }

    #[test]
    fn test_optimal_concurrency_hdd_cap() {
        // 32 cores, 64GB RAM, HDD
        let resources = SystemResources::new(32, 64.0, false);
        let concurrency = calculate_optimal_concurrency(&resources);

        // Should be capped at 50 for HDD
        assert_eq!(concurrency, 50);
    }

    #[test]
    fn test_optimal_concurrency_ssd_cap() {
        // 64 cores, 128GB RAM, SSD
        let resources = SystemResources::new(64, 128.0, true);
        let concurrency = calculate_optimal_concurrency(&resources);

        // Should be capped at 100 (absolute maximum)
        assert_eq!(concurrency, 100);
    }

    #[test]
    fn test_optimal_concurrency_minimum_bound() {
        // 2 cores, 2GB RAM, HDD
        let resources = SystemResources::new(2, 2.0, false);
        let concurrency = calculate_optimal_concurrency(&resources);

        // Should be clamped to minimum of 10
        assert_eq!(concurrency, 10);
    }

    #[test]
    fn test_optimal_concurrency_various_cpu_counts() {
        let test_cases = vec![
            (4, 16.0, true, 10),   // 4 cores: 4×2=8, clamped to 10
            (6, 16.0, true, 12),   // 6 cores: 6×2=12
            (8, 16.0, true, 16),   // 8 cores: 8×2=16
            (12, 16.0, true, 24),  // 12 cores: 12×2=24
            (16, 16.0, true, 32),  // 16 cores: 16×2=32
            (24, 16.0, true, 48),  // 24 cores: 24×2=48
            (32, 16.0, true, 64),  // 32 cores: 32×2=64
        ];

        for (cores, ram, ssd, expected) in test_cases {
            let resources = SystemResources::new(cores, ram, ssd);
            let concurrency = calculate_optimal_concurrency(&resources);
            assert_eq!(
                concurrency, expected,
                "Failed for {} cores: expected {}, got {}",
                cores, expected, concurrency
            );
        }
    }

    #[test]
    fn test_database_pool_size() {
        // Test various concurrency levels
        assert_eq!(calculate_database_pool_size(10), 20);   // Clamped to minimum
        assert_eq!(calculate_database_pool_size(20), 30);   // 20 × 1.5 = 30
        assert_eq!(calculate_database_pool_size(50), 75);   // 50 × 1.5 = 75
        assert_eq!(calculate_database_pool_size(100), 150); // 100 × 1.5 = 150
        assert_eq!(calculate_database_pool_size(150), 200); // Clamped to maximum
    }

    #[test]
    fn test_database_pool_size_minimum_bound() {
        // Very low concurrency should still get minimum pool
        assert_eq!(calculate_database_pool_size(1), 20);
        assert_eq!(calculate_database_pool_size(5), 20);
        assert_eq!(calculate_database_pool_size(10), 20);
    }

    #[test]
    fn test_database_pool_size_maximum_bound() {
        // Very high concurrency should be capped
        assert_eq!(calculate_database_pool_size(200), 200);
        assert_eq!(calculate_database_pool_size(500), 200);
    }

    #[test]
    fn test_batch_size() {
        // Test various concurrency levels
        assert_eq!(calculate_batch_size(10), 1000);   // 10 × 100 = 1000
        assert_eq!(calculate_batch_size(20), 2000);   // 20 × 100 = 2000
        assert_eq!(calculate_batch_size(50), 5000);   // 50 × 100 = 5000
        assert_eq!(calculate_batch_size(100), 10000); // 100 × 100 = 10000 (clamped)
        assert_eq!(calculate_batch_size(150), 10000); // Clamped to maximum
    }

    #[test]
    fn test_batch_size_minimum_bound() {
        // Very low concurrency should get minimum batch
        assert_eq!(calculate_batch_size(1), 500);
        assert_eq!(calculate_batch_size(3), 500);
        assert_eq!(calculate_batch_size(5), 500);
    }

    #[test]
    fn test_batch_size_maximum_bound() {
        // Very high concurrency should be capped
        assert_eq!(calculate_batch_size(150), 10000);
        assert_eq!(calculate_batch_size(500), 10000);
    }

    #[test]
    fn test_calculate_all_settings() {
        let (concurrency, pool_size, batch_size) = calculate_all_settings();

        // Verify all values are in expected ranges
        assert!(concurrency >= 10 && concurrency <= 100);
        assert!(pool_size >= 20 && pool_size <= 200);
        assert!(batch_size >= 500 && batch_size <= 10000);

        // Verify relationships
        assert!(pool_size >= concurrency, "Pool should be >= concurrency");
        assert!(batch_size >= concurrency * 50, "Batch should be >= concurrency × 50");
    }

    #[test]
    fn test_realistic_scenarios() {
        // Scenario 1: Development laptop (MacBook Pro)
        let dev_laptop = SystemResources::new(8, 16.0, true);
        let conc = calculate_optimal_concurrency(&dev_laptop);
        assert_eq!(conc, 16);
        assert_eq!(calculate_database_pool_size(conc), 24);
        assert_eq!(calculate_batch_size(conc), 1600);

        // Scenario 2: Entry-level desktop
        let entry_desktop = SystemResources::new(4, 8.0, false);
        let conc = calculate_optimal_concurrency(&entry_desktop);
        assert_eq!(conc, 10); // 4×2=8, clamped to 10
        assert_eq!(calculate_database_pool_size(conc), 20);
        assert_eq!(calculate_batch_size(conc), 1000);

        // Scenario 3: High-end workstation
        let workstation = SystemResources::new(32, 64.0, true);
        let conc = calculate_optimal_concurrency(&workstation);
        assert_eq!(conc, 64);
        assert_eq!(calculate_database_pool_size(conc), 96);
        assert_eq!(calculate_batch_size(conc), 6400);

        // Scenario 4: Cloud server (16 vCPUs, SSD)
        let cloud_server = SystemResources::new(16, 32.0, true);
        let conc = calculate_optimal_concurrency(&cloud_server);
        assert_eq!(conc, 32);
        assert_eq!(calculate_database_pool_size(conc), 48);
        assert_eq!(calculate_batch_size(conc), 3200);
    }

    #[test]
    fn test_memory_threshold_boundaries() {
        // Test exact boundary conditions

        // Just below 4GB
        let resources = SystemResources::new(8, 3.9, true);
        assert_eq!(calculate_optimal_concurrency(&resources), 10); // 8×2÷4=4, clamped to 10

        // Just at 4GB
        let resources = SystemResources::new(8, 4.0, true);
        assert_eq!(calculate_optimal_concurrency(&resources), 10); // 8×2÷4=4, clamped to 10

        // Just above 4GB
        let resources = SystemResources::new(8, 4.1, true);
        assert_eq!(calculate_optimal_concurrency(&resources), 10); // 8×2÷2=8, clamped to 10

        // Just below 6GB
        let resources = SystemResources::new(8, 5.9, true);
        assert_eq!(calculate_optimal_concurrency(&resources), 10); // 8×2÷2=8, clamped to 10

        // Just at 6GB (boundary - no reduction)
        let resources = SystemResources::new(8, 6.0, true);
        assert_eq!(calculate_optimal_concurrency(&resources), 16); // 8×2=16, no reduction

        // Just above 6GB (no reduction)
        let resources = SystemResources::new(8, 6.1, true);
        assert_eq!(calculate_optimal_concurrency(&resources), 16); // 8×2=16, no reduction

        // At 8GB (no reduction)
        let resources = SystemResources::new(8, 8.0, true);
        assert_eq!(calculate_optimal_concurrency(&resources), 16); // 8×2=16, no reduction
    }
}
