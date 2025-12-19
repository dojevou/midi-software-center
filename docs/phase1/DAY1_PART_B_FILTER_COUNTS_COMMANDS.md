# Day 1, Part 1B: Filter Counts Tauri Commands

**Duration:** 1 hour
**Prerequisites:** Part 1A completed
**Files to modify:** 1

---

## Overview

In Part 1A, you created the models and partial repository. Now you'll:
1. Complete the VIP3Repository with remaining count methods
2. Implement the main `get_filter_counts()` method with parallel execution
3. Create the Tauri command wrapper
4. Register the command in main.rs

---

## Step 1: Complete VIP3Repository Count Methods (20 min)

Add the remaining count methods to `app/src-tauri/src/db/repositories/vip3_repository.rs`:

```rust
impl Vip3Repository {
    // ... existing methods from Part 1A ...

    /// Count files by BPM range
    async fn count_by_bpm_range(
        &self,
        filters: &Vip3Filters,
        bpm_min: Option<f64>,
        bpm_max: Option<f64>,
    ) -> Result<HashMap<String, usize>, String> {
        let mut counts = HashMap::new();

        // Define BPM ranges: 60-80, 80-100, 100-120, 120-140, 140-160, 160-180, 180+
        let ranges = vec![
            (60.0, 80.0, "60-80"),
            (80.0, 100.0, "80-100"),
            (100.0, 120.0, "100-120"),
            (120.0, 140.0, "120-140"),
            (140.0, 160.0, "140-160"),
            (160.0, 180.0, "160-180"),
            (180.0, 999.0, "180+"),
        ];

        for (min, max, label) in ranges {
            // Build WHERE clause, excluding BPM filters
            let mut where_clause = self.build_where_clause(filters, Some("bpm"));
            where_clause.push_str(" AND mm.bpm >= $1 AND mm.bpm < $2");

            let query_str = format!(
                "SELECT COUNT(DISTINCT f.id)::int8 as count
                 FROM files f
                 LEFT JOIN musical_metadata mm ON f.id = mm.file_id
                 LEFT JOIN file_tags ft ON f.id = ft.file_id
                 LEFT JOIN tags t ON ft.tag_id = t.id
                 LEFT JOIN file_timbres ftim ON f.id = ftim.file_id
                 LEFT JOIN file_styles fs ON f.id = fs.file_id
                 LEFT JOIN file_articulations fa ON f.id = fa.file_id
                 WHERE {}",
                where_clause
            );

            let row = sqlx::query(&query_str)
                .bind(min)
                .bind(max)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("BPM count query failed: {}", e))?;

            let count: i64 = row.try_get("count").unwrap_or(0);
            counts.insert(label.to_string(), count as usize);
        }

        Ok(counts)
    }

    /// Count files by key signature
    async fn count_by_key(
        &self,
        filters: &Vip3Filters,
    ) -> Result<HashMap<String, usize>, String> {
        let mut counts = HashMap::new();

        // All 24 keys (12 major + 12 minor)
        let keys = vec![
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
            "Cm", "C#m", "Dm", "D#m", "Em", "Fm", "F#m", "Gm", "G#m", "Am", "A#m", "Bm",
        ];

        for key in keys {
            let mut where_clause = self.build_where_clause(filters, Some("key"));
            where_clause.push_str(" AND mm.key_signature = $1");

            let query_str = format!(
                "SELECT COUNT(DISTINCT f.id)::int8 as count
                 FROM files f
                 LEFT JOIN musical_metadata mm ON f.id = mm.file_id
                 LEFT JOIN file_tags ft ON f.id = ft.file_id
                 LEFT JOIN tags t ON ft.tag_id = t.id
                 LEFT JOIN file_timbres ftim ON f.id = ftim.file_id
                 LEFT JOIN file_styles fs ON f.id = fs.file_id
                 LEFT JOIN file_articulations fa ON f.id = fa.file_id
                 WHERE {}",
                where_clause
            );

            let row = sqlx::query(&query_str)
                .bind(key)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("Key count query failed: {}", e))?;

            let count: i64 = row.try_get("count").unwrap_or(0);
            counts.insert(key.to_string(), count as usize);
        }

        Ok(counts)
    }

    /// Count files by MIDI channel count
    async fn count_by_channels(
        &self,
        filters: &Vip3Filters,
    ) -> Result<HashMap<String, usize>, String> {
        let mut counts = HashMap::new();

        // Channel count ranges: 1, 2-4, 5-8, 9-16
        let channel_ranges = vec![
            (1, 1, "1"),
            (2, 4, "2-4"),
            (5, 8, "5-8"),
            (9, 16, "9-16"),
        ];

        for (min, max, label) in channel_ranges {
            let mut where_clause = self.build_where_clause(filters, Some("channels"));
            where_clause.push_str(" AND mm.channel_count >= $1 AND mm.channel_count <= $2");

            let query_str = format!(
                "SELECT COUNT(DISTINCT f.id)::int8 as count
                 FROM files f
                 LEFT JOIN musical_metadata mm ON f.id = mm.file_id
                 LEFT JOIN file_tags ft ON f.id = ft.file_id
                 LEFT JOIN tags t ON ft.tag_id = t.id
                 LEFT JOIN file_timbres ftim ON f.id = ftim.file_id
                 LEFT JOIN file_styles fs ON f.id = fs.file_id
                 LEFT JOIN file_articulations fa ON f.id = fa.file_id
                 WHERE {}",
                where_clause
            );

            let row = sqlx::query(&query_str)
                .bind(min)
                .bind(max)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("Channel count query failed: {}", e))?;

            let count: i64 = row.try_get("count").unwrap_or(0);
            counts.insert(label.to_string(), count as usize);
        }

        Ok(counts)
    }
}
```

---

## Step 2: Implement Main get_filter_counts Method (25 min)

Add the main method that executes all counts in parallel:

```rust
use tokio::join;
use std::time::Instant;

impl Vip3Repository {
    /// Get all filter counts in parallel
    /// Target: <50ms total execution time
    pub async fn get_filter_counts(
        &self,
        filters: &Vip3Filters,
    ) -> Result<FilterCounts, String> {
        let start = Instant::now();

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
        ) = join!(
            self.count_by_folder(filters),
            self.count_by_instrument(filters),
            self.count_by_timbre(filters),
            self.count_by_style(filters),
            self.count_by_articulation(filters),
            self.count_by_bpm_range(filters, None, None),
            self.count_by_key(filters),
            self.count_by_channels(filters),
        );

        // Get total matches (files matching all current filters)
        let total_matches = self.count_total_matches(filters).await?;

        let elapsed = start.elapsed();
        log::info!("Filter counts completed in {:?} (target: <50ms)", elapsed);

        if elapsed.as_millis() > 50 {
            log::warn!("Filter counts exceeded 50ms target: {:?}", elapsed);
        }

        Ok(FilterCounts {
            folder_counts: folder_counts?,
            instrument_counts: instrument_counts?,
            timbre_counts: timbre_counts?,
            style_counts: style_counts?,
            articulation_counts: articulation_counts?,
            bpm_counts: bpm_counts?,
            key_counts: key_counts?,
            channel_counts: channel_counts?,
            total_matches,
        })
    }

    /// Count total files matching all current filters
    async fn count_total_matches(&self, filters: &Vip3Filters) -> Result<usize, String> {
        let where_clause = self.build_where_clause(filters, None);

        let query_str = format!(
            "SELECT COUNT(DISTINCT f.id)::int8 as count
             FROM files f
             LEFT JOIN musical_metadata mm ON f.id = mm.file_id
             LEFT JOIN file_tags ft ON f.id = ft.file_id
             LEFT JOIN tags t ON ft.tag_id = t.id
             LEFT JOIN file_timbres ftim ON f.id = ftim.file_id
             LEFT JOIN file_styles fs ON f.id = fs.file_id
             LEFT JOIN file_articulations fa ON f.id = fa.file_id
             WHERE {}",
            where_clause
        );

        let row = sqlx::query(&query_str)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Total count query failed: {}", e))?;

        let count: i64 = row.try_get("count").unwrap_or(0);
        Ok(count as usize)
    }
}
```

---

## Step 3: Create Tauri Command (15 min)

Create `app/src-tauri/src/commands/pipeline/vip3/filter_counts.rs`:

```rust
use crate::db::{
    models::{FilterCounts, Vip3Filters},
    repositories::Vip3Repository,
};
use crate::AppState;
use tauri::State;

/// Get VIP3 filter counts for all categories
/// Returns counts for folders, instruments, timbres, styles, articulations, BPM, key, channels
#[tauri::command]
pub async fn get_vip3_filter_counts(
    filters: Vip3Filters,
    state: State<'_, AppState>,
) -> Result<FilterCounts, String> {
    log::info!("Getting VIP3 filter counts with filters: {:?}", filters);

    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());

    let counts = repo.get_filter_counts(&filters).await?;

    log::info!("Filter counts retrieved: {} total matches", counts.total_matches);

    Ok(counts)
}
```

Update `app/src-tauri/src/commands/pipeline/vip3/mod.rs`:

```rust
pub mod filter_counts;

pub use filter_counts::get_vip3_filter_counts;
```

Update `app/src-tauri/src/commands/pipeline/mod.rs` to include vip3:

```rust
pub mod vip3;

pub use vip3::*;
```

---

## Step 4: Register Command in main.rs (5 min)

In `app/src-tauri/src/main.rs`, add the command to the invoke handler:

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            db_pool: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...

            // VIP3 commands
            get_vip3_filter_counts,

            // ... other commands ...
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Also add the import at the top:

```rust
use midi_app::commands::pipeline::get_vip3_filter_counts;
```

---

## Verification (5 min)

### 1. Compilation Check

```bash
cd app/src-tauri
cargo check
```

**Expected:** No errors, command registered successfully

### 2. Test the Command

Start the dev server:

```bash
make dev
```

In the browser console:

```javascript
// Test with empty filters (should return all counts)
const counts = await window.__TAURI__.invoke('get_vip3_filter_counts', {
  filters: {}
});
console.log('Filter counts:', counts);
console.log('Total matches:', counts.total_matches);
console.log('Folders:', counts.folder_counts);
console.log('BPM ranges:', counts.bpm_counts);

// Test with folder filter
const folderCounts = await window.__TAURI__.invoke('get_vip3_filter_counts', {
  filters: { folder_ids: [1] }
});
console.log('Counts with folder filter:', folderCounts);
```

**Expected Output:**
```javascript
{
  folder_counts: { "1": 150, "2": 300, ... },
  instrument_counts: { "5": 200, "12": 450, ... },
  timbre_counts: { "1": 100, "2": 250, ... },
  style_counts: { "1": 180, "3": 320, ... },
  articulation_counts: { "1": 90, "2": 210, ... },
  bpm_counts: { "60-80": 50, "100-120": 300, ... },
  key_counts: { "C": 100, "Am": 80, ... },
  channel_counts: { "1": 500, "2-4": 300, ... },
  total_matches: 1500
}
```

### 3. Performance Check

Look for the timing log in the terminal:

```
INFO Filter counts completed in 35ms (target: <50ms)
```

**Target:** <50ms execution time

If it exceeds 50ms, you'll see:
```
WARN Filter counts exceeded 50ms target: 75ms
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Command not found" | Verify registration in `main.rs` invoke_handler |
| Compilation error on `join!` | Add `use tokio::join;` at top of vip3_repository.rs |
| Counts taking >50ms | Continue to Part 2 (Database Optimization) for index creation |
| Empty counts returned | Check database has files and tags from import |
| Type mismatch on HashMap | Ensure `use std::collections::HashMap;` in models |

---

## What's Next?

✅ **You've completed:**
- VIP3Repository with all count methods
- Parallel execution with tokio::join!
- Tauri command wrapper
- Command registration

**Next:** [Part 1C: Frontend API & Store](./DAY1_PART_C_FILTER_COUNTS_FRONTEND.md)
- Create TypeScript types matching Rust structs
- Implement vip3BrowserApi.ts
- Create Svelte store for filter state
- Wire up reactive filter count updates

**Performance Note:** If counts are >50ms, you'll optimize in Part 2 (Day 2) with database indexes.
