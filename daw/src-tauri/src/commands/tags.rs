//! Tag Commands - Tauri commands for tag CRUD operations in DAW
//!
//! This module provides frontend-facing commands for:
//! - Creating, updating, and deleting tags
//! - Merging tags
//! - Importing and exporting tags as CSV
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use crate::commands::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

// =============================================================================
// TYPE DEFINITIONS
// =============================================================================

/// Tag for JSON serialization (frontend-friendly)
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub usage_count: i32,
}

/// CSV import result
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ImportResult {
    pub imported: i32,
    pub skipped: i32,
    pub errors: Vec<String>,
}

/// Request to create a new tag
#[derive(Clone, Debug, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
    pub category: String,
    pub description: Option<String>,
}

/// Request to update a tag
#[derive(Clone, Debug, Deserialize)]
pub struct UpdateTagRequest {
    pub name: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

/// Create a new tag
///
/// # Arguments
/// * `tag` - Tag data (name, category, optional description)
///
/// # Returns
/// The newly created tag with its assigned ID
#[tauri::command]
pub async fn create_tag(
    tag: CreateTagRequest,
    state: State<'_, AppState>,
) -> Result<Tag, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or("Database not connected")?;

    // Insert the new tag
    let row = sqlx::query_as::<_, (i32, String, Option<String>, i32)>(
        r#"
        INSERT INTO tags (name, category, usage_count, created_at)
        VALUES ($1, $2, 0, NOW())
        ON CONFLICT (name) DO UPDATE SET
            category = COALESCE(EXCLUDED.category, tags.category)
        RETURNING id, name, category, usage_count
        "#,
    )
    .bind(&tag.name)
    .bind(&tag.category)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to create tag: {}", e))?;

    Ok(Tag {
        id: row.0,
        name: row.1,
        category: row.2,
        description: tag.description,
        usage_count: row.3,
    })
}

/// Update an existing tag
///
/// # Arguments
/// * `tag_id` - ID of the tag to update
/// * `updates` - Fields to update (all optional)
#[tauri::command]
pub async fn update_tag(
    tag_id: i32,
    updates: UpdateTagRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or("Database not connected")?;

    // Build dynamic update query based on provided fields
    let mut set_clauses = Vec::new();
    let mut param_idx = 1;

    if updates.name.is_some() {
        param_idx += 1;
        set_clauses.push(format!("name = ${}", param_idx - 1));
    }
    if updates.category.is_some() {
        param_idx += 1;
        set_clauses.push(format!("category = ${}", param_idx - 1));
    }

    if set_clauses.is_empty() {
        return Ok(()); // Nothing to update
    }

    // We need to use raw SQL since we're building dynamic updates
    // Build the query carefully to avoid injection
    let query = if updates.name.is_some() && updates.category.is_some() {
        sqlx::query("UPDATE tags SET name = $1, category = $2 WHERE id = $3")
            .bind(updates.name.as_ref().unwrap())
            .bind(updates.category.as_ref().unwrap())
            .bind(tag_id)
    } else if updates.name.is_some() {
        sqlx::query("UPDATE tags SET name = $1 WHERE id = $2")
            .bind(updates.name.as_ref().unwrap())
            .bind(tag_id)
    } else if updates.category.is_some() {
        sqlx::query("UPDATE tags SET category = $1 WHERE id = $2")
            .bind(updates.category.as_ref().unwrap())
            .bind(tag_id)
    } else {
        return Ok(());
    };

    query
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update tag: {}", e))?;

    Ok(())
}

/// Delete a tag and its associations
///
/// # Arguments
/// * `tag_id` - ID of the tag to delete
#[tauri::command]
pub async fn delete_tag(tag_id: i32, state: State<'_, AppState>) -> Result<(), String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or("Database not connected")?;

    // Start a transaction to ensure atomicity
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // First, delete all file_tags associations
    sqlx::query("DELETE FROM file_tags WHERE tag_id = $1")
        .bind(tag_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to remove tag associations: {}", e))?;

    // Then delete the tag itself
    let result = sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(tag_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to delete tag: {}", e))?;

    if result.rows_affected() == 0 {
        return Err(format!("Tag with id {} not found", tag_id));
    }

    // Commit the transaction
    tx.commit()
        .await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(())
}

/// Merge multiple tags into a single target tag
///
/// All files tagged with source tags will be re-tagged with the target tag.
/// Source tags will be deleted after merging.
///
/// # Arguments
/// * `source_tag_ids` - IDs of tags to merge from (these will be deleted)
/// * `target_tag_id` - ID of tag to merge into (this will remain)
#[tauri::command]
pub async fn merge_tags(
    source_tag_ids: Vec<i32>,
    target_tag_id: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if source_tag_ids.is_empty() {
        return Ok(());
    }

    if source_tag_ids.contains(&target_tag_id) {
        return Err("Target tag cannot be in source tags".to_string());
    }

    let pool = state
        .db_pool
        .as_ref()
        .ok_or("Database not connected")?;

    // Start a transaction
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // Move all file associations from source tags to target tag
    // Use ON CONFLICT to avoid duplicates
    sqlx::query(
        r#"
        INSERT INTO file_tags (file_id, tag_id, added_at, added_by)
        SELECT DISTINCT file_id, $1, NOW(), 'merge'
        FROM file_tags
        WHERE tag_id = ANY($2)
        ON CONFLICT (file_id, tag_id) DO NOTHING
        "#,
    )
    .bind(target_tag_id)
    .bind(&source_tag_ids)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to move tag associations: {}", e))?;

    // Delete old associations
    sqlx::query("DELETE FROM file_tags WHERE tag_id = ANY($1)")
        .bind(&source_tag_ids)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to remove old associations: {}", e))?;

    // Delete the source tags
    sqlx::query("DELETE FROM tags WHERE id = ANY($1)")
        .bind(&source_tag_ids)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to delete source tags: {}", e))?;

    // Update usage count for target tag
    sqlx::query(
        r#"
        UPDATE tags
        SET usage_count = (SELECT COUNT(*) FROM file_tags WHERE tag_id = $1)
        WHERE id = $1
        "#,
    )
    .bind(target_tag_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("Failed to update usage count: {}", e))?;

    // Commit the transaction
    tx.commit()
        .await
        .map_err(|e| format!("Failed to commit transaction: {}", e))?;

    Ok(())
}

/// Export all tags to CSV format
///
/// # Returns
/// CSV string with columns: id, name, category, count
#[tauri::command]
pub async fn export_tags_csv(state: State<'_, AppState>) -> Result<String, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or("Database not connected")?;

    // Fetch all tags
    let tags = sqlx::query_as::<_, (i32, String, Option<String>, i32)>(
        r#"
        SELECT id, name, category, usage_count
        FROM tags
        ORDER BY category, name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch tags: {}", e))?;

    // Build CSV
    let mut csv = String::from("id,name,category,count\n");
    for (id, name, category, count) in tags {
        // Escape CSV fields (handle commas and quotes)
        let escaped_name = escape_csv_field(&name);
        let escaped_category = category
            .as_ref()
            .map(|c| escape_csv_field(c))
            .unwrap_or_default();
        csv.push_str(&format!("{},{},{},{}\n", id, escaped_name, escaped_category, count));
    }

    Ok(csv)
}

/// Import tags from CSV string
///
/// Expected format: id,name,category,count (header row required)
/// The id and count columns are ignored during import.
///
/// # Arguments
/// * `csv` - CSV data to import
///
/// # Returns
/// Import result with counts of imported, skipped, and errored rows
#[tauri::command]
pub async fn import_tags_csv(csv: String, state: State<'_, AppState>) -> Result<ImportResult, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or("Database not connected")?;

    let mut result = ImportResult {
        imported: 0,
        skipped: 0,
        errors: Vec::new(),
    };

    let lines: Vec<&str> = csv.lines().collect();
    if lines.is_empty() {
        return Ok(result);
    }

    // Skip header row
    for (line_num, line) in lines.iter().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        // Parse CSV line (simple parser, doesn't handle all edge cases)
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 3 {
            result.errors.push(format!("Line {}: Invalid format", line_num + 1));
            continue;
        }

        // fields[0] is id (ignored), fields[1] is name, fields[2] is category
        let name = fields[1].trim().trim_matches('"');
        let category = fields[2].trim().trim_matches('"');

        if name.is_empty() {
            result.errors.push(format!("Line {}: Empty name", line_num + 1));
            continue;
        }

        // Insert or update tag
        let insert_result = sqlx::query(
            r#"
            INSERT INTO tags (name, category, usage_count, created_at)
            VALUES ($1, NULLIF($2, ''), 0, NOW())
            ON CONFLICT (name) DO NOTHING
            "#,
        )
        .bind(name)
        .bind(category)
        .execute(pool)
        .await;

        match insert_result {
            Ok(r) => {
                if r.rows_affected() > 0 {
                    result.imported += 1;
                } else {
                    result.skipped += 1; // Tag already exists
                }
            }
            Err(e) => {
                result.errors.push(format!("Line {}: {}", line_num + 1, e));
            }
        }
    }

    Ok(result)
}

/// Get a single tag by ID
#[tauri::command]
pub async fn get_tag(tag_id: i32, state: State<'_, AppState>) -> Result<Tag, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or("Database not connected")?;

    let row = sqlx::query_as::<_, (i32, String, Option<String>, i32)>(
        r#"
        SELECT id, name, category, usage_count
        FROM tags
        WHERE id = $1
        "#,
    )
    .bind(tag_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to fetch tag: {}", e))?
    .ok_or_else(|| format!("Tag with id {} not found", tag_id))?;

    Ok(Tag {
        id: row.0,
        name: row.1,
        category: row.2,
        description: None,
        usage_count: row.3,
    })
}

/// Get all tags
#[tauri::command]
pub async fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<Tag>, String> {
    let pool = state
        .db_pool
        .as_ref()
        .ok_or("Database not connected")?;

    let rows = sqlx::query_as::<_, (i32, String, Option<String>, i32)>(
        r#"
        SELECT id, name, category, usage_count
        FROM tags
        ORDER BY category, name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to fetch tags: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|(id, name, category, usage_count)| Tag {
            id,
            name,
            category,
            description: None,
            usage_count,
        })
        .collect())
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Escape a CSV field by quoting if necessary
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_field_simple() {
        assert_eq!(escape_csv_field("hello"), "hello");
    }

    #[test]
    fn test_escape_csv_field_with_comma() {
        assert_eq!(escape_csv_field("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn test_escape_csv_field_with_quotes() {
        assert_eq!(escape_csv_field("say \"hello\""), "\"say \"\"hello\"\"\"");
    }

    #[test]
    fn test_escape_csv_field_with_newline() {
        assert_eq!(escape_csv_field("hello\nworld"), "\"hello\nworld\"");
    }
}
