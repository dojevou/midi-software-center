   /// Performance optimization modules
   ///
   /// This module contains pure logic for optimizing file processing performance
   /// based on system resources and workload characteristics.
   ///
   /// # Architecture
   ///
   /// All modules in this package are **Trusty Modules** - pure logic with no I/O.
   ///
   /// # Modules
   ///
   /// - `concurrency`: Dynamic concurrency tuning based on system resources

pub mod concurrency;

// Re-export commonly used items
pub use concurrency::{
    SystemResources,
    detect_system_resources,
    calculate_optimal_concurrency,
    calculate_database_pool_size,
    calculate_batch_size,
    calculate_all_settings,
};
