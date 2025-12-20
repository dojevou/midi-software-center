# Day 5, Part 5A: Favorites System

**Duration:** 1.5 hours
**Prerequisites:** Days 1-4 completed
**Files to create:** 3

---

## Overview

Add favorites functionality to VIP3 browser:
1. Add `is_favorite` flag to files table
2. Create Tauri commands to toggle favorites
3. Add favorites UI to VIP3Browser
4. Show favorites count in header

---

## Step 1: Database Migration (15 min)

Create `database/migrations/021_add_favorites.sql`:

```sql
-- Migration 021: Add favorites support
-- Adds is_favorite flag to files table and index

BEGIN;

-- Add is_favorite column (defaults to false)
ALTER TABLE files
ADD COLUMN IF NOT EXISTS is_favorite BOOLEAN NOT NULL DEFAULT false;

-- Create index for querying favorites
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_is_favorite
ON files(is_favorite)
WHERE is_favorite = true;

-- Create index for folder + favorite queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_folder_favorite
ON files(folder_id, is_favorite)
WHERE is_favorite = true;

COMMIT;
```

Apply migration:

```bash
make db-migrate
```

---

## Step 2: Add FileRepository Methods (20 min)

Update `app/src-tauri/src/db/repositories/file_repository.rs`:

```rust
impl FileRepository {
    // ... existing methods ...

    /// Toggle favorite status for a file
    pub async fn toggle_favorite(&self, file_id: i64) -> Result<bool, String> {
        let row = sqlx::query(
            "UPDATE files
             SET is_favorite = NOT is_favorite
             WHERE id = $1
             RETURNING is_favorite"
        )
        .bind(file_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to toggle favorite: {}", e))?;

        let is_favorite: bool = row.try_get("is_favorite")
            .map_err(|e| format!("Failed to get is_favorite: {}", e))?;

        log::debug!("Toggled favorite for file {}: {}", file_id, is_favorite);

        Ok(is_favorite)
    }

    /// Set favorite status for a file
    pub async fn set_favorite(&self, file_id: i64, is_favorite: bool) -> Result<(), String> {
        sqlx::query("UPDATE files SET is_favorite = $1 WHERE id = $2")
            .bind(is_favorite)
            .bind(file_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to set favorite: {}", e))?;

        Ok(())
    }

    /// Get all favorite files
    pub async fn get_favorites(&self, limit: Option<i64>) -> Result<Vec<crate::db::models::File>, String> {
        let query = if let Some(l) = limit {
            format!(
                "SELECT {} FROM files WHERE is_favorite = true ORDER BY updated_at DESC LIMIT {}",
                self.file_columns(),
                l
            )
        } else {
            format!(
                "SELECT {} FROM files WHERE is_favorite = true ORDER BY updated_at DESC",
                self.file_columns()
            )
        };

        sqlx::query_as::<_, crate::db::models::File>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to fetch favorites: {}", e))
    }

    /// Count favorite files
    pub async fn count_favorites(&self) -> Result<i64, String> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM files WHERE is_favorite = true"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to count favorites: {}", e))?;

        Ok(count)
    }

    /// Get favorite files by folder
    pub async fn get_favorites_by_folder(
        &self,
        folder_id: i64,
        limit: Option<i64>,
    ) -> Result<Vec<crate::db::models::File>, String> {
        let query = if let Some(l) = limit {
            format!(
                "SELECT {} FROM files
                 WHERE is_favorite = true AND folder_id = $1
                 ORDER BY updated_at DESC
                 LIMIT {}",
                self.file_columns(),
                l
            )
        } else {
            format!(
                "SELECT {} FROM files
                 WHERE is_favorite = true AND folder_id = $1
                 ORDER BY updated_at DESC",
                self.file_columns()
            )
        };

        sqlx::query_as::<_, crate::db::models::File>(&query)
            .bind(folder_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to fetch favorites: {}", e))
    }
}
```

---

## Step 3: Tauri Commands (15 min)

Create `app/src-tauri/src/commands/pipeline/vip3/favorites.rs`:

```rust
use crate::db::{models::File, repositories::FileRepository};
use crate::AppState;
use tauri::State;

/// Toggle favorite status for a file
/// Returns the new favorite status (true/false)
#[tauri::command]
pub async fn toggle_favorite(file_id: i64, state: State<'_, AppState>) -> Result<bool, String> {
    log::info!("Toggling favorite for file {}", file_id);

    let pool = state.db_pool.lock().await;
    let repo = FileRepository::new(pool.clone());

    repo.toggle_favorite(file_id).await
}

/// Set favorite status for a file
#[tauri::command]
pub async fn set_favorite(
    file_id: i64,
    is_favorite: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting favorite for file {}: {}", file_id, is_favorite);

    let pool = state.db_pool.lock().await;
    let repo = FileRepository::new(pool.clone());

    repo.set_favorite(file_id, is_favorite).await
}

/// Get all favorite files
#[tauri::command]
pub async fn get_favorites(
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<File>, String> {
    let pool = state.db_pool.lock().await;
    let repo = FileRepository::new(pool.clone());

    repo.get_favorites(limit).await
}

/// Count favorite files
#[tauri::command]
pub async fn count_favorites(state: State<'_, AppState>) -> Result<i64, String> {
    let pool = state.db_pool.lock().await;
    let repo = FileRepository::new(pool.clone());

    repo.count_favorites().await
}

/// Get favorite files by folder
#[tauri::command]
pub async fn get_favorites_by_folder(
    folder_id: i64,
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<File>, String> {
    let pool = state.db_pool.lock().await;
    let repo = FileRepository::new(pool.clone());

    repo.get_favorites_by_folder(folder_id, limit).await
}
```

Update `app/src-tauri/src/commands/pipeline/vip3/mod.rs`:

```rust
pub mod favorites;
pub use favorites::*;
```

Register in `main.rs`:

```rust
use midi_app::commands::pipeline::{
    // ... existing ...
    toggle_favorite,
    set_favorite,
    get_favorites,
    count_favorites,
    get_favorites_by_folder,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...
            toggle_favorite,
            set_favorite,
            get_favorites,
            count_favorites,
            get_favorites_by_folder,
        ])
        // ...
}
```

---

## Step 4: Frontend Component (40 min)

Create `app/src/lib/components/VIP3/VIP3Favorites.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface FavoriteFile {
    id: number;
    file_name: string;
    file_path: string;
    folder_id: number;
    // Add other file fields as needed
  }

  let favorites: FavoriteFile[] = [];
  let favoriteCount = 0;
  let loading = false;

  onMount(async () => {
    await loadFavorites();
  });

  async function loadFavorites() {
    loading = true;
    try {
      [favorites, favoriteCount] = await Promise.all([
        invoke<FavoriteFile[]>('get_favorites', { limit: 50 }),
        invoke<number>('count_favorites')
      ]);
    } catch (error) {
      console.error('Failed to load favorites:', error);
    } finally {
      loading = false;
    }
  }

  async function toggleFavorite(fileId: number) {
    try {
      const isFavorite = await invoke<boolean>('toggle_favorite', { fileId });

      if (!isFavorite) {
        // Removed from favorites
        favorites = favorites.filter((f) => f.id !== fileId);
        favoriteCount--;
      }
    } catch (error) {
      console.error('Failed to toggle favorite:', error);
    }
  }

  function getFolderName(folderId: number): string {
    // In real implementation, look up folder name
    return `Folder ${folderId}`;
  }
</script>

<div class="favorites">
  <div class="header">
    <h3>Favorites</h3>
    <span class="count">{favoriteCount} files</span>
  </div>

  {#if loading}
    <p class="loading">Loading favorites...</p>
  {:else if favorites.length === 0}
    <p class="empty">
      No favorites yet.<br />
      Click the ★ icon on files to add them here.
    </p>
  {:else}
    <div class="favorite-list">
      {#each favorites as file (file.id)}
        <div class="favorite-item">
          <button
            class="star-btn active"
            on:click={() => toggleFavorite(file.id)}
            title="Remove from favorites"
          >
            ★
          </button>

          <div class="file-info">
            <span class="file-name">{file.file_name}</span>
            <span class="folder-name">{getFolderName(file.folder_id)}</span>
          </div>

          <button class="load-btn" title="Load to DAW">
            ▶
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .favorites {
    padding: 16px;
    background: #1a1a1a;
    border-right: 1px solid #333;
    min-width: 280px;
    max-width: 320px;
    overflow-y: auto;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .header h3 {
    margin: 0;
    font-size: 16px;
    color: #fff;
  }

  .count {
    font-size: 14px;
    color: #999;
  }

  .favorite-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .favorite-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px;
    background: #252525;
    border-radius: 4px;
  }

  .star-btn {
    padding: 4px 8px;
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    font-size: 18px;
    line-height: 1;
  }

  .star-btn.active {
    color: #fbbf24;
  }

  .star-btn:hover {
    color: #fcd34d;
  }

  .file-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .file-name {
    font-size: 14px;
    color: #fff;
    font-weight: 500;
  }

  .folder-name {
    font-size: 12px;
    color: #999;
  }

  .load-btn {
    padding: 6px 12px;
    background: #3b82f6;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .load-btn:hover {
    background: #2563eb;
  }

  .loading,
  .empty {
    padding: 24px;
    text-align: center;
    color: #666;
  }
</style>
```

Add star button to VIP3 results grid (in future VIP3Results component):

```svelte
<!-- Example: Add to each file result -->
<button
  class="star-btn"
  class:active={file.is_favorite}
  on:click={() => handleToggleFavorite(file.id)}
>
  ★
</button>
```

---

## Verification (10 min)

```bash
cargo check
make dev
```

Test in browser console:

```javascript
// Toggle favorite
const isFavorite = await window.__TAURI__.invoke('toggle_favorite', {
  fileId: 1
});
console.log('Is favorite:', isFavorite);

// Get all favorites
const favorites = await window.__TAURI__.invoke('get_favorites', { limit: 10 });
console.log('Favorites:', favorites);

// Count favorites
const count = await window.__TAURI__.invoke('count_favorites');
console.log('Total favorites:', count);
```

---

## What's Next?

✅ **Day 5 Part A Complete! Favorites system implemented:**
- ✅ Database column and indexes
- ✅ Toggle, set, get, count commands
- ✅ Favorites sidebar component

**Next:** [Part 5B: Category Management](./DAY5_PART_B_CATEGORIES.md) - Add/remove timbres, styles, articulations
