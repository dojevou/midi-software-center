//! Project repository for DAW project database operations.
//!
//! Provides CRUD operations for the projects table with JSONB storage
//! for complete project state serialization.

use crate::daw_models::project::Project;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{PgPool, Row};
use tracing::info;

/// Database model for project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub bpm: f64,
    pub time_signature_numerator: i32,
    pub time_signature_denominator: i32,
    pub project_data: serde_json::Value,
    pub schema_version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Parameters for creating a new project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectParams {
    pub name: String,
    pub description: Option<String>,
    pub bpm: Option<f64>,
    pub time_signature_numerator: Option<i32>,
    pub time_signature_denominator: Option<i32>,
    pub project_data: Option<serde_json::Value>,
}

/// Parameters for updating a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub bpm: Option<f64>,
    pub time_signature_numerator: Option<i32>,
    pub time_signature_denominator: Option<i32>,
    pub project_data: Option<serde_json::Value>,
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
    pub projects: Vec<ProjectRecord>,
    pub total_count: i64,
}

/// Repository for project operations
#[derive(Clone)]
pub struct ProjectRepository {
    pool: PgPool,
}

impl ProjectRepository {
    /// Create new repository with connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new project from a Project struct
    pub async fn create_from_project(&self, project: &Project) -> Result<i64, String> {
        // Serialize project to JSON
        let project_json = serde_json::to_value(project)
            .map_err(|e| format!("Failed to serialize project: {}", e))?;

        // Extract metadata for indexed columns
        let params = CreateProjectParams {
            name: project.metadata.name.clone(),
            description: project.metadata.description.clone(),
            bpm: Some(project.settings.bpm),
            time_signature_numerator: Some(project.settings.time_signature_numerator as i32),
            time_signature_denominator: Some(project.settings.time_signature_denominator as i32),
            project_data: Some(project_json),
        };

        self.create(params).await
    }

    /// Create a new project from parameters
    pub async fn create(&self, params: CreateProjectParams) -> Result<i64, String> {
        let project_data = params.project_data.unwrap_or_else(|| {
            // Create a minimal project if none provided
            let project = Project::new(params.name.clone());
            serde_json::to_value(&project).expect("Failed to serialize default project")
        });

        let row = sqlx::query(
            r#"
            INSERT INTO projects (
                name, description, bpm,
                time_signature_numerator, time_signature_denominator,
                project_data, schema_version
            )
            VALUES ($1, $2, $3, $4, $5, $6, 1)
            RETURNING id
            "#,
        )
        .bind(&params.name)
        .bind(&params.description)
        .bind(params.bpm.unwrap_or(120.0))
        .bind(params.time_signature_numerator.unwrap_or(4))
        .bind(params.time_signature_denominator.unwrap_or(4))
        .bind(Json(project_data))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create project: {}", e))?;

        let id: i64 = row
            .try_get("id")
            .map_err(|e| format!("Failed to get project id: {}", e))?;

        info!("Created project: {} (ID: {})", params.name, id);
        Ok(id)
    }

    /// Load a project by ID and deserialize to Project struct
    pub async fn load(&self, id: i64) -> Result<Option<Project>, String> {
        let record = self.get(id).await?;

        match record {
            Some(rec) => {
                let project: Project = serde_json::from_value(rec.project_data)
                    .map_err(|e| format!("Failed to deserialize project: {}", e))?;
                Ok(Some(project))
            }
            None => Ok(None),
        }
    }

    /// Get a project record by ID
    pub async fn get(&self, id: i64) -> Result<Option<ProjectRecord>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, bpm,
                   time_signature_numerator, time_signature_denominator,
                   project_data, schema_version,
                   created_at, updated_at, deleted_at
            FROM projects
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to get project: {}", e))?;

        match row {
            Some(r) => Ok(Some(row_to_record(&r)?)),
            None => Ok(None),
        }
    }

    /// Update a project from a Project struct
    pub async fn update_from_project(&self, id: i64, project: &Project) -> Result<(), String> {
        // Serialize project to JSON
        let project_json = serde_json::to_value(project)
            .map_err(|e| format!("Failed to serialize project: {}", e))?;

        // Extract metadata for indexed columns
        let params = UpdateProjectParams {
            name: Some(project.metadata.name.clone()),
            description: project.metadata.description.clone(),
            bpm: Some(project.settings.bpm),
            time_signature_numerator: Some(project.settings.time_signature_numerator as i32),
            time_signature_denominator: Some(project.settings.time_signature_denominator as i32),
            project_data: Some(project_json),
        };

        self.update(id, params).await
    }

    /// Update a project by ID
    pub async fn update(&self, id: i64, params: UpdateProjectParams) -> Result<(), String> {
        // Build dynamic UPDATE query
        let mut set_clauses = vec!["updated_at = NOW()".to_string()];
        let mut param_count = 1;

        let mut query = String::from("UPDATE projects SET ");

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
        if params.project_data.is_some() {
            param_count += 1;
            set_clauses.push(format!("project_data = ${}", param_count));
        }

        query.push_str(&set_clauses.join(", "));
        query.push_str(" WHERE id = $1 AND deleted_at IS NULL");

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
        if let Some(ref data) = params.project_data {
            q = q.bind(Json(data));
        }

        let result = q
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update project: {}", e))?;

        if result.rows_affected() == 0 {
            return Err(format!("Project {} not found", id));
        }

        info!("Updated project ID: {}", id);
        Ok(())
    }

    /// Delete a project (soft delete)
    pub async fn delete(&self, id: i64) -> Result<bool, String> {
        let result = sqlx::query(
            r#"
            UPDATE projects
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete project: {}", e))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("Deleted project ID: {}", id);
        }
        Ok(deleted)
    }

    /// Permanently delete a project (hard delete)
    pub async fn delete_permanent(&self, id: i64) -> Result<bool, String> {
        let result = sqlx::query("DELETE FROM projects WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to permanently delete project: {}", e))?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("Permanently deleted project ID: {}", id);
        }
        Ok(deleted)
    }

    /// List projects with optional filtering and pagination
    pub async fn list(&self, filters: ProjectFilters) -> Result<ProjectListResponse, String> {
        let limit = filters.limit.unwrap_or(50);
        let offset = filters.offset.unwrap_or(0);
        let sort_by = filters.sort_by.as_deref().unwrap_or("updated_at");
        let sort_dir = if filters.sort_desc.unwrap_or(true) {
            "DESC"
        } else {
            "ASC"
        };

        // Validate sort column to prevent SQL injection
        let valid_columns = ["id", "name", "bpm", "created_at", "updated_at"];
        let sort_column = if valid_columns.contains(&sort_by) {
            sort_by
        } else {
            "updated_at"
        };

        // Build WHERE clause for search
        let (where_clause, search_param) = if let Some(ref query) = filters.query {
            (
                "WHERE (name ILIKE $1 OR description ILIKE $1) AND deleted_at IS NULL",
                Some(format!("%{}%", query)),
            )
        } else {
            ("WHERE deleted_at IS NULL", None)
        };

        // Count total
        let count_query = format!("SELECT COUNT(*) as count FROM projects {}", where_clause);
        let count_row = if let Some(ref param) = search_param {
            sqlx::query(&count_query).bind(param).fetch_one(&self.pool).await
        } else {
            sqlx::query(&count_query).fetch_one(&self.pool).await
        }
        .map_err(|e| format!("Count query failed: {}", e))?;

        let total_count: i64 = count_row
            .try_get("count")
            .map_err(|e| format!("Failed to get count: {}", e))?;

        // Fetch projects
        let select_query = format!(
            r#"
            SELECT id, name, description, bpm,
                   time_signature_numerator, time_signature_denominator,
                   project_data, schema_version,
                   created_at, updated_at, deleted_at
            FROM projects
            {}
            ORDER BY {} {}
            LIMIT {} OFFSET {}
            "#,
            where_clause, sort_column, sort_dir, limit, offset
        );

        let rows = if let Some(ref param) = search_param {
            sqlx::query(&select_query).bind(param).fetch_all(&self.pool).await
        } else {
            sqlx::query(&select_query).fetch_all(&self.pool).await
        }
        .map_err(|e| format!("Select query failed: {}", e))?;

        let projects: Result<Vec<ProjectRecord>, String> =
            rows.iter().map(row_to_record).collect();

        Ok(ProjectListResponse {
            projects: projects?,
            total_count,
        })
    }

    /// Get recent projects (last updated)
    pub async fn get_recent(&self, limit: Option<i64>) -> Result<Vec<ProjectRecord>, String> {
        let limit = limit.unwrap_or(10);

        let rows = sqlx::query(
            r#"
            SELECT id, name, description, bpm,
                   time_signature_numerator, time_signature_denominator,
                   project_data, schema_version,
                   created_at, updated_at, deleted_at
            FROM projects
            WHERE deleted_at IS NULL
            ORDER BY updated_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get recent projects: {}", e))?;

        let projects: Result<Vec<ProjectRecord>, String> =
            rows.iter().map(row_to_record).collect();
        projects
    }

    /// Search projects by name or description
    pub async fn search(&self, query: &str) -> Result<Vec<ProjectRecord>, String> {
        let search_pattern = format!("%{}%", query);

        let rows = sqlx::query(
            r#"
            SELECT id, name, description, bpm,
                   time_signature_numerator, time_signature_denominator,
                   project_data, schema_version,
                   created_at, updated_at, deleted_at
            FROM projects
            WHERE (name ILIKE $1 OR description ILIKE $1)
              AND deleted_at IS NULL
            ORDER BY updated_at DESC
            LIMIT 50
            "#,
        )
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Search query failed: {}", e))?;

        let projects: Result<Vec<ProjectRecord>, String> =
            rows.iter().map(row_to_record).collect();
        projects
    }

    /// Get projects by tag (requires project_tags table)
    pub async fn get_by_tag(&self, tag: &str) -> Result<Vec<ProjectRecord>, String> {
        let rows = sqlx::query(
            r#"
            SELECT p.id, p.name, p.description, p.bpm,
                   p.time_signature_numerator, p.time_signature_denominator,
                   p.project_data, p.schema_version,
                   p.created_at, p.updated_at, p.deleted_at
            FROM projects p
            INNER JOIN project_tags pt ON p.id = pt.project_id
            WHERE pt.tag_name = $1 AND p.deleted_at IS NULL
            ORDER BY p.updated_at DESC
            "#,
        )
        .bind(tag)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Tag query failed: {}", e))?;

        let projects: Result<Vec<ProjectRecord>, String> =
            rows.iter().map(row_to_record).collect();
        projects
    }

    /// Add a tag to a project
    pub async fn add_tag(&self, project_id: i64, tag: &str) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO project_tags (project_id, tag_name)
            VALUES ($1, $2)
            ON CONFLICT (project_id, tag_name) DO NOTHING
            "#,
        )
        .bind(project_id)
        .bind(tag)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to add tag: {}", e))?;

        Ok(())
    }

    /// Remove a tag from a project
    pub async fn remove_tag(&self, project_id: i64, tag: &str) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM project_tags
            WHERE project_id = $1 AND tag_name = $2
            "#,
        )
        .bind(project_id)
        .bind(tag)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to remove tag: {}", e))?;

        Ok(())
    }

    /// Get all tags for a project
    pub async fn get_tags(&self, project_id: i64) -> Result<Vec<String>, String> {
        let rows = sqlx::query(
            r#"
            SELECT tag_name
            FROM project_tags
            WHERE project_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to get tags: {}", e))?;

        let tags: Result<Vec<String>, String> = rows
            .iter()
            .map(|row| row.try_get("tag_name").map_err(|e| e.to_string()))
            .collect();

        tags
    }
}

/// Helper function to convert a database row to ProjectRecord
fn row_to_record(row: &sqlx::postgres::PgRow) -> Result<ProjectRecord, String> {
    Ok(ProjectRecord {
        id: row
            .try_get("id")
            .map_err(|e| format!("Failed to get id: {}", e))?,
        name: row
            .try_get("name")
            .map_err(|e| format!("Failed to get name: {}", e))?,
        description: row.try_get("description").ok(),
        bpm: row
            .try_get("bpm")
            .map_err(|e| format!("Failed to get bpm: {}", e))?,
        time_signature_numerator: row
            .try_get("time_signature_numerator")
            .map_err(|e| format!("Failed to get time_signature_numerator: {}", e))?,
        time_signature_denominator: row
            .try_get("time_signature_denominator")
            .map_err(|e| format!("Failed to get time_signature_denominator: {}", e))?,
        project_data: row
            .try_get("project_data")
            .map_err(|e| format!("Failed to get project_data: {}", e))?,
        schema_version: row
            .try_get("schema_version")
            .map_err(|e| format!("Failed to get schema_version: {}", e))?,
        created_at: row
            .try_get("created_at")
            .map_err(|e| format!("Failed to get created_at: {}", e))?,
        updated_at: row
            .try_get("updated_at")
            .map_err(|e| format!("Failed to get updated_at: {}", e))?,
        deleted_at: row.try_get("deleted_at").ok(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project_params() {
        let params = CreateProjectParams {
            name: "Test Project".to_string(),
            description: None,
            bpm: Some(140.0),
            time_signature_numerator: None,
            time_signature_denominator: None,
            project_data: None,
        };

        assert_eq!(params.name, "Test Project");
        assert_eq!(params.bpm, Some(140.0));
    }

    #[test]
    fn test_update_project_params() {
        let params = UpdateProjectParams {
            name: Some("Updated".to_string()),
            description: None,
            bpm: Some(128.0),
            time_signature_numerator: None,
            time_signature_denominator: None,
            project_data: None,
        };

        assert_eq!(params.name, Some("Updated".to_string()));
        assert_eq!(params.bpm, Some(128.0));
    }

    #[test]
    fn test_project_filters_default() {
        let filters = ProjectFilters::default();
        assert!(filters.query.is_none());
        assert!(filters.limit.is_none());
    }
}
