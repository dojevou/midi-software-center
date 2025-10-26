//! Archive Collection Import Command
//!
//! Processes entire collections of nested archives, extracting and importing
//! all MIDI files with automatic tagging.
//!
//! # Archetype: Grown-up Script (Tauri Command Wrapper)
//! - Thin wrapper around core functionality
//! - Coordinates decompressor + file import modules
//! - Provides progress feedback to UI

use crate::AppState;
use crate::io::decompressor::extractor::{extract_archive, ExtractionConfig};
use crate::commands::file_import::import_directory;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::{Emitter, State, Window};

/// Summary of archive collection import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveImportSummary {
    pub total_archives: usize,
    pub total_files_imported: usize,
    pub total_files_skipped: usize,
    pub total_errors: usize,
    pub duration_secs: f64,
    pub archives_processed: Vec<ArchiveStatus>,
}

/// Status of individual archive processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStatus {
    pub archive_name: String,
    pub midi_files_found: usize,
    pub files_imported: usize,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Import entire collection of archives (recursively extracts and imports all MIDI files)
///
/// # Arguments
/// * `collection_path` - Directory containing zip archives
/// * `state` - Application state
/// * `window` - Tauri window for progress events
///
/// # Frontend Usage
/// ```typescript
/// await invoke('import_archive_collection', {
///   collectionPath: '/home/user/midi-collection/'
/// });
/// ```
#[tauri::command]
pub async fn import_archive_collection(
    collection_path: String,
    state: State<'_, AppState>,
    window: Window,
) -> Result<ArchiveImportSummary, String> {
    let start_time = std::time::Instant::now();
    let collection_dir = Path::new(&collection_path);

    if !collection_dir.exists() {
        return Err(format!("Collection directory not found: {}", collection_path));
    }

    if !collection_dir.is_dir() {
        return Err(format!("Path is not a directory: {}", collection_path));
    }

    println!("\n🚀 Starting archive collection import from: {}", collection_path);
    println!("📦 Scanning for zip archives...\n");

    // Scan for zip files
    let archives: Vec<_> = std::fs::read_dir(collection_dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("zip"))
                .unwrap_or(false)
        })
        .collect();

    let total_archives = archives.len();
    println!("✅ Found {} archives to process\n", total_archives);

    let mut archive_statuses = Vec::new();
    let mut total_files_imported = 0;
    let mut total_files_skipped = 0;
    let mut total_errors = 0;

    // Process each archive
    for (index, entry) in archives.iter().enumerate() {
        let archive_path = entry.path();
        let archive_name = archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📦 [{}/{}] Processing: {}", index + 1, total_archives, archive_name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Emit progress event
        let _ = window.emit("archive-progress", serde_json::json!({
            "current": index + 1,
            "total": total_archives,
            "archive_name": archive_name
        }));

        // Process this archive
        let status = process_single_archive(
            &archive_path,
            &archive_name,
            state.clone(),
            window.clone(),
        ).await;

        match &status {
            Ok(s) => {
                total_files_imported += s.files_imported;
                total_files_skipped += s.midi_files_found.saturating_sub(s.files_imported);
                println!("✅ Success: {} MIDIs found, {} imported\n", s.midi_files_found, s.files_imported);
            }
            Err(e) => {
                total_errors += 1;
                println!("❌ Error: {}\n", e);
            }
        }

        archive_statuses.push(status.unwrap_or_else(|e| ArchiveStatus {
            archive_name: archive_name.clone(),
            midi_files_found: 0,
            files_imported: 0,
            success: false,
            error_message: Some(e),
        }));
    }

    let duration = start_time.elapsed();
    let duration_secs = duration.as_secs_f64();

    println!("\n╔══════════════════════════════════════════════╗");
    println!("║      ARCHIVE COLLECTION IMPORT COMPLETE      ║");
    println!("╠══════════════════════════════════════════════╣");
    println!("║ Archives Processed: {:>28} ║", total_archives);
    println!("║ Files Imported:     {:>28} ║", total_files_imported);
    println!("║ Files Skipped:      {:>28} ║", total_files_skipped);
    println!("║ Errors:             {:>28} ║", total_errors);
    println!("║ Duration:           {:>25.1}s ║", duration_secs);
    println!("║ Rate:               {:>23.0} f/s ║", total_files_imported as f64 / duration_secs);
    println!("╚══════════════════════════════════════════════╝\n");

    Ok(ArchiveImportSummary {
        total_archives,
        total_files_imported,
        total_files_skipped,
        total_errors,
        duration_secs,
        archives_processed: archive_statuses,
    })
}

/// Process a single archive file
async fn process_single_archive(
    archive_path: &Path,
    archive_name: &str,
    state: State<'_, AppState>,
    window: Window,
) -> Result<ArchiveStatus, String> {
    // Create temporary extraction directory
    let temp_dir = std::env::temp_dir().join(format!("midi_extract_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    // Extract with recursive decompression
    println!("   📂 Extracting (recursive, max depth 10)...");
    let config = ExtractionConfig::default(); // Uses max_depth: 10
    let extract_result = extract_archive(archive_path, &temp_dir, &config)
        .map_err(|e| format!("Extraction failed: {}", e))?;

    let midi_count = extract_result.midi_files.len();
    println!("   🎵 Found {} MIDI files", midi_count);

    if midi_count == 0 {
        // Cleanup and return
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Ok(ArchiveStatus {
            archive_name: archive_name.to_string(),
            midi_files_found: 0,
            files_imported: 0,
            success: true,
            error_message: None,
        });
    }

    // Import extracted files using existing import_directory command
    println!("   💾 Importing to database with auto-tagging...");
    let import_result = import_directory(
        temp_dir.to_string_lossy().to_string(),
        true, // recursive
        Some(archive_name.trim_end_matches(".zip").to_string()), // category from archive name
        state.clone(),
        window.clone(),
    ).await;

    // Cleanup temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    match import_result {
        Ok(summary) => {
            Ok(ArchiveStatus {
                archive_name: archive_name.to_string(),
                midi_files_found: midi_count,
                files_imported: summary.imported,
                success: true,
                error_message: None,
            })
        }
        Err(e) => {
            Err(format!("Import failed: {}", e))
        }
    }
}
