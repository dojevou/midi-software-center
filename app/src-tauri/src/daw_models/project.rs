//! Project serialization and file format definitions
//!
//! This module defines the complete project data structure for saving/loading
//! DAW projects to files or database JSONB storage.
//!
//! File format: JSON with .mscproj extension (MIDI Software Center Project)
//! Version: 1.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Current project file format version
pub const PROJECT_VERSION: u32 = 1;

/// File extension for project files
pub const PROJECT_EXTENSION: &str = "mscproj";

// ============================================================================
// CORE PROJECT STRUCTURE
// ============================================================================

/// Complete project state serializable to JSON/JSONB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Schema version for compatibility checking
    pub version: u32,

    /// Project metadata
    pub metadata: ProjectMetadata,

    /// Musical settings
    pub settings: MusicalSettings,

    /// Track definitions
    pub tracks: Vec<Track>,

    /// Mixer state (placeholder for Stream B integration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mixer: Option<MixerState>,

    /// Automation lanes (placeholder for Stream D integration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automation: Option<HashMap<u32, Vec<AutomationLane>>>,

    /// Routing configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<RoutingConfig>,

    /// Transport state
    pub transport: TransportState,

    /// Markers and regions
    #[serde(default)]
    pub markers: Vec<Marker>,

    /// User-defined tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Custom user data
    #[serde(default)]
    pub user_data: HashMap<String, serde_json::Value>,
}

// ============================================================================
// PROJECT METADATA
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Project name
    pub name: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Creation timestamp (ISO 8601)
    pub created_at: String,

    /// Last modification timestamp (ISO 8601)
    pub updated_at: String,

    /// Author name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Application version that created this file
    pub app_version: String,
}

// ============================================================================
// MUSICAL SETTINGS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicalSettings {
    /// Tempo in BPM
    pub bpm: f64,

    /// Time signature numerator (e.g., 4 in 4/4)
    pub time_signature_numerator: u32,

    /// Time signature denominator (e.g., 4 in 4/4)
    pub time_signature_denominator: u32,

    /// Key signature (e.g., "C", "G", "Am", "F#m")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_signature: Option<String>,

    /// Pulses per quarter note (MIDI resolution)
    #[serde(default = "default_ppqn")]
    pub ppqn: u32,

    /// Sample rate for audio rendering
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    /// Bit depth for audio rendering
    #[serde(default = "default_bit_depth")]
    pub bit_depth: u16,
}

fn default_ppqn() -> u32 {
    480
}

fn default_sample_rate() -> u32 {
    44100
}

fn default_bit_depth() -> u16 {
    16
}

// ============================================================================
// TRACK DEFINITIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Unique track ID (within this project)
    pub id: u32,

    /// Track name
    pub name: String,

    /// Track number/position in the project
    pub track_number: u32,

    /// MIDI channel (0-15)
    pub channel: u8,

    /// MIDI program/instrument number (0-127)
    pub program_number: u8,

    /// Bank select MSB (0-127)
    #[serde(default)]
    pub bank_msb: u8,

    /// Bank select LSB (0-127)
    #[serde(default)]
    pub bank_lsb: u8,

    /// Track mute state
    #[serde(default)]
    pub muted: bool,

    /// Track solo state
    #[serde(default)]
    pub solo: bool,

    /// Record arm state
    #[serde(default)]
    pub armed: bool,

    /// Volume (0.0-2.0, 1.0 = unity gain)
    #[serde(default = "default_volume")]
    pub volume: f64,

    /// Pan (-1.0 = left, 0.0 = center, 1.0 = right)
    #[serde(default)]
    pub pan: f64,

    /// Track color (hex RGB, e.g., "#3B82F6")
    #[serde(default = "default_track_color")]
    pub color: String,

    /// Track height in pixels
    #[serde(default = "default_track_height")]
    pub height: u32,

    /// Whether track is collapsed in UI
    #[serde(default)]
    pub collapsed: bool,

    /// Reference to source file ID in database
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file_id: Option<i64>,

    /// MIDI clips/regions on this track
    #[serde(default)]
    pub clips: Vec<Clip>,

    /// Track notes (if not using clips)
    #[serde(default)]
    pub notes: Vec<Note>,
}

fn default_volume() -> f64 {
    1.0
}

fn default_track_color() -> String {
    "#3B82F6".to_string()
}

fn default_track_height() -> u32 {
    100
}

// ============================================================================
// CLIPS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    /// Unique clip ID (within this project)
    pub id: u32,

    /// Clip name
    pub name: String,

    /// Clip color (hex RGB)
    #[serde(default = "default_clip_color")]
    pub color: String,

    /// Start position in ticks
    pub start_tick: i64,

    /// Duration in ticks
    pub duration_ticks: i64,

    /// Reference to source file ID in database
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file_id: Option<i64>,

    /// Whether clip is muted
    #[serde(default)]
    pub muted: bool,

    /// Clip gain in dB
    #[serde(default)]
    pub gain_db: f64,

    /// Start offset within source file (ticks)
    #[serde(default)]
    pub source_start_tick: i64,

    /// End offset within source file (ticks, None = use file length)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_end_tick: Option<i64>,

    /// MIDI notes in this clip (if not referencing a file)
    #[serde(default)]
    pub notes: Vec<Note>,
}

fn default_clip_color() -> String {
    "#3B82F6".to_string()
}

// ============================================================================
// NOTES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// MIDI pitch (0-127)
    pub pitch: u8,

    /// Velocity (1-127, 0 = note off)
    pub velocity: u8,

    /// Start position in ticks
    pub start_tick: i64,

    /// Duration in ticks
    pub duration_ticks: i64,

    /// MIDI channel override (None = use track channel)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<u8>,
}

// ============================================================================
// MIXER STATE (Placeholder for Stream B)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerState {
    /// Master volume
    pub master_gain_db: f32,

    /// Master pan
    pub master_pan: f32,

    /// Track mixer settings (track_id -> settings)
    pub tracks: HashMap<u32, TrackMixerSettings>,

    /// Bus configurations
    #[serde(default)]
    pub buses: Vec<Bus>,

    /// Send configurations
    #[serde(default)]
    pub sends: Vec<Send>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackMixerSettings {
    /// Track gain in dB
    pub gain_db: f32,

    /// Track pan (-1.0 to 1.0)
    pub pan: f32,

    /// Effect chain
    #[serde(default)]
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bus {
    pub id: u32,
    pub name: String,
    pub gain_db: f32,
    pub pan: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Send {
    pub from_track_id: u32,
    pub to_bus_id: u32,
    pub level: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub id: u32,
    pub effect_type: String,
    pub enabled: bool,
    pub params: HashMap<String, f32>,
}

// ============================================================================
// AUTOMATION (Placeholder for Stream D)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationLane {
    /// Parameter being automated
    pub parameter: String,

    /// Automation mode (off, read, write, latch, touch)
    pub mode: String,

    /// Automation points
    pub points: Vec<AutomationPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationPoint {
    /// Time in beats
    pub time: f64,

    /// Normalized value (0.0-1.0)
    pub value: f32,

    /// Curve type (linear, bezier, step)
    pub curve: String,
}

// ============================================================================
// ROUTING
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// Track routing (track_id -> destination)
    pub track_routing: HashMap<u32, u32>,

    /// Sidechain routing
    #[serde(default)]
    pub sidechains: Vec<SidechainRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidechainRoute {
    pub from_track_id: u32,
    pub to_track_id: u32,
    pub to_effect_id: u32,
}

// ============================================================================
// TRANSPORT STATE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportState {
    /// Current playhead position in ticks
    #[serde(default)]
    pub position: i64,

    /// Whether loop is enabled
    #[serde(default)]
    pub loop_enabled: bool,

    /// Loop start position in ticks
    #[serde(default)]
    pub loop_start: i64,

    /// Loop end position in ticks
    #[serde(default = "default_loop_end")]
    pub loop_end: i64,

    /// Whether playback is active (usually false when saved)
    #[serde(default)]
    pub playing: bool,

    /// Whether recording is active (usually false when saved)
    #[serde(default)]
    pub recording: bool,
}

fn default_loop_end() -> i64 {
    1920 // 4 bars at 480 ppqn
}

// ============================================================================
// MARKERS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marker {
    /// Marker name
    pub name: String,

    /// Position in ticks
    pub position: i64,

    /// Marker color (hex RGB)
    #[serde(default = "default_marker_color")]
    pub color: String,

    /// Whether this is a region marker (has duration)
    #[serde(default)]
    pub is_region: bool,

    /// Region duration in ticks (if is_region = true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
}

fn default_marker_color() -> String {
    "#EAB308".to_string()
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

impl Project {
    /// Create a new empty project with default settings
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();

        Self {
            version: PROJECT_VERSION,
            metadata: ProjectMetadata {
                name,
                description: None,
                created_at: now.clone(),
                updated_at: now,
                author: None,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
            },
            settings: MusicalSettings {
                bpm: 120.0,
                time_signature_numerator: 4,
                time_signature_denominator: 4,
                key_signature: Some("C".to_string()),
                ppqn: 480,
                sample_rate: 44100,
                bit_depth: 16,
            },
            tracks: Vec::new(),
            mixer: None,
            automation: None,
            routing: None,
            transport: TransportState {
                position: 0,
                loop_enabled: false,
                loop_start: 0,
                loop_end: 1920,
                playing: false,
                recording: false,
            },
            markers: Vec::new(),
            tags: Vec::new(),
            user_data: HashMap::new(),
        }
    }

    /// Serialize project to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize project to compact JSON string
    pub fn to_json_compact(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize project from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Save project to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let json = self.to_json().map_err(std::io::Error::other)?;
        fs::write(path, json)
    }

    /// Load project from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(path)?;
        Self::from_json(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Update the updated_at timestamp to now
    pub fn touch(&mut self) {
        self.metadata.updated_at = chrono::Utc::now().to_rfc3339();
    }

    /// Validate project data (basic checks)
    pub fn validate(&self) -> Result<(), String> {
        // Check version compatibility
        if self.version > PROJECT_VERSION {
            return Err(format!(
                "Project version {} is newer than supported version {}",
                self.version, PROJECT_VERSION
            ));
        }

        // Validate BPM range
        if self.settings.bpm < 20.0 || self.settings.bpm > 400.0 {
            return Err(format!("Invalid BPM: {}", self.settings.bpm));
        }

        // Validate time signature
        if self.settings.time_signature_numerator == 0
            || self.settings.time_signature_denominator == 0
        {
            return Err("Invalid time signature".to_string());
        }

        // Validate track IDs are unique
        let mut track_ids = std::collections::HashSet::new();
        for track in &self.tracks {
            if !track_ids.insert(track.id) {
                return Err(format!("Duplicate track ID: {}", track.id));
            }
        }

        Ok(())
    }

    /// Get total project duration in ticks
    pub fn get_duration_ticks(&self) -> i64 {
        let mut max_tick = 0i64;

        for track in &self.tracks {
            // Check clips
            for clip in &track.clips {
                let end_tick = clip.start_tick + clip.duration_ticks;
                max_tick = max_tick.max(end_tick);
            }

            // Check individual notes
            for note in &track.notes {
                let end_tick = note.start_tick + note.duration_ticks;
                max_tick = max_tick.max(end_tick);
            }
        }

        max_tick
    }

    /// Get total project duration in seconds
    pub fn get_duration_seconds(&self) -> f64 {
        let ticks = self.get_duration_ticks();
        let beats = ticks as f64 / self.settings.ppqn as f64;
        (beats * 60.0) / self.settings.bpm
    }

    /// Add a track to the project
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
        self.touch();
    }

    /// Remove a track by ID
    pub fn remove_track(&mut self, track_id: u32) -> Option<Track> {
        if let Some(index) = self.tracks.iter().position(|t| t.id == track_id) {
            self.touch();
            Some(self.tracks.remove(index))
        } else {
            None
        }
    }

    /// Get a track by ID
    pub fn get_track(&self, track_id: u32) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == track_id)
    }

    /// Get a mutable track by ID
    pub fn get_track_mut(&mut self, track_id: u32) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == track_id)
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new("Untitled Project".to_string())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_project() {
        let project = Project::new("Test Project".to_string());
        assert_eq!(project.metadata.name, "Test Project");
        assert_eq!(project.settings.bpm, 120.0);
        assert_eq!(project.tracks.len(), 0);
    }

    #[test]
    fn test_project_serialization() {
        let project = Project::new("Test".to_string());
        let json = project.to_json().expect("Should serialize");
        // Pretty-printed JSON has spaces after colons
        assert!(json.contains("\"name\": \"Test\""));
        assert!(json.contains("\"bpm\": 120.0"));
    }

    #[test]
    fn test_project_deserialization() {
        let project = Project::new("Test".to_string());
        let json = project.to_json().expect("Should serialize");
        let loaded = Project::from_json(&json).expect("Should deserialize");
        assert_eq!(loaded.metadata.name, "Test");
        assert_eq!(loaded.settings.bpm, 120.0);
    }

    #[test]
    fn test_project_validation() {
        let mut project = Project::new("Test".to_string());
        assert!(project.validate().is_ok());

        // Test invalid BPM
        project.settings.bpm = 500.0;
        assert!(project.validate().is_err());

        project.settings.bpm = 120.0;
        assert!(project.validate().is_ok());
    }

    #[test]
    fn test_add_remove_track() {
        let mut project = Project::new("Test".to_string());

        let track = Track {
            id: 1,
            name: "Track 1".to_string(),
            track_number: 1,
            channel: 0,
            program_number: 0,
            bank_msb: 0,
            bank_lsb: 0,
            muted: false,
            solo: false,
            armed: false,
            volume: 1.0,
            pan: 0.0,
            color: "#3B82F6".to_string(),
            height: 100,
            collapsed: false,
            source_file_id: None,
            clips: vec![],
            notes: vec![],
        };

        project.add_track(track);
        assert_eq!(project.tracks.len(), 1);

        let removed = project.remove_track(1);
        assert!(removed.is_some());
        assert_eq!(project.tracks.len(), 0);
    }

    #[test]
    fn test_project_duration() {
        let mut project = Project::new("Test".to_string());

        let mut track = Track {
            id: 1,
            name: "Track 1".to_string(),
            track_number: 1,
            channel: 0,
            program_number: 0,
            bank_msb: 0,
            bank_lsb: 0,
            muted: false,
            solo: false,
            armed: false,
            volume: 1.0,
            pan: 0.0,
            color: "#3B82F6".to_string(),
            height: 100,
            collapsed: false,
            source_file_id: None,
            clips: vec![],
            notes: vec![],
        };

        // Add a note from tick 0 to 480 (1 beat at 480 ppqn)
        track.notes.push(Note {
            pitch: 60,
            velocity: 100,
            start_tick: 0,
            duration_ticks: 480,
            channel: None,
        });

        project.add_track(track);

        assert_eq!(project.get_duration_ticks(), 480);

        // Duration in seconds: 1 beat at 120 BPM = 0.5 seconds
        let duration_seconds = project.get_duration_seconds();
        assert!((duration_seconds - 0.5).abs() < 0.001);
    }
}
