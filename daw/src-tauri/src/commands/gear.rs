//! Gear Commands - Gear Profiles, CC Mappings, Programs, User Gear
//!
//! This module provides Tauri commands for managing external gear:
//! - Gear profiles (manufacturer, model, capabilities)
//! - CC mappings (control change parameter definitions)
//! - Program/preset banks
//! - User's owned gear collection
#![allow(dead_code)] // Commands are called externally via Tauri IPC

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{command, State};

// =============================================================================
// GEAR PROFILES
// =============================================================================

/// Gear profile defining a MIDI device's capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GearProfile {
    pub id: u64,
    pub name: String,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub device_type: String, // "synth", "drum_machine", "sampler", "controller", "effect", "other"
    pub midi_in_channels: u8,  // 1-16
    pub midi_out_channels: u8, // 1-16
    pub supports_sysex: bool,
    pub supports_nrpn: bool,
    pub supports_rpn: bool,
    pub supports_mpe: bool,
    pub mpe_zones: Option<u8>,
    pub polyphony: Option<u16>,
    pub multitimbral_parts: Option<u8>,
    pub notes: Option<String>,
    pub icon_path: Option<String>,
    pub is_factory: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for GearProfile {
    fn default() -> Self {
        Self {
            id: 0,
            name: "Generic MIDI Device".to_string(),
            manufacturer: None,
            model: None,
            device_type: "synth".to_string(),
            midi_in_channels: 16,
            midi_out_channels: 16,
            supports_sysex: false,
            supports_nrpn: false,
            supports_rpn: true,
            supports_mpe: false,
            mpe_zones: None,
            polyphony: None,
            multitimbral_parts: None,
            notes: None,
            icon_path: None,
            is_factory: false,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

// =============================================================================
// CC MAPPINGS
// =============================================================================

/// CC parameter mapping for a gear profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GearCCMapping {
    pub id: u64,
    pub profile_id: u64,
    pub cc_number: u8, // 0-127
    pub name: String,
    pub description: Option<String>,
    pub min_value: u8,
    pub max_value: u8,
    pub default_value: Option<u8>,
    pub is_continuous: bool, // vs. switch/toggle
    pub value_labels: Option<HashMap<u8, String>>, // For discrete values
}

impl Default for GearCCMapping {
    fn default() -> Self {
        Self {
            id: 0,
            profile_id: 0,
            cc_number: 0,
            name: "Unknown".to_string(),
            description: None,
            min_value: 0,
            max_value: 127,
            default_value: Some(64),
            is_continuous: true,
            value_labels: None,
        }
    }
}

// =============================================================================
// PROGRAMS/PRESETS
// =============================================================================

/// Program/preset for a gear profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GearProgram {
    pub id: u64,
    pub profile_id: u64,
    pub bank_msb: u8,
    pub bank_lsb: u8,
    pub program_number: u8, // 0-127
    pub name: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

impl Default for GearProgram {
    fn default() -> Self {
        Self {
            id: 0,
            profile_id: 0,
            bank_msb: 0,
            bank_lsb: 0,
            program_number: 0,
            name: "Init".to_string(),
            category: None,
            tags: Vec::new(),
            notes: None,
        }
    }
}

// =============================================================================
// USER GEAR
// =============================================================================

/// User's instance of a gear profile with custom settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGear {
    pub id: u64,
    pub profile_id: u64,
    pub custom_name: Option<String>,
    pub midi_port_in: Option<String>,
    pub midi_port_out: Option<String>,
    pub midi_channel: Option<u8>, // 1-16, None for omni
    pub color: Option<String>,
    pub is_favorite: bool,
    pub notes: Option<String>,
    pub last_used: Option<String>,
    pub custom_cc_overrides: Option<Vec<GearCCMapping>>,
}

impl Default for UserGear {
    fn default() -> Self {
        Self {
            id: 0,
            profile_id: 0,
            custom_name: None,
            midi_port_in: None,
            midi_port_out: None,
            midi_channel: None,
            color: None,
            is_favorite: false,
            notes: None,
            last_used: None,
            custom_cc_overrides: None,
        }
    }
}

// =============================================================================
// STATE
// =============================================================================

/// Gear management state
pub struct GearState {
    pub profiles: Mutex<Vec<GearProfile>>,
    pub cc_mappings: Mutex<Vec<GearCCMapping>>,
    pub programs: Mutex<Vec<GearProgram>>,
    pub user_gear: Mutex<Vec<UserGear>>,
    next_profile_id: Mutex<u64>,
    next_cc_id: Mutex<u64>,
    next_program_id: Mutex<u64>,
    next_user_gear_id: Mutex<u64>,
}

impl Default for GearState {
    fn default() -> Self {
        // Create some factory profiles
        let profiles = vec![
            // Generic GM
            GearProfile {
                id: 1,
                name: "General MIDI".to_string(),
                manufacturer: None,
                model: None,
                device_type: "synth".to_string(),
                midi_in_channels: 16,
                midi_out_channels: 16,
                supports_sysex: true,
                supports_nrpn: false,
                supports_rpn: true,
                supports_mpe: false,
                mpe_zones: None,
                polyphony: Some(128),
                multitimbral_parts: Some(16),
                notes: Some("Standard General MIDI compatible device".to_string()),
                icon_path: None,
                is_factory: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
            // Akai MPC
            GearProfile {
                id: 2,
                name: "Akai MPC One".to_string(),
                manufacturer: Some("Akai Professional".to_string()),
                model: Some("MPC One".to_string()),
                device_type: "sampler".to_string(),
                midi_in_channels: 16,
                midi_out_channels: 16,
                supports_sysex: true,
                supports_nrpn: true,
                supports_rpn: true,
                supports_mpe: false,
                mpe_zones: None,
                polyphony: Some(128),
                multitimbral_parts: Some(128),
                notes: Some("Standalone sampler/sequencer workstation".to_string()),
                icon_path: None,
                is_factory: true,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        ];

        // Standard CC mappings for General MIDI
        let cc_mappings = vec![
            GearCCMapping {
                id: 1,
                profile_id: 1, // GM
                cc_number: 1,
                name: "Modulation".to_string(),
                description: Some("Modulation wheel".to_string()),
                min_value: 0,
                max_value: 127,
                default_value: Some(0),
                is_continuous: true,
                value_labels: None,
            },
            GearCCMapping {
                id: 2,
                profile_id: 1,
                cc_number: 7,
                name: "Volume".to_string(),
                description: Some("Channel volume".to_string()),
                min_value: 0,
                max_value: 127,
                default_value: Some(100),
                is_continuous: true,
                value_labels: None,
            },
            GearCCMapping {
                id: 3,
                profile_id: 1,
                cc_number: 10,
                name: "Pan".to_string(),
                description: Some("Stereo panning".to_string()),
                min_value: 0,
                max_value: 127,
                default_value: Some(64),
                is_continuous: true,
                value_labels: None,
            },
            GearCCMapping {
                id: 4,
                profile_id: 1,
                cc_number: 11,
                name: "Expression".to_string(),
                description: Some("Expression controller".to_string()),
                min_value: 0,
                max_value: 127,
                default_value: Some(127),
                is_continuous: true,
                value_labels: None,
            },
            GearCCMapping {
                id: 5,
                profile_id: 1,
                cc_number: 64,
                name: "Sustain".to_string(),
                description: Some("Sustain pedal".to_string()),
                min_value: 0,
                max_value: 127,
                default_value: Some(0),
                is_continuous: false,
                value_labels: Some(HashMap::from([(0, "Off".to_string()), (127, "On".to_string())])),
            },
            GearCCMapping {
                id: 6,
                profile_id: 1,
                cc_number: 91,
                name: "Reverb".to_string(),
                description: Some("Reverb send level".to_string()),
                min_value: 0,
                max_value: 127,
                default_value: Some(40),
                is_continuous: true,
                value_labels: None,
            },
            GearCCMapping {
                id: 7,
                profile_id: 1,
                cc_number: 93,
                name: "Chorus".to_string(),
                description: Some("Chorus send level".to_string()),
                min_value: 0,
                max_value: 127,
                default_value: Some(0),
                is_continuous: true,
                value_labels: None,
            },
        ];

        Self {
            profiles: Mutex::new(profiles),
            cc_mappings: Mutex::new(cc_mappings),
            programs: Mutex::new(Vec::new()),
            user_gear: Mutex::new(Vec::new()),
            next_profile_id: Mutex::new(3),
            next_cc_id: Mutex::new(8),
            next_program_id: Mutex::new(1),
            next_user_gear_id: Mutex::new(1),
        }
    }
}

// =============================================================================
// GEAR PROFILE COMMANDS
// =============================================================================

/// List all gear profiles
#[command]
pub async fn gear_profiles_list(state: State<'_, GearState>) -> Result<Vec<GearProfile>, String> {
    let profiles = state.profiles.lock().unwrap();
    Ok(profiles.clone())
}

/// List profiles by device type
#[command]
pub async fn gear_profiles_list_by_type(
    state: State<'_, GearState>,
    device_type: String,
) -> Result<Vec<GearProfile>, String> {
    let profiles = state.profiles.lock().unwrap();
    Ok(profiles
        .iter()
        .filter(|p| p.device_type == device_type)
        .cloned()
        .collect())
}

/// Get a profile by ID
#[command]
pub async fn gear_profiles_get(
    state: State<'_, GearState>,
    id: u64,
) -> Result<Option<GearProfile>, String> {
    let profiles = state.profiles.lock().unwrap();
    Ok(profiles.iter().find(|p| p.id == id).cloned())
}

/// Search profiles by name
#[command]
pub async fn gear_profiles_search(
    state: State<'_, GearState>,
    query: String,
) -> Result<Vec<GearProfile>, String> {
    let profiles = state.profiles.lock().unwrap();
    let query_lower = query.to_lowercase();
    Ok(profiles
        .iter()
        .filter(|p| {
            p.name.to_lowercase().contains(&query_lower)
                || p.manufacturer
                    .as_ref()
                    .map(|m| m.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || p.model
                    .as_ref()
                    .map(|m| m.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
        })
        .cloned()
        .collect())
}

/// Create a new gear profile
#[command]
pub async fn gear_profiles_create(
    state: State<'_, GearState>,
    name: String,
    manufacturer: Option<String>,
    model: Option<String>,
    device_type: String,
    supports_sysex: Option<bool>,
    supports_nrpn: Option<bool>,
    supports_mpe: Option<bool>,
    polyphony: Option<u16>,
    multitimbral_parts: Option<u8>,
    notes: Option<String>,
) -> Result<GearProfile, String> {
    let mut profiles = state.profiles.lock().unwrap();
    let mut next_id = state.next_profile_id.lock().unwrap();

    let profile = GearProfile {
        id: *next_id,
        name,
        manufacturer,
        model,
        device_type,
        midi_in_channels: 16,
        midi_out_channels: 16,
        supports_sysex: supports_sysex.unwrap_or(false),
        supports_nrpn: supports_nrpn.unwrap_or(false),
        supports_rpn: true,
        supports_mpe: supports_mpe.unwrap_or(false),
        mpe_zones: None,
        polyphony,
        multitimbral_parts,
        notes,
        icon_path: None,
        is_factory: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    *next_id += 1;
    profiles.push(profile.clone());
    Ok(profile)
}

/// Update a gear profile
#[command]
pub async fn gear_profiles_update(
    state: State<'_, GearState>,
    id: u64,
    updates: HashMap<String, serde_json::Value>,
) -> Result<Option<GearProfile>, String> {
    let mut profiles = state.profiles.lock().unwrap();

    if let Some(profile) = profiles.iter_mut().find(|p| p.id == id) {
        // Don't allow modifying factory profiles
        if profile.is_factory {
            return Err("Cannot modify factory profiles".to_string());
        }

        if let Some(v) = updates.get("name") {
            if let Some(s) = v.as_str() {
                profile.name = s.to_string();
            }
        }
        if let Some(v) = updates.get("manufacturer") {
            profile.manufacturer = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = updates.get("model") {
            profile.model = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = updates.get("device_type") {
            if let Some(s) = v.as_str() {
                profile.device_type = s.to_string();
            }
        }
        if let Some(v) = updates.get("supports_sysex") {
            if let Some(b) = v.as_bool() {
                profile.supports_sysex = b;
            }
        }
        if let Some(v) = updates.get("supports_nrpn") {
            if let Some(b) = v.as_bool() {
                profile.supports_nrpn = b;
            }
        }
        if let Some(v) = updates.get("supports_mpe") {
            if let Some(b) = v.as_bool() {
                profile.supports_mpe = b;
            }
        }
        if let Some(v) = updates.get("polyphony") {
            profile.polyphony = v.as_u64().map(|n| n as u16);
        }
        if let Some(v) = updates.get("multitimbral_parts") {
            profile.multitimbral_parts = v.as_u64().map(|n| n as u8);
        }
        if let Some(v) = updates.get("notes") {
            profile.notes = v.as_str().map(|s| s.to_string());
        }

        profile.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(Some(profile.clone()))
    } else {
        Ok(None)
    }
}

/// Delete a gear profile
#[command]
pub async fn gear_profiles_delete(state: State<'_, GearState>, id: u64) -> Result<bool, String> {
    let mut profiles = state.profiles.lock().unwrap();

    // Don't delete factory profiles
    if profiles.iter().any(|p| p.id == id && p.is_factory) {
        return Err("Cannot delete factory profiles".to_string());
    }

    let len_before = profiles.len();
    profiles.retain(|p| p.id != id);

    // Also delete associated CC mappings, programs, and user gear
    if profiles.len() < len_before {
        let mut cc = state.cc_mappings.lock().unwrap();
        cc.retain(|m| m.profile_id != id);

        let mut programs = state.programs.lock().unwrap();
        programs.retain(|p| p.profile_id != id);

        let mut user_gear = state.user_gear.lock().unwrap();
        user_gear.retain(|g| g.profile_id != id);
    }

    Ok(profiles.len() < len_before)
}

// =============================================================================
// CC MAPPING COMMANDS
// =============================================================================

/// List CC mappings for a profile
#[command]
pub async fn gear_cc_list(
    state: State<'_, GearState>,
    profile_id: u64,
) -> Result<Vec<GearCCMapping>, String> {
    let cc_mappings = state.cc_mappings.lock().unwrap();
    Ok(cc_mappings
        .iter()
        .filter(|m| m.profile_id == profile_id)
        .cloned()
        .collect())
}

/// Get a CC mapping by ID
#[command]
pub async fn gear_cc_get(
    state: State<'_, GearState>,
    id: u64,
) -> Result<Option<GearCCMapping>, String> {
    let cc_mappings = state.cc_mappings.lock().unwrap();
    Ok(cc_mappings.iter().find(|m| m.id == id).cloned())
}

/// Get CC mapping by profile and CC number
#[command]
pub async fn gear_cc_get_by_number(
    state: State<'_, GearState>,
    profile_id: u64,
    cc_number: u8,
) -> Result<Option<GearCCMapping>, String> {
    let cc_mappings = state.cc_mappings.lock().unwrap();
    Ok(cc_mappings
        .iter()
        .find(|m| m.profile_id == profile_id && m.cc_number == cc_number)
        .cloned())
}

/// Create a CC mapping
#[command]
pub async fn gear_cc_create(
    state: State<'_, GearState>,
    profile_id: u64,
    cc_number: u8,
    name: String,
    description: Option<String>,
    min_value: Option<u8>,
    max_value: Option<u8>,
    default_value: Option<u8>,
    is_continuous: Option<bool>,
) -> Result<GearCCMapping, String> {
    // Check profile exists
    {
        let profiles = state.profiles.lock().unwrap();
        if !profiles.iter().any(|p| p.id == profile_id) {
            return Err("Profile not found".to_string());
        }
    }

    let mut cc_mappings = state.cc_mappings.lock().unwrap();
    let mut next_id = state.next_cc_id.lock().unwrap();

    // Check for duplicate CC number
    if cc_mappings
        .iter()
        .any(|m| m.profile_id == profile_id && m.cc_number == cc_number)
    {
        return Err(format!(
            "CC {} already mapped for this profile",
            cc_number
        ));
    }

    let mapping = GearCCMapping {
        id: *next_id,
        profile_id,
        cc_number,
        name,
        description,
        min_value: min_value.unwrap_or(0),
        max_value: max_value.unwrap_or(127),
        default_value,
        is_continuous: is_continuous.unwrap_or(true),
        value_labels: None,
    };

    *next_id += 1;
    cc_mappings.push(mapping.clone());
    Ok(mapping)
}

/// Update a CC mapping
#[command]
pub async fn gear_cc_update(
    state: State<'_, GearState>,
    id: u64,
    name: Option<String>,
    description: Option<String>,
    min_value: Option<u8>,
    max_value: Option<u8>,
    default_value: Option<u8>,
    is_continuous: Option<bool>,
) -> Result<Option<GearCCMapping>, String> {
    let mut cc_mappings = state.cc_mappings.lock().unwrap();

    if let Some(mapping) = cc_mappings.iter_mut().find(|m| m.id == id) {
        if let Some(n) = name {
            mapping.name = n;
        }
        if let Some(d) = description {
            mapping.description = Some(d);
        }
        if let Some(v) = min_value {
            mapping.min_value = v;
        }
        if let Some(v) = max_value {
            mapping.max_value = v;
        }
        if let Some(v) = default_value {
            mapping.default_value = Some(v);
        }
        if let Some(c) = is_continuous {
            mapping.is_continuous = c;
        }
        Ok(Some(mapping.clone()))
    } else {
        Ok(None)
    }
}

/// Delete a CC mapping
#[command]
pub async fn gear_cc_delete(state: State<'_, GearState>, id: u64) -> Result<bool, String> {
    let mut cc_mappings = state.cc_mappings.lock().unwrap();
    let len_before = cc_mappings.len();
    cc_mappings.retain(|m| m.id != id);
    Ok(cc_mappings.len() < len_before)
}

// =============================================================================
// PROGRAM COMMANDS
// =============================================================================

/// List programs for a profile
#[command]
pub async fn gear_programs_list(
    state: State<'_, GearState>,
    profile_id: u64,
) -> Result<Vec<GearProgram>, String> {
    let programs = state.programs.lock().unwrap();
    Ok(programs
        .iter()
        .filter(|p| p.profile_id == profile_id)
        .cloned()
        .collect())
}

/// List programs by bank
#[command]
pub async fn gear_programs_list_by_bank(
    state: State<'_, GearState>,
    profile_id: u64,
    bank_msb: u8,
    bank_lsb: u8,
) -> Result<Vec<GearProgram>, String> {
    let programs = state.programs.lock().unwrap();
    Ok(programs
        .iter()
        .filter(|p| p.profile_id == profile_id && p.bank_msb == bank_msb && p.bank_lsb == bank_lsb)
        .cloned()
        .collect())
}

/// Get a program by ID
#[command]
pub async fn gear_programs_get(
    state: State<'_, GearState>,
    id: u64,
) -> Result<Option<GearProgram>, String> {
    let programs = state.programs.lock().unwrap();
    Ok(programs.iter().find(|p| p.id == id).cloned())
}

/// Search programs by name
#[command]
pub async fn gear_programs_search(
    state: State<'_, GearState>,
    profile_id: u64,
    query: String,
) -> Result<Vec<GearProgram>, String> {
    let programs = state.programs.lock().unwrap();
    let query_lower = query.to_lowercase();
    Ok(programs
        .iter()
        .filter(|p| {
            p.profile_id == profile_id
                && (p.name.to_lowercase().contains(&query_lower)
                    || p.category
                        .as_ref()
                        .map(|c| c.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&query_lower)))
        })
        .cloned()
        .collect())
}

/// Create a program
#[command]
pub async fn gear_programs_create(
    state: State<'_, GearState>,
    profile_id: u64,
    bank_msb: u8,
    bank_lsb: u8,
    program_number: u8,
    name: String,
    category: Option<String>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
) -> Result<GearProgram, String> {
    let mut programs = state.programs.lock().unwrap();
    let mut next_id = state.next_program_id.lock().unwrap();

    let program = GearProgram {
        id: *next_id,
        profile_id,
        bank_msb,
        bank_lsb,
        program_number,
        name,
        category,
        tags: tags.unwrap_or_default(),
        notes,
    };

    *next_id += 1;
    programs.push(program.clone());
    Ok(program)
}

/// Update a program
#[command]
pub async fn gear_programs_update(
    state: State<'_, GearState>,
    id: u64,
    name: Option<String>,
    category: Option<String>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
) -> Result<Option<GearProgram>, String> {
    let mut programs = state.programs.lock().unwrap();

    if let Some(program) = programs.iter_mut().find(|p| p.id == id) {
        if let Some(n) = name {
            program.name = n;
        }
        if let Some(c) = category {
            program.category = Some(c);
        }
        if let Some(t) = tags {
            program.tags = t;
        }
        if let Some(n) = notes {
            program.notes = Some(n);
        }
        Ok(Some(program.clone()))
    } else {
        Ok(None)
    }
}

/// Delete a program
#[command]
pub async fn gear_programs_delete(state: State<'_, GearState>, id: u64) -> Result<bool, String> {
    let mut programs = state.programs.lock().unwrap();
    let len_before = programs.len();
    programs.retain(|p| p.id != id);
    Ok(programs.len() < len_before)
}

// =============================================================================
// USER GEAR COMMANDS
// =============================================================================

/// List user's gear
#[command]
pub async fn user_gear_list(state: State<'_, GearState>) -> Result<Vec<UserGear>, String> {
    let user_gear = state.user_gear.lock().unwrap();
    Ok(user_gear.clone())
}

/// List user's favorite gear
#[command]
pub async fn user_gear_list_favorites(state: State<'_, GearState>) -> Result<Vec<UserGear>, String> {
    let user_gear = state.user_gear.lock().unwrap();
    Ok(user_gear.iter().filter(|g| g.is_favorite).cloned().collect())
}

/// Get user gear by ID
#[command]
pub async fn user_gear_get(
    state: State<'_, GearState>,
    id: u64,
) -> Result<Option<UserGear>, String> {
    let user_gear = state.user_gear.lock().unwrap();
    Ok(user_gear.iter().find(|g| g.id == id).cloned())
}

/// Add gear to user's collection
#[command]
pub async fn user_gear_add(
    state: State<'_, GearState>,
    profile_id: u64,
    custom_name: Option<String>,
    midi_port_in: Option<String>,
    midi_port_out: Option<String>,
    midi_channel: Option<u8>,
    color: Option<String>,
    notes: Option<String>,
) -> Result<UserGear, String> {
    // Check profile exists
    {
        let profiles = state.profiles.lock().unwrap();
        if !profiles.iter().any(|p| p.id == profile_id) {
            return Err("Profile not found".to_string());
        }
    }

    let mut user_gear = state.user_gear.lock().unwrap();
    let mut next_id = state.next_user_gear_id.lock().unwrap();

    let gear = UserGear {
        id: *next_id,
        profile_id,
        custom_name,
        midi_port_in,
        midi_port_out,
        midi_channel,
        color,
        is_favorite: false,
        notes,
        last_used: None,
        custom_cc_overrides: None,
    };

    *next_id += 1;
    user_gear.push(gear.clone());
    Ok(gear)
}

/// Update user gear
#[command]
pub async fn user_gear_update(
    state: State<'_, GearState>,
    id: u64,
    updates: HashMap<String, serde_json::Value>,
) -> Result<Option<UserGear>, String> {
    let mut user_gear = state.user_gear.lock().unwrap();

    if let Some(gear) = user_gear.iter_mut().find(|g| g.id == id) {
        if let Some(v) = updates.get("custom_name") {
            gear.custom_name = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = updates.get("midi_port_in") {
            gear.midi_port_in = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = updates.get("midi_port_out") {
            gear.midi_port_out = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = updates.get("midi_channel") {
            gear.midi_channel = v.as_u64().map(|n| n as u8);
        }
        if let Some(v) = updates.get("color") {
            gear.color = v.as_str().map(|s| s.to_string());
        }
        if let Some(v) = updates.get("is_favorite") {
            if let Some(b) = v.as_bool() {
                gear.is_favorite = b;
            }
        }
        if let Some(v) = updates.get("notes") {
            gear.notes = v.as_str().map(|s| s.to_string());
        }
        Ok(Some(gear.clone()))
    } else {
        Ok(None)
    }
}

/// Set user gear as favorite
#[command]
pub async fn user_gear_set_favorite(
    state: State<'_, GearState>,
    id: u64,
    is_favorite: bool,
) -> Result<Option<UserGear>, String> {
    let mut user_gear = state.user_gear.lock().unwrap();

    if let Some(gear) = user_gear.iter_mut().find(|g| g.id == id) {
        gear.is_favorite = is_favorite;
        Ok(Some(gear.clone()))
    } else {
        Ok(None)
    }
}

/// Update last used timestamp
#[command]
pub async fn user_gear_mark_used(
    state: State<'_, GearState>,
    id: u64,
) -> Result<Option<UserGear>, String> {
    let mut user_gear = state.user_gear.lock().unwrap();

    if let Some(gear) = user_gear.iter_mut().find(|g| g.id == id) {
        gear.last_used = Some(chrono::Utc::now().to_rfc3339());
        Ok(Some(gear.clone()))
    } else {
        Ok(None)
    }
}

/// Remove gear from user's collection
#[command]
pub async fn user_gear_remove(state: State<'_, GearState>, id: u64) -> Result<bool, String> {
    let mut user_gear = state.user_gear.lock().unwrap();
    let len_before = user_gear.len();
    user_gear.retain(|g| g.id != id);
    Ok(user_gear.len() < len_before)
}

/// Get with profile details joined
#[command]
pub async fn user_gear_get_with_profile(
    state: State<'_, GearState>,
    id: u64,
) -> Result<Option<(UserGear, GearProfile)>, String> {
    let user_gear = state.user_gear.lock().unwrap();
    let profiles = state.profiles.lock().unwrap();

    if let Some(gear) = user_gear.iter().find(|g| g.id == id) {
        if let Some(profile) = profiles.iter().find(|p| p.id == gear.profile_id) {
            return Ok(Some((gear.clone(), profile.clone())));
        }
    }
    Ok(None)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Gear Profile Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_gear_profile_default() {
        let profile = GearProfile::default();
        assert_eq!(profile.id, 0);
        assert_eq!(profile.name, "Generic MIDI Device");
        assert_eq!(profile.device_type, "synth");
        assert_eq!(profile.midi_in_channels, 16);
        assert_eq!(profile.midi_out_channels, 16);
        assert!(!profile.supports_sysex);
        assert!(!profile.supports_nrpn);
        assert!(profile.supports_rpn);
        assert!(!profile.supports_mpe);
        assert!(!profile.is_factory);
    }

    #[test]
    fn test_gear_state_default_has_factory_profiles() {
        let state = GearState::default();
        let profiles = state.profiles.lock().unwrap();

        // Should have 2 factory profiles
        assert_eq!(profiles.len(), 2);

        // Check General MIDI profile
        let gm = profiles.iter().find(|p| p.id == 1).unwrap();
        assert_eq!(gm.name, "General MIDI");
        assert!(gm.is_factory);
        assert!(gm.supports_sysex);
        assert_eq!(gm.polyphony, Some(128));

        // Check MPC One profile
        let mpc = profiles.iter().find(|p| p.id == 2).unwrap();
        assert_eq!(mpc.name, "Akai MPC One");
        assert_eq!(mpc.manufacturer, Some("Akai Professional".to_string()));
        assert_eq!(mpc.device_type, "sampler");
        assert!(mpc.is_factory);
    }

    #[test]
    fn test_gear_state_default_has_cc_mappings() {
        let state = GearState::default();
        let cc = state.cc_mappings.lock().unwrap();

        // Should have 7 standard GM CC mappings
        assert_eq!(cc.len(), 7);

        // Check modulation (CC1)
        let mod_cc = cc.iter().find(|m| m.cc_number == 1).unwrap();
        assert_eq!(mod_cc.name, "Modulation");
        assert!(mod_cc.is_continuous);

        // Check sustain (CC64) - not continuous
        let sustain = cc.iter().find(|m| m.cc_number == 64).unwrap();
        assert_eq!(sustain.name, "Sustain");
        assert!(!sustain.is_continuous);
        assert!(sustain.value_labels.is_some());
    }

    #[test]
    fn test_gear_profile_device_types() {
        // Valid device types
        let valid_types = vec!["synth", "drum_machine", "sampler", "controller", "effect", "other"];

        for dt in valid_types {
            let mut profile = GearProfile::default();
            profile.device_type = dt.to_string();
            assert_eq!(profile.device_type, dt);
        }
    }

    #[test]
    fn test_gear_profile_mpe_settings() {
        let mut profile = GearProfile::default();
        profile.supports_mpe = true;
        profile.mpe_zones = Some(2);

        assert!(profile.supports_mpe);
        assert_eq!(profile.mpe_zones, Some(2));
    }

    // -------------------------------------------------------------------------
    // CC Mapping Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cc_mapping_default() {
        let mapping = GearCCMapping::default();
        assert_eq!(mapping.id, 0);
        assert_eq!(mapping.profile_id, 0);
        assert_eq!(mapping.cc_number, 0);
        assert_eq!(mapping.name, "Unknown");
        assert_eq!(mapping.min_value, 0);
        assert_eq!(mapping.max_value, 127);
        assert_eq!(mapping.default_value, Some(64));
        assert!(mapping.is_continuous);
        assert!(mapping.value_labels.is_none());
    }

    #[test]
    fn test_cc_mapping_with_value_labels() {
        let mut mapping = GearCCMapping::default();
        mapping.is_continuous = false;
        mapping.value_labels = Some(HashMap::from([
            (0, "Off".to_string()),
            (64, "Half".to_string()),
            (127, "On".to_string()),
        ]));

        let labels = mapping.value_labels.as_ref().unwrap();
        assert_eq!(labels.get(&0), Some(&"Off".to_string()));
        assert_eq!(labels.get(&127), Some(&"On".to_string()));
    }

    #[test]
    fn test_cc_mapping_range_validation() {
        let mut mapping = GearCCMapping::default();

        // Custom range
        mapping.min_value = 10;
        mapping.max_value = 100;
        mapping.default_value = Some(55);

        assert_eq!(mapping.min_value, 10);
        assert_eq!(mapping.max_value, 100);
        assert_eq!(mapping.default_value, Some(55));
    }

    // -------------------------------------------------------------------------
    // Program Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_gear_program_default() {
        let program = GearProgram::default();
        assert_eq!(program.id, 0);
        assert_eq!(program.profile_id, 0);
        assert_eq!(program.bank_msb, 0);
        assert_eq!(program.bank_lsb, 0);
        assert_eq!(program.program_number, 0);
        assert_eq!(program.name, "Init");
        assert!(program.category.is_none());
        assert!(program.tags.is_empty());
        assert!(program.notes.is_none());
    }

    #[test]
    fn test_gear_program_with_bank_selection() {
        let mut program = GearProgram::default();
        program.bank_msb = 1;
        program.bank_lsb = 32;
        program.program_number = 64;
        program.name = "Electric Piano".to_string();
        program.category = Some("Keys".to_string());

        assert_eq!(program.bank_msb, 1);
        assert_eq!(program.bank_lsb, 32);
        assert_eq!(program.program_number, 64);
    }

    #[test]
    fn test_gear_program_tags() {
        let mut program = GearProgram::default();
        program.tags = vec!["bass".to_string(), "analog".to_string(), "fat".to_string()];

        assert_eq!(program.tags.len(), 3);
        assert!(program.tags.contains(&"bass".to_string()));
        assert!(program.tags.contains(&"analog".to_string()));
    }

    // -------------------------------------------------------------------------
    // User Gear Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_user_gear_default() {
        let gear = UserGear::default();
        assert_eq!(gear.id, 0);
        assert_eq!(gear.profile_id, 0);
        assert!(gear.custom_name.is_none());
        assert!(gear.midi_port_in.is_none());
        assert!(gear.midi_port_out.is_none());
        assert!(gear.midi_channel.is_none());
        assert!(gear.color.is_none());
        assert!(!gear.is_favorite);
        assert!(gear.notes.is_none());
        assert!(gear.last_used.is_none());
        assert!(gear.custom_cc_overrides.is_none());
    }

    #[test]
    fn test_user_gear_with_midi_config() {
        let mut gear = UserGear::default();
        gear.profile_id = 2; // MPC One
        gear.custom_name = Some("My MPC".to_string());
        gear.midi_port_in = Some("MPC One MIDI In".to_string());
        gear.midi_port_out = Some("MPC One MIDI Out".to_string());
        gear.midi_channel = Some(10); // Drums
        gear.color = Some("#FF5500".to_string());

        assert_eq!(gear.midi_channel, Some(10));
        assert_eq!(gear.color, Some("#FF5500".to_string()));
    }

    #[test]
    fn test_user_gear_favorite() {
        let mut gear = UserGear::default();
        assert!(!gear.is_favorite);

        gear.is_favorite = true;
        assert!(gear.is_favorite);
    }

    #[test]
    fn test_user_gear_custom_cc_overrides() {
        let mut gear = UserGear::default();

        let custom_cc = GearCCMapping {
            id: 100,
            profile_id: gear.profile_id,
            cc_number: 74,
            name: "Filter Cutoff".to_string(),
            description: Some("Custom mapping for my synth".to_string()),
            min_value: 0,
            max_value: 127,
            default_value: Some(64),
            is_continuous: true,
            value_labels: None,
        };

        gear.custom_cc_overrides = Some(vec![custom_cc]);

        let overrides = gear.custom_cc_overrides.as_ref().unwrap();
        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0].cc_number, 74);
        assert_eq!(overrides[0].name, "Filter Cutoff");
    }

    // -------------------------------------------------------------------------
    // GearState Operation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_gear_state_next_ids() {
        let state = GearState::default();

        // After 2 factory profiles, next should be 3
        let next_profile = *state.next_profile_id.lock().unwrap();
        assert_eq!(next_profile, 3);

        // After 7 factory CC mappings, next should be 8
        let next_cc = *state.next_cc_id.lock().unwrap();
        assert_eq!(next_cc, 8);

        // No factory programs/user gear
        let next_program = *state.next_program_id.lock().unwrap();
        assert_eq!(next_program, 1);

        let next_user = *state.next_user_gear_id.lock().unwrap();
        assert_eq!(next_user, 1);
    }

    #[test]
    fn test_profiles_filter_by_type() {
        let state = GearState::default();
        let profiles = state.profiles.lock().unwrap();

        // Filter synths
        let synths: Vec<_> = profiles.iter().filter(|p| p.device_type == "synth").collect();
        assert_eq!(synths.len(), 1);
        assert_eq!(synths[0].name, "General MIDI");

        // Filter samplers
        let samplers: Vec<_> = profiles.iter().filter(|p| p.device_type == "sampler").collect();
        assert_eq!(samplers.len(), 1);
        assert_eq!(samplers[0].name, "Akai MPC One");
    }

    #[test]
    fn test_cc_filter_by_profile() {
        let state = GearState::default();
        let cc = state.cc_mappings.lock().unwrap();

        // All CC mappings are for profile 1 (GM)
        let gm_cc: Vec<_> = cc.iter().filter(|m| m.profile_id == 1).collect();
        assert_eq!(gm_cc.len(), 7);

        // No CC mappings for profile 2 (MPC)
        let mpc_cc: Vec<_> = cc.iter().filter(|m| m.profile_id == 2).collect();
        assert_eq!(mpc_cc.len(), 0);
    }

    #[test]
    fn test_profile_search_by_name() {
        let state = GearState::default();
        let profiles = state.profiles.lock().unwrap();

        let query = "mpc".to_lowercase();
        let results: Vec<_> = profiles
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query)
                    || p.manufacturer
                        .as_ref()
                        .map(|m| m.to_lowercase().contains(&query))
                        .unwrap_or(false)
            })
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Akai MPC One");
    }

    #[test]
    fn test_profile_search_by_manufacturer() {
        let state = GearState::default();
        let profiles = state.profiles.lock().unwrap();

        let query = "akai".to_lowercase();
        let results: Vec<_> = profiles
            .iter()
            .filter(|p| {
                p.manufacturer
                    .as_ref()
                    .map(|m| m.to_lowercase().contains(&query))
                    .unwrap_or(false)
            })
            .collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].manufacturer, Some("Akai Professional".to_string()));
    }

    // -------------------------------------------------------------------------
    // Data Integrity Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_cc_number_range() {
        // CC numbers should be 0-127
        let mut mapping = GearCCMapping::default();
        for cc in 0u8..=127 {
            mapping.cc_number = cc;
            assert!(mapping.cc_number <= 127);
        }
    }

    #[test]
    fn test_midi_channel_range() {
        // MIDI channels should be 1-16 (or None for omni)
        let mut gear = UserGear::default();

        gear.midi_channel = None; // Omni
        assert!(gear.midi_channel.is_none());

        for channel in 1u8..=16 {
            gear.midi_channel = Some(channel);
            assert!(gear.midi_channel.unwrap() >= 1 && gear.midi_channel.unwrap() <= 16);
        }
    }

    #[test]
    fn test_program_number_range() {
        // Program numbers should be 0-127
        let mut program = GearProgram::default();
        for pn in 0u8..=127 {
            program.program_number = pn;
            assert!(program.program_number <= 127);
        }
    }

    #[test]
    fn test_bank_select_range() {
        // Bank MSB/LSB should be 0-127
        let mut program = GearProgram::default();
        for bank in 0u8..=127 {
            program.bank_msb = bank;
            program.bank_lsb = bank;
            assert!(program.bank_msb <= 127);
            assert!(program.bank_lsb <= 127);
        }
    }
}
