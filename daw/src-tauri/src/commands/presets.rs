//! Presets Commands - Mixer Presets, Track Templates, Project Templates
//!
//! This module provides Tauri commands for managing presets and templates:
//! - Mixer presets (save/recall mixer channel settings)
//! - Track templates (reusable track configurations)
//! - Project templates (full project starting points)
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{command, State};

// =============================================================================
// MIXER PRESETS
// =============================================================================

/// Saved mixer channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerPresetChannel {
    pub track_number: u32,
    pub volume: u8,
    pub pan: u8,
    pub muted: bool,
    pub solo: bool,
    pub transpose: i8,
    pub octave_shift: i8,
    pub velocity: u8,
    pub velocity_min: u8,
    pub velocity_max: u8,
    pub pitch_bend: i16,
    pub pitch_bend_range: u8,
    pub note_low: u8,
    pub note_high: u8,
    pub quantize_grid: Option<String>,
    pub swing_amount: u8,
    pub delay_ticks: i32,
    pub humanize_timing: u8,
    pub humanize_velocity: u8,
    pub cc_values: HashMap<u8, u8>,
}

impl Default for MixerPresetChannel {
    fn default() -> Self {
        Self {
            track_number: 1,
            volume: 100,
            pan: 64,
            muted: false,
            solo: false,
            transpose: 0,
            octave_shift: 0,
            velocity: 100,
            velocity_min: 1,
            velocity_max: 127,
            pitch_bend: 0,
            pitch_bend_range: 2,
            note_low: 0,
            note_high: 127,
            quantize_grid: None,
            swing_amount: 0,
            delay_ticks: 0,
            humanize_timing: 0,
            humanize_velocity: 0,
            cc_values: HashMap::new(),
        }
    }
}

/// Saved mixer master configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerPresetMaster {
    pub volume: u8,
    pub pan: u8,
    pub tempo: f32,
    pub time_signature_numerator: u8,
    pub time_signature_denominator: u8,
    pub key_signature: Option<String>,
    pub swing_amount: u8,
    pub metronome_enabled: bool,
    pub metronome_volume: u8,
}

impl Default for MixerPresetMaster {
    fn default() -> Self {
        Self {
            volume: 100,
            pan: 64,
            tempo: 120.0,
            time_signature_numerator: 4,
            time_signature_denominator: 4,
            key_signature: None,
            swing_amount: 0,
            metronome_enabled: false,
            metronome_volume: 80,
        }
    }
}

/// Complete mixer preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerPreset {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub channels: Vec<MixerPresetChannel>,
    pub master: MixerPresetMaster,
    pub is_factory: bool,
    pub created_at: String,
    pub updated_at: String,
}

// =============================================================================
// TRACK TEMPLATES
// =============================================================================

/// MIDI routing configuration for a track template
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrackMidiRouting {
    pub input_port: Option<String>,
    pub input_channel: Option<u8>,
    pub output_port: Option<String>,
    pub output_channel: Option<u8>,
    pub program_change: Option<u8>,
    pub bank_msb: Option<u8>,
    pub bank_lsb: Option<u8>,
}

/// Track template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackTemplate {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub track_type: String, // "midi", "audio", "group", "return"
    pub color: Option<String>,
    pub icon: Option<String>,
    pub mixer_settings: MixerPresetChannel,
    pub midi_routing: TrackMidiRouting,
    pub gear_profile_id: Option<u64>,
    pub default_instrument: Option<String>,
    pub is_factory: bool,
    pub created_at: String,
    pub updated_at: String,
}

// =============================================================================
// PROJECT TEMPLATES
// =============================================================================

/// Project template track entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplateTrack {
    pub name: String,
    pub track_type: String,
    pub color: Option<String>,
    pub mixer_settings: MixerPresetChannel,
    pub midi_routing: TrackMidiRouting,
    pub gear_profile_id: Option<u64>,
}

/// Project template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub id: u64,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub master_settings: MixerPresetMaster,
    pub tracks: Vec<ProjectTemplateTrack>,
    pub default_time_signature: String,
    pub default_tempo: f32,
    pub default_key: Option<String>,
    pub thumbnail_path: Option<String>,
    pub is_factory: bool,
    pub created_at: String,
    pub updated_at: String,
}

// =============================================================================
// STATE
// =============================================================================

/// Presets state management
pub struct PresetsState {
    pub mixer_presets: Mutex<Vec<MixerPreset>>,
    pub track_templates: Mutex<Vec<TrackTemplate>>,
    pub project_templates: Mutex<Vec<ProjectTemplate>>,
    next_mixer_preset_id: Mutex<u64>,
    next_track_template_id: Mutex<u64>,
    next_project_template_id: Mutex<u64>,
}

impl Default for PresetsState {
    fn default() -> Self {
        // Create some factory presets
        let mixer_presets = vec![MixerPreset {
            id: 1,
            name: "Default Mix".to_string(),
            description: Some("Standard starting mix".to_string()),
            category: Some("General".to_string()),
            tags: vec!["default".to_string()],
            channels: vec![MixerPresetChannel::default()],
            master: MixerPresetMaster::default(),
            is_factory: true,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }];

        let track_templates = vec![
            TrackTemplate {
                id: 1,
                name: "Basic MIDI Track".to_string(),
                description: Some("Simple MIDI track with default settings".to_string()),
                category: Some("General".to_string()),
                tags: vec!["midi".to_string(), "default".to_string()],
                track_type: "midi".to_string(),
                color: Some("#3498db".to_string()),
                icon: None,
                mixer_settings: MixerPresetChannel::default(),
                midi_routing: TrackMidiRouting::default(),
                gear_profile_id: None,
                default_instrument: None,
                is_factory: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
            TrackTemplate {
                id: 2,
                name: "Drums (Channel 10)".to_string(),
                description: Some("Drum track on MIDI channel 10".to_string()),
                category: Some("Drums".to_string()),
                tags: vec!["drums".to_string(), "percussion".to_string()],
                track_type: "midi".to_string(),
                color: Some("#e74c3c".to_string()),
                icon: None,
                mixer_settings: MixerPresetChannel { track_number: 10, ..Default::default() },
                midi_routing: TrackMidiRouting { output_channel: Some(10), ..Default::default() },
                gear_profile_id: None,
                default_instrument: Some("Standard Drum Kit".to_string()),
                is_factory: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
            TrackTemplate {
                id: 3,
                name: "Bass".to_string(),
                description: Some("Bass track template".to_string()),
                category: Some("Bass".to_string()),
                tags: vec!["bass".to_string()],
                track_type: "midi".to_string(),
                color: Some("#9b59b6".to_string()),
                icon: None,
                mixer_settings: MixerPresetChannel {
                    track_number: 2,
                    octave_shift: -1,
                    ..Default::default()
                },
                midi_routing: TrackMidiRouting {
                    output_channel: Some(2),
                    program_change: Some(33), // Acoustic Bass
                    ..Default::default()
                },
                gear_profile_id: Some(1), // GM
                default_instrument: Some("Acoustic Bass".to_string()),
                is_factory: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        ];

        let project_templates = vec![
            ProjectTemplate {
                id: 1,
                name: "Empty Project".to_string(),
                description: Some("Blank canvas with no tracks".to_string()),
                category: Some("General".to_string()),
                tags: vec!["empty".to_string(), "default".to_string()],
                master_settings: MixerPresetMaster::default(),
                tracks: Vec::new(),
                default_time_signature: "4/4".to_string(),
                default_tempo: 120.0,
                default_key: None,
                thumbnail_path: None,
                is_factory: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
            ProjectTemplate {
                id: 2,
                name: "Basic Band".to_string(),
                description: Some("Drums, bass, keys, and lead tracks".to_string()),
                category: Some("Band".to_string()),
                tags: vec!["band".to_string(), "live".to_string()],
                master_settings: MixerPresetMaster::default(),
                tracks: vec![
                    ProjectTemplateTrack {
                        name: "Drums".to_string(),
                        track_type: "midi".to_string(),
                        color: Some("#e74c3c".to_string()),
                        mixer_settings: MixerPresetChannel {
                            track_number: 10,
                            ..Default::default()
                        },
                        midi_routing: TrackMidiRouting {
                            output_channel: Some(10),
                            ..Default::default()
                        },
                        gear_profile_id: None,
                    },
                    ProjectTemplateTrack {
                        name: "Bass".to_string(),
                        track_type: "midi".to_string(),
                        color: Some("#9b59b6".to_string()),
                        mixer_settings: MixerPresetChannel {
                            track_number: 2,
                            octave_shift: -1,
                            ..Default::default()
                        },
                        midi_routing: TrackMidiRouting {
                            output_channel: Some(2),
                            program_change: Some(33),
                            ..Default::default()
                        },
                        gear_profile_id: None,
                    },
                    ProjectTemplateTrack {
                        name: "Keys".to_string(),
                        track_type: "midi".to_string(),
                        color: Some("#3498db".to_string()),
                        mixer_settings: MixerPresetChannel {
                            track_number: 3,
                            ..Default::default()
                        },
                        midi_routing: TrackMidiRouting {
                            output_channel: Some(3),
                            program_change: Some(0), // Piano
                            ..Default::default()
                        },
                        gear_profile_id: None,
                    },
                    ProjectTemplateTrack {
                        name: "Lead".to_string(),
                        track_type: "midi".to_string(),
                        color: Some("#2ecc71".to_string()),
                        mixer_settings: MixerPresetChannel {
                            track_number: 4,
                            ..Default::default()
                        },
                        midi_routing: TrackMidiRouting {
                            output_channel: Some(4),
                            program_change: Some(80), // Lead Synth
                            ..Default::default()
                        },
                        gear_profile_id: None,
                    },
                ],
                default_time_signature: "4/4".to_string(),
                default_tempo: 120.0,
                default_key: Some("C".to_string()),
                thumbnail_path: None,
                is_factory: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        ];

        Self {
            mixer_presets: Mutex::new(mixer_presets),
            track_templates: Mutex::new(track_templates),
            project_templates: Mutex::new(project_templates),
            next_mixer_preset_id: Mutex::new(2),
            next_track_template_id: Mutex::new(4),
            next_project_template_id: Mutex::new(3),
        }
    }
}

// =============================================================================
// MIXER PRESET COMMANDS
// =============================================================================

/// List all mixer presets
#[command]
pub async fn mixer_presets_list(
    state: State<'_, PresetsState>,
) -> Result<Vec<MixerPreset>, String> {
    let presets = state.mixer_presets.lock().unwrap();
    Ok(presets.clone())
}

/// List presets by category
#[command]
pub async fn mixer_presets_list_by_category(
    state: State<'_, PresetsState>,
    category: String,
) -> Result<Vec<MixerPreset>, String> {
    let presets = state.mixer_presets.lock().unwrap();
    Ok(presets
        .iter()
        .filter(|p| p.category.as_ref() == Some(&category))
        .cloned()
        .collect())
}

/// Get a mixer preset by ID
#[command]
pub async fn mixer_presets_get(
    state: State<'_, PresetsState>,
    id: u64,
) -> Result<Option<MixerPreset>, String> {
    let presets = state.mixer_presets.lock().unwrap();
    Ok(presets.iter().find(|p| p.id == id).cloned())
}

/// Search mixer presets
#[command]
pub async fn mixer_presets_search(
    state: State<'_, PresetsState>,
    query: String,
) -> Result<Vec<MixerPreset>, String> {
    let presets = state.mixer_presets.lock().unwrap();
    let query_lower = query.to_lowercase();
    Ok(presets
        .iter()
        .filter(|p| {
            p.name.to_lowercase().contains(&query_lower)
                || p.description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || p.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect())
}

/// Create a mixer preset
#[command]
pub async fn mixer_presets_create(
    state: State<'_, PresetsState>,
    name: String,
    description: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    channels: Vec<MixerPresetChannel>,
    master: MixerPresetMaster,
) -> Result<MixerPreset, String> {
    let mut presets = state.mixer_presets.lock().unwrap();
    let mut next_id = state.next_mixer_preset_id.lock().unwrap();

    let preset = MixerPreset {
        id: *next_id,
        name,
        description,
        category,
        tags: tags.unwrap_or_default(),
        channels,
        master,
        is_factory: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    *next_id += 1;
    presets.push(preset.clone());
    Ok(preset)
}

/// Update a mixer preset
#[allow(clippy::too_many_arguments)]
#[command]
pub async fn mixer_presets_update(
    state: State<'_, PresetsState>,
    id: u64,
    name: Option<String>,
    description: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    channels: Option<Vec<MixerPresetChannel>>,
    master: Option<MixerPresetMaster>,
) -> Result<Option<MixerPreset>, String> {
    let mut presets = state.mixer_presets.lock().unwrap();

    if let Some(preset) = presets.iter_mut().find(|p| p.id == id) {
        if preset.is_factory {
            return Err("Cannot modify factory presets".to_string());
        }

        if let Some(n) = name {
            preset.name = n;
        }
        if let Some(d) = description {
            preset.description = Some(d);
        }
        if let Some(c) = category {
            preset.category = Some(c);
        }
        if let Some(t) = tags {
            preset.tags = t;
        }
        if let Some(ch) = channels {
            preset.channels = ch;
        }
        if let Some(m) = master {
            preset.master = m;
        }
        preset.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(Some(preset.clone()))
    } else {
        Ok(None)
    }
}

/// Delete a mixer preset
#[command]
pub async fn mixer_presets_delete(state: State<'_, PresetsState>, id: u64) -> Result<bool, String> {
    let mut presets = state.mixer_presets.lock().unwrap();

    if presets.iter().any(|p| p.id == id && p.is_factory) {
        return Err("Cannot delete factory presets".to_string());
    }

    let len_before = presets.len();
    presets.retain(|p| p.id != id);
    Ok(presets.len() < len_before)
}

// =============================================================================
// TRACK TEMPLATE COMMANDS
// =============================================================================

/// List all track templates
#[command]
pub async fn track_templates_list(
    state: State<'_, PresetsState>,
) -> Result<Vec<TrackTemplate>, String> {
    let templates = state.track_templates.lock().unwrap();
    Ok(templates.clone())
}

/// List templates by category
#[command]
pub async fn track_templates_list_by_category(
    state: State<'_, PresetsState>,
    category: String,
) -> Result<Vec<TrackTemplate>, String> {
    let templates = state.track_templates.lock().unwrap();
    Ok(templates
        .iter()
        .filter(|t| t.category.as_ref() == Some(&category))
        .cloned()
        .collect())
}

/// List templates by track type
#[command]
pub async fn track_templates_list_by_type(
    state: State<'_, PresetsState>,
    track_type: String,
) -> Result<Vec<TrackTemplate>, String> {
    let templates = state.track_templates.lock().unwrap();
    Ok(templates.iter().filter(|t| t.track_type == track_type).cloned().collect())
}

/// Get a track template by ID
#[command]
pub async fn track_templates_get(
    state: State<'_, PresetsState>,
    id: u64,
) -> Result<Option<TrackTemplate>, String> {
    let templates = state.track_templates.lock().unwrap();
    Ok(templates.iter().find(|t| t.id == id).cloned())
}

/// Search track templates
#[command]
pub async fn track_templates_search(
    state: State<'_, PresetsState>,
    query: String,
) -> Result<Vec<TrackTemplate>, String> {
    let templates = state.track_templates.lock().unwrap();
    let query_lower = query.to_lowercase();
    Ok(templates
        .iter()
        .filter(|t| {
            t.name.to_lowercase().contains(&query_lower)
                || t.description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect())
}

/// Create a track template
#[allow(clippy::too_many_arguments)]
#[command]
pub async fn track_templates_create(
    state: State<'_, PresetsState>,
    name: String,
    description: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    track_type: String,
    color: Option<String>,
    mixer_settings: MixerPresetChannel,
    midi_routing: TrackMidiRouting,
    gear_profile_id: Option<u64>,
    default_instrument: Option<String>,
) -> Result<TrackTemplate, String> {
    let mut templates = state.track_templates.lock().unwrap();
    let mut next_id = state.next_track_template_id.lock().unwrap();

    let template = TrackTemplate {
        id: *next_id,
        name,
        description,
        category,
        tags: tags.unwrap_or_default(),
        track_type,
        color,
        icon: None,
        mixer_settings,
        midi_routing,
        gear_profile_id,
        default_instrument,
        is_factory: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    *next_id += 1;
    templates.push(template.clone());
    Ok(template)
}

/// Update a track template
#[command]
pub async fn track_templates_update(
    state: State<'_, PresetsState>,
    id: u64,
    updates: HashMap<String, serde_json::Value>,
) -> Result<Option<TrackTemplate>, String> {
    let mut templates = state.track_templates.lock().unwrap();

    if let Some(template) = templates.iter_mut().find(|t| t.id == id) {
        if template.is_factory {
            return Err("Cannot modify factory templates".to_string());
        }

        if let Some(v) = updates.get("name") {
            if let Some(s) = v.as_str() {
                template.name = s.to_string();
            }
        }
        if let Some(v) = updates.get("description") {
            template.description = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = updates.get("category") {
            template.category = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = updates.get("color") {
            template.color = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = updates.get("default_instrument") {
            template.default_instrument = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = updates.get("gear_profile_id") {
            template.gear_profile_id = v.as_u64();
        }

        template.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(Some(template.clone()))
    } else {
        Ok(None)
    }
}

/// Delete a track template
#[command]
pub async fn track_templates_delete(
    state: State<'_, PresetsState>,
    id: u64,
) -> Result<bool, String> {
    let mut templates = state.track_templates.lock().unwrap();

    if templates.iter().any(|t| t.id == id && t.is_factory) {
        return Err("Cannot delete factory templates".to_string());
    }

    let len_before = templates.len();
    templates.retain(|t| t.id != id);
    Ok(templates.len() < len_before)
}

// =============================================================================
// PROJECT TEMPLATE COMMANDS
// =============================================================================

/// List all project templates
#[command]
pub async fn project_templates_list(
    state: State<'_, PresetsState>,
) -> Result<Vec<ProjectTemplate>, String> {
    let templates = state.project_templates.lock().unwrap();
    Ok(templates.clone())
}

/// List templates by category
#[command]
pub async fn project_templates_list_by_category(
    state: State<'_, PresetsState>,
    category: String,
) -> Result<Vec<ProjectTemplate>, String> {
    let templates = state.project_templates.lock().unwrap();
    Ok(templates
        .iter()
        .filter(|t| t.category.as_ref() == Some(&category))
        .cloned()
        .collect())
}

/// Get a project template by ID
#[command]
pub async fn project_templates_get(
    state: State<'_, PresetsState>,
    id: u64,
) -> Result<Option<ProjectTemplate>, String> {
    let templates = state.project_templates.lock().unwrap();
    Ok(templates.iter().find(|t| t.id == id).cloned())
}

/// Search project templates
#[command]
pub async fn project_templates_search(
    state: State<'_, PresetsState>,
    query: String,
) -> Result<Vec<ProjectTemplate>, String> {
    let templates = state.project_templates.lock().unwrap();
    let query_lower = query.to_lowercase();
    Ok(templates
        .iter()
        .filter(|t| {
            t.name.to_lowercase().contains(&query_lower)
                || t.description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect())
}

/// Create a project template
#[allow(clippy::too_many_arguments)]
#[command]
pub async fn project_templates_create(
    state: State<'_, PresetsState>,
    name: String,
    description: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    master_settings: MixerPresetMaster,
    tracks: Vec<ProjectTemplateTrack>,
    default_time_signature: Option<String>,
    default_tempo: Option<f32>,
    default_key: Option<String>,
) -> Result<ProjectTemplate, String> {
    let mut templates = state.project_templates.lock().unwrap();
    let mut next_id = state.next_project_template_id.lock().unwrap();

    let template = ProjectTemplate {
        id: *next_id,
        name,
        description,
        category,
        tags: tags.unwrap_or_default(),
        master_settings,
        tracks,
        default_time_signature: default_time_signature.unwrap_or_else(|| "4/4".to_string()),
        default_tempo: default_tempo.unwrap_or(120.0),
        default_key,
        thumbnail_path: None,
        is_factory: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    *next_id += 1;
    templates.push(template.clone());
    Ok(template)
}

/// Update a project template
#[allow(clippy::too_many_arguments)]
#[command]
pub async fn project_templates_update(
    state: State<'_, PresetsState>,
    id: u64,
    name: Option<String>,
    description: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    master_settings: Option<MixerPresetMaster>,
    tracks: Option<Vec<ProjectTemplateTrack>>,
    default_time_signature: Option<String>,
    default_tempo: Option<f32>,
    default_key: Option<String>,
) -> Result<Option<ProjectTemplate>, String> {
    let mut templates = state.project_templates.lock().unwrap();

    if let Some(template) = templates.iter_mut().find(|t| t.id == id) {
        if template.is_factory {
            return Err("Cannot modify factory templates".to_string());
        }

        if let Some(n) = name {
            template.name = n;
        }
        if let Some(d) = description {
            template.description = Some(d);
        }
        if let Some(c) = category {
            template.category = Some(c);
        }
        if let Some(t) = tags {
            template.tags = t;
        }
        if let Some(m) = master_settings {
            template.master_settings = m;
        }
        if let Some(tr) = tracks {
            template.tracks = tr;
        }
        if let Some(ts) = default_time_signature {
            template.default_time_signature = ts;
        }
        if let Some(tempo) = default_tempo {
            template.default_tempo = tempo;
        }
        if let Some(key) = default_key {
            template.default_key = Some(key);
        }

        template.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(Some(template.clone()))
    } else {
        Ok(None)
    }
}

/// Delete a project template
#[command]
pub async fn project_templates_delete(
    state: State<'_, PresetsState>,
    id: u64,
) -> Result<bool, String> {
    let mut templates = state.project_templates.lock().unwrap();

    if templates.iter().any(|t| t.id == id && t.is_factory) {
        return Err("Cannot delete factory templates".to_string());
    }

    let len_before = templates.len();
    templates.retain(|t| t.id != id);
    Ok(templates.len() < len_before)
}

/// Duplicate a project template
#[command]
pub async fn project_templates_duplicate(
    state: State<'_, PresetsState>,
    id: u64,
    new_name: Option<String>,
) -> Result<Option<ProjectTemplate>, String> {
    let templates = state.project_templates.lock().unwrap();
    let mut next_id = state.next_project_template_id.lock().unwrap();

    if let Some(original) = templates.iter().find(|t| t.id == id) {
        let mut duplicate = original.clone();
        duplicate.id = *next_id;
        duplicate.name = new_name.unwrap_or_else(|| format!("{} (Copy)", original.name));
        duplicate.is_factory = false;
        duplicate.created_at = chrono::Utc::now().to_rfc3339();
        duplicate.updated_at = chrono::Utc::now().to_rfc3339();

        *next_id += 1;

        drop(templates); // Release the lock before re-acquiring
        let mut templates = state.project_templates.lock().unwrap();
        templates.push(duplicate.clone());

        Ok(Some(duplicate))
    } else {
        Ok(None)
    }
}

// =============================================================================
// IMPLEMENTATION FUNCTIONS (for testing)
// =============================================================================

impl PresetsState {
    /// Create a new empty PresetsState (for testing)
    pub fn new_empty() -> Self {
        Self {
            mixer_presets: Mutex::new(Vec::new()),
            track_templates: Mutex::new(Vec::new()),
            project_templates: Mutex::new(Vec::new()),
            next_mixer_preset_id: Mutex::new(1),
            next_track_template_id: Mutex::new(1),
            next_project_template_id: Mutex::new(1),
        }
    }

    /// List all mixer presets
    pub fn list_mixer_presets(&self) -> Vec<MixerPreset> {
        self.mixer_presets.lock().unwrap().clone()
    }

    /// List mixer presets by category
    pub fn list_mixer_presets_by_category(&self, category: &str) -> Vec<MixerPreset> {
        self.mixer_presets
            .lock()
            .unwrap()
            .iter()
            .filter(|p| p.category.as_ref() == Some(&category.to_string()))
            .cloned()
            .collect()
    }

    /// Get mixer preset by ID
    pub fn get_mixer_preset(&self, id: u64) -> Option<MixerPreset> {
        self.mixer_presets.lock().unwrap().iter().find(|p| p.id == id).cloned()
    }

    /// Search mixer presets
    pub fn search_mixer_presets(&self, query: &str) -> Vec<MixerPreset> {
        let query_lower = query.to_lowercase();
        self.mixer_presets
            .lock()
            .unwrap()
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Create a mixer preset
    pub fn create_mixer_preset(
        &self,
        name: String,
        description: Option<String>,
        category: Option<String>,
        tags: Vec<String>,
        channels: Vec<MixerPresetChannel>,
        master: MixerPresetMaster,
    ) -> MixerPreset {
        let mut presets = self.mixer_presets.lock().unwrap();
        let mut next_id = self.next_mixer_preset_id.lock().unwrap();

        let preset = MixerPreset {
            id: *next_id,
            name,
            description,
            category,
            tags,
            channels,
            master,
            is_factory: false,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        *next_id += 1;
        presets.push(preset.clone());
        preset
    }

    /// Update a mixer preset
    pub fn update_mixer_preset(
        &self,
        id: u64,
        name: Option<String>,
        description: Option<String>,
        category: Option<String>,
    ) -> Result<Option<MixerPreset>, String> {
        let mut presets = self.mixer_presets.lock().unwrap();

        if let Some(preset) = presets.iter_mut().find(|p| p.id == id) {
            if preset.is_factory {
                return Err("Cannot modify factory presets".to_string());
            }

            if let Some(n) = name {
                preset.name = n;
            }
            if let Some(d) = description {
                preset.description = Some(d);
            }
            if let Some(c) = category {
                preset.category = Some(c);
            }
            preset.updated_at = chrono::Utc::now().to_rfc3339();
            Ok(Some(preset.clone()))
        } else {
            Ok(None)
        }
    }

    /// Delete a mixer preset
    pub fn delete_mixer_preset(&self, id: u64) -> Result<bool, String> {
        let mut presets = self.mixer_presets.lock().unwrap();

        if presets.iter().any(|p| p.id == id && p.is_factory) {
            return Err("Cannot delete factory presets".to_string());
        }

        let len_before = presets.len();
        presets.retain(|p| p.id != id);
        Ok(presets.len() < len_before)
    }

    /// List all track templates
    pub fn list_track_templates(&self) -> Vec<TrackTemplate> {
        self.track_templates.lock().unwrap().clone()
    }

    /// List track templates by category
    pub fn list_track_templates_by_category(&self, category: &str) -> Vec<TrackTemplate> {
        self.track_templates
            .lock()
            .unwrap()
            .iter()
            .filter(|t| t.category.as_ref() == Some(&category.to_string()))
            .cloned()
            .collect()
    }

    /// List track templates by type
    pub fn list_track_templates_by_type(&self, track_type: &str) -> Vec<TrackTemplate> {
        self.track_templates
            .lock()
            .unwrap()
            .iter()
            .filter(|t| t.track_type == track_type)
            .cloned()
            .collect()
    }

    /// Get track template by ID
    pub fn get_track_template(&self, id: u64) -> Option<TrackTemplate> {
        self.track_templates.lock().unwrap().iter().find(|t| t.id == id).cloned()
    }

    /// Search track templates
    pub fn search_track_templates(&self, query: &str) -> Vec<TrackTemplate> {
        let query_lower = query.to_lowercase();
        self.track_templates
            .lock()
            .unwrap()
            .iter()
            .filter(|t| {
                t.name.to_lowercase().contains(&query_lower)
                    || t.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Create a track template
    #[allow(clippy::too_many_arguments)]
    pub fn create_track_template(
        &self,
        name: String,
        description: Option<String>,
        category: Option<String>,
        tags: Vec<String>,
        track_type: String,
        color: Option<String>,
        mixer_settings: MixerPresetChannel,
        midi_routing: TrackMidiRouting,
        gear_profile_id: Option<u64>,
        default_instrument: Option<String>,
    ) -> TrackTemplate {
        let mut templates = self.track_templates.lock().unwrap();
        let mut next_id = self.next_track_template_id.lock().unwrap();

        let template = TrackTemplate {
            id: *next_id,
            name,
            description,
            category,
            tags,
            track_type,
            color,
            icon: None,
            mixer_settings,
            midi_routing,
            gear_profile_id,
            default_instrument,
            is_factory: false,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        *next_id += 1;
        templates.push(template.clone());
        template
    }

    /// Delete a track template
    pub fn delete_track_template(&self, id: u64) -> Result<bool, String> {
        let mut templates = self.track_templates.lock().unwrap();

        if templates.iter().any(|t| t.id == id && t.is_factory) {
            return Err("Cannot delete factory templates".to_string());
        }

        let len_before = templates.len();
        templates.retain(|t| t.id != id);
        Ok(templates.len() < len_before)
    }

    /// List all project templates
    pub fn list_project_templates(&self) -> Vec<ProjectTemplate> {
        self.project_templates.lock().unwrap().clone()
    }

    /// List project templates by category
    pub fn list_project_templates_by_category(&self, category: &str) -> Vec<ProjectTemplate> {
        self.project_templates
            .lock()
            .unwrap()
            .iter()
            .filter(|t| t.category.as_ref() == Some(&category.to_string()))
            .cloned()
            .collect()
    }

    /// Get project template by ID
    pub fn get_project_template(&self, id: u64) -> Option<ProjectTemplate> {
        self.project_templates.lock().unwrap().iter().find(|t| t.id == id).cloned()
    }

    /// Search project templates
    pub fn search_project_templates(&self, query: &str) -> Vec<ProjectTemplate> {
        let query_lower = query.to_lowercase();
        self.project_templates
            .lock()
            .unwrap()
            .iter()
            .filter(|t| {
                t.name.to_lowercase().contains(&query_lower)
                    || t.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Create a project template
    #[allow(clippy::too_many_arguments)]
    pub fn create_project_template(
        &self,
        name: String,
        description: Option<String>,
        category: Option<String>,
        tags: Vec<String>,
        master_settings: MixerPresetMaster,
        tracks: Vec<ProjectTemplateTrack>,
        default_time_signature: String,
        default_tempo: f32,
        default_key: Option<String>,
    ) -> ProjectTemplate {
        let mut templates = self.project_templates.lock().unwrap();
        let mut next_id = self.next_project_template_id.lock().unwrap();

        let template = ProjectTemplate {
            id: *next_id,
            name,
            description,
            category,
            tags,
            master_settings,
            tracks,
            default_time_signature,
            default_tempo,
            default_key,
            thumbnail_path: None,
            is_factory: false,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        *next_id += 1;
        templates.push(template.clone());
        template
    }

    /// Delete a project template
    pub fn delete_project_template(&self, id: u64) -> Result<bool, String> {
        let mut templates = self.project_templates.lock().unwrap();

        if templates.iter().any(|t| t.id == id && t.is_factory) {
            return Err("Cannot delete factory templates".to_string());
        }

        let len_before = templates.len();
        templates.retain(|t| t.id != id);
        Ok(templates.len() < len_before)
    }

    /// Duplicate a project template
    pub fn duplicate_project_template(
        &self,
        id: u64,
        new_name: Option<String>,
    ) -> Option<ProjectTemplate> {
        let templates = self.project_templates.lock().unwrap();

        if let Some(original) = templates.iter().find(|t| t.id == id) {
            let mut next_id = self.next_project_template_id.lock().unwrap();
            let mut duplicate = original.clone();
            duplicate.id = *next_id;
            duplicate.name = new_name.unwrap_or_else(|| format!("{} (Copy)", original.name));
            duplicate.is_factory = false;
            duplicate.created_at = chrono::Utc::now().to_rfc3339();
            duplicate.updated_at = chrono::Utc::now().to_rfc3339();

            *next_id += 1;

            drop(templates);
            let mut templates = self.project_templates.lock().unwrap();
            templates.push(duplicate.clone());

            Some(duplicate)
        } else {
            None
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // MixerPresetChannel Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mixer_preset_channel_defaults() {
        let channel = MixerPresetChannel::default();

        assert_eq!(channel.track_number, 1);
        assert_eq!(channel.volume, 100);
        assert_eq!(channel.pan, 64); // Center
        assert!(!channel.muted);
        assert!(!channel.solo);
        assert_eq!(channel.transpose, 0);
        assert_eq!(channel.octave_shift, 0);
        assert_eq!(channel.velocity, 100);
        assert_eq!(channel.velocity_min, 1);
        assert_eq!(channel.velocity_max, 127);
        assert_eq!(channel.pitch_bend, 0);
        assert_eq!(channel.pitch_bend_range, 2);
        assert_eq!(channel.note_low, 0);
        assert_eq!(channel.note_high, 127);
        assert!(channel.quantize_grid.is_none());
        assert_eq!(channel.swing_amount, 0);
        assert_eq!(channel.delay_ticks, 0);
        assert_eq!(channel.humanize_timing, 0);
        assert_eq!(channel.humanize_velocity, 0);
        assert!(channel.cc_values.is_empty());
    }

    #[test]
    fn test_mixer_preset_channel_with_cc_values() {
        let mut channel = MixerPresetChannel::default();
        channel.cc_values.insert(1, 64); // Modulation
        channel.cc_values.insert(7, 100); // Volume
        channel.cc_values.insert(10, 64); // Pan
        channel.cc_values.insert(64, 127); // Sustain

        assert_eq!(channel.cc_values.len(), 4);
        assert_eq!(channel.cc_values.get(&1), Some(&64));
        assert_eq!(channel.cc_values.get(&64), Some(&127));
    }

    // -------------------------------------------------------------------------
    // MixerPresetMaster Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mixer_preset_master_defaults() {
        let master = MixerPresetMaster::default();

        assert_eq!(master.volume, 100);
        assert_eq!(master.pan, 64);
        assert_eq!(master.tempo, 120.0);
        assert_eq!(master.time_signature_numerator, 4);
        assert_eq!(master.time_signature_denominator, 4);
        assert!(master.key_signature.is_none());
        assert_eq!(master.swing_amount, 0);
        assert!(!master.metronome_enabled);
        assert_eq!(master.metronome_volume, 80);
    }

    #[test]
    fn test_mixer_preset_master_custom() {
        let master = MixerPresetMaster {
            volume: 90,
            pan: 64,
            tempo: 140.0,
            time_signature_numerator: 6,
            time_signature_denominator: 8,
            key_signature: Some("G".to_string()),
            swing_amount: 25,
            metronome_enabled: true,
            metronome_volume: 100,
        };

        assert_eq!(master.tempo, 140.0);
        assert_eq!(master.time_signature_numerator, 6);
        assert_eq!(master.time_signature_denominator, 8);
        assert_eq!(master.key_signature, Some("G".to_string()));
        assert_eq!(master.swing_amount, 25);
        assert!(master.metronome_enabled);
    }

    // -------------------------------------------------------------------------
    // TrackMidiRouting Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_track_midi_routing_defaults() {
        let routing = TrackMidiRouting::default();

        assert!(routing.input_port.is_none());
        assert!(routing.input_channel.is_none());
        assert!(routing.output_port.is_none());
        assert!(routing.output_channel.is_none());
        assert!(routing.program_change.is_none());
        assert!(routing.bank_msb.is_none());
        assert!(routing.bank_lsb.is_none());
    }

    #[test]
    fn test_track_midi_routing_custom() {
        let routing = TrackMidiRouting {
            input_port: Some("MIDI In".to_string()),
            input_channel: Some(1),
            output_port: Some("MIDI Out".to_string()),
            output_channel: Some(10),
            program_change: Some(0), // Acoustic Grand Piano
            bank_msb: Some(0),
            bank_lsb: Some(0),
        };

        assert_eq!(routing.input_port, Some("MIDI In".to_string()));
        assert_eq!(routing.output_channel, Some(10));
        assert_eq!(routing.program_change, Some(0));
    }

    // -------------------------------------------------------------------------
    // PresetsState Factory Presets Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_presets_state_has_factory_presets() {
        let state = PresetsState::default();

        let mixer_presets = state.list_mixer_presets();
        let track_templates = state.list_track_templates();
        let project_templates = state.list_project_templates();

        // Should have factory presets
        assert!(!mixer_presets.is_empty());
        assert!(!track_templates.is_empty());
        assert!(!project_templates.is_empty());

        // First mixer preset should be "Default Mix"
        assert_eq!(mixer_presets[0].name, "Default Mix");
        assert!(mixer_presets[0].is_factory);

        // Should have 3 factory track templates
        assert_eq!(track_templates.len(), 3);
        assert_eq!(track_templates[0].name, "Basic MIDI Track");
        assert_eq!(track_templates[1].name, "Drums (Channel 10)");
        assert_eq!(track_templates[2].name, "Bass");

        // Should have 2 factory project templates
        assert_eq!(project_templates.len(), 2);
        assert_eq!(project_templates[0].name, "Empty Project");
        assert_eq!(project_templates[1].name, "Basic Band");
    }

    #[test]
    fn test_empty_presets_state() {
        let state = PresetsState::new_empty();

        assert!(state.list_mixer_presets().is_empty());
        assert!(state.list_track_templates().is_empty());
        assert!(state.list_project_templates().is_empty());
    }

    // -------------------------------------------------------------------------
    // Mixer Preset Operations Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_create_mixer_preset() {
        let state = PresetsState::new_empty();

        let preset = state.create_mixer_preset(
            "My Mix".to_string(),
            Some("A custom mix".to_string()),
            Some("Custom".to_string()),
            vec!["test".to_string()],
            vec![MixerPresetChannel::default()],
            MixerPresetMaster::default(),
        );

        assert_eq!(preset.id, 1);
        assert_eq!(preset.name, "My Mix");
        assert_eq!(preset.description, Some("A custom mix".to_string()));
        assert_eq!(preset.category, Some("Custom".to_string()));
        assert_eq!(preset.tags, vec!["test".to_string()]);
        assert!(!preset.is_factory);

        // Verify it's in the list
        let presets = state.list_mixer_presets();
        assert_eq!(presets.len(), 1);
    }

    #[test]
    fn test_get_mixer_preset() {
        let state = PresetsState::new_empty();

        state.create_mixer_preset(
            "Preset 1".to_string(),
            None,
            None,
            vec![],
            vec![],
            MixerPresetMaster::default(),
        );

        let found = state.get_mixer_preset(1);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Preset 1");

        let not_found = state.get_mixer_preset(999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_search_mixer_presets() {
        let state = PresetsState::new_empty();

        state.create_mixer_preset(
            "Rock Mix".to_string(),
            Some("For rock music".to_string()),
            Some("Rock".to_string()),
            vec!["rock".to_string(), "heavy".to_string()],
            vec![],
            MixerPresetMaster::default(),
        );

        state.create_mixer_preset(
            "Jazz Mix".to_string(),
            Some("Smooth jazz settings".to_string()),
            Some("Jazz".to_string()),
            vec!["jazz".to_string()],
            vec![],
            MixerPresetMaster::default(),
        );

        // Search by name
        let results = state.search_mixer_presets("rock");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Rock Mix");

        // Search by description
        let results = state.search_mixer_presets("smooth");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Jazz Mix");

        // Search by tag
        let results = state.search_mixer_presets("heavy");
        assert_eq!(results.len(), 1);

        // Case insensitive
        let results = state.search_mixer_presets("JAZZ");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_list_mixer_presets_by_category() {
        let state = PresetsState::new_empty();

        state.create_mixer_preset(
            "Rock 1".to_string(),
            None,
            Some("Rock".to_string()),
            vec![],
            vec![],
            MixerPresetMaster::default(),
        );

        state.create_mixer_preset(
            "Rock 2".to_string(),
            None,
            Some("Rock".to_string()),
            vec![],
            vec![],
            MixerPresetMaster::default(),
        );

        state.create_mixer_preset(
            "Jazz 1".to_string(),
            None,
            Some("Jazz".to_string()),
            vec![],
            vec![],
            MixerPresetMaster::default(),
        );

        let rock_presets = state.list_mixer_presets_by_category("Rock");
        assert_eq!(rock_presets.len(), 2);

        let jazz_presets = state.list_mixer_presets_by_category("Jazz");
        assert_eq!(jazz_presets.len(), 1);

        let none_presets = state.list_mixer_presets_by_category("Classical");
        assert!(none_presets.is_empty());
    }

    #[test]
    fn test_update_mixer_preset() {
        let state = PresetsState::new_empty();

        state.create_mixer_preset(
            "Original".to_string(),
            None,
            None,
            vec![],
            vec![],
            MixerPresetMaster::default(),
        );

        let updated = state
            .update_mixer_preset(
                1,
                Some("Updated Name".to_string()),
                Some("New description".to_string()),
                Some("New Category".to_string()),
            )
            .unwrap();

        assert!(updated.is_some());
        let preset = updated.unwrap();
        assert_eq!(preset.name, "Updated Name");
        assert_eq!(preset.description, Some("New description".to_string()));
        assert_eq!(preset.category, Some("New Category".to_string()));
    }

    #[test]
    fn test_cannot_update_factory_preset() {
        let state = PresetsState::default();

        // First preset (id=1) is a factory preset
        let result = state.update_mixer_preset(1, Some("New Name".to_string()), None, None);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot modify factory presets");
    }

    #[test]
    fn test_delete_mixer_preset() {
        let state = PresetsState::new_empty();

        state.create_mixer_preset(
            "To Delete".to_string(),
            None,
            None,
            vec![],
            vec![],
            MixerPresetMaster::default(),
        );

        assert_eq!(state.list_mixer_presets().len(), 1);

        let deleted = state.delete_mixer_preset(1).unwrap();
        assert!(deleted);
        assert!(state.list_mixer_presets().is_empty());

        // Delete non-existent
        let deleted = state.delete_mixer_preset(999).unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_cannot_delete_factory_preset() {
        let state = PresetsState::default();

        let result = state.delete_mixer_preset(1);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Cannot delete factory presets");
    }

    // -------------------------------------------------------------------------
    // Track Template Operations Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_create_track_template() {
        let state = PresetsState::new_empty();

        let template = state.create_track_template(
            "Synth Lead".to_string(),
            Some("Lead synth track".to_string()),
            Some("Synth".to_string()),
            vec!["synth".to_string(), "lead".to_string()],
            "midi".to_string(),
            Some("#ff6600".to_string()),
            MixerPresetChannel::default(),
            TrackMidiRouting {
                output_channel: Some(5),
                program_change: Some(80),
                ..Default::default()
            },
            Some(1),
            Some("Lead Synth".to_string()),
        );

        assert_eq!(template.id, 1);
        assert_eq!(template.name, "Synth Lead");
        assert_eq!(template.track_type, "midi");
        assert_eq!(template.color, Some("#ff6600".to_string()));
        assert_eq!(template.midi_routing.output_channel, Some(5));
        assert_eq!(template.midi_routing.program_change, Some(80));
        assert_eq!(template.gear_profile_id, Some(1));
        assert!(!template.is_factory);
    }

    #[test]
    fn test_get_track_template() {
        let state = PresetsState::new_empty();

        state.create_track_template(
            "Template 1".to_string(),
            None,
            None,
            vec![],
            "midi".to_string(),
            None,
            MixerPresetChannel::default(),
            TrackMidiRouting::default(),
            None,
            None,
        );

        let found = state.get_track_template(1);
        assert!(found.is_some());

        let not_found = state.get_track_template(999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_search_track_templates() {
        let state = PresetsState::new_empty();

        state.create_track_template(
            "Drum Kit".to_string(),
            Some("Standard drum kit".to_string()),
            Some("Drums".to_string()),
            vec!["drums".to_string(), "percussion".to_string()],
            "midi".to_string(),
            None,
            MixerPresetChannel::default(),
            TrackMidiRouting::default(),
            None,
            None,
        );

        state.create_track_template(
            "Bass Synth".to_string(),
            Some("Fat bass synth".to_string()),
            Some("Bass".to_string()),
            vec!["bass".to_string(), "synth".to_string()],
            "midi".to_string(),
            None,
            MixerPresetChannel::default(),
            TrackMidiRouting::default(),
            None,
            None,
        );

        // Search by name
        let results = state.search_track_templates("drum");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Drum Kit");

        // Search by tag
        let results = state.search_track_templates("synth");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Bass Synth");

        // Search by description
        let results = state.search_track_templates("fat");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_list_track_templates_by_category() {
        let state = PresetsState::new_empty();

        state.create_track_template(
            "Drums 1".to_string(),
            None,
            Some("Drums".to_string()),
            vec![],
            "midi".to_string(),
            None,
            MixerPresetChannel::default(),
            TrackMidiRouting::default(),
            None,
            None,
        );

        state.create_track_template(
            "Bass 1".to_string(),
            None,
            Some("Bass".to_string()),
            vec![],
            "midi".to_string(),
            None,
            MixerPresetChannel::default(),
            TrackMidiRouting::default(),
            None,
            None,
        );

        let drums = state.list_track_templates_by_category("Drums");
        assert_eq!(drums.len(), 1);
        assert_eq!(drums[0].name, "Drums 1");
    }

    #[test]
    fn test_list_track_templates_by_type() {
        let state = PresetsState::new_empty();

        state.create_track_template(
            "MIDI 1".to_string(),
            None,
            None,
            vec![],
            "midi".to_string(),
            None,
            MixerPresetChannel::default(),
            TrackMidiRouting::default(),
            None,
            None,
        );

        state.create_track_template(
            "Audio 1".to_string(),
            None,
            None,
            vec![],
            "audio".to_string(),
            None,
            MixerPresetChannel::default(),
            TrackMidiRouting::default(),
            None,
            None,
        );

        let midi = state.list_track_templates_by_type("midi");
        assert_eq!(midi.len(), 1);

        let audio = state.list_track_templates_by_type("audio");
        assert_eq!(audio.len(), 1);
    }

    #[test]
    fn test_delete_track_template() {
        let state = PresetsState::new_empty();

        state.create_track_template(
            "To Delete".to_string(),
            None,
            None,
            vec![],
            "midi".to_string(),
            None,
            MixerPresetChannel::default(),
            TrackMidiRouting::default(),
            None,
            None,
        );

        let deleted = state.delete_track_template(1).unwrap();
        assert!(deleted);
        assert!(state.list_track_templates().is_empty());
    }

    #[test]
    fn test_cannot_delete_factory_track_template() {
        let state = PresetsState::default();

        // First template (id=1) is factory
        let result = state.delete_track_template(1);
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // Project Template Operations Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_create_project_template() {
        let state = PresetsState::new_empty();

        let tracks = vec![
            ProjectTemplateTrack {
                name: "Lead".to_string(),
                track_type: "midi".to_string(),
                color: Some("#ff0000".to_string()),
                mixer_settings: MixerPresetChannel::default(),
                midi_routing: TrackMidiRouting::default(),
                gear_profile_id: None,
            },
            ProjectTemplateTrack {
                name: "Bass".to_string(),
                track_type: "midi".to_string(),
                color: Some("#0000ff".to_string()),
                mixer_settings: MixerPresetChannel::default(),
                midi_routing: TrackMidiRouting::default(),
                gear_profile_id: None,
            },
        ];

        let template = state.create_project_template(
            "Duo Setup".to_string(),
            Some("Lead and bass duo".to_string()),
            Some("Duo".to_string()),
            vec!["duo".to_string()],
            MixerPresetMaster { tempo: 130.0, ..Default::default() },
            tracks,
            "4/4".to_string(),
            130.0,
            Some("Am".to_string()),
        );

        assert_eq!(template.id, 1);
        assert_eq!(template.name, "Duo Setup");
        assert_eq!(template.tracks.len(), 2);
        assert_eq!(template.default_tempo, 130.0);
        assert_eq!(template.default_key, Some("Am".to_string()));
        assert!(!template.is_factory);
    }

    #[test]
    fn test_get_project_template() {
        let state = PresetsState::new_empty();

        state.create_project_template(
            "Project 1".to_string(),
            None,
            None,
            vec![],
            MixerPresetMaster::default(),
            vec![],
            "4/4".to_string(),
            120.0,
            None,
        );

        let found = state.get_project_template(1);
        assert!(found.is_some());

        let not_found = state.get_project_template(999);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_search_project_templates() {
        let state = PresetsState::new_empty();

        state.create_project_template(
            "Rock Band".to_string(),
            Some("Full rock band setup".to_string()),
            Some("Rock".to_string()),
            vec!["rock".to_string(), "band".to_string()],
            MixerPresetMaster::default(),
            vec![],
            "4/4".to_string(),
            120.0,
            None,
        );

        state.create_project_template(
            "Electronic".to_string(),
            Some("EDM production template".to_string()),
            Some("Electronic".to_string()),
            vec!["edm".to_string(), "electronic".to_string()],
            MixerPresetMaster::default(),
            vec![],
            "4/4".to_string(),
            128.0,
            None,
        );

        // Search by name
        let results = state.search_project_templates("rock");
        assert_eq!(results.len(), 1);

        // Search by tag
        let results = state.search_project_templates("edm");
        assert_eq!(results.len(), 1);

        // Search by description
        let results = state.search_project_templates("production");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_list_project_templates_by_category() {
        let state = PresetsState::new_empty();

        state.create_project_template(
            "Rock 1".to_string(),
            None,
            Some("Rock".to_string()),
            vec![],
            MixerPresetMaster::default(),
            vec![],
            "4/4".to_string(),
            120.0,
            None,
        );

        state.create_project_template(
            "Jazz 1".to_string(),
            None,
            Some("Jazz".to_string()),
            vec![],
            MixerPresetMaster::default(),
            vec![],
            "4/4".to_string(),
            120.0,
            None,
        );

        let rock = state.list_project_templates_by_category("Rock");
        assert_eq!(rock.len(), 1);
    }

    #[test]
    fn test_delete_project_template() {
        let state = PresetsState::new_empty();

        state.create_project_template(
            "To Delete".to_string(),
            None,
            None,
            vec![],
            MixerPresetMaster::default(),
            vec![],
            "4/4".to_string(),
            120.0,
            None,
        );

        let deleted = state.delete_project_template(1).unwrap();
        assert!(deleted);
        assert!(state.list_project_templates().is_empty());
    }

    #[test]
    fn test_cannot_delete_factory_project_template() {
        let state = PresetsState::default();

        // First template (id=1) is factory
        let result = state.delete_project_template(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_project_template() {
        let state = PresetsState::new_empty();

        state.create_project_template(
            "Original Project".to_string(),
            Some("Original description".to_string()),
            Some("Test".to_string()),
            vec!["test".to_string()],
            MixerPresetMaster::default(),
            vec![],
            "4/4".to_string(),
            120.0,
            Some("C".to_string()),
        );

        // Duplicate with auto-generated name
        let dup1 = state.duplicate_project_template(1, None);
        assert!(dup1.is_some());
        let dup1 = dup1.unwrap();
        assert_eq!(dup1.id, 2);
        assert_eq!(dup1.name, "Original Project (Copy)");
        assert!(!dup1.is_factory);

        // Duplicate with custom name
        let dup2 = state.duplicate_project_template(1, Some("Custom Name".to_string()));
        assert!(dup2.is_some());
        let dup2 = dup2.unwrap();
        assert_eq!(dup2.name, "Custom Name");

        // Verify we now have 3 templates
        assert_eq!(state.list_project_templates().len(), 3);
    }

    #[test]
    fn test_duplicate_nonexistent_template() {
        let state = PresetsState::new_empty();

        let result = state.duplicate_project_template(999, None);
        assert!(result.is_none());
    }

    // -------------------------------------------------------------------------
    // ID Generation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mixer_preset_id_generation() {
        let state = PresetsState::new_empty();

        let p1 = state.create_mixer_preset(
            "P1".to_string(),
            None,
            None,
            vec![],
            vec![],
            MixerPresetMaster::default(),
        );
        let p2 = state.create_mixer_preset(
            "P2".to_string(),
            None,
            None,
            vec![],
            vec![],
            MixerPresetMaster::default(),
        );
        let p3 = state.create_mixer_preset(
            "P3".to_string(),
            None,
            None,
            vec![],
            vec![],
            MixerPresetMaster::default(),
        );

        assert_eq!(p1.id, 1);
        assert_eq!(p2.id, 2);
        assert_eq!(p3.id, 3);
    }

    #[test]
    fn test_track_template_id_generation() {
        let state = PresetsState::new_empty();

        let t1 = state.create_track_template(
            "T1".to_string(),
            None,
            None,
            vec![],
            "midi".to_string(),
            None,
            MixerPresetChannel::default(),
            TrackMidiRouting::default(),
            None,
            None,
        );
        let t2 = state.create_track_template(
            "T2".to_string(),
            None,
            None,
            vec![],
            "midi".to_string(),
            None,
            MixerPresetChannel::default(),
            TrackMidiRouting::default(),
            None,
            None,
        );

        assert_eq!(t1.id, 1);
        assert_eq!(t2.id, 2);
    }

    #[test]
    fn test_project_template_id_generation() {
        let state = PresetsState::new_empty();

        let t1 = state.create_project_template(
            "P1".to_string(),
            None,
            None,
            vec![],
            MixerPresetMaster::default(),
            vec![],
            "4/4".to_string(),
            120.0,
            None,
        );
        let t2 = state.create_project_template(
            "P2".to_string(),
            None,
            None,
            vec![],
            MixerPresetMaster::default(),
            vec![],
            "4/4".to_string(),
            120.0,
            None,
        );

        assert_eq!(t1.id, 1);
        assert_eq!(t2.id, 2);
    }

    // -------------------------------------------------------------------------
    // Data Integrity Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mixer_preset_with_all_fields() {
        let mut channel = MixerPresetChannel::default();
        channel.volume = 80;
        channel.pan = 32; // Left
        channel.muted = true;
        channel.transpose = -2;
        channel.octave_shift = 1;
        channel.velocity = 90;
        channel.velocity_min = 20;
        channel.velocity_max = 110;
        channel.pitch_bend = 100;
        channel.pitch_bend_range = 12;
        channel.note_low = 36;
        channel.note_high = 96;
        channel.quantize_grid = Some("1/16".to_string());
        channel.swing_amount = 30;
        channel.delay_ticks = -10;
        channel.humanize_timing = 5;
        channel.humanize_velocity = 10;
        channel.cc_values.insert(1, 64);

        let master = MixerPresetMaster {
            volume: 90,
            pan: 64,
            tempo: 145.0,
            time_signature_numerator: 7,
            time_signature_denominator: 8,
            key_signature: Some("Bb".to_string()),
            swing_amount: 20,
            metronome_enabled: true,
            metronome_volume: 70,
        };

        let state = PresetsState::new_empty();
        let preset = state.create_mixer_preset(
            "Complex Preset".to_string(),
            Some("Full preset".to_string()),
            Some("Complex".to_string()),
            vec!["complex".to_string(), "test".to_string()],
            vec![channel.clone()],
            master.clone(),
        );

        // Verify all data persisted
        let retrieved = state.get_mixer_preset(preset.id).unwrap();
        assert_eq!(retrieved.channels.len(), 1);
        assert_eq!(retrieved.channels[0].volume, 80);
        assert_eq!(retrieved.channels[0].transpose, -2);
        assert_eq!(
            retrieved.channels[0].quantize_grid,
            Some("1/16".to_string())
        );
        assert_eq!(retrieved.master.tempo, 145.0);
        assert_eq!(retrieved.master.time_signature_numerator, 7);
    }

    #[test]
    fn test_basic_band_template_has_correct_tracks() {
        let state = PresetsState::default();

        // Get the "Basic Band" template (id=2)
        let template = state.get_project_template(2).unwrap();

        assert_eq!(template.name, "Basic Band");
        assert_eq!(template.tracks.len(), 4);

        // Verify track names
        let track_names: Vec<&str> = template.tracks.iter().map(|t| t.name.as_str()).collect();
        assert!(track_names.contains(&"Drums"));
        assert!(track_names.contains(&"Bass"));
        assert!(track_names.contains(&"Keys"));
        assert!(track_names.contains(&"Lead"));

        // Verify drums is on channel 10
        let drums = template.tracks.iter().find(|t| t.name == "Drums").unwrap();
        assert_eq!(drums.midi_routing.output_channel, Some(10));

        // Verify bass has octave shift
        let bass = template.tracks.iter().find(|t| t.name == "Bass").unwrap();
        assert_eq!(bass.mixer_settings.octave_shift, -1);
    }
}
