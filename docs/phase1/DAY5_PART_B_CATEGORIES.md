# Day 5, Part 5B: Category Management

**Duration:** 2 hours
**Prerequisites:** Days 1-5A completed
**Files to create:** 2

---

## Overview

Complete VIP3 category management:
1. Create commands to add/remove timbres, styles, articulations
2. Create commands to manage category definitions (CRUD)
3. Build category management UI
4. Allow tagging files with categories

**Note:** Tables `timbres`, `styles`, `articulations`, `file_timbres`, `file_styles`, `file_articulations` already exist from migration 001.

---

## Step 1: Repository Methods (45 min)

Update `app/src-tauri/src/db/repositories/vip3_repository.rs`:

```rust
impl Vip3Repository {
    // ... existing filter count methods ...

    // ============================================================================
    // TIMBRE MANAGEMENT
    // ============================================================================

    /// Get all timbres
    pub async fn get_all_timbres(&self) -> Result<Vec<Timbre>, String> {
        sqlx::query_as::<_, Timbre>(
            "SELECT id, name, description FROM timbres ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch timbres: {}", e))
    }

    /// Create a new timbre
    pub async fn create_timbre(&self, name: &str, description: Option<&str>) -> Result<Timbre, String> {
        sqlx::query_as::<_, Timbre>(
            "INSERT INTO timbres (name, description) VALUES ($1, $2)
             RETURNING id, name, description"
        )
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create timbre: {}", e))
    }

    /// Delete a timbre (and all file associations)
    pub async fn delete_timbre(&self, id: i32) -> Result<(), String> {
        sqlx::query("DELETE FROM timbres WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete timbre: {}", e))?;

        Ok(())
    }

    /// Add timbre to a file
    pub async fn add_timbre_to_file(&self, file_id: i64, timbre_id: i32) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO file_timbres (file_id, timbre_id)
             VALUES ($1, $2)
             ON CONFLICT (file_id, timbre_id) DO NOTHING"
        )
        .bind(file_id)
        .bind(timbre_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to add timbre to file: {}", e))?;

        log::debug!("Added timbre {} to file {}", timbre_id, file_id);

        Ok(())
    }

    /// Remove timbre from a file
    pub async fn remove_timbre_from_file(&self, file_id: i64, timbre_id: i32) -> Result<(), String> {
        sqlx::query("DELETE FROM file_timbres WHERE file_id = $1 AND timbre_id = $2")
            .bind(file_id)
            .bind(timbre_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to remove timbre from file: {}", e))?;

        log::debug!("Removed timbre {} from file {}", timbre_id, file_id);

        Ok(())
    }

    /// Get all timbres for a file
    pub async fn get_file_timbres(&self, file_id: i64) -> Result<Vec<Timbre>, String> {
        sqlx::query_as::<_, Timbre>(
            "SELECT t.id, t.name, t.description
             FROM timbres t
             JOIN file_timbres ft ON t.id = ft.timbre_id
             WHERE ft.file_id = $1
             ORDER BY t.name"
        )
        .bind(file_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch file timbres: {}", e))
    }

    // ============================================================================
    // STYLE MANAGEMENT
    // ============================================================================

    /// Get all styles
    pub async fn get_all_styles(&self) -> Result<Vec<Style>, String> {
        sqlx::query_as::<_, Style>(
            "SELECT id, name, description FROM styles ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch styles: {}", e))
    }

    /// Create a new style
    pub async fn create_style(&self, name: &str, description: Option<&str>) -> Result<Style, String> {
        sqlx::query_as::<_, Style>(
            "INSERT INTO styles (name, description) VALUES ($1, $2)
             RETURNING id, name, description"
        )
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create style: {}", e))
    }

    /// Delete a style
    pub async fn delete_style(&self, id: i32) -> Result<(), String> {
        sqlx::query("DELETE FROM styles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete style: {}", e))?;

        Ok(())
    }

    /// Add style to a file
    pub async fn add_style_to_file(&self, file_id: i64, style_id: i32) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO file_styles (file_id, style_id)
             VALUES ($1, $2)
             ON CONFLICT (file_id, style_id) DO NOTHING"
        )
        .bind(file_id)
        .bind(style_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to add style to file: {}", e))?;

        Ok(())
    }

    /// Remove style from a file
    pub async fn remove_style_from_file(&self, file_id: i64, style_id: i32) -> Result<(), String> {
        sqlx::query("DELETE FROM file_styles WHERE file_id = $1 AND style_id = $2")
            .bind(file_id)
            .bind(style_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to remove style from file: {}", e))?;

        Ok(())
    }

    /// Get all styles for a file
    pub async fn get_file_styles(&self, file_id: i64) -> Result<Vec<Style>, String> {
        sqlx::query_as::<_, Style>(
            "SELECT s.id, s.name, s.description
             FROM styles s
             JOIN file_styles fs ON s.id = fs.style_id
             WHERE fs.file_id = $1
             ORDER BY s.name"
        )
        .bind(file_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch file styles: {}", e))
    }

    // ============================================================================
    // ARTICULATION MANAGEMENT
    // ============================================================================

    /// Get all articulations
    pub async fn get_all_articulations(&self) -> Result<Vec<Articulation>, String> {
        sqlx::query_as::<_, Articulation>(
            "SELECT id, name, description FROM articulations ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch articulations: {}", e))
    }

    /// Create a new articulation
    pub async fn create_articulation(&self, name: &str, description: Option<&str>) -> Result<Articulation, String> {
        sqlx::query_as::<_, Articulation>(
            "INSERT INTO articulations (name, description) VALUES ($1, $2)
             RETURNING id, name, description"
        )
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create articulation: {}", e))
    }

    /// Delete an articulation
    pub async fn delete_articulation(&self, id: i32) -> Result<(), String> {
        sqlx::query("DELETE FROM articulations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete articulation: {}", e))?;

        Ok(())
    }

    /// Add articulation to a file
    pub async fn add_articulation_to_file(&self, file_id: i64, articulation_id: i32) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO file_articulations (file_id, articulation_id)
             VALUES ($1, $2)
             ON CONFLICT (file_id, articulation_id) DO NOTHING"
        )
        .bind(file_id)
        .bind(articulation_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to add articulation to file: {}", e))?;

        Ok(())
    }

    /// Remove articulation from a file
    pub async fn remove_articulation_from_file(&self, file_id: i64, articulation_id: i32) -> Result<(), String> {
        sqlx::query("DELETE FROM file_articulations WHERE file_id = $1 AND articulation_id = $2")
            .bind(file_id)
            .bind(articulation_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to remove articulation from file: {}", e))?;

        Ok(())
    }

    /// Get all articulations for a file
    pub async fn get_file_articulations(&self, file_id: i64) -> Result<Vec<Articulation>, String> {
        sqlx::query_as::<_, Articulation>(
            "SELECT a.id, a.name, a.description
             FROM articulations a
             JOIN file_articulations fa ON a.id = fa.articulation_id
             WHERE fa.file_id = $1
             ORDER BY a.name"
        )
        .bind(file_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch file articulations: {}", e))
    }
}
```

Add type definitions to `app/src-tauri/src/db/models/vip3_filters.rs`:

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Timbre {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Style {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Articulation {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}
```

---

## Step 2: Tauri Commands (30 min)

Create `app/src-tauri/src/commands/pipeline/vip3/categories.rs`:

```rust
use crate::db::{
    models::{Articulation, Style, Timbre},
    repositories::Vip3Repository,
};
use crate::AppState;
use tauri::State;

// ============================================================================
// TIMBRE COMMANDS
// ============================================================================

#[tauri::command]
pub async fn get_all_timbres(state: State<'_, AppState>) -> Result<Vec<Timbre>, String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.get_all_timbres().await
}

#[tauri::command]
pub async fn create_timbre(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<Timbre, String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.create_timbre(&name, description.as_deref()).await
}

#[tauri::command]
pub async fn delete_timbre(id: i32, state: State<'_, AppState>) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.delete_timbre(id).await
}

#[tauri::command]
pub async fn add_timbre_to_file(
    file_id: i64,
    timbre_id: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.add_timbre_to_file(file_id, timbre_id).await
}

#[tauri::command]
pub async fn remove_timbre_from_file(
    file_id: i64,
    timbre_id: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.remove_timbre_from_file(file_id, timbre_id).await
}

// ============================================================================
// STYLE COMMANDS
// ============================================================================

#[tauri::command]
pub async fn get_all_styles(state: State<'_, AppState>) -> Result<Vec<Style>, String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.get_all_styles().await
}

#[tauri::command]
pub async fn create_style(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<Style, String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.create_style(&name, description.as_deref()).await
}

#[tauri::command]
pub async fn delete_style(id: i32, state: State<'_, AppState>) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.delete_style(id).await
}

#[tauri::command]
pub async fn add_style_to_file(
    file_id: i64,
    style_id: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.add_style_to_file(file_id, style_id).await
}

#[tauri::command]
pub async fn remove_style_from_file(
    file_id: i64,
    style_id: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.remove_style_from_file(file_id, style_id).await
}

// ============================================================================
// ARTICULATION COMMANDS
// ============================================================================

#[tauri::command]
pub async fn get_all_articulations(
    state: State<'_, AppState>,
) -> Result<Vec<Articulation>, String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.get_all_articulations().await
}

#[tauri::command]
pub async fn create_articulation(
    name: String,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<Articulation, String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.create_articulation(&name, description.as_deref()).await
}

#[tauri::command]
pub async fn delete_articulation(id: i32, state: State<'_, AppState>) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.delete_articulation(id).await
}

#[tauri::command]
pub async fn add_articulation_to_file(
    file_id: i64,
    articulation_id: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.add_articulation_to_file(file_id, articulation_id).await
}

#[tauri::command]
pub async fn remove_articulation_from_file(
    file_id: i64,
    articulation_id: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state.db_pool.lock().await;
    let repo = Vip3Repository::new(pool.clone());
    repo.remove_articulation_from_file(file_id, articulation_id).await
}
```

Update mod.rs and main.rs to register all commands.

---

## Step 3: Category Management UI (45 min)

Create `app/src/lib/components/VIP3/VIP3CategoryManager.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface Category {
    id: number;
    name: string;
    description?: string;
  }

  type CategoryType = 'timbre' | 'style' | 'articulation';

  export let categoryType: CategoryType;

  let categories: Category[] = [];
  let showCreateDialog = false;
  let newName = '';
  let newDescription = '';

  const typeNames = {
    timbre: 'Timbre',
    style: 'Style',
    articulation: 'Articulation'
  };

  const commands = {
    timbre: {
      getAll: 'get_all_timbres',
      create: 'create_timbre',
      delete: 'delete_timbre'
    },
    style: {
      getAll: 'get_all_styles',
      create: 'create_style',
      delete: 'delete_style'
    },
    articulation: {
      getAll: 'get_all_articulations',
      create: 'create_articulation',
      delete: 'delete_articulation'
    }
  };

  onMount(async () => {
    await loadCategories();
  });

  async function loadCategories() {
    const cmd = commands[categoryType].getAll;
    categories = await invoke<Category[]>(cmd);
  }

  async function createCategory() {
    if (!newName.trim()) return;

    try {
      const cmd = commands[categoryType].create;
      await invoke(cmd, {
        name: newName,
        description: newDescription || undefined
      });

      await loadCategories();
      newName = '';
      newDescription = '';
      showCreateDialog = false;
    } catch (error) {
      alert(`Failed to create ${typeNames[categoryType]}: ${error}`);
    }
  }

  async function deleteCategory(id: number, name: string) {
    if (!confirm(`Delete ${typeNames[categoryType]} "${name}"?`)) return;

    try {
      const cmd = commands[categoryType].delete;
      await invoke(cmd, { id });
      await loadCategories();
    } catch (error) {
      alert(`Failed to delete: ${error}`);
    }
  }
</script>

<div class="category-manager">
  <div class="header">
    <h3>{typeNames[categoryType]}s</h3>
    <button class="create-btn" on:click={() => (showCreateDialog = true)}>
      + Add
    </button>
  </div>

  <div class="category-list">
    {#each categories as category (category.id)}
      <div class="category-item">
        <div class="category-info">
          <span class="name">{category.name}</span>
          {#if category.description}
            <span class="desc">{category.description}</span>
          {/if}
        </div>
        <button
          class="delete-btn"
          on:click={() => deleteCategory(category.id, category.name)}
        >
          ×
        </button>
      </div>
    {/each}
  </div>
</div>

{#if showCreateDialog}
  <div class="dialog-overlay" on:click={() => (showCreateDialog = false)}>
    <div class="dialog" on:click|stopPropagation>
      <h3>New {typeNames[categoryType]}</h3>
      <label>
        Name *
        <input type="text" bind:value={newName} placeholder="e.g., Bright, Dark" />
      </label>
      <label>
        Description
        <textarea bind:value={newDescription} rows="2" />
      </label>
      <div class="dialog-actions">
        <button on:click={() => (showCreateDialog = false)}>Cancel</button>
        <button class="primary" on:click={createCategory}>Create</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .category-manager {
    padding: 16px;
  }

  .header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .header h3 {
    margin: 0;
    font-size: 16px;
    color: #fff;
  }

  .create-btn {
    padding: 4px 12px;
    background: #3b82f6;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }

  .category-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .category-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    background: #252525;
    border-radius: 4px;
  }

  .category-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .name {
    color: #fff;
    font-size: 14px;
  }

  .desc {
    color: #999;
    font-size: 12px;
  }

  .delete-btn {
    padding: 4px 8px;
    background: none;
    border: none;
    color: #666;
    cursor: pointer;
    font-size: 18px;
  }

  .delete-btn:hover {
    background: #991b1b;
    color: #fff;
    border-radius: 4px;
  }

  /* Dialog styles */
  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: #2a2a2a;
    padding: 24px;
    border-radius: 8px;
    width: 90%;
    max-width: 400px;
  }

  .dialog h3 {
    margin: 0 0 16px 0;
    color: #fff;
  }

  .dialog label {
    display: block;
    margin-bottom: 12px;
    color: #ccc;
    font-size: 14px;
  }

  .dialog input,
  .dialog textarea {
    width: 100%;
    margin-top: 4px;
    padding: 8px;
    background: #1a1a1a;
    border: 1px solid #444;
    border-radius: 4px;
    color: #fff;
    font-family: inherit;
  }

  .dialog-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 16px;
  }

  .dialog-actions button {
    padding: 6px 16px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .dialog-actions button.primary {
    background: #3b82f6;
    color: #fff;
  }
</style>
```

---

## Verification (10 min)

```bash
cargo check
make dev
```

Test creating categories:

```javascript
// Create timbre
const timbre = await window.__TAURI__.invoke('create_timbre', {
  name: 'Bright',
  description: 'Bright, clear sound'
});

// Get all timbres
const timbres = await window.__TAURI__.invoke('get_all_timbres');
console.log('Timbres:', timbres);

// Tag a file
await window.__TAURI__.invoke('add_timbre_to_file', {
  fileId: 1,
  timbreId: timbre.id
});
```

---

## Phase 1 Complete!

🎉 **All 11 parts implemented:**

### Day 1-2: Filter Counts System
- ✅ Part 1A: Backend models & repository
- ✅ Part 1B: Tauri commands with parallel execution
- ✅ Part 1C: Frontend API & Svelte store
- ✅ Part 1D: UI components with real-time counts
- ✅ Day 2: Database optimization (<50ms performance)

### Day 3: Saved Searches
- ✅ Part 3A: Backend (models, repository, commands)
- ✅ Part 3B: Frontend (API, UI, save/load flow)

### Day 4: Collections
- ✅ Part 4A: Backend (models, repository, ordering)
- ✅ Part 4B: Frontend (UI with drag-and-drop)

### Day 5: Favorites & Categories
- ✅ Part 5A: Favorites system
- ✅ Part 5B: Category management (timbres, styles, articulations)

---

## What's Next?

**Phase 2: DAW Mixer System** (See `../IMPLEMENTATION_ROADMAP.md`)
- Full mixer implementation (30+ commands)
- VU metering with real-time updates
- Effect chain management
- Automation recording/playback

**Or continue VIP3 polish:**
- VIP3Results grid component
- Drag files from VIP3 to DAW
- Implement `load_file_to_daw` convenience wrapper
