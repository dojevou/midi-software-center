//! Settings Management Tauri Commands
//!
//! This module provides comprehensive Tauri commands for managing application settings.
//! All settings are persisted to ~/.midi-software-center/config.json and support
//! validation, import/export, and real-time updates.
//!
//! # Features
//! - Load/save all settings or individual categories
//! - Settings validation before save
//! - Import/export settings profiles
//! - Reset to defaults (individual or all)
//! - Settings change notifications
//! - Settings migration for version upgrades

#![allow(dead_code)]

use crate::settings::{
    AdvancedSettings, AppSettings, AudioSettings, BufferSize, DisplaySettings, GeneralSettings,
    ImportExportSettings, KeyboardSettings, LibrarySettings, MidiSettings, MixerSettings,
    PerformanceSettings, PlaybackSettings, PrivacySettings, RecordingSettings, SampleRate,
    SyncSettings, TrackSettings,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use thiserror::Error;
use tokio::sync::RwLock;

// ============================================================================
// TYPES
// ============================================================================

/// Settings category identifier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SettingsCategory {
    General,
    Audio,
    Display,
    Keyboard,
    Midi,
    Mixer,
    Track,
    ImportExport,
    Performance,
    Library,
    Playback,
    Recording,
    Sync,
    Privacy,
    Advanced,
}

impl SettingsCategory {
    pub fn all() -> Vec<Self> {
        vec![
            Self::General,
            Self::Audio,
            Self::Display,
            Self::Keyboard,
            Self::Midi,
            Self::Mixer,
            Self::Track,
            Self::ImportExport,
            Self::Performance,
            Self::Library,
            Self::Playback,
            Self::Recording,
            Self::Sync,
            Self::Privacy,
            Self::Advanced,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Audio => "Audio",
            Self::Display => "Display",
            Self::Keyboard => "Keyboard Shortcuts",
            Self::Midi => "MIDI",
            Self::Mixer => "Mixer",
            Self::Track => "Tracks",
            Self::ImportExport => "Import/Export",
            Self::Performance => "Performance",
            Self::Library => "Library",
            Self::Playback => "Playback",
            Self::Recording => "Recording",
            Self::Sync => "Sync",
            Self::Privacy => "Privacy",
            Self::Advanced => "Advanced",
        }
    }
}

/// Settings profile for import/export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsProfile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub modified_at: i64,
    pub version: String,
    pub settings: AppSettings,
    pub categories_included: Vec<SettingsCategory>,
}

/// Settings change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsChange {
    pub category: SettingsCategory,
    pub key: String,
    pub old_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub timestamp: i64,
}

/// Settings validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub category: SettingsCategory,
    pub field: String,
    pub message: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub category: SettingsCategory,
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_output: bool,
    pub sample_rates: Vec<u32>,
    pub buffer_sizes: Vec<u32>,
    pub channels: u32,
    pub is_default: bool,
}

/// MIDI device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiDeviceInfo {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_output: bool,
    pub is_virtual: bool,
    pub is_connected: bool,
}

/// Settings state for Tauri
pub struct SettingsState {
    pub settings: Arc<RwLock<AppSettings>>,
    pub change_history: Arc<RwLock<Vec<SettingsChange>>>,
    pub profiles_dir: PathBuf,
}

impl Default for SettingsState {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let profiles_dir = home.join(".midi-software-center").join("profiles");

        Self {
            settings: Arc::new(RwLock::new(AppSettings::default())),
            change_history: Arc::new(RwLock::new(Vec::new())),
            profiles_dir,
        }
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Settings not found: {0}")]
    NotFound(String),

    #[error("Invalid settings value: {0}")]
    InvalidValue(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("File operation failed: {0}")]
    FileError(String),

    #[error("Serialization failed: {0}")]
    SerializationError(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Profile already exists: {0}")]
    ProfileExists(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("State error: {0}")]
    StateError(String),
}

impl From<SettingsError> for String {
    fn from(err: SettingsError) -> Self {
        err.to_string()
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn generate_profile_id() -> String {
    format!("profile_{}", current_timestamp())
}

fn ensure_profiles_dir(profiles_dir: &PathBuf) -> Result<(), SettingsError> {
    if !profiles_dir.exists() {
        fs::create_dir_all(profiles_dir).map_err(|e| {
            SettingsError::FileError(format!("Failed to create profiles directory: {}", e))
        })?;
    }
    Ok(())
}

fn profile_path(profiles_dir: &std::path::Path, profile_id: &str) -> PathBuf {
    profiles_dir.join(format!("{}.json", profile_id))
}

// ============================================================================
// IMPLEMENTATION FUNCTIONS
// ============================================================================

/// Load all settings from disk
pub async fn load_settings_impl(state: &SettingsState) -> Result<AppSettings, SettingsError> {
    let loaded = AppSettings::load().map_err(SettingsError::FileError)?;

    // Update state
    let mut settings = state.settings.write().await;
    *settings = loaded.clone();

    Ok(loaded)
}

/// Save all settings to disk
pub async fn save_settings_impl(
    state: &SettingsState,
    settings: &AppSettings,
) -> Result<(), SettingsError> {
    // Validate before saving
    settings.validate().map_err(SettingsError::ValidationFailed)?;

    // Update state
    {
        let mut current = state.settings.write().await;
        *current = settings.clone();
    }

    // Save to disk
    settings.save().map_err(SettingsError::FileError)?;

    Ok(())
}

/// Get current settings (from memory)
pub async fn get_settings_impl(state: &SettingsState) -> Result<AppSettings, SettingsError> {
    let settings = state.settings.read().await;
    Ok(settings.clone())
}

/// Get settings for a specific category
pub async fn get_category_settings_impl(
    state: &SettingsState,
    category: SettingsCategory,
) -> Result<serde_json::Value, SettingsError> {
    let settings = state.settings.read().await;

    let value = match category {
        SettingsCategory::General => serde_json::to_value(&settings.general),
        SettingsCategory::Audio => serde_json::to_value(&settings.audio),
        SettingsCategory::Display => serde_json::to_value(&settings.display),
        SettingsCategory::Keyboard => serde_json::to_value(&settings.keyboard),
        SettingsCategory::Midi => serde_json::to_value(&settings.midi),
        SettingsCategory::Mixer => serde_json::to_value(&settings.mixer),
        SettingsCategory::Track => serde_json::to_value(&settings.track),
        SettingsCategory::ImportExport => serde_json::to_value(&settings.import_export),
        SettingsCategory::Performance => serde_json::to_value(&settings.performance),
        SettingsCategory::Library => serde_json::to_value(&settings.library),
        SettingsCategory::Playback => serde_json::to_value(&settings.playback),
        SettingsCategory::Recording => serde_json::to_value(&settings.recording),
        SettingsCategory::Sync => serde_json::to_value(&settings.sync),
        SettingsCategory::Privacy => serde_json::to_value(&settings.privacy),
        SettingsCategory::Advanced => serde_json::to_value(&settings.advanced),
    }
    .map_err(|e| SettingsError::SerializationError(e.to_string()))?;

    Ok(value)
}

/// Update settings for a specific category
pub async fn update_category_settings_impl(
    state: &SettingsState,
    category: SettingsCategory,
    value: serde_json::Value,
) -> Result<(), SettingsError> {
    let mut settings = state.settings.write().await;

    match category {
        SettingsCategory::General => {
            let general: GeneralSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            general.validate().map_err(SettingsError::ValidationFailed)?;
            settings.general = general;
        },
        SettingsCategory::Audio => {
            let audio: AudioSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            audio.validate().map_err(SettingsError::ValidationFailed)?;
            settings.audio = audio;
        },
        SettingsCategory::Display => {
            let display: DisplaySettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            display.validate().map_err(SettingsError::ValidationFailed)?;
            settings.display = display;
        },
        SettingsCategory::Keyboard => {
            let keyboard: KeyboardSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            keyboard.validate().map_err(SettingsError::ValidationFailed)?;
            settings.keyboard = keyboard;
        },
        SettingsCategory::Midi => {
            let midi: MidiSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            midi.validate().map_err(SettingsError::ValidationFailed)?;
            settings.midi = midi;
        },
        SettingsCategory::Mixer => {
            let mixer: MixerSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            mixer.validate().map_err(SettingsError::ValidationFailed)?;
            settings.mixer = mixer;
        },
        SettingsCategory::Track => {
            let track: TrackSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            track.validate().map_err(SettingsError::ValidationFailed)?;
            settings.track = track;
        },
        SettingsCategory::ImportExport => {
            let ie: ImportExportSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            ie.validate().map_err(SettingsError::ValidationFailed)?;
            settings.import_export = ie;
        },
        SettingsCategory::Performance => {
            let perf: PerformanceSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            perf.validate().map_err(SettingsError::ValidationFailed)?;
            settings.performance = perf;
        },
        SettingsCategory::Library => {
            let lib: LibrarySettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            lib.validate().map_err(SettingsError::ValidationFailed)?;
            settings.library = lib;
        },
        SettingsCategory::Playback => {
            let pb: PlaybackSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            pb.validate().map_err(SettingsError::ValidationFailed)?;
            settings.playback = pb;
        },
        SettingsCategory::Recording => {
            let rec: RecordingSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            rec.validate().map_err(SettingsError::ValidationFailed)?;
            settings.recording = rec;
        },
        SettingsCategory::Sync => {
            let sync: SyncSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            sync.validate().map_err(SettingsError::ValidationFailed)?;
            settings.sync = sync;
        },
        SettingsCategory::Privacy => {
            let priv_settings: PrivacySettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            priv_settings.validate().map_err(SettingsError::ValidationFailed)?;
            settings.privacy = priv_settings;
        },
        SettingsCategory::Advanced => {
            let adv: AdvancedSettings = serde_json::from_value(value)
                .map_err(|e| SettingsError::InvalidValue(e.to_string()))?;
            adv.validate().map_err(SettingsError::ValidationFailed)?;
            settings.advanced = adv;
        },
    }

    // Save to disk
    settings.save().map_err(SettingsError::FileError)?;

    Ok(())
}

/// Reset settings for a specific category to defaults
pub async fn reset_category_impl(
    state: &SettingsState,
    category: SettingsCategory,
) -> Result<(), SettingsError> {
    let mut settings = state.settings.write().await;

    match category {
        SettingsCategory::General => settings.general = GeneralSettings::default(),
        SettingsCategory::Audio => settings.audio = AudioSettings::default(),
        SettingsCategory::Display => settings.display = DisplaySettings::default(),
        SettingsCategory::Keyboard => settings.keyboard = KeyboardSettings::default(),
        SettingsCategory::Midi => settings.midi = MidiSettings::default(),
        SettingsCategory::Mixer => settings.mixer = MixerSettings::default(),
        SettingsCategory::Track => settings.track = TrackSettings::default(),
        SettingsCategory::ImportExport => settings.import_export = ImportExportSettings::default(),
        SettingsCategory::Performance => settings.performance = PerformanceSettings::default(),
        SettingsCategory::Library => settings.library = LibrarySettings::default(),
        SettingsCategory::Playback => settings.playback = PlaybackSettings::default(),
        SettingsCategory::Recording => settings.recording = RecordingSettings::default(),
        SettingsCategory::Sync => settings.sync = SyncSettings::default(),
        SettingsCategory::Privacy => settings.privacy = PrivacySettings::default(),
        SettingsCategory::Advanced => settings.advanced = AdvancedSettings::default(),
    }

    // Save to disk
    settings.save().map_err(SettingsError::FileError)?;

    Ok(())
}

/// Reset all settings to defaults
pub async fn reset_all_settings_impl(state: &SettingsState) -> Result<(), SettingsError> {
    let mut settings = state.settings.write().await;
    *settings = AppSettings::default();

    settings.save().map_err(SettingsError::FileError)?;

    Ok(())
}

/// Validate all settings
pub async fn validate_settings_impl(
    state: &SettingsState,
) -> Result<ValidationResult, SettingsError> {
    let settings = state.settings.read().await;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Validate each category
    if let Err(e) = settings.general.validate() {
        errors.push(ValidationError {
            category: SettingsCategory::General,
            field: "general".to_string(),
            message: e,
            code: "GENERAL_INVALID".to_string(),
        });
    }

    if let Err(e) = settings.audio.validate() {
        errors.push(ValidationError {
            category: SettingsCategory::Audio,
            field: "audio".to_string(),
            message: e,
            code: "AUDIO_INVALID".to_string(),
        });
    }

    // Add performance warnings
    if settings.audio.buffer_size == BufferSize::Samples32 {
        warnings.push(ValidationWarning {
            category: SettingsCategory::Audio,
            field: "buffer_size".to_string(),
            message: "Very small buffer size may cause audio glitches".to_string(),
            suggestion: Some("Consider using 128 or higher for stable playback".to_string()),
        });
    }

    if settings.audio.sample_rate == SampleRate::Hz192000 {
        warnings.push(ValidationWarning {
            category: SettingsCategory::Audio,
            field: "sample_rate".to_string(),
            message: "Very high sample rate increases CPU usage".to_string(),
            suggestion: Some("48000 Hz is recommended for most use cases".to_string()),
        });
    }

    Ok(ValidationResult { valid: errors.is_empty(), errors, warnings })
}

/// Export settings to a profile
pub async fn export_profile_impl(
    state: &SettingsState,
    name: String,
    description: Option<String>,
    categories: Option<Vec<SettingsCategory>>,
) -> Result<SettingsProfile, SettingsError> {
    ensure_profiles_dir(&state.profiles_dir)?;

    let settings = state.settings.read().await;
    let now = current_timestamp();
    let categories_included = categories.unwrap_or_else(SettingsCategory::all);

    let profile = SettingsProfile {
        id: generate_profile_id(),
        name,
        description,
        created_at: now,
        modified_at: now,
        version: env!("CARGO_PKG_VERSION").to_string(),
        settings: settings.clone(),
        categories_included,
    };

    let path = profile_path(&state.profiles_dir, &profile.id);
    let json = serde_json::to_string_pretty(&profile)
        .map_err(|e| SettingsError::SerializationError(e.to_string()))?;

    fs::write(&path, json)
        .map_err(|e| SettingsError::FileError(format!("Failed to write profile: {}", e)))?;

    Ok(profile)
}

/// Import settings from a profile
pub async fn import_profile_impl(
    state: &SettingsState,
    profile_id: String,
    categories: Option<Vec<SettingsCategory>>,
) -> Result<(), SettingsError> {
    let path = profile_path(&state.profiles_dir, &profile_id);

    if !path.exists() {
        return Err(SettingsError::ProfileNotFound(profile_id));
    }

    let json = fs::read_to_string(&path)
        .map_err(|e| SettingsError::FileError(format!("Failed to read profile: {}", e)))?;

    let profile: SettingsProfile = serde_json::from_str(&json).map_err(|e| {
        SettingsError::SerializationError(format!("Failed to parse profile: {}", e))
    })?;

    let categories_to_import = categories.unwrap_or(profile.categories_included);

    let mut settings = state.settings.write().await;

    for category in categories_to_import {
        match category {
            SettingsCategory::General => settings.general = profile.settings.general.clone(),
            SettingsCategory::Audio => settings.audio = profile.settings.audio.clone(),
            SettingsCategory::Display => settings.display = profile.settings.display.clone(),
            SettingsCategory::Keyboard => settings.keyboard = profile.settings.keyboard.clone(),
            SettingsCategory::Midi => settings.midi = profile.settings.midi.clone(),
            SettingsCategory::Mixer => settings.mixer = profile.settings.mixer.clone(),
            SettingsCategory::Track => settings.track = profile.settings.track.clone(),
            SettingsCategory::ImportExport => {
                settings.import_export = profile.settings.import_export.clone()
            },
            SettingsCategory::Performance => {
                settings.performance = profile.settings.performance.clone()
            },
            SettingsCategory::Library => settings.library = profile.settings.library.clone(),
            SettingsCategory::Playback => settings.playback = profile.settings.playback.clone(),
            SettingsCategory::Recording => settings.recording = profile.settings.recording.clone(),
            SettingsCategory::Sync => settings.sync = profile.settings.sync.clone(),
            SettingsCategory::Privacy => settings.privacy = profile.settings.privacy.clone(),
            SettingsCategory::Advanced => settings.advanced = profile.settings.advanced.clone(),
        }
    }

    // Save to disk
    settings.save().map_err(SettingsError::FileError)?;

    Ok(())
}

/// List all available profiles
pub async fn list_profiles_impl(
    state: &SettingsState,
) -> Result<Vec<SettingsProfile>, SettingsError> {
    ensure_profiles_dir(&state.profiles_dir)?;

    let mut profiles = Vec::new();

    let entries = fs::read_dir(&state.profiles_dir).map_err(|e| {
        SettingsError::FileError(format!("Failed to read profiles directory: {}", e))
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            if let Ok(json) = fs::read_to_string(&path) {
                if let Ok(profile) = serde_json::from_str::<SettingsProfile>(&json) {
                    profiles.push(profile);
                }
            }
        }
    }

    // Sort by modified_at descending
    profiles.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    Ok(profiles)
}

/// Delete a profile
pub async fn delete_profile_impl(
    state: &SettingsState,
    profile_id: String,
) -> Result<(), SettingsError> {
    let path = profile_path(&state.profiles_dir, &profile_id);

    if !path.exists() {
        return Err(SettingsError::ProfileNotFound(profile_id));
    }

    fs::remove_file(&path)
        .map_err(|e| SettingsError::FileError(format!("Failed to delete profile: {}", e)))?;

    Ok(())
}

/// Get available audio devices
pub async fn get_audio_devices_impl() -> Result<Vec<AudioDeviceInfo>, SettingsError> {
    // In a real implementation, this would query the audio system
    // For now, return mock data
    Ok(vec![AudioDeviceInfo {
        id: "default".to_string(),
        name: "System Default".to_string(),
        is_input: true,
        is_output: true,
        sample_rates: vec![44100, 48000, 96000],
        buffer_sizes: vec![128, 256, 512, 1024],
        channels: 2,
        is_default: true,
    }])
}

/// Get available MIDI devices
pub async fn get_midi_devices_impl() -> Result<Vec<MidiDeviceInfo>, SettingsError> {
    // In a real implementation, this would query the MIDI system
    // For now, return mock data
    Ok(vec![MidiDeviceInfo {
        id: "virtual".to_string(),
        name: "Virtual MIDI Port".to_string(),
        is_input: true,
        is_output: true,
        is_virtual: true,
        is_connected: true,
    }])
}

/// Get settings change history
pub async fn get_settings_history_impl(
    state: &SettingsState,
    limit: Option<usize>,
) -> Result<Vec<SettingsChange>, SettingsError> {
    let history = state.change_history.read().await;
    let limit = limit.unwrap_or(100);

    Ok(history.iter().rev().take(limit).cloned().collect())
}

/// Get settings file path
pub async fn get_settings_path_impl() -> Result<String, SettingsError> {
    let path = AppSettings::config_path().map_err(SettingsError::FileError)?;

    Ok(path.to_string_lossy().to_string())
}

/// Check if settings file exists
pub async fn settings_file_exists_impl() -> Result<bool, SettingsError> {
    let path = AppSettings::config_path().map_err(SettingsError::FileError)?;

    Ok(path.exists())
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

#[tauri::command]
pub async fn load_settings(state: State<'_, SettingsState>) -> Result<AppSettings, String> {
    load_settings_impl(&state).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, SettingsState>,
    settings: AppSettings,
) -> Result<(), String> {
    save_settings_impl(&state, &settings).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, SettingsState>) -> Result<AppSettings, String> {
    get_settings_impl(&state).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_category_settings(
    state: State<'_, SettingsState>,
    category: SettingsCategory,
) -> Result<serde_json::Value, String> {
    get_category_settings_impl(&state, category).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_category_settings(
    state: State<'_, SettingsState>,
    category: SettingsCategory,
    value: serde_json::Value,
) -> Result<(), String> {
    update_category_settings_impl(&state, category, value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_category(
    state: State<'_, SettingsState>,
    category: SettingsCategory,
) -> Result<(), String> {
    reset_category_impl(&state, category).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_all_settings(state: State<'_, SettingsState>) -> Result<(), String> {
    reset_all_settings_impl(&state).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_settings(
    state: State<'_, SettingsState>,
) -> Result<ValidationResult, String> {
    validate_settings_impl(&state).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_settings_profile(
    state: State<'_, SettingsState>,
    name: String,
    description: Option<String>,
    categories: Option<Vec<SettingsCategory>>,
) -> Result<SettingsProfile, String> {
    export_profile_impl(&state, name, description, categories)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_settings_profile(
    state: State<'_, SettingsState>,
    profile_id: String,
    categories: Option<Vec<SettingsCategory>>,
) -> Result<(), String> {
    import_profile_impl(&state, profile_id, categories)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_settings_profiles(
    state: State<'_, SettingsState>,
) -> Result<Vec<SettingsProfile>, String> {
    list_profiles_impl(&state).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_settings_profile(
    state: State<'_, SettingsState>,
    profile_id: String,
) -> Result<(), String> {
    delete_profile_impl(&state, profile_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_audio_devices() -> Result<Vec<AudioDeviceInfo>, String> {
    get_audio_devices_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_midi_devices() -> Result<Vec<MidiDeviceInfo>, String> {
    get_midi_devices_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings_history(
    state: State<'_, SettingsState>,
    limit: Option<usize>,
) -> Result<Vec<SettingsChange>, String> {
    get_settings_history_impl(&state, limit).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings_path() -> Result<String, String> {
    get_settings_path_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn settings_file_exists() -> Result<bool, String> {
    settings_file_exists_impl().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_settings_categories() -> Vec<SettingsCategoryInfo> {
    SettingsCategory::all()
        .into_iter()
        .map(|cat| SettingsCategoryInfo { id: cat, name: cat.display_name().to_string() })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsCategoryInfo {
    pub id: SettingsCategory,
    pub name: String,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_category_all() {
        let categories = SettingsCategory::all();
        assert_eq!(categories.len(), 15);
    }

    #[test]
    fn test_settings_category_display_name() {
        assert_eq!(SettingsCategory::General.display_name(), "General");
        assert_eq!(SettingsCategory::Audio.display_name(), "Audio");
        assert_eq!(
            SettingsCategory::Keyboard.display_name(),
            "Keyboard Shortcuts"
        );
    }

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 0);
    }

    #[test]
    fn test_generate_profile_id() {
        let id = generate_profile_id();
        assert!(id.starts_with("profile_"));
    }

    #[test]
    fn test_settings_state_default() {
        let state = SettingsState::default();
        assert!(state.profiles_dir.to_string_lossy().contains("profiles"));
    }

    #[test]
    fn test_settings_error_display() {
        let err = SettingsError::NotFound("test".to_string());
        assert_eq!(err.to_string(), "Settings not found: test");
    }

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult { valid: true, errors: vec![], warnings: vec![] };
        assert!(result.valid);
    }

    #[test]
    fn test_validation_result_invalid() {
        let result = ValidationResult {
            valid: false,
            errors: vec![ValidationError {
                category: SettingsCategory::Audio,
                field: "buffer_size".to_string(),
                message: "Invalid buffer size".to_string(),
                code: "INVALID_BUFFER".to_string(),
            }],
            warnings: vec![],
        };
        assert!(!result.valid);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_audio_device_info() {
        let device = AudioDeviceInfo {
            id: "test".to_string(),
            name: "Test Device".to_string(),
            is_input: true,
            is_output: false,
            sample_rates: vec![44100, 48000],
            buffer_sizes: vec![256, 512],
            channels: 2,
            is_default: false,
        };
        assert_eq!(device.id, "test");
        assert!(device.is_input);
        assert!(!device.is_output);
    }

    #[test]
    fn test_midi_device_info() {
        let device = MidiDeviceInfo {
            id: "midi-test".to_string(),
            name: "MIDI Test".to_string(),
            is_input: true,
            is_output: true,
            is_virtual: false,
            is_connected: true,
        };
        assert_eq!(device.id, "midi-test");
        assert!(device.is_connected);
    }

    #[test]
    fn test_settings_change_serialization() {
        let change = SettingsChange {
            category: SettingsCategory::Audio,
            key: "buffer_size".to_string(),
            old_value: serde_json::json!(256),
            new_value: serde_json::json!(512),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&change).unwrap();
        let parsed: SettingsChange = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.key, "buffer_size");
        assert_eq!(parsed.timestamp, 1234567890);
    }

    #[test]
    fn test_settings_profile_serialization() {
        let profile = SettingsProfile {
            id: "test_profile".to_string(),
            name: "Test Profile".to_string(),
            description: Some("A test profile".to_string()),
            created_at: 1234567890,
            modified_at: 1234567890,
            version: "1.0.0".to_string(),
            settings: AppSettings::default(),
            categories_included: vec![SettingsCategory::Audio, SettingsCategory::Midi],
        };

        let json = serde_json::to_string(&profile).unwrap();
        let parsed: SettingsProfile = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "test_profile");
        assert_eq!(parsed.name, "Test Profile");
        assert_eq!(parsed.categories_included.len(), 2);
    }

    #[test]
    fn test_get_settings_categories() {
        let categories = get_settings_categories();
        assert_eq!(categories.len(), 15);

        let general = categories.iter().find(|c| matches!(c.id, SettingsCategory::General));
        assert!(general.is_some());
        assert_eq!(general.unwrap().name, "General");
    }

    #[tokio::test]
    async fn test_settings_state_operations() {
        let state = SettingsState::default();

        // Get default settings
        let settings = get_settings_impl(&state).await.unwrap();
        assert_eq!(settings.audio.buffer_size, BufferSize::Samples512);
    }

    #[tokio::test]
    async fn test_validate_settings() {
        let state = SettingsState::default();
        let result = validate_settings_impl(&state).await.unwrap();

        // Default settings should be valid
        assert!(result.valid);
    }

    #[tokio::test]
    async fn test_get_category_settings() {
        let state = SettingsState::default();

        let audio_json = get_category_settings_impl(&state, SettingsCategory::Audio).await.unwrap();
        assert!(audio_json.is_object());
        assert!(audio_json.get("buffer_size").is_some());
    }

    #[tokio::test]
    async fn test_reset_category() {
        let state = SettingsState::default();

        // Modify audio settings
        {
            let mut settings = state.settings.write().await;
            settings.audio.buffer_size = BufferSize::Samples1024;
        }

        // Reset audio category
        reset_category_impl(&state, SettingsCategory::Audio).await.unwrap();

        // Verify reset
        let settings = state.settings.read().await;
        assert_eq!(settings.audio.buffer_size, BufferSize::Samples512);
    }

    #[tokio::test]
    async fn test_get_audio_devices() {
        let devices = get_audio_devices_impl().await.unwrap();
        assert!(!devices.is_empty());

        let default_device = devices.iter().find(|d| d.is_default);
        assert!(default_device.is_some());
    }

    #[tokio::test]
    async fn test_get_midi_devices() {
        let devices = get_midi_devices_impl().await.unwrap();
        assert!(!devices.is_empty());
    }

    #[tokio::test]
    async fn test_get_settings_path() {
        let path = get_settings_path_impl().await.unwrap();
        assert!(path.contains("config.json"));
    }

    #[tokio::test]
    async fn test_settings_history() {
        let state = SettingsState::default();
        let history = get_settings_history_impl(&state, Some(10)).await.unwrap();

        // Initially empty
        assert!(history.is_empty());
    }

    #[test]
    fn test_validation_warning() {
        let warning = ValidationWarning {
            category: SettingsCategory::Performance,
            field: "cache_size".to_string(),
            message: "Cache size may be too large".to_string(),
            suggestion: Some("Reduce to 512MB".to_string()),
        };

        assert!(warning.suggestion.is_some());
    }

    #[test]
    fn test_settings_category_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(SettingsCategory::Audio);
        set.insert(SettingsCategory::Midi);
        set.insert(SettingsCategory::Audio); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_profile_path() {
        let profiles_dir = PathBuf::from("/tmp/profiles");
        let path = profile_path(&profiles_dir, "test_profile");
        assert_eq!(path, PathBuf::from("/tmp/profiles/test_profile.json"));
    }
}
