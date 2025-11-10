
pub mod memory;
/// Performance Profiling Module
///
/// Comprehensive memory profiling, caching, and rendering optimization.
///
/// # Modules
///
/// - [`query_analyzer`]: Database query profiling and optimization
/// - [`memory`]: Memory metrics tracking and allocation pools
/// - [`query_cache`]: Database query result caching (disabled - needs refactoring)
/// - [`render_metrics`]: UI rendering performance tracking
/// - [`commands`]: Tauri command implementations
pub mod query_analyzer;
// pub mod query_cache;  // Disabled - has lifetime issues, needs refactoring
pub mod commands;
pub mod render_metrics;

// Re-export query analyzer types
pub use query_analyzer::{
    CacheRecommendation, IndexRecommendation, LoadTestResult, QueryAnalysis, QueryAnalyzer,
    QueryProfilingReport,
};

// Re-export memory profiling types
pub use memory::{
    format_bytes, AllocationPool, CacheStats, MemoryCache, MemoryMetrics, MemoryTracker,
};

// Re-export query cache types (disabled)
// pub use query_cache::{QueryCache, QueryKey};

// Re-export rendering metrics types
pub use render_metrics::{DomUpdateMetrics, RenderMetrics, VirtualScrollMetrics};
