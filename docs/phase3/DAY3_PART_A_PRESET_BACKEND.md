# Day 3, Part 3A: Preset Backend

**Duration:** 2 hours
**Prerequisites:** Days 1-2 complete (automation system)
**Files to create:** 3

---

## Overview

Build preset save/load system:
1. Preset models (track, mixer, effect, project templates)
2. Preset repository
3. Serialization/deserialization
4. Database schema
5. Tauri commands

---

## Step 1: Preset Models (30 min)

Create `app/src-tauri/src/daw/presets/models.rs`:

```rust
use crate::daw::mixer::{TrackState, EffectChain};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Preset type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PresetType {
    /// Single track preset (all track settings)
    Track,

    /// Mixer preset (all tracks)
    Mixer,

    /// Single effect preset
    Effect,

    /// Project template (starter project)
    ProjectTemplate,
}

/// Preset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub preset_type: PresetType,
    pub description: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Serialized preset data (JSON)
    pub data: String,
}

impl Preset {
    pub fn new(id: String, name: String, preset_type: PresetType, data: String) -> Self {
        let now = chrono::Utc::now();

        Self {
            id,
            name,
            preset_type,
            description: None,
            author: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            data,
        }
    }
}

/// Track preset data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackPresetData {
    pub track_state: TrackState,
    pub effect_chain: EffectChain,
    pub automation_lanes: Vec<String>, // Lane IDs (optional)
}

impl TrackPresetData {
    pub fn new(track_state: TrackState, effect_chain: EffectChain) -> Self {
        Self {
            track_state,
            effect_chain,
            automation_lanes: Vec::new(),
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize track preset: {}", e))
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize track preset: {}", e))
    }
}

/// Mixer preset data (all tracks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerPresetData {
    pub tracks: HashMap<u32, TrackPresetData>,
    pub master_gain_db: f32,
    pub master_enabled: bool,
}

impl MixerPresetData {
    pub fn new() -> Self {
        Self {
            tracks: HashMap::new(),
            master_gain_db: 0.0,
            master_enabled: true,
        }
    }

    pub fn add_track(&mut self, track_id: u32, track_data: TrackPresetData) {
        self.tracks.insert(track_id, track_data);
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize mixer preset: {}", e))
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize mixer preset: {}", e))
    }
}

impl Default for MixerPresetData {
    fn default() -> Self {
        Self::new()
    }
}

/// Effect preset data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectPresetData {
    pub effect_type: String,
    pub name: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

impl EffectPresetData {
    pub fn new(effect_type: String, name: String) -> Self {
        Self {
            effect_type,
            name,
            parameters: HashMap::new(),
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize effect preset: {}", e))
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize effect preset: {}", e))
    }
}

/// Project template data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplateData {
    pub name: String,
    pub bpm: f32,
    pub time_signature: (u8, u8),
    pub mixer_preset: MixerPresetData,
    pub default_tracks: Vec<TrackPresetData>,
}

impl ProjectTemplateData {
    pub fn new(name: String, bpm: f32) -> Self {
        Self {
            name,
            bpm,
            time_signature: (4, 4),
            mixer_preset: MixerPresetData::new(),
            default_tracks: Vec::new(),
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize project template: {}", e))
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize project template: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_preset_serialization() {
        let track_state = TrackState::new(1, "Track 1".to_string());
        let effect_chain = EffectChain::new(1);
        let preset_data = TrackPresetData::new(track_state, effect_chain);

        let json = preset_data.to_json().unwrap();
        let deserialized = TrackPresetData::from_json(&json).unwrap();

        assert_eq!(deserialized.track_state.track_id, 1);
        assert_eq!(deserialized.track_state.name, "Track 1");
    }

    #[test]
    fn test_mixer_preset_serialization() {
        let mut mixer_preset = MixerPresetData::new();

        let track_state = TrackState::new(1, "Track 1".to_string());
        let effect_chain = EffectChain::new(1);
        let track_data = TrackPresetData::new(track_state, effect_chain);

        mixer_preset.add_track(1, track_data);

        let json = mixer_preset.to_json().unwrap();
        let deserialized = MixerPresetData::from_json(&json).unwrap();

        assert_eq!(deserialized.tracks.len(), 1);
        assert!(deserialized.tracks.contains_key(&1));
    }
}
```

---

## Step 2: Database Schema (30 min)

Add to `database/migrations/022_automation_presets_projects.sql`:

```sql
-- Presets table
CREATE TABLE IF NOT EXISTS presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    preset_type TEXT NOT NULL CHECK (preset_type IN ('Track', 'Mixer', 'Effect', 'ProjectTemplate')),
    description TEXT,
    author TEXT,
    tags TEXT[], -- Array of tags
    data TEXT NOT NULL, -- JSON data
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_presets_type ON presets(preset_type);
CREATE INDEX idx_presets_name ON presets(name);
CREATE INDEX idx_presets_tags ON presets USING GIN(tags);
CREATE INDEX idx_presets_created ON presets(created_at DESC);

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_preset_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER preset_update_timestamp
    BEFORE UPDATE ON presets
    FOR EACH ROW
    EXECUTE FUNCTION update_preset_timestamp();

-- Function to search presets by tag
CREATE OR REPLACE FUNCTION search_presets_by_tag(p_tag TEXT)
RETURNS TABLE(
    id TEXT,
    name TEXT,
    preset_type TEXT,
    created_at TIMESTAMP
) AS $$
BEGIN
    RETURN QUERY
    SELECT p.id, p.name, p.preset_type, p.created_at
    FROM presets p
    WHERE p_tag = ANY(p.tags)
    ORDER BY p.created_at DESC;
END;
$$ LANGUAGE plpgsql;
```

---

## Step 3: Preset Repository (40 min)

Create `app/src-tauri/src/daw/presets/repository.rs`:

```rust
use super::models::{Preset, PresetType};
use sqlx::PgPool;

pub struct PresetRepository {
    pool: PgPool,
}

impl PresetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Save preset
    pub async fn save(&self, preset: &Preset) -> Result<(), sqlx::Error> {
        let preset_type_str = format!("{:?}", preset.preset_type);
        let tags_array: Vec<&str> = preset.tags.iter().map(|s| s.as_str()).collect();

        sqlx::query!(
            r#"
            INSERT INTO presets (id, name, preset_type, description, author, tags, data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id)
            DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                author = EXCLUDED.author,
                tags = EXCLUDED.tags,
                data = EXCLUDED.data,
                updated_at = CURRENT_TIMESTAMP
            "#,
            preset.id,
            preset.name,
            preset_type_str,
            preset.description,
            preset.author,
            &tags_array,
            preset.data,
            preset.created_at,
            preset.updated_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load preset by ID
    pub async fn load(&self, preset_id: &str) -> Result<Option<Preset>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, preset_type, description, author, tags, data, created_at, updated_at
            FROM presets
            WHERE id = $1
            "#,
            preset_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let preset_type = match row.preset_type.as_str() {
                "Track" => PresetType::Track,
                "Mixer" => PresetType::Mixer,
                "Effect" => PresetType::Effect,
                "ProjectTemplate" => PresetType::ProjectTemplate,
                _ => PresetType::Track,
            };

            let tags = row.tags.unwrap_or_default();

            Ok(Some(Preset {
                id: row.id,
                name: row.name,
                preset_type,
                description: row.description,
                author: row.author,
                tags,
                data: row.data,
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
            }))
        } else {
            Ok(None)
        }
    }

    /// List all presets of a type
    pub async fn list_by_type(&self, preset_type: PresetType) -> Result<Vec<Preset>, sqlx::Error> {
        let type_str = format!("{:?}", preset_type);

        let rows = sqlx::query!(
            r#"
            SELECT id, name, preset_type, description, author, tags, data, created_at, updated_at
            FROM presets
            WHERE preset_type = $1
            ORDER BY created_at DESC
            "#,
            type_str,
        )
        .fetch_all(&self.pool)
        .await?;

        let presets = rows
            .into_iter()
            .map(|row| {
                let preset_type = match row.preset_type.as_str() {
                    "Track" => PresetType::Track,
                    "Mixer" => PresetType::Mixer,
                    "Effect" => PresetType::Effect,
                    "ProjectTemplate" => PresetType::ProjectTemplate,
                    _ => PresetType::Track,
                };

                let tags = row.tags.unwrap_or_default();

                Preset {
                    id: row.id,
                    name: row.name,
                    preset_type,
                    description: row.description,
                    author: row.author,
                    tags,
                    data: row.data,
                    created_at: row.created_at.and_utc(),
                    updated_at: row.updated_at.and_utc(),
                }
            })
            .collect();

        Ok(presets)
    }

    /// Delete preset
    pub async fn delete(&self, preset_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM presets WHERE id = $1", preset_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Search presets by name
    pub async fn search(&self, query: &str) -> Result<Vec<Preset>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);

        let rows = sqlx::query!(
            r#"
            SELECT id, name, preset_type, description, author, tags, data, created_at, updated_at
            FROM presets
            WHERE name ILIKE $1 OR description ILIKE $1
            ORDER BY created_at DESC
            "#,
            search_pattern,
        )
        .fetch_all(&self.pool)
        .await?;

        let presets = rows
            .into_iter()
            .map(|row| {
                let preset_type = match row.preset_type.as_str() {
                    "Track" => PresetType::Track,
                    "Mixer" => PresetType::Mixer,
                    "Effect" => PresetType::Effect,
                    "ProjectTemplate" => PresetType::ProjectTemplate,
                    _ => PresetType::Track,
                };

                let tags = row.tags.unwrap_or_default();

                Preset {
                    id: row.id,
                    name: row.name,
                    preset_type,
                    description: row.description,
                    author: row.author,
                    tags,
                    data: row.data,
                    created_at: row.created_at.and_utc(),
                    updated_at: row.updated_at.and_utc(),
                }
            })
            .collect();

        Ok(presets)
    }
}
```

Add to repositories mod:

```rust
// In app/src-tauri/src/db/repositories/mod.rs
pub mod preset_repository;
pub use preset_repository::PresetRepository;
```

---

## Step 4: Tauri Commands (15 min)

Create `app/src-tauri/src/commands/daw/preset_commands.rs`:

```rust
use crate::daw::presets::models::{Preset, PresetType, TrackPresetData};
use crate::db::repositories::PresetRepository;
use crate::AppState;
use tauri::State;

/// Save preset
#[tauri::command]
pub async fn save_preset(
    preset: Preset,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Saving preset: {} ({})", preset.name, preset.id);

    let repo = PresetRepository::new(state.db_pool.clone());
    repo.save(&preset)
        .await
        .map_err(|e| format!("Failed to save preset: {}", e))
}

/// Load preset
#[tauri::command]
pub async fn load_preset(
    preset_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Preset>, String> {
    log::info!("Loading preset: {}", preset_id);

    let repo = PresetRepository::new(state.db_pool.clone());
    repo.load(&preset_id)
        .await
        .map_err(|e| format!("Failed to load preset: {}", e))
}

/// List presets by type
#[tauri::command]
pub async fn list_presets(
    preset_type: PresetType,
    state: State<'_, AppState>,
) -> Result<Vec<Preset>, String> {
    log::info!("Listing presets of type: {:?}", preset_type);

    let repo = PresetRepository::new(state.db_pool.clone());
    repo.list_by_type(preset_type)
        .await
        .map_err(|e| format!("Failed to list presets: {}", e))
}

/// Delete preset
#[tauri::command]
pub async fn delete_preset(
    preset_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Deleting preset: {}", preset_id);

    let repo = PresetRepository::new(state.db_pool.clone());
    repo.delete(&preset_id)
        .await
        .map_err(|e| format!("Failed to delete preset: {}", e))
}

/// Search presets
#[tauri::command]
pub async fn search_presets(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<Preset>, String> {
    log::info!("Searching presets: {}", query);

    let repo = PresetRepository::new(state.db_pool.clone());
    repo.search(&query)
        .await
        .map_err(|e| format!("Failed to search presets: {}", e))
}
```

Register commands in `main.rs`:

```rust
use midi_app::commands::daw::{
    save_preset,
    load_preset,
    list_presets,
    delete_preset,
    search_presets,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing
            save_preset,
            load_preset,
            list_presets,
            delete_preset,
            search_presets,
        ])
        // ...
}
```

---

## Step 5: Module Organization (5 min)

Create `app/src-tauri/src/daw/presets/mod.rs`:

```rust
pub mod models;
pub mod repository;

pub use models::{
    Preset, PresetType, TrackPresetData, MixerPresetData,
    EffectPresetData, ProjectTemplateData,
};
pub use repository::PresetRepository;
```

Update `app/src-tauri/src/daw/mod.rs`:

```rust
pub mod automation;
pub mod mixer;
pub mod sequencer;
pub mod presets;  // NEW
```

---

## Verification (15 min)

```bash
cd app/src-tauri
cargo check
cargo test --lib presets
```

Test in browser console:

```javascript
// Create track preset
const trackState = { track_id: 1, name: 'Piano', gain_db: 0, pan: 0, muted: false, soloed: false, enabled: true };
const effectChain = { track_id: 1, effects: [] };
const presetData = { track_state: trackState, effect_chain: effectChain, automation_lanes: [] };

const preset = {
  id: 'piano-preset-1',
  name: 'Bright Piano',
  preset_type: 'Track',
  description: 'Bright piano sound with reverb',
  author: 'User',
  tags: ['piano', 'bright'],
  data: JSON.stringify(presetData),
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

await window.__TAURI__.invoke('save_preset', { preset });

// List track presets
const presets = await window.__TAURI__.invoke('list_presets', { presetType: 'Track' });
console.log('Track presets:', presets);

// Load preset
const loaded = await window.__TAURI__.invoke('load_preset', { presetId: 'piano-preset-1' });
console.log('Loaded preset:', loaded);

// Search presets
const found = await window.__TAURI__.invoke('search_presets', { query: 'piano' });
console.log('Found presets:', found);
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Serialization error | Check JSON format, verify all fields serializable |
| Database constraint error | Verify unique ID, check preset_type enum values |
| Load fails | Ensure preset exists, check deserialization logic |
| Tags not working | Verify PostgreSQL array syntax, check GIN index |

---

## What's Next?

✅ **You've completed:**
- Preset models for track, mixer, effect, project templates
- Database schema with search and tagging
- PresetRepository for save/load operations
- 5 Tauri commands for preset management
- JSON serialization/deserialization

**Next:** [Part 3B: Preset Frontend](./DAY3_PART_B_PRESET_FRONTEND.md)
- Preset browser UI
- Save/load dialogs
- Preset preview
- Tag filtering
