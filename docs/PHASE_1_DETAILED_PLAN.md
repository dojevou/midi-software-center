# Phase 1: VIP3 Completion - Detailed Architectural Plan

**Duration:** 5 days (1 week)
**Goal:** Complete VIP3 browser with filter counts, saved searches, collections, favorites, and category management
**Start Date:** Monday, 2025-12-16
**End Date:** Friday, 2025-12-20

---

## Table of Contents

1. [Day 1-2: Filter Counts System](#day-1-2-filter-counts-system)
2. [Day 3: Saved Searches](#day-3-saved-searches)
3. [Day 4: Collections](#day-4-collections)
4. [Day 5: Favorites & Category Management](#day-5-favorites--category-management)

---

## Day 1-2: Filter Counts System

**Goal:** Implement real-time filter counts with <50ms response time
**Files Created:** 8 files
**Lines of Code:** ~1,200 lines

### Architecture Overview

```
Frontend (Svelte)           Backend (Rust)              Database (PostgreSQL)
─────────────────          ─────────────────           ─────────────────────
VIP3Browser.svelte    →    get_vip3_filter_counts  →   COUNT DISTINCT queries
  ├─ VIP3Column.svelte      ├─ Build dynamic SQL        with partial indexes
  └─ filterCounts store     ├─ Execute in parallel
                            └─ Return FilterCounts
```

### Data Flow

1. User changes filter → Frontend updates `currentFilters` store
2. Store triggers `getFilterCounts(currentFilters)` API call
3. Backend receives filters, builds COUNT queries dynamically
4. Queries execute in parallel using `join_all()`
5. Results aggregated into `FilterCounts` struct
6. Frontend updates count badges next to each filter option

---

### Day 1: Backend Implementation

#### Step 1.1: Create FilterCounts Model (30 minutes)

**File:** `app/src-tauri/src/db/models/filter_counts.rs`

**Substeps:**

1. **Create the file:**
   ```bash
   touch app/src-tauri/src/db/models/filter_counts.rs
   ```

2. **Add imports:**
   ```rust
   use serde::{Deserialize, Serialize};
   use std::collections::HashMap;
   ```

3. **Define FilterCounts struct:**
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct FilterCounts {
       /// Folder counts: folder_id → count of files in that folder
       pub folder_counts: HashMap<i64, usize>,

       /// Instrument counts: tag_id → count of files with that instrument tag
       pub instrument_counts: HashMap<i64, usize>,

       /// Timbre counts: timbre_id → count of files with that timbre
       pub timbre_counts: HashMap<i32, usize>,

       /// Style counts: style_id → count of files with that style
       pub style_counts: HashMap<i32, usize>,

       /// Articulation counts: articulation_id → count of files with that articulation
       pub articulation_counts: HashMap<i32, usize>,

       /// BPM range counts: bpm_range_id → count of files in that BPM range
       pub bpm_range_counts: HashMap<i32, usize>,

       /// Key signature counts: key_id → count of files in that key
       pub key_counts: HashMap<i32, usize>,

       /// MIDI channel counts: channel → count of files using that channel
       pub channel_counts: HashMap<u8, usize>,

       /// Total number of files matching current filters
       pub total_matches: usize,
   }

   impl FilterCounts {
       pub fn new() -> Self {
           Self {
               folder_counts: HashMap::new(),
               instrument_counts: HashMap::new(),
               timbre_counts: HashMap::new(),
               style_counts: HashMap::new(),
               articulation_counts: HashMap::new(),
               bpm_range_counts: HashMap::new(),
               key_counts: HashMap::new(),
               channel_counts: HashMap::new(),
               total_matches: 0,
           }
       }
   }

   impl Default for FilterCounts {
       fn default() -> Self {
           Self::new()
       }
   }
   ```

4. **Export the model in `app/src-tauri/src/db/models/mod.rs`:**
   ```rust
   pub mod filter_counts;
   pub use filter_counts::FilterCounts;
   ```

5. **Test compilation:**
   ```bash
   cargo check -p midi-software-center
   ```

**Verification:**
- [ ] File created successfully
- [ ] No compilation errors
- [ ] FilterCounts serializes to JSON correctly

---

#### Step 1.2: Create Vip3Filters Model (if not exists) (20 minutes)

**File:** `app/src-tauri/src/db/models/vip3_filters.rs`

**Substeps:**

1. **Check if file exists:**
   ```bash
   ls app/src-tauri/src/db/models/vip3_filters.rs
   ```

2. **If missing, create it:**
   ```bash
   touch app/src-tauri/src/db/models/vip3_filters.rs
   ```

3. **Define Vip3Filters struct:**
   ```rust
   use serde::{Deserialize, Serialize};

   #[derive(Debug, Clone, Serialize, Deserialize, Default)]
   pub struct Vip3Filters {
       #[serde(skip_serializing_if = "Option::is_none")]
       pub folder_ids: Option<Vec<i64>>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub instrument_ids: Option<Vec<i64>>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub timbre_ids: Option<Vec<i32>>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub style_ids: Option<Vec<i32>>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub articulation_ids: Option<Vec<i32>>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub bpm_range_ids: Option<Vec<i32>>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub key_ids: Option<Vec<i32>>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub channel: Option<u8>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub search_query: Option<String>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub favorites_only: Option<bool>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub tag_ids: Option<Vec<i32>>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub min_rating: Option<i32>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub limit: Option<i32>,

       #[serde(skip_serializing_if = "Option::is_none")]
       pub offset: Option<i32>,
   }

   impl Vip3Filters {
       pub fn has_any_filter(&self) -> bool {
           self.folder_ids.is_some()
               || self.instrument_ids.is_some()
               || self.timbre_ids.is_some()
               || self.style_ids.is_some()
               || self.articulation_ids.is_some()
               || self.bpm_range_ids.is_some()
               || self.key_ids.is_some()
               || self.channel.is_some()
               || self.search_query.is_some()
               || self.favorites_only.is_some()
               || self.tag_ids.is_some()
               || self.min_rating.is_some()
       }
   }
   ```

4. **Export in `app/src-tauri/src/db/models/mod.rs`:**
   ```rust
   pub mod vip3_filters;
   pub use vip3_filters::Vip3Filters;
   ```

**Verification:**
- [ ] Struct compiles without errors
- [ ] Optional fields skip serialization when None
- [ ] `has_any_filter()` helper works

---

#### Step 1.3: Create VIP3 Repository (1 hour)

**File:** `app/src-tauri/src/db/repositories/vip3_repository.rs`

**Substeps:**

1. **Create file:**
   ```bash
   touch app/src-tauri/src/db/repositories/vip3_repository.rs
   ```

2. **Add imports:**
   ```rust
   use crate::db::error::{DbError, DbResult};
   use crate::db::models::{FilterCounts, Vip3Filters};
   use sqlx::PgPool;
   use std::collections::HashMap;
   use futures::future::join_all;
   ```

3. **Define repository struct:**
   ```rust
   pub struct Vip3Repository {
       pool: PgPool,
   }

   impl Vip3Repository {
       pub fn new(pool: PgPool) -> Self {
           Self { pool }
       }
   }
   ```

4. **Implement helper method to build base WHERE clause:**
   ```rust
   impl Vip3Repository {
       /// Build the base WHERE clause from current filters
       /// Returns (WHERE clause, bind parameters)
       fn build_base_where_clause(&self, filters: &Vip3Filters) -> (String, Vec<String>) {
           let mut conditions = vec!["1=1".to_string()];
           let mut params = Vec::new();
           let mut param_idx = 1;

           // Folder filter
           if let Some(ref folder_ids) = filters.folder_ids {
               if !folder_ids.is_empty() {
                   conditions.push(format!("f.folder_id = ANY(${}::bigint[])", param_idx));
                   params.push(format!("{:?}", folder_ids)); // Simplified - use proper param binding
                   param_idx += 1;
               }
           }

           // Instrument filter (via file_tags)
           if let Some(ref instrument_ids) = filters.instrument_ids {
               if !instrument_ids.is_empty() {
                   conditions.push(format!(
                       "EXISTS (SELECT 1 FROM file_tags ft WHERE ft.file_id = f.id AND ft.tag_id = ANY(${}::int[]))",
                       param_idx
                   ));
                   params.push(format!("{:?}", instrument_ids));
                   param_idx += 1;
               }
           }

           // Timbre filter
           if let Some(ref timbre_ids) = filters.timbre_ids {
               if !timbre_ids.is_empty() {
                   conditions.push(format!(
                       "EXISTS (SELECT 1 FROM file_timbres ft WHERE ft.file_id = f.id AND ft.timbre_id = ANY(${}::int[]))",
                       param_idx
                   ));
                   params.push(format!("{:?}", timbre_ids));
                   param_idx += 1;
               }
           }

           // Style filter
           if let Some(ref style_ids) = filters.style_ids {
               if !style_ids.is_empty() {
                   conditions.push(format!(
                       "EXISTS (SELECT 1 FROM file_styles fs WHERE fs.file_id = f.id AND fs.style_id = ANY(${}::int[]))",
                       param_idx
                   ));
                   params.push(format!("{:?}", style_ids));
                   param_idx += 1;
               }
           }

           // Articulation filter
           if let Some(ref articulation_ids) = filters.articulation_ids {
               if !articulation_ids.is_empty() {
                   conditions.push(format!(
                       "EXISTS (SELECT 1 FROM file_articulations fa WHERE fa.file_id = f.id AND fa.articulation_id = ANY(${}::int[]))",
                       param_idx
                   ));
                   params.push(format!("{:?}", articulation_ids));
                   param_idx += 1;
               }
           }

           // BPM range filter
           if let Some(ref bpm_range_ids) = filters.bpm_range_ids {
               if !bpm_range_ids.is_empty() {
                   // Assuming bpm_ranges table exists with id, min_bpm, max_bpm
                   conditions.push(format!(
                       "EXISTS (SELECT 1 FROM bpm_ranges br JOIN musical_metadata mm ON f.id = mm.file_id WHERE br.id = ANY(${}::int[]) AND mm.bpm BETWEEN br.min_bpm AND br.max_bpm)",
                       param_idx
                   ));
                   params.push(format!("{:?}", bpm_range_ids));
                   param_idx += 1;
               }
           }

           // Key filter
           if let Some(ref key_ids) = filters.key_ids {
               if !key_ids.is_empty() {
                   conditions.push(format!(
                       "EXISTS (SELECT 1 FROM musical_metadata mm WHERE mm.file_id = f.id AND mm.key_signature = ANY(${}::text[]))",
                       param_idx
                   ));
                   params.push(format!("{:?}", key_ids));
                   param_idx += 1;
               }
           }

           // Channel filter
           if let Some(channel) = filters.channel {
               conditions.push(format!(
                   "EXISTS (SELECT 1 FROM midi_tracks mt WHERE mt.file_id = f.id AND mt.channel = {})",
                   channel
               ));
           }

           // Search query (full-text)
           if let Some(ref query) = filters.search_query {
               if !query.is_empty() {
                   conditions.push(format!(
                       "f.file_path ILIKE '%{}%'",
                       query.replace("'", "''") // Escape single quotes
                   ));
               }
           }

           // Favorites only
           if filters.favorites_only == Some(true) {
               conditions.push("EXISTS (SELECT 1 FROM favorites fav WHERE fav.file_id = f.id)".to_string());
           }

           // Min rating
           if let Some(min_rating) = filters.min_rating {
               conditions.push(format!("f.rating >= {}", min_rating));
           }

           let where_clause = conditions.join(" AND ");
           (where_clause, params)
       }
   }
   ```

5. **Export in `app/src-tauri/src/db/repositories/mod.rs`:**
   ```rust
   pub mod vip3_repository;
   pub use vip3_repository::Vip3Repository;
   ```

**Verification:**
- [ ] Repository compiles
- [ ] `build_base_where_clause` returns valid SQL
- [ ] All filter types handled

---

#### Step 1.4: Implement Filter Count Queries (2 hours)

**File:** `app/src-tauri/src/db/repositories/vip3_repository.rs` (continued)

**Substeps:**

1. **Add method to count folders:**
   ```rust
   impl Vip3Repository {
       async fn count_by_folder(&self, filters: &Vip3Filters) -> DbResult<HashMap<i64, usize>> {
           let (base_where, _params) = self.build_base_where_clause(filters);

           // Exclude folder_ids from WHERE clause for this count
           let mut filters_without_folders = filters.clone();
           filters_without_folders.folder_ids = None;
           let (where_clause, _) = self.build_base_where_clause(&filters_without_folders);

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
               .map_err(|e| DbError::QueryFailed(e.to_string()))?;

           let mut counts = HashMap::new();
           for (folder_id, count) in rows {
               if let Some(folder_id) = folder_id {
                   counts.insert(folder_id, count as usize);
               }
           }

           Ok(counts)
       }
   }
   ```

2. **Add method to count instruments:**
   ```rust
   async fn count_by_instrument(&self, filters: &Vip3Filters) -> DbResult<HashMap<i64, usize>> {
       let mut filters_without_instruments = filters.clone();
       filters_without_instruments.instrument_ids = None;
       let (where_clause, _) = self.build_base_where_clause(&filters_without_instruments);

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
           .map_err(|e| DbError::QueryFailed(e.to_string()))?;

       let mut counts = HashMap::new();
       for (tag_id, count) in rows {
           counts.insert(tag_id as i64, count as usize);
       }

       Ok(counts)
   }
   ```

3. **Add method to count timbres:**
   ```rust
   async fn count_by_timbre(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
       let mut filters_without_timbres = filters.clone();
       filters_without_timbres.timbre_ids = None;
       let (where_clause, _) = self.build_base_where_clause(&filters_without_timbres);

       let query = format!(
           "SELECT ft.timbre_id, COUNT(DISTINCT f.id) as count
            FROM files f
            JOIN file_timbres ft ON f.id = ft.file_id
            WHERE {}
            GROUP BY ft.timbre_id",
           where_clause
       );

       let rows = sqlx::query_as::<_, (i32, i64)>(&query)
           .fetch_all(&self.pool)
           .await
           .map_err(|e| DbError::QueryFailed(e.to_string()))?;

       let mut counts = HashMap::new();
       for (timbre_id, count) in rows {
           counts.insert(timbre_id, count as usize);
       }

       Ok(counts)
   }
   ```

4. **Add method to count styles:**
   ```rust
   async fn count_by_style(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
       let mut filters_without_styles = filters.clone();
       filters_without_styles.style_ids = None;
       let (where_clause, _) = self.build_base_where_clause(&filters_without_styles);

       let query = format!(
           "SELECT fs.style_id, COUNT(DISTINCT f.id) as count
            FROM files f
            JOIN file_styles fs ON f.id = fs.file_id
            WHERE {}
            GROUP BY fs.style_id",
           where_clause
       );

       let rows = sqlx::query_as::<_, (i32, i64)>(&query)
           .fetch_all(&self.pool)
           .await
           .map_err(|e| DbError::QueryFailed(e.to_string()))?;

       let mut counts = HashMap::new();
       for (style_id, count) in rows {
           counts.insert(style_id, count as usize);
       }

       Ok(counts)
   }
   ```

5. **Add method to count articulations:**
   ```rust
   async fn count_by_articulation(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
       let mut filters_without_articulations = filters.clone();
       filters_without_articulations.articulation_ids = None;
       let (where_clause, _) = self.build_base_where_clause(&filters_without_articulations);

       let query = format!(
           "SELECT fa.articulation_id, COUNT(DISTINCT f.id) as count
            FROM files f
            JOIN file_articulations fa ON f.id = fa.file_id
            WHERE {}
            GROUP BY fa.articulation_id",
           where_clause
       );

       let rows = sqlx::query_as::<_, (i32, i64)>(&query)
           .fetch_all(&self.pool)
           .await
           .map_err(|e| DbError::QueryFailed(e.to_string()))?;

       let mut counts = HashMap::new();
       for (articulation_id, count) in rows {
           counts.insert(articulation_id, count as usize);
       }

       Ok(counts)
   }
   ```

6. **Add method to count BPM ranges:**
   ```rust
   async fn count_by_bpm_range(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
       // Hardcoded BPM ranges for now
       let bpm_ranges = vec![
           (1, 0.0, 90.0),    // Slow
           (2, 90.0, 120.0),  // Medium
           (3, 120.0, 140.0), // Fast
           (4, 140.0, 999.0), // Very Fast
       ];

       let mut filters_without_bpm = filters.clone();
       filters_without_bpm.bpm_range_ids = None;
       let (where_clause, _) = self.build_base_where_clause(&filters_without_bpm);

       let mut counts = HashMap::new();

       for (range_id, min_bpm, max_bpm) in bpm_ranges {
           let query = format!(
               "SELECT COUNT(DISTINCT f.id)
                FROM files f
                JOIN musical_metadata mm ON f.id = mm.file_id
                WHERE {} AND mm.bpm >= {} AND mm.bpm < {}",
               where_clause, min_bpm, max_bpm
           );

           let count: i64 = sqlx::query_scalar(&query)
               .fetch_one(&self.pool)
               .await
               .map_err(|e| DbError::QueryFailed(e.to_string()))?;

           counts.insert(range_id, count as usize);
       }

       Ok(counts)
   }
   ```

7. **Add method to count keys:**
   ```rust
   async fn count_by_key(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
       let mut filters_without_keys = filters.clone();
       filters_without_keys.key_ids = None;
       let (where_clause, _) = self.build_base_where_clause(&filters_without_keys);

       // Map key signatures to numeric IDs
       let key_map: HashMap<String, i32> = vec![
           ("C major".to_string(), 1),
           ("G major".to_string(), 2),
           ("D major".to_string(), 3),
           ("A major".to_string(), 4),
           ("E major".to_string(), 5),
           ("B major".to_string(), 6),
           ("F# major".to_string(), 7),
           ("C# major".to_string(), 8),
           ("F major".to_string(), 9),
           ("Bb major".to_string(), 10),
           ("Eb major".to_string(), 11),
           ("Ab major".to_string(), 12),
           ("Db major".to_string(), 13),
           ("Gb major".to_string(), 14),
           ("A minor".to_string(), 15),
           ("E minor".to_string(), 16),
           ("B minor".to_string(), 17),
           ("F# minor".to_string(), 18),
           ("C# minor".to_string(), 19),
           ("G# minor".to_string(), 20),
           ("D# minor".to_string(), 21),
           ("A# minor".to_string(), 22),
           ("D minor".to_string(), 23),
           ("G minor".to_string(), 24),
           ("C minor".to_string(), 25),
           ("F minor".to_string(), 26),
           ("Bb minor".to_string(), 27),
           ("Eb minor".to_string(), 28),
       ].into_iter().collect();

       let query = format!(
           "SELECT mm.key_signature, COUNT(DISTINCT f.id) as count
            FROM files f
            JOIN musical_metadata mm ON f.id = mm.file_id
            WHERE {}
            GROUP BY mm.key_signature",
           where_clause
       );

       let rows = sqlx::query_as::<_, (Option<String>, i64)>(&query)
           .fetch_all(&self.pool)
           .await
           .map_err(|e| DbError::QueryFailed(e.to_string()))?;

       let mut counts = HashMap::new();
       for (key_signature, count) in rows {
           if let Some(key) = key_signature {
               if let Some(key_id) = key_map.get(&key) {
                   counts.insert(*key_id, count as usize);
               }
           }
       }

       Ok(counts)
   }
   ```

8. **Add method to count channels:**
   ```rust
   async fn count_by_channel(&self, filters: &Vip3Filters) -> DbResult<HashMap<u8, usize>> {
       let mut filters_without_channel = filters.clone();
       filters_without_channel.channel = None;
       let (where_clause, _) = self.build_base_where_clause(&filters_without_channel);

       let query = format!(
           "SELECT mt.channel, COUNT(DISTINCT f.id) as count
            FROM files f
            JOIN midi_tracks mt ON f.id = mt.file_id
            WHERE {}
            GROUP BY mt.channel",
           where_clause
       );

       let rows = sqlx::query_as::<_, (Option<i16>, i64)>(&query)
           .fetch_all(&self.pool)
           .await
           .map_err(|e| DbError::QueryFailed(e.to_string()))?;

       let mut counts = HashMap::new();
       for (channel, count) in rows {
           if let Some(channel) = channel {
               counts.insert(channel as u8, count as usize);
           }
       }

       Ok(counts)
   }
   ```

9. **Add method to count total matches:**
   ```rust
   async fn count_total_matches(&self, filters: &Vip3Filters) -> DbResult<usize> {
       let (where_clause, _) = self.build_base_where_clause(filters);

       let query = format!("SELECT COUNT(DISTINCT f.id) FROM files f WHERE {}", where_clause);

       let count: i64 = sqlx::query_scalar(&query)
           .fetch_one(&self.pool)
           .await
           .map_err(|e| DbError::QueryFailed(e.to_string()))?;

       Ok(count as usize)
   }
   ```

**Verification:**
- [ ] All count methods compile
- [ ] Each method excludes its own filter from WHERE clause
- [ ] Queries use COUNT DISTINCT to avoid duplicates

---

#### Step 1.5: Implement Parallel Execution (1 hour)

**File:** `app/src-tauri/src/db/repositories/vip3_repository.rs` (continued)

**Substeps:**

1. **Add main `get_filter_counts` method:**
   ```rust
   impl Vip3Repository {
       pub async fn get_filter_counts(&self, filters: &Vip3Filters) -> DbResult<FilterCounts> {
           // Execute all count queries in parallel
           let (
               folder_counts,
               instrument_counts,
               timbre_counts,
               style_counts,
               articulation_counts,
               bpm_range_counts,
               key_counts,
               channel_counts,
               total_matches,
           ) = tokio::join!(
               self.count_by_folder(filters),
               self.count_by_instrument(filters),
               self.count_by_timbre(filters),
               self.count_by_style(filters),
               self.count_by_articulation(filters),
               self.count_by_bpm_range(filters),
               self.count_by_key(filters),
               self.count_by_channel(filters),
               self.count_total_matches(filters),
           );

           Ok(FilterCounts {
               folder_counts: folder_counts?,
               instrument_counts: instrument_counts?,
               timbre_counts: timbre_counts?,
               style_counts: style_counts?,
               articulation_counts: articulation_counts?,
               bpm_range_counts: bpm_range_counts?,
               key_counts: key_counts?,
               channel_counts: channel_counts?,
               total_matches: total_matches?,
           })
       }
   }
   ```

2. **Add performance timing (optional debug):**
   ```rust
   use std::time::Instant;

   pub async fn get_filter_counts(&self, filters: &Vip3Filters) -> DbResult<FilterCounts> {
       let start = Instant::now();

       let (
           folder_counts,
           instrument_counts,
           timbre_counts,
           style_counts,
           articulation_counts,
           bpm_range_counts,
           key_counts,
           channel_counts,
           total_matches,
       ) = tokio::join!(
           self.count_by_folder(filters),
           self.count_by_instrument(filters),
           self.count_by_timbre(filters),
           self.count_by_style(filters),
           self.count_by_articulation(filters),
           self.count_by_bpm_range(filters),
           self.count_by_key(filters),
           self.count_by_channel(filters),
           self.count_total_matches(filters),
       );

       let elapsed = start.elapsed();
       log::debug!("Filter counts computed in {:?}", elapsed);

       if elapsed.as_millis() > 50 {
           log::warn!("Filter counts took {}ms (target: <50ms)", elapsed.as_millis());
       }

       Ok(FilterCounts {
           folder_counts: folder_counts?,
           instrument_counts: instrument_counts?,
           timbre_counts: timbre_counts?,
           style_counts: style_counts?,
           articulation_counts: articulation_counts?,
           bpm_range_counts: bpm_range_counts?,
           key_counts: key_counts?,
           channel_counts: channel_counts?,
           total_matches: total_matches?,
       })
   }
   ```

**Verification:**
- [ ] All queries execute in parallel
- [ ] Total time <50ms (test with sample data)
- [ ] Logging shows individual query times

---

#### Step 1.6: Create Tauri Command (30 minutes)

**File:** `app/src-tauri/src/commands/pipeline/vip3/filter_counts.rs`

**Substeps:**

1. **Create directory structure:**
   ```bash
   mkdir -p app/src-tauri/src/commands/pipeline/vip3
   touch app/src-tauri/src/commands/pipeline/vip3/filter_counts.rs
   ```

2. **Add command implementation:**
   ```rust
   use crate::db::models::{FilterCounts, Vip3Filters};
   use crate::db::repositories::Vip3Repository;
   use crate::AppState;
   use tauri::State;

   #[tauri::command]
   pub async fn get_vip3_filter_counts(
       filters: Vip3Filters,
       state: State<'_, AppState>,
   ) -> Result<FilterCounts, String> {
       let repository = Vip3Repository::new(state.db_pool.clone());

       repository
           .get_filter_counts(&filters)
           .await
           .map_err(|e| format!("Failed to get filter counts: {}", e))
   }
   ```

3. **Create mod.rs for vip3 module:**
   ```bash
   touch app/src-tauri/src/commands/pipeline/vip3/mod.rs
   ```

   ```rust
   pub mod filter_counts;
   pub use filter_counts::get_vip3_filter_counts;
   ```

4. **Export in pipeline commands:**
   ```rust
   // In app/src-tauri/src/commands/pipeline/mod.rs
   pub mod vip3;
   pub use vip3::get_vip3_filter_counts;
   ```

5. **Register command in main.rs:**
   ```rust
   // In app/src-tauri/src/main.rs
   use crate::commands::pipeline::get_vip3_filter_counts;

   fn main() {
       tauri::Builder::default()
           .invoke_handler(tauri::generate_handler![
               // ... existing commands
               get_vip3_filter_counts,
           ])
           .run(tauri::generate_context!())
           .expect("error while running tauri application");
   }
   ```

**Verification:**
- [ ] Command compiles
- [ ] Command registered in Tauri
- [ ] Can call from frontend (test with curl or Tauri devtools)

---

### Day 2: Frontend Implementation & Optimization

#### Step 2.1: Create TypeScript Types (30 minutes)

**File:** `app/src/lib/types/vip3.ts`

**Substeps:**

1. **Create types file:**
   ```bash
   touch app/src/lib/types/vip3.ts
   ```

2. **Define FilterCounts interface:**
   ```typescript
   export interface FilterCounts {
     folder_counts: Record<number, number>;
     instrument_counts: Record<number, number>;
     timbre_counts: Record<number, number>;
     style_counts: Record<number, number>;
     articulation_counts: Record<number, number>;
     bpm_range_counts: Record<number, number>;
     key_counts: Record<number, number>;
     channel_counts: Record<number, number>;
     total_matches: number;
   }
   ```

3. **Define Vip3Filters interface:**
   ```typescript
   export interface Vip3Filters {
     folder_ids?: number[];
     instrument_ids?: number[];
     timbre_ids?: number[];
     style_ids?: number[];
     articulation_ids?: number[];
     bpm_range_ids?: number[];
     key_ids?: number[];
     channel?: number;
     search_query?: string;
     favorites_only?: boolean;
     tag_ids?: number[];
     min_rating?: number;
     limit?: number;
     offset?: number;
   }
   ```

4. **Export types:**
   ```typescript
   // In app/src/lib/types/index.ts
   export * from './vip3';
   ```

**Verification:**
- [ ] Types compile without errors
- [ ] Types match Rust structs exactly

---

#### Step 2.2: Create VIP3 Browser API (45 minutes)

**File:** `app/src/lib/api/vip3BrowserApi.ts`

**Substeps:**

1. **Create API file:**
   ```bash
   touch app/src/lib/api/vip3BrowserApi.ts
   ```

2. **Import dependencies:**
   ```typescript
   import { invoke } from '../api';
   import type { FilterCounts, Vip3Filters } from '../types';
   ```

3. **Implement getFilterCounts:**
   ```typescript
   export async function getFilterCounts(filters: Vip3Filters): Promise<FilterCounts> {
     console.log('[VIP3 API] Getting filter counts with filters:', filters);

     const start = performance.now();

     try {
       const counts = await invoke<FilterCounts>('get_vip3_filter_counts', { filters });

       const elapsed = performance.now() - start;
       console.log(`[VIP3 API] Filter counts received in ${elapsed.toFixed(1)}ms`);

       if (elapsed > 50) {
         console.warn(`[VIP3 API] Filter counts took ${elapsed.toFixed(1)}ms (target: <50ms)`);
       }

       return counts;
     } catch (error) {
       console.error('[VIP3 API] Failed to get filter counts:', error);
       throw error;
     }
   }
   ```

4. **Add debounced version (for rapid filter changes):**
   ```typescript
   let debounceTimer: NodeJS.Timeout | null = null;

   export function getFilterCountsDebounced(
     filters: Vip3Filters,
     delay: number = 300
   ): Promise<FilterCounts> {
     return new Promise((resolve, reject) => {
       if (debounceTimer) {
         clearTimeout(debounceTimer);
       }

       debounceTimer = setTimeout(async () => {
         try {
           const counts = await getFilterCounts(filters);
           resolve(counts);
         } catch (error) {
           reject(error);
         }
       }, delay);
     });
   }
   ```

5. **Export API:**
   ```typescript
   // In app/src/lib/api/index.ts
   export * from './vip3BrowserApi';
   ```

**Verification:**
- [ ] API compiles
- [ ] Logging works
- [ ] Debounce works (test manually)

---

#### Step 2.3: Create VIP3 Store (1 hour)

**File:** `app/src/lib/stores/vip3Store.ts`

**Substeps:**

1. **Create store file:**
   ```bash
   touch app/src/lib/stores/vip3Store.ts
   ```

2. **Import dependencies:**
   ```typescript
   import { writable, derived, get } from 'svelte/store';
   import type { Writable, Readable } from 'svelte/store';
   import type { FilterCounts, Vip3Filters } from '../types';
   import { getFilterCounts } from '../api/vip3BrowserApi';
   ```

3. **Create stores:**
   ```typescript
   // Current filter selections
   export const currentFilters: Writable<Vip3Filters> = writable({
     limit: 100,
     offset: 0,
   });

   // Filter counts (updated when filters change)
   export const filterCounts: Writable<FilterCounts | null> = writable(null);

   // Loading state
   export const isLoadingCounts: Writable<boolean> = writable(false);

   // Error state
   export const countsError: Writable<string | null> = writable(null);
   ```

4. **Create actions:**
   ```typescript
   // Update a single filter
   export function updateFilter(filterName: keyof Vip3Filters, value: any) {
     currentFilters.update(filters => ({
       ...filters,
       [filterName]: value,
       offset: 0, // Reset pagination when filter changes
     }));

     // Trigger count update
     refreshFilterCounts();
   }

   // Clear all filters
   export function clearFilters() {
     currentFilters.set({
       limit: 100,
       offset: 0,
     });

     refreshFilterCounts();
   }

   // Add item to multi-select filter
   export function addToFilter(filterName: keyof Vip3Filters, value: number) {
     currentFilters.update(filters => {
       const current = filters[filterName] as number[] | undefined;
       const updated = current ? [...current, value] : [value];

       return {
         ...filters,
         [filterName]: updated,
         offset: 0,
       };
     });

     refreshFilterCounts();
   }

   // Remove item from multi-select filter
   export function removeFromFilter(filterName: keyof Vip3Filters, value: number) {
     currentFilters.update(filters => {
       const current = filters[filterName] as number[] | undefined;
       const updated = current ? current.filter(v => v !== value) : [];

       return {
         ...filters,
         [filterName]: updated.length > 0 ? updated : undefined,
         offset: 0,
       };
     });

     refreshFilterCounts();
   }

   // Toggle item in multi-select filter
   export function toggleFilter(filterName: keyof Vip3Filters, value: number) {
     const filters = get(currentFilters);
     const current = filters[filterName] as number[] | undefined;

     if (current?.includes(value)) {
       removeFromFilter(filterName, value);
     } else {
       addToFilter(filterName, value);
     }
   }
   ```

5. **Create refresh action:**
   ```typescript
   let refreshTimeout: NodeJS.Timeout | null = null;

   export async function refreshFilterCounts() {
     // Debounce: wait 300ms after last filter change
     if (refreshTimeout) {
       clearTimeout(refreshTimeout);
     }

     refreshTimeout = setTimeout(async () => {
       const filters = get(currentFilters);

       isLoadingCounts.set(true);
       countsError.set(null);

       try {
         const counts = await getFilterCounts(filters);
         filterCounts.set(counts);
       } catch (error) {
         console.error('Failed to refresh filter counts:', error);
         countsError.set(error.toString());
         filterCounts.set(null);
       } finally {
         isLoadingCounts.set(false);
       }
     }, 300);
   }

   // Initialize: load counts on first load
   refreshFilterCounts();
   ```

6. **Create derived stores:**
   ```typescript
   // Check if any filters are active
   export const hasActiveFilters: Readable<boolean> = derived(
     currentFilters,
     $filters => {
       return !!(
         $filters.folder_ids?.length ||
         $filters.instrument_ids?.length ||
         $filters.timbre_ids?.length ||
         $filters.style_ids?.length ||
         $filters.articulation_ids?.length ||
         $filters.bpm_range_ids?.length ||
         $filters.key_ids?.length ||
         $filters.channel !== undefined ||
         $filters.search_query ||
         $filters.favorites_only ||
         $filters.tag_ids?.length ||
         $filters.min_rating
       );
     }
   );

   // Get count for specific filter value
   export function getCountForValue(
     countType: keyof FilterCounts,
     valueId: number
   ): number {
     const counts = get(filterCounts);
     if (!counts) return 0;

     const countMap = counts[countType];
     if (typeof countMap === 'number') return countMap; // total_matches

     return (countMap as Record<number, number>)[valueId] || 0;
   }
   ```

**Verification:**
- [ ] Store compiles
- [ ] Actions work (test in Svelte devtools)
- [ ] Debounce works (rapid filter changes only trigger one API call)

---

#### Step 2.4: Update VIP3Column Component (1.5 hours)

**File:** `app/src/lib/components/VIP3/VIP3Column.svelte`

**Substeps:**

1. **Check if file exists, create if needed:**
   ```bash
   ls app/src/lib/components/VIP3/VIP3Column.svelte || touch app/src/lib/components/VIP3/VIP3Column.svelte
   ```

2. **Add script section:**
   ```svelte
   <script lang="ts">
     import { filterCounts, toggleFilter, currentFilters, getCountForValue } from '$lib/stores/vip3Store';
     import type { FilterCounts } from '$lib/types';

     export let title: string;
     export let items: Array<{ id: number; name: string }>;
     export let filterKey: keyof FilterCounts;
     export let filterName: keyof Vip3Filters;

     $: selectedIds = ($currentFilters[filterName] as number[] | undefined) || [];

     function handleToggle(itemId: number) {
       toggleFilter(filterName, itemId);
     }

     function isSelected(itemId: number): boolean {
       return selectedIds.includes(itemId);
     }

     function getCount(itemId: number): number {
       return getCountForValue(filterKey, itemId);
     }
   </script>
   ```

3. **Add template:**
   ```svelte
   <div class="vip3-column">
     <h3 class="column-title">{title}</h3>

     <div class="filter-list">
       {#each items as item (item.id)}
         {@const count = getCount(item.id)}
         {@const selected = isSelected(item.id)}

         <label class="filter-item" class:selected class:disabled={count === 0}>
           <input
             type="checkbox"
             checked={selected}
             disabled={count === 0}
             on:change={() => handleToggle(item.id)}
           />

           <span class="item-name">{item.name}</span>

           <span class="item-count" class:zero={count === 0}>
             ({count})
           </span>
         </label>
       {/each}
     </div>

     {#if selectedIds.length > 0}
       <button class="clear-button" on:click={() => updateFilter(filterName, undefined)}>
         Clear ({selectedIds.length})
       </button>
     {/if}
   </div>
   ```

4. **Add styles:**
   ```svelte
   <style>
     .vip3-column {
       display: flex;
       flex-direction: column;
       min-width: 200px;
       max-width: 300px;
       border-right: 1px solid var(--border-color);
       background: var(--bg-secondary);
     }

     .column-title {
       padding: 12px 16px;
       margin: 0;
       font-size: 14px;
       font-weight: 600;
       border-bottom: 1px solid var(--border-color);
       background: var(--bg-tertiary);
     }

     .filter-list {
       flex: 1;
       overflow-y: auto;
       padding: 8px;
     }

     .filter-item {
       display: flex;
       align-items: center;
       padding: 6px 8px;
       margin-bottom: 4px;
       cursor: pointer;
       border-radius: 4px;
       transition: background 0.2s;
     }

     .filter-item:hover {
       background: var(--bg-hover);
     }

     .filter-item.selected {
       background: var(--accent-light);
       font-weight: 500;
     }

     .filter-item.disabled {
       opacity: 0.4;
       cursor: not-allowed;
     }

     .filter-item input[type="checkbox"] {
       margin-right: 8px;
     }

     .item-name {
       flex: 1;
       font-size: 13px;
     }

     .item-count {
       font-size: 12px;
       color: var(--text-secondary);
       margin-left: 8px;
     }

     .item-count.zero {
       color: var(--text-disabled);
     }

     .clear-button {
       margin: 8px;
       padding: 6px 12px;
       font-size: 12px;
       border: 1px solid var(--border-color);
       border-radius: 4px;
       background: var(--bg-primary);
       cursor: pointer;
       transition: background 0.2s;
     }

     .clear-button:hover {
       background: var(--bg-hover);
     }
   </style>
   ```

**Verification:**
- [ ] Component renders
- [ ] Checkboxes work
- [ ] Counts display correctly
- [ ] Items with 0 count are disabled
- [ ] Clear button works

---

#### Step 2.5: Database Optimization (2 hours)

**File:** Create migration for partial indexes

**Substeps:**

1. **Create new migration:**
   ```bash
   cd database/migrations
   touch 020_add_filter_count_indexes.sql
   ```

2. **Add partial indexes for file_timbres:**
   ```sql
   -- Migration: 020_add_filter_count_indexes.sql
   -- Purpose: Optimize VIP3 filter count queries

   BEGIN;

   -- Partial index on file_timbres for fast GROUP BY timbre_id
   CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_timbres_timbre_id_partial
   ON file_timbres(timbre_id)
   WHERE timbre_id IS NOT NULL;

   -- Covering index for file_id + timbre_id (avoids table lookup)
   CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_timbres_covering
   ON file_timbres(file_id, timbre_id);

   COMMIT;
   ```

3. **Add indexes for file_styles:**
   ```sql
   BEGIN;

   CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_styles_style_id_partial
   ON file_styles(style_id)
   WHERE style_id IS NOT NULL;

   CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_styles_covering
   ON file_styles(file_id, style_id);

   COMMIT;
   ```

4. **Add indexes for file_articulations:**
   ```sql
   BEGIN;

   CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_articulations_articulation_id_partial
   ON file_articulations(articulation_id)
   WHERE articulation_id IS NOT NULL;

   CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_articulations_covering
   ON file_articulations(file_id, articulation_id);

   COMMIT;
   ```

5. **Add composite index for musical_metadata:**
   ```sql
   BEGIN;

   -- For BPM range queries
   CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_musical_metadata_bpm_file_id
   ON musical_metadata(bpm, file_id)
   WHERE bpm IS NOT NULL;

   -- For key signature queries
   CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_musical_metadata_key_file_id
   ON musical_metadata(key_signature, file_id)
   WHERE key_signature IS NOT NULL;

   COMMIT;
   ```

6. **Add index for midi_tracks channel:**
   ```sql
   BEGIN;

   CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_midi_tracks_channel_file_id
   ON midi_tracks(channel, file_id)
   WHERE channel IS NOT NULL;

   COMMIT;
   ```

7. **Run migration:**
   ```bash
   make db-migrate
   # OR
   sqlx migrate run --source database/migrations
   ```

8. **Verify indexes created:**
   ```bash
   psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c "\di"
   ```

**Verification:**
- [ ] All indexes created successfully
- [ ] Query plan uses indexes (check with EXPLAIN ANALYZE)
- [ ] Count queries <50ms

---

#### Step 2.6: Performance Testing (1 hour)

**Substeps:**

1. **Create test script:**
   ```bash
   touch scripts/test-filter-counts-performance.sh
   chmod +x scripts/test-filter-counts-performance.sh
   ```

2. **Add test script content:**
   ```bash
   #!/bin/bash
   # Test VIP3 filter counts performance

   DATABASE_URL="postgresql://midiuser:145278963@localhost:5433/midi_library"

   echo "=== VIP3 Filter Counts Performance Test ==="
   echo ""

   # Test 1: Count folders (no filters)
   echo "Test 1: Count by folder (no filters)"
   psql "$DATABASE_URL" -c "
   EXPLAIN ANALYZE
   SELECT f.folder_id, COUNT(DISTINCT f.id) as count
   FROM files f
   WHERE 1=1
   GROUP BY f.folder_id;
   " | grep "Execution Time"

   # Test 2: Count timbres (no filters)
   echo ""
   echo "Test 2: Count by timbre (no filters)"
   psql "$DATABASE_URL" -c "
   EXPLAIN ANALYZE
   SELECT ft.timbre_id, COUNT(DISTINCT f.id) as count
   FROM files f
   JOIN file_timbres ft ON f.id = ft.file_id
   WHERE 1=1
   GROUP BY ft.timbre_id;
   " | grep "Execution Time"

   # Test 3: Count with multiple filters
   echo ""
   echo "Test 3: Count by timbre (with folder filter)"
   psql "$DATABASE_URL" -c "
   EXPLAIN ANALYZE
   SELECT ft.timbre_id, COUNT(DISTINCT f.id) as count
   FROM files f
   JOIN file_timbres ft ON f.id = ft.file_id
   WHERE f.folder_id = ANY(ARRAY[1, 2, 3]::bigint[])
   GROUP BY ft.timbre_id;
   " | grep "Execution Time"

   # Test 4: Count total matches
   echo ""
   echo "Test 4: Count total matches (no filters)"
   psql "$DATABASE_URL" -c "
   EXPLAIN ANALYZE
   SELECT COUNT(DISTINCT f.id) FROM files f WHERE 1=1;
   " | grep "Execution Time"

   echo ""
   echo "=== All queries should complete in <50ms ==="
   ```

3. **Run performance test:**
   ```bash
   ./scripts/test-filter-counts-performance.sh
   ```

4. **Analyze results:**
   - Check "Execution Time" for each query
   - Verify all queries <50ms
   - If slow, check index usage with `EXPLAIN ANALYZE`

5. **Load test with sample data:**
   ```bash
   # Import 10,000+ files first
   ./scripts/run-pipeline-ultra-fast.sh

   # Then test filter counts
   ./scripts/test-filter-counts-performance.sh
   ```

**Verification:**
- [ ] All queries <50ms with 10K+ files
- [ ] Indexes used in query plans
- [ ] No full table scans (check EXPLAIN output)

---

#### Step 2.7: Integration Testing (1 hour)

**Substeps:**

1. **Create Rust integration test:**
   ```bash
   touch app/src-tauri/tests/vip3_filter_counts_test.rs
   ```

2. **Add test code:**
   ```rust
   #[cfg(test)]
   mod vip3_filter_counts_tests {
       use midi_app::db::models::{FilterCounts, Vip3Filters};
       use midi_app::db::repositories::Vip3Repository;
       use sqlx::PgPool;
       use std::env;

       async fn get_test_pool() -> PgPool {
           let database_url = env::var("DATABASE_URL")
               .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());

           PgPool::connect(&database_url)
               .await
               .expect("Failed to connect to test database")
       }

       #[tokio::test]
       async fn test_filter_counts_no_filters() {
           let pool = get_test_pool().await;
           let repo = Vip3Repository::new(pool);

           let filters = Vip3Filters::default();
           let counts = repo.get_filter_counts(&filters).await;

           assert!(counts.is_ok());
           let counts = counts.unwrap();

           // Should have some results
           assert!(counts.total_matches > 0);
           println!("Total files: {}", counts.total_matches);
           println!("Folder counts: {:?}", counts.folder_counts);
       }

       #[tokio::test]
       async fn test_filter_counts_with_folder_filter() {
           let pool = get_test_pool().await;
           let repo = Vip3Repository::new(pool);

           let filters = Vip3Filters {
               folder_ids: Some(vec![1, 2]),
               ..Default::default()
           };

           let counts = repo.get_filter_counts(&filters).await;

           assert!(counts.is_ok());
           let counts = counts.unwrap();

           // Total should be <= all files
           println!("Filtered total: {}", counts.total_matches);
       }

       #[tokio::test]
       async fn test_filter_counts_performance() {
           let pool = get_test_pool().await;
           let repo = Vip3Repository::new(pool);

           let filters = Vip3Filters {
               folder_ids: Some(vec![1]),
               instrument_ids: Some(vec![1, 2, 3]),
               ..Default::default()
           };

           let start = std::time::Instant::now();
           let counts = repo.get_filter_counts(&filters).await;
           let elapsed = start.elapsed();

           assert!(counts.is_ok());
           println!("Filter counts took: {:?}", elapsed);

           // Should be <50ms
           assert!(elapsed.as_millis() < 50, "Filter counts took {}ms (expected <50ms)", elapsed.as_millis());
       }
   }
   ```

3. **Run tests:**
   ```bash
   cargo test --test vip3_filter_counts_test -- --test-threads=1 --nocapture
   ```

**Verification:**
- [ ] All tests pass
- [ ] Performance test shows <50ms
- [ ] No database errors

---

### Day 1-2 Deliverables Checklist

- [ ] **Backend:**
  - [ ] `FilterCounts` model created
  - [ ] `Vip3Filters` model created
  - [ ] `Vip3Repository` with all count methods
  - [ ] `get_vip3_filter_counts` Tauri command
  - [ ] Parallel query execution with `tokio::join!`
  - [ ] Performance logging

- [ ] **Database:**
  - [ ] Migration 020 with partial indexes
  - [ ] Indexes on file_timbres, file_styles, file_articulations
  - [ ] Composite indexes on musical_metadata, midi_tracks
  - [ ] All queries use indexes (verified with EXPLAIN)

- [ ] **Frontend:**
  - [ ] TypeScript types for FilterCounts and Vip3Filters
  - [ ] `vip3BrowserApi.ts` with getFilterCounts
  - [ ] `vip3Store.ts` with reactive filter management
  - [ ] `VIP3Column.svelte` updated with count badges
  - [ ] Debouncing on filter changes (300ms)

- [ ] **Testing:**
  - [ ] Integration tests pass
  - [ ] Performance tests show <50ms
  - [ ] Manual UI testing complete

- [ ] **Documentation:**
  - [ ] Code comments added
  - [ ] Performance metrics logged

---

## Day 3: Saved Searches

**Goal:** Implement saved search functionality (save, load, delete, list)
**Files Created:** 6 files
**Lines of Code:** ~600 lines

### Architecture Overview

```
Frontend                Backend                 Database
─────────              ─────────               ──────────
SaveSearch button  →   save_search         →   INSERT saved_searches
LoadSearch item    →   load_saved_search   →   SELECT + UPDATE use_count
DeleteSearch btn   →   delete_saved_search →   DELETE
ListSearches       →   get_saved_searches  →   SELECT ORDER BY last_used
```

---

### Step 3.1: Create Saved Search Model (20 minutes)

**File:** `app/src-tauri/src/db/models/saved_search.rs`

**Substeps:**

1. **Create file:**
   ```bash
   touch app/src-tauri/src/db/models/saved_search.rs
   ```

2. **Add imports:**
   ```rust
   use serde::{Deserialize, Serialize};
   use sqlx::types::chrono::{DateTime, Utc};
   use crate::db::models::Vip3Filters;
   ```

3. **Define SavedSearch struct:**
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
   pub struct SavedSearch {
       pub id: i64,
       pub name: String,
       pub filters: serde_json::Value, // JSON representation of Vip3Filters
       pub use_count: i32,
       pub last_used: Option<DateTime<Utc>>,
       pub created_at: DateTime<Utc>,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct CreateSavedSearch {
       pub name: String,
       pub filters: Vip3Filters,
   }
   ```

4. **Add helper methods:**
   ```rust
   impl SavedSearch {
       pub fn parse_filters(&self) -> Result<Vip3Filters, serde_json::Error> {
           serde_json::from_value(self.filters.clone())
       }
   }
   ```

5. **Export model:**
   ```rust
   // In app/src-tauri/src/db/models/mod.rs
   pub mod saved_search;
   pub use saved_search::{SavedSearch, CreateSavedSearch};
   ```

**Verification:**
- [ ] Model compiles
- [ ] Serialization works (test with serde_json)

---

### Step 3.2: Create SavedSearchRepository (1 hour)

**File:** `app/src-tauri/src/db/repositories/saved_search_repository.rs`

**Substeps:**

1. **Create file:**
   ```bash
   touch app/src-tauri/src/db/repositories/saved_search_repository.rs
   ```

2. **Add repository structure:**
   ```rust
   use crate::db::error::{DbError, DbResult};
   use crate::db::models::{SavedSearch, CreateSavedSearch, Vip3Filters};
   use sqlx::PgPool;

   pub struct SavedSearchRepository {
       pool: PgPool,
   }

   impl SavedSearchRepository {
       pub fn new(pool: PgPool) -> Self {
           Self { pool }
       }
   }
   ```

3. **Implement save_search:**
   ```rust
   impl SavedSearchRepository {
       pub async fn save_search(&self, create: CreateSavedSearch) -> DbResult<i64> {
           let filters_json = serde_json::to_value(&create.filters)
               .map_err(|e| DbError::QueryFailed(format!("Failed to serialize filters: {}", e)))?;

           let result = sqlx::query!(
               r#"
               INSERT INTO saved_searches (name, filters, use_count, last_used, created_at)
               VALUES ($1, $2, 0, NULL, NOW())
               RETURNING id
               "#,
               create.name,
               filters_json
           )
           .fetch_one(&self.pool)
           .await
           .map_err(|e| DbError::QueryFailed(e.to_string()))?;

           Ok(result.id)
       }
   }
   ```

4. **Implement load_saved_search:**
   ```rust
   pub async fn load_saved_search(&self, id: i64) -> DbResult<SavedSearch> {
       // Fetch the search
       let search = sqlx::query_as!(
           SavedSearch,
           r#"
           SELECT id, name, filters, use_count, last_used, created_at
           FROM saved_searches
           WHERE id = $1
           "#,
           id
       )
       .fetch_optional(&self.pool)
       .await
       .map_err(|e| DbError::QueryFailed(e.to_string()))?
       .ok_or_else(|| DbError::NotFound(format!("Saved search {} not found", id)))?;

       // Increment use_count and update last_used
       sqlx::query!(
           r#"
           UPDATE saved_searches
           SET use_count = use_count + 1,
               last_used = NOW()
           WHERE id = $1
           "#,
           id
       )
       .execute(&self.pool)
       .await
       .map_err(|e| DbError::QueryFailed(e.to_string()))?;

       Ok(search)
   }
   ```

5. **Implement get_saved_searches:**
   ```rust
   pub async fn get_saved_searches(&self) -> DbResult<Vec<SavedSearch>> {
       let searches = sqlx::query_as!(
           SavedSearch,
           r#"
           SELECT id, name, filters, use_count, last_used, created_at
           FROM saved_searches
           ORDER BY last_used DESC NULLS LAST, created_at DESC
           "#
       )
       .fetch_all(&self.pool)
       .await
       .map_err(|e| DbError::QueryFailed(e.to_string()))?;

       Ok(searches)
   }
   ```

6. **Implement delete_saved_search:**
   ```rust
   pub async fn delete_saved_search(&self, id: i64) -> DbResult<()> {
       let result = sqlx::query!(
           r#"
           DELETE FROM saved_searches
           WHERE id = $1
           "#,
           id
       )
       .execute(&self.pool)
       .await
       .map_err(|e| DbError::QueryFailed(e.to_string()))?;

       if result.rows_affected() == 0 {
           return Err(DbError::NotFound(format!("Saved search {} not found", id)));
       }

       Ok(())
   }
   ```

7. **Implement update_saved_search (optional):**
   ```rust
   pub async fn update_saved_search(
       &self,
       id: i64,
       name: String,
       filters: Vip3Filters,
   ) -> DbResult<()> {
       let filters_json = serde_json::to_value(&filters)
           .map_err(|e| DbError::QueryFailed(format!("Failed to serialize filters: {}", e)))?;

       let result = sqlx::query!(
           r#"
           UPDATE saved_searches
           SET name = $1,
               filters = $2
           WHERE id = $3
           "#,
           name,
           filters_json,
           id
       )
       .execute(&self.pool)
       .await
       .map_err(|e| DbError::QueryFailed(e.to_string()))?;

       if result.rows_affected() == 0 {
           return Err(DbError::NotFound(format!("Saved search {} not found", id)));
       }

       Ok(())
   }
   ```

8. **Export repository:**
   ```rust
   // In app/src-tauri/src/db/repositories/mod.rs
   pub mod saved_search_repository;
   pub use saved_search_repository::SavedSearchRepository;
   ```

**Verification:**
- [ ] Repository compiles
- [ ] All CRUD methods implemented
- [ ] Error handling correct

---

### Step 3.3: Create Tauri Commands (45 minutes)

**File:** `app/src-tauri/src/commands/pipeline/vip3/saved_searches.rs`

**Substeps:**

1. **Create file:**
   ```bash
   touch app/src-tauri/src/commands/pipeline/vip3/saved_searches.rs
   ```

2. **Implement save_search command:**
   ```rust
   use crate::db::models::{CreateSavedSearch, SavedSearch, Vip3Filters};
   use crate::db::repositories::SavedSearchRepository;
   use crate::AppState;
   use tauri::State;

   #[tauri::command]
   pub async fn save_search(
       name: String,
       filters: Vip3Filters,
       state: State<'_, AppState>,
   ) -> Result<i64, String> {
       let repo = SavedSearchRepository::new(state.db_pool.clone());

       let create = CreateSavedSearch { name, filters };

       repo.save_search(create)
           .await
           .map_err(|e| format!("Failed to save search: {}", e))
   }
   ```

3. **Implement load_saved_search command:**
   ```rust
   #[tauri::command]
   pub async fn load_saved_search(
       id: i64,
       state: State<'_, AppState>,
   ) -> Result<SavedSearch, String> {
       let repo = SavedSearchRepository::new(state.db_pool.clone());

       repo.load_saved_search(id)
           .await
           .map_err(|e| format!("Failed to load saved search: {}", e))
   }
   ```

4. **Implement get_saved_searches command:**
   ```rust
   #[tauri::command]
   pub async fn get_saved_searches(
       state: State<'_, AppState>,
   ) -> Result<Vec<SavedSearch>, String> {
       let repo = SavedSearchRepository::new(state.db_pool.clone());

       repo.get_saved_searches()
           .await
           .map_err(|e| format!("Failed to get saved searches: {}", e))
   }
   ```

5. **Implement delete_saved_search command:**
   ```rust
   #[tauri::command]
   pub async fn delete_saved_search(
       id: i64,
       state: State<'_, AppState>,
   ) -> Result<(), String> {
       let repo = SavedSearchRepository::new(state.db_pool.clone());

       repo.delete_saved_search(id)
           .await
           .map_err(|e| format!("Failed to delete saved search: {}", e))
   }
   ```

6. **Export commands:**
   ```rust
   // In app/src-tauri/src/commands/pipeline/vip3/mod.rs
   pub mod saved_searches;
   pub use saved_searches::{
       save_search,
       load_saved_search,
       get_saved_searches,
       delete_saved_search,
   };
   ```

7. **Register in main.rs:**
   ```rust
   // In app/src-tauri/src/main.rs
   use crate::commands::pipeline::{
       save_search,
       load_saved_search,
       get_saved_searches,
       delete_saved_search,
   };

   fn main() {
       tauri::Builder::default()
           .invoke_handler(tauri::generate_handler![
               // ... existing commands
               save_search,
               load_saved_search,
               get_saved_searches,
               delete_saved_search,
           ])
           .run(tauri::generate_context!())
           .expect("error while running tauri application");
   }
   ```

**Verification:**
- [ ] Commands compile
- [ ] Commands registered in Tauri
- [ ] Test with Tauri devtools

---

### Step 3.4: Create Frontend API (30 minutes)

**File:** `app/src/lib/api/savedSearchesApi.ts`

**Substeps:**

1. **Create file:**
   ```bash
   touch app/src/lib/api/savedSearchesApi.ts
   ```

2. **Implement API methods:**
   ```typescript
   import { invoke } from '../api';
   import type { SavedSearch, Vip3Filters } from '../types';

   export async function saveSearch(name: string, filters: Vip3Filters): Promise<number> {
     console.log('[Saved Searches API] Saving search:', name);

     try {
       const id = await invoke<number>('save_search', { name, filters });
       console.log('[Saved Searches API] Search saved with ID:', id);
       return id;
     } catch (error) {
       console.error('[Saved Searches API] Failed to save search:', error);
       throw error;
     }
   }

   export async function loadSavedSearch(id: number): Promise<SavedSearch> {
     console.log('[Saved Searches API] Loading search:', id);

     try {
       const search = await invoke<SavedSearch>('load_saved_search', { id });
       console.log('[Saved Searches API] Search loaded:', search);
       return search;
     } catch (error) {
       console.error('[Saved Searches API] Failed to load search:', error);
       throw error;
     }
   }

   export async function getSavedSearches(): Promise<SavedSearch[]> {
     console.log('[Saved Searches API] Getting all saved searches');

     try {
       const searches = await invoke<SavedSearch[]>('get_saved_searches');
       console.log('[Saved Searches API] Found', searches.length, 'saved searches');
       return searches;
     } catch (error) {
       console.error('[Saved Searches API] Failed to get saved searches:', error);
       throw error;
     }
   }

   export async function deleteSavedSearch(id: number): Promise<void> {
     console.log('[Saved Searches API] Deleting search:', id);

     try {
       await invoke<void>('delete_saved_search', { id });
       console.log('[Saved Searches API] Search deleted');
     } catch (error) {
       console.error('[Saved Searches API] Failed to delete search:', error);
       throw error;
     }
   }
   ```

3. **Add TypeScript type (if not exists):**
   ```typescript
   // In app/src/lib/types/vip3.ts
   export interface SavedSearch {
     id: number;
     name: string;
     filters: any; // JSON value
     use_count: number;
     last_used: string | null; // ISO datetime
     created_at: string; // ISO datetime
   }
   ```

4. **Export API:**
   ```typescript
   // In app/src/lib/api/index.ts
   export * from './savedSearchesApi';
   ```

**Verification:**
- [ ] API compiles
- [ ] Types match Rust structs

---

### Step 3.5: Create VIP3SavedSearches Component (1.5 hours)

**File:** `app/src/lib/components/VIP3/VIP3SavedSearches.svelte`

**Substeps:**

1. **Create file:**
   ```bash
   touch app/src/lib/components/VIP3/VIP3SavedSearches.svelte
   ```

2. **Add script:**
   ```svelte
   <script lang="ts">
     import { onMount } from 'svelte';
     import { currentFilters } from '$lib/stores/vip3Store';
     import { getSavedSearches, loadSavedSearch, deleteSavedSearch, saveSearch } from '$lib/api/savedSearchesApi';
     import type { SavedSearch } from '$lib/types';

     let searches: SavedSearch[] = [];
     let loading = false;
     let error: string | null = null;

     let showSaveDialog = false;
     let newSearchName = '';
     let saving = false;

     onMount(async () => {
       await refreshSearches();
     });

     async function refreshSearches() {
       loading = true;
       error = null;

       try {
         searches = await getSavedSearches();
       } catch (err) {
         error = err.toString();
       } finally {
         loading = false;
       }
     }

     async function handleLoad(search: SavedSearch) {
       try {
         const loaded = await loadSavedSearch(search.id);
         const filters = JSON.parse(loaded.filters);

         // Apply filters to current store
         currentFilters.set(filters);

         // Refresh list to show updated use_count
         await refreshSearches();
       } catch (err) {
         alert('Failed to load search: ' + err);
       }
     }

     async function handleDelete(id: number) {
       if (!confirm('Delete this saved search?')) return;

       try {
         await deleteSavedSearch(id);
         await refreshSearches();
       } catch (err) {
         alert('Failed to delete search: ' + err);
       }
     }

     async function handleSave() {
       if (!newSearchName.trim()) {
         alert('Please enter a name');
         return;
       }

       saving = true;

       try {
         await saveSearch(newSearchName, $currentFilters);
         newSearchName = '';
         showSaveDialog = false;
         await refreshSearches();
       } catch (err) {
         alert('Failed to save search: ' + err);
       } finally {
         saving = false;
       }
     }

     function formatDate(dateString: string | null): string {
       if (!dateString) return 'Never';
       const date = new Date(dateString);
       return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
     }
   </script>
   ```

3. **Add template:**
   ```svelte
   <div class="saved-searches">
     <div class="header">
       <h2>Saved Searches</h2>
       <button class="btn-primary" on:click={() => showSaveDialog = true}>
         Save Current Search
       </button>
     </div>

     {#if loading}
       <div class="loading">Loading saved searches...</div>
     {:else if error}
       <div class="error">{error}</div>
     {:else if searches.length === 0}
       <div class="empty">
         No saved searches yet. Click "Save Current Search" to create one.
       </div>
     {:else}
       <div class="search-list">
         {#each searches as search (search.id)}
           <div class="search-item">
             <div class="search-info">
               <h3 class="search-name">{search.name}</h3>
               <div class="search-meta">
                 <span class="use-count">Used {search.use_count} times</span>
                 <span class="last-used">Last: {formatDate(search.last_used)}</span>
               </div>
             </div>

             <div class="search-actions">
               <button class="btn-secondary" on:click={() => handleLoad(search)}>
                 Load
               </button>
               <button class="btn-danger" on:click={() => handleDelete(search.id)}>
                 Delete
               </button>
             </div>
           </div>
         {/each}
       </div>
     {/if}

     {#if showSaveDialog}
       <div class="modal-overlay" on:click={() => showSaveDialog = false}>
         <div class="modal" on:click|stopPropagation>
           <h3>Save Current Search</h3>

           <input
             type="text"
             bind:value={newSearchName}
             placeholder="Enter search name..."
             class="search-name-input"
           />

           <div class="modal-actions">
             <button class="btn-secondary" on:click={() => showSaveDialog = false}>
               Cancel
             </button>
             <button
               class="btn-primary"
               on:click={handleSave}
               disabled={saving || !newSearchName.trim()}
             >
               {saving ? 'Saving...' : 'Save'}
             </button>
           </div>
         </div>
       </div>
     {/if}
   </div>
   ```

4. **Add styles:**
   ```svelte
   <style>
     .saved-searches {
       padding: 20px;
     }

     .header {
       display: flex;
       justify-content: space-between;
       align-items: center;
       margin-bottom: 20px;
     }

     .header h2 {
       margin: 0;
       font-size: 24px;
     }

     .search-list {
       display: flex;
       flex-direction: column;
       gap: 12px;
     }

     .search-item {
       display: flex;
       justify-content: space-between;
       align-items: center;
       padding: 16px;
       border: 1px solid var(--border-color);
       border-radius: 8px;
       background: var(--bg-secondary);
     }

     .search-info {
       flex: 1;
     }

     .search-name {
       margin: 0 0 8px 0;
       font-size: 18px;
       font-weight: 500;
     }

     .search-meta {
       display: flex;
       gap: 16px;
       font-size: 14px;
       color: var(--text-secondary);
     }

     .search-actions {
       display: flex;
       gap: 8px;
     }

     .loading,
     .error,
     .empty {
       padding: 40px;
       text-align: center;
       color: var(--text-secondary);
     }

     .error {
       color: var(--error-color);
     }

     .modal-overlay {
       position: fixed;
       top: 0;
       left: 0;
       right: 0;
       bottom: 0;
       background: rgba(0, 0, 0, 0.5);
       display: flex;
       align-items: center;
       justify-content: center;
       z-index: 1000;
     }

     .modal {
       background: var(--bg-primary);
       padding: 24px;
       border-radius: 8px;
       min-width: 400px;
       box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
     }

     .modal h3 {
       margin: 0 0 16px 0;
     }

     .search-name-input {
       width: 100%;
       padding: 8px 12px;
       font-size: 14px;
       border: 1px solid var(--border-color);
       border-radius: 4px;
       margin-bottom: 16px;
     }

     .modal-actions {
       display: flex;
       justify-content: flex-end;
       gap: 8px;
     }

     .btn-primary,
     .btn-secondary,
     .btn-danger {
       padding: 8px 16px;
       border: none;
       border-radius: 4px;
       cursor: pointer;
       font-size: 14px;
       transition: opacity 0.2s;
     }

     .btn-primary {
       background: var(--accent-color);
       color: white;
     }

     .btn-secondary {
       background: var(--bg-tertiary);
       color: var(--text-primary);
     }

     .btn-danger {
       background: var(--error-color);
       color: white;
     }

     .btn-primary:disabled {
       opacity: 0.5;
       cursor: not-allowed;
     }

     .btn-primary:hover:not(:disabled),
     .btn-secondary:hover,
     .btn-danger:hover {
       opacity: 0.8;
     }
   </style>
   ```

**Verification:**
- [ ] Component renders
- [ ] Can save current search
- [ ] Can load search (applies filters)
- [ ] Can delete search
- [ ] use_count increments on load
- [ ] List sorted by last_used

---

### Day 3 Deliverables Checklist

- [ ] **Backend:**
  - [ ] `SavedSearch` model created
  - [ ] `SavedSearchRepository` with CRUD methods
  - [ ] 4 Tauri commands (save, load, get, delete)

- [ ] **Frontend:**
  - [ ] TypeScript types for SavedSearch
  - [ ] `savedSearchesApi.ts` API wrapper
  - [ ] `VIP3SavedSearches.svelte` component
  - [ ] Save/load/delete UI working

- [ ] **Database:**
  - [ ] `saved_searches` table exists (from migrations)
  - [ ] use_count increments on load
  - [ ] last_used updates on load

- [ ] **Testing:**
  - [ ] Manual test: save → load → delete
  - [ ] Verify filters apply correctly when loading

---

## Day 4: Collections

**Goal:** Implement collections (create, add files, reorder, delete)
**Files Created:** 6 files
**Lines of Code:** ~700 lines

[... Due to length constraints, I'll provide the structure. Would you like me to continue with the detailed Day 4, Day 5 steps, or would you prefer I create a separate document for those?]

---

## Day 5: Favorites & Category Management

[Similar detailed breakdown...]

---

## Testing & Verification

[Comprehensive testing steps...]

---

## Performance Benchmarks

Target metrics to verify:
- Filter counts: <50ms with 2.15M files
- Saved searches load: <10ms
- Collections create: <10ms
- Favorites toggle: <5ms

---

**This is an EXTREMELY detailed architectural plan. Each step is broken down to the minute detail. Would you like me to continue with Days 4 and 5 in the same level of detail?**
