//! Effect Commands
//!
//! Tauri commands for audio effect management including:
//! - Effect chain management (add, remove, reorder)
//! - Parameter control with automation
//! - Preset management (save, load, factory presets)
//! - Effect bypass and wet/dry mixing
//! - Real-time parameter modulation

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use thiserror::Error;
use tokio::sync::RwLock;

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Error)]
pub enum EffectError {
    #[error("Effect not found: {0}")]
    EffectNotFound(String),

    #[error("Effect type not found: {0}")]
    EffectTypeNotFound(String),

    #[error("Track not found: {0}")]
    TrackNotFound(i64),

    #[error("Parameter not found: {0}")]
    ParameterNotFound(String),

    #[error("Invalid parameter value: {0} (expected {1} to {2})")]
    InvalidParameterValue(f64, f64, f64),

    #[error("Invalid slot index: {0}")]
    InvalidSlotIndex(usize),

    #[error("Maximum effects reached: {0}")]
    MaxEffectsReached(usize),

    #[error("Preset not found: {0}")]
    PresetNotFound(String),

    #[error("Cannot save factory preset")]
    CannotSaveFactoryPreset,

    #[error("Preset already exists: {0}")]
    PresetAlreadyExists(String),

    #[error("Effect chain locked")]
    ChainLocked,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("State error: {0}")]
    StateError(String),
}

impl Serialize for EffectError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// ============================================================================
// DATA TYPES
// ============================================================================

/// Unique identifier for an effect instance
pub type EffectId = String;

/// Effect categories
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EffectCategory {
    Dynamics,
    Eq,
    Filter,
    Distortion,
    Modulation,
    Delay,
    Reverb,
    Pitch,
    Utility,
    Analyzer,
    Midi,
    Custom,
}

/// Built-in effect types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    // Dynamics
    Compressor,
    Limiter,
    Gate,
    Expander,
    Transient,

    // EQ
    ParametricEq,
    GraphicEq,
    HighPass,
    LowPass,
    BandPass,
    Notch,
    Shelf,

    // Filter
    LadderFilter,
    StateVariableFilter,
    CombFilter,
    FormantFilter,

    // Distortion
    Overdrive,
    Distortion,
    Fuzz,
    BitCrusher,
    WaveShaper,
    Saturation,

    // Modulation
    Chorus,
    Flanger,
    Phaser,
    Tremolo,
    Vibrato,
    AutoPan,
    RingMod,

    // Delay
    Delay,
    PingPong,
    MultiTap,
    Tape,

    // Reverb
    Room,
    Hall,
    Plate,
    Spring,
    Convolution,
    Shimmer,

    // Pitch
    PitchShifter,
    HarmonyGen,
    AutoTune,

    // Utility
    Gain,
    Pan,
    StereoWidth,
    MonoSum,
    PhaseInvert,
    DcFilter,

    // Analyzer
    Spectrum,
    Oscilloscope,
    LevelMeter,
    Correlation,

    // Custom/External
    Custom(String),
}

impl EffectType {
    pub fn category(&self) -> EffectCategory {
        match self {
            Self::Compressor | Self::Limiter | Self::Gate | Self::Expander | Self::Transient => {
                EffectCategory::Dynamics
            },
            Self::ParametricEq
            | Self::GraphicEq
            | Self::HighPass
            | Self::LowPass
            | Self::BandPass
            | Self::Notch
            | Self::Shelf => EffectCategory::Eq,
            Self::LadderFilter
            | Self::StateVariableFilter
            | Self::CombFilter
            | Self::FormantFilter => EffectCategory::Filter,
            Self::Overdrive
            | Self::Distortion
            | Self::Fuzz
            | Self::BitCrusher
            | Self::WaveShaper
            | Self::Saturation => EffectCategory::Distortion,
            Self::Chorus
            | Self::Flanger
            | Self::Phaser
            | Self::Tremolo
            | Self::Vibrato
            | Self::AutoPan
            | Self::RingMod => EffectCategory::Modulation,
            Self::Delay | Self::PingPong | Self::MultiTap | Self::Tape => EffectCategory::Delay,
            Self::Room
            | Self::Hall
            | Self::Plate
            | Self::Spring
            | Self::Convolution
            | Self::Shimmer => EffectCategory::Reverb,
            Self::PitchShifter | Self::HarmonyGen | Self::AutoTune => EffectCategory::Pitch,
            Self::Gain
            | Self::Pan
            | Self::StereoWidth
            | Self::MonoSum
            | Self::PhaseInvert
            | Self::DcFilter => EffectCategory::Utility,
            Self::Spectrum | Self::Oscilloscope | Self::LevelMeter | Self::Correlation => {
                EffectCategory::Analyzer
            },
            Self::Custom(_) => EffectCategory::Custom,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Compressor => "Compressor",
            Self::Limiter => "Limiter",
            Self::Gate => "Noise Gate",
            Self::Expander => "Expander",
            Self::Transient => "Transient Shaper",
            Self::ParametricEq => "Parametric EQ",
            Self::GraphicEq => "Graphic EQ",
            Self::HighPass => "High Pass Filter",
            Self::LowPass => "Low Pass Filter",
            Self::BandPass => "Band Pass Filter",
            Self::Notch => "Notch Filter",
            Self::Shelf => "Shelf EQ",
            Self::LadderFilter => "Ladder Filter",
            Self::StateVariableFilter => "State Variable Filter",
            Self::CombFilter => "Comb Filter",
            Self::FormantFilter => "Formant Filter",
            Self::Overdrive => "Overdrive",
            Self::Distortion => "Distortion",
            Self::Fuzz => "Fuzz",
            Self::BitCrusher => "Bit Crusher",
            Self::WaveShaper => "Wave Shaper",
            Self::Saturation => "Saturation",
            Self::Chorus => "Chorus",
            Self::Flanger => "Flanger",
            Self::Phaser => "Phaser",
            Self::Tremolo => "Tremolo",
            Self::Vibrato => "Vibrato",
            Self::AutoPan => "Auto Pan",
            Self::RingMod => "Ring Modulator",
            Self::Delay => "Delay",
            Self::PingPong => "Ping Pong Delay",
            Self::MultiTap => "Multi-Tap Delay",
            Self::Tape => "Tape Delay",
            Self::Room => "Room Reverb",
            Self::Hall => "Hall Reverb",
            Self::Plate => "Plate Reverb",
            Self::Spring => "Spring Reverb",
            Self::Convolution => "Convolution Reverb",
            Self::Shimmer => "Shimmer Reverb",
            Self::PitchShifter => "Pitch Shifter",
            Self::HarmonyGen => "Harmony Generator",
            Self::AutoTune => "Auto Tune",
            Self::Gain => "Gain",
            Self::Pan => "Pan",
            Self::StereoWidth => "Stereo Width",
            Self::MonoSum => "Mono Sum",
            Self::PhaseInvert => "Phase Invert",
            Self::DcFilter => "DC Filter",
            Self::Spectrum => "Spectrum Analyzer",
            Self::Oscilloscope => "Oscilloscope",
            Self::LevelMeter => "Level Meter",
            Self::Correlation => "Correlation Meter",
            Self::Custom(name) => name,
        }
    }
}

/// Effect parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub id: String,
    pub name: String,
    pub unit: String,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub step: f64,
    /// Logarithmic scaling
    pub logarithmic: bool,
    /// Parameter display format
    pub format: ParameterFormat,
    /// Parameter group for UI organization
    pub group: Option<String>,
    /// Whether this parameter can be automated
    pub automatable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterFormat {
    Float { decimals: u8 },
    Db,
    Hz,
    Ms,
    Percent,
    Ratio,
    Semitones,
    Cents,
    OnOff,
    Enum { values: Vec<String> },
}

/// Current parameter value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValue {
    pub id: String,
    pub value: f64,
    pub normalized: f64,
    pub display: String,
    pub is_automated: bool,
    pub modulation_amount: f64,
}

/// Complete effect instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub id: EffectId,
    pub effect_type: EffectType,
    pub name: String,
    pub bypass: bool,
    pub wet_dry: f64,
    pub slot_index: usize,
    pub parameters: Vec<ParameterValue>,
    pub preset_name: Option<String>,
    pub is_expanded: bool,
}

/// Effect chain for a track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectChain {
    pub track_id: i64,
    pub effects: Vec<Effect>,
    pub pre_fader_split: usize, // Effects before this index are pre-fader
    pub locked: bool,
}

/// Effect preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectPreset {
    pub id: String,
    pub name: String,
    pub effect_type: EffectType,
    pub parameters: HashMap<String, f64>,
    pub is_factory: bool,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub description: Option<String>,
}

/// Effect type metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectTypeMeta {
    pub effect_type: EffectType,
    pub name: String,
    pub category: EffectCategory,
    pub parameters: Vec<ParameterDef>,
    pub has_sidechain: bool,
    pub latency_samples: u32,
}

/// Managed state for effects
#[derive(Debug, Default)]
pub struct EffectState {
    pub chains: HashMap<i64, EffectChain>,
    pub presets: HashMap<String, EffectPreset>,
    pub type_registry: HashMap<String, EffectTypeMeta>,
}

pub type EffectStateHandle = Arc<RwLock<EffectState>>;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn generate_effect_id() -> EffectId {
    format!(
        "fx_{}",
        &uuid::Uuid::new_v4().to_string().replace("-", "")[..12]
    )
}

fn normalize_value(value: f64, min: f64, max: f64, logarithmic: bool) -> f64 {
    if logarithmic && min > 0.0 {
        let log_min = min.ln();
        let log_max = max.ln();
        let log_value = value.ln();
        (log_value - log_min) / (log_max - log_min)
    } else {
        (value - min) / (max - min)
    }
}

fn denormalize_value(normalized: f64, min: f64, max: f64, logarithmic: bool) -> f64 {
    if logarithmic && min > 0.0 {
        let log_min = min.ln();
        let log_max = max.ln();
        (log_min + normalized * (log_max - log_min)).exp()
    } else {
        min + normalized * (max - min)
    }
}

fn format_parameter_value(value: f64, format: &ParameterFormat) -> String {
    match format {
        ParameterFormat::Float { decimals } => format!("{:.1$}", value, *decimals as usize),
        ParameterFormat::Db => format!("{:.1} dB", value),
        ParameterFormat::Hz => {
            if value >= 1000.0 {
                format!("{:.2} kHz", value / 1000.0)
            } else {
                format!("{:.1} Hz", value)
            }
        },
        ParameterFormat::Ms => {
            if value >= 1000.0 {
                format!("{:.2} s", value / 1000.0)
            } else {
                format!("{:.1} ms", value)
            }
        },
        ParameterFormat::Percent => format!("{:.0}%", value * 100.0),
        ParameterFormat::Ratio => {
            if value >= 1.0 {
                format!("{:.1}:1", value)
            } else {
                format!("1:{:.1}", 1.0 / value)
            }
        },
        ParameterFormat::Semitones => {
            if value >= 0.0 {
                format!("+{:.0} st", value)
            } else {
                format!("{:.0} st", value)
            }
        },
        ParameterFormat::Cents => {
            if value >= 0.0 {
                format!("+{:.0} ct", value)
            } else {
                format!("{:.0} ct", value)
            }
        },
        ParameterFormat::OnOff => {
            if value >= 0.5 {
                "On".to_string()
            } else {
                "Off".to_string()
            }
        },
        ParameterFormat::Enum { values } => {
            let index = (value.round() as usize).min(values.len().saturating_sub(1));
            values.get(index).cloned().unwrap_or_else(|| format!("{}", index))
        },
    }
}

/// Get default parameters for an effect type
fn get_default_parameters(effect_type: &EffectType) -> Vec<ParameterDef> {
    match effect_type {
        EffectType::Compressor => vec![
            ParameterDef {
                id: "threshold".into(),
                name: "Threshold".into(),
                unit: "dB".into(),
                min: -60.0,
                max: 0.0,
                default: -20.0,
                step: 0.5,
                logarithmic: false,
                format: ParameterFormat::Db,
                group: Some("Dynamics".into()),
                automatable: true,
            },
            ParameterDef {
                id: "ratio".into(),
                name: "Ratio".into(),
                unit: ":1".into(),
                min: 1.0,
                max: 20.0,
                default: 4.0,
                step: 0.1,
                logarithmic: true,
                format: ParameterFormat::Ratio,
                group: Some("Dynamics".into()),
                automatable: true,
            },
            ParameterDef {
                id: "attack".into(),
                name: "Attack".into(),
                unit: "ms".into(),
                min: 0.1,
                max: 100.0,
                default: 10.0,
                step: 0.1,
                logarithmic: true,
                format: ParameterFormat::Ms,
                group: Some("Envelope".into()),
                automatable: true,
            },
            ParameterDef {
                id: "release".into(),
                name: "Release".into(),
                unit: "ms".into(),
                min: 10.0,
                max: 1000.0,
                default: 100.0,
                step: 1.0,
                logarithmic: true,
                format: ParameterFormat::Ms,
                group: Some("Envelope".into()),
                automatable: true,
            },
            ParameterDef {
                id: "makeup".into(),
                name: "Makeup Gain".into(),
                unit: "dB".into(),
                min: 0.0,
                max: 24.0,
                default: 0.0,
                step: 0.5,
                logarithmic: false,
                format: ParameterFormat::Db,
                group: Some("Output".into()),
                automatable: true,
            },
            ParameterDef {
                id: "knee".into(),
                name: "Knee".into(),
                unit: "dB".into(),
                min: 0.0,
                max: 12.0,
                default: 6.0,
                step: 0.5,
                logarithmic: false,
                format: ParameterFormat::Db,
                group: Some("Dynamics".into()),
                automatable: true,
            },
        ],
        EffectType::Delay => vec![
            ParameterDef {
                id: "time".into(),
                name: "Time".into(),
                unit: "ms".into(),
                min: 1.0,
                max: 2000.0,
                default: 250.0,
                step: 1.0,
                logarithmic: true,
                format: ParameterFormat::Ms,
                group: Some("Delay".into()),
                automatable: true,
            },
            ParameterDef {
                id: "feedback".into(),
                name: "Feedback".into(),
                unit: "%".into(),
                min: 0.0,
                max: 1.0,
                default: 0.3,
                step: 0.01,
                logarithmic: false,
                format: ParameterFormat::Percent,
                group: Some("Delay".into()),
                automatable: true,
            },
            ParameterDef {
                id: "lowcut".into(),
                name: "Low Cut".into(),
                unit: "Hz".into(),
                min: 20.0,
                max: 2000.0,
                default: 100.0,
                step: 1.0,
                logarithmic: true,
                format: ParameterFormat::Hz,
                group: Some("Filter".into()),
                automatable: true,
            },
            ParameterDef {
                id: "highcut".into(),
                name: "High Cut".into(),
                unit: "Hz".into(),
                min: 1000.0,
                max: 20000.0,
                default: 8000.0,
                step: 10.0,
                logarithmic: true,
                format: ParameterFormat::Hz,
                group: Some("Filter".into()),
                automatable: true,
            },
            ParameterDef {
                id: "sync".into(),
                name: "Tempo Sync".into(),
                unit: "".into(),
                min: 0.0,
                max: 1.0,
                default: 0.0,
                step: 1.0,
                logarithmic: false,
                format: ParameterFormat::OnOff,
                group: Some("Sync".into()),
                automatable: false,
            },
        ],
        EffectType::Room
        | EffectType::Hall
        | EffectType::Plate
        | EffectType::Spring
        | EffectType::Convolution
        | EffectType::Shimmer => vec![
            ParameterDef {
                id: "size".into(),
                name: "Size".into(),
                unit: "%".into(),
                min: 0.0,
                max: 1.0,
                default: 0.5,
                step: 0.01,
                logarithmic: false,
                format: ParameterFormat::Percent,
                group: Some("Room".into()),
                automatable: true,
            },
            ParameterDef {
                id: "decay".into(),
                name: "Decay".into(),
                unit: "s".into(),
                min: 0.1,
                max: 10.0,
                default: 2.0,
                step: 0.1,
                logarithmic: true,
                format: ParameterFormat::Float { decimals: 1 },
                group: Some("Room".into()),
                automatable: true,
            },
            ParameterDef {
                id: "predelay".into(),
                name: "Pre-delay".into(),
                unit: "ms".into(),
                min: 0.0,
                max: 200.0,
                default: 20.0,
                step: 1.0,
                logarithmic: false,
                format: ParameterFormat::Ms,
                group: Some("Room".into()),
                automatable: true,
            },
            ParameterDef {
                id: "damping".into(),
                name: "Damping".into(),
                unit: "%".into(),
                min: 0.0,
                max: 1.0,
                default: 0.5,
                step: 0.01,
                logarithmic: false,
                format: ParameterFormat::Percent,
                group: Some("Tone".into()),
                automatable: true,
            },
            ParameterDef {
                id: "diffusion".into(),
                name: "Diffusion".into(),
                unit: "%".into(),
                min: 0.0,
                max: 1.0,
                default: 0.7,
                step: 0.01,
                logarithmic: false,
                format: ParameterFormat::Percent,
                group: Some("Tone".into()),
                automatable: true,
            },
        ],
        EffectType::Gain => vec![ParameterDef {
            id: "gain".into(),
            name: "Gain".into(),
            unit: "dB".into(),
            min: -24.0,
            max: 24.0,
            default: 0.0,
            step: 0.1,
            logarithmic: false,
            format: ParameterFormat::Db,
            group: None,
            automatable: true,
        }],
        _ => vec![
            // Generic wet/dry for unknown types
            ParameterDef {
                id: "mix".into(),
                name: "Mix".into(),
                unit: "%".into(),
                min: 0.0,
                max: 1.0,
                default: 1.0,
                step: 0.01,
                logarithmic: false,
                format: ParameterFormat::Percent,
                group: None,
                automatable: true,
            },
        ],
    }
}

/// Create default parameter values from definitions
fn create_default_values(defs: &[ParameterDef]) -> Vec<ParameterValue> {
    defs.iter()
        .map(|def| {
            let normalized = normalize_value(def.default, def.min, def.max, def.logarithmic);
            ParameterValue {
                id: def.id.clone(),
                value: def.default,
                normalized,
                display: format_parameter_value(def.default, &def.format),
                is_automated: false,
                modulation_amount: 0.0,
            }
        })
        .collect()
}

// ============================================================================
// INTERNAL IMPLEMENTATION FUNCTIONS
// ============================================================================

/// Get all available effect types
pub fn get_available_effects_impl() -> Vec<EffectTypeMeta> {
    let effect_types = vec![
        EffectType::Compressor,
        EffectType::Limiter,
        EffectType::Gate,
        EffectType::ParametricEq,
        EffectType::HighPass,
        EffectType::LowPass,
        EffectType::Overdrive,
        EffectType::Distortion,
        EffectType::BitCrusher,
        EffectType::Chorus,
        EffectType::Flanger,
        EffectType::Phaser,
        EffectType::Delay,
        EffectType::PingPong,
        EffectType::Room,
        EffectType::Hall,
        EffectType::Plate,
        EffectType::PitchShifter,
        EffectType::Gain,
        EffectType::Pan,
        EffectType::StereoWidth,
        EffectType::Spectrum,
        EffectType::LevelMeter,
    ];

    effect_types
        .into_iter()
        .map(|effect_type| {
            let parameters = get_default_parameters(&effect_type);
            EffectTypeMeta {
                name: effect_type.display_name().to_string(),
                category: effect_type.category(),
                has_sidechain: matches!(
                    effect_type,
                    EffectType::Compressor | EffectType::Gate | EffectType::Expander
                ),
                latency_samples: 0, // Most effects are zero-latency
                effect_type,
                parameters,
            }
        })
        .collect()
}

/// Get effect chain for a track
pub async fn get_effect_chain_impl(
    track_id: i64,
    state: &EffectStateHandle,
) -> Result<EffectChain, EffectError> {
    let state_guard = state.read().await;
    state_guard
        .chains
        .get(&track_id)
        .cloned()
        .ok_or(EffectError::TrackNotFound(track_id))
}

/// Create a new effect chain for a track
pub async fn create_effect_chain_impl(
    track_id: i64,
    state: &EffectStateHandle,
) -> Result<EffectChain, EffectError> {
    let chain = EffectChain { track_id, effects: Vec::new(), pre_fader_split: 0, locked: false };

    let mut state_guard = state.write().await;
    state_guard.chains.insert(track_id, chain.clone());

    Ok(chain)
}

/// Add an effect to a track's chain
pub async fn add_effect_impl(
    track_id: i64,
    effect_type: EffectType,
    slot_index: Option<usize>,
    state: &EffectStateHandle,
) -> Result<Effect, EffectError> {
    let mut state_guard = state.write().await;

    // Get or create chain
    let chain = state_guard.chains.entry(track_id).or_insert_with(|| EffectChain {
        track_id,
        effects: Vec::new(),
        pre_fader_split: 0,
        locked: false,
    });

    if chain.locked {
        return Err(EffectError::ChainLocked);
    }

    const MAX_EFFECTS: usize = 16;
    if chain.effects.len() >= MAX_EFFECTS {
        return Err(EffectError::MaxEffectsReached(MAX_EFFECTS));
    }

    let param_defs = get_default_parameters(&effect_type);
    let parameters = create_default_values(&param_defs);

    let slot = slot_index.unwrap_or(chain.effects.len());
    let slot = slot.min(chain.effects.len());

    let effect = Effect {
        id: generate_effect_id(),
        name: effect_type.display_name().to_string(),
        effect_type,
        bypass: false,
        wet_dry: 1.0,
        slot_index: slot,
        parameters,
        preset_name: None,
        is_expanded: true,
    };

    chain.effects.insert(slot, effect.clone());

    // Update slot indices
    for (i, fx) in chain.effects.iter_mut().enumerate() {
        fx.slot_index = i;
    }

    Ok(effect)
}

/// Remove an effect from a track's chain
pub async fn remove_effect_impl(
    track_id: i64,
    effect_id: &str,
    state: &EffectStateHandle,
) -> Result<(), EffectError> {
    let mut state_guard = state.write().await;

    let chain = state_guard
        .chains
        .get_mut(&track_id)
        .ok_or(EffectError::TrackNotFound(track_id))?;

    if chain.locked {
        return Err(EffectError::ChainLocked);
    }

    let index = chain
        .effects
        .iter()
        .position(|fx| fx.id == effect_id)
        .ok_or_else(|| EffectError::EffectNotFound(effect_id.to_string()))?;

    chain.effects.remove(index);

    // Update slot indices
    for (i, fx) in chain.effects.iter_mut().enumerate() {
        fx.slot_index = i;
    }

    Ok(())
}

/// Move an effect to a different slot
pub async fn move_effect_impl(
    track_id: i64,
    effect_id: &str,
    new_slot: usize,
    state: &EffectStateHandle,
) -> Result<EffectChain, EffectError> {
    let mut state_guard = state.write().await;

    let chain = state_guard
        .chains
        .get_mut(&track_id)
        .ok_or(EffectError::TrackNotFound(track_id))?;

    if chain.locked {
        return Err(EffectError::ChainLocked);
    }

    let current_index = chain
        .effects
        .iter()
        .position(|fx| fx.id == effect_id)
        .ok_or_else(|| EffectError::EffectNotFound(effect_id.to_string()))?;

    if new_slot >= chain.effects.len() {
        return Err(EffectError::InvalidSlotIndex(new_slot));
    }

    let effect = chain.effects.remove(current_index);
    chain.effects.insert(new_slot, effect);

    // Update slot indices
    for (i, fx) in chain.effects.iter_mut().enumerate() {
        fx.slot_index = i;
    }

    Ok(chain.clone())
}

/// Set effect bypass state
pub async fn set_effect_bypass_impl(
    track_id: i64,
    effect_id: &str,
    bypass: bool,
    state: &EffectStateHandle,
) -> Result<Effect, EffectError> {
    let mut state_guard = state.write().await;

    let chain = state_guard
        .chains
        .get_mut(&track_id)
        .ok_or(EffectError::TrackNotFound(track_id))?;

    let effect = chain
        .effects
        .iter_mut()
        .find(|fx| fx.id == effect_id)
        .ok_or_else(|| EffectError::EffectNotFound(effect_id.to_string()))?;

    effect.bypass = bypass;

    Ok(effect.clone())
}

/// Set effect wet/dry mix
pub async fn set_effect_wet_dry_impl(
    track_id: i64,
    effect_id: &str,
    wet_dry: f64,
    state: &EffectStateHandle,
) -> Result<Effect, EffectError> {
    let wet_dry = wet_dry.clamp(0.0, 1.0);

    let mut state_guard = state.write().await;

    let chain = state_guard
        .chains
        .get_mut(&track_id)
        .ok_or(EffectError::TrackNotFound(track_id))?;

    let effect = chain
        .effects
        .iter_mut()
        .find(|fx| fx.id == effect_id)
        .ok_or_else(|| EffectError::EffectNotFound(effect_id.to_string()))?;

    effect.wet_dry = wet_dry;

    Ok(effect.clone())
}

/// Set a parameter value
pub async fn set_parameter_impl(
    track_id: i64,
    effect_id: &str,
    param_id: &str,
    value: f64,
    state: &EffectStateHandle,
) -> Result<ParameterValue, EffectError> {
    let mut state_guard = state.write().await;

    let chain = state_guard
        .chains
        .get_mut(&track_id)
        .ok_or(EffectError::TrackNotFound(track_id))?;

    let effect = chain
        .effects
        .iter_mut()
        .find(|fx| fx.id == effect_id)
        .ok_or_else(|| EffectError::EffectNotFound(effect_id.to_string()))?;

    let param = effect
        .parameters
        .iter_mut()
        .find(|p| p.id == param_id)
        .ok_or_else(|| EffectError::ParameterNotFound(param_id.to_string()))?;

    // Get parameter definition for formatting
    let param_defs = get_default_parameters(&effect.effect_type);
    let param_def = param_defs
        .iter()
        .find(|d| d.id == param_id)
        .ok_or_else(|| EffectError::ParameterNotFound(param_id.to_string()))?;

    // Validate and clamp value
    let clamped_value = value.clamp(param_def.min, param_def.max);

    param.value = clamped_value;
    param.normalized = normalize_value(
        clamped_value,
        param_def.min,
        param_def.max,
        param_def.logarithmic,
    );
    param.display = format_parameter_value(clamped_value, &param_def.format);

    Ok(param.clone())
}

/// Set parameter by normalized value (0-1)
pub async fn set_parameter_normalized_impl(
    track_id: i64,
    effect_id: &str,
    param_id: &str,
    normalized: f64,
    state: &EffectStateHandle,
) -> Result<ParameterValue, EffectError> {
    let state_guard = state.read().await;

    let chain = state_guard.chains.get(&track_id).ok_or(EffectError::TrackNotFound(track_id))?;

    let effect = chain
        .effects
        .iter()
        .find(|fx| fx.id == effect_id)
        .ok_or_else(|| EffectError::EffectNotFound(effect_id.to_string()))?;

    let param_defs = get_default_parameters(&effect.effect_type);
    let param_def = param_defs
        .iter()
        .find(|d| d.id == param_id)
        .ok_or_else(|| EffectError::ParameterNotFound(param_id.to_string()))?;

    drop(state_guard);

    let actual_value = denormalize_value(
        normalized.clamp(0.0, 1.0),
        param_def.min,
        param_def.max,
        param_def.logarithmic,
    );

    set_parameter_impl(track_id, effect_id, param_id, actual_value, state).await
}

/// Load a preset
pub async fn load_preset_impl(
    track_id: i64,
    effect_id: &str,
    preset_id: &str,
    state: &EffectStateHandle,
) -> Result<Effect, EffectError> {
    let state_guard = state.read().await;

    let preset = state_guard
        .presets
        .get(preset_id)
        .cloned()
        .ok_or_else(|| EffectError::PresetNotFound(preset_id.to_string()))?;

    drop(state_guard);

    // Apply each parameter from preset
    for (param_id, value) in &preset.parameters {
        set_parameter_impl(track_id, effect_id, param_id, *value, state).await?;
    }

    // Update preset name
    let mut state_guard = state.write().await;
    let chain = state_guard
        .chains
        .get_mut(&track_id)
        .ok_or(EffectError::TrackNotFound(track_id))?;

    let effect = chain
        .effects
        .iter_mut()
        .find(|fx| fx.id == effect_id)
        .ok_or_else(|| EffectError::EffectNotFound(effect_id.to_string()))?;

    effect.preset_name = Some(preset.name.clone());

    Ok(effect.clone())
}

/// Save current settings as a preset
pub async fn save_preset_impl(
    track_id: i64,
    effect_id: &str,
    preset_name: &str,
    category: Option<String>,
    tags: Vec<String>,
    state: &EffectStateHandle,
) -> Result<EffectPreset, EffectError> {
    let state_guard = state.read().await;

    let chain = state_guard.chains.get(&track_id).ok_or(EffectError::TrackNotFound(track_id))?;

    let effect = chain
        .effects
        .iter()
        .find(|fx| fx.id == effect_id)
        .ok_or_else(|| EffectError::EffectNotFound(effect_id.to_string()))?;

    let preset_id = format!(
        "user_{}_{}",
        preset_name.to_lowercase().replace(" ", "_"),
        chrono::Utc::now().timestamp()
    );

    // Check if preset name already exists
    if state_guard.presets.values().any(|p| p.name == preset_name && !p.is_factory) {
        return Err(EffectError::PresetAlreadyExists(preset_name.to_string()));
    }

    let parameters: HashMap<String, f64> =
        effect.parameters.iter().map(|p| (p.id.clone(), p.value)).collect();

    let preset = EffectPreset {
        id: preset_id.clone(),
        name: preset_name.to_string(),
        effect_type: effect.effect_type.clone(),
        parameters,
        is_factory: false,
        category,
        tags,
        author: None,
        description: None,
    };

    drop(state_guard);

    let mut state_guard = state.write().await;
    state_guard.presets.insert(preset_id, preset.clone());

    Ok(preset)
}

/// Get all presets for an effect type
pub async fn get_presets_impl(
    effect_type: &EffectType,
    state: &EffectStateHandle,
) -> Result<Vec<EffectPreset>, EffectError> {
    let state_guard = state.read().await;

    let presets: Vec<EffectPreset> = state_guard
        .presets
        .values()
        .filter(|p| &p.effect_type == effect_type)
        .cloned()
        .collect();

    Ok(presets)
}

/// Delete a user preset
pub async fn delete_preset_impl(
    preset_id: &str,
    state: &EffectStateHandle,
) -> Result<(), EffectError> {
    let mut state_guard = state.write().await;

    let preset = state_guard
        .presets
        .get(preset_id)
        .ok_or_else(|| EffectError::PresetNotFound(preset_id.to_string()))?;

    if preset.is_factory {
        return Err(EffectError::CannotSaveFactoryPreset);
    }

    state_guard.presets.remove(preset_id);

    Ok(())
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

#[tauri::command]
pub fn get_available_effects() -> Vec<EffectTypeMeta> {
    get_available_effects_impl()
}

#[tauri::command]
pub async fn get_effect_chain(
    track_id: i64,
    state: State<'_, EffectStateHandle>,
) -> Result<EffectChain, EffectError> {
    get_effect_chain_impl(track_id, &state).await
}

#[tauri::command]
pub async fn create_effect_chain(
    track_id: i64,
    state: State<'_, EffectStateHandle>,
) -> Result<EffectChain, EffectError> {
    create_effect_chain_impl(track_id, &state).await
}

#[tauri::command]
pub async fn add_effect(
    track_id: i64,
    effect_type: EffectType,
    slot_index: Option<usize>,
    state: State<'_, EffectStateHandle>,
) -> Result<Effect, EffectError> {
    add_effect_impl(track_id, effect_type, slot_index, &state).await
}

#[tauri::command]
pub async fn remove_effect(
    track_id: i64,
    effect_id: String,
    state: State<'_, EffectStateHandle>,
) -> Result<(), EffectError> {
    remove_effect_impl(track_id, &effect_id, &state).await
}

#[tauri::command]
pub async fn move_effect(
    track_id: i64,
    effect_id: String,
    new_slot: usize,
    state: State<'_, EffectStateHandle>,
) -> Result<EffectChain, EffectError> {
    move_effect_impl(track_id, &effect_id, new_slot, &state).await
}

#[tauri::command]
pub async fn set_effect_bypass(
    track_id: i64,
    effect_id: String,
    bypass: bool,
    state: State<'_, EffectStateHandle>,
) -> Result<Effect, EffectError> {
    set_effect_bypass_impl(track_id, &effect_id, bypass, &state).await
}

#[tauri::command]
pub async fn set_effect_wet_dry(
    track_id: i64,
    effect_id: String,
    wet_dry: f64,
    state: State<'_, EffectStateHandle>,
) -> Result<Effect, EffectError> {
    set_effect_wet_dry_impl(track_id, &effect_id, wet_dry, &state).await
}

#[tauri::command]
pub async fn set_effect_parameter(
    track_id: i64,
    effect_id: String,
    param_id: String,
    value: f64,
    state: State<'_, EffectStateHandle>,
) -> Result<ParameterValue, EffectError> {
    set_parameter_impl(track_id, &effect_id, &param_id, value, &state).await
}

#[tauri::command]
pub async fn set_effect_parameter_normalized(
    track_id: i64,
    effect_id: String,
    param_id: String,
    normalized: f64,
    state: State<'_, EffectStateHandle>,
) -> Result<ParameterValue, EffectError> {
    set_parameter_normalized_impl(track_id, &effect_id, &param_id, normalized, &state).await
}

#[tauri::command]
pub async fn load_effect_preset(
    track_id: i64,
    effect_id: String,
    preset_id: String,
    state: State<'_, EffectStateHandle>,
) -> Result<Effect, EffectError> {
    load_preset_impl(track_id, &effect_id, &preset_id, &state).await
}

#[tauri::command]
pub async fn save_effect_preset(
    track_id: i64,
    effect_id: String,
    preset_name: String,
    category: Option<String>,
    tags: Vec<String>,
    state: State<'_, EffectStateHandle>,
) -> Result<EffectPreset, EffectError> {
    save_preset_impl(track_id, &effect_id, &preset_name, category, tags, &state).await
}

#[tauri::command]
pub async fn get_effect_presets(
    effect_type: EffectType,
    state: State<'_, EffectStateHandle>,
) -> Result<Vec<EffectPreset>, EffectError> {
    get_presets_impl(&effect_type, &state).await
}

#[tauri::command]
pub async fn delete_effect_preset(
    preset_id: String,
    state: State<'_, EffectStateHandle>,
) -> Result<(), EffectError> {
    delete_preset_impl(&preset_id, &state).await
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_type_category() {
        assert_eq!(EffectType::Compressor.category(), EffectCategory::Dynamics);
        assert_eq!(EffectType::ParametricEq.category(), EffectCategory::Eq);
        assert_eq!(EffectType::Delay.category(), EffectCategory::Delay);
        assert_eq!(EffectType::Hall.category(), EffectCategory::Reverb);
        assert_eq!(EffectType::Chorus.category(), EffectCategory::Modulation);
    }

    #[test]
    fn test_normalize_linear() {
        assert_eq!(normalize_value(0.0, 0.0, 100.0, false), 0.0);
        assert_eq!(normalize_value(50.0, 0.0, 100.0, false), 0.5);
        assert_eq!(normalize_value(100.0, 0.0, 100.0, false), 1.0);
    }

    #[test]
    fn test_denormalize_linear() {
        assert_eq!(denormalize_value(0.0, 0.0, 100.0, false), 0.0);
        assert_eq!(denormalize_value(0.5, 0.0, 100.0, false), 50.0);
        assert_eq!(denormalize_value(1.0, 0.0, 100.0, false), 100.0);
    }

    #[test]
    fn test_format_parameter() {
        assert_eq!(
            format_parameter_value(-6.0, &ParameterFormat::Db),
            "-6.0 dB"
        );
        assert_eq!(
            format_parameter_value(0.5, &ParameterFormat::Percent),
            "50%"
        );
        assert_eq!(
            format_parameter_value(1000.0, &ParameterFormat::Hz),
            "1.00 kHz"
        );
        assert_eq!(
            format_parameter_value(4.0, &ParameterFormat::Ratio),
            "4.0:1"
        );
    }

    #[test]
    fn test_get_default_parameters() {
        let params = get_default_parameters(&EffectType::Compressor);
        assert!(!params.is_empty());
        assert!(params.iter().any(|p| p.id == "threshold"));
        assert!(params.iter().any(|p| p.id == "ratio"));
    }

    #[tokio::test]
    async fn test_effect_chain_operations() {
        let state = Arc::new(RwLock::new(EffectState::default()));

        // Create chain
        let chain = create_effect_chain_impl(1, &state).await.unwrap();
        assert_eq!(chain.track_id, 1);
        assert!(chain.effects.is_empty());

        // Add effects
        let fx1 = add_effect_impl(1, EffectType::Compressor, None, &state).await.unwrap();
        assert_eq!(fx1.slot_index, 0);

        let fx2 = add_effect_impl(1, EffectType::Delay, None, &state).await.unwrap();
        assert_eq!(fx2.slot_index, 1);

        // Verify chain
        let chain = get_effect_chain_impl(1, &state).await.unwrap();
        assert_eq!(chain.effects.len(), 2);

        // Remove effect
        remove_effect_impl(1, &fx1.id, &state).await.unwrap();
        let chain = get_effect_chain_impl(1, &state).await.unwrap();
        assert_eq!(chain.effects.len(), 1);
    }

    #[tokio::test]
    async fn test_effect_bypass() {
        let state = Arc::new(RwLock::new(EffectState::default()));

        create_effect_chain_impl(1, &state).await.unwrap();
        let fx = add_effect_impl(1, EffectType::Compressor, None, &state).await.unwrap();

        assert!(!fx.bypass);

        let fx = set_effect_bypass_impl(1, &fx.id, true, &state).await.unwrap();
        assert!(fx.bypass);

        let fx = set_effect_bypass_impl(1, &fx.id, false, &state).await.unwrap();
        assert!(!fx.bypass);
    }
}
