# Day 4, Part 4A: Project Models

**Duration:** 2.5 hours
**Prerequisites:** Days 1-3 complete (automation, presets)
**Files to create:** 3

---

## Overview

Build project save/load system:
1. Project models (session state)
2. Session serializer
3. Project repository
4. Database schema
5. Tauri commands

---

## Step 1: Project Models (40 min)

Create `app/src-tauri/src/daw/project/models.rs`:

```rust
use crate::daw::automation::AutomationLane;
use crate::daw::mixer::{MixerConfig, RoutingConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub file_path: Option<PathBuf>,
    pub bpm: f32,
    pub time_signature: (u8, u8),
    pub sample_rate: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_opened_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Project session state (serialized JSON)
    pub session_data: String,
}

impl Project {
    pub fn new(id: String, name: String) -> Self {
        let now = chrono::Utc::now();

        Self {
            id,
            name,
            file_path: None,
            bpm: 120.0,
            time_signature: (4, 4),
            sample_rate: 48000,
            created_at: now,
            updated_at: now,
            last_opened_at: Some(now),
            session_data: String::new(),
        }
    }

    pub fn with_file_path(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }
}

/// Complete session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Project metadata
    pub project_name: String,
    pub bpm: f32,
    pub time_signature: (u8, u8),
    pub sample_rate: u32,

    /// Mixer state
    pub mixer: MixerConfig,

    /// Routing state
    pub routing: RoutingConfig,

    /// Automation lanes
    pub automation_lanes: Vec<AutomationLane>,

    /// Playhead position in ticks
    pub playhead_position: u64,

    /// Loop settings
    pub loop_enabled: bool,
    pub loop_start_ticks: u64,
    pub loop_end_ticks: u64,

    /// MIDI tracks (track_id -> MIDI file path)
    pub midi_files: std::collections::HashMap<u32, PathBuf>,
}

impl SessionState {
    pub fn new(project_name: String) -> Self {
        Self {
            project_name,
            bpm: 120.0,
            time_signature: (4, 4),
            sample_rate: 48000,
            mixer: MixerConfig::default(),
            routing: RoutingConfig::default(),
            automation_lanes: Vec::new(),
            playhead_position: 0,
            loop_enabled: false,
            loop_start_ticks: 0,
            loop_end_ticks: 1920, // One bar at 480 PPQ
            midi_files: std::collections::HashMap::new(),
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize session: {}", e))
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json)
            .map_err(|e| format!("Failed to deserialize session: {}", e))
    }
}

/// Recent project entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub id: String,
    pub name: String,
    pub file_path: Option<PathBuf>,
    pub last_opened_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_serialization() {
        let session = SessionState::new("Test Project".to_string());

        let json = session.to_json().unwrap();
        let deserialized = SessionState::from_json(&json).unwrap();

        assert_eq!(deserialized.project_name, "Test Project");
        assert_eq!(deserialized.bpm, 120.0);
        assert_eq!(deserialized.time_signature, (4, 4));
    }

    #[test]
    fn test_project_creation() {
        let project = Project::new("proj1".to_string(), "My Project".to_string());

        assert_eq!(project.id, "proj1");
        assert_eq!(project.name, "My Project");
        assert_eq!(project.bpm, 120.0);
        assert!(project.last_opened_at.is_some());
    }
}
```

---

## Step 2: Session Serializer (30 min)

Create `app/src-tauri/src/daw/project/serializer.rs`:

```rust
use super::models::SessionState;
use crate::daw::automation::AutomationRecorder;
use crate::daw::mixer::MixerConfig;
use crate::daw::mixer::routing::RoutingConfig;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Serialize current session state
pub async fn serialize_session(
    project_name: String,
    mixer: Arc<Mutex<MixerConfig>>,
    routing: Arc<Mutex<RoutingConfig>>,
    automation: Arc<AutomationRecorder>,
    playhead_position: u64,
) -> Result<SessionState, String> {
    let mixer_config = mixer.lock().await.clone();
    let routing_config = routing.lock().await.clone();
    let automation_lanes = automation.get_all_lanes().await;

    let mut session = SessionState::new(project_name);
    session.mixer = mixer_config;
    session.routing = routing_config;
    session.automation_lanes = automation_lanes;
    session.playhead_position = playhead_position;

    Ok(session)
}

/// Deserialize and restore session state
pub async fn deserialize_session(
    session: SessionState,
    mixer: Arc<Mutex<MixerConfig>>,
    routing: Arc<Mutex<RoutingConfig>>,
    automation: Arc<AutomationRecorder>,
) -> Result<(), String> {
    // Restore mixer
    {
        let mut mixer_config = mixer.lock().await;
        *mixer_config = session.mixer;
    }

    // Restore routing
    {
        let mut routing_config = routing.lock().await;
        *routing_config = session.routing;
    }

    // Restore automation lanes
    for lane in session.automation_lanes {
        automation.add_lane(lane).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daw::automation::AutomationRecorder;
    use crate::daw::mixer::MixerConfig;
    use crate::daw::mixer::routing::RoutingConfig;

    #[tokio::test]
    async fn test_serialize_deserialize() {
        let mixer = Arc::new(Mutex::new(MixerConfig::default()));
        let routing = Arc::new(Mutex::new(RoutingConfig::default()));
        let automation = Arc::new(AutomationRecorder::new());

        // Serialize
        let session = serialize_session(
            "Test".to_string(),
            Arc::clone(&mixer),
            Arc::clone(&routing),
            Arc::clone(&automation),
            0,
        )
        .await
        .unwrap();

        // Create new state
        let new_mixer = Arc::new(Mutex::new(MixerConfig::default()));
        let new_routing = Arc::new(Mutex::new(RoutingConfig::default()));
        let new_automation = Arc::new(AutomationRecorder::new());

        // Deserialize
        deserialize_session(session, new_mixer, new_routing, new_automation)
            .await
            .unwrap();
    }
}
```

---

## Step 3: Database Schema (20 min)

Add to `database/migrations/022_automation_presets_projects.sql`:

```sql
-- Projects table
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    file_path TEXT,
    bpm REAL NOT NULL DEFAULT 120.0,
    time_signature_num INTEGER NOT NULL DEFAULT 4,
    time_signature_denom INTEGER NOT NULL DEFAULT 4,
    sample_rate INTEGER NOT NULL DEFAULT 48000,
    session_data TEXT NOT NULL, -- JSON session state
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_opened_at TIMESTAMP
);

CREATE INDEX idx_projects_name ON projects(name);
CREATE INDEX idx_projects_last_opened ON projects(last_opened_at DESC NULLS LAST);

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_project_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER project_update_timestamp
    BEFORE UPDATE ON projects
    FOR EACH ROW
    EXECUTE FUNCTION update_project_timestamp();

-- Function to get recent projects
CREATE OR REPLACE FUNCTION get_recent_projects(p_limit INTEGER DEFAULT 10)
RETURNS TABLE(
    id TEXT,
    name TEXT,
    file_path TEXT,
    last_opened_at TIMESTAMP
) AS $$
BEGIN
    RETURN QUERY
    SELECT p.id, p.name, p.file_path, p.last_opened_at
    FROM projects p
    WHERE p.last_opened_at IS NOT NULL
    ORDER BY p.last_opened_at DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;
```

---

## Step 4: Project Repository (40 min)

Create `app/src-tauri/src/daw/project/repository.rs`:

```rust
use super::models::{Project, RecentProject};
use sqlx::PgPool;
use std::path::PathBuf;

pub struct ProjectRepository {
    pool: PgPool,
}

impl ProjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Save project
    pub async fn save(&self, project: &Project) -> Result<(), sqlx::Error> {
        let file_path_str = project.file_path.as_ref().map(|p| p.to_string_lossy().to_string());

        sqlx::query!(
            r#"
            INSERT INTO projects (id, name, file_path, bpm, time_signature_num, time_signature_denom,
                                  sample_rate, session_data, created_at, updated_at, last_opened_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id)
            DO UPDATE SET
                name = EXCLUDED.name,
                file_path = EXCLUDED.file_path,
                bpm = EXCLUDED.bpm,
                time_signature_num = EXCLUDED.time_signature_num,
                time_signature_denom = EXCLUDED.time_signature_denom,
                sample_rate = EXCLUDED.sample_rate,
                session_data = EXCLUDED.session_data,
                last_opened_at = EXCLUDED.last_opened_at,
                updated_at = CURRENT_TIMESTAMP
            "#,
            project.id,
            project.name,
            file_path_str,
            project.bpm,
            project.time_signature.0 as i32,
            project.time_signature.1 as i32,
            project.sample_rate as i32,
            project.session_data,
            project.created_at,
            project.updated_at,
            project.last_opened_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load project by ID
    pub async fn load(&self, project_id: &str) -> Result<Option<Project>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, name, file_path, bpm, time_signature_num, time_signature_denom,
                   sample_rate, session_data, created_at, updated_at, last_opened_at
            FROM projects
            WHERE id = $1
            "#,
            project_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let file_path = row.file_path.map(PathBuf::from);

            Ok(Some(Project {
                id: row.id,
                name: row.name,
                file_path,
                bpm: row.bpm,
                time_signature: (row.time_signature_num as u8, row.time_signature_denom as u8),
                sample_rate: row.sample_rate as u32,
                session_data: row.session_data,
                created_at: row.created_at.and_utc(),
                updated_at: row.updated_at.and_utc(),
                last_opened_at: row.last_opened_at.map(|dt| dt.and_utc()),
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete project
    pub async fn delete(&self, project_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM projects WHERE id = $1", project_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// List all projects
    pub async fn list_all(&self) -> Result<Vec<Project>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, file_path, bpm, time_signature_num, time_signature_denom,
                   sample_rate, session_data, created_at, updated_at, last_opened_at
            FROM projects
            ORDER BY updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let projects = rows
            .into_iter()
            .map(|row| {
                let file_path = row.file_path.map(PathBuf::from);

                Project {
                    id: row.id,
                    name: row.name,
                    file_path,
                    bpm: row.bpm,
                    time_signature: (row.time_signature_num as u8, row.time_signature_denom as u8),
                    sample_rate: row.sample_rate as u32,
                    session_data: row.session_data,
                    created_at: row.created_at.and_utc(),
                    updated_at: row.updated_at.and_utc(),
                    last_opened_at: row.last_opened_at.map(|dt| dt.and_utc()),
                }
            })
            .collect();

        Ok(projects)
    }

    /// Get recent projects
    pub async fn get_recent(&self, limit: i32) -> Result<Vec<RecentProject>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, file_path, last_opened_at
            FROM projects
            WHERE last_opened_at IS NOT NULL
            ORDER BY last_opened_at DESC
            LIMIT $1
            "#,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        let recent = rows
            .into_iter()
            .filter_map(|row| {
                row.last_opened_at.map(|last_opened| RecentProject {
                    id: row.id,
                    name: row.name,
                    file_path: row.file_path.map(PathBuf::from),
                    last_opened_at: last_opened.and_utc(),
                })
            })
            .collect();

        Ok(recent)
    }

    /// Update last_opened_at timestamp
    pub async fn update_last_opened(&self, project_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE projects
            SET last_opened_at = CURRENT_TIMESTAMP
            WHERE id = $1
            "#,
            project_id,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
```

Add to repositories mod:

```rust
// In app/src-tauri/src/db/repositories/mod.rs
pub mod project_repository;
pub use project_repository::ProjectRepository;
```

---

## Step 5: Module Organization (10 min)

Create `app/src-tauri/src/daw/project/mod.rs`:

```rust
pub mod models;
pub mod repository;
pub mod serializer;

pub use models::{Project, SessionState, RecentProject};
pub use repository::ProjectRepository;
pub use serializer::{serialize_session, deserialize_session};
```

Update `app/src-tauri/src/daw/mod.rs`:

```rust
pub mod automation;
pub mod mixer;
pub mod presets;
pub mod project;  // NEW
pub mod sequencer;
```

---

## Step 6: Tauri Commands (20 min)

Create `app/src-tauri/src/commands/daw/project_commands.rs`:

```rust
use crate::daw::project::models::{Project, SessionState, RecentProject};
use crate::daw::project::{serialize_session, deserialize_session};
use crate::db::repositories::ProjectRepository;
use crate::AppState;
use tauri::State;

/// Create new project
#[tauri::command]
pub async fn create_project(
    name: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    log::info!("Creating new project: {}", name);

    let project_id = format!("proj-{}", uuid::Uuid::new_v4());
    let project = Project::new(project_id, name);

    let repo = ProjectRepository::new(state.db_pool.clone());
    repo.save(&project)
        .await
        .map_err(|e| format!("Failed to save project: {}", e))?;

    Ok(project)
}

/// Save current project state
#[tauri::command]
pub async fn save_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Saving project: {}", project_id);

    let sequencer = state.sequencer.lock().await;
    let repo = ProjectRepository::new(state.db_pool.clone());

    // Load existing project
    let mut project = repo
        .load(&project_id)
        .await
        .map_err(|e| format!("Failed to load project: {}", e))?
        .ok_or_else(|| format!("Project {} not found", project_id))?;

    // Serialize current session
    let session = serialize_session(
        project.name.clone(),
        sequencer.get_mixer_arc(),
        sequencer.get_routing_arc(),
        sequencer.get_automation_recorder(),
        sequencer.get_playhead_position().await,
    )
    .await?;

    project.session_data = session.to_json()?;

    // Save to database
    repo.save(&project)
        .await
        .map_err(|e| format!("Failed to save project: {}", e))?;

    Ok(())
}

/// Load project
#[tauri::command]
pub async fn load_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    log::info!("Loading project: {}", project_id);

    let sequencer = state.sequencer.lock().await;
    let repo = ProjectRepository::new(state.db_pool.clone());

    // Load project from database
    let project = repo
        .load(&project_id)
        .await
        .map_err(|e| format!("Failed to load project: {}", e))?
        .ok_or_else(|| format!("Project {} not found", project_id))?;

    // Deserialize session
    let session = SessionState::from_json(&project.session_data)?;

    deserialize_session(
        session,
        sequencer.get_mixer_arc(),
        sequencer.get_routing_arc(),
        sequencer.get_automation_recorder(),
    )
    .await?;

    // Update last_opened_at
    repo.update_last_opened(&project_id)
        .await
        .map_err(|e| format!("Failed to update last_opened: {}", e))?;

    Ok(project)
}

/// Delete project
#[tauri::command]
pub async fn delete_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Deleting project: {}", project_id);

    let repo = ProjectRepository::new(state.db_pool.clone());
    repo.delete(&project_id)
        .await
        .map_err(|e| format!("Failed to delete project: {}", e))
}

/// List all projects
#[tauri::command]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    log::info!("Listing all projects");

    let repo = ProjectRepository::new(state.db_pool.clone());
    repo.list_all()
        .await
        .map_err(|e| format!("Failed to list projects: {}", e))
}

/// Get recent projects
#[tauri::command]
pub async fn get_recent_projects(
    limit: i32,
    state: State<'_, AppState>,
) -> Result<Vec<RecentProject>, String> {
    log::info!("Getting {} recent projects", limit);

    let repo = ProjectRepository::new(state.db_pool.clone());
    repo.get_recent(limit)
        .await
        .map_err(|e| format!("Failed to get recent projects: {}", e))
}
```

Register in `main.rs`:

```rust
use midi_app::commands::daw::{
    create_project,
    save_project,
    load_project,
    delete_project,
    list_projects,
    get_recent_projects,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing
            create_project,
            save_project,
            load_project,
            delete_project,
            list_projects,
            get_recent_projects,
        ])
        // ...
}
```

---

## Verification (10 min)

```bash
cd app/src-tauri
cargo check
cargo test --lib project
```

Test in browser console:

```javascript
// Create project
const project = await window.__TAURI__.invoke('create_project', {
  name: 'My First Project'
});
console.log('Created:', project);

// Save project (captures current session)
await window.__TAURI__.invoke('save_project', { projectId: project.id });

// Load project (restores session)
const loaded = await window.__TAURI__.invoke('load_project', { projectId: project.id });
console.log('Loaded:', loaded);

// List all projects
const projects = await window.__TAURI__.invoke('list_projects');
console.log('All projects:', projects);

// Get recent projects
const recent = await window.__TAURI__.invoke('get_recent_projects', { limit: 5 });
console.log('Recent:', recent);
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Serialization fails | Check all models implement Serialize, verify no circular refs |
| Load doesn't restore state | Ensure deserialize_session applies all state correctly |
| Recent projects empty | Check last_opened_at is set, verify query sorts correctly |
| Session data too large | Consider compressing JSON, split into separate tables |

---

## What's Next?

✅ **You've completed:**
- Project models with session state
- Session serializer/deserializer
- ProjectRepository with save/load
- Database schema with recent projects
- 6 Tauri commands for project management

**Next:** [Part 4B: Project Management UI](./DAY4_PART_B_PROJECT_UI.md)
- Project manager component
- New/open/save dialogs
- Recent projects list
- File menu integration
