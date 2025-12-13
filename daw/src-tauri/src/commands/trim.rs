#![allow(dead_code)]
//! MIDI and Audio Trimming Commands
//!
//! This module provides Tauri commands for trimming MIDI and audio files,
//! including removing leading/trailing silence, trimming to specific regions,
//! and batch trimming operations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tauri::{command, State};
use thiserror::Error;
use tokio::sync::RwLock;

// ============================================================================
// CONSTANTS
// ============================================================================

/// MIDI header magic number
const MIDI_HEADER: &[u8] = b"MThd";
/// MIDI track header magic number
const TRACK_HEADER: &[u8] = b"MTrk";
/// End of track meta event
const END_OF_TRACK: &[u8] = &[0xFF, 0x2F, 0x00];
/// Default silence threshold in ticks
const DEFAULT_SILENCE_THRESHOLD_TICKS: u32 = 480; // One quarter note at 480 PPQN
/// Default silence threshold in samples (for audio)
const DEFAULT_SILENCE_THRESHOLD_SAMPLES: u32 = 4410; // 100ms at 44.1kHz
/// Default amplitude threshold for silence detection
const DEFAULT_AMPLITUDE_THRESHOLD: f32 = 0.001;
/// Maximum file size for processing (100MB)
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Error, Debug)]
pub enum TrimError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("File too large: {0} bytes (max {1} bytes)")]
    FileTooLarge(u64, u64),

    #[error("Invalid trim range: start {0} >= end {1}")]
    InvalidRange(u64, u64),

    #[error("No content remaining after trim")]
    NoContentRemaining,

    #[error("Trim cancelled")]
    Cancelled,

    #[error("Batch operation in progress")]
    BatchInProgress,

    #[error("No batch operation in progress")]
    NoBatchInProgress,

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("Write error: {0}")]
    WriteError(String),
}

impl From<TrimError> for String {
    fn from(e: TrimError) -> Self {
        e.to_string()
    }
}

// ============================================================================
// TYPES
// ============================================================================

/// Trim mode specifying what to trim
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrimMode {
    /// Remove leading silence only
    LeadingSilence,
    /// Remove trailing silence only
    TrailingSilence,
    /// Remove both leading and trailing silence
    BothSilence,
    /// Trim to specific tick/sample range
    ToRange,
    /// Trim to specific time range (milliseconds)
    ToTimeRange,
    /// Trim to loop region
    ToLoopRegion,
    /// Trim to selection
    ToSelection,
    /// Normalize and trim (adjust content to start at tick 0)
    Normalize,
}

/// File type being trimmed
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrimFileType {
    Midi,
    Audio,
    Auto,
}

/// Trim options configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimOptions {
    /// Trim mode
    pub mode: TrimMode,
    /// File type (auto-detect if not specified)
    pub file_type: TrimFileType,
    /// Silence threshold in ticks (for MIDI)
    pub silence_threshold_ticks: Option<u32>,
    /// Silence threshold in samples (for audio)
    pub silence_threshold_samples: Option<u32>,
    /// Amplitude threshold for silence detection (0.0 - 1.0)
    pub amplitude_threshold: Option<f32>,
    /// Start position for range trim (ticks or samples)
    pub start_position: Option<u64>,
    /// End position for range trim (ticks or samples)
    pub end_position: Option<u64>,
    /// Start time in milliseconds
    pub start_time_ms: Option<f64>,
    /// End time in milliseconds
    pub end_time_ms: Option<f64>,
    /// Whether to create backup before trimming
    pub create_backup: bool,
    /// Backup suffix
    pub backup_suffix: Option<String>,
    /// Output path (if different from input)
    pub output_path: Option<String>,
    /// Whether to overwrite existing files
    pub overwrite: bool,
    /// Preserve MIDI tempo/time signature events at start
    pub preserve_tempo_events: bool,
    /// Add fade in/out for audio (samples)
    pub fade_samples: Option<u32>,
    /// PPQN for MIDI files (pulses per quarter note)
    pub ppqn: Option<u16>,
}

impl Default for TrimOptions {
    fn default() -> Self {
        Self {
            mode: TrimMode::BothSilence,
            file_type: TrimFileType::Auto,
            silence_threshold_ticks: Some(DEFAULT_SILENCE_THRESHOLD_TICKS),
            silence_threshold_samples: Some(DEFAULT_SILENCE_THRESHOLD_SAMPLES),
            amplitude_threshold: Some(DEFAULT_AMPLITUDE_THRESHOLD),
            start_position: None,
            end_position: None,
            start_time_ms: None,
            end_time_ms: None,
            create_backup: true,
            backup_suffix: Some(".bak".to_string()),
            output_path: None,
            overwrite: false,
            preserve_tempo_events: true,
            fade_samples: None,
            ppqn: Some(480),
        }
    }
}

/// Result of a trim analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimAnalysis {
    /// Original file path
    pub file_path: String,
    /// Detected file type
    pub file_type: TrimFileType,
    /// Original file size in bytes
    pub original_size: u64,
    /// Original duration in ticks (MIDI) or samples (audio)
    pub original_duration: u64,
    /// Original duration in milliseconds
    pub original_duration_ms: f64,
    /// Detected leading silence in ticks/samples
    pub leading_silence: u64,
    /// Detected trailing silence in ticks/samples
    pub trailing_silence: u64,
    /// First event/sample position
    pub content_start: u64,
    /// Last event/sample position
    pub content_end: u64,
    /// Recommended trim start
    pub recommended_start: u64,
    /// Recommended trim end
    pub recommended_end: u64,
    /// Estimated size after trim
    pub estimated_trimmed_size: u64,
    /// Space savings percentage
    pub space_savings_percent: f32,
    /// Number of MIDI events (MIDI only)
    pub event_count: Option<u32>,
    /// Number of tracks (MIDI only)
    pub track_count: Option<u16>,
    /// Sample rate (audio only)
    pub sample_rate: Option<u32>,
    /// Channels (audio only)
    pub channels: Option<u16>,
    /// Whether file needs trimming
    pub needs_trimming: bool,
    /// Warnings about the file
    pub warnings: Vec<String>,
}

/// Result of a trim operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimResult {
    /// Original file path
    pub original_path: String,
    /// Output file path
    pub output_path: String,
    /// Backup file path (if created)
    pub backup_path: Option<String>,
    /// Whether the operation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Original file size
    pub original_size: u64,
    /// Trimmed file size
    pub trimmed_size: u64,
    /// Bytes saved
    pub bytes_saved: i64,
    /// Original duration
    pub original_duration: u64,
    /// Trimmed duration
    pub trimmed_duration: u64,
    /// Ticks/samples removed from start
    pub removed_from_start: u64,
    /// Ticks/samples removed from end
    pub removed_from_end: u64,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Batch trim progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTrimProgress {
    /// Total files to process
    pub total_files: usize,
    /// Files processed so far
    pub processed_files: usize,
    /// Files successfully trimmed
    pub successful: usize,
    /// Files that failed
    pub failed: usize,
    /// Files skipped (no trim needed)
    pub skipped: usize,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Overall progress percentage (0-100)
    pub progress_percent: f32,
    /// Estimated time remaining in seconds
    pub estimated_remaining_secs: Option<f64>,
    /// Total bytes saved so far
    pub total_bytes_saved: i64,
    /// Whether the batch is complete
    pub is_complete: bool,
    /// Whether cancellation was requested
    pub cancel_requested: bool,
}

/// Batch trim result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTrimResult {
    /// Individual results
    pub results: Vec<TrimResult>,
    /// Total files processed
    pub total_processed: usize,
    /// Successful trims
    pub successful: usize,
    /// Failed trims
    pub failed: usize,
    /// Skipped files
    pub skipped: usize,
    /// Total bytes saved
    pub total_bytes_saved: i64,
    /// Total processing time in milliseconds
    pub total_time_ms: u64,
    /// Average time per file in milliseconds
    pub avg_time_per_file_ms: f64,
    /// Was cancelled
    pub was_cancelled: bool,
}

/// Trim statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrimStatistics {
    /// Total files analyzed
    pub total_analyzed: u64,
    /// Total files trimmed
    pub total_trimmed: u64,
    /// Total files skipped
    pub total_skipped: u64,
    /// Total files failed
    pub total_failed: u64,
    /// Total bytes saved
    pub total_bytes_saved: i64,
    /// Total ticks/samples removed
    pub total_content_removed: u64,
    /// Average trim percentage
    pub avg_trim_percent: f32,
    /// Files by type
    pub files_by_type: HashMap<String, u64>,
    /// Total processing time in milliseconds
    pub total_processing_time_ms: u64,
}

/// Shared state for trim operations
#[derive(Debug, Default)]
pub struct TrimState {
    /// Statistics
    pub statistics: Arc<RwLock<TrimStatistics>>,
    /// Current batch progress
    pub current_batch: Arc<RwLock<Option<BatchTrimProgress>>>,
    /// Cancel flag
    pub cancel_requested: Arc<RwLock<bool>>,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Read file bytes with size validation
fn read_file_bytes(path: &Path) -> Result<Vec<u8>, TrimError> {
    let metadata =
        fs::metadata(path).map_err(|_| TrimError::FileNotFound(path.display().to_string()))?;

    let size = metadata.len();
    if size > MAX_FILE_SIZE {
        return Err(TrimError::FileTooLarge(size, MAX_FILE_SIZE));
    }

    fs::read(path).map_err(TrimError::Io)
}

/// Detect file type from extension and content
fn detect_file_type(path: &Path, data: &[u8]) -> TrimFileType {
    // Check extension first
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        match ext_lower.as_str() {
            "mid" | "midi" | "smf" => return TrimFileType::Midi,
            "wav" | "wave" | "aif" | "aiff" | "mp3" | "flac" | "ogg" => return TrimFileType::Audio,
            _ => {},
        }
    }

    // Check magic bytes
    if data.len() >= 4 && &data[0..4] == MIDI_HEADER {
        return TrimFileType::Midi;
    }

    // Check for RIFF (WAV) header
    if data.len() >= 4 && &data[0..4] == b"RIFF" {
        return TrimFileType::Audio;
    }

    // Default to MIDI for this project
    TrimFileType::Midi
}

/// Read variable-length quantity from MIDI data
fn read_vlq(data: &[u8], offset: usize) -> Option<(u32, usize)> {
    let mut value: u32 = 0;
    let mut bytes_read = 0;

    for i in 0..4 {
        if offset + i >= data.len() {
            return None;
        }
        let byte = data[offset + i];
        value = (value << 7) | (byte & 0x7F) as u32;
        bytes_read += 1;

        if byte & 0x80 == 0 {
            break;
        }
    }

    Some((value, bytes_read))
}

/// Write variable-length quantity to buffer
fn write_vlq(value: u32) -> Vec<u8> {
    if value == 0 {
        return vec![0];
    }

    let mut bytes = Vec::new();
    let mut v = value;
    let mut first = true;

    while v > 0 || first {
        let mut byte = (v & 0x7F) as u8;
        v >>= 7;
        if !first {
            byte |= 0x80;
        }
        bytes.push(byte);
        first = false;
    }

    bytes.reverse();
    bytes
}

/// Read 16-bit big-endian value
fn read_u16_be(data: &[u8], offset: usize) -> Option<u16> {
    if offset + 2 > data.len() {
        return None;
    }
    Some(u16::from_be_bytes([data[offset], data[offset + 1]]))
}

/// Read 32-bit big-endian value
fn read_u32_be(data: &[u8], offset: usize) -> Option<u32> {
    if offset + 4 > data.len() {
        return None;
    }
    Some(u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

/// Write 32-bit big-endian value
fn write_u32_be(value: u32) -> [u8; 4] {
    value.to_be_bytes()
}

/// Analyze MIDI file for trimming
fn analyze_midi_file(data: &[u8], options: &TrimOptions) -> Result<TrimAnalysis, TrimError> {
    if data.len() < 14 || &data[0..4] != MIDI_HEADER {
        return Err(TrimError::InvalidFormat(
            "Not a valid MIDI file".to_string(),
        ));
    }

    let header_length = read_u32_be(data, 4)
        .ok_or_else(|| TrimError::InvalidFormat("Invalid header length".to_string()))?;
    let _format = read_u16_be(data, 8)
        .ok_or_else(|| TrimError::InvalidFormat("Invalid format".to_string()))?;
    let num_tracks = read_u16_be(data, 10)
        .ok_or_else(|| TrimError::InvalidFormat("Invalid track count".to_string()))?;
    let ppqn = read_u16_be(data, 12)
        .ok_or_else(|| TrimError::InvalidFormat("Invalid PPQN".to_string()))?;

    let mut offset = 8 + header_length as usize;
    let mut first_event_tick: u64 = u64::MAX;
    let mut last_event_tick: u64 = 0;
    let mut total_events: u32 = 0;
    let mut warnings = Vec::new();

    // Parse each track
    for _track_idx in 0..num_tracks {
        if offset + 8 > data.len() {
            warnings.push("Truncated track header".to_string());
            break;
        }

        if &data[offset..offset + 4] != TRACK_HEADER {
            warnings.push("Invalid track header".to_string());
            break;
        }

        let track_length = read_u32_be(data, offset + 4)
            .ok_or_else(|| TrimError::InvalidFormat("Invalid track length".to_string()))?
            as usize;

        let track_start = offset + 8;
        let track_end = track_start + track_length;

        if track_end > data.len() {
            warnings.push("Track extends beyond file".to_string());
            break;
        }

        // Parse events in this track
        let mut pos = track_start;
        let mut current_tick: u64 = 0;
        let mut running_status: u8 = 0;

        while pos < track_end {
            // Read delta time
            if let Some((delta, vlq_len)) = read_vlq(data, pos) {
                current_tick += delta as u64;
                pos += vlq_len;
            } else {
                break;
            }

            if pos >= track_end {
                break;
            }

            let status = data[pos];

            // Handle event
            if status == 0xFF {
                // Meta event
                if pos + 2 >= track_end {
                    break;
                }
                let meta_type = data[pos + 1];
                if let Some((length, vlq_len)) = read_vlq(data, pos + 2) {
                    pos += 2 + vlq_len + length as usize;

                    // End of track
                    if meta_type == 0x2F {
                        break;
                    }
                } else {
                    break;
                }
            } else if status == 0xF0 || status == 0xF7 {
                // SysEx event
                if let Some((length, vlq_len)) = read_vlq(data, pos + 1) {
                    pos += 1 + vlq_len + length as usize;
                } else {
                    break;
                }
            } else if status & 0x80 != 0 {
                // Regular channel event
                running_status = status;
                pos += 1;

                let event_type = status & 0xF0;
                let data_bytes = match event_type {
                    0xC0 | 0xD0 => 1, // Program change, channel pressure
                    _ => 2,           // Note, control, pitch bend, etc.
                };

                // Track note events for trimming analysis
                if event_type == 0x90 || event_type == 0x80 {
                    total_events += 1;
                    if current_tick < first_event_tick {
                        first_event_tick = current_tick;
                    }
                    if current_tick > last_event_tick {
                        last_event_tick = current_tick;
                    }
                }

                pos += data_bytes;
            } else {
                // Running status
                let event_type = running_status & 0xF0;
                let data_bytes = match event_type {
                    0xC0 | 0xD0 => 0, // Already read one byte
                    _ => 1,           // Need one more byte
                };

                if event_type == 0x90 || event_type == 0x80 {
                    total_events += 1;
                    if current_tick < first_event_tick {
                        first_event_tick = current_tick;
                    }
                    if current_tick > last_event_tick {
                        last_event_tick = current_tick;
                    }
                }

                pos += 1 + data_bytes;
            }
        }

        offset = track_end;
    }

    // Calculate results
    if first_event_tick == u64::MAX {
        first_event_tick = 0;
    }

    let silence_threshold =
        options.silence_threshold_ticks.unwrap_or(DEFAULT_SILENCE_THRESHOLD_TICKS) as u64;

    let leading_silence = first_event_tick;
    let trailing_silence = if last_event_tick > 0 && offset > 0 {
        // Estimate based on typical patterns
        silence_threshold.min(last_event_tick / 10)
    } else {
        0
    };

    let original_duration = last_event_tick.max(first_event_tick);
    let ppqn_val = options.ppqn.unwrap_or(ppqn) as f64;
    let original_duration_ms = (original_duration as f64 / ppqn_val) * 500.0; // Assuming 120 BPM

    let needs_trimming = leading_silence > silence_threshold;
    let estimated_trimmed_size = if needs_trimming {
        (data.len() as f64 * 0.95) as u64 // Rough estimate
    } else {
        data.len() as u64
    };

    let space_savings = if data.len() > 0 {
        ((data.len() as u64 - estimated_trimmed_size) as f32 / data.len() as f32) * 100.0
    } else {
        0.0
    };

    Ok(TrimAnalysis {
        file_path: String::new(),
        file_type: TrimFileType::Midi,
        original_size: data.len() as u64,
        original_duration,
        original_duration_ms,
        leading_silence,
        trailing_silence,
        content_start: first_event_tick,
        content_end: last_event_tick,
        recommended_start: first_event_tick,
        recommended_end: last_event_tick,
        estimated_trimmed_size,
        space_savings_percent: space_savings,
        event_count: Some(total_events),
        track_count: Some(num_tracks),
        sample_rate: None,
        channels: None,
        needs_trimming,
        warnings,
    })
}

/// Trim MIDI file by removing leading silence
fn trim_midi_file(
    data: &[u8],
    options: &TrimOptions,
    analysis: &TrimAnalysis,
) -> Result<Vec<u8>, TrimError> {
    if data.len() < 14 || &data[0..4] != MIDI_HEADER {
        return Err(TrimError::InvalidFormat(
            "Not a valid MIDI file".to_string(),
        ));
    }

    let header_length = read_u32_be(data, 4)
        .ok_or_else(|| TrimError::InvalidFormat("Invalid header length".to_string()))?;
    let num_tracks = read_u16_be(data, 10)
        .ok_or_else(|| TrimError::InvalidFormat("Invalid track count".to_string()))?;

    // Calculate trim amount based on mode
    let trim_start = match options.mode {
        TrimMode::LeadingSilence | TrimMode::BothSilence | TrimMode::Normalize => {
            analysis.leading_silence
        },
        TrimMode::ToRange => options.start_position.unwrap_or(0),
        _ => 0,
    };

    if trim_start == 0 {
        // No trimming needed
        return Ok(data.to_vec());
    }

    // Build new MIDI file
    let mut output = Vec::with_capacity(data.len());

    // Copy header
    output.extend_from_slice(&data[0..8 + header_length as usize]);

    let mut offset = 8 + header_length as usize;

    // Process each track
    for _track_idx in 0..num_tracks {
        if offset + 8 > data.len() {
            break;
        }

        if &data[offset..offset + 4] != TRACK_HEADER {
            break;
        }

        let track_length = read_u32_be(data, offset + 4)
            .ok_or_else(|| TrimError::InvalidFormat("Invalid track length".to_string()))?
            as usize;

        let track_start = offset + 8;
        let track_end = track_start + track_length;

        if track_end > data.len() {
            break;
        }

        // Build new track data
        let mut new_track_data = Vec::with_capacity(track_length);
        let mut pos = track_start;
        let mut current_tick: u64 = 0;
        let mut running_status: u8 = 0;
        let mut first_event_in_track = true;

        while pos < track_end {
            let _event_start = pos;

            // Read delta time
            if let Some((delta, vlq_len)) = read_vlq(data, pos) {
                let new_tick = current_tick + delta as u64;
                pos += vlq_len;

                if pos >= track_end {
                    break;
                }

                let status = data[pos];

                // Calculate event length
                let event_data_start = pos;
                let event_end;

                if status == 0xFF {
                    // Meta event
                    if pos + 2 >= track_end {
                        break;
                    }
                    let meta_type = data[pos + 1];
                    if let Some((length, vlq_len)) = read_vlq(data, pos + 2) {
                        event_end = pos + 2 + vlq_len + length as usize;

                        // Always preserve tempo and time signature events at start
                        let is_tempo_event = meta_type == 0x51 || meta_type == 0x58;
                        let should_preserve = options.preserve_tempo_events && is_tempo_event;

                        if should_preserve || new_tick >= trim_start {
                            // Adjust delta time
                            let adjusted_tick = if new_tick >= trim_start {
                                new_tick - trim_start
                            } else {
                                0
                            };

                            let new_delta = if first_event_in_track {
                                adjusted_tick as u32
                            } else {
                                (adjusted_tick - current_tick.saturating_sub(trim_start)) as u32
                            };

                            new_track_data.extend(write_vlq(new_delta));
                            new_track_data.extend_from_slice(&data[event_data_start..event_end]);
                            first_event_in_track = false;
                        }

                        // End of track
                        if meta_type == 0x2F {
                            break;
                        }
                    } else {
                        break;
                    }
                } else if status == 0xF0 || status == 0xF7 {
                    // SysEx event
                    if let Some((length, vlq_len)) = read_vlq(data, pos + 1) {
                        event_end = pos + 1 + vlq_len + length as usize;

                        if new_tick >= trim_start {
                            let adjusted_delta = (new_tick - trim_start) as u32;
                            let new_delta = if first_event_in_track {
                                adjusted_delta
                            } else {
                                adjusted_delta.saturating_sub(
                                    (current_tick.saturating_sub(trim_start)) as u32,
                                )
                            };

                            new_track_data.extend(write_vlq(new_delta));
                            new_track_data.extend_from_slice(&data[event_data_start..event_end]);
                            first_event_in_track = false;
                        }
                    } else {
                        break;
                    }
                } else if status & 0x80 != 0 {
                    // Regular channel event
                    running_status = status;
                    let event_type = status & 0xF0;
                    let data_bytes = match event_type {
                        0xC0 | 0xD0 => 1,
                        _ => 2,
                    };
                    event_end = pos + 1 + data_bytes;

                    if new_tick >= trim_start {
                        let adjusted_delta = (new_tick - trim_start) as u32;
                        let new_delta = if first_event_in_track {
                            adjusted_delta
                        } else {
                            adjusted_delta
                                .saturating_sub((current_tick.saturating_sub(trim_start)) as u32)
                        };

                        new_track_data.extend(write_vlq(new_delta));
                        new_track_data.extend_from_slice(&data[event_data_start..event_end]);
                        first_event_in_track = false;
                    }
                } else {
                    // Running status
                    let event_type = running_status & 0xF0;
                    let data_bytes = match event_type {
                        0xC0 | 0xD0 => 1,
                        _ => 2,
                    };
                    event_end = pos + data_bytes;

                    if new_tick >= trim_start {
                        let adjusted_delta = (new_tick - trim_start) as u32;
                        let new_delta = if first_event_in_track {
                            adjusted_delta
                        } else {
                            adjusted_delta
                                .saturating_sub((current_tick.saturating_sub(trim_start)) as u32)
                        };

                        new_track_data.extend(write_vlq(new_delta));
                        new_track_data.extend_from_slice(&data[event_data_start..event_end]);
                        first_event_in_track = false;
                    }
                }

                current_tick = new_tick;
                pos = event_end;
            } else {
                break;
            }
        }

        // Ensure end of track marker
        if !new_track_data.ends_with(END_OF_TRACK) {
            new_track_data.extend(write_vlq(0));
            new_track_data.extend_from_slice(END_OF_TRACK);
        }

        // Write track header and data
        output.extend_from_slice(TRACK_HEADER);
        output.extend_from_slice(&write_u32_be(new_track_data.len() as u32));
        output.extend(new_track_data);

        offset = track_end;
    }

    Ok(output)
}

/// Get current timestamp in milliseconds
fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ============================================================================
// IMPLEMENTATION FUNCTIONS
// ============================================================================

/// Analyze a file for potential trimming
pub fn analyze_file_impl(path: &str, options: &TrimOptions) -> Result<TrimAnalysis, TrimError> {
    let file_path = Path::new(path);
    let data = read_file_bytes(file_path)?;

    let file_type = match options.file_type {
        TrimFileType::Auto => detect_file_type(file_path, &data),
        ft => ft,
    };

    let mut analysis = match file_type {
        TrimFileType::Midi => analyze_midi_file(&data, options)?,
        TrimFileType::Audio => {
            // Audio analysis not yet implemented
            return Err(TrimError::UnsupportedFileType(
                "Audio trimming not yet implemented".to_string(),
            ));
        },
        TrimFileType::Auto => {
            // Should not reach here
            return Err(TrimError::InvalidFormat(
                "Could not detect file type".to_string(),
            ));
        },
    };

    analysis.file_path = path.to_string();
    analysis.file_type = file_type;

    Ok(analysis)
}

/// Trim a single file
pub fn trim_file_impl(path: &str, options: &TrimOptions) -> Result<TrimResult, TrimError> {
    let start_time = current_timestamp_ms();
    let file_path = Path::new(path);

    // Read and analyze file
    let data = read_file_bytes(file_path)?;
    let file_type = match options.file_type {
        TrimFileType::Auto => detect_file_type(file_path, &data),
        ft => ft,
    };

    let analysis = match file_type {
        TrimFileType::Midi => analyze_midi_file(&data, options)?,
        _ => {
            return Err(TrimError::UnsupportedFileType(
                "Only MIDI files supported".to_string(),
            ));
        },
    };

    // Skip if no trimming needed
    if !analysis.needs_trimming {
        return Ok(TrimResult {
            original_path: path.to_string(),
            output_path: path.to_string(),
            backup_path: None,
            success: true,
            error: None,
            original_size: data.len() as u64,
            trimmed_size: data.len() as u64,
            bytes_saved: 0,
            original_duration: analysis.original_duration,
            trimmed_duration: analysis.original_duration,
            removed_from_start: 0,
            removed_from_end: 0,
            processing_time_ms: current_timestamp_ms() - start_time,
        });
    }

    // Create backup if requested
    let backup_path = if options.create_backup {
        let suffix = options.backup_suffix.as_deref().unwrap_or(".bak");
        let backup = format!("{}{}", path, suffix);
        fs::copy(file_path, &backup).map_err(TrimError::Io)?;
        Some(backup)
    } else {
        None
    };

    // Perform trim
    let trimmed_data = match file_type {
        TrimFileType::Midi => trim_midi_file(&data, options, &analysis)?,
        _ => data.clone(),
    };

    // Determine output path
    let output_path = options.output_path.clone().unwrap_or_else(|| path.to_string());
    let output_file_path = Path::new(&output_path);

    // Check if we can write
    if output_file_path.exists() && !options.overwrite && output_path != path {
        return Err(TrimError::WriteError(format!(
            "Output file already exists: {}",
            output_path
        )));
    }

    // Write trimmed file
    fs::write(output_file_path, &trimmed_data).map_err(TrimError::Io)?;

    let processing_time = current_timestamp_ms() - start_time;

    Ok(TrimResult {
        original_path: path.to_string(),
        output_path,
        backup_path,
        success: true,
        error: None,
        original_size: data.len() as u64,
        trimmed_size: trimmed_data.len() as u64,
        bytes_saved: data.len() as i64 - trimmed_data.len() as i64,
        original_duration: analysis.original_duration,
        trimmed_duration: analysis.original_duration - analysis.leading_silence,
        removed_from_start: analysis.leading_silence,
        removed_from_end: analysis.trailing_silence,
        processing_time_ms: processing_time,
    })
}

/// Batch trim multiple files
pub async fn batch_trim_impl(
    paths: Vec<String>,
    options: &TrimOptions,
    state: &TrimState,
) -> Result<BatchTrimResult, TrimError> {
    // Check if batch already in progress
    {
        let current = state.current_batch.read().await;
        if current.is_some() {
            return Err(TrimError::BatchInProgress);
        }
    }

    // Clear cancel flag
    {
        let mut cancel = state.cancel_requested.write().await;
        *cancel = false;
    }

    let start_time = current_timestamp_ms();
    let total_files = paths.len();
    let mut results = Vec::with_capacity(total_files);
    let mut successful = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut total_bytes_saved: i64 = 0;
    let mut was_cancelled = false;

    // Initialize progress
    {
        let mut progress = state.current_batch.write().await;
        *progress = Some(BatchTrimProgress {
            total_files,
            processed_files: 0,
            successful: 0,
            failed: 0,
            skipped: 0,
            current_file: None,
            progress_percent: 0.0,
            estimated_remaining_secs: None,
            total_bytes_saved: 0,
            is_complete: false,
            cancel_requested: false,
        });
    }

    for (idx, path) in paths.iter().enumerate() {
        // Check for cancellation
        {
            let cancel = state.cancel_requested.read().await;
            if *cancel {
                was_cancelled = true;
                break;
            }
        }

        // Update progress
        {
            let mut progress = state.current_batch.write().await;
            if let Some(ref mut p) = *progress {
                p.current_file = Some(path.clone());
                p.processed_files = idx;
                p.progress_percent = (idx as f32 / total_files as f32) * 100.0;

                if idx > 0 {
                    let elapsed = (current_timestamp_ms() - start_time) as f64;
                    let per_file = elapsed / idx as f64;
                    let remaining = (total_files - idx) as f64 * per_file;
                    p.estimated_remaining_secs = Some(remaining / 1000.0);
                }
            }
        }

        // Process file
        match trim_file_impl(path, options) {
            Ok(result) => {
                if result.bytes_saved > 0 {
                    successful += 1;
                    total_bytes_saved += result.bytes_saved;
                } else {
                    skipped += 1;
                }
                results.push(result);
            },
            Err(e) => {
                failed += 1;
                results.push(TrimResult {
                    original_path: path.clone(),
                    output_path: path.clone(),
                    backup_path: None,
                    success: false,
                    error: Some(e.to_string()),
                    original_size: 0,
                    trimmed_size: 0,
                    bytes_saved: 0,
                    original_duration: 0,
                    trimmed_duration: 0,
                    removed_from_start: 0,
                    removed_from_end: 0,
                    processing_time_ms: 0,
                });
            },
        }

        // Update running totals
        {
            let mut progress = state.current_batch.write().await;
            if let Some(ref mut p) = *progress {
                p.successful = successful;
                p.failed = failed;
                p.skipped = skipped;
                p.total_bytes_saved = total_bytes_saved;
            }
        }
    }

    let total_time = current_timestamp_ms() - start_time;
    let processed = results.len();

    // Finalize progress
    {
        let mut progress = state.current_batch.write().await;
        if let Some(ref mut p) = *progress {
            p.is_complete = true;
            p.progress_percent = 100.0;
            p.current_file = None;
        }
    }

    // Update statistics
    {
        let mut stats = state.statistics.write().await;
        stats.total_trimmed += successful as u64;
        stats.total_skipped += skipped as u64;
        stats.total_failed += failed as u64;
        stats.total_bytes_saved += total_bytes_saved;
        stats.total_processing_time_ms += total_time;
    }

    // Clear current batch
    {
        let mut progress = state.current_batch.write().await;
        *progress = None;
    }

    Ok(BatchTrimResult {
        results,
        total_processed: processed,
        successful,
        failed,
        skipped,
        total_bytes_saved,
        total_time_ms: total_time,
        avg_time_per_file_ms: if processed > 0 {
            total_time as f64 / processed as f64
        } else {
            0.0
        },
        was_cancelled,
    })
}

/// Get batch progress for trim operations
pub async fn trim_get_batch_progress_impl(state: &TrimState) -> Option<BatchTrimProgress> {
    let progress = state.current_batch.read().await;
    progress.clone()
}

/// Cancel batch trim operation
pub async fn trim_cancel_batch_impl(state: &TrimState) -> Result<(), TrimError> {
    let current = state.current_batch.read().await;
    if current.is_none() {
        return Err(TrimError::NoBatchInProgress);
    }
    drop(current);

    let mut cancel = state.cancel_requested.write().await;
    *cancel = true;

    let mut progress = state.current_batch.write().await;
    if let Some(ref mut p) = *progress {
        p.cancel_requested = true;
    }

    Ok(())
}

/// Get trim statistics
pub async fn trim_get_statistics_impl(state: &TrimState) -> TrimStatistics {
    let stats = state.statistics.read().await;
    stats.clone()
}

/// Reset trim statistics
pub async fn trim_reset_statistics_impl(state: &TrimState) {
    let mut stats = state.statistics.write().await;
    *stats = TrimStatistics::default();
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

/// Analyze a file for trimming potential
#[command]
pub async fn analyze_trim(
    path: String,
    options: Option<TrimOptions>,
) -> Result<TrimAnalysis, String> {
    let opts = options.unwrap_or_default();
    analyze_file_impl(&path, &opts).map_err(|e| e.to_string())
}

/// Trim a single file
#[command]
pub async fn trim_file(path: String, options: Option<TrimOptions>) -> Result<TrimResult, String> {
    let opts = options.unwrap_or_default();
    trim_file_impl(&path, &opts).map_err(|e| e.to_string())
}

/// Batch trim multiple files
#[command]
pub async fn batch_trim_files(
    paths: Vec<String>,
    options: Option<TrimOptions>,
    state: State<'_, TrimState>,
) -> Result<BatchTrimResult, String> {
    let opts = options.unwrap_or_default();
    batch_trim_impl(paths, &opts, &state).await.map_err(|e| e.to_string())
}

/// Get batch trim progress
#[command]
pub async fn get_trim_progress(
    state: State<'_, TrimState>,
) -> Result<Option<BatchTrimProgress>, String> {
    Ok(trim_get_batch_progress_impl(&state).await)
}

/// Cancel batch trim operation
#[command]
pub async fn cancel_trim_batch(state: State<'_, TrimState>) -> Result<(), String> {
    trim_cancel_batch_impl(&state).await.map_err(|e| e.to_string())
}

/// Get trim statistics
#[command]
pub async fn get_trim_statistics(state: State<'_, TrimState>) -> Result<TrimStatistics, String> {
    Ok(trim_get_statistics_impl(&state).await)
}

/// Reset trim statistics
#[command]
pub async fn reset_trim_statistics(state: State<'_, TrimState>) -> Result<(), String> {
    trim_reset_statistics_impl(&state).await;
    Ok(())
}

/// Get available trim modes
#[command]
pub fn get_trim_modes() -> Vec<String> {
    vec![
        "leading_silence".to_string(),
        "trailing_silence".to_string(),
        "both_silence".to_string(),
        "to_range".to_string(),
        "to_time_range".to_string(),
        "to_loop_region".to_string(),
        "to_selection".to_string(),
        "normalize".to_string(),
    ]
}

/// Get default trim options
#[command]
pub fn get_default_trim_options() -> TrimOptions {
    TrimOptions::default()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vlq_roundtrip() {
        for value in [0, 1, 127, 128, 255, 256, 16383, 16384, 2097151] {
            let encoded = write_vlq(value);
            let (decoded, _) = read_vlq(&encoded, 0).unwrap();
            assert_eq!(value, decoded, "VLQ roundtrip failed for {}", value);
        }
    }

    #[test]
    fn test_trim_options_default() {
        let opts = TrimOptions::default();
        assert_eq!(opts.mode, TrimMode::BothSilence);
        assert_eq!(opts.file_type, TrimFileType::Auto);
        assert!(opts.create_backup);
        assert!(opts.preserve_tempo_events);
    }

    #[test]
    fn test_trim_error_display() {
        let err = TrimError::FileNotFound("test.mid".to_string());
        assert!(err.to_string().contains("test.mid"));

        let err = TrimError::InvalidRange(100, 50);
        assert!(err.to_string().contains("100"));
        assert!(err.to_string().contains("50"));
    }

    #[test]
    fn test_detect_file_type() {
        // MIDI header
        let midi_data = [0x4D, 0x54, 0x68, 0x64, 0x00, 0x00, 0x00, 0x06];
        let ft = detect_file_type(Path::new("test.mid"), &midi_data);
        assert_eq!(ft, TrimFileType::Midi);

        // WAV header
        let wav_data = [0x52, 0x49, 0x46, 0x46]; // "RIFF"
        let ft = detect_file_type(Path::new("test.wav"), &wav_data);
        assert_eq!(ft, TrimFileType::Audio);
    }

    #[test]
    fn test_trim_analysis_defaults() {
        let analysis = TrimAnalysis {
            file_path: "test.mid".to_string(),
            file_type: TrimFileType::Midi,
            original_size: 1000,
            original_duration: 1920,
            original_duration_ms: 2000.0,
            leading_silence: 480,
            trailing_silence: 96,
            content_start: 480,
            content_end: 1824,
            recommended_start: 480,
            recommended_end: 1824,
            estimated_trimmed_size: 900,
            space_savings_percent: 10.0,
            event_count: Some(50),
            track_count: Some(2),
            sample_rate: None,
            channels: None,
            needs_trimming: true,
            warnings: vec![],
        };

        assert!(analysis.needs_trimming);
        assert_eq!(analysis.track_count, Some(2));
    }

    #[test]
    fn test_trim_modes() {
        let modes = get_trim_modes();
        assert!(modes.contains(&"leading_silence".to_string()));
        assert!(modes.contains(&"both_silence".to_string()));
        assert!(modes.contains(&"normalize".to_string()));
    }

    #[test]
    fn test_batch_progress_default() {
        let progress = BatchTrimProgress {
            total_files: 100,
            processed_files: 50,
            successful: 45,
            failed: 3,
            skipped: 2,
            current_file: Some("test.mid".to_string()),
            progress_percent: 50.0,
            estimated_remaining_secs: Some(30.0),
            total_bytes_saved: 10000,
            is_complete: false,
            cancel_requested: false,
        };

        assert_eq!(progress.progress_percent, 50.0);
        assert!(!progress.is_complete);
    }

    #[test]
    fn test_trim_result_success() {
        let result = TrimResult {
            original_path: "input.mid".to_string(),
            output_path: "output.mid".to_string(),
            backup_path: Some("input.mid.bak".to_string()),
            success: true,
            error: None,
            original_size: 1000,
            trimmed_size: 800,
            bytes_saved: 200,
            original_duration: 1920,
            trimmed_duration: 1440,
            removed_from_start: 480,
            removed_from_end: 0,
            processing_time_ms: 15,
        };

        assert!(result.success);
        assert_eq!(result.bytes_saved, 200);
    }

    #[test]
    fn test_statistics_default() {
        let stats = TrimStatistics::default();
        assert_eq!(stats.total_trimmed, 0);
        assert_eq!(stats.total_bytes_saved, 0);
    }
}
