#![allow(dead_code)]
//! MIDI File Repair Tauri Commands
//!
//! This module provides comprehensive Tauri commands for detecting and repairing
//! corrupted or malformed MIDI files. Supports multiple repair strategies and
//! batch processing.
//!
//! # Features
//! - Detect various MIDI corruption types
//! - Auto-repair with multiple strategies
//! - Batch repair for large collections
//! - Repair preview (dry run)
//! - Detailed repair reports
//! - Backup original files before repair

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write as IoWrite};
use std::path::Path;
use std::sync::Arc;
use tauri::State;
use thiserror::Error;
use tokio::sync::RwLock;

// ============================================================================
// TYPES
// ============================================================================

/// Types of MIDI file corruption
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CorruptionType {
    /// Missing end-of-track marker
    MissingEndOfTrack,
    /// Invalid header chunk
    InvalidHeader,
    /// Truncated file
    TruncatedFile,
    /// Invalid track length
    InvalidTrackLength,
    /// Garbage data at end
    GarbageAtEnd,
    /// Invalid delta time
    InvalidDeltaTime,
    /// Invalid event type
    InvalidEventType,
    /// Invalid running status
    InvalidRunningStatus,
    /// Invalid meta event
    InvalidMetaEvent,
    /// Invalid sysex event
    InvalidSysexEvent,
    /// Missing track header
    MissingTrackHeader,
    /// Track count mismatch
    TrackCountMismatch,
    /// Nested sysex events
    NestedSysex,
    /// Invalid channel number
    InvalidChannel,
    /// Invalid note number
    InvalidNote,
    /// Invalid velocity
    InvalidVelocity,
    /// File too large
    FileTooLarge,
    /// Unknown corruption
    Unknown,
}

impl CorruptionType {
    pub fn description(&self) -> &'static str {
        match self {
            Self::MissingEndOfTrack => "End-of-track marker (0xFF 0x2F 0x00) is missing",
            Self::InvalidHeader => "MIDI file header is invalid or corrupted",
            Self::TruncatedFile => "File is truncated before expected end",
            Self::InvalidTrackLength => "Track length does not match actual data",
            Self::GarbageAtEnd => "Extra data after last track",
            Self::InvalidDeltaTime => "Delta time contains invalid variable-length quantity",
            Self::InvalidEventType => "Unknown or invalid MIDI event type",
            Self::InvalidRunningStatus => "Running status used incorrectly",
            Self::InvalidMetaEvent => "Meta event has invalid structure",
            Self::InvalidSysexEvent => "System exclusive event is malformed",
            Self::MissingTrackHeader => "Track chunk header (MTrk) is missing",
            Self::TrackCountMismatch => "Number of tracks does not match header",
            Self::NestedSysex => "Sysex event started before previous one ended",
            Self::InvalidChannel => "Channel number out of range (0-15)",
            Self::InvalidNote => "Note number out of range (0-127)",
            Self::InvalidVelocity => "Velocity out of range (0-127)",
            Self::FileTooLarge => "File exceeds maximum allowed size",
            Self::Unknown => "Unknown or unidentified corruption",
        }
    }

    pub fn severity(&self) -> CorruptionSeverity {
        match self {
            Self::GarbageAtEnd | Self::MissingEndOfTrack => CorruptionSeverity::Minor,
            Self::InvalidTrackLength | Self::TrackCountMismatch => CorruptionSeverity::Moderate,
            Self::InvalidHeader | Self::TruncatedFile | Self::MissingTrackHeader => {
                CorruptionSeverity::Severe
            },
            _ => CorruptionSeverity::Moderate,
        }
    }
}

/// Severity level of corruption
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum CorruptionSeverity {
    Minor,
    Moderate,
    Severe,
    Critical,
}

/// Repair strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RepairStrategy {
    /// Add missing end-of-track markers
    AddEndOfTrack,
    /// Remove garbage data at end
    TrimGarbage,
    /// Fix track length headers
    FixTrackLength,
    /// Fix header track count
    FixTrackCount,
    /// Remove invalid events
    RemoveInvalidEvents,
    /// Clamp out-of-range values
    ClampValues,
    /// Rebuild entire file structure
    Rebuild,
    /// Conservative repair (safest)
    Conservative,
    /// Aggressive repair (most fixes)
    Aggressive,
    /// Auto-detect best strategy
    Auto,
}

impl RepairStrategy {
    pub fn description(&self) -> &'static str {
        match self {
            Self::AddEndOfTrack => "Add missing end-of-track markers to all tracks",
            Self::TrimGarbage => "Remove garbage data after the last valid track",
            Self::FixTrackLength => "Correct track length values in headers",
            Self::FixTrackCount => "Update header to match actual track count",
            Self::RemoveInvalidEvents => "Remove events that cannot be parsed",
            Self::ClampValues => "Clamp out-of-range note/velocity/channel values",
            Self::Rebuild => "Rebuild entire file from parsed events",
            Self::Conservative => "Apply only safe repairs (minimal changes)",
            Self::Aggressive => "Apply all possible repairs",
            Self::Auto => "Automatically detect and apply appropriate repairs",
        }
    }
}

/// Detection result for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub file_path: String,
    pub file_size: u64,
    pub is_valid: bool,
    pub corruptions: Vec<CorruptionInfo>,
    pub repairable: bool,
    pub suggested_strategies: Vec<RepairStrategy>,
    pub confidence: f64,
}

/// Information about a single corruption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorruptionInfo {
    pub corruption_type: CorruptionType,
    pub severity: CorruptionSeverity,
    pub location: Option<u64>,
    pub description: String,
    pub repairable: bool,
}

/// Repair result for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairResult {
    pub file_path: String,
    pub success: bool,
    pub strategy_used: RepairStrategy,
    pub corruptions_fixed: Vec<CorruptionType>,
    pub corruptions_remaining: Vec<CorruptionType>,
    pub backup_path: Option<String>,
    pub original_size: u64,
    pub repaired_size: u64,
    pub error: Option<String>,
}

/// Options for repair operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairOptions {
    pub strategy: RepairStrategy,
    pub create_backup: bool,
    pub backup_suffix: String,
    pub dry_run: bool,
    pub max_file_size: u64,
    pub stop_on_critical: bool,
}

impl Default for RepairOptions {
    fn default() -> Self {
        Self {
            strategy: RepairStrategy::Auto,
            create_backup: true,
            backup_suffix: ".bak".to_string(),
            dry_run: false,
            max_file_size: 100 * 1024 * 1024, // 100 MB
            stop_on_critical: true,
        }
    }
}

/// Batch repair progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    pub total_files: usize,
    pub processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub current_file: Option<String>,
    pub elapsed_ms: u64,
    pub estimated_remaining_ms: Option<u64>,
}

/// Batch repair result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRepairResult {
    pub total_files: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub results: Vec<RepairResult>,
    pub elapsed_ms: u64,
}

/// Repair statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepairStatistics {
    pub files_scanned: u64,
    pub files_repaired: u64,
    pub files_failed: u64,
    pub corruptions_by_type: HashMap<CorruptionType, u64>,
    pub strategies_used: HashMap<RepairStrategy, u64>,
    pub total_bytes_processed: u64,
    pub total_bytes_saved: i64,
}

/// State for repair operations
pub struct RepairState {
    pub statistics: Arc<RwLock<RepairStatistics>>,
    pub current_batch: Arc<RwLock<Option<BatchProgress>>>,
    pub cancel_requested: Arc<RwLock<bool>>,
}

impl Default for RepairState {
    fn default() -> Self {
        Self {
            statistics: Arc::new(RwLock::new(RepairStatistics::default())),
            current_batch: Arc::new(RwLock::new(None)),
            cancel_requested: Arc::new(RwLock::new(false)),
        }
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Error)]
pub enum RepairError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Not a MIDI file: {0}")]
    NotMidiFile(String),

    #[error("File too large: {0} bytes (max: {1})")]
    FileTooLarge(u64, u64),

    #[error("Cannot repair: {0}")]
    CannotRepair(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Backup failed: {0}")]
    BackupFailed(String),

    #[error("Critical corruption: {0}")]
    CriticalCorruption(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("State error: {0}")]
    StateError(String),
}

impl From<std::io::Error> for RepairError {
    fn from(err: std::io::Error) -> Self {
        RepairError::IoError(err.to_string())
    }
}

impl From<RepairError> for String {
    fn from(err: RepairError) -> Self {
        err.to_string()
    }
}

// ============================================================================
// MIDI PARSING HELPERS
// ============================================================================

const MIDI_HEADER: &[u8] = b"MThd";
const TRACK_HEADER: &[u8] = b"MTrk";
const END_OF_TRACK: &[u8] = &[0xFF, 0x2F, 0x00];

fn read_file_bytes(path: &Path, max_size: u64) -> Result<Vec<u8>, RepairError> {
    let metadata = fs::metadata(path)
        .map_err(|e| RepairError::IoError(format!("Cannot read metadata: {}", e)))?;

    if metadata.len() > max_size {
        return Err(RepairError::FileTooLarge(metadata.len(), max_size));
    }

    let mut file = fs::File::open(path)?;
    let mut buffer = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn is_midi_file(data: &[u8]) -> bool {
    data.len() >= 14 && &data[0..4] == MIDI_HEADER
}

fn find_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
    data.windows(pattern.len()).position(|w| w == pattern)
}

fn find_all_patterns(data: &[u8], pattern: &[u8]) -> Vec<usize> {
    data.windows(pattern.len())
        .enumerate()
        .filter(|(_, w)| *w == pattern)
        .map(|(i, _)| i)
        .collect()
}

fn _read_u16_be(data: &[u8], offset: usize) -> u16 {
    if offset + 2 > data.len() {
        return 0;
    }
    ((data[offset] as u16) << 8) | (data[offset + 1] as u16)
}

fn read_u32_be(data: &[u8], offset: usize) -> u32 {
    if offset + 4 > data.len() {
        return 0;
    }
    ((data[offset] as u32) << 24)
        | ((data[offset + 1] as u32) << 16)
        | ((data[offset + 2] as u32) << 8)
        | (data[offset + 3] as u32)
}

fn write_u32_be(value: u32) -> [u8; 4] {
    [(value >> 24) as u8, (value >> 16) as u8, (value >> 8) as u8, value as u8]
}

// ============================================================================
// IMPLEMENTATION FUNCTIONS
// ============================================================================

/// Detect corruption in a MIDI file
pub async fn detect_corruption_impl(
    file_path: &str,
    max_file_size: u64,
) -> Result<DetectionResult, RepairError> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(RepairError::FileNotFound(file_path.to_string()));
    }

    let data = read_file_bytes(path, max_file_size)?;
    let file_size = data.len() as u64;

    if !is_midi_file(&data) {
        return Err(RepairError::NotMidiFile(file_path.to_string()));
    }

    let mut corruptions = Vec::new();
    let mut suggested_strategies = Vec::new();

    // Check header
    let header_length = read_u32_be(&data, 4);
    if header_length != 6 {
        corruptions.push(CorruptionInfo {
            corruption_type: CorruptionType::InvalidHeader,
            severity: CorruptionSeverity::Severe,
            location: Some(4),
            description: format!("Header length is {} (expected 6)", header_length),
            repairable: false,
        });
    }

    // Get track count from header
    let declared_tracks = _read_u16_be(&data, 10);

    // Count actual tracks
    let track_positions = find_all_patterns(&data, TRACK_HEADER);
    let actual_tracks = track_positions.len();

    if declared_tracks as usize != actual_tracks {
        corruptions.push(CorruptionInfo {
            corruption_type: CorruptionType::TrackCountMismatch,
            severity: CorruptionSeverity::Moderate,
            location: Some(10),
            description: format!(
                "Header declares {} tracks but found {}",
                declared_tracks, actual_tracks
            ),
            repairable: true,
        });
        suggested_strategies.push(RepairStrategy::FixTrackCount);
    }

    // Check each track for end-of-track marker
    let eot_positions = find_all_patterns(&data, END_OF_TRACK);
    if eot_positions.len() < actual_tracks {
        corruptions.push(CorruptionInfo {
            corruption_type: CorruptionType::MissingEndOfTrack,
            severity: CorruptionSeverity::Minor,
            location: None,
            description: format!(
                "Found {} end-of-track markers for {} tracks",
                eot_positions.len(),
                actual_tracks
            ),
            repairable: true,
        });
        suggested_strategies.push(RepairStrategy::AddEndOfTrack);
    }

    // Check for garbage at end
    if let Some(&last_eot) = eot_positions.last() {
        let expected_end = last_eot + 3;
        if expected_end < data.len() {
            let garbage_size = data.len() - expected_end;
            corruptions.push(CorruptionInfo {
                corruption_type: CorruptionType::GarbageAtEnd,
                severity: CorruptionSeverity::Minor,
                location: Some(expected_end as u64),
                description: format!("{} bytes of garbage after last track", garbage_size),
                repairable: true,
            });
            suggested_strategies.push(RepairStrategy::TrimGarbage);
        }
    }

    // Check track lengths
    for (i, &pos) in track_positions.iter().enumerate() {
        let track_length = read_u32_be(&data, pos + 4);
        let track_data_start = pos + 8;
        let track_data_end = track_data_start + track_length as usize;

        if track_data_end > data.len() {
            corruptions.push(CorruptionInfo {
                corruption_type: CorruptionType::InvalidTrackLength,
                severity: CorruptionSeverity::Moderate,
                location: Some((pos + 4) as u64),
                description: format!(
                    "Track {} length {} extends beyond file end",
                    i, track_length
                ),
                repairable: true,
            });
            suggested_strategies.push(RepairStrategy::FixTrackLength);
        }
    }

    let is_valid = corruptions.is_empty();
    let repairable = corruptions.iter().all(|c| c.repairable);

    // Calculate confidence based on corruption count and severity
    let confidence = if is_valid {
        1.0
    } else {
        let severity_weight: f64 = corruptions
            .iter()
            .map(|c| match c.severity {
                CorruptionSeverity::Minor => 0.1,
                CorruptionSeverity::Moderate => 0.25,
                CorruptionSeverity::Severe => 0.5,
                CorruptionSeverity::Critical => 0.75,
            })
            .sum();
        (1.0 - severity_weight).max(0.0)
    };

    if suggested_strategies.is_empty() && !is_valid {
        suggested_strategies.push(RepairStrategy::Auto);
    }

    Ok(DetectionResult {
        file_path: file_path.to_string(),
        file_size,
        is_valid,
        corruptions,
        repairable,
        suggested_strategies,
        confidence,
    })
}

/// Repair a MIDI file
pub async fn repair_file_impl(
    file_path: &str,
    options: &RepairOptions,
    state: &RepairState,
) -> Result<RepairResult, RepairError> {
    // Check for cancellation
    if *state.cancel_requested.read().await {
        return Err(RepairError::Cancelled);
    }

    let path = Path::new(file_path);

    if !path.exists() {
        return Err(RepairError::FileNotFound(file_path.to_string()));
    }

    let data = read_file_bytes(path, options.max_file_size)?;
    let original_size = data.len() as u64;

    if !is_midi_file(&data) {
        return Err(RepairError::NotMidiFile(file_path.to_string()));
    }

    // Detect corruptions first
    let detection = detect_corruption_impl(file_path, options.max_file_size).await?;

    if detection.is_valid {
        return Ok(RepairResult {
            file_path: file_path.to_string(),
            success: true,
            strategy_used: options.strategy,
            corruptions_fixed: vec![],
            corruptions_remaining: vec![],
            backup_path: None,
            original_size,
            repaired_size: original_size,
            error: None,
        });
    }

    if !detection.repairable {
        return Ok(RepairResult {
            file_path: file_path.to_string(),
            success: false,
            strategy_used: options.strategy,
            corruptions_fixed: vec![],
            corruptions_remaining: detection
                .corruptions
                .iter()
                .map(|c| c.corruption_type)
                .collect(),
            backup_path: None,
            original_size,
            repaired_size: original_size,
            error: Some("File contains unrepairable corruption".to_string()),
        });
    }

    // Check for critical corruption if configured
    if options.stop_on_critical {
        let has_critical =
            detection.corruptions.iter().any(|c| c.severity == CorruptionSeverity::Critical);

        if has_critical {
            return Err(RepairError::CriticalCorruption(
                "File has critical corruption".to_string(),
            ));
        }
    }

    // Create backup if requested
    let backup_path = if options.create_backup && !options.dry_run {
        let backup = format!("{}{}", file_path, options.backup_suffix);
        fs::copy(path, &backup).map_err(|e| RepairError::BackupFailed(e.to_string()))?;
        Some(backup)
    } else {
        None
    };

    // Apply repairs
    let mut repaired_data = data.clone();
    let mut corruptions_fixed = Vec::new();

    let strategy = if options.strategy == RepairStrategy::Auto {
        // Choose best strategy based on corruptions
        if detection
            .corruptions
            .iter()
            .any(|c| c.corruption_type == CorruptionType::GarbageAtEnd)
        {
            RepairStrategy::TrimGarbage
        } else if detection
            .corruptions
            .iter()
            .any(|c| c.corruption_type == CorruptionType::MissingEndOfTrack)
        {
            RepairStrategy::AddEndOfTrack
        } else {
            RepairStrategy::Conservative
        }
    } else {
        options.strategy
    };

    match strategy {
        RepairStrategy::TrimGarbage | RepairStrategy::Conservative | RepairStrategy::Auto => {
            // Find last end-of-track and trim
            let eot_positions = find_all_patterns(&repaired_data, END_OF_TRACK);
            if let Some(&last_eot) = eot_positions.last() {
                let new_len = last_eot + 3;
                if new_len < repaired_data.len() {
                    repaired_data.truncate(new_len);
                    corruptions_fixed.push(CorruptionType::GarbageAtEnd);
                }
            }
        },
        RepairStrategy::AddEndOfTrack => {
            // Find tracks without end-of-track and add
            let track_positions = find_all_patterns(&repaired_data, TRACK_HEADER);

            for &pos in track_positions.iter().rev() {
                let track_length = read_u32_be(&repaired_data, pos + 4);
                let track_end = pos + 8 + track_length as usize;

                // Check if this track has EOT
                let track_data = &repaired_data[pos + 8..track_end.min(repaired_data.len())];
                if find_pattern(track_data, END_OF_TRACK).is_none() {
                    // Add EOT at end of track
                    let insert_pos = track_end.min(repaired_data.len());
                    repaired_data.splice(insert_pos..insert_pos, END_OF_TRACK.iter().cloned());

                    // Update track length
                    let new_length = track_length + 3;
                    let length_bytes = write_u32_be(new_length);
                    repaired_data[pos + 4..pos + 8].copy_from_slice(&length_bytes);

                    corruptions_fixed.push(CorruptionType::MissingEndOfTrack);
                }
            }
        },
        RepairStrategy::FixTrackLength => {
            // Calculate and fix track lengths
            let track_positions = find_all_patterns(&repaired_data, TRACK_HEADER);

            for i in 0..track_positions.len() {
                let pos = track_positions[i];
                let next_pos = if i + 1 < track_positions.len() {
                    track_positions[i + 1]
                } else {
                    repaired_data.len()
                };

                let actual_length = (next_pos - pos - 8) as u32;
                let declared_length = read_u32_be(&repaired_data, pos + 4);

                if actual_length != declared_length {
                    let length_bytes = write_u32_be(actual_length);
                    repaired_data[pos + 4..pos + 8].copy_from_slice(&length_bytes);
                    corruptions_fixed.push(CorruptionType::InvalidTrackLength);
                }
            }
        },
        RepairStrategy::FixTrackCount => {
            // Update header track count
            let track_positions = find_all_patterns(&repaired_data, TRACK_HEADER);
            let actual_count = track_positions.len() as u16;

            repaired_data[10] = (actual_count >> 8) as u8;
            repaired_data[11] = actual_count as u8;
            corruptions_fixed.push(CorruptionType::TrackCountMismatch);
        },
        RepairStrategy::Aggressive | RepairStrategy::Rebuild => {
            // Apply all applicable repairs
            // Trim garbage
            let eot_positions = find_all_patterns(&repaired_data, END_OF_TRACK);
            if let Some(&last_eot) = eot_positions.last() {
                let new_len = last_eot + 3;
                if new_len < repaired_data.len() {
                    repaired_data.truncate(new_len);
                    corruptions_fixed.push(CorruptionType::GarbageAtEnd);
                }
            }

            // Fix track count
            let track_positions = find_all_patterns(&repaired_data, TRACK_HEADER);
            let actual_count = track_positions.len() as u16;
            repaired_data[10] = (actual_count >> 8) as u8;
            repaired_data[11] = actual_count as u8;
            corruptions_fixed.push(CorruptionType::TrackCountMismatch);
        },
        _ => {},
    }

    // Write repaired file if not dry run
    if !options.dry_run {
        let mut file = fs::File::create(path)?;
        file.write_all(&repaired_data)?;
    }

    let repaired_size = repaired_data.len() as u64;

    // Update statistics
    {
        let mut stats = state.statistics.write().await;
        stats.files_scanned += 1;
        if !corruptions_fixed.is_empty() {
            stats.files_repaired += 1;
        }
        stats.total_bytes_processed += original_size;
        stats.total_bytes_saved += (original_size as i64) - (repaired_size as i64);

        for corruption in &corruptions_fixed {
            *stats.corruptions_by_type.entry(*corruption).or_insert(0) += 1;
        }
        *stats.strategies_used.entry(strategy).or_insert(0) += 1;
    }

    // Determine remaining corruptions
    let fixed_set: std::collections::HashSet<_> = corruptions_fixed.iter().collect();
    let corruptions_remaining: Vec<_> = detection
        .corruptions
        .iter()
        .filter(|c| !fixed_set.contains(&c.corruption_type))
        .map(|c| c.corruption_type)
        .collect();

    Ok(RepairResult {
        file_path: file_path.to_string(),
        success: corruptions_remaining.is_empty(),
        strategy_used: strategy,
        corruptions_fixed,
        corruptions_remaining,
        backup_path,
        original_size,
        repaired_size,
        error: None,
    })
}

/// Batch repair multiple files
pub async fn batch_repair_impl(
    file_paths: Vec<String>,
    options: &RepairOptions,
    state: &RepairState,
) -> Result<BatchRepairResult, RepairError> {
    let total_files = file_paths.len();
    let start_time = std::time::Instant::now();

    // Initialize progress
    {
        let mut progress = state.current_batch.write().await;
        *progress = Some(BatchProgress {
            total_files,
            processed: 0,
            successful: 0,
            failed: 0,
            skipped: 0,
            current_file: None,
            elapsed_ms: 0,
            estimated_remaining_ms: None,
        });
    }

    let mut results = Vec::with_capacity(total_files);
    let mut successful = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for (i, file_path) in file_paths.iter().enumerate() {
        // Check for cancellation
        if *state.cancel_requested.read().await {
            return Err(RepairError::Cancelled);
        }

        // Update progress
        {
            let mut progress = state.current_batch.write().await;
            if let Some(ref mut p) = *progress {
                p.current_file = Some(file_path.clone());
                p.processed = i;
                p.elapsed_ms = start_time.elapsed().as_millis() as u64;

                if i > 0 {
                    let avg_time_per_file = p.elapsed_ms as f64 / i as f64;
                    let remaining = (total_files - i) as f64 * avg_time_per_file;
                    p.estimated_remaining_ms = Some(remaining as u64);
                }
            }
        }

        match repair_file_impl(file_path, options, state).await {
            Ok(result) => {
                if result.success {
                    successful += 1;
                } else {
                    failed += 1;
                }
                results.push(result);
            },
            Err(RepairError::NotMidiFile(_)) | Err(RepairError::FileNotFound(_)) => {
                skipped += 1;
                results.push(RepairResult {
                    file_path: file_path.clone(),
                    success: false,
                    strategy_used: options.strategy,
                    corruptions_fixed: vec![],
                    corruptions_remaining: vec![],
                    backup_path: None,
                    original_size: 0,
                    repaired_size: 0,
                    error: Some("Skipped: Not a valid MIDI file".to_string()),
                });
            },
            Err(e) => {
                failed += 1;
                results.push(RepairResult {
                    file_path: file_path.clone(),
                    success: false,
                    strategy_used: options.strategy,
                    corruptions_fixed: vec![],
                    corruptions_remaining: vec![],
                    backup_path: None,
                    original_size: 0,
                    repaired_size: 0,
                    error: Some(e.to_string()),
                });
            },
        }

        // Update final progress
        {
            let mut progress = state.current_batch.write().await;
            if let Some(ref mut p) = *progress {
                p.processed = i + 1;
                p.successful = successful;
                p.failed = failed;
                p.skipped = skipped;
            }
        }
    }

    let elapsed_ms = start_time.elapsed().as_millis() as u64;

    // Clear progress
    {
        let mut progress = state.current_batch.write().await;
        *progress = None;
    }

    Ok(BatchRepairResult { total_files, successful, failed, skipped, results, elapsed_ms })
}

/// Get repair statistics
pub async fn get_statistics_impl(state: &RepairState) -> RepairStatistics {
    state.statistics.read().await.clone()
}

/// Reset repair statistics
pub async fn reset_statistics_impl(state: &RepairState) {
    let mut stats = state.statistics.write().await;
    *stats = RepairStatistics::default();
}

/// Get current batch progress
pub async fn get_batch_progress_impl(state: &RepairState) -> Option<BatchProgress> {
    state.current_batch.read().await.clone()
}

/// Cancel current batch operation
pub async fn cancel_batch_impl(state: &RepairState) {
    let mut cancel = state.cancel_requested.write().await;
    *cancel = true;
}

/// Clear cancel flag
pub async fn clear_cancel_impl(state: &RepairState) {
    let mut cancel = state.cancel_requested.write().await;
    *cancel = false;
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

#[tauri::command]
pub async fn detect_midi_corruption(
    file_path: String,
    max_file_size: Option<u64>,
) -> Result<DetectionResult, String> {
    let max_size = max_file_size.unwrap_or(100 * 1024 * 1024);
    detect_corruption_impl(&file_path, max_size).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn repair_midi_file(
    state: State<'_, RepairState>,
    file_path: String,
    options: Option<RepairOptions>,
) -> Result<RepairResult, String> {
    let opts = options.unwrap_or_default();
    repair_file_impl(&file_path, &opts, &state).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn batch_repair_midi_files(
    state: State<'_, RepairState>,
    file_paths: Vec<String>,
    options: Option<RepairOptions>,
) -> Result<BatchRepairResult, String> {
    // Clear any previous cancel flag
    clear_cancel_impl(&state).await;

    let opts = options.unwrap_or_default();
    batch_repair_impl(file_paths, &opts, &state).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_repair_statistics(
    state: State<'_, RepairState>,
) -> Result<RepairStatistics, String> {
    Ok(get_statistics_impl(&state).await)
}

#[tauri::command]
pub async fn reset_repair_statistics(state: State<'_, RepairState>) -> Result<(), String> {
    reset_statistics_impl(&state).await;
    Ok(())
}

#[tauri::command]
pub async fn get_repair_progress(
    state: State<'_, RepairState>,
) -> Result<Option<BatchProgress>, String> {
    Ok(get_batch_progress_impl(&state).await)
}

#[tauri::command]
pub async fn cancel_repair_batch(state: State<'_, RepairState>) -> Result<(), String> {
    cancel_batch_impl(&state).await;
    Ok(())
}

#[tauri::command]
pub fn get_repair_strategies() -> Vec<RepairStrategyInfo> {
    vec![
        RepairStrategy::Auto,
        RepairStrategy::Conservative,
        RepairStrategy::Aggressive,
        RepairStrategy::AddEndOfTrack,
        RepairStrategy::TrimGarbage,
        RepairStrategy::FixTrackLength,
        RepairStrategy::FixTrackCount,
        RepairStrategy::RemoveInvalidEvents,
        RepairStrategy::ClampValues,
        RepairStrategy::Rebuild,
    ]
    .into_iter()
    .map(|s| RepairStrategyInfo {
        id: s,
        name: format!("{:?}", s),
        description: s.description().to_string(),
    })
    .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairStrategyInfo {
    pub id: RepairStrategy,
    pub name: String,
    pub description: String,
}

#[tauri::command]
pub fn get_corruption_types() -> Vec<CorruptionTypeInfo> {
    vec![
        CorruptionType::MissingEndOfTrack,
        CorruptionType::InvalidHeader,
        CorruptionType::TruncatedFile,
        CorruptionType::InvalidTrackLength,
        CorruptionType::GarbageAtEnd,
        CorruptionType::InvalidDeltaTime,
        CorruptionType::InvalidEventType,
        CorruptionType::InvalidRunningStatus,
        CorruptionType::InvalidMetaEvent,
        CorruptionType::InvalidSysexEvent,
        CorruptionType::MissingTrackHeader,
        CorruptionType::TrackCountMismatch,
        CorruptionType::NestedSysex,
        CorruptionType::InvalidChannel,
        CorruptionType::InvalidNote,
        CorruptionType::InvalidVelocity,
        CorruptionType::FileTooLarge,
        CorruptionType::Unknown,
    ]
    .into_iter()
    .map(|t| CorruptionTypeInfo {
        id: t,
        name: format!("{:?}", t),
        description: t.description().to_string(),
        severity: t.severity(),
    })
    .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorruptionTypeInfo {
    pub id: CorruptionType,
    pub name: String,
    pub description: String,
    pub severity: CorruptionSeverity,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corruption_type_description() {
        let ct = CorruptionType::MissingEndOfTrack;
        assert!(!ct.description().is_empty());
    }

    #[test]
    fn test_corruption_type_severity() {
        assert_eq!(
            CorruptionType::GarbageAtEnd.severity(),
            CorruptionSeverity::Minor
        );
        assert_eq!(
            CorruptionType::InvalidHeader.severity(),
            CorruptionSeverity::Severe
        );
    }

    #[test]
    fn test_repair_strategy_description() {
        let rs = RepairStrategy::Auto;
        assert!(!rs.description().is_empty());
    }

    #[test]
    fn test_repair_options_default() {
        let opts = RepairOptions::default();
        assert!(opts.create_backup);
        assert!(!opts.dry_run);
        assert_eq!(opts.strategy, RepairStrategy::Auto);
    }

    #[test]
    fn test_repair_state_default() {
        let state = RepairState::default();
        assert!(state.cancel_requested.try_read().is_ok());
    }

    #[test]
    fn testis_midi_file() {
        let valid_header = b"MThd\x00\x00\x00\x06\x00\x01\x00\x01\x01\xe0";
        assert!(is_midi_file(valid_header));

        let invalid = b"RIFF....WAVE";
        assert!(!is_midi_file(invalid));
    }

    #[test]
    fn test_read_u16_be() {
        let data = [0x01, 0x00, 0x02, 0x58];
        assert_eq!(_read_u16_be(&data, 0), 256);
        assert_eq!(_read_u16_be(&data, 2), 600);
    }

    #[test]
    fn testread_u32_be() {
        let data = [0x00, 0x00, 0x01, 0x00];
        assert_eq!(read_u32_be(&data, 0), 256);
    }

    #[test]
    fn testwrite_u32_be() {
        let bytes = write_u32_be(256);
        assert_eq!(bytes, [0x00, 0x00, 0x01, 0x00]);
    }

    #[test]
    fn testfind_pattern() {
        let data = b"MThd\x00\x00\x00\x06MTrk";
        assert_eq!(find_pattern(data, b"MThd"), Some(0));
        assert_eq!(find_pattern(data, b"MTrk"), Some(8));
        assert_eq!(find_pattern(data, b"NOTF"), None);
    }

    #[test]
    fn testfind_all_patterns() {
        let data = b"MTrkABCMTrkDEFMTrk";
        let positions = find_all_patterns(data, b"MTrk");
        assert_eq!(positions, vec![0, 7, 14]);
    }

    #[test]
    fn test_detection_result_serialization() {
        let result = DetectionResult {
            file_path: "/test.mid".to_string(),
            file_size: 1024,
            is_valid: false,
            corruptions: vec![CorruptionInfo {
                corruption_type: CorruptionType::MissingEndOfTrack,
                severity: CorruptionSeverity::Minor,
                location: Some(100),
                description: "Missing EOT".to_string(),
                repairable: true,
            }],
            repairable: true,
            suggested_strategies: vec![RepairStrategy::AddEndOfTrack],
            confidence: 0.9,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: DetectionResult = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.file_path, "/test.mid");
        assert!(!parsed.is_valid);
    }

    #[test]
    fn test_repair_result_serialization() {
        let result = RepairResult {
            file_path: "/test.mid".to_string(),
            success: true,
            strategy_used: RepairStrategy::Auto,
            corruptions_fixed: vec![CorruptionType::GarbageAtEnd],
            corruptions_remaining: vec![],
            backup_path: Some("/test.mid.bak".to_string()),
            original_size: 1024,
            repaired_size: 1000,
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: RepairResult = serde_json::from_str(&json).unwrap();

        assert!(parsed.success);
        assert_eq!(parsed.corruptions_fixed.len(), 1);
    }

    #[test]
    fn test_batch_progress_serialization() {
        let progress = BatchProgress {
            total_files: 100,
            processed: 50,
            successful: 45,
            failed: 3,
            skipped: 2,
            current_file: Some("/current.mid".to_string()),
            elapsed_ms: 5000,
            estimated_remaining_ms: Some(5000),
        };

        let json = serde_json::to_string(&progress).unwrap();
        let parsed: BatchProgress = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.total_files, 100);
        assert_eq!(parsed.processed, 50);
    }

    #[test]
    fn test_get_repair_strategies() {
        let strategies = get_repair_strategies();
        assert!(!strategies.is_empty());

        let auto = strategies.iter().find(|s| s.id == RepairStrategy::Auto);
        assert!(auto.is_some());
    }

    #[test]
    fn test_get_corruption_types() {
        let types = get_corruption_types();
        assert!(!types.is_empty());

        let eot = types.iter().find(|t| t.id == CorruptionType::MissingEndOfTrack);
        assert!(eot.is_some());
    }

    #[tokio::test]
    async fn test_statistics_reset() {
        let state = RepairState::default();

        // Add some data
        {
            let mut stats = state.statistics.write().await;
            stats.files_scanned = 100;
            stats.files_repaired = 50;
        }

        // Reset
        reset_statistics_impl(&state).await;

        // Verify
        let stats = get_statistics_impl(&state).await;
        assert_eq!(stats.files_scanned, 0);
        assert_eq!(stats.files_repaired, 0);
    }

    #[tokio::test]
    async fn test_cancel_flag() {
        let state = RepairState::default();

        // Initially not cancelled
        assert!(!*state.cancel_requested.read().await);

        // Cancel
        cancel_batch_impl(&state).await;
        assert!(*state.cancel_requested.read().await);

        // Clear
        clear_cancel_impl(&state).await;
        assert!(!*state.cancel_requested.read().await);
    }

    #[tokio::test]
    async fn test_batch_progress() {
        let state = RepairState::default();

        // Initially no progress
        assert!(get_batch_progress_impl(&state).await.is_none());

        // Set progress
        {
            let mut progress = state.current_batch.write().await;
            *progress = Some(BatchProgress {
                total_files: 10,
                processed: 5,
                successful: 4,
                failed: 1,
                skipped: 0,
                current_file: Some("/test.mid".to_string()),
                elapsed_ms: 1000,
                estimated_remaining_ms: Some(1000),
            });
        }

        // Get progress
        let progress = get_batch_progress_impl(&state).await;
        assert!(progress.is_some());
        assert_eq!(progress.unwrap().processed, 5);
    }

    #[test]
    fn test_repair_error_display() {
        let err = RepairError::FileNotFound("/test.mid".to_string());
        assert!(err.to_string().contains("test.mid"));

        let err = RepairError::FileTooLarge(200, 100);
        assert!(err.to_string().contains("200"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(CorruptionSeverity::Minor < CorruptionSeverity::Moderate);
        assert!(CorruptionSeverity::Moderate < CorruptionSeverity::Severe);
        assert!(CorruptionSeverity::Severe < CorruptionSeverity::Critical);
    }
}
