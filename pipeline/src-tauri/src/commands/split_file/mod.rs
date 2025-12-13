//! Track Splitting Commands - Split multi-track MIDI files into individual tracks
//!
//! Handles database queries, file I/O, and transaction management.
//! Actual splitting logic delegated to track_splitter module.

pub mod core;
pub mod db;
pub mod helpers;
pub mod types;

use std::path::PathBuf;

use crate::AppState;
use tauri::State;

pub use core::split_and_import;
pub use helpers::{generate_split_filename, sanitize_filename};
pub use types::{SplitCommandError, SplitResult};

/// Tauri command: Split a multi-track MIDI file into individual tracks.
#[tauri::command]
pub async fn split_file(
    state: State<'_, AppState>,
    file_id: i64,
    output_dir: String,
) -> Result<SplitResult, String> {
    let pool = state.database.pool().await;
    let output_path = PathBuf::from(output_dir);

    split_and_import(file_id, output_path, &pool).await.map_err(|e| e.to_string())
}

/// Tauri command: Split multiple MIDI files into individual tracks (batch).
#[tauri::command]
pub async fn split_file_batch(
    state: State<'_, AppState>,
    file_ids: Vec<i64>,
    output_dir: String,
) -> Result<Vec<Result<SplitResult, String>>, String> {
    let pool = state.database.pool().await;
    let output_path = PathBuf::from(output_dir);

    let mut results = Vec::with_capacity(file_ids.len());

    for file_id in file_ids {
        let result = split_and_import(file_id, output_path.clone(), &pool)
            .await
            .map_err(|e| e.to_string());
        results.push(result);
    }

    Ok(results)
}
