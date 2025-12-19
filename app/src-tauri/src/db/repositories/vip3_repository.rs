use crate::db::models::{DbError, DbResult, FilterCounts, Vip3Filters};
use dashmap::DashMap;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Cached filter counts with timestamp
#[derive(Clone)]
struct CachedFilterCounts {
    counts: FilterCounts,
    cached_at: Instant,
}

impl CachedFilterCounts {
    fn new(counts: FilterCounts) -> Self {
        Self {
            counts,
            cached_at: Instant::now(),
        }
    }

    fn is_valid(&self, ttl: Duration) -> bool {
        self.cached_at.elapsed() < ttl
    }
}

pub struct Vip3Repository {
    pool: PgPool,
    /// Cache for filter counts with 5-second TTL
    /// Key: serialized Vip3Filters, Value: CachedFilterCounts
    cache: Arc<DashMap<String, CachedFilterCounts>>,
    cache_ttl: Duration,
}

impl Vip3Repository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cache: Arc::new(DashMap::new()),
            cache_ttl: Duration::from_secs(5),
        }
    }

    /// Generate a cache key from filters
    fn cache_key(filters: &Vip3Filters) -> String {
        // Simple serialization for cache key
        format!(
            "f:{:?}|i:{:?}|t:{:?}|s:{:?}|a:{:?}|b:{:?}|k:{:?}|c:{:?}|q:{:?}|fav:{:?}|tag:{:?}|r:{:?}",
            filters.folder_ids,
            filters.instrument_ids,
            filters.timbre_ids,
            filters.style_ids,
            filters.articulation_ids,
            filters.bpm_range_ids,
            filters.key_ids,
            filters.channel,
            filters.search_query,
            filters.favorites_only,
            filters.tag_ids,
            filters.min_rating
        )
    }

    /// Get all filter counts in parallel
    /// Target: <50ms total execution time (with 5-second cache)
    pub async fn get_filter_counts(
        &self,
        filters: &Vip3Filters,
    ) -> DbResult<FilterCounts> {
        let start = Instant::now();

        // Check cache first
        let cache_key = Self::cache_key(filters);
        if let Some(cached) = self.cache.get(&cache_key) {
            if cached.is_valid(self.cache_ttl) {
                let elapsed = start.elapsed();
                info!(
                    "Filter counts served from cache in {:?} (cached {} ago)",
                    elapsed,
                    cached.cached_at.elapsed().as_millis()
                );
                return Ok(cached.counts.clone());
            } else {
                // Remove stale entry
                drop(cached);
                self.cache.remove(&cache_key);
            }
        }

        // Execute all count queries in parallel using tokio::join!
        let (
            folder_counts,
            instrument_counts,
            timbre_counts,
            style_counts,
            articulation_counts,
            bpm_counts,
            key_counts,
            channel_counts,
        ) = tokio::join!(
            self.count_by_folder(filters),
            self.count_by_instrument(filters),
            self.count_by_timbre(filters),
            self.count_by_style(filters),
            self.count_by_articulation(filters),
            self.count_by_bpm_range(filters),
            self.count_by_key(filters),
            self.count_by_channel(filters),
        );

        // Get total matches (files matching all current filters)
        let total_matches = self.count_total_matches(filters).await?;

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_millis();

        // Unwrap results for logging and response
        let folder_counts = folder_counts?;
        let instrument_counts = instrument_counts?;
        let timbre_counts = timbre_counts?;
        let style_counts = style_counts?;
        let articulation_counts = articulation_counts?;
        let bpm_counts = bpm_counts?;
        let key_counts = key_counts?;
        let channel_counts = channel_counts?;

        if elapsed_ms > 50 {
            warn!(
                "Dynamic filter counts slow: {}ms (folders: {}, instruments: {}, timbres: {}, styles: {}, articulations: {}, bpm_ranges: {}, keys: {}, channels: {}, total: {}) - CACHED for 5s",
                elapsed_ms,
                folder_counts.len(),
                instrument_counts.len(),
                timbre_counts.len(),
                style_counts.len(),
                articulation_counts.len(),
                bpm_counts.len(),
                key_counts.len(),
                channel_counts.len(),
                total_matches
            );
        } else {
            info!("Filter counts completed in {:?} (target: <50ms)", elapsed);
        }

        // Build result
        let result = FilterCounts {
            folder_counts,
            instrument_counts,
            timbre_counts,
            style_counts,
            articulation_counts,
            bpm_range_counts: bpm_counts,
            key_counts,
            channel_counts,
            total_matches,
        };

        // Cache the result for 5 seconds
        self.cache
            .insert(cache_key, CachedFilterCounts::new(result.clone()));

        Ok(result)
    }

    /// Build base WHERE clause from filters
    /// Excludes the specified filter type (for counting that type)
    fn build_where_clause(
        &self,
        filters: &Vip3Filters,
        exclude_filter: Option<&str>,
    ) -> String {
        let mut conditions = vec!["1=1".to_string()];

        // Folder filter
        if exclude_filter != Some("folder") {
            if let Some(ref folder_ids) = filters.folder_ids {
                if !folder_ids.is_empty() {
                    let ids_str = folder_ids.iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    conditions.push(format!("f.folder_id IN ({})", ids_str));
                }
            }
        }

        // Instrument filter
        if exclude_filter != Some("instrument") {
            if let Some(ref instrument_ids) = filters.instrument_ids {
                if !instrument_ids.is_empty() {
                    let ids_str = instrument_ids.iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    conditions.push(format!(
                        "EXISTS (SELECT 1 FROM file_tags ft WHERE ft.file_id = f.id AND ft.tag_id IN ({}))",
                        ids_str
                    ));
                }
            }
        }

        // Timbre filter
        if exclude_filter != Some("timbre") {
            if let Some(ref timbre_ids) = filters.timbre_ids {
                if !timbre_ids.is_empty() {
                    let ids_str = timbre_ids.iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    conditions.push(format!(
                        "EXISTS (SELECT 1 FROM midi_file_timbres ft WHERE ft.file_id = f.id AND ft.timbre_id IN ({}))",
                        ids_str
                    ));
                }
            }
        }

        // Style filter
        if exclude_filter != Some("style") {
            if let Some(ref style_ids) = filters.style_ids {
                if !style_ids.is_empty() {
                    let ids_str = style_ids.iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    conditions.push(format!(
                        "EXISTS (SELECT 1 FROM midi_file_styles fs WHERE fs.file_id = f.id AND fs.style_id IN ({}))",
                        ids_str
                    ));
                }
            }
        }

        // Articulation filter
        if exclude_filter != Some("articulation") {
            if let Some(ref articulation_ids) = filters.articulation_ids {
                if !articulation_ids.is_empty() {
                    let ids_str = articulation_ids.iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    conditions.push(format!(
                        "EXISTS (SELECT 1 FROM midi_file_articulations fa WHERE fa.file_id = f.id AND fa.articulation_id IN ({}))",
                        ids_str
                    ));
                }
            }
        }

        // BPM range filter
        if exclude_filter != Some("bpm") {
            if let Some(ref bpm_range_ids) = filters.bpm_range_ids {
                if !bpm_range_ids.is_empty() {
                    let ids_str = bpm_range_ids.iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    conditions.push(format!("f.bpm_range_id IN ({})", ids_str));
                }
            }
        }

        // Key filter
        if exclude_filter != Some("key") {
            if let Some(ref key_ids) = filters.key_ids {
                if !key_ids.is_empty() {
                    let ids_str = key_ids.iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    conditions.push(format!("f.key_id IN ({})", ids_str));
                }
            }
        }

        // Channel filter
        if exclude_filter != Some("channels") {
            if let Some(channel) = filters.channel {
                conditions.push(format!("f.channel_count = {}", channel));
            }
        }

        // Search query
        if let Some(ref query) = filters.search_query {
            if !query.is_empty() {
                let escaped = query.replace("'", "''");
                conditions.push(format!("f.file_path ILIKE '%{}%'", escaped));
            }
        }

        // Favorites only
        if filters.favorites_only == Some(true) {
            conditions.push(
                "EXISTS (SELECT 1 FROM favorites fav WHERE fav.file_id = f.id)".to_string()
            );
        }

        // Min rating
        if let Some(min_rating) = filters.min_rating {
            conditions.push(format!("f.rating >= {}", min_rating));
        }

        conditions.join(" AND ")
    }

    async fn count_by_folder(&self, filters: &Vip3Filters) -> DbResult<HashMap<i64, usize>> {
        let where_clause = self.build_where_clause(filters, Some("folder"));

        let query = format!(
            "SELECT f.folder_id, COUNT(DISTINCT f.id) as count
             FROM files f
             WHERE {}
             GROUP BY f.folder_id",
            where_clause
        );

        let rows = sqlx::query_as::<_, (Option<i64>, i64)>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        let mut counts = HashMap::new();
        for (folder_id, count) in rows {
            if let Some(folder_id) = folder_id {
                counts.insert(folder_id, count as usize);
            }
        }

        Ok(counts)
    }

    async fn count_by_instrument(&self, filters: &Vip3Filters) -> DbResult<HashMap<i64, usize>> {
        let where_clause = self.build_where_clause(filters, Some("instrument"));

        let query = format!(
            "SELECT ft.tag_id, COUNT(DISTINCT f.id) as count
             FROM files f
             JOIN file_tags ft ON f.id = ft.file_id
             WHERE {}
             GROUP BY ft.tag_id",
            where_clause
        );

        let rows = sqlx::query_as::<_, (i32, i64)>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        let mut counts = HashMap::new();
        for (tag_id, count) in rows {
            counts.insert(tag_id as i64, count as usize);
        }

        Ok(counts)
    }

    async fn count_by_timbre(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
        let where_clause = self.build_where_clause(filters, Some("timbre"));

        let query = format!(
            "SELECT ft.timbre_id, COUNT(DISTINCT f.id) as count
             FROM files f
             JOIN midi_file_timbres ft ON f.id = ft.file_id
             WHERE {}
             GROUP BY ft.timbre_id",
            where_clause
        );

        let rows = sqlx::query_as::<_, (i32, i64)>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        let mut counts = HashMap::new();
        for (timbre_id, count) in rows {
            counts.insert(timbre_id, count as usize);
        }

        Ok(counts)
    }

    async fn count_by_style(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
        let where_clause = self.build_where_clause(filters, Some("style"));

        let query = format!(
            "SELECT fs.style_id, COUNT(DISTINCT f.id) as count
             FROM files f
             JOIN midi_file_styles fs ON f.id = fs.file_id
             WHERE {}
             GROUP BY fs.style_id",
            where_clause
        );

        let rows = sqlx::query_as::<_, (i32, i64)>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        let mut counts = HashMap::new();
        for (style_id, count) in rows {
            counts.insert(style_id, count as usize);
        }

        Ok(counts)
    }

    async fn count_by_articulation(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
        let where_clause = self.build_where_clause(filters, Some("articulation"));

        let query = format!(
            "SELECT fa.articulation_id, COUNT(DISTINCT f.id) as count
             FROM files f
             JOIN midi_file_articulations fa ON f.id = fa.file_id
             WHERE {}
             GROUP BY fa.articulation_id",
            where_clause
        );

        let rows = sqlx::query_as::<_, (i32, i64)>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        let mut counts = HashMap::new();
        for (articulation_id, count) in rows {
            counts.insert(articulation_id, count as usize);
        }

        Ok(counts)
    }

    /// Count files by BPM range
    async fn count_by_bpm_range(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
        let where_clause = self.build_where_clause(filters, Some("bpm"));

        let query = format!(
            "SELECT f.bpm_range_id, COUNT(DISTINCT f.id) as count
             FROM files f
             WHERE {} AND f.bpm_range_id IS NOT NULL
             GROUP BY f.bpm_range_id",
            where_clause
        );

        let rows = sqlx::query_as::<_, (i32, i64)>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        let mut counts = HashMap::new();
        for (bpm_range_id, count) in rows {
            counts.insert(bpm_range_id, count as usize);
        }

        Ok(counts)
    }

    /// Count files by key signature
    async fn count_by_key(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
        let where_clause = self.build_where_clause(filters, Some("key"));

        let query = format!(
            "SELECT f.key_id, COUNT(DISTINCT f.id) as count
             FROM files f
             WHERE {} AND f.key_id IS NOT NULL
             GROUP BY f.key_id",
            where_clause
        );

        let rows = sqlx::query_as::<_, (i32, i64)>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        let mut counts = HashMap::new();
        for (key_id, count) in rows {
            counts.insert(key_id, count as usize);
        }

        Ok(counts)
    }

    /// Count files by MIDI channel
    async fn count_by_channel(&self, filters: &Vip3Filters) -> DbResult<HashMap<u8, usize>> {
        let where_clause = self.build_where_clause(filters, Some("channels"));

        let query = format!(
            "SELECT f.channel_count, COUNT(DISTINCT f.id) as count
             FROM files f
             WHERE {} AND f.channel_count IS NOT NULL
             GROUP BY f.channel_count",
            where_clause
        );

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        let mut counts = HashMap::new();
        for row in rows {
            // Channel count is i16 (smallint)
            let channel_count: i16 = row.try_get("channel_count").unwrap_or(0);
            let count: i64 = row.try_get("count").unwrap_or(0);
            if channel_count > 0 && channel_count <= 16 {
                counts.insert(channel_count as u8, count as usize);
            }
        }

        Ok(counts)
    }

    /// Count total files matching all current filters
    async fn count_total_matches(&self, filters: &Vip3Filters) -> DbResult<usize> {
        let where_clause = self.build_where_clause(filters, None);

        let query = format!("SELECT COUNT(DISTINCT f.id) FROM files f WHERE {}", where_clause);

        let count: i64 = sqlx::query_scalar(&query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        Ok(count as usize)
    }
}
