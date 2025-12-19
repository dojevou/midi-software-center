# Day 1, Part A: Filter Counts Backend

**Duration:** 3 hours
**Focus:** Create Rust models and repository for filter counts
**Files Created:** 3 files
**Prerequisites:** Database running, migrations applied

---

## Overview

This part implements the backend foundation for VIP3 filter counts:
- `FilterCounts` model - Stores count results
- `Vip3Filters` model - Represents filter selections
- `Vip3Repository` - Executes COUNT queries

**Data Flow:**
```
Vip3Filters → Vip3Repository.get_filter_counts() → FilterCounts
```

---

## Step 1: Create FilterCounts Model (30 minutes)

### 1.1 Create the file

```bash
touch app/src-tauri/src/db/models/filter_counts.rs
```

### 1.2 Add complete code

**File:** `app/src-tauri/src/db/models/filter_counts.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Filter counts for VIP3 browser
/// Each HashMap maps filter value ID to count of matching files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCounts {
    /// folder_id → count of files in that folder
    pub folder_counts: HashMap<i64, usize>,

    /// tag_id → count of files with that instrument tag
    pub instrument_counts: HashMap<i64, usize>,

    /// timbre_id → count of files with that timbre
    pub timbre_counts: HashMap<i32, usize>,

    /// style_id → count of files with that style
    pub style_counts: HashMap<i32, usize>,

    /// articulation_id → count of files with that articulation
    pub articulation_counts: HashMap<i32, usize>,

    /// bpm_range_id → count of files in that BPM range
    pub bpm_range_counts: HashMap<i32, usize>,

    /// key_id → count of files in that key signature
    pub key_counts: HashMap<i32, usize>,

    /// channel → count of files using that MIDI channel
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

### 1.3 Export the model

Edit: `app/src-tauri/src/db/models/mod.rs`

Add:
```rust
pub mod filter_counts;
pub use filter_counts::FilterCounts;
```

### 1.4 Verify compilation

```bash
cd app/src-tauri
cargo check
```

**Expected:** No errors

---

## Step 2: Create Vip3Filters Model (30 minutes)

### 2.1 Create the file

```bash
touch app/src-tauri/src/db/models/vip3_filters.rs
```

### 2.2 Add complete code

**File:** `app/src-tauri/src/db/models/vip3_filters.rs`

```rust
use serde::{Deserialize, Serialize};

/// Filter selections for VIP3 browser search
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Vip3Filters {
    /// Selected folder IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_ids: Option<Vec<i64>>,

    /// Selected instrument tag IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_ids: Option<Vec<i64>>,

    /// Selected timbre category IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timbre_ids: Option<Vec<i32>>,

    /// Selected style category IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_ids: Option<Vec<i32>>,

    /// Selected articulation category IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub articulation_ids: Option<Vec<i32>>,

    /// Selected BPM range IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bpm_range_ids: Option<Vec<i32>>,

    /// Selected key signature IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_ids: Option<Vec<i32>>,

    /// Selected MIDI channel (1-16)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,

    /// Free-text search query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_query: Option<String>,

    /// Filter to favorites only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorites_only: Option<bool>,

    /// Selected tag IDs (general tags, not instruments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_ids: Option<Vec<i32>>,

    /// Minimum rating (1-5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_rating: Option<i32>,

    /// Result limit (pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,

    /// Result offset (pagination)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
}

impl Vip3Filters {
    /// Check if any filter is active
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

    /// Get effective limit (default 100, max 500)
    pub fn effective_limit(&self) -> i32 {
        self.limit.unwrap_or(100).min(500)
    }

    /// Get effective offset (default 0)
    pub fn effective_offset(&self) -> i32 {
        self.offset.unwrap_or(0).max(0)
    }
}
```

### 2.3 Export the model

Edit: `app/src-tauri/src/db/models/mod.rs`

Add:
```rust
pub mod vip3_filters;
pub use vip3_filters::Vip3Filters;
```

### 2.4 Verify compilation

```bash
cargo check
```

---

## Step 3: Create VIP3 Repository (2 hours)

### 3.1 Create the file

```bash
touch app/src-tauri/src/db/repositories/vip3_repository.rs
```

### 3.2 Add repository structure

**File:** `app/src-tauri/src/db/repositories/vip3_repository.rs`

```rust
use crate::db::error::{DbError, DbResult};
use crate::db::models::{FilterCounts, Vip3Filters};
use sqlx::PgPool;
use std::collections::HashMap;

pub struct Vip3Repository {
    pool: PgPool,
}

impl Vip3Repository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
```

### 3.3 Implement WHERE clause builder

Add to `impl Vip3Repository`:

```rust
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
                    "EXISTS (SELECT 1 FROM file_timbres ft WHERE ft.file_id = f.id AND ft.timbre_id IN ({}))",
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
                    "EXISTS (SELECT 1 FROM file_styles fs WHERE fs.file_id = f.id AND fs.style_id IN ({}))",
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
                    "EXISTS (SELECT 1 FROM file_articulations fa WHERE fa.file_id = f.id AND fa.articulation_id IN ({}))",
                    ids_str
                ));
            }
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
```

### 3.4 Implement count methods

Add each count method to `impl Vip3Repository`:

**Count by folder:**
```rust
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
        .map_err(|e| DbError::QueryFailed(e.to_string()))?;

    let mut counts = HashMap::new();
    for (folder_id, count) in rows {
        if let Some(folder_id) = folder_id {
            counts.insert(folder_id, count as usize);
        }
    }

    Ok(counts)
}
```

**Count by instrument:**
```rust
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
        .map_err(|e| DbError::QueryFailed(e.to_string()))?;

    let mut counts = HashMap::new();
    for (tag_id, count) in rows {
        counts.insert(tag_id as i64, count as usize);
    }

    Ok(counts)
}
```

**Count by timbre:**
```rust
async fn count_by_timbre(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
    let where_clause = self.build_where_clause(filters, Some("timbre"));

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

**Count by style:**
```rust
async fn count_by_style(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
    let where_clause = self.build_where_clause(filters, Some("style"));

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

**Count by articulation:**
```rust
async fn count_by_articulation(&self, filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
    let where_clause = self.build_where_clause(filters, Some("articulation"));

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

**Count total matches:**
```rust
async fn count_total_matches(&self, filters: &Vip3Filters) -> DbResult<usize> {
    let where_clause = self.build_where_clause(filters, None);

    let query = format!("SELECT COUNT(DISTINCT f.id) FROM files f WHERE {}", where_clause);

    let count: i64 = sqlx::query_scalar(&query)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DbError::QueryFailed(e.to_string()))?;

    Ok(count as usize)
}
```

**Stub methods for BPM, key, channel (implement in Part B):**
```rust
async fn count_by_bpm_range(&self, _filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
    // TODO: Implement BPM range counting
    Ok(HashMap::new())
}

async fn count_by_key(&self, _filters: &Vip3Filters) -> DbResult<HashMap<i32, usize>> {
    // TODO: Implement key counting
    Ok(HashMap::new())
}

async fn count_by_channel(&self, _filters: &Vip3Filters) -> DbResult<HashMap<u8, usize>> {
    // TODO: Implement channel counting
    Ok(HashMap::new())
}
```

### 3.5 Export repository

Edit: `app/src-tauri/src/db/repositories/mod.rs`

Add:
```rust
pub mod vip3_repository;
pub use vip3_repository::Vip3Repository;
```

### 3.6 Verify compilation

```bash
cargo check
```

**Expected:** No errors (warnings about unused async functions are OK)

---

## Verification Checklist

- [ ] `filter_counts.rs` created and compiles
- [ ] `vip3_filters.rs` created and compiles
- [ ] `vip3_repository.rs` created and compiles
- [ ] All models exported in `mod.rs`
- [ ] Repository exported in `mod.rs`
- [ ] `cargo check` shows no errors

---

## What's Next?

**Next:** [Day 1, Part B: Filter Counts Commands](./DAY1_PART_B_FILTER_COUNTS_COMMANDS.md)

You'll add:
- Main `get_filter_counts()` method with parallel execution
- Tauri command
- Command registration

**Estimated time:** 1 hour

---

## Troubleshooting

**Error: "cannot find type `DbError` in this scope"**
- Check `app/src-tauri/src/db/error.rs` exists
- Verify `use crate::db::error::{DbError, DbResult};` is present

**Error: "no method named `new` found for struct `PgPool`"**
- Add dependency: `sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres"] }`

**Compilation slow:**
- First build is always slow (downloads dependencies)
- Subsequent `cargo check` should be fast (<10 seconds)

---

**Ready for Part B?** → [DAY1_PART_B_FILTER_COUNTS_COMMANDS.md](./DAY1_PART_B_FILTER_COUNTS_COMMANDS.md)
