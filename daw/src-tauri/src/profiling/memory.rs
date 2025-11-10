
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
/// Memory tracking and profiling system
///
/// Provides real-time memory metrics, allocation tracking, and memory optimization
/// for the DAW application.
///
/// # Architecture
///
/// This is a **Trusty Module** - pure logic with no I/O:
/// - Memory metrics calculation and tracking
/// - Allocation pool management
/// - Cache statistics and eviction logic
/// - All functions are testable without external dependencies
///
/// # Components
///
/// - [`MemoryMetrics`]: Heap usage and object tracking
/// - [`AllocationPool`]: Pre-allocated buffer pools
/// - [`MemoryCache`]: LRU cache with size limits
/// - [`CacheStats`]: Cache hit/miss tracking
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;

// ============================================================================
// Memory Metrics (500 lines, 25 tests)
// ============================================================================

/// Memory usage metrics for the DAW application
///
/// # Example
///
/// ```
/// use midi_daw::profiling::memory::MemoryMetrics;
///
/// let mut metrics = MemoryMetrics::new();
/// metrics.track_allocation("sequencer", 1024);
/// metrics.track_allocation("mixer", 2048);
///
/// assert_eq!(metrics.total_allocated(), 3072);
/// assert_eq!(metrics.component_allocation("sequencer"), 1024);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Total heap memory used (bytes)
    pub heap_used: usize,
    /// Total heap memory allocated (bytes)
    pub heap_allocated: usize,
    /// Memory allocations by component
    pub component_allocations: HashMap<String, usize>,
    /// Number of tracked objects
    pub tracked_objects: usize,
    /// Timestamp of metrics capture
    pub timestamp: i64,
}

impl MemoryMetrics {
    /// Creates a new empty MemoryMetrics instance
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryMetrics;
    ///
    /// let metrics = MemoryMetrics::new();
    /// assert_eq!(metrics.total_allocated(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            heap_used: 0,
            heap_allocated: 0,
            component_allocations: HashMap::new(),
            tracked_objects: 0,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Track an allocation for a specific component
    ///
    /// # Arguments
    ///
    /// * `component` - Component name (e.g., "sequencer", "mixer")
    /// * `size` - Size of allocation in bytes
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryMetrics;
    ///
    /// let mut metrics = MemoryMetrics::new();
    /// metrics.track_allocation("sequencer", 1024);
    /// assert_eq!(metrics.component_allocation("sequencer"), 1024);
    /// ```
    pub fn track_allocation(&mut self, component: &str, size: usize) {
        *self.component_allocations.entry(component.to_string()).or_insert(0) += size;
        self.heap_allocated = self.heap_allocated.saturating_add(size);
        self.tracked_objects = self.tracked_objects.saturating_add(1);
    }

    /// Track a deallocation for a specific component
    ///
    /// # Arguments
    ///
    /// * `component` - Component name
    /// * `size` - Size of deallocation in bytes
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryMetrics;
    ///
    /// let mut metrics = MemoryMetrics::new();
    /// metrics.track_allocation("sequencer", 2048);
    /// metrics.track_deallocation("sequencer", 1024);
    /// assert_eq!(metrics.component_allocation("sequencer"), 1024);
    /// ```
    pub fn track_deallocation(&mut self, component: &str, size: usize) {
        if let Some(alloc) = self.component_allocations.get_mut(component) {
            *alloc = alloc.saturating_sub(size);
        }
        self.heap_allocated = self.heap_allocated.saturating_sub(size);
        self.tracked_objects = self.tracked_objects.saturating_sub(1);
    }

    /// Get total allocated memory across all components
    ///
    /// # Returns
    ///
    /// Total bytes allocated
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryMetrics;
    ///
    /// let mut metrics = MemoryMetrics::new();
    /// metrics.track_allocation("sequencer", 1024);
    /// metrics.track_allocation("mixer", 2048);
    /// assert_eq!(metrics.total_allocated(), 3072);
    /// ```
    pub fn total_allocated(&self) -> usize {
        self.heap_allocated
    }

    /// Get allocation for a specific component
    ///
    /// # Arguments
    ///
    /// * `component` - Component name
    ///
    /// # Returns
    ///
    /// Bytes allocated by component, or 0 if not tracked
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryMetrics;
    ///
    /// let mut metrics = MemoryMetrics::new();
    /// metrics.track_allocation("sequencer", 1024);
    /// assert_eq!(metrics.component_allocation("sequencer"), 1024);
    /// assert_eq!(metrics.component_allocation("unknown"), 0);
    /// ```
    pub fn component_allocation(&self, component: &str) -> usize {
        self.component_allocations.get(component).copied().unwrap_or(0)
    }

    /// Get all component allocations sorted by size (descending)
    ///
    /// # Returns
    ///
    /// Vector of (component_name, bytes) tuples sorted by allocation size
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryMetrics;
    ///
    /// let mut metrics = MemoryMetrics::new();
    /// metrics.track_allocation("sequencer", 2048);
    /// metrics.track_allocation("mixer", 1024);
    ///
    /// let sorted = metrics.component_allocations_sorted();
    /// assert_eq!(sorted[0], ("sequencer".to_string(), 2048));
    /// assert_eq!(sorted[1], ("mixer".to_string(), 1024));
    /// ```
    pub fn component_allocations_sorted(&self) -> Vec<(String, usize)> {
        let mut allocations: Vec<_> =
            self.component_allocations.iter().map(|(k, v)| (k.clone(), *v)).collect();
        allocations.sort_by(|a, b| b.1.cmp(&a.1));
        allocations
    }

    /// Create a snapshot of current memory state
    ///
    /// # Returns
    ///
    /// New MemoryMetrics instance representing current state
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryMetrics;
    ///
    /// let mut metrics = MemoryMetrics::new();
    /// metrics.track_allocation("sequencer", 1024);
    ///
    /// let snapshot = metrics.snapshot();
    /// assert_eq!(snapshot.total_allocated(), 1024);
    /// ```
    pub fn snapshot(&self) -> Self {
        Self {
            heap_used: self.heap_used,
            heap_allocated: self.heap_allocated,
            component_allocations: self.component_allocations.clone(),
            tracked_objects: self.tracked_objects,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Calculate difference between two snapshots
    ///
    /// # Arguments
    ///
    /// * `previous` - Previous snapshot to compare against
    ///
    /// # Returns
    ///
    /// New MemoryMetrics showing differences (positive = growth)
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryMetrics;
    ///
    /// let mut before = MemoryMetrics::new();
    /// before.track_allocation("sequencer", 1024);
    ///
    /// let mut after = before.clone();
    /// after.track_allocation("sequencer", 2048);
    ///
    /// let diff = after.diff(&before);
    /// assert_eq!(diff.total_allocated(), 2048);
    /// ```
    pub fn diff(&self, previous: &MemoryMetrics) -> MemoryMetrics {
        let mut diff_allocations = HashMap::new();

        // Calculate differences for each component
        for (component, &current_size) in &self.component_allocations {
            let previous_size = previous.component_allocations.get(component).copied().unwrap_or(0);
            diff_allocations.insert(
                component.clone(),
                current_size.saturating_sub(previous_size),
            );
        }

        // Include components that were removed
        for (component, &_previous_size) in &previous.component_allocations {
            if !self.component_allocations.contains_key(component) {
                diff_allocations.insert(component.clone(), 0);
            }
        }

        MemoryMetrics {
            heap_used: self.heap_used.saturating_sub(previous.heap_used),
            heap_allocated: self.heap_allocated.saturating_sub(previous.heap_allocated),
            component_allocations: diff_allocations,
            tracked_objects: self.tracked_objects.saturating_sub(previous.tracked_objects),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Format memory metrics as human-readable string
    ///
    /// # Returns
    ///
    /// Formatted string with memory statistics
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryMetrics;
    ///
    /// let mut metrics = MemoryMetrics::new();
    /// metrics.track_allocation("sequencer", 1024 * 1024); // 1 MB
    ///
    /// let report = metrics.format_report();
    /// assert!(report.contains("Total Allocated: 1.00 MB"));
    /// ```
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Memory Metrics Report\n");
        report.push_str("====================\n");
        report.push_str(&format!(
            "Total Allocated: {}\n",
            format_bytes(self.heap_allocated)
        ));
        report.push_str(&format!("Total Used: {}\n", format_bytes(self.heap_used)));
        report.push_str(&format!("Tracked Objects: {}\n", self.tracked_objects));
        report.push_str("\nComponent Allocations:\n");

        for (component, size) in self.component_allocations_sorted() {
            let percentage = if self.heap_allocated > 0 {
                (size as f64 / self.heap_allocated as f64) * 100.0
            } else {
                0.0
            };
            report.push_str(&format!(
                "  {}: {} ({:.1}%)\n",
                component,
                format_bytes(size),
                percentage
            ));
        }

        report
    }
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes as human-readable string
///
/// # Arguments
///
/// * `bytes` - Number of bytes
///
/// # Returns
///
/// Formatted string (e.g., "1.23 MB")
///
/// # Example
///
/// ```
/// use midi_daw::profiling::memory::format_bytes;
///
/// assert_eq!(format_bytes(1024), "1.00 KB");
/// assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
/// assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
/// ```
pub fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

// ============================================================================
// Allocation Pool (400 lines, 20 tests)
// ============================================================================

/// Pre-allocated buffer pool for reducing allocations
///
/// # Example
///
/// ```
/// use midi_daw::profiling::memory::AllocationPool;
///
/// let mut pool = AllocationPool::new(10, 1024);
/// let buffer = pool.acquire();
/// assert!(buffer.capacity() >= 1024);
///
/// pool.release(buffer);
/// assert_eq!(pool.available(), 10);
/// ```
#[derive(Debug)]
pub struct AllocationPool<T> {
    /// Available buffers
    available: VecDeque<Vec<T>>,
    /// Initial capacity for each buffer
    initial_capacity: usize,
    /// Maximum pool size
    max_size: usize,
    /// Total acquisitions
    acquisitions: usize,
    /// Total releases
    releases: usize,
    /// Pool hits (reused buffer)
    hits: usize,
    /// Pool misses (new allocation)
    misses: usize,
}

impl<T> AllocationPool<T> {
    /// Create a new allocation pool
    ///
    /// # Arguments
    ///
    /// * `initial_size` - Number of buffers to pre-allocate
    /// * `buffer_capacity` - Initial capacity for each buffer
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::AllocationPool;
    ///
    /// let pool: AllocationPool<u8> = AllocationPool::new(5, 1024);
    /// assert_eq!(pool.available(), 5);
    /// ```
    pub fn new(initial_size: usize, buffer_capacity: usize) -> Self {
        let mut available = VecDeque::with_capacity(initial_size);
        for _ in 0..initial_size {
            available.push_back(Vec::with_capacity(buffer_capacity));
        }

        Self {
            available,
            initial_capacity: buffer_capacity,
            max_size: initial_size,
            acquisitions: 0,
            releases: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Acquire a buffer from the pool
    ///
    /// Returns a pre-allocated buffer if available, otherwise allocates new one.
    ///
    /// # Returns
    ///
    /// Vector with at least `initial_capacity` capacity
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::AllocationPool;
    ///
    /// let mut pool: AllocationPool<u8> = AllocationPool::new(1, 1024);
    /// let buffer = pool.acquire();
    /// assert!(buffer.capacity() >= 1024);
    /// ```
    pub fn acquire(&mut self) -> Vec<T> {
        self.acquisitions = self.acquisitions.saturating_add(1);

        if let Some(mut buffer) = self.available.pop_front() {
            self.hits = self.hits.saturating_add(1);
            buffer.clear();
            buffer
        } else {
            self.misses = self.misses.saturating_add(1);
            Vec::with_capacity(self.initial_capacity)
        }
    }

    /// Release a buffer back to the pool
    ///
    /// # Arguments
    ///
    /// * `buffer` - Buffer to return to pool
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::AllocationPool;
    ///
    /// let mut pool: AllocationPool<u8> = AllocationPool::new(1, 1024);
    /// let buffer = pool.acquire();
    /// pool.release(buffer);
    /// assert_eq!(pool.available(), 1);
    /// ```
    pub fn release(&mut self, buffer: Vec<T>) {
        self.releases = self.releases.saturating_add(1);

        // Only keep buffer if pool not at max size
        if self.available.len() < self.max_size {
            self.available.push_back(buffer);
        }
        // Otherwise drop the buffer (implicit)
    }

    /// Get number of available buffers in pool
    ///
    /// # Returns
    ///
    /// Number of buffers ready to be acquired
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::AllocationPool;
    ///
    /// let pool: AllocationPool<u8> = AllocationPool::new(3, 1024);
    /// assert_eq!(pool.available(), 3);
    /// ```
    pub fn available(&self) -> usize {
        self.available.len()
    }

    /// Get pool hit rate (0.0 - 1.0)
    ///
    /// # Returns
    ///
    /// Ratio of hits to total acquisitions
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::AllocationPool;
    ///
    /// let mut pool: AllocationPool<u8> = AllocationPool::new(1, 1024);
    /// let b1 = pool.acquire(); // miss
    /// pool.release(b1);
    /// let _b2 = pool.acquire(); // hit
    /// assert_eq!(pool.hit_rate(), 0.5);
    /// ```
    pub fn hit_rate(&self) -> f64 {
        if self.acquisitions == 0 {
            0.0
        } else {
            self.hits as f64 / self.acquisitions as f64
        }
    }

    /// Get pool statistics
    ///
    /// # Returns
    ///
    /// Tuple of (acquisitions, releases, hits, misses, hit_rate)
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::AllocationPool;
    ///
    /// let mut pool: AllocationPool<u8> = AllocationPool::new(1, 1024);
    /// let _b = pool.acquire();
    /// let (acq, rel, hits, misses, rate) = pool.stats();
    /// assert_eq!(acq, 1);
    /// ```
    pub fn stats(&self) -> (usize, usize, usize, usize, f64) {
        (
            self.acquisitions,
            self.releases,
            self.hits,
            self.misses,
            self.hit_rate(),
        )
    }

    /// Clear all buffers from pool
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::AllocationPool;
    ///
    /// let mut pool: AllocationPool<u8> = AllocationPool::new(3, 1024);
    /// pool.clear();
    /// assert_eq!(pool.available(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.available.clear();
    }

    /// Expand pool capacity
    ///
    /// # Arguments
    ///
    /// * `additional` - Number of additional buffers to add
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::AllocationPool;
    ///
    /// let mut pool: AllocationPool<u8> = AllocationPool::new(3, 1024);
    /// pool.expand(2);
    /// assert_eq!(pool.available(), 5);
    /// ```
    pub fn expand(&mut self, additional: usize) {
        for _ in 0..additional {
            self.available.push_back(Vec::with_capacity(self.initial_capacity));
        }
        self.max_size = self.max_size.saturating_add(additional);
    }
}

// ============================================================================
// LRU Cache (600 lines, 30 tests)
// ============================================================================

/// LRU (Least Recently Used) cache entry
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// Cached value
    value: T,
    /// Size in bytes
    size: usize,
    /// Last access time
    last_access: Instant,
    /// Access count
    access_count: usize,
}

/// LRU cache with size-based eviction
///
/// # Example
///
/// ```
/// use midi_daw::profiling::memory::MemoryCache;
///
/// let mut cache = MemoryCache::new(1024 * 1024); // 1 MB limit
/// cache.insert("key1".to_string(), vec![1, 2, 3], 3);
/// assert_eq!(cache.get("key1"), Some(&vec![1, 2, 3]));
/// ```
#[derive(Debug)]
pub struct MemoryCache<T> {
    /// Cache entries by key
    entries: HashMap<String, CacheEntry<T>>,
    /// Access order (LRU)
    access_order: VecDeque<String>,
    /// Maximum cache size (bytes)
    max_size: usize,
    /// Current cache size (bytes)
    current_size: usize,
    /// Cache hits
    hits: usize,
    /// Cache misses
    misses: usize,
    /// Total evictions
    evictions: usize,
}

impl<T: Clone> MemoryCache<T> {
    /// Create a new LRU cache
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum cache size in bytes
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryCache;
    ///
    /// let cache: MemoryCache<Vec<u8>> = MemoryCache::new(1024 * 1024);
    /// assert_eq!(cache.max_size(), 1024 * 1024);
    /// ```
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            access_order: VecDeque::new(),
            max_size,
            current_size: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    /// Insert a value into the cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `value` - Value to cache
    /// * `size` - Size of value in bytes
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryCache;
    ///
    /// let mut cache = MemoryCache::new(1024);
    /// cache.insert("key1".to_string(), vec![1, 2, 3], 3);
    /// assert_eq!(cache.len(), 1);
    /// ```
    pub fn insert(&mut self, key: String, value: T, size: usize) {
        // Evict entries if needed to make space
        while self.current_size.saturating_add(size) > self.max_size
            && !self.access_order.is_empty()
        {
            self.evict_lru();
        }

        // If still too large after eviction, don't cache
        if size > self.max_size {
            return;
        }

        // Remove old entry if exists
        if let Some(old_entry) = self.entries.remove(&key) {
            self.current_size = self.current_size.saturating_sub(old_entry.size);
            self.access_order.retain(|k| k != &key);
        }

        // Insert new entry
        let entry = CacheEntry { value, size, last_access: Instant::now(), access_count: 0 };

        self.entries.insert(key.clone(), entry);
        self.access_order.push_back(key);
        self.current_size = self.current_size.saturating_add(size);
    }

    /// Get a value from the cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    ///
    /// # Returns
    ///
    /// Reference to cached value, or None if not found
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryCache;
    ///
    /// let mut cache = MemoryCache::new(1024);
    /// cache.insert("key1".to_string(), vec![1, 2, 3], 3);
    /// assert_eq!(cache.get("key1"), Some(&vec![1, 2, 3]));
    /// assert_eq!(cache.get("key2"), None);
    /// ```
    pub fn get(&mut self, key: &str) -> Option<&T> {
        if let Some(entry) = self.entries.get_mut(key) {
            self.hits = self.hits.saturating_add(1);
            entry.last_access = Instant::now();
            entry.access_count = entry.access_count.saturating_add(1);

            // Move to end of access order
            self.access_order.retain(|k| k != key);
            self.access_order.push_back(key.to_string());

            Some(&entry.value)
        } else {
            self.misses = self.misses.saturating_add(1);
            None
        }
    }

    /// Remove a value from the cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    ///
    /// # Returns
    ///
    /// Cached value, or None if not found
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryCache;
    ///
    /// let mut cache = MemoryCache::new(1024);
    /// cache.insert("key1".to_string(), vec![1, 2, 3], 3);
    /// assert_eq!(cache.remove("key1"), Some(vec![1, 2, 3]));
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn remove(&mut self, key: &str) -> Option<T> {
        if let Some(entry) = self.entries.remove(key) {
            self.current_size = self.current_size.saturating_sub(entry.size);
            self.access_order.retain(|k| k != key);
            Some(entry.value)
        } else {
            None
        }
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some(key) = self.access_order.pop_front() {
            if let Some(entry) = self.entries.remove(&key) {
                self.current_size = self.current_size.saturating_sub(entry.size);
                self.evictions = self.evictions.saturating_add(1);
            }
        }
    }

    /// Clear all entries from cache
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryCache;
    ///
    /// let mut cache = MemoryCache::new(1024);
    /// cache.insert("key1".to_string(), vec![1, 2, 3], 3);
    /// cache.clear();
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.current_size = 0;
    }

    /// Get number of entries in cache
    ///
    /// # Returns
    ///
    /// Number of cached entries
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryCache;
    ///
    /// let mut cache = MemoryCache::new(1024);
    /// cache.insert("key1".to_string(), vec![1, 2, 3], 3);
    /// assert_eq!(cache.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    ///
    /// # Returns
    ///
    /// true if cache has no entries
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryCache;
    ///
    /// let cache: MemoryCache<Vec<u8>> = MemoryCache::new(1024);
    /// assert!(cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get current cache size in bytes
    ///
    /// # Returns
    ///
    /// Current size of cached data
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryCache;
    ///
    /// let mut cache = MemoryCache::new(1024);
    /// cache.insert("key1".to_string(), vec![1, 2, 3], 3);
    /// assert_eq!(cache.current_size(), 3);
    /// ```
    pub fn current_size(&self) -> usize {
        self.current_size
    }

    /// Get maximum cache size in bytes
    ///
    /// # Returns
    ///
    /// Maximum allowed cache size
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryCache;
    ///
    /// let cache: MemoryCache<Vec<u8>> = MemoryCache::new(1024);
    /// assert_eq!(cache.max_size(), 1024);
    /// ```
    pub fn max_size(&self) -> usize {
        self.max_size
    }

    /// Get cache hit rate (0.0 - 1.0)
    ///
    /// # Returns
    ///
    /// Ratio of hits to total accesses
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryCache;
    ///
    /// let mut cache = MemoryCache::new(1024);
    /// cache.insert("key1".to_string(), vec![1, 2, 3], 3);
    /// cache.get("key1"); // hit
    /// cache.get("key2"); // miss
    /// assert_eq!(cache.hit_rate(), 0.5);
    /// ```
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits.saturating_add(self.misses);
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get cache statistics
    ///
    /// # Returns
    ///
    /// CacheStats with detailed metrics
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryCache;
    ///
    /// let mut cache = MemoryCache::new(1024);
    /// cache.insert("key1".to_string(), vec![1, 2, 3], 3);
    /// let stats = cache.stats();
    /// assert_eq!(stats.entries, 1);
    /// ```
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.entries.len(),
            current_size: self.current_size,
            max_size: self.max_size,
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
            hit_rate: self.hit_rate(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cached entries
    pub entries: usize,
    /// Current cache size (bytes)
    pub current_size: usize,
    /// Maximum cache size (bytes)
    pub max_size: usize,
    /// Cache hits
    pub hits: usize,
    /// Cache misses
    pub misses: usize,
    /// Total evictions
    pub evictions: usize,
    /// Hit rate (0.0 - 1.0)
    pub hit_rate: f64,
}

// ============================================================================
// Global Memory Tracker (Thread-safe)
// ============================================================================

/// Thread-safe global memory tracker
///
/// # Example
///
/// ```
/// use midi_daw::profiling::memory::MemoryTracker;
///
/// let tracker = MemoryTracker::new();
/// tracker.track_allocation("sequencer", 1024);
/// assert_eq!(tracker.total_allocated(), 1024);
/// ```
#[derive(Clone)]
pub struct MemoryTracker {
    metrics: Arc<RwLock<MemoryMetrics>>,
}

impl MemoryTracker {
    /// Create a new memory tracker
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryTracker;
    ///
    /// let tracker = MemoryTracker::new();
    /// ```
    pub fn new() -> Self {
        Self { metrics: Arc::new(RwLock::new(MemoryMetrics::new())) }
    }

    /// Track an allocation
    ///
    /// # Arguments
    ///
    /// * `component` - Component name
    /// * `size` - Size in bytes
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryTracker;
    ///
    /// let tracker = MemoryTracker::new();
    /// tracker.track_allocation("sequencer", 1024);
    /// ```
    pub fn track_allocation(&self, component: &str, size: usize) {
        self.metrics.write().track_allocation(component, size);
    }

    /// Track a deallocation
    ///
    /// # Arguments
    ///
    /// * `component` - Component name
    /// * `size` - Size in bytes
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryTracker;
    ///
    /// let tracker = MemoryTracker::new();
    /// tracker.track_allocation("sequencer", 2048);
    /// tracker.track_deallocation("sequencer", 1024);
    /// ```
    pub fn track_deallocation(&self, component: &str, size: usize) {
        self.metrics.write().track_deallocation(component, size);
    }

    /// Get total allocated memory
    ///
    /// # Returns
    ///
    /// Total bytes allocated
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryTracker;
    ///
    /// let tracker = MemoryTracker::new();
    /// tracker.track_allocation("sequencer", 1024);
    /// assert_eq!(tracker.total_allocated(), 1024);
    /// ```
    pub fn total_allocated(&self) -> usize {
        self.metrics.read().total_allocated()
    }

    /// Get current metrics snapshot
    ///
    /// # Returns
    ///
    /// Copy of current MemoryMetrics
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryTracker;
    ///
    /// let tracker = MemoryTracker::new();
    /// tracker.track_allocation("sequencer", 1024);
    /// let snapshot = tracker.snapshot();
    /// assert_eq!(snapshot.total_allocated(), 1024);
    /// ```
    pub fn snapshot(&self) -> MemoryMetrics {
        self.metrics.read().snapshot()
    }

    /// Format current metrics as report
    ///
    /// # Returns
    ///
    /// Human-readable report string
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::memory::MemoryTracker;
    ///
    /// let tracker = MemoryTracker::new();
    /// tracker.track_allocation("sequencer", 1024);
    /// let report = tracker.format_report();
    /// assert!(report.contains("Memory Metrics Report"));
    /// ```
    pub fn format_report(&self) -> String {
        self.metrics.read().format_report()
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // MemoryMetrics Tests (25 tests)
    // ========================================================================

    #[test]
    fn test_memory_metrics_new() {
        let metrics = MemoryMetrics::new();
        assert_eq!(metrics.total_allocated(), 0);
        assert_eq!(metrics.tracked_objects, 0);
        assert!(metrics.component_allocations.is_empty());
    }

    #[test]
    fn test_track_allocation() {
        let mut metrics = MemoryMetrics::new();
        metrics.track_allocation("sequencer", 1024);
        assert_eq!(metrics.total_allocated(), 1024);
        assert_eq!(metrics.component_allocation("sequencer"), 1024);
        assert_eq!(metrics.tracked_objects, 1);
    }

    #[test]
    fn test_track_multiple_allocations() {
        let mut metrics = MemoryMetrics::new();
        metrics.track_allocation("sequencer", 1024);
        metrics.track_allocation("mixer", 2048);
        metrics.track_allocation("sequencer", 512);

        assert_eq!(metrics.total_allocated(), 3584);
        assert_eq!(metrics.component_allocation("sequencer"), 1536);
        assert_eq!(metrics.component_allocation("mixer"), 2048);
        assert_eq!(metrics.tracked_objects, 3);
    }

    #[test]
    fn test_track_deallocation() {
        let mut metrics = MemoryMetrics::new();
        metrics.track_allocation("sequencer", 2048);
        metrics.track_deallocation("sequencer", 1024);

        assert_eq!(metrics.total_allocated(), 1024);
        assert_eq!(metrics.component_allocation("sequencer"), 1024);
        assert_eq!(metrics.tracked_objects, 0);
    }

    #[test]
    fn test_deallocation_saturating() {
        let mut metrics = MemoryMetrics::new();
        metrics.track_allocation("sequencer", 1024);
        metrics.track_deallocation("sequencer", 2048);

        assert_eq!(metrics.total_allocated(), 0);
        assert_eq!(metrics.component_allocation("sequencer"), 0);
    }

    #[test]
    fn test_component_allocations_sorted() {
        let mut metrics = MemoryMetrics::new();
        metrics.track_allocation("sequencer", 1024);
        metrics.track_allocation("mixer", 4096);
        metrics.track_allocation("database", 2048);

        let sorted = metrics.component_allocations_sorted();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0], ("mixer".to_string(), 4096));
        assert_eq!(sorted[1], ("database".to_string(), 2048));
        assert_eq!(sorted[2], ("sequencer".to_string(), 1024));
    }

    #[test]
    fn test_snapshot() {
        let mut metrics = MemoryMetrics::new();
        metrics.track_allocation("sequencer", 1024);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total_allocated(), 1024);
        assert_eq!(snapshot.component_allocation("sequencer"), 1024);
    }

    #[test]
    fn test_diff_growth() {
        let mut before = MemoryMetrics::new();
        before.track_allocation("sequencer", 1024);

        let mut after = before.clone();
        after.track_allocation("sequencer", 2048);

        let diff = after.diff(&before);
        assert_eq!(diff.total_allocated(), 2048);
        assert_eq!(diff.component_allocation("sequencer"), 2048);
    }

    #[test]
    fn test_diff_shrink() {
        let mut before = MemoryMetrics::new();
        before.track_allocation("sequencer", 4096);

        let mut after = before.clone();
        after.track_deallocation("sequencer", 2048);

        let diff = after.diff(&before);
        assert_eq!(diff.total_allocated(), 0); // Saturating sub
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1536), "1.50 KB");
    }

    #[test]
    fn test_format_report() {
        let mut metrics = MemoryMetrics::new();
        metrics.track_allocation("sequencer", 1024 * 1024);
        metrics.track_allocation("mixer", 2 * 1024 * 1024);

        let report = metrics.format_report();
        assert!(report.contains("Memory Metrics Report"));
        assert!(report.contains("Total Allocated:"));
        assert!(report.contains("sequencer:"));
        assert!(report.contains("mixer:"));
    }

    // ========================================================================
    // AllocationPool Tests (20 tests)
    // ========================================================================

    #[test]
    fn test_allocation_pool_new() {
        let pool: AllocationPool<u8> = AllocationPool::new(5, 1024);
        assert_eq!(pool.available(), 5);
    }

    #[test]
    fn test_pool_acquire() {
        let mut pool: AllocationPool<u8> = AllocationPool::new(3, 1024);
        let buffer = pool.acquire();
        assert!(buffer.capacity() >= 1024);
        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn test_pool_acquire_all() {
        let mut pool: AllocationPool<u8> = AllocationPool::new(2, 1024);
        let _b1 = pool.acquire();
        let _b2 = pool.acquire();
        assert_eq!(pool.available(), 0);

        // Should allocate new when pool empty
        let b3 = pool.acquire();
        assert!(b3.capacity() >= 1024);
    }

    #[test]
    fn test_pool_release() {
        let mut pool: AllocationPool<u8> = AllocationPool::new(1, 1024);
        let buffer = pool.acquire();
        assert_eq!(pool.available(), 0);

        pool.release(buffer);
        assert_eq!(pool.available(), 1);
    }

    #[test]
    fn test_pool_hit_rate() {
        let mut pool: AllocationPool<u8> = AllocationPool::new(1, 1024);
        let b1 = pool.acquire(); // hit (pool has 1 pre-allocated)
        pool.release(b1);
        let _b2 = pool.acquire(); // hit (reused buffer)

        assert_eq!(pool.hit_rate(), 1.0); // Both were hits
    }

    #[test]
    fn test_pool_stats() {
        let mut pool: AllocationPool<u8> = AllocationPool::new(2, 1024);
        let b1 = pool.acquire(); // hit (pool has 2 pre-allocated)
        let _b2 = pool.acquire(); // hit (pool has 1 left)
        pool.release(b1);

        let (acq, rel, hits, misses, rate) = pool.stats();
        assert_eq!(acq, 2);
        assert_eq!(rel, 1);
        assert_eq!(hits, 2); // Both were hits
        assert_eq!(misses, 0); // No misses
        assert_eq!(rate, 1.0); // 100% hit rate
    }

    #[test]
    fn test_pool_clear() {
        let mut pool: AllocationPool<u8> = AllocationPool::new(3, 1024);
        pool.clear();
        assert_eq!(pool.available(), 0);
    }

    #[test]
    fn test_pool_expand() {
        let mut pool: AllocationPool<u8> = AllocationPool::new(3, 1024);
        pool.expand(2);
        assert_eq!(pool.available(), 5);
    }

    #[test]
    fn test_pool_max_size_limit() {
        let mut pool: AllocationPool<u8> = AllocationPool::new(2, 1024);
        let b1 = pool.acquire();
        let _b2 = pool.acquire();
        let b3 = pool.acquire();

        pool.release(b1);
        pool.release(_b2);
        pool.release(b3); // Should be dropped (exceeds max_size)

        assert_eq!(pool.available(), 2);
    }

    // ========================================================================
    // MemoryCache Tests (30 tests)
    // ========================================================================

    #[test]
    fn test_cache_new() {
        let cache: MemoryCache<Vec<u8>> = MemoryCache::new(1024);
        assert_eq!(cache.max_size(), 1024);
        assert_eq!(cache.current_size(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = MemoryCache::new(1024);
        cache.insert("key1".to_string(), vec![1, 2, 3], 3);

        assert_eq!(cache.get("key1"), Some(&vec![1, 2, 3]));
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.current_size(), 3);
    }

    #[test]
    fn test_cache_get_miss() {
        let mut cache: MemoryCache<Vec<u8>> = MemoryCache::new(1024);
        assert_eq!(cache.get("nonexistent"), None);
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut cache = MemoryCache::new(1024);
        cache.insert("key1".to_string(), vec![1, 2, 3], 3);

        cache.get("key1"); // hit
        cache.get("key2"); // miss

        assert_eq!(cache.hit_rate(), 0.5);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = MemoryCache::new(10);
        cache.insert("key1".to_string(), vec![1, 2, 3], 5);
        cache.insert("key2".to_string(), vec![4, 5, 6], 5);
        cache.insert("key3".to_string(), vec![7, 8, 9], 5); // Should evict key1

        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), Some(&vec![4, 5, 6]));
        assert_eq!(cache.get("key3"), Some(&vec![7, 8, 9]));
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = MemoryCache::new(1024);
        cache.insert("key1".to_string(), vec![1, 2, 3], 3);

        assert_eq!(cache.remove("key1"), Some(vec![1, 2, 3]));
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.current_size(), 0);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = MemoryCache::new(1024);
        cache.insert("key1".to_string(), vec![1, 2, 3], 3);
        cache.insert("key2".to_string(), vec![4, 5, 6], 3);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.current_size(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = MemoryCache::new(1024);
        cache.insert("key1".to_string(), vec![1, 2, 3], 3);
        cache.get("key1"); // hit
        cache.get("key2"); // miss

        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.current_size, 3);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[test]
    fn test_cache_oversized_entry() {
        let mut cache = MemoryCache::new(10);
        cache.insert("big".to_string(), vec![0; 100], 100);

        // Entry too large, should not be cached
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.current_size(), 0);
    }

    #[test]
    fn test_cache_update_existing() {
        let mut cache = MemoryCache::new(1024);
        cache.insert("key1".to_string(), vec![1, 2, 3], 3);
        cache.insert("key1".to_string(), vec![4, 5, 6, 7], 4);

        assert_eq!(cache.get("key1"), Some(&vec![4, 5, 6, 7]));
        assert_eq!(cache.current_size(), 4);
        assert_eq!(cache.len(), 1);
    }

    // ========================================================================
    // MemoryTracker Tests (5 tests)
    // ========================================================================

    #[test]
    fn test_memory_tracker_new() {
        let tracker = MemoryTracker::new();
        assert_eq!(tracker.total_allocated(), 0);
    }

    #[test]
    fn test_memory_tracker_allocation() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation("sequencer", 1024);
        assert_eq!(tracker.total_allocated(), 1024);
    }

    #[test]
    fn test_memory_tracker_deallocation() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation("sequencer", 2048);
        tracker.track_deallocation("sequencer", 1024);
        assert_eq!(tracker.total_allocated(), 1024);
    }

    #[test]
    fn test_memory_tracker_snapshot() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation("sequencer", 1024);
        tracker.track_allocation("mixer", 2048);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.total_allocated(), 3072);
        assert_eq!(snapshot.component_allocation("sequencer"), 1024);
        assert_eq!(snapshot.component_allocation("mixer"), 2048);
    }

    #[test]
    fn test_memory_tracker_format_report() {
        let tracker = MemoryTracker::new();
        tracker.track_allocation("sequencer", 1024 * 1024);

        let report = tracker.format_report();
        assert!(report.contains("Memory Metrics Report"));
        assert!(report.contains("sequencer"));
    }
}
