// Settings modules for DAW configuration
// All settings are persisted to ~/.midi-software-center/config.json

pub mod advanced;
pub mod audio;
pub mod display;
pub mod general;
pub mod import_export;
pub mod keyboard;
pub mod library;
pub mod midi;
pub mod mixer;
pub mod performance;
pub mod playback;
pub mod privacy;
pub mod recording;
pub mod sync;
pub mod track;

// Re-export all settings types
pub use advanced::{AdvancedSettings, LogLevel};
pub use audio::{AudioSettings, BufferSize, SampleRate};
pub use display::{DisplaySettings, FontSize, GridSnapOption};
pub use general::{GeneralSettings, Language, StartupBehavior, Theme};
pub use import_export::{DuplicateHandling, ImportExportSettings};
pub use keyboard::{KeybindingProfile, KeyboardSettings};
pub use library::{LibrarySettings, WatchMode};
pub use midi::{MidiSettings, SyncMode};
pub use mixer::{FaderType, MeteringMode, MixerSettings};
pub use performance::{PerformanceSettings, VirtualScrollingThreshold};
pub use playback::{ClickSound, PlaybackSettings};
pub use privacy::{DataRetentionPolicy, PrivacySettings};
pub use recording::{RecordingFormat, RecordingSettings};
pub use sync::{SyncInterval, SyncSettings};
pub use track::{TrackColor, TrackSettings};

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Master settings container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct AppSettings {
    pub general: GeneralSettings,
    pub audio: AudioSettings,
    pub display: DisplaySettings,
    pub keyboard: KeyboardSettings,
    pub midi: MidiSettings,
    pub mixer: MixerSettings,
    pub track: TrackSettings,
    pub import_export: ImportExportSettings,
    pub performance: PerformanceSettings,
    pub library: LibrarySettings,
    pub playback: PlaybackSettings,
    pub recording: RecordingSettings,
    pub sync: SyncSettings,
    pub privacy: PrivacySettings,
    pub advanced: AdvancedSettings,
}


impl AppSettings {
    /// Get the path to the config file
    pub fn config_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        let config_dir = home.join(".midi-software-center");
        Ok(config_dir.join("config.json"))
    }

    /// Load settings from config file
    pub fn load() -> Result<Self, String> {
        let path = Self::config_path()?;

        if !path.exists() {
            // Return defaults if file doesn't exist
            return Ok(Self::default());
        }

        let contents =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read config file: {}", e))?;

        let settings: Self = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(settings)
    }

    /// Save settings to config file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Validate all settings
    pub fn validate(&self) -> Result<(), String> {
        self.general.validate()?;
        self.audio.validate()?;
        self.display.validate()?;
        self.keyboard.validate()?;
        self.midi.validate()?;
        self.mixer.validate()?;
        self.track.validate()?;
        self.import_export.validate()?;
        self.performance.validate()?;
        self.library.validate()?;
        self.playback.validate()?;
        self.recording.validate()?;
        self.sync.validate()?;
        self.privacy.validate()?;
        self.advanced.validate()?;
        Ok(())
    }
}
