   /// Database query result caching
   ///
   /// Provides time-based caching for database query results with automatic invalidation.
   ///
   /// # Architecture
   ///
   /// This is a **Trusty Module** - pure caching logic with no I/O:
   /// - Cache entry management with TTL
   /// - Query result serialization
   /// - Cache invalidation on data changes
   /// - Hit/miss tracking
   ///
   /// # Components
   ///
   /// - [`QueryCache`]: Time-based cache with automatic expiration
   /// - [`QueryKey`]: Cache key generation from queries
   /// - [`CacheEntry`]: Cached query result with metadata

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// Default cache TTL (5 minutes)
const DEFAULT_TTL: Duration = Duration::from_secs(300);

/// Query cache key
///
/// # Example
///
/// ```
/// use midi_daw::profiling::query_cache::QueryKey;
///
/// let key = QueryKey::from_query("SELECT * FROM files WHERE id = ?", &[1]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryKey {
    /// SQL query string
    query: String,
    /// Query parameters (serialized)
    params: Vec<String>,
}

impl QueryKey {
    /// Create a new query key
    ///
    /// # Arguments
    ///
    /// * `query` - SQL query string
    /// * `params` - Query parameters
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::QueryKey;
    ///
    /// let key = QueryKey::new("SELECT * FROM files".to_string(), vec![]);
    /// ```
    pub fn new(query: String, params: Vec<String>) -> Self {
        Self { query, params }
    }

    /// Create a query key from query and parameters
    ///
    /// # Arguments
    ///
    /// * `query` - SQL query string
    /// * `params` - Query parameters (any serializable type)
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::QueryKey;
    ///
    /// let key = QueryKey::from_query("SELECT * FROM files WHERE id = ?", &[1, 2, 3]);
    /// ```
    pub fn from_query<T: std::fmt::Display>(query: &str, params: &[T]) -> Self {
        let param_strings: Vec<String> = params.iter().map(|p| p.to_string()).collect();
        Self {
            query: query.to_string(),
            params: param_strings,
        }
    }

    /// Get query string
    ///
    /// # Returns
    ///
    /// Reference to SQL query
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::QueryKey;
    ///
    /// let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
    /// assert_eq!(key.query(), "SELECT * FROM files");
    /// ```
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Get query parameters
    ///
    /// # Returns
    ///
    /// Reference to parameter list
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::QueryKey;
    ///
    /// let key = QueryKey::from_query("SELECT * FROM files WHERE id = ?", &[42]);
    /// assert_eq!(key.params(), &["42".to_string()]);
    /// ```
    pub fn params(&self) -> &[String] {
        &self.params
    }
}

/// Cached query result entry
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// Cached result
    result: T,
    /// Cache entry creation time
    created_at: Instant,
    /// Time-to-live
    ttl: Duration,
    /// Number of times accessed
    access_count: usize,
    /// Last access time
    last_access: Instant,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    fn new(result: T, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            result,
            created_at: now,
            ttl,
            access_count: 0,
            last_access: now,
        }
    }

    /// Check if entry is expired
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    /// Access the cached result
    fn access(&mut self) -> &T {
        self.access_count = self.access_count.saturating_add(1);
        self.last_access = Instant::now();
        &self.result
    }
}

/// Query result cache with time-based expiration
///
/// # Example
///
/// ```
/// use midi_daw::profiling::query_cache::{QueryCache, QueryKey};
/// use std::time::Duration;
///
/// let mut cache = QueryCache::new(Duration::from_secs(300));
/// let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
/// cache.insert(key.clone(), vec!["result1".to_string()]);
/// assert_eq!(cache.get(&key), Some(vec!["result1".to_string()]));
/// ```
#[derive(Debug)]
pub struct QueryCache<T> {
    /// Cache entries
    entries: HashMap<QueryKey, CacheEntry<T>>,
    /// Default TTL for new entries
    default_ttl: Duration,
    /// Total cache hits
    hits: usize,
    /// Total cache misses
    misses: usize,
    /// Total expirations
    expirations: usize,
    /// Total invalidations
    invalidations: usize,
}

impl<T: Clone> QueryCache<T> {
    /// Create a new query cache
    ///
    /// # Arguments
    ///
    /// * `default_ttl` - Default time-to-live for cache entries
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::QueryCache;
    /// use std::time::Duration;
    ///
    /// let cache: QueryCache<Vec<String>> = QueryCache::new(Duration::from_secs(300));
    /// ```
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            entries: HashMap::new(),
            default_ttl,
            hits: 0,
            misses: 0,
            expirations: 0,
            invalidations: 0,
        }
    }

    /// Create a new query cache with default TTL (5 minutes)
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::QueryCache;
    ///
    /// let cache: QueryCache<Vec<String>> = QueryCache::with_default_ttl();
    /// ```
    pub fn with_default_ttl() -> Self {
        Self::new(DEFAULT_TTL)
    }

    /// Insert a query result into the cache
    ///
    /// # Arguments
    ///
    /// * `key` - Query cache key
    /// * `result` - Query result to cache
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::{QueryCache, QueryKey};
    ///
    /// let mut cache = QueryCache::with_default_ttl();
    /// let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
    /// cache.insert(key, vec!["result".to_string()]);
    /// ```
    pub fn insert(&mut self, key: QueryKey, result: T) {
        self.insert_with_ttl(key, result, self.default_ttl);
    }

    /// Insert a query result with custom TTL
    ///
    /// # Arguments
    ///
    /// * `key` - Query cache key
    /// * `result` - Query result to cache
    /// * `ttl` - Time-to-live for this entry
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::{QueryCache, QueryKey};
    /// use std::time::Duration;
    ///
    /// let mut cache = QueryCache::with_default_ttl();
    /// let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
    /// cache.insert_with_ttl(key, vec!["result".to_string()], Duration::from_secs(60));
    /// ```
    pub fn insert_with_ttl(&mut self, key: QueryKey, result: T, ttl: Duration) {
        let entry = CacheEntry::new(result, ttl);
        self.entries.insert(key, entry);
    }

    /// Get a cached query result (cloned)
    ///
    /// # Arguments
    ///
    /// * `key` - Query cache key
    ///
    /// # Returns
    ///
    /// Cloned cached value, or None if not found or expired
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::{QueryCache, QueryKey};
    ///
    /// let mut cache = QueryCache::with_default_ttl();
    /// let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
    /// cache.insert(key.clone(), vec!["result".to_string()]);
    /// assert_eq!(cache.get(&key), Some(vec!["result".to_string()]));
    /// ```
    pub fn get(&mut self, key: &QueryKey) -> Option<T> {
        // Check if entry exists and is expired
        let is_expired = self.entries.get(key).map(|e| e.is_expired()).unwrap_or(false);

        if is_expired {
            self.entries.remove(key);
            self.misses = self.misses.saturating_add(1);
            self.expirations = self.expirations.saturating_add(1);
            return None;
        }

        // Get and update entry
        if let Some(entry) = self.entries.get_mut(key) {
            self.hits = self.hits.saturating_add(1);
            entry.access_count = entry.access_count.saturating_add(1);
            entry.last_access = Instant::now();
            Some(entry.result.clone())
        } else {
            self.misses = self.misses.saturating_add(1);
            None
        }
    }

    /// Remove a cached entry
    ///
    /// # Arguments
    ///
    /// * `key` - Query cache key
    ///
    /// # Returns
    ///
    /// Cached result, or None if not found
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::{QueryCache, QueryKey};
    ///
    /// let mut cache = QueryCache::with_default_ttl();
    /// let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
    /// cache.insert(key.clone(), vec!["result".to_string()]);
    /// assert_eq!(cache.remove(&key), Some(vec!["result".to_string()]));
    /// ```
    pub fn remove(&mut self, key: &QueryKey) -> Option<T> {
        self.entries.remove(key).map(|entry| entry.result)
    }

    /// Invalidate cache entries matching a pattern
    ///
    /// # Arguments
    ///
    /// * `pattern` - SQL pattern to match (e.g., "files" to invalidate all file queries)
    ///
    /// # Returns
    ///
    /// Number of entries invalidated
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::{QueryCache, QueryKey};
    ///
    /// let mut cache = QueryCache::with_default_ttl();
    /// let key1 = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
    /// let key2 = QueryKey::from_query("SELECT * FROM tags", &[] as &[i32]);
    /// cache.insert(key1, vec!["files".to_string()]);
    /// cache.insert(key2, vec!["tags".to_string()]);
    ///
    /// let invalidated = cache.invalidate_pattern("files");
    /// assert_eq!(invalidated, 1);
    /// ```
    pub fn invalidate_pattern(&mut self, pattern: &str) -> usize {
        let keys_to_remove: Vec<QueryKey> = self.entries
            .keys()
            .filter(|k| k.query.contains(pattern))
            .cloned()
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.entries.remove(&key);
        }

        self.invalidations = self.invalidations.saturating_add(count);
        count
    }

    /// Clear all cache entries
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::{QueryCache, QueryKey};
    ///
    /// let mut cache = QueryCache::with_default_ttl();
    /// let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
    /// cache.insert(key, vec!["result".to_string()]);
    /// cache.clear();
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        let count = self.entries.len();
        self.entries.clear();
        self.invalidations = self.invalidations.saturating_add(count);
    }

    /// Remove expired entries
    ///
    /// # Returns
    ///
    /// Number of entries removed
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::{QueryCache, QueryKey};
    /// use std::time::Duration;
    /// use std::thread;
    ///
    /// let mut cache = QueryCache::new(Duration::from_millis(10));
    /// let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
    /// cache.insert(key, vec!["result".to_string()]);
    ///
    /// thread::sleep(Duration::from_millis(20));
    /// let removed = cache.cleanup_expired();
    /// assert_eq!(removed, 1);
    /// ```
    pub fn cleanup_expired(&mut self) -> usize {
        let keys_to_remove: Vec<QueryKey> = self.entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            self.entries.remove(&key);
        }

        self.expirations = self.expirations.saturating_add(count);
        count
    }

    /// Get number of cached entries
    ///
    /// # Returns
    ///
    /// Number of entries in cache
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::{QueryCache, QueryKey};
    ///
    /// let mut cache = QueryCache::with_default_ttl();
    /// let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
    /// cache.insert(key, vec!["result".to_string()]);
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
    /// use midi_daw::profiling::query_cache::QueryCache;
    ///
    /// let cache: QueryCache<Vec<String>> = QueryCache::with_default_ttl();
    /// assert!(cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
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
    /// use midi_daw::profiling::query_cache::{QueryCache, QueryKey};
    ///
    /// let mut cache = QueryCache::with_default_ttl();
    /// let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
    /// cache.insert(key.clone(), vec!["result".to_string()]);
    /// cache.get(&key); // hit
    /// cache.get(&QueryKey::from_query("SELECT * FROM tags", &[] as &[i32])); // miss
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
    /// Tuple of (hits, misses, expirations, invalidations, hit_rate)
    ///
    /// # Example
    ///
    /// ```
    /// use midi_daw::profiling::query_cache::{QueryCache, QueryKey};
    ///
    /// let mut cache = QueryCache::with_default_ttl();
    /// let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
    /// cache.insert(key.clone(), vec!["result".to_string()]);
    /// cache.get(&key);
    ///
    /// let (hits, misses, expirations, invalidations, hit_rate) = cache.stats();
    /// assert_eq!(hits, 1);
    /// ```
    pub fn stats(&self) -> (usize, usize, usize, usize, f64) {
        (self.hits, self.misses, self.expirations, self.invalidations, self.hit_rate())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    // ========================================================================
    // QueryKey Tests
    // ========================================================================

    #[test]
    fn test_query_key_new() {
        let key = QueryKey::new("SELECT * FROM files".to_string(), vec![]);
        assert_eq!(key.query(), "SELECT * FROM files");
        assert_eq!(key.params(), &[] as &[String]);
    }

    #[test]
    fn test_query_key_from_query() {
        let key = QueryKey::from_query("SELECT * FROM files WHERE id = ?", &[42]);
        assert_eq!(key.query(), "SELECT * FROM files WHERE id = ?");
        assert_eq!(key.params(), &["42".to_string()]);
    }

    #[test]
    fn test_query_key_equality() {
        let key1 = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        let key2 = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_query_key_different_params() {
        let key1 = QueryKey::from_query("SELECT * FROM files WHERE id = ?", &[1]);
        let key2 = QueryKey::from_query("SELECT * FROM files WHERE id = ?", &[2]);
        assert_ne!(key1, key2);
    }

    // ========================================================================
    // QueryCache Tests
    // ========================================================================

    #[test]
    fn test_query_cache_new() {
        let cache: QueryCache<Vec<String>> = QueryCache::with_default_ttl();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_query_cache_insert_and_get() {
        let mut cache = QueryCache::with_default_ttl();
        let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        cache.insert(key.clone(), vec!["result".to_string()]);

        assert_eq!(cache.get(&key), Some(vec!["result".to_string()]));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_query_cache_get_miss() {
        let mut cache: QueryCache<Vec<String>> = QueryCache::with_default_ttl();
        let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_query_cache_hit_rate() {
        let mut cache = QueryCache::with_default_ttl();
        let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        cache.insert(key.clone(), vec!["result".to_string()]);

        cache.get(&key); // hit
        cache.get(&QueryKey::from_query("SELECT * FROM tags", &[] as &[i32])); // miss

        assert_eq!(cache.hit_rate(), 0.5);
    }

    #[test]
    fn test_query_cache_expiration() {
        let mut cache = QueryCache::new(Duration::from_millis(10));
        let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        cache.insert(key.clone(), vec!["result".to_string()]);

        thread::sleep(Duration::from_millis(20));
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn test_query_cache_remove() {
        let mut cache = QueryCache::with_default_ttl();
        let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        cache.insert(key.clone(), vec!["result".to_string()]);

        assert_eq!(cache.remove(&key), Some(vec!["result".to_string()]));
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_query_cache_invalidate_pattern() {
        let mut cache = QueryCache::with_default_ttl();
        let key1 = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        let key2 = QueryKey::from_query("SELECT * FROM tags", &[] as &[i32]);
        cache.insert(key1, vec!["files".to_string()]);
        cache.insert(key2, vec!["tags".to_string()]);

        let invalidated = cache.invalidate_pattern("files");
        assert_eq!(invalidated, 1);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_query_cache_clear() {
        let mut cache = QueryCache::with_default_ttl();
        let key1 = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        let key2 = QueryKey::from_query("SELECT * FROM tags", &[] as &[i32]);
        cache.insert(key1, vec!["files".to_string()]);
        cache.insert(key2, vec!["tags".to_string()]);

        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_query_cache_cleanup_expired() {
        let mut cache = QueryCache::new(Duration::from_millis(10));
        let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        cache.insert(key, vec!["result".to_string()]);

        thread::sleep(Duration::from_millis(20));
        let removed = cache.cleanup_expired();
        assert_eq!(removed, 1);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_query_cache_stats() {
        let mut cache = QueryCache::with_default_ttl();
        let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        cache.insert(key.clone(), vec!["result".to_string()]);

        cache.get(&key); // hit
        cache.get(&QueryKey::from_query("SELECT * FROM tags", &[] as &[i32])); // miss

        let (hits, misses, expirations, invalidations, hit_rate) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert_eq!(hit_rate, 0.5);
    }

    #[test]
    fn test_query_cache_custom_ttl() {
        let mut cache = QueryCache::with_default_ttl();
        let key = QueryKey::from_query("SELECT * FROM files", &[] as &[i32]);
        cache.insert_with_ttl(key.clone(), vec!["result".to_string()], Duration::from_millis(10));

        thread::sleep(Duration::from_millis(20));
        assert_eq!(cache.get(&key), None);
    }
}
