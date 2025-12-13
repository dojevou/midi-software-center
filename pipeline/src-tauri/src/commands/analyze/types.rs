//! Types for musical analysis operations

use serde::{Deserialize, Serialize};

/// Progress event for real-time UI updates
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
    pub rate: f64,
    pub eta_seconds: f64,
}

/// Summary of analysis operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub total_files: usize,
    pub analyzed: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
    pub duration_secs: f64,
    pub rate: f64,
}

impl AnalysisSummary {
    pub fn empty() -> Self {
        Self {
            total_files: 0,
            analyzed: 0,
            skipped: 0,
            errors: vec![],
            duration_secs: 0.0,
            rate: 0.0,
        }
    }
}

/// File record from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FileRecord {
    pub id: i64,
    pub filepath: String,
    pub filename: String,
}

/// Analyzed file data ready for database insertion
#[derive(Debug, Clone)]
pub struct AnalyzedFile {
    pub file_id: i64,
    pub tempo_bpm: Option<f64>,
    pub bpm_confidence: Option<f64>,
    pub has_tempo_variation: bool,
    pub key_signature: Option<String>,
    pub key_confidence: Option<f64>,
    pub scale_type: Option<String>,
    pub time_signature_num: Option<i16>,
    pub time_signature_den: Option<i16>,
    pub duration_seconds: Option<f64>,
    pub duration_ticks: Option<i32>,
    pub note_count: i32,
    pub unique_pitches: Option<i32>,
    pub pitch_range_low: Option<i16>,
    pub pitch_range_high: Option<i16>,
    pub pitch_range_semitones: Option<i16>,
    pub avg_velocity: Option<f64>,
    pub velocity_range_low: Option<i16>,
    pub velocity_range_high: Option<i16>,
    pub note_density: Option<f64>,
    pub polyphony_max: Option<i16>,
    pub polyphony_avg: Option<f64>,
    pub is_monophonic: bool,
    pub is_polyphonic: bool,
    pub is_percussive: bool,
    pub has_chords: bool,
    pub chord_progression: Option<Vec<String>>,
    pub chord_types: Option<Vec<String>>,
    pub has_seventh_chords: bool,
    pub has_extended_chords: bool,
    pub chord_change_rate: Option<f32>,
    pub chord_complexity_score: Option<f32>,
    pub has_melody: bool,
    pub melodic_range: Option<i16>,
    pub tempo_changes: Option<String>,
    pub key_changes: Option<String>,
    pub time_signature_changes: Option<String>,
    pub controller_data: Option<String>,
    pub articulation_data: Option<String>,
    pub structure_data: Option<String>,
    pub complexity_score: Option<f64>,
    pub instruments: Vec<String>,
    pub track_instruments: Vec<TrackInstrument>,
    pub has_pitch_bend: bool,
    pub has_cc_messages: bool,
}

/// Track-level instrument information
#[derive(Debug, Clone)]
pub struct TrackInstrument {
    pub channel: i16,
    pub program_number: i16,
    pub program_name: String,
    pub instrument_family: String,
    pub instrument_type: String,
    pub note_count: i32,
    pub avg_velocity: Option<f64>,
    pub pitch_range_low: Option<i16>,
    pub pitch_range_high: Option<i16>,
    pub is_primary: bool,
}

/// Note statistics from MIDI analysis
#[derive(Debug, Clone)]
pub struct NoteStats {
    pub note_count: i32,
    pub unique_pitches: Option<i32>,
    pub pitch_range_low: Option<i16>,
    pub pitch_range_high: Option<i16>,
    pub pitch_range_semitones: Option<i16>,
    pub avg_velocity: Option<f64>,
    pub velocity_range_low: Option<i16>,
    pub velocity_range_high: Option<i16>,
    pub note_density: Option<f64>,
    pub polyphony_max: Option<i16>,
    pub polyphony_avg: Option<f64>,
    pub is_monophonic: bool,
    pub is_polyphonic: bool,
    pub is_percussive: bool,
}

impl Default for NoteStats {
    fn default() -> Self {
        Self {
            note_count: 0,
            unique_pitches: None,
            pitch_range_low: None,
            pitch_range_high: None,
            pitch_range_semitones: None,
            avg_velocity: None,
            velocity_range_low: None,
            velocity_range_high: None,
            note_density: None,
            polyphony_max: None,
            polyphony_avg: None,
            is_monophonic: false,
            is_polyphonic: false,
            is_percussive: false,
        }
    }
}
