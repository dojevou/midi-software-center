//! Project and track loading commands
//!
//! Commands for loading multiple tracks into the sequencer from the database,
//! and CRUD operations for DAW projects stored in the daw_projects table.
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use crate::commands::daw::DawAppState;
use crate::core::midi::loader::load_midi_file;
use crate::daw_models::sequencer::Track;
use crate::sequencer::{ScheduledEvent, SequencerEngine};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use tauri::State;
use tracing::{error, info, warn};

/// Track with loaded events ready for scheduling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackWithEvents {
    pub track: Track,
    pub events: Vec<ScheduledEvent>,
}

/// Load multiple MIDI files as sequencer tracks
///
/// This command loads multiple files from the database and prepares them
/// as sequencer tracks with their MIDI events ready for playback.
///
/// # Arguments
/// * `file_ids` - List of database file IDs to load
/// * `state` - Application state with database connection
/// * `engine` - Sequencer engine
#[tauri::command]
pub async fn load_multiple_tracks(
    file_ids: Vec<i32>,
    state: State<'_, DawAppState>,
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<Vec<Track>, String> {
    info!("Loading {} files as sequencer tracks", file_ids.len());

    let mut loaded_tracks = Vec::new();
    let mut failed_count = 0;

    for (idx, file_id) in file_ids.iter().enumerate() {
        // Query database for file information
        let file_result = match sqlx::query!(
            r#"
            SELECT id, filepath, filename
            FROM files
            WHERE id = $1
            "#,
            *file_id as i64
        )
        .fetch_one(state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?)
        .await
        {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to query file {} from database: {}", file_id, e);
                failed_count += 1;
                continue;
            },
        };

        // Load MIDI file and parse events
        let loaded_midi = match load_midi_file(&file_result.filepath) {
            Ok(midi) => midi,
            Err(e) => {
                error!(
                    "Failed to load MIDI file {} ({}): {}",
                    file_result.filename, file_result.filepath, e
                );
                failed_count += 1;
                continue;
            },
        };

        info!(
            "Loaded {} events from {} ({}/{})",
            loaded_midi.events.len(),
            file_result.filename,
            idx + 1,
            file_ids.len()
        );

        // Add track with loaded events
        let track_manager = engine.track_manager();
        let channel = (idx % 16) as u8; // Distribute across MIDI channels

        match track_manager
            .add_track(file_result.id as i32, channel, loaded_midi.events)
            .await
        {
            Ok(track) => loaded_tracks.push(track),
            Err(e) => {
                error!("Failed to add track for file {}: {}", file_id, e);
                failed_count += 1;
            },
        }
    }

    if failed_count > 0 {
        warn!(
            "Failed to load {} out of {} tracks",
            failed_count,
            file_ids.len()
        );
    }

    // Reload tracks in engine to update scheduler
    engine.load_tracks().await;

    info!(
        "Successfully loaded {} tracks into sequencer",
        loaded_tracks.len()
    );

    Ok(loaded_tracks)
}

/// Clear all tracks from the sequencer
#[tauri::command]
pub async fn clear_all_tracks(engine: State<'_, Arc<SequencerEngine>>) -> Result<(), String> {
    info!("Clearing all tracks from sequencer");

    let track_manager = engine.track_manager();
    track_manager.clear().await;

    let scheduler = engine.scheduler();
    scheduler.clear().await;

    Ok(())
}

/// Get detailed information about loaded tracks
#[tauri::command]
pub async fn get_track_details(
    engine: State<'_, Arc<SequencerEngine>>,
) -> Result<Vec<TrackDetails>, String> {
    let track_manager = engine.track_manager();
    let tracks = track_manager.get_tracks().await;

    let details: Vec<TrackDetails> = tracks
        .into_iter()
        .map(|track| {
            let event_count = track.events.len();
            TrackDetails {
                id: track.id,
                name: track.name,
                file_id: track.file_id,
                channel: track.channel,
                muted: track.muted,
                solo: track.solo,
                volume: track.volume,
                pan: track.pan,
                event_count,
            }
        })
        .collect();

    Ok(details)
}

/// Track details for frontend display
#[derive(Debug, Serialize, Deserialize)]
pub struct TrackDetails {
    pub id: i32,
    pub name: String,
    pub file_id: i32,
    pub channel: u8,
    pub muted: bool,
    pub solo: bool,
    pub volume: u8,
    pub pan: u8,
    pub event_count: usize,
}

// ============================================================================
// DAW PROJECT DATABASE COMMANDS
// ============================================================================

/// DAW Project stored in daw_projects table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DawProject {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub bpm: f64,
    pub time_signature_numerator: i32,
    pub time_signature_denominator: i32,
    pub key_signature: String,
    pub sample_rate: i32,
    pub bit_depth: i32,
    pub file_path: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_opened_at: Option<DateTime<Utc>>,
}

/// Parameters for creating a new project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectParams {
    pub name: String,
    pub description: Option<String>,
    pub bpm: Option<f64>,
    pub time_signature_numerator: Option<i32>,
    pub time_signature_denominator: Option<i32>,
    pub key_signature: Option<String>,
    pub sample_rate: Option<i32>,
    pub bit_depth: Option<i32>,
}

/// Parameters for updating an existing project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub bpm: Option<f64>,
    pub time_signature_numerator: Option<i32>,
    pub time_signature_denominator: Option<i32>,
    pub key_signature: Option<String>,
    pub sample_rate: Option<i32>,
    pub bit_depth: Option<i32>,
    pub file_path: Option<String>,
    pub file_size_bytes: Option<i64>,
}

/// Project search/filter parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectFilters {
    pub query: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
}

/// Project list response with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectListResponse {
    pub projects: Vec<DawProject>,
    pub total_count: i64,
}

/// Create a new DAW project in the database
#[tauri::command]
pub async fn project_create(
    state: State<'_, DawAppState>,
    params: CreateProjectParams,
) -> Result<DawProject, String> {
    let pool = state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let row = sqlx::query(
        r#"
        INSERT INTO daw_projects (
            name, description, bpm,
            time_signature_numerator, time_signature_denominator,
            key_signature, sample_rate, bit_depth
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, name, description, bpm,
                  time_signature_numerator, time_signature_denominator,
                  key_signature, sample_rate, bit_depth,
                  file_path, file_size_bytes, version,
                  created_at, updated_at, last_opened_at
        "#,
    )
    .bind(&params.name)
    .bind(&params.description)
    .bind(params.bpm.unwrap_or(120.0))
    .bind(params.time_signature_numerator.unwrap_or(4))
    .bind(params.time_signature_denominator.unwrap_or(4))
    .bind(params.key_signature.as_deref().unwrap_or("C"))
    .bind(params.sample_rate.unwrap_or(44100))
    .bind(params.bit_depth.unwrap_or(16))
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to create project: {}", e))?;

    let project = row_to_project(&row)?;
    info!("Created project: {} (ID: {})", project.name, project.id);
    Ok(project)
}

/// Load a DAW project by ID
#[tauri::command]
pub async fn project_load(
    state: State<'_, DawAppState>,
    id: i64,
) -> Result<Option<DawProject>, String> {
    let pool = state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    // Update last_opened_at and fetch project
    let row = sqlx::query(
        r#"
        UPDATE daw_projects
        SET last_opened_at = NOW()
        WHERE id = $1
        RETURNING id, name, description, bpm,
                  time_signature_numerator, time_signature_denominator,
                  key_signature, sample_rate, bit_depth,
                  file_path, file_size_bytes, version,
                  created_at, updated_at, last_opened_at
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to load project: {}", e))?;

    match row {
        Some(r) => {
            let project = row_to_project(&r)?;
            info!("Loaded project: {} (ID: {})", project.name, project.id);
            Ok(Some(project))
        },
        None => Ok(None),
    }
}

/// Get a project by ID without updating last_opened_at
#[tauri::command]
pub async fn project_get(
    state: State<'_, DawAppState>,
    id: i64,
) -> Result<Option<DawProject>, String> {
    let pool = state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let row = sqlx::query(
        r#"
        SELECT id, name, description, bpm,
               time_signature_numerator, time_signature_denominator,
               key_signature, sample_rate, bit_depth,
               file_path, file_size_bytes, version,
               created_at, updated_at, last_opened_at
        FROM daw_projects
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to get project: {}", e))?;

    match row {
        Some(r) => Ok(Some(row_to_project(&r)?)),
        None => Ok(None),
    }
}

/// Update a DAW project
#[tauri::command]
pub async fn project_update(
    state: State<'_, DawAppState>,
    id: i64,
    params: UpdateProjectParams,
) -> Result<DawProject, String> {
    let pool = state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    // Build dynamic UPDATE query
    let mut set_clauses = vec!["updated_at = NOW()".to_string()];
    let mut param_count = 1;

    // We'll use dynamic SQL for this
    let mut query = String::from("UPDATE daw_projects SET ");

    if params.name.is_some() {
        param_count += 1;
        set_clauses.push(format!("name = ${}", param_count));
    }
    if params.description.is_some() {
        param_count += 1;
        set_clauses.push(format!("description = ${}", param_count));
    }
    if params.bpm.is_some() {
        param_count += 1;
        set_clauses.push(format!("bpm = ${}", param_count));
    }
    if params.time_signature_numerator.is_some() {
        param_count += 1;
        set_clauses.push(format!("time_signature_numerator = ${}", param_count));
    }
    if params.time_signature_denominator.is_some() {
        param_count += 1;
        set_clauses.push(format!("time_signature_denominator = ${}", param_count));
    }
    if params.key_signature.is_some() {
        param_count += 1;
        set_clauses.push(format!("key_signature = ${}", param_count));
    }
    if params.sample_rate.is_some() {
        param_count += 1;
        set_clauses.push(format!("sample_rate = ${}", param_count));
    }
    if params.bit_depth.is_some() {
        param_count += 1;
        set_clauses.push(format!("bit_depth = ${}", param_count));
    }
    if params.file_path.is_some() {
        param_count += 1;
        set_clauses.push(format!("file_path = ${}", param_count));
    }
    if params.file_size_bytes.is_some() {
        param_count += 1;
        set_clauses.push(format!("file_size_bytes = ${}", param_count));
    }

    // Increment version
    set_clauses.push("version = version + 1".to_string());

    query.push_str(&set_clauses.join(", "));
    query.push_str(" WHERE id = $1 RETURNING id, name, description, bpm, time_signature_numerator, time_signature_denominator, key_signature, sample_rate, bit_depth, file_path, file_size_bytes, version, created_at, updated_at, last_opened_at");

    // Build the query with bindings
    let mut q = sqlx::query(&query).bind(id);

    if let Some(ref name) = params.name {
        q = q.bind(name);
    }
    if let Some(ref description) = params.description {
        q = q.bind(description);
    }
    if let Some(bpm) = params.bpm {
        q = q.bind(bpm);
    }
    if let Some(num) = params.time_signature_numerator {
        q = q.bind(num);
    }
    if let Some(denom) = params.time_signature_denominator {
        q = q.bind(denom);
    }
    if let Some(ref key) = params.key_signature {
        q = q.bind(key);
    }
    if let Some(rate) = params.sample_rate {
        q = q.bind(rate);
    }
    if let Some(depth) = params.bit_depth {
        q = q.bind(depth);
    }
    if let Some(ref path) = params.file_path {
        q = q.bind(path);
    }
    if let Some(size) = params.file_size_bytes {
        q = q.bind(size);
    }

    let row = q
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Failed to update project: {}", e))?;

    let project = row_to_project(&row)?;
    info!("Updated project: {} (ID: {})", project.name, project.id);
    Ok(project)
}

/// Delete a DAW project
#[tauri::command]
pub async fn project_delete(state: State<'_, DawAppState>, id: i64) -> Result<bool, String> {
    let pool = state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let result = sqlx::query("DELETE FROM daw_projects WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to delete project: {}", e))?;

    let deleted = result.rows_affected() > 0;
    if deleted {
        info!("Deleted project ID: {}", id);
    } else {
        warn!("Project ID {} not found for deletion", id);
    }
    Ok(deleted)
}

/// List DAW projects with optional filtering and pagination
#[tauri::command]
pub async fn project_list(
    state: State<'_, DawAppState>,
    filters: ProjectFilters,
) -> Result<ProjectListResponse, String> {
    let pool = state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let limit = filters.limit.unwrap_or(50);
    let offset = filters.offset.unwrap_or(0);
    let sort_by = filters.sort_by.as_deref().unwrap_or("updated_at");
    let sort_dir = if filters.sort_desc.unwrap_or(true) {
        "DESC"
    } else {
        "ASC"
    };

    // Validate sort column to prevent SQL injection
    let valid_columns = ["id", "name", "bpm", "created_at", "updated_at", "last_opened_at"];
    let sort_column = if valid_columns.contains(&sort_by) {
        sort_by
    } else {
        "updated_at"
    };

    // Build WHERE clause for search
    let (where_clause, search_param) = if let Some(ref query) = filters.query {
        (
            "WHERE name ILIKE $1 OR description ILIKE $1",
            Some(format!("%{}%", query)),
        )
    } else {
        ("", None)
    };

    // Count total
    let count_query = format!(
        "SELECT COUNT(*) as count FROM daw_projects {}",
        where_clause
    );
    let count_row = if let Some(ref param) = search_param {
        sqlx::query(&count_query).bind(param).fetch_one(pool).await
    } else {
        sqlx::query(&count_query).fetch_one(pool).await
    }
    .map_err(|e| format!("Count query failed: {}", e))?;

    let total_count: i64 =
        count_row.try_get("count").map_err(|e| format!("Failed to get count: {}", e))?;

    // Fetch projects
    let select_query = format!(
        r#"
        SELECT id, name, description, bpm,
               time_signature_numerator, time_signature_denominator,
               key_signature, sample_rate, bit_depth,
               file_path, file_size_bytes, version,
               created_at, updated_at, last_opened_at
        FROM daw_projects
        {}
        ORDER BY {} {}
        LIMIT {} OFFSET {}
        "#,
        where_clause, sort_column, sort_dir, limit, offset
    );

    let rows = if let Some(ref param) = search_param {
        sqlx::query(&select_query).bind(param).fetch_all(pool).await
    } else {
        sqlx::query(&select_query).fetch_all(pool).await
    }
    .map_err(|e| format!("Select query failed: {}", e))?;

    let projects: Result<Vec<DawProject>, String> = rows.iter().map(row_to_project).collect();

    Ok(ProjectListResponse { projects: projects?, total_count })
}

/// Get recent projects (last opened)
#[tauri::command]
pub async fn project_get_recent(
    state: State<'_, DawAppState>,
    limit: Option<i64>,
) -> Result<Vec<DawProject>, String> {
    let pool = state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    let limit = limit.unwrap_or(10);

    let rows = sqlx::query(
        r#"
        SELECT id, name, description, bpm,
               time_signature_numerator, time_signature_denominator,
               key_signature, sample_rate, bit_depth,
               file_path, file_size_bytes, version,
               created_at, updated_at, last_opened_at
        FROM daw_projects
        WHERE last_opened_at IS NOT NULL
        ORDER BY last_opened_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get recent projects: {}", e))?;

    let projects: Result<Vec<DawProject>, String> = rows.iter().map(row_to_project).collect();
    projects
}

/// Helper function to convert a database row to DawProject
fn row_to_project(row: &sqlx::postgres::PgRow) -> Result<DawProject, String> {
    Ok(DawProject {
        id: row.try_get("id").map_err(|e| format!("Failed to get id: {}", e))?,
        name: row.try_get("name").map_err(|e| format!("Failed to get name: {}", e))?,
        description: row.try_get("description").ok(),
        bpm: row.try_get("bpm").map_err(|e| format!("Failed to get bpm: {}", e))?,
        time_signature_numerator: row
            .try_get("time_signature_numerator")
            .map_err(|e| format!("Failed to get time_signature_numerator: {}", e))?,
        time_signature_denominator: row
            .try_get("time_signature_denominator")
            .map_err(|e| format!("Failed to get time_signature_denominator: {}", e))?,
        key_signature: row
            .try_get("key_signature")
            .map_err(|e| format!("Failed to get key_signature: {}", e))?,
        sample_rate: row
            .try_get("sample_rate")
            .map_err(|e| format!("Failed to get sample_rate: {}", e))?,
        bit_depth: row
            .try_get("bit_depth")
            .map_err(|e| format!("Failed to get bit_depth: {}", e))?,
        file_path: row.try_get("file_path").ok(),
        file_size_bytes: row.try_get("file_size_bytes").ok(),
        version: row.try_get("version").map_err(|e| format!("Failed to get version: {}", e))?,
        created_at: row
            .try_get("created_at")
            .map_err(|e| format!("Failed to get created_at: {}", e))?,
        updated_at: row
            .try_get("updated_at")
            .map_err(|e| format!("Failed to get updated_at: {}", e))?,
        last_opened_at: row.try_get("last_opened_at").ok(),
    })
}

// ============================================================================
// PROJECT TRACKS AND CLIPS COMMANDS
// ============================================================================

/// Track data for saving/loading with a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTrack {
    pub id: Option<i64>,
    pub name: String,
    pub track_number: i32,
    pub channel: u8,
    pub program_number: u8,
    pub is_muted: bool,
    pub is_solo: bool,
    pub is_armed: bool,
    pub volume: f64,
    pub pan: f64,
    pub color: String,
    pub height: i32,
    pub collapsed: bool,
    pub source_file_id: Option<i64>,
    pub clips: Vec<ProjectClip>,
}

/// Clip data for saving/loading with a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectClip {
    pub id: Option<i64>,
    pub name: String,
    pub color: String,
    pub start_tick: i64,
    pub duration_ticks: i64,
    pub source_file_id: Option<i64>,
    pub is_muted: bool,
    pub gain_db: f64,
    pub source_start_tick: i64,
    pub source_end_tick: Option<i64>,
}

/// Full project data including tracks and clips
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullProjectData {
    pub project: DawProject,
    pub tracks: Vec<ProjectTrack>,
}

/// Save project tracks and clips to the database
#[tauri::command]
pub async fn project_save_tracks(
    state: State<'_, DawAppState>,
    project_id: i64,
    tracks: Vec<ProjectTrack>,
) -> Result<Vec<ProjectTrack>, String> {
    let pool = state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    info!("Saving {} tracks for project {}", tracks.len(), project_id);

    let mut saved_tracks = Vec::new();

    for track in tracks {
        // Insert or update track
        let track_row = if let Some(track_id) = track.id {
            // Update existing track
            sqlx::query(
                r#"
                UPDATE daw_tracks SET
                    name = $2,
                    track_number = $3,
                    channel = $4,
                    program_number = $5,
                    is_muted = $6,
                    is_solo = $7,
                    is_armed = $8,
                    volume = $9,
                    pan = $10,
                    color = $11,
                    height = $12,
                    collapsed = $13,
                    source_file_id = $14,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING id
                "#,
            )
            .bind(track_id)
            .bind(&track.name)
            .bind(track.track_number)
            .bind(track.channel as i16)
            .bind(track.program_number as i16)
            .bind(track.is_muted)
            .bind(track.is_solo)
            .bind(track.is_armed)
            .bind(track.volume)
            .bind(track.pan)
            .bind(&track.color)
            .bind(track.height)
            .bind(track.collapsed)
            .bind(track.source_file_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to update track: {}", e))?
        } else {
            // Insert new track
            sqlx::query(
                r#"
                INSERT INTO daw_tracks (
                    project_id, name, track_number, channel, program_number,
                    is_muted, is_solo, is_armed, volume, pan, color, height,
                    collapsed, source_file_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                RETURNING id
                "#,
            )
            .bind(project_id)
            .bind(&track.name)
            .bind(track.track_number)
            .bind(track.channel as i16)
            .bind(track.program_number as i16)
            .bind(track.is_muted)
            .bind(track.is_solo)
            .bind(track.is_armed)
            .bind(track.volume)
            .bind(track.pan)
            .bind(&track.color)
            .bind(track.height)
            .bind(track.collapsed)
            .bind(track.source_file_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to insert track: {}", e))?
        };

        let track_id: i64 =
            track_row.try_get("id").map_err(|e| format!("Failed to get track id: {}", e))?;

        // Delete existing clips for this track (we'll re-insert all clips)
        sqlx::query("DELETE FROM daw_clips WHERE track_id = $1")
            .bind(track_id)
            .execute(pool)
            .await
            .map_err(|e| format!("Failed to delete existing clips: {}", e))?;

        // Insert clips
        let mut saved_clips = Vec::new();
        for clip in &track.clips {
            let clip_row = sqlx::query(
                r#"
                INSERT INTO daw_clips (
                    track_id, name, color, start_tick, duration_ticks,
                    source_file_id, is_muted, gain_db, source_start_tick,
                    source_end_tick
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING id
                "#,
            )
            .bind(track_id)
            .bind(&clip.name)
            .bind(&clip.color)
            .bind(clip.start_tick)
            .bind(clip.duration_ticks)
            .bind(clip.source_file_id)
            .bind(clip.is_muted)
            .bind(clip.gain_db)
            .bind(clip.source_start_tick)
            .bind(clip.source_end_tick)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("Failed to insert clip: {}", e))?;

            let clip_id: i64 =
                clip_row.try_get("id").map_err(|e| format!("Failed to get clip id: {}", e))?;

            saved_clips.push(ProjectClip { id: Some(clip_id), ..clip.clone() });
        }

        saved_tracks.push(ProjectTrack { id: Some(track_id), clips: saved_clips, ..track });
    }

    // Update project's last saved timestamp
    sqlx::query("UPDATE daw_projects SET last_saved_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update project timestamp: {}", e))?;

    info!(
        "Successfully saved {} tracks for project {}",
        saved_tracks.len(),
        project_id
    );

    Ok(saved_tracks)
}

/// Load project tracks and clips from the database
#[tauri::command]
pub async fn project_load_tracks(
    state: State<'_, DawAppState>,
    project_id: i64,
) -> Result<Vec<ProjectTrack>, String> {
    let pool = state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    info!("Loading tracks for project {}", project_id);

    // Load tracks
    let track_rows = sqlx::query(
        r#"
        SELECT id, name, track_number, channel, program_number,
               is_muted, is_solo, is_armed, volume, pan, color,
               height, collapsed, source_file_id
        FROM daw_tracks
        WHERE project_id = $1
        ORDER BY track_number ASC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to load tracks: {}", e))?;

    let mut tracks = Vec::new();

    for track_row in track_rows {
        let track_id: i64 =
            track_row.try_get("id").map_err(|e| format!("Failed to get track id: {}", e))?;

        // Load clips for this track
        let clip_rows = sqlx::query(
            r#"
            SELECT id, name, color, start_tick, duration_ticks,
                   source_file_id, is_muted, gain_db, source_start_tick,
                   source_end_tick
            FROM daw_clips
            WHERE track_id = $1
            ORDER BY start_tick ASC
            "#,
        )
        .bind(track_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Failed to load clips: {}", e))?;

        let clips: Vec<ProjectClip> = clip_rows
            .iter()
            .map(|row| {
                Ok(ProjectClip {
                    id: Some(row.try_get("id").map_err(|e| e.to_string())?),
                    name: row.try_get("name").map_err(|e| e.to_string())?,
                    color: row.try_get("color").unwrap_or_else(|_| "#3B82F6".to_string()),
                    start_tick: row.try_get("start_tick").map_err(|e| e.to_string())?,
                    duration_ticks: row.try_get("duration_ticks").map_err(|e| e.to_string())?,
                    source_file_id: row.try_get("source_file_id").ok(),
                    is_muted: row.try_get("is_muted").unwrap_or(false),
                    gain_db: row.try_get("gain_db").unwrap_or(0.0),
                    source_start_tick: row.try_get("source_start_tick").unwrap_or(0),
                    source_end_tick: row.try_get("source_end_tick").ok(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let channel: i16 = track_row.try_get("channel").unwrap_or(0);
        let program: i16 = track_row.try_get("program_number").unwrap_or(0);
        let volume: f64 = track_row.try_get("volume").unwrap_or(1.0);
        let pan: f64 = track_row.try_get("pan").unwrap_or(0.0);

        tracks.push(ProjectTrack {
            id: Some(track_id),
            name: track_row.try_get("name").map_err(|e| format!("Failed to get name: {}", e))?,
            track_number: track_row
                .try_get("track_number")
                .map_err(|e| format!("Failed to get track_number: {}", e))?,
            channel: channel as u8,
            program_number: program as u8,
            is_muted: track_row.try_get("is_muted").unwrap_or(false),
            is_solo: track_row.try_get("is_solo").unwrap_or(false),
            is_armed: track_row.try_get("is_armed").unwrap_or(false),
            volume,
            pan,
            color: track_row.try_get("color").unwrap_or_else(|_| "#3B82F6".to_string()),
            height: track_row.try_get("height").unwrap_or(100),
            collapsed: track_row.try_get("collapsed").unwrap_or(false),
            source_file_id: track_row.try_get("source_file_id").ok(),
            clips,
        });
    }

    info!("Loaded {} tracks for project {}", tracks.len(), project_id);

    Ok(tracks)
}

/// Load full project data including tracks and clips
#[tauri::command]
pub async fn project_load_full(
    state: State<'_, DawAppState>,
    id: i64,
) -> Result<Option<FullProjectData>, String> {
    // First load the project
    let project = project_load(state.clone(), id).await?;

    match project {
        Some(proj) => {
            // Then load tracks
            let tracks = project_load_tracks(state, id).await?;
            Ok(Some(FullProjectData { project: proj, tracks }))
        },
        None => Ok(None),
    }
}

/// Delete tracks from a project
#[tauri::command]
pub async fn project_delete_tracks(
    state: State<'_, DawAppState>,
    project_id: i64,
    track_ids: Vec<i64>,
) -> Result<i64, String> {
    let pool = state.db_pool.as_ref().ok_or_else(|| "Database not initialized".to_string())?;

    info!(
        "Deleting {} tracks from project {}",
        track_ids.len(),
        project_id
    );

    // Delete tracks (clips will cascade delete)
    let result = sqlx::query(
        r#"
        DELETE FROM daw_tracks
        WHERE id = ANY($1) AND project_id = $2
        "#,
    )
    .bind(&track_ids)
    .bind(project_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to delete tracks: {}", e))?;

    let deleted = result.rows_affected() as i64;
    info!("Deleted {} tracks from project {}", deleted, project_id);
    Ok(deleted)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project_params_defaults() {
        let params = CreateProjectParams {
            name: "Test Project".to_string(),
            description: None,
            bpm: None,
            time_signature_numerator: None,
            time_signature_denominator: None,
            key_signature: None,
            sample_rate: None,
            bit_depth: None,
        };

        assert_eq!(params.name, "Test Project");
        assert!(params.description.is_none());
        assert!(params.bpm.is_none());
    }

    #[test]
    fn test_update_project_params_partial() {
        let params = UpdateProjectParams {
            name: Some("New Name".to_string()),
            description: None,
            bpm: Some(140.0),
            time_signature_numerator: None,
            time_signature_denominator: None,
            key_signature: Some("G".to_string()),
            sample_rate: None,
            bit_depth: None,
            file_path: None,
            file_size_bytes: None,
        };

        assert_eq!(params.name, Some("New Name".to_string()));
        assert_eq!(params.bpm, Some(140.0));
        assert_eq!(params.key_signature, Some("G".to_string()));
        assert!(params.description.is_none());
    }

    #[test]
    fn test_project_filters_defaults() {
        let filters = ProjectFilters::default();

        assert!(filters.query.is_none());
        assert!(filters.limit.is_none());
        assert!(filters.offset.is_none());
        assert!(filters.sort_by.is_none());
        assert!(filters.sort_desc.is_none());
    }

    #[test]
    fn test_project_list_response_serialization() {
        let response = ProjectListResponse { projects: vec![], total_count: 0 };

        let json = serde_json::to_string(&response).expect("Should serialize");
        assert!(json.contains("\"projects\":[]"));
        assert!(json.contains("\"total_count\":0"));
    }

    #[test]
    fn test_daw_project_serialization() {
        let project = DawProject {
            id: 1,
            name: "Test".to_string(),
            description: Some("Description".to_string()),
            bpm: 120.0,
            time_signature_numerator: 4,
            time_signature_denominator: 4,
            key_signature: "C".to_string(),
            sample_rate: 44100,
            bit_depth: 16,
            file_path: None,
            file_size_bytes: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_opened_at: None,
        };

        let json = serde_json::to_string(&project).expect("Should serialize");
        assert!(json.contains("\"name\":\"Test\""));
        assert!(json.contains("\"bpm\":120.0"));
        assert!(json.contains("\"key_signature\":\"C\""));
    }
}
