# VIP3 Mock Data Replacement Guide

## Overview

You've completed VIP3 Part 1D with mock data. This guide explains how to replace mock data with real database queries.

## Schema Analysis Complete ✅

I've analyzed your actual database schema and found:

### Database Structure

| Category | Storage | Column/Table |
|----------|---------|--------------|
| **Folders** | `files.parent_folder` (TEXT) | ✅ Exists |
| **Instruments** | `tags` table where `category = 'instrument'` | ✅ Exists |
| **Timbres** | `timbres` table | ✅ Exists (migration 019) |
| **Styles** | `styles` table | ✅ Exists (migration 019) |
| **Articulations** | `articulations` table | ✅ Exists (migration 019) |
| **Manufacturers** | `files.manufacturer` (TEXT) | ✅ Exists |
| **BPM Ranges** | `bpm_ranges` table | ✅ Exists (migration 019) |
| **Musical Keys** | `musical_keys` table | ✅ Exists (migration 019) |

---

## What I've Completed

### ✅ Backend Commands (Rust)

**File: `app/src-tauri/src/commands/pipeline/vip3/categories.rs`**

Added 6 new commands:
- `get_vip3_folders()` - Returns all unique `parent_folder` values
- `get_vip3_instruments()` - Returns instrument names from tags (category='instrument')
- `get_vip3_timbres()` - Returns timbre names from timbres table
- `get_vip3_styles()` - Returns style names from styles table
- `get_vip3_articulations()` - Returns articulation names from articulations table
- `get_vip3_manufacturers()` - Returns all unique manufacturers

**File: `app/src-tauri/src/commands/pipeline/vip3/mod.rs`**

Exported the new commands so they can be used.

**File: `app/src-tauri/src/main.rs`** (lines 261-267)

Registered the new commands in Tauri's `invoke_handler`:
```rust
// VIP3 Browser - Category List Fetching (for initialization)
midi_app::commands::pipeline::vip3::categories::get_vip3_folders,
midi_app::commands::pipeline::vip3::categories::get_vip3_instruments,
midi_app::commands::pipeline::vip3::categories::get_vip3_timbres,
midi_app::commands::pipeline::vip3::categories::get_vip3_styles,
midi_app::commands::pipeline::vip3::categories::get_vip3_articulations,
midi_app::commands::pipeline::vip3::categories::get_vip3_manufacturers,
```

### ✅ Frontend API (TypeScript)

**File: `app/src/lib/api/vip3BrowserApi.ts`**

Added 6 new methods:
- `getFolders()` - Invokes `get_vip3_folders`
- `getInstruments()` - Invokes `get_vip3_instruments`
- `getTimbreNames()` - Invokes `get_vip3_timbres`
- `getStyleNames()` - Invokes `get_vip3_styles`
- `getArticulationNames()` - Invokes `get_vip3_articulations`
- `getManufacturers()` - Invokes `get_vip3_manufacturers`

---

## What You Need to Do

### Current State

Your `vip3Store.ts` currently calls:
```typescript
const categories = await vip3BrowserApi.getAllCategories();
```

This tries to invoke a backend command `vip3_get_all_categories` that **doesn't exist yet**.

### Two Options

#### Option 1: Create Combined Backend Command (Recommended - More Efficient)

Create a single Rust command that fetches all categories in parallel.

**Add to `app/src-tauri/src/commands/pipeline/vip3/categories.rs`:**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AllCategories {
    pub folders: Vec<String>,
    pub instruments: Vec<String>,
    pub timbres: Vec<String>,
    pub styles: Vec<String>,
    pub articulations: Vec<String>,
    pub manufacturers: Vec<String>,
}

/// Get all VIP3 categories in a single call (more efficient than 6 separate calls)
#[tauri::command]
pub async fn get_all_vip3_categories(state: State<'_, AppState>) -> Result<AllCategories, String> {
    let pool = state.database.pool().await;

    // Fetch all categories in parallel using tokio::try_join!
    let (folders, instruments, timbres, styles, articulations, manufacturers) = tokio::try_join!(
        sqlx::query_scalar!("SELECT DISTINCT parent_folder FROM files WHERE parent_folder IS NOT NULL ORDER BY parent_folder").fetch_all(&pool),
        sqlx::query_scalar!("SELECT DISTINCT name FROM tags WHERE category = 'instrument' AND is_active = true ORDER BY name").fetch_all(&pool),
        sqlx::query_scalar!("SELECT name FROM timbres ORDER BY sort_order, name").fetch_all(&pool),
        sqlx::query_scalar!("SELECT name FROM styles ORDER BY sort_order, name").fetch_all(&pool),
        sqlx::query_scalar!("SELECT name FROM articulations ORDER BY sort_order, name").fetch_all(&pool),
        sqlx::query_scalar!("SELECT DISTINCT manufacturer FROM files WHERE manufacturer IS NOT NULL ORDER BY manufacturer").fetch_all(&pool),
    ).map_err(|e| format!("Failed to fetch categories: {}", e))?;

    Ok(AllCategories {
        folders,
        instruments,
        timbres,
        styles,
        articulations,
        manufacturers,
    })
}
```

**Update `mod.rs` to export it:**
```rust
pub use categories::{
    // ... existing exports ...
    get_all_vip3_categories,
};
```

**Register in `main.rs`:**
```rust
midi_app::commands::pipeline::vip3::categories::get_all_vip3_categories,
```

**Update `vip3Browser.ts` to use the correct command:**
```typescript
getAllCategories: async (): Promise<VIP3Categories> => {
  try {
    const result = await invoke<AllCategories>('get_all_vip3_categories');

    // Convert to VIP3Categories format expected by store
    return {
      timbres: result.timbres.map((name, idx) => ({ id: idx + 1, name, description: '', file_count: 0 })),
      styles: result.styles.map((name, idx) => ({ id: idx + 1, name, description: '', file_count: 0 })),
      articulations: result.articulations.map((name, idx) => ({ id: idx + 1, name, description: '', file_count: 0 })),
      bpmRanges: [], // Fetch separately if needed
      musicalKeys: [], // Fetch separately if needed
    };
  } catch (error) {
    console.error('[vip3BrowserApi] Failed to get categories:', error);
    throw error;
  }
},
```

#### Option 2: Use Individual API Calls (Simpler to Implement)

Update `vip3Store.ts` to call the individual methods:

```typescript
// In vip3Actions.loadCategories():
async loadCategories() {
  vip3Store.update((s) => ({ ...s, isLoading: true, error: null }));
  try {
    // Use the new API methods from Vip3BrowserApi
    const [folders, instruments, timbreNames, styleNames, articulationNames, manufacturers] = await Promise.all([
      Vip3BrowserApi.getFolders(),
      Vip3BrowserApi.getInstruments(),
      Vip3BrowserApi.getTimbreNames(),
      Vip3BrowserApi.getStyleNames(),
      Vip3BrowserApi.getArticulationNames(),
      Vip3BrowserApi.getManufacturers(),
    ]);

    // Convert to the format expected by the store
    const timbres: Timbre[] = timbreNames.map((name, idx) => ({
      id: idx + 1,
      name,
      description: '',
      file_count: 0,
    }));

    const styles: Style[] = styleNames.map((name, idx) => ({
      id: idx + 1,
      name,
      description: '',
      file_count: 0,
    }));

    const articulations: Articulation[] = articulationNames.map((name, idx) => ({
      id: idx + 1,
      name,
      description: '',
      file_count: 0,
    }));

    vip3Store.update((s) => ({
      ...s,
      timbres,
      styles,
      articulations,
      isLoading: false,
    }));

    console.log(`✓ Loaded categories: ${timbres.length} timbres, ${styles.length} styles, ${articulations.length} articulations`);
  } catch (error) {
    console.error('Failed to load categories:', error);
    vip3Store.update((s) => ({
      ...s,
      isLoading: false,
      error: error instanceof Error ? error.message : 'Failed to load categories',
    }));
  }
}
```

---

## Verification Steps

### 1. Check Database Has Data

```bash
# Check folders
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c \
  "SELECT COUNT(DISTINCT parent_folder) FROM files WHERE parent_folder IS NOT NULL;"

# Check instruments
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c \
  "SELECT COUNT(*) FROM tags WHERE category = 'instrument';"

# Check timbres
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c \
  "SELECT COUNT(*) FROM timbres;"

# Check styles
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c \
  "SELECT COUNT(*) FROM styles;"

# Check articulations
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" -c \
  "SELECT COUNT(*) FROM articulations;"
```

### 2. Build Backend

```bash
cd app/src-tauri
cargo build
```

### 3. Test Commands in Browser DevTools

```javascript
// Test individual commands
invoke('get_vip3_folders').then(console.log);
invoke('get_vip3_instruments').then(console.log);
invoke('get_vip3_timbres').then(console.log);
invoke('get_vip3_styles').then(console.log);
invoke('get_vip3_articulations').then(console.log);

// Or test the combined command (if you implemented Option 1)
invoke('get_all_vip3_categories').then(console.log);
```

### 4. Run Frontend

```bash
cd app
pnpm run dev
```

### 5. Verify in VIP3 Browser

- Open VIP3 Browser tab
- Check that real data appears instead of mock data
- Verify filter counts are accurate
- Test filtering works correctly

---

## Key Differences from My Initial Proposal

| Aspect | What I Initially Suggested | What Actually Exists |
|--------|----------------------------|---------------------|
| Folders column | `folder_name` | `parent_folder` ✅ |
| Instruments source | Assumed tags table | Confirmed tags with category='instrument' ✅ |
| VIP3 categories | Assumed tags table | Dedicated tables exist (timbres, styles, articulations) ✅ |
| API structure | Single vip3Api.ts | Two files: vip3Browser.ts + vip3BrowserApi.ts 📝 |

---

## Recommendations

I recommend **Option 1** (combined backend command) because:

1. **Performance**: Single round-trip to database instead of 6
2. **Atomicity**: All categories fetched together or none
3. **Consistency**: Data snapshot is consistent across all categories
4. **Efficiency**: Parallel query execution with `tokio::try_join!`

The individual commands I created are still useful for:
- Refreshing a single category without fetching all
- Future incremental updates
- Flexibility in frontend implementation

---

## Next Steps

1. Choose Option 1 or Option 2
2. Implement the chosen option
3. Run verification steps
4. Replace any remaining mock data in components
5. Test the complete VIP3 workflow

Let me know which option you'd like to implement and I can help with the specific code!
