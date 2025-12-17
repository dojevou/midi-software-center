//! Project management commands (Version 2)
//!
//! Commands for the new JSONB-based projects table with complete state serialization.
//! This is separate from the existing project.rs commands to avoid conflicts.

use crate::commands::daw::DawAppState;
use crate::daw_models::project::Project;
use crate::db::repositories::{
    CreateProjectParams, ProjectFilters, ProjectListResponse, ProjectRecord, ProjectRepository,
    UpdateProjectParams,
};
use std::path::PathBuf;
use tauri::State;
use tracing::{error, info};

// ============================================================================
// PROJECT CRUD COMMANDS
// ============================================================================

/// Create a new project with default settings
#[tauri::command]
pub async fn project_v2_create(
    state: State<'_, DawAppState>,
    name: String,
    description: Option<String>,
    bpm: Option<f64>,
) -> Result<i64, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    // Create a new Project struct
    let mut project = Project::new(name);

    if let Some(desc) = description {
        project.metadata.description = Some(desc);
    }

    if let Some(tempo) = bpm {
        project.settings.bpm = tempo;
    }

    // Save to database
    repo.create_from_project(&project)
        .await
        .map_err(|e| format!("Failed to create project: {}", e))
}

/// Load a project by ID and return the full Project struct
#[tauri::command]
pub async fn project_v2_load(
    state: State<'_, DawAppState>,
    id: i64,
) -> Result<Option<Project>, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    repo.load(id)
        .await
        .map_err(|e| format!("Failed to load project: {}", e))
}

/// Get project metadata without loading full project data
#[tauri::command]
pub async fn project_v2_get_metadata(
    state: State<'_, DawAppState>,
    id: i64,
) -> Result<Option<ProjectRecord>, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    repo.get(id)
        .await
        .map_err(|e| format!("Failed to get project: {}", e))
}

/// Update a project with new data
#[tauri::command]
pub async fn project_v2_update(
    state: State<'_, DawAppState>,
    id: i64,
    project: Project,
) -> Result<(), String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    repo.update_from_project(id, &project)
        .await
        .map_err(|e| format!("Failed to update project: {}", e))
}

/// Delete a project (soft delete)
#[tauri::command]
pub async fn project_v2_delete(state: State<'_, DawAppState>, id: i64) -> Result<bool, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    repo.delete(id)
        .await
        .map_err(|e| format!("Failed to delete project: {}", e))
}

/// List projects with optional filtering and pagination
#[tauri::command]
pub async fn project_v2_list(
    state: State<'_, DawAppState>,
    filters: ProjectFilters,
) -> Result<ProjectListResponse, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    repo.list(filters)
        .await
        .map_err(|e| format!("Failed to list projects: {}", e))
}

/// Get recent projects
#[tauri::command]
pub async fn project_v2_get_recent(
    state: State<'_, DawAppState>,
    limit: Option<i64>,
) -> Result<Vec<ProjectRecord>, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    repo.get_recent(limit)
        .await
        .map_err(|e| format!("Failed to get recent projects: {}", e))
}

/// Search projects by name or description
#[tauri::command]
pub async fn project_v2_search(
    state: State<'_, DawAppState>,
    query: String,
) -> Result<Vec<ProjectRecord>, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    repo.search(&query)
        .await
        .map_err(|e| format!("Failed to search projects: {}", e))
}

// ============================================================================
// FILE IMPORT/EXPORT COMMANDS
// ============================================================================

/// Export a project to a .mscproj file
#[tauri::command]
pub async fn project_v2_export(
    state: State<'_, DawAppState>,
    id: i64,
    path: String,
) -> Result<(), String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    // Load project from database
    let project = repo
        .load(id)
        .await
        .map_err(|e| format!("Failed to load project: {}", e))?
        .ok_or_else(|| format!("Project {} not found", id))?;

    // Ensure path has .mscproj extension
    let mut file_path = PathBuf::from(path);
    if file_path.extension().is_none() {
        file_path.set_extension("mscproj");
    }

    // Save to file
    project
        .save_to_file(&file_path)
        .map_err(|e| format!("Failed to save project file: {}", e))?;

    info!("Exported project {} to {:?}", id, file_path);
    Ok(())
}

/// Import a project from a .mscproj file
#[tauri::command]
pub async fn project_v2_import(
    state: State<'_, DawAppState>,
    path: String,
) -> Result<i64, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    // Load from file
    let project = Project::load_from_file(&path)
        .map_err(|e| format!("Failed to load project file: {}", e))?;

    // Validate
    project
        .validate()
        .map_err(|e| format!("Invalid project file: {}", e))?;

    // Save to database
    let id = repo
        .create_from_project(&project)
        .await
        .map_err(|e| format!("Failed to import project: {}", e))?;

    info!("Imported project from {} with ID {}", path, id);
    Ok(id)
}

/// Save current project state to file (quick export)
#[tauri::command]
pub async fn project_v2_save_as(
    _state: State<'_, DawAppState>,
    project: Project,
    path: String,
) -> Result<(), String> {
    // Validate first
    project
        .validate()
        .map_err(|e| format!("Invalid project: {}", e))?;

    // Ensure path has .mscproj extension
    let mut file_path = PathBuf::from(path);
    if file_path.extension().is_none() {
        file_path.set_extension("mscproj");
    }

    // Save to file
    project
        .save_to_file(&file_path)
        .map_err(|e| format!("Failed to save project: {}", e))?;

    info!("Saved project to {:?}", file_path);
    Ok(())
}

/// Load project from file without importing to database
#[tauri::command]
pub async fn project_v2_load_from_file(path: String) -> Result<Project, String> {
    let project = Project::load_from_file(&path)
        .map_err(|e| format!("Failed to load project: {}", e))?;

    // Validate
    project
        .validate()
        .map_err(|e| format!("Invalid project file: {}", e))?;

    Ok(project)
}

// ============================================================================
// TAG COMMANDS
// ============================================================================

/// Add a tag to a project
#[tauri::command]
pub async fn project_v2_add_tag(
    state: State<'_, DawAppState>,
    project_id: i64,
    tag: String,
) -> Result<(), String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    repo.add_tag(project_id, &tag)
        .await
        .map_err(|e| format!("Failed to add tag: {}", e))
}

/// Remove a tag from a project
#[tauri::command]
pub async fn project_v2_remove_tag(
    state: State<'_, DawAppState>,
    project_id: i64,
    tag: String,
) -> Result<(), String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    repo.remove_tag(project_id, &tag)
        .await
        .map_err(|e| format!("Failed to remove tag: {}", e))
}

/// Get all tags for a project
#[tauri::command]
pub async fn project_v2_get_tags(
    state: State<'_, DawAppState>,
    project_id: i64,
) -> Result<Vec<String>, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    repo.get_tags(project_id)
        .await
        .map_err(|e| format!("Failed to get tags: {}", e))
}

/// Get projects by tag
#[tauri::command]
pub async fn project_v2_get_by_tag(
    state: State<'_, DawAppState>,
    tag: String,
) -> Result<Vec<ProjectRecord>, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let repo = ProjectRepository::new(pool.clone());

    repo.get_by_tag(&tag)
        .await
        .map_err(|e| format!("Failed to get projects by tag: {}", e))
}

// ============================================================================
// UTILITY COMMANDS
// ============================================================================

/// Validate a project's data structure
#[tauri::command]
pub async fn project_v2_validate(project: Project) -> Result<(), String> {
    project.validate()
}

/// Get project duration in ticks and seconds
#[tauri::command]
pub async fn project_v2_get_duration(project: Project) -> Result<ProjectDuration, String> {
    Ok(ProjectDuration {
        ticks: project.get_duration_ticks(),
        seconds: project.get_duration_seconds(),
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectDuration {
    pub ticks: i64,
    pub seconds: f64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_duration_serialization() {
        let duration = ProjectDuration {
            ticks: 1920,
            seconds: 4.0,
        };

        let json = serde_json::to_string(&duration).expect("Should serialize");
        assert!(json.contains("\"ticks\":1920"));
        assert!(json.contains("\"seconds\":4"));
    }
}
