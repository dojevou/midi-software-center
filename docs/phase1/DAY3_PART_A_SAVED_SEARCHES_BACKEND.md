# Day 3, Part 3A: Saved Searches Backend

**Duration:** 2 hours
**Prerequisites:** Days 1-2 completed
**Files to create/modify:** 3

---

## Overview

The `saved_searches` table already exists from migration 001. Now you'll:
1. Create SavedSearch Rust model
2. Implement SavedSearchRepository with CRUD operations
3. Create Tauri commands for save/load/delete/increment
4. Track use_count for popular searches

---

## Step 1: SavedSearch Model (20 min)

Create `app/src-tauri/src/db/models/saved_search.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::Vip3Filters;

/// Saved search entry
/// Stores a named VIP3 filter combination for quick recall
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SavedSearch {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,

    /// The filter state as JSON
    /// Stored as JSONB in database, deserialized to Vip3Filters
    #[sqlx(json)]
    pub filters: Vip3Filters,

    /// Number of times this search has been loaded
    pub use_count: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New saved search (without ID and timestamps)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSavedSearch {
    pub name: String,
    pub description: Option<String>,
    pub filters: Vip3Filters,
}

impl NewSavedSearch {
    pub fn new(name: String, filters: Vip3Filters) -> Self {
        Self {
            name,
            description: None,
            filters,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}
```

Update `app/src-tauri/src/db/models/mod.rs`:

```rust
pub mod saved_search;

pub use saved_search::{NewSavedSearch, SavedSearch};
```

---

## Step 2: SavedSearchRepository (1 hour)

Create `app/src-tauri/src/db/repositories/saved_search_repository.rs`:

```rust
use sqlx::{PgPool, Row};

use crate::db::models::{NewSavedSearch, SavedSearch, Vip3Filters};

pub struct SavedSearchRepository {
    pool: PgPool,
}

impl SavedSearchRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get all saved searches, ordered by use_count DESC
    pub async fn get_all(&self) -> Result<Vec<SavedSearch>, String> {
        sqlx::query_as::<_, SavedSearch>(
            "SELECT id, name, description, filters, use_count, created_at, updated_at
             FROM saved_searches
             ORDER BY use_count DESC, updated_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch saved searches: {}", e))
    }

    /// Get a saved search by ID
    pub async fn get_by_id(&self, id: i64) -> Result<SavedSearch, String> {
        sqlx::query_as::<_, SavedSearch>(
            "SELECT id, name, description, filters, use_count, created_at, updated_at
             FROM saved_searches
             WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch saved search: {}", e))
    }

    /// Get a saved search by name
    pub async fn get_by_name(&self, name: &str) -> Result<Option<SavedSearch>, String> {
        sqlx::query_as::<_, SavedSearch>(
            "SELECT id, name, description, filters, use_count, created_at, updated_at
             FROM saved_searches
             WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch saved search: {}", e))
    }

    /// Create a new saved search
    /// Returns error if name already exists
    pub async fn create(&self, new_search: &NewSavedSearch) -> Result<SavedSearch, String> {
        // Check if name already exists
        if let Some(existing) = self.get_by_name(&new_search.name).await? {
            return Err(format!(
                "Saved search '{}' already exists (ID: {})",
                existing.name, existing.id
            ));
        }

        let search = sqlx::query_as::<_, SavedSearch>(
            "INSERT INTO saved_searches (name, description, filters, use_count)
             VALUES ($1, $2, $3, 0)
             RETURNING id, name, description, filters, use_count, created_at, updated_at"
        )
        .bind(&new_search.name)
        .bind(&new_search.description)
        .bind(sqlx::types::Json(&new_search.filters))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create saved search: {}", e))?;

        log::info!("Created saved search: {} (ID: {})", search.name, search.id);

        Ok(search)
    }

    /// Update a saved search's name, description, or filters
    pub async fn update(
        &self,
        id: i64,
        name: Option<String>,
        description: Option<String>,
        filters: Option<Vip3Filters>,
    ) -> Result<SavedSearch, String> {
        // Build dynamic UPDATE query based on what's being updated
        let mut query = String::from("UPDATE saved_searches SET updated_at = NOW()");
        let mut param_count = 1;

        if name.is_some() {
            query.push_str(&format!(", name = ${}", param_count));
            param_count += 1;
        }

        if description.is_some() {
            query.push_str(&format!(", description = ${}", param_count));
            param_count += 1;
        }

        if filters.is_some() {
            query.push_str(&format!(", filters = ${}", param_count));
            param_count += 1;
        }

        query.push_str(&format!(
            " WHERE id = ${}
             RETURNING id, name, description, filters, use_count, created_at, updated_at",
            param_count
        ));

        let mut q = sqlx::query_as::<_, SavedSearch>(&query);

        if let Some(n) = &name {
            q = q.bind(n);
        }

        if let Some(d) = &description {
            q = q.bind(d);
        }

        if let Some(f) = &filters {
            q = q.bind(sqlx::types::Json(f));
        }

        q = q.bind(id);

        let search = q
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to update saved search: {}", e))?;

        log::info!("Updated saved search: {} (ID: {})", search.name, search.id);

        Ok(search)
    }

    /// Delete a saved search
    pub async fn delete(&self, id: i64) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM saved_searches WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete saved search: {}", e))?;

        if result.rows_affected() == 0 {
            return Err(format!("Saved search {} not found", id));
        }

        log::info!("Deleted saved search ID: {}", id);

        Ok(())
    }

    /// Increment use_count when a saved search is loaded
    pub async fn increment_use_count(&self, id: i64) -> Result<(), String> {
        sqlx::query(
            "UPDATE saved_searches
             SET use_count = use_count + 1, updated_at = NOW()
             WHERE id = $1"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to increment use_count: {}", e))?;

        log::debug!("Incremented use_count for saved search ID: {}", id);

        Ok(())
    }

    /// Get most popular saved searches (by use_count)
    pub async fn get_popular(&self, limit: i64) -> Result<Vec<SavedSearch>, String> {
        sqlx::query_as::<_, SavedSearch>(
            "SELECT id, name, description, filters, use_count, created_at, updated_at
             FROM saved_searches
             WHERE use_count > 0
             ORDER BY use_count DESC, updated_at DESC
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch popular searches: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::Vip3Filters;

    async fn setup_test_db() -> PgPool {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://midiuser:145278963@localhost:5433/midi_library".to_string());

        PgPool::connect(&db_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_and_get_saved_search() {
        let pool = setup_test_db().await;
        let repo = SavedSearchRepository::new(pool.clone());

        // Create a test search
        let filters = Vip3Filters {
            folder_ids: Some(vec![1, 2]),
            bpm_min: Some(100.0),
            bpm_max: Some(140.0),
            ..Default::default()
        };

        let new_search = NewSavedSearch::new("Test Search".to_string(), filters.clone())
            .with_description("A test saved search".to_string());

        let created = repo.create(&new_search).await.unwrap();

        assert_eq!(created.name, "Test Search");
        assert_eq!(created.use_count, 0);
        assert_eq!(created.filters.folder_ids, Some(vec![1, 2]));

        // Get by ID
        let fetched = repo.get_by_id(created.id).await.unwrap();
        assert_eq!(fetched.id, created.id);

        // Get by name
        let by_name = repo.get_by_name("Test Search").await.unwrap().unwrap();
        assert_eq!(by_name.id, created.id);

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_increment_use_count() {
        let pool = setup_test_db().await;
        let repo = SavedSearchRepository::new(pool.clone());

        let filters = Vip3Filters::default();
        let new_search = NewSavedSearch::new("Use Count Test".to_string(), filters);

        let created = repo.create(&new_search).await.unwrap();

        // Increment 3 times
        repo.increment_use_count(created.id).await.unwrap();
        repo.increment_use_count(created.id).await.unwrap();
        repo.increment_use_count(created.id).await.unwrap();

        let updated = repo.get_by_id(created.id).await.unwrap();
        assert_eq!(updated.use_count, 3);

        // Cleanup
        repo.delete(created.id).await.unwrap();
    }
}
```

Update `app/src-tauri/src/db/repositories/mod.rs`:

```rust
pub mod saved_search_repository;

pub use saved_search_repository::SavedSearchRepository;
```

---

## Step 3: Tauri Commands (30 min)

Create `app/src-tauri/src/commands/pipeline/vip3/saved_searches.rs`:

```rust
use crate::db::{
    models::{NewSavedSearch, SavedSearch, Vip3Filters},
    repositories::SavedSearchRepository,
};
use crate::AppState;
use tauri::State;

/// Get all saved searches
#[tauri::command]
pub async fn get_saved_searches(state: State<'_, AppState>) -> Result<Vec<SavedSearch>, String> {
    let pool = state.db_pool.lock().await;
    let repo = SavedSearchRepository::new(pool.clone());

    repo.get_all().await
}

/// Get a saved search by ID
#[tauri::command]
pub async fn get_saved_search(
    id: i64,
    state: State<'_, AppState>,
) -> Result<SavedSearch, String> {
    let pool = state.db_pool.lock().await;
    let repo = SavedSearchRepository::new(pool.clone());

    repo.get_by_id(id).await
}

/// Create a new saved search
#[tauri::command]
pub async fn create_saved_search(
    name: String,
    description: Option<String>,
    filters: Vip3Filters,
    state: State<'_, AppState>,
) -> Result<SavedSearch, String> {
    log::info!("Creating saved search: {}", name);

    let pool = state.db_pool.lock().await;
    let repo = SavedSearchRepository::new(pool.clone());

    let mut new_search = NewSavedSearch::new(name, filters);

    if let Some(desc) = description {
        new_search = new_search.with_description(desc);
    }

    repo.create(&new_search).await
}

/// Update a saved search
#[tauri::command]
pub async fn update_saved_search(
    id: i64,
    name: Option<String>,
    description: Option<String>,
    filters: Option<Vip3Filters>,
    state: State<'_, AppState>,
) -> Result<SavedSearch, String> {
    log::info!("Updating saved search ID: {}", id);

    let pool = state.db_pool.lock().await;
    let repo = SavedSearchRepository::new(pool.clone());

    repo.update(id, name, description, filters).await
}

/// Delete a saved search
#[tauri::command]
pub async fn delete_saved_search(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    log::info!("Deleting saved search ID: {}", id);

    let pool = state.db_pool.lock().await;
    let repo = SavedSearchRepository::new(pool.clone());

    repo.delete(id).await
}

/// Load a saved search (increments use_count and returns filters)
#[tauri::command]
pub async fn load_saved_search(
    id: i64,
    state: State<'_, AppState>,
) -> Result<Vip3Filters, String> {
    log::info!("Loading saved search ID: {}", id);

    let pool = state.db_pool.lock().await;
    let repo = SavedSearchRepository::new(pool.clone());

    // Get the search
    let search = repo.get_by_id(id).await?;

    // Increment use count
    repo.increment_use_count(id).await?;

    Ok(search.filters)
}

/// Get popular saved searches (by use_count)
#[tauri::command]
pub async fn get_popular_saved_searches(
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<SavedSearch>, String> {
    let pool = state.db_pool.lock().await;
    let repo = SavedSearchRepository::new(pool.clone());

    repo.get_popular(limit.unwrap_or(10)).await
}
```

Update `app/src-tauri/src/commands/pipeline/vip3/mod.rs`:

```rust
pub mod filter_counts;
pub mod saved_searches;

pub use filter_counts::get_vip3_filter_counts;
pub use saved_searches::*;
```

Register commands in `app/src-tauri/src/main.rs`:

```rust
use midi_app::commands::pipeline::{
    get_vip3_filter_counts,
    get_saved_searches,
    get_saved_search,
    create_saved_search,
    update_saved_search,
    delete_saved_search,
    load_saved_search,
    get_popular_saved_searches,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...

            // VIP3 saved searches
            get_saved_searches,
            get_saved_search,
            create_saved_search,
            update_saved_search,
            delete_saved_search,
            load_saved_search,
            get_popular_saved_searches,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Verification (10 min)

### 1. Compilation Check

```bash
cd app/src-tauri
cargo check
cargo test saved_search
```

**Expected:** Tests pass, no compilation errors

### 2. Test Commands

```bash
make dev
```

In browser console:

```javascript
// Create a saved search
const created = await window.__TAURI__.invoke('create_saved_search', {
  name: 'High Energy Drums',
  description: 'Fast drums with high BPM',
  filters: {
    folder_ids: [1],
    instrument_ids: [5, 12],
    bpm_min: 140,
    bpm_max: 180
  }
});
console.log('Created:', created);

// Get all saved searches
const all = await window.__TAURI__.invoke('get_saved_searches');
console.log('All searches:', all);

// Load a saved search (increments use_count)
const filters = await window.__TAURI__.invoke('load_saved_search', { id: created.id });
console.log('Loaded filters:', filters);

// Get by ID to verify use_count incremented
const updated = await window.__TAURI__.invoke('get_saved_search', { id: created.id });
console.log('Use count:', updated.use_count); // Should be 1
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Command not found" | Verify commands registered in `main.rs` |
| "already exists" error | Name must be unique, use different name or delete existing |
| JSONB serialization error | Verify `Vip3Filters` derives `Serialize` and `Deserialize` |
| Test failures | Ensure test database is running: `make docker-up` |

---

## What's Next?

✅ **You've completed:**
- SavedSearch model with JSONB filters
- SavedSearchRepository with CRUD and use_count tracking
- 7 Tauri commands for saved search management

**Next:** [Part 3B: Saved Searches Frontend](./DAY3_PART_B_SAVED_SEARCHES_FRONTEND.md)
- Create TypeScript types
- Implement savedSearchesApi.ts
- Create VIP3SavedSearches component
- Add "Save Search" button to VIP3Browser
- Show popular searches in sidebar
