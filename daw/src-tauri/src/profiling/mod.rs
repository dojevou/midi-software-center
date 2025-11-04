//! Performance Profiling Module
//!
//! Comprehensive memory profiling, caching, and rendering optimization.
//!
//! # Modules
//!
//! - [`query_analyzer`]: Database query profiling and optimization
//! - [`memory`]: Memory metrics tracking and allocation pools
//! - [`query_cache`]: Database query result caching (disabled - needs refactoring)
//! - [`render_metrics`]: UI rendering performance tracking
//! - [`commands`]: Tauri command implementations

pub mod query_analyzer;
pub mod memory;
// pub mod query_cache;  // Disabled - has lifetime issues, needs refactoring
pub mod render_metrics;
pub mod commands;

// Re-export query analyzer types
pub use query_analyzer::{
    QueryAnalyzer,
    QueryProfilingReport,
    QueryAnalysis,
    IndexRecommendation,
    CacheRecommendation,
    LoadTestResult,
};

// Re-export memory profiling types
pub use memory::{
    MemoryMetrics, MemoryTracker, AllocationPool, MemoryCache, CacheStats, format_bytes,
};

// Re-export query cache types (disabled)
// pub use query_cache::{QueryCache, QueryKey};

// Re-export rendering metrics types
pub use render_metrics::{RenderMetrics, VirtualScrollMetrics, DomUpdateMetrics};
