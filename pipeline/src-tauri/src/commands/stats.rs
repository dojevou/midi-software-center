/// Statistics command handlers - GROWN-UP SCRIPT ARCHETYPE
///
/// PURPOSE: Database statistics and metrics
/// ARCHETYPE: Grown-up Script (I/O operations)
///
/// ✅ CAN: Perform database I/O
/// ✅ CAN: Have side effects (complex queries)
/// ✅ SHOULD: Handle errors properly
/// ❌ NO: Complex business logic (delegate to Trusty Modules)
use crate::AppState;
use std::collections::HashMap;
use tauri::State;

// =============================================================================
// TAURI COMMANDS
// =============================================================================

/// Get file count breakdown by category (implementation for tests and reuse)
pub async fn get_category_stats_impl(state: &AppState) -> Result<HashMap<String, i64>, String> {
    let results: Vec<(Option<String>, i64)> = sqlx::query_as(
        r#"
        SELECT fc.primary_category::text as category, COUNT(*) as count
        FROM files f
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        GROUP BY fc.primary_category
        ORDER BY count DESC
        "#,
    )
    .fetch_all(&state.database.pool().await)
    .await
    .map_err(|e| format!("Failed to get category stats: {}", e))?;

    let mut stats = HashMap::new();
    for (category, count) in results {
        let category_name = category.unwrap_or_else(|| "Uncategorized".to_string());
        stats.insert(category_name, count);
    }

    Ok(stats)
}

/// Get file count breakdown by category
///
/// Returns a map of category names to file counts.
///
/// # Frontend Usage
///
/// ```typescript
/// const stats = await invoke<Record<string, number>>('get_category_stats');
/// // { "bass": 150, "drums": 200, "melody": 100 }
/// ```
#[tauri::command]
pub async fn get_category_stats(
    state: State<'_, AppState>,
) -> Result<HashMap<String, i64>, String> {
    get_category_stats_impl(&state).await
}

/// Get file count breakdown by manufacturer
///
/// Returns a map of manufacturer names to file counts.
///
/// # Frontend Usage
///
/// ```typescript
/// const stats = await invoke<Record<string, number>>('get_manufacturer_stats');
/// ```
#[tauri::command]
pub async fn get_manufacturer_stats(
    state: State<'_, AppState>,
) -> Result<HashMap<String, i64>, String> {
    let results: Vec<(Option<String>, i64)> = sqlx::query_as(
        r#"
        SELECT mm.manufacturer::text as manufacturer, COUNT(*) as count
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        WHERE mm.manufacturer IS NOT NULL
        GROUP BY mm.manufacturer
        ORDER BY count DESC
        "#,
    )
    .fetch_all(&state.database.pool().await)
    .await
    .map_err(|e| format!("Failed to get manufacturer stats: {}", e))?;

    let mut stats = HashMap::new();
    for (manufacturer, count) in results {
        if let Some(mfr) = manufacturer {
            stats.insert(mfr, count);
        }
    }

    Ok(stats)
}

/// Get file count breakdown by key signature
///
/// Returns a map of key signatures to file counts.
///
/// # Frontend Usage
///
/// ```typescript
/// const stats = await invoke<Record<string, number>>('get_key_signature_stats');
/// ```
#[tauri::command]
pub async fn get_key_signature_stats(
    state: State<'_, AppState>,
) -> Result<HashMap<String, i64>, String> {
    let results: Vec<(Option<String>, i64)> = sqlx::query_as(
        r#"
        SELECT mm.key_signature::text as key_sig, COUNT(*) as count
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        WHERE mm.key_signature IS NOT NULL
        GROUP BY mm.key_signature
        ORDER BY count DESC
        "#,
    )
    .fetch_all(&state.database.pool().await)
    .await
    .map_err(|e| format!("Failed to get key signature stats: {}", e))?;

    let mut stats = HashMap::new();
    for (key_sig, count) in results {
        if let Some(key) = key_sig {
            stats.insert(key, count);
        }
    }

    Ok(stats)
}

/// Get count of recently added files (last 7 days)
///
/// # Frontend Usage
///
/// ```typescript
/// const count = await invoke<number>('get_recently_added_count');
/// ```
#[tauri::command]
pub async fn get_recently_added_count(state: State<'_, AppState>) -> Result<i64, String> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM files
        WHERE created_at >= NOW() - INTERVAL '7 days'
        "#,
    )
    .fetch_one(&state.database.pool().await)
    .await
    .map_err(|e| format!("Failed to get recently added count: {}", e))?;

    Ok(count.0)
}

/// Get count of duplicate files
///
/// Files are considered duplicates if they have the same content hash.
///
/// # Frontend Usage
///
/// ```typescript
/// const count = await invoke<number>('get_duplicate_count');
/// ```
#[tauri::command]
pub async fn get_duplicate_count(state: State<'_, AppState>) -> Result<i64, String> {
    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM (
            SELECT content_hash
            FROM files
            GROUP BY content_hash
            HAVING COUNT(*) > 1
        ) as duplicates
        "#,
    )
    .fetch_one(&state.database.pool().await)
    .await
    .map_err(|e| format!("Failed to get duplicate count: {}", e))?;

    Ok(count.0)
}

/// Get database size as formatted string (implementation for tests and reuse)
pub async fn get_database_size_impl(state: &AppState) -> Result<String, String> {
    let size: (Option<String>,) = sqlx::query_as(
        r#"
        SELECT pg_size_pretty(pg_database_size(current_database()))
        "#,
    )
    .fetch_one(&state.database.pool().await)
    .await
    .map_err(|e| format!("Failed to get database size: {}", e))?;

    Ok(size.0.unwrap_or_else(|| "Unknown".to_string()))
}

/// Get database size as formatted string
///
/// Returns the total size of the database in a human-readable format.
///
/// # Frontend Usage
///
/// ```typescript
/// const size = await invoke<string>('get_database_size');
/// // "125.4 MB"
/// ```
#[tauri::command]
pub async fn get_database_size(state: State<'_, AppState>) -> Result<String, String> {
    get_database_size_impl(&state).await
}

/// Check database health status
///
/// Returns health status based on connection and basic query tests.
///
/// # Frontend Usage
///
/// ```typescript
/// const health = await invoke<'good' | 'warning' | 'error'>('check_database_health');
/// ```
#[tauri::command]
pub async fn check_database_health(state: State<'_, AppState>) -> Result<String, String> {
    // Try a simple query
    match state.database.test_connection().await {
        Ok(_) => {
            // Check if we can count files
            match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM files")
                .fetch_one(&state.database.pool().await)
                .await
            {
                Ok(_) => Ok("good".to_string()),
                Err(_) => Ok("warning".to_string()),
            }
        },
        Err(_) => Ok("error".to_string()),
    }
}
