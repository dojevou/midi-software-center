/// Tauri commands for memory profiling
///
/// Provides frontend access to memory metrics, cache management,
/// and rendering statistics.
///
/// # Architecture
///
/// This is a **Grown-up Script** - Tauri command wrappers with error handling:
/// - Entry point for profiling operations
/// - State management for memory tracker and caches
/// - Error conversion to String for Tauri
///
/// # Commands
///
/// - `get_memory_metrics` - Get current memory usage
/// - `get_cache_stats` - Get cache statistics
/// - `cache_clear` - Clear all caches
/// - `get_render_metrics` - Get rendering performance
use crate::profiling::{MemoryMetrics, MemoryTracker, RenderMetrics};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Global profiling state
pub struct ProfilingState {
    /// Memory tracker
    pub memory_tracker: MemoryTracker,
    /// Render metrics
    pub render_metrics: Arc<RwLock<RenderMetrics>>,
}

impl ProfilingState {
    /// Create a new profiling state
    pub fn new() -> Self {
        Self {
            memory_tracker: MemoryTracker::new(),
            render_metrics: Arc::new(RwLock::new(RenderMetrics::new())),
        }
    }
}

impl Default for ProfilingState {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current memory metrics
///
/// # Returns
///
/// MemoryMetrics with current memory usage
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
///
/// const metrics = await invoke('get_memory_metrics');
/// console.log(`Memory used: ${metrics.heap_used} bytes`);
/// ```
#[tauri::command]
pub async fn get_memory_metrics(
    state: tauri::State<'_, ProfilingState>,
) -> Result<MemoryMetrics, String> {
    Ok(state.memory_tracker.snapshot())
}

/// Get cache statistics
///
/// # Arguments
///
/// * `cache_name` - Name of cache to query
///
/// # Returns
///
/// Cache statistics
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
///
/// const stats = await invoke('get_cache_stats', { cacheName: 'query' });
/// console.log(`Hit rate: ${stats.hit_rate * 100}%`);
/// ```
#[tauri::command]
pub async fn get_cache_stats(cache_name: String) -> Result<CacheStatsResponse, String> {
    // Note: This is a simplified example. In a real implementation,
    // you would need to maintain cache instances in the state
    Ok(CacheStatsResponse {
        name: cache_name,
        entries: 0,
        size: 0,
        hits: 0,
        misses: 0,
        hit_rate: 0.0,
    })
}

/// Cache statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatsResponse {
    /// Cache name
    pub name: String,
    /// Number of entries
    pub entries: usize,
    /// Total size in bytes
    pub size: usize,
    /// Cache hits
    pub hits: usize,
    /// Cache misses
    pub misses: usize,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f64,
}

/// Clear all caches
///
/// # Returns
///
/// Number of caches cleared
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
///
/// const cleared = await invoke('cache_clear');
/// console.log(`Cleared ${cleared} caches`);
/// ```
#[tauri::command]
pub async fn cache_clear() -> Result<usize, String> {
    // Note: This is a simplified example. In a real implementation,
    // you would clear all cache instances maintained in the state
    Ok(0)
}

/// Get rendering metrics
///
/// # Returns
///
/// Current rendering performance metrics
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
///
/// const metrics = await invoke('get_render_metrics');
/// console.log(`FPS: ${metrics.fps}`);
/// ```
#[tauri::command]
pub async fn get_render_metrics(
    state: tauri::State<'_, ProfilingState>,
) -> Result<RenderMetricsResponse, String> {
    let metrics = state.render_metrics.read();
    Ok(RenderMetricsResponse {
        fps: metrics.fps(),
        average_frame_time: metrics.average_frame_time(),
        min_frame_time: metrics.min_frame_time(),
        max_frame_time: metrics.max_frame_time(),
        total_frames: metrics.total_frames(),
        dropped_frames: metrics.dropped_frames(),
        dropped_frame_rate: metrics.dropped_frame_rate(),
        is_smooth: metrics.is_smooth(),
    })
}

/// Rendering metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMetricsResponse {
    /// Current FPS
    pub fps: f64,
    /// Average frame time (ms)
    pub average_frame_time: f64,
    /// Minimum frame time (ms)
    pub min_frame_time: f64,
    /// Maximum frame time (ms)
    pub max_frame_time: f64,
    /// Total frames rendered
    pub total_frames: usize,
    /// Dropped frames
    pub dropped_frames: usize,
    /// Dropped frame rate (0.0 - 1.0)
    pub dropped_frame_rate: f64,
    /// Is rendering smooth (>= 60 FPS)
    pub is_smooth: bool,
}

/// Record a frame rendering time
///
/// # Arguments
///
/// * `frame_time_ms` - Frame time in milliseconds
///
/// # Returns
///
/// Success message
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
///
/// await invoke('record_frame', { frameTimeMs: 16.7 });
/// ```
#[tauri::command]
pub async fn record_frame(
    frame_time_ms: f64,
    state: tauri::State<'_, ProfilingState>,
) -> Result<String, String> {
    state.render_metrics.write().record_frame(frame_time_ms);
    Ok("Frame recorded".to_string())
}

/// Reset rendering metrics
///
/// # Returns
///
/// Success message
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
///
/// await invoke('reset_render_metrics');
/// ```
#[tauri::command]
pub async fn reset_render_metrics(
    state: tauri::State<'_, ProfilingState>,
) -> Result<String, String> {
    state.render_metrics.write().reset();
    Ok("Render metrics reset".to_string())
}

/// Get memory report
///
/// # Returns
///
/// Human-readable memory report
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
///
/// const report = await invoke('get_memory_report');
/// console.log(report);
/// ```
#[tauri::command]
pub async fn get_memory_report(state: tauri::State<'_, ProfilingState>) -> Result<String, String> {
    Ok(state.memory_tracker.format_report())
}

/// Track allocation
///
/// # Arguments
///
/// * `component` - Component name
/// * `size` - Size in bytes
///
/// # Returns
///
/// Success message
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
///
/// await invoke('track_allocation', { component: 'sequencer', size: 1024 });
/// ```
#[tauri::command]
pub async fn track_allocation(
    component: String,
    size: usize,
    state: tauri::State<'_, ProfilingState>,
) -> Result<String, String> {
    state.memory_tracker.track_allocation(&component, size);
    Ok("Allocation tracked".to_string())
}

/// Track deallocation
///
/// # Arguments
///
/// * `component` - Component name
/// * `size` - Size in bytes
///
/// # Returns
///
/// Success message
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { invoke } from '@tauri-apps/api/core';
///
/// await invoke('track_deallocation', { component: 'sequencer', size: 1024 });
/// ```
#[tauri::command]
pub async fn track_deallocation(
    component: String,
    size: usize,
    state: tauri::State<'_, ProfilingState>,
) -> Result<String, String> {
    state.memory_tracker.track_deallocation(&component, size);
    Ok("Deallocation tracked".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profiling_state_new() {
        let state = ProfilingState::new();
        assert_eq!(state.memory_tracker.total_allocated(), 0);
    }

    #[test]
    fn test_track_allocation_direct() {
        let state = ProfilingState::new();
        state.memory_tracker.track_allocation("sequencer", 1024);
        assert_eq!(state.memory_tracker.total_allocated(), 1024);
    }

    #[test]
    fn test_get_memory_metrics_direct() {
        let state = ProfilingState::new();
        state.memory_tracker.track_allocation("sequencer", 1024);

        let metrics = state.memory_tracker.snapshot();
        assert_eq!(metrics.total_allocated(), 1024);
    }

    #[test]
    fn test_record_frame_direct() {
        let state = ProfilingState::new();
        state.render_metrics.write().record_frame(16.7);

        let render_metrics = state.render_metrics.read();
        assert_eq!(render_metrics.total_frames(), 1);
    }

    #[test]
    fn test_get_render_metrics_direct() {
        let state = ProfilingState::new();
        state.render_metrics.write().record_frame(16.0); // < 16.67ms

        let metrics = state.render_metrics.read();
        assert_eq!(metrics.total_frames(), 1);
        assert!(metrics.is_smooth());
    }
}
