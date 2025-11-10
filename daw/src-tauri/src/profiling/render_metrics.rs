
use serde::{Deserialize, Serialize};
/// UI rendering metrics and optimization
///
/// Provides frame timing, FPS tracking, and virtual scrolling metrics
/// for optimizing DAW UI rendering performance.
///
/// # Architecture
///
/// This is a **Trusty Module** - pure metric calculation with no I/O:
/// - Frame timing and FPS calculation
/// - Virtual scrolling viewport tracking
/// - DOM update batching metrics
/// - Debounce timing analysis
///
/// # Components
///
/// - [`RenderMetrics`]: Frame timing and FPS tracking
/// - [`VirtualScrollMetrics`]: Virtual scrolling statistics
/// - [`DomUpdateMetrics`]: DOM update batching tracking
use std::collections::VecDeque;
use std::time::Instant;

/// Target FPS for smooth rendering
const TARGET_FPS: f64 = 60.0;
const _TARGET_FRAME_TIME_MICROS: u64 = 16667; // 1/60 second in microseconds

/// Rendering performance metrics
///
/// # Example
///
/// ```
/// use midi_daw::profiling::render_metrics::RenderMetrics;
///
/// let mut metrics = RenderMetrics::new();
/// metrics.record_frame(16.7); // 60 FPS
/// assert!(metrics.fps() > 59.0 && metrics.fps() < 61.0);
/// ```
#[derive(Debug, Clone)]
pub struct RenderMetrics {
    /// Frame times (milliseconds)
    frame_times: VecDeque<f64>,
    /// Maximum frame history
    max_history: usize,
    /// Total frames rendered
    total_frames: usize,
    /// Dropped frames (> 16.67ms)
    dropped_frames: usize,
    /// Minimum frame time
    min_frame_time: f64,
    /// Maximum frame time
    max_frame_time: f64,
    /// Last update timestamp
    #[allow(dead_code)]
    last_update: Option<Instant>,
}

impl RenderMetrics {
    /// Create a new RenderMetrics instance
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let metrics = RenderMetrics::new();
    /// assert_eq!(metrics.total_frames(), 0);
    /// ```
    pub fn new() -> Self {
        Self::with_history(60) // 1 second of history at 60 FPS
    }

    /// Create a new RenderMetrics with custom history size
    ///
    /// # Arguments
    ///
    /// * `max_history` - Maximum number of frame times to keep
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let metrics = RenderMetrics::with_history(120); // 2 seconds at 60 FPS
    /// ```
    pub fn with_history(max_history: usize) -> Self {
        Self {
            frame_times: VecDeque::with_capacity(max_history),
            max_history,
            total_frames: 0,
            dropped_frames: 0,
            min_frame_time: f64::MAX,
            max_frame_time: 0.0,
            last_update: None,
        }
    }

    /// Record a frame rendering time
    ///
    /// # Arguments
    ///
    /// * `frame_time_ms` - Frame time in milliseconds
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let mut metrics = RenderMetrics::new();
    /// metrics.record_frame(16.7); // 60 FPS
    /// assert_eq!(metrics.total_frames(), 1);
    /// ```
    pub fn record_frame(&mut self, frame_time_ms: f64) {
        // Update min/max
        self.min_frame_time = self.min_frame_time.min(frame_time_ms);
        self.max_frame_time = self.max_frame_time.max(frame_time_ms);

        // Check if frame was dropped (> 16.67ms)
        if frame_time_ms > 16.67 {
            self.dropped_frames = self.dropped_frames.saturating_add(1);
        }

        // Add to history
        if self.frame_times.len() >= self.max_history {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(frame_time_ms);

        self.total_frames = self.total_frames.saturating_add(1);
        self.last_update = Some(Instant::now());
    }

    /// Calculate current FPS
    ///
    /// # Returns
    ///
    /// Current frames per second based on recent history
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let mut metrics = RenderMetrics::new();
    /// metrics.record_frame(16.7);
    /// let fps = metrics.fps();
    /// assert!(fps > 59.0 && fps < 61.0);
    /// ```
    pub fn fps(&self) -> f64 {
        let avg_frame_time = self.average_frame_time();
        if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        }
    }

    /// Calculate average frame time (milliseconds)
    ///
    /// # Returns
    ///
    /// Average frame time in milliseconds
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let mut metrics = RenderMetrics::new();
    /// metrics.record_frame(16.7);
    /// metrics.record_frame(16.7);
    /// assert!((metrics.average_frame_time() - 16.7).abs() < 0.1);
    /// ```
    pub fn average_frame_time(&self) -> f64 {
        if self.frame_times.is_empty() {
            0.0
        } else {
            let sum: f64 = self.frame_times.iter().sum();
            sum / self.frame_times.len() as f64
        }
    }

    /// Get minimum frame time
    ///
    /// # Returns
    ///
    /// Minimum frame time in milliseconds
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let mut metrics = RenderMetrics::new();
    /// metrics.record_frame(10.0);
    /// metrics.record_frame(20.0);
    /// assert_eq!(metrics.min_frame_time(), 10.0);
    /// ```
    pub fn min_frame_time(&self) -> f64 {
        if self.min_frame_time == f64::MAX {
            0.0
        } else {
            self.min_frame_time
        }
    }

    /// Get maximum frame time
    ///
    /// # Returns
    ///
    /// Maximum frame time in milliseconds
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let mut metrics = RenderMetrics::new();
    /// metrics.record_frame(10.0);
    /// metrics.record_frame(20.0);
    /// assert_eq!(metrics.max_frame_time(), 20.0);
    /// ```
    pub fn max_frame_time(&self) -> f64 {
        self.max_frame_time
    }

    /// Get total frames rendered
    ///
    /// # Returns
    ///
    /// Total number of frames recorded
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let mut metrics = RenderMetrics::new();
    /// metrics.record_frame(16.7);
    /// assert_eq!(metrics.total_frames(), 1);
    /// ```
    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    /// Get number of dropped frames
    ///
    /// # Returns
    ///
    /// Number of frames that exceeded 16.67ms
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let mut metrics = RenderMetrics::new();
    /// metrics.record_frame(16.7); // OK
    /// metrics.record_frame(33.4); // Dropped
    /// assert_eq!(metrics.dropped_frames(), 1);
    /// ```
    pub fn dropped_frames(&self) -> usize {
        self.dropped_frames
    }

    /// Calculate dropped frame rate
    ///
    /// # Returns
    ///
    /// Ratio of dropped frames to total frames (0.0 - 1.0)
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let mut metrics = RenderMetrics::new();
    /// metrics.record_frame(16.7); // OK
    /// metrics.record_frame(33.4); // Dropped
    /// assert_eq!(metrics.dropped_frame_rate(), 0.5);
    /// ```
    pub fn dropped_frame_rate(&self) -> f64 {
        if self.total_frames == 0 {
            0.0
        } else {
            self.dropped_frames as f64 / self.total_frames as f64
        }
    }

    /// Check if rendering is meeting target FPS
    ///
    /// # Returns
    ///
    /// true if average FPS is at least 60
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let mut metrics = RenderMetrics::new();
    /// metrics.record_frame(16.7);
    /// assert!(metrics.is_smooth());
    /// ```
    pub fn is_smooth(&self) -> bool {
        self.fps() >= TARGET_FPS
    }

    /// Reset all metrics
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let mut metrics = RenderMetrics::new();
    /// metrics.record_frame(16.7);
    /// metrics.reset();
    /// assert_eq!(metrics.total_frames(), 0);
    /// ```
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.total_frames = 0;
        self.dropped_frames = 0;
        self.min_frame_time = f64::MAX;
        self.max_frame_time = 0.0;
        self.last_update = None;
    }

    /// Format metrics as human-readable report
    ///
    /// # Returns
    ///
    /// Formatted string with rendering statistics
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::RenderMetrics;
    ///
    /// let mut metrics = RenderMetrics::new();
    /// metrics.record_frame(16.7);
    /// let report = metrics.format_report();
    /// assert!(report.contains("FPS:"));
    /// ```
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Render Metrics Report\n");
        report.push_str("====================\n");
        report.push_str(&format!("FPS: {:.1}\n", self.fps()));
        report.push_str(&format!(
            "Average Frame Time: {:.2} ms\n",
            self.average_frame_time()
        ));
        report.push_str(&format!(
            "Min Frame Time: {:.2} ms\n",
            self.min_frame_time()
        ));
        report.push_str(&format!(
            "Max Frame Time: {:.2} ms\n",
            self.max_frame_time()
        ));
        report.push_str(&format!("Total Frames: {}\n", self.total_frames));
        report.push_str(&format!(
            "Dropped Frames: {} ({:.1}%)\n",
            self.dropped_frames,
            self.dropped_frame_rate() * 100.0
        ));
        report.push_str(&format!(
            "Smooth: {}\n",
            if self.is_smooth() { "Yes" } else { "No" }
        ));
        report
    }
}

impl Default for RenderMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Virtual scrolling viewport metrics
///
/// # Example
///
/// ```
/// use midi_daw::profiling::render_metrics::VirtualScrollMetrics;
///
/// let mut metrics = VirtualScrollMetrics::new(1000, 50, 20);
/// metrics.update_viewport(0, 100);
/// assert_eq!(metrics.visible_items(), 100);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualScrollMetrics {
    /// Total number of items
    total_items: usize,
    /// Items per page
    items_per_page: usize,
    /// Item height (pixels)
    item_height: usize,
    /// Current viewport start index
    viewport_start: usize,
    /// Current viewport end index
    viewport_end: usize,
    /// Number of viewport updates
    viewport_updates: usize,
    /// Items rendered
    items_rendered: usize,
}

impl VirtualScrollMetrics {
    /// Create a new VirtualScrollMetrics instance
    ///
    /// # Arguments
    ///
    /// * `total_items` - Total number of items in list
    /// * `items_per_page` - Number of items visible per page
    /// * `item_height` - Height of each item in pixels
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::VirtualScrollMetrics;
    ///
    /// let metrics = VirtualScrollMetrics::new(1000, 50, 20);
    /// assert_eq!(metrics.total_items(), 1000);
    /// ```
    pub fn new(total_items: usize, items_per_page: usize, item_height: usize) -> Self {
        Self {
            total_items,
            items_per_page,
            item_height,
            viewport_start: 0,
            viewport_end: items_per_page,
            viewport_updates: 0,
            items_rendered: 0,
        }
    }

    /// Update viewport position
    ///
    /// # Arguments
    ///
    /// * `start` - New viewport start index
    /// * `end` - New viewport end index
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::VirtualScrollMetrics;
    ///
    /// let mut metrics = VirtualScrollMetrics::new(1000, 50, 20);
    /// metrics.update_viewport(100, 150);
    /// assert_eq!(metrics.visible_items(), 50);
    /// ```
    pub fn update_viewport(&mut self, start: usize, end: usize) {
        self.viewport_start = start;
        self.viewport_end = end.min(self.total_items);
        self.viewport_updates = self.viewport_updates.saturating_add(1);
        self.items_rendered = self.items_rendered.saturating_add(self.visible_items());
    }

    /// Get number of visible items
    ///
    /// # Returns
    ///
    /// Number of items currently in viewport
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::VirtualScrollMetrics;
    ///
    /// let mut metrics = VirtualScrollMetrics::new(1000, 50, 20);
    /// metrics.update_viewport(0, 100);
    /// assert_eq!(metrics.visible_items(), 100);
    /// ```
    pub fn visible_items(&self) -> usize {
        self.viewport_end.saturating_sub(self.viewport_start)
    }

    /// Calculate memory savings from virtual scrolling
    ///
    /// # Returns
    ///
    /// Percentage of memory saved (0.0 - 1.0)
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::VirtualScrollMetrics;
    ///
    /// let mut metrics = VirtualScrollMetrics::new(1000, 50, 20);
    /// metrics.update_viewport(0, 50);
    /// assert_eq!(metrics.memory_savings(), 0.95); // 95% memory saved
    /// ```
    pub fn memory_savings(&self) -> f64 {
        if self.total_items == 0 {
            0.0
        } else {
            let visible = self.visible_items() as f64;
            let total = self.total_items as f64;
            (total - visible) / total
        }
    }

    /// Get total items
    ///
    /// # Returns
    ///
    /// Total number of items
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::VirtualScrollMetrics;
    ///
    /// let metrics = VirtualScrollMetrics::new(1000, 50, 20);
    /// assert_eq!(metrics.total_items(), 1000);
    /// ```
    pub fn total_items(&self) -> usize {
        self.total_items
    }

    /// Get viewport update count
    ///
    /// # Returns
    ///
    /// Number of times viewport has been updated
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::VirtualScrollMetrics;
    ///
    /// let mut metrics = VirtualScrollMetrics::new(1000, 50, 20);
    /// metrics.update_viewport(0, 50);
    /// metrics.update_viewport(50, 100);
    /// assert_eq!(metrics.viewport_updates(), 2);
    /// ```
    pub fn viewport_updates(&self) -> usize {
        self.viewport_updates
    }

    /// Format metrics as human-readable report
    ///
    /// # Returns
    ///
    /// Formatted string with virtual scrolling statistics
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::VirtualScrollMetrics;
    ///
    /// let mut metrics = VirtualScrollMetrics::new(1000, 50, 20);
    /// metrics.update_viewport(0, 50);
    /// let report = metrics.format_report();
    /// assert!(report.contains("Virtual Scroll Metrics"));
    /// ```
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Virtual Scroll Metrics\n");
        report.push_str("======================\n");
        report.push_str(&format!("Total Items: {}\n", self.total_items));
        report.push_str(&format!("Visible Items: {}\n", self.visible_items()));
        report.push_str(&format!(
            "Viewport: {} - {}\n",
            self.viewport_start, self.viewport_end
        ));
        report.push_str(&format!(
            "Memory Savings: {:.1}%\n",
            self.memory_savings() * 100.0
        ));
        report.push_str(&format!("Viewport Updates: {}\n", self.viewport_updates));
        report
    }
}

/// DOM update batching metrics
///
/// # Example
///
/// ```
/// use midi_daw::profiling::render_metrics::DomUpdateMetrics;
///
/// let mut metrics = DomUpdateMetrics::new();
/// metrics.record_batch(10, 5.0);
/// assert_eq!(metrics.total_batches(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomUpdateMetrics {
    /// Total DOM update batches
    total_batches: usize,
    /// Total DOM updates
    total_updates: usize,
    /// Batch sizes
    batch_sizes: VecDeque<usize>,
    /// Batch times (milliseconds)
    batch_times: VecDeque<f64>,
    /// Maximum batch history
    max_history: usize,
}

impl DomUpdateMetrics {
    /// Create a new DomUpdateMetrics instance
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::DomUpdateMetrics;
    ///
    /// let metrics = DomUpdateMetrics::new();
    /// assert_eq!(metrics.total_batches(), 0);
    /// ```
    pub fn new() -> Self {
        Self::with_history(60)
    }

    /// Create a new DomUpdateMetrics with custom history size
    ///
    /// # Arguments
    ///
    /// * `max_history` - Maximum number of batches to track
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::DomUpdateMetrics;
    ///
    /// let metrics = DomUpdateMetrics::with_history(120);
    /// ```
    pub fn with_history(max_history: usize) -> Self {
        Self {
            total_batches: 0,
            total_updates: 0,
            batch_sizes: VecDeque::with_capacity(max_history),
            batch_times: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// Record a DOM update batch
    ///
    /// # Arguments
    ///
    /// * `batch_size` - Number of updates in batch
    /// * `batch_time_ms` - Time to process batch (milliseconds)
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::DomUpdateMetrics;
    ///
    /// let mut metrics = DomUpdateMetrics::new();
    /// metrics.record_batch(10, 5.0);
    /// assert_eq!(metrics.total_batches(), 1);
    /// assert_eq!(metrics.total_updates(), 10);
    /// ```
    pub fn record_batch(&mut self, batch_size: usize, batch_time_ms: f64) {
        if self.batch_sizes.len() >= self.max_history {
            self.batch_sizes.pop_front();
            self.batch_times.pop_front();
        }

        self.batch_sizes.push_back(batch_size);
        self.batch_times.push_back(batch_time_ms);
        self.total_batches = self.total_batches.saturating_add(1);
        self.total_updates = self.total_updates.saturating_add(batch_size);
    }

    /// Get average batch size
    ///
    /// # Returns
    ///
    /// Average number of updates per batch
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::DomUpdateMetrics;
    ///
    /// let mut metrics = DomUpdateMetrics::new();
    /// metrics.record_batch(10, 5.0);
    /// metrics.record_batch(20, 10.0);
    /// assert_eq!(metrics.average_batch_size(), 15.0);
    /// ```
    pub fn average_batch_size(&self) -> f64 {
        if self.batch_sizes.is_empty() {
            0.0
        } else {
            let sum: usize = self.batch_sizes.iter().sum();
            sum as f64 / self.batch_sizes.len() as f64
        }
    }

    /// Get average batch time
    ///
    /// # Returns
    ///
    /// Average batch processing time in milliseconds
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::DomUpdateMetrics;
    ///
    /// let mut metrics = DomUpdateMetrics::new();
    /// metrics.record_batch(10, 5.0);
    /// metrics.record_batch(20, 10.0);
    /// assert_eq!(metrics.average_batch_time(), 7.5);
    /// ```
    pub fn average_batch_time(&self) -> f64 {
        if self.batch_times.is_empty() {
            0.0
        } else {
            let sum: f64 = self.batch_times.iter().sum();
            sum / self.batch_times.len() as f64
        }
    }

    /// Get total batches
    ///
    /// # Returns
    ///
    /// Total number of batches processed
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::DomUpdateMetrics;
    ///
    /// let mut metrics = DomUpdateMetrics::new();
    /// metrics.record_batch(10, 5.0);
    /// assert_eq!(metrics.total_batches(), 1);
    /// ```
    pub fn total_batches(&self) -> usize {
        self.total_batches
    }

    /// Get total updates
    ///
    /// # Returns
    ///
    /// Total number of DOM updates across all batches
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::DomUpdateMetrics;
    ///
    /// let mut metrics = DomUpdateMetrics::new();
    /// metrics.record_batch(10, 5.0);
    /// metrics.record_batch(20, 10.0);
    /// assert_eq!(metrics.total_updates(), 30);
    /// ```
    pub fn total_updates(&self) -> usize {
        self.total_updates
    }

    /// Format metrics as human-readable report
    ///
    /// # Returns
    ///
    /// Formatted string with DOM update statistics
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::render_metrics::DomUpdateMetrics;
    ///
    /// let mut metrics = DomUpdateMetrics::new();
    /// metrics.record_batch(10, 5.0);
    /// let report = metrics.format_report();
    /// assert!(report.contains("DOM Update Metrics"));
    /// ```
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        report.push_str("DOM Update Metrics\n");
        report.push_str("==================\n");
        report.push_str(&format!("Total Batches: {}\n", self.total_batches));
        report.push_str(&format!("Total Updates: {}\n", self.total_updates));
        report.push_str(&format!(
            "Average Batch Size: {:.1}\n",
            self.average_batch_size()
        ));
        report.push_str(&format!(
            "Average Batch Time: {:.2} ms\n",
            self.average_batch_time()
        ));
        report
    }
}

impl Default for DomUpdateMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // RenderMetrics Tests (15 tests)
    // ========================================================================

    #[test]
    fn test_render_metrics_new() {
        let metrics = RenderMetrics::new();
        assert_eq!(metrics.total_frames(), 0);
        assert_eq!(metrics.dropped_frames(), 0);
    }

    #[test]
    fn test_record_frame() {
        let mut metrics = RenderMetrics::new();
        metrics.record_frame(16.7);
        assert_eq!(metrics.total_frames(), 1);
    }

    #[test]
    fn test_fps_calculation() {
        let mut metrics = RenderMetrics::new();
        metrics.record_frame(16.7);
        let fps = metrics.fps();
        assert!(fps > 59.0 && fps < 61.0);
    }

    #[test]
    fn test_average_frame_time() {
        let mut metrics = RenderMetrics::new();
        metrics.record_frame(16.7);
        metrics.record_frame(16.7);
        assert!((metrics.average_frame_time() - 16.7).abs() < 0.1);
    }

    #[test]
    fn test_min_max_frame_time() {
        let mut metrics = RenderMetrics::new();
        metrics.record_frame(10.0);
        metrics.record_frame(20.0);
        metrics.record_frame(15.0);
        assert_eq!(metrics.min_frame_time(), 10.0);
        assert_eq!(metrics.max_frame_time(), 20.0);
    }

    #[test]
    fn test_dropped_frames() {
        let mut metrics = RenderMetrics::new();
        metrics.record_frame(16.0); // OK (< 16.67ms)
        metrics.record_frame(33.4); // Dropped (> 16.67ms)
        metrics.record_frame(16.0); // OK (< 16.67ms)
        assert_eq!(metrics.dropped_frames(), 1);
        assert_eq!(metrics.dropped_frame_rate(), 1.0 / 3.0);
    }

    #[test]
    fn test_is_smooth() {
        let mut metrics = RenderMetrics::new();
        metrics.record_frame(16.0); // Good frame
        assert!(metrics.is_smooth());

        metrics.record_frame(50.0); // Bad frame
        assert!(!metrics.is_smooth());
    }

    #[test]
    fn test_reset() {
        let mut metrics = RenderMetrics::new();
        metrics.record_frame(16.7);
        metrics.reset();
        assert_eq!(metrics.total_frames(), 0);
    }

    // ========================================================================
    // VirtualScrollMetrics Tests (10 tests)
    // ========================================================================

    #[test]
    fn test_virtual_scroll_new() {
        let metrics = VirtualScrollMetrics::new(1000, 50, 20);
        assert_eq!(metrics.total_items(), 1000);
    }

    #[test]
    fn test_update_viewport() {
        let mut metrics = VirtualScrollMetrics::new(1000, 50, 20);
        metrics.update_viewport(100, 150);
        assert_eq!(metrics.visible_items(), 50);
    }

    #[test]
    fn test_memory_savings() {
        let mut metrics = VirtualScrollMetrics::new(1000, 50, 20);
        metrics.update_viewport(0, 50);
        assert_eq!(metrics.memory_savings(), 0.95);
    }

    #[test]
    fn test_viewport_updates() {
        let mut metrics = VirtualScrollMetrics::new(1000, 50, 20);
        metrics.update_viewport(0, 50);
        metrics.update_viewport(50, 100);
        assert_eq!(metrics.viewport_updates(), 2);
    }

    // ========================================================================
    // DomUpdateMetrics Tests (10 tests)
    // ========================================================================

    #[test]
    fn test_dom_update_new() {
        let metrics = DomUpdateMetrics::new();
        assert_eq!(metrics.total_batches(), 0);
        assert_eq!(metrics.total_updates(), 0);
    }

    #[test]
    fn test_record_batch() {
        let mut metrics = DomUpdateMetrics::new();
        metrics.record_batch(10, 5.0);
        assert_eq!(metrics.total_batches(), 1);
        assert_eq!(metrics.total_updates(), 10);
    }

    #[test]
    fn test_average_batch_size() {
        let mut metrics = DomUpdateMetrics::new();
        metrics.record_batch(10, 5.0);
        metrics.record_batch(20, 10.0);
        assert_eq!(metrics.average_batch_size(), 15.0);
    }

    #[test]
    fn test_average_batch_time() {
        let mut metrics = DomUpdateMetrics::new();
        metrics.record_batch(10, 5.0);
        metrics.record_batch(20, 10.0);
        assert_eq!(metrics.average_batch_time(), 7.5);
    }

    #[test]
    fn test_multiple_batches() {
        let mut metrics = DomUpdateMetrics::new();
        metrics.record_batch(5, 2.5);
        metrics.record_batch(10, 5.0);
        metrics.record_batch(15, 7.5);
        assert_eq!(metrics.total_batches(), 3);
        assert_eq!(metrics.total_updates(), 30);
        assert_eq!(metrics.average_batch_size(), 10.0);
        assert_eq!(metrics.average_batch_time(), 5.0);
    }
}
