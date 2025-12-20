//! VIP3 Dynamic Filter Counts Command
//!
//! This module provides a command to get filter counts that update dynamically
//! based on the current filter selections. Unlike the static counts from
//! `get_vip3_filter_counts`, these counts show how many files would match
//! if a particular filter value were selected given the current filters.

use crate::db::models::{FilterCounts, Vip3Filters};
use crate::db::repositories::Vip3Repository;
use crate::AppState;
use tauri::State;
use tracing::{error, info};

/// Get VIP3 dynamic filter counts for all categories
///
/// This command returns counts for folders, instruments, timbres, styles,
/// articulations, BPM ranges, keys, and channels that update based on the
/// current filter selections.
///
/// # Arguments
/// * `filters` - Current filter selections to apply
/// * `state` - Application state containing database connection
///
/// # Returns
/// * `FilterCounts` - Counts for each filter category
///
/// # Performance
/// Target execution time: <50ms with parallel query execution
#[tauri::command]
pub async fn get_vip3_dynamic_filter_counts(
    filters: Vip3Filters,
    state: State<'_, AppState>,
) -> Result<FilterCounts, String> {
    info!("Getting VIP3 dynamic filter counts with filters: {:?}", filters);

    let pool = state.database.pool().await;
    let repo = Vip3Repository::new(pool);

    let counts = repo.get_filter_counts(&filters).await.map_err(|e| {
        error!("Failed to get filter counts: {:?}", e);
        format!("Failed to get filter counts: {}", e)
    })?;

    info!(
        "Dynamic filter counts retrieved: {} total matches",
        counts.total_matches
    );

    Ok(counts)
}
