# Day 4, Part 4A: Collections Backend

**Duration:** 2 hours
**Prerequisites:** Days 1-3 completed
**Files to create/modify:** 3

---

## Overview

The `collections` and `collection_files` tables already exist. Now you'll:
1. Create Collection and CollectionFile models
2. Implement CollectionRepository with ordering support
3. Create Tauri commands for CRUD and reordering
4. Support adding/removing files from collections

---

## Step 1: Collection Models (25 min)

Create `app/src-tauri/src/db/models/collection.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Collection of MIDI files
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// File in a collection with ordering
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CollectionFile {
    pub id: i64,
    pub collection_id: i64,
    pub file_id: i64,

    /// Order within collection (0-indexed)
    /// Lower numbers appear first
    pub order_index: i32,

    pub added_at: DateTime<Utc>,
}

/// File with collection metadata
/// Returned when fetching collection files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionFileWithMetadata {
    pub id: i64,
    pub collection_id: i64,
    pub file_id: i64,
    pub order_index: i32,
    pub added_at: DateTime<Utc>,

    // File metadata
    pub file_name: String,
    pub file_path: String,
    pub bpm: Option<f64>,
    pub key_signature: Option<String>,
    pub duration_seconds: Option<f64>,
}

/// New collection (without ID and timestamps)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCollection {
    pub name: String,
    pub description: Option<String>,
}

impl NewCollection {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

/// Request to add file to collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFileToCollection {
    pub file_id: i64,
    /// Optional specific order (if None, adds to end)
    pub order_index: Option<i32>,
}

/// Request to reorder files in collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorderCollectionFiles {
    /// List of file IDs in new order
    pub file_ids: Vec<i64>,
}
```

Update `app/src-tauri/src/db/models/mod.rs`:

```rust
pub mod collection;

pub use collection::{
    AddFileToCollection, Collection, CollectionFile, CollectionFileWithMetadata,
    NewCollection, ReorderCollectionFiles,
};
```

---

## Step 2: CollectionRepository (1 hour 15 min)

Create `app/src-tauri/src/db/repositories/collection_repository.rs`:

```rust
use sqlx::{PgPool, Row};

use crate::db::models::{
    AddFileToCollection, Collection, CollectionFile, CollectionFileWithMetadata,
    NewCollection,
};

pub struct CollectionRepository {
    pool: PgPool,
}

impl CollectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get all collections
    pub async fn get_all(&self) -> Result<Vec<Collection>, String> {
        sqlx::query_as::<_, Collection>(
            "SELECT id, name, description, created_at, updated_at
             FROM collections
             ORDER BY updated_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch collections: {}", e))
    }

    /// Get a collection by ID
    pub async fn get_by_id(&self, id: i64) -> Result<Collection, String> {
        sqlx::query_as::<_, Collection>(
            "SELECT id, name, description, created_at, updated_at
             FROM collections
             WHERE id = $1"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch collection: {}", e))
    }

    /// Create a new collection
    pub async fn create(&self, new_collection: &NewCollection) -> Result<Collection, String> {
        let collection = sqlx::query_as::<_, Collection>(
            "INSERT INTO collections (name, description)
             VALUES ($1, $2)
             RETURNING id, name, description, created_at, updated_at"
        )
        .bind(&new_collection.name)
        .bind(&new_collection.description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create collection: {}", e))?;

        log::info!("Created collection: {} (ID: {})", collection.name, collection.id);

        Ok(collection)
    }

    /// Update a collection's name and/or description
    pub async fn update(
        &self,
        id: i64,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Collection, String> {
        let mut query = String::from("UPDATE collections SET updated_at = NOW()");
        let mut param_count = 1;

        if name.is_some() {
            query.push_str(&format!(", name = ${}", param_count));
            param_count += 1;
        }

        if description.is_some() {
            query.push_str(&format!(", description = ${}", param_count));
            param_count += 1;
        }

        query.push_str(&format!(
            " WHERE id = ${}
             RETURNING id, name, description, created_at, updated_at",
            param_count
        ));

        let mut q = sqlx::query_as::<_, Collection>(&query);

        if let Some(n) = &name {
            q = q.bind(n);
        }

        if let Some(d) = &description {
            q = q.bind(d);
        }

        q = q.bind(id);

        let collection = q
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to update collection: {}", e))?;

        log::info!("Updated collection: {} (ID: {})", collection.name, collection.id);

        Ok(collection)
    }

    /// Delete a collection and all its files
    pub async fn delete(&self, id: i64) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM collections WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete collection: {}", e))?;

        if result.rows_affected() == 0 {
            return Err(format!("Collection {} not found", id));
        }

        log::info!("Deleted collection ID: {}", id);

        Ok(())
    }

    /// Get all files in a collection, ordered by order_index
    pub async fn get_files(
        &self,
        collection_id: i64,
    ) -> Result<Vec<CollectionFileWithMetadata>, String> {
        let files = sqlx::query_as::<_, CollectionFileWithMetadata>(
            "SELECT
                cf.id,
                cf.collection_id,
                cf.file_id,
                cf.order_index,
                cf.added_at,
                f.file_name,
                f.file_path,
                mm.bpm,
                mm.key_signature,
                mm.duration_seconds
             FROM collection_files cf
             JOIN files f ON cf.file_id = f.id
             LEFT JOIN musical_metadata mm ON f.id = mm.file_id
             WHERE cf.collection_id = $1
             ORDER BY cf.order_index ASC"
        )
        .bind(collection_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch collection files: {}", e))?;

        Ok(files)
    }

    /// Add a file to a collection
    /// If order_index is None, adds to end
    pub async fn add_file(
        &self,
        collection_id: i64,
        request: &AddFileToCollection,
    ) -> Result<CollectionFile, String> {
        // Get current max order_index
        let max_order: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(order_index) FROM collection_files WHERE collection_id = $1"
        )
        .bind(collection_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to get max order: {}", e))?;

        let order_index = request.order_index.unwrap_or_else(|| {
            max_order.map(|m| m + 1).unwrap_or(0)
        });

        // Check if file already in collection
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM collection_files WHERE collection_id = $1 AND file_id = $2)"
        )
        .bind(collection_id)
        .bind(request.file_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to check file existence: {}", e))?;

        if exists {
            return Err(format!(
                "File {} already in collection {}",
                request.file_id, collection_id
            ));
        }

        let collection_file = sqlx::query_as::<_, CollectionFile>(
            "INSERT INTO collection_files (collection_id, file_id, order_index)
             VALUES ($1, $2, $3)
             RETURNING id, collection_id, file_id, order_index, added_at"
        )
        .bind(collection_id)
        .bind(request.file_id)
        .bind(order_index)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to add file to collection: {}", e))?;

        // Update collection updated_at
        sqlx::query("UPDATE collections SET updated_at = NOW() WHERE id = $1")
            .bind(collection_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update collection timestamp: {}", e))?;

        log::info!(
            "Added file {} to collection {} at index {}",
            request.file_id,
            collection_id,
            order_index
        );

        Ok(collection_file)
    }

    /// Remove a file from a collection
    pub async fn remove_file(
        &self,
        collection_id: i64,
        file_id: i64,
    ) -> Result<(), String> {
        let result = sqlx::query(
            "DELETE FROM collection_files WHERE collection_id = $1 AND file_id = $2"
        )
        .bind(collection_id)
        .bind(file_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to remove file from collection: {}", e))?;

        if result.rows_affected() == 0 {
            return Err(format!(
                "File {} not found in collection {}",
                file_id, collection_id
            ));
        }

        // Update collection updated_at
        sqlx::query("UPDATE collections SET updated_at = NOW() WHERE id = $1")
            .bind(collection_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update collection timestamp: {}", e))?;

        log::info!("Removed file {} from collection {}", file_id, collection_id);

        Ok(())
    }

    /// Reorder files in a collection
    /// file_ids should be the full list of file IDs in new order
    pub async fn reorder_files(
        &self,
        collection_id: i64,
        file_ids: &[i64],
    ) -> Result<(), String> {
        // Start transaction
        let mut tx = self.pool.begin().await
            .map_err(|e| format!("Failed to start transaction: {}", e))?;

        // Update each file's order_index
        for (index, file_id) in file_ids.iter().enumerate() {
            sqlx::query(
                "UPDATE collection_files
                 SET order_index = $1
                 WHERE collection_id = $2 AND file_id = $3"
            )
            .bind(index as i32)
            .bind(collection_id)
            .bind(file_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to reorder file: {}", e))?;
        }

        // Update collection updated_at
        sqlx::query("UPDATE collections SET updated_at = NOW() WHERE id = $1")
            .bind(collection_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to update collection timestamp: {}", e))?;

        tx.commit().await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        log::info!("Reordered {} files in collection {}", file_ids.len(), collection_id);

        Ok(())
    }

    /// Get count of files in a collection
    pub async fn get_file_count(&self, collection_id: i64) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM collection_files WHERE collection_id = $1"
        )
        .bind(collection_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count collection files: {}", e))?;

        Ok(count)
    }
}
```

Update `app/src-tauri/src/db/repositories/mod.rs`:

```rust
pub mod collection_repository;

pub use collection_repository::CollectionRepository;
```

---

## Step 3: Tauri Commands (20 min)

Create `app/src-tauri/src/commands/pipeline/vip3/collections.rs`:

```rust
use crate::db::{
    models::{
        AddFileToCollection, Collection, CollectionFileWithMetadata,
        NewCollection,
    },
    repositories::CollectionRepository,
};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_collections(state: State<'_, AppState>) -> Result<Vec<Collection>, String> {
    let pool = state.db_pool.lock().await;
    let repo = CollectionRepository::new(pool.clone());
    repo.get_all().await
}

#[tauri::command]
pub async fn get_collection(
    id: i64,
    state: State<'_, AppState>,
) -> Result<Collection, String> {
    let pool = state.db_pool.lock().await;
    let repo = CollectionRepository::new(pool.clone());
    repo.get_by_id(id).await
}

#[tauri::command]
pub async fn create_collection(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<Collection, String> {
    let pool = state.db_pool.lock().await;
    let repo = CollectionRepository::new(pool.clone());

    let mut new_collection = NewCollection::new(name);
    if let Some(desc) = description {
        new_collection = new_collection.with_description(desc);
    }

    repo.create(&new_collection).await
}

#[tauri::command]
pub async fn update_collection(
    id: i64,
    name: Option<String>,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<Collection, String> {
    let pool = state.db_pool.lock().await;
    let repo = CollectionRepository::new(pool.clone());
    repo.update(id, name, description).await
}

#[tauri::command]
pub async fn delete_collection(id: i64, state: State<'_, AppState>) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = CollectionRepository::new(pool.clone());
    repo.delete(id).await
}

#[tauri::command]
pub async fn get_collection_files(
    collection_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<CollectionFileWithMetadata>, String> {
    let pool = state.db_pool.lock().await;
    let repo = CollectionRepository::new(pool.clone());
    repo.get_files(collection_id).await
}

#[tauri::command]
pub async fn add_file_to_collection(
    collection_id: i64,
    file_id: i64,
    order_index: Option<i32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = CollectionRepository::new(pool.clone());

    let request = AddFileToCollection {
        file_id,
        order_index,
    };

    repo.add_file(collection_id, &request).await?;
    Ok(())
}

#[tauri::command]
pub async fn remove_file_from_collection(
    collection_id: i64,
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = CollectionRepository::new(pool.clone());
    repo.remove_file(collection_id, file_id).await
}

#[tauri::command]
pub async fn reorder_collection_files(
    collection_id: i64,
    file_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = CollectionRepository::new(pool.clone());
    repo.reorder_files(collection_id, &file_ids).await
}
```

Update `app/src-tauri/src/commands/pipeline/vip3/mod.rs` and register in `main.rs`:

```rust
// In mod.rs
pub mod collections;
pub use collections::*;

// In main.rs
use midi_app::commands::pipeline::{
    // ... existing imports ...
    get_collections,
    get_collection,
    create_collection,
    update_collection,
    delete_collection,
    get_collection_files,
    add_file_to_collection,
    remove_file_from_collection,
    reorder_collection_files,
};
```

---

## Verification (5 min)

```bash
cargo check
cargo test collection
```

Test in browser console:

```javascript
// Create collection
const coll = await window.__TAURI__.invoke('create_collection', {
  name: 'Favorite Drums',
  description: 'My favorite drum loops'
});

// Add files
await window.__TAURI__.invoke('add_file_to_collection', {
  collectionId: coll.id,
  fileId: 1
});

// Get files
const files = await window.__TAURI__.invoke('get_collection_files', {
  collectionId: coll.id
});
console.log('Collection files:', files);
```

---

## What's Next?

**Next:** [Part 4B: Collections Frontend](./DAY4_PART_B_COLLECTIONS_FRONTEND.md) - Drag-and-drop UI for collections
