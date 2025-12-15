// =============================================================================
// MIDI LEARN
// =============================================================================
// Map MIDI CC, notes, and other messages to application parameters.
//
// CLAUDE CODE INSTRUCTIONS:
// 1. Location: daw/src-tauri/src/midi/learn.rs
// 2. Works with any parameter that can be controlled
// 3. Persistent mappings saved to database
//
// FEATURES:
// - Learn mode: Touch a control, move a CC
// - Support for CC, notes, pitch bend, aftertouch
// - Value scaling and curves
// - Relative and absolute modes
// - Multi-parameter mapping
// - Pickup mode to avoid jumps
// =============================================================================

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

/// Mapping target type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MappingTarget {
    /// Internal parameter by path (e.g., "mixer/channel/1/volume")
    Parameter { path: String },
    /// Transport control
    Transport { action: TransportAction },
    /// Send MIDI to output
    MidiThrough {
        output: String,
        transform: Option<MidiTransform>,
    },
    /// Execute script
    Script { script_id: String },
    /// UI action
    UIAction { action: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TransportAction {
    Play,
    Stop,
    Record,
    Rewind,
    FastForward,
    LoopToggle,
    TapTempo,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MidiTransform {
    pub transpose: Option<i8>,
    pub channel: Option<u8>,
    pub velocity_scale: Option<f32>,
}

/// MIDI source for mapping
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MidiSource {
    pub device_id: String,
    pub message_type: MidiMessageType,
    pub channel: Option<u8>, // None = any channel
    pub data1: Option<u8>,   // CC number, note number, etc.
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MidiMessageType {
    NoteOn,
    NoteOff,
    ControlChange,
    ProgramChange,
    PitchBend,
    Aftertouch,
    PolyAftertouch,
}

/// Value scaling mode
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ScalingMode {
    /// Linear 0-127 to min-max
    Linear { min: f32, max: f32 },
    /// Logarithmic (good for frequency/volume)
    Logarithmic { min: f32, max: f32 },
    /// Exponential
    Exponential { min: f32, max: f32, curve: f32 },
    /// Stepped values
    Stepped { values: Vec<f32> },
    /// Toggle (any value > 63 = on)
    Toggle,
    /// Momentary (on while held)
    Momentary,
    /// Relative (increment/decrement)
    Relative { sensitivity: f32 },
}

impl ScalingMode {
    /// Convert MIDI value (0-127) to scaled value
    pub fn scale(&self, midi_value: u8, current_value: Option<f32>) -> f32 {
        let normalized = midi_value as f32 / 127.0;

        match self {
            ScalingMode::Linear { min, max } => min + (max - min) * normalized,
            ScalingMode::Logarithmic { min, max } => {
                let log_min = min.ln();
                let log_max = max.ln();
                (log_min + (log_max - log_min) * normalized).exp()
            },
            ScalingMode::Exponential { min, max, curve } => {
                let curved = normalized.powf(*curve);
                min + (max - min) * curved
            },
            ScalingMode::Stepped { values } => {
                let index = ((normalized * (values.len() - 1) as f32).round() as usize)
                    .min(values.len() - 1);
                values[index]
            },
            ScalingMode::Toggle => {
                if midi_value > 63 {
                    1.0
                } else {
                    0.0
                }
            },
            ScalingMode::Momentary => {
                if midi_value > 0 {
                    1.0
                } else {
                    0.0
                }
            },
            ScalingMode::Relative { sensitivity } => {
                let current = current_value.unwrap_or(0.5);
                let delta = if midi_value > 64 {
                    // Relative mode: values > 64 are decrements (128 - value)
                    -((128i16 - midi_value as i16) as f32) * sensitivity
                } else {
                    midi_value as f32 * sensitivity
                };
                (current + delta).clamp(0.0, 1.0)
            },
        }
    }
}

/// Pickup mode (avoid parameter jumps)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PickupMode {
    /// Jump immediately to new value
    Jump,
    /// Wait until control matches current value
    Pickup,
    /// Scale relative to where control was when touched
    ScaleFrom,
}

/// Complete MIDI mapping
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct MidiMapping {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub source: MidiSource,
    pub target: MappingTarget,
    pub scaling: ScalingMode,
    pub pickup_mode: PickupMode,

    // Runtime state (not persisted)
    #[serde(skip)]
    pub last_midi_value: Option<u8>,
    #[serde(skip)]
    pub picked_up: bool,
}

impl MidiMapping {
    pub fn new(name: &str, source: MidiSource, target: MappingTarget) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            enabled: true,
            source,
            target,
            scaling: ScalingMode::Linear { min: 0.0, max: 1.0 },
            pickup_mode: PickupMode::Jump,
            last_midi_value: None,
            picked_up: false,
        }
    }
}

/// Learn mode state
#[derive(Debug, Clone)]
pub struct LearnState {
    pub active: bool,
    pub target: Option<MappingTarget>,
    pub waiting_for_midi: bool,
}

/// MIDI Learn Manager
pub struct MidiLearn {
    mappings: RwLock<HashMap<String, MidiMapping>>,
    learn_state: RwLock<LearnState>,

    // Event broadcasting
    event_tx: broadcast::Sender<LearnEvent>,

    // Parameter value requests
    param_request_tx: mpsc::UnboundedSender<ParameterRequest>,
}

/// Events from MIDI learn system
#[derive(Debug, Clone)]
pub enum LearnEvent {
    MappingCreated(MidiMapping),
    MappingDeleted(String),
    MappingUpdated(MidiMapping),
    ParameterChanged { path: String, value: f32 },
    TransportTriggered(TransportAction),
    LearnModeStarted { target: MappingTarget },
    LearnModeCompleted { mapping: MidiMapping },
    LearnModeCancelled,
}

/// Request for parameter info
#[allow(dead_code)]
pub struct ParameterRequest {
    pub path: String,
    pub response_tx: tokio::sync::oneshot::Sender<Option<f32>>,
}

impl MidiLearn {
    /// Create new MIDI learn manager
    pub fn new() -> (
        Self,
        broadcast::Receiver<LearnEvent>,
        mpsc::UnboundedReceiver<ParameterRequest>,
    ) {
        let (event_tx, event_rx) = broadcast::channel(256);
        let (param_tx, param_rx) = mpsc::unbounded_channel();

        (
            Self {
                mappings: RwLock::new(HashMap::new()),
                learn_state: RwLock::new(LearnState {
                    active: false,
                    target: None,
                    waiting_for_midi: false,
                }),
                event_tx,
                param_request_tx: param_tx,
            },
            event_rx,
            param_rx,
        )
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<LearnEvent> {
        self.event_tx.subscribe()
    }

    /// Start learn mode for a target
    pub fn start_learn(&self, target: MappingTarget) {
        let mut state = self.learn_state.write();
        state.active = true;
        state.target = Some(target.clone());
        state.waiting_for_midi = true;

        let _ = self.event_tx.send(LearnEvent::LearnModeStarted { target });
    }

    /// Cancel learn mode
    pub fn cancel_learn(&self) {
        let mut state = self.learn_state.write();
        state.active = false;
        state.target = None;
        state.waiting_for_midi = false;

        let _ = self.event_tx.send(LearnEvent::LearnModeCancelled);
    }

    /// Check if in learn mode
    pub fn is_learning(&self) -> bool {
        self.learn_state.read().active
    }

    /// Process incoming MIDI message
    pub fn process_midi(
        &self,
        device_id: &str,
        msg_type: MidiMessageType,
        channel: u8,
        data1: u8,
        data2: u8,
    ) {
        // Check if we're in learn mode
        {
            let state = self.learn_state.read();
            if state.active && state.waiting_for_midi {
                drop(state);
                self.complete_learn(device_id, msg_type.clone(), channel, data1);
                return;
            }
        }

        // Find matching mappings
        let source = MidiSource {
            device_id: device_id.to_string(),
            message_type: msg_type.clone(),
            channel: Some(channel),
            data1: Some(data1),
        };

        let mappings = self.mappings.read();
        for mapping in mappings.values() {
            if !mapping.enabled {
                continue;
            }

            // Check if source matches
            if !self.source_matches(&mapping.source, &source) {
                continue;
            }

            // Apply mapping
            self.apply_mapping(mapping, data2);
        }
    }

    fn source_matches(&self, pattern: &MidiSource, actual: &MidiSource) -> bool {
        if pattern.device_id != actual.device_id && pattern.device_id != "*" {
            return false;
        }
        if pattern.message_type != actual.message_type {
            return false;
        }
        if let Some(ch) = pattern.channel {
            if Some(ch) != actual.channel {
                return false;
            }
        }
        if let Some(d1) = pattern.data1 {
            if Some(d1) != actual.data1 {
                return false;
            }
        }
        true
    }

    fn complete_learn(&self, device_id: &str, msg_type: MidiMessageType, channel: u8, data1: u8) {
        let mut state = self.learn_state.write();
        let target = state.target.take();
        state.active = false;
        state.waiting_for_midi = false;
        drop(state);

        if let Some(target) = target {
            let source = MidiSource {
                device_id: device_id.to_string(),
                message_type: msg_type,
                channel: Some(channel),
                data1: Some(data1),
            };

            let mapping = MidiMapping::new(&format!("Mapping {}", data1), source, target);

            let mapping_clone = mapping.clone();
            self.mappings.write().insert(mapping.id.clone(), mapping);

            let _ = self.event_tx.send(LearnEvent::LearnModeCompleted { mapping: mapping_clone });
        }
    }

    fn apply_mapping(&self, mapping: &MidiMapping, midi_value: u8) {
        // Get current value for relative mode
        let current_value = self.get_current_value(&mapping.target);

        // Check pickup mode
        if matches!(mapping.pickup_mode, PickupMode::Pickup) {
            if let Some(current) = current_value {
                let expected_midi = (current * 127.0) as u8;
                if (midi_value as i16 - expected_midi as i16).abs() > 5 {
                    // Not picked up yet
                    return;
                }
            }
        }

        // Scale value
        let scaled = mapping.scaling.scale(midi_value, current_value);

        // Apply to target
        match &mapping.target {
            MappingTarget::Parameter { path } => {
                let _ = self
                    .event_tx
                    .send(LearnEvent::ParameterChanged { path: path.clone(), value: scaled });
            },
            MappingTarget::Transport { action } => {
                if midi_value > 0 {
                    let _ = self.event_tx.send(LearnEvent::TransportTriggered(action.clone()));
                }
            },
            MappingTarget::Script { script_id } => {
                // Handle script execution
                tracing::info!("Execute script: {} with value {}", script_id, scaled);
            },
            MappingTarget::UIAction { action } => {
                tracing::info!("UI action: {} with value {}", action, scaled);
            },
            MappingTarget::MidiThrough { .. } => {
                // MIDI through handled separately
            },
        }
    }

    fn get_current_value(&self, target: &MappingTarget) -> Option<f32> {
        if let MappingTarget::Parameter { path } = target {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let request = ParameterRequest { path: path.clone(), response_tx: tx };
            let _ = self.param_request_tx.send(request);

            // Try to get response (non-blocking in practice would use async)
            rx.blocking_recv().ok().flatten()
        } else {
            None
        }
    }

    /// Add mapping directly
    pub fn add_mapping(&self, mapping: MidiMapping) {
        let mapping_clone = mapping.clone();
        self.mappings.write().insert(mapping.id.clone(), mapping);
        let _ = self.event_tx.send(LearnEvent::MappingCreated(mapping_clone));
    }

    /// Remove mapping
    pub fn remove_mapping(&self, id: &str) {
        if self.mappings.write().remove(id).is_some() {
            let _ = self.event_tx.send(LearnEvent::MappingDeleted(id.to_string()));
        }
    }

    /// Update mapping
    pub fn update_mapping(&self, mapping: MidiMapping) {
        let mapping_clone = mapping.clone();
        self.mappings.write().insert(mapping.id.clone(), mapping);
        let _ = self.event_tx.send(LearnEvent::MappingUpdated(mapping_clone));
    }

    /// Get all mappings
    pub fn list_mappings(&self) -> Vec<MidiMapping> {
        self.mappings.read().values().cloned().collect()
    }

    /// Clear all mappings
    pub fn clear_mappings(&self) {
        self.mappings.write().clear();
    }

    /// Save mappings to JSON
    pub fn export_mappings(&self) -> String {
        let mappings: Vec<MidiMapping> = self.mappings.read().values().cloned().collect();
        serde_json::to_string_pretty(&mappings).unwrap_or_default()
    }

    /// Load mappings from JSON
    pub fn import_mappings(&self, json: &str) -> Result<usize, serde_json::Error> {
        let mappings: Vec<MidiMapping> = serde_json::from_str(json)?;
        let count = mappings.len();

        for mapping in mappings {
            self.mappings.write().insert(mapping.id.clone(), mapping);
        }

        Ok(count)
    }
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

use tauri::State;

pub struct MidiLearnState(pub Arc<MidiLearn>);

// SAFETY: MidiLearn only contains channels and atomics, which are Send+Sync
unsafe impl Send for MidiLearnState {}
unsafe impl Sync for MidiLearnState {}

impl Default for MidiLearnState {
    fn default() -> Self {
        let (learn, _events, _param_requests) = MidiLearn::new();
        Self(Arc::new(learn))
    }
}

#[tauri::command]
pub fn learn_start(state: State<MidiLearnState>, target_path: String) {
    let target = MappingTarget::Parameter { path: target_path };
    state.0.start_learn(target);
}

#[tauri::command]
pub fn learn_cancel(state: State<MidiLearnState>) {
    state.0.cancel_learn();
}

#[tauri::command]
pub fn learn_is_active(state: State<MidiLearnState>) -> bool {
    state.0.is_learning()
}

#[tauri::command]
pub fn learn_list_mappings(state: State<MidiLearnState>) -> Vec<MidiMapping> {
    state.0.list_mappings()
}

#[tauri::command]
pub fn learn_remove_mapping(state: State<MidiLearnState>, mapping_id: String) {
    state.0.remove_mapping(&mapping_id);
}

#[tauri::command]
pub fn learn_export_mappings(state: State<MidiLearnState>) -> String {
    state.0.export_mappings()
}

#[tauri::command]
pub fn learn_import_mappings(state: State<MidiLearnState>, json: String) -> Result<usize, String> {
    state.0.import_mappings(&json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn learn_add_mapping(
    state: State<MidiLearnState>,
    name: String,
    device_id: String,
    message_type: MidiMessageType,
    channel: Option<u8>,
    data1: Option<u8>,
    target_path: String,
) {
    let source = MidiSource { device_id, message_type, channel, data1 };
    let target = MappingTarget::Parameter { path: target_path };
    let mapping = MidiMapping::new(&name, source, target);
    state.0.add_mapping(mapping);
}

#[tauri::command]
pub fn learn_update_mapping(state: State<MidiLearnState>, mapping: MidiMapping) {
    state.0.update_mapping(mapping);
}

#[tauri::command]
pub fn learn_clear_mappings(state: State<MidiLearnState>) {
    state.0.clear_mappings();
}

#[tauri::command]
pub fn learn_process_midi(
    state: State<MidiLearnState>,
    device_id: String,
    msg_type: MidiMessageType,
    channel: u8,
    data1: u8,
    data2: u8,
) {
    state.0.process_midi(&device_id, msg_type, channel, data1, data2);
}

/// Scale a MIDI value according to a scaling mode
#[tauri::command]
pub fn learn_scale_value(
    scaling_mode: ScalingMode,
    midi_value: u8,
    current_value: Option<f32>,
) -> f32 {
    scaling_mode.scale(midi_value, current_value)
}

/// Subscribe to MIDI Learn events via Tauri events
#[tauri::command]
pub async fn learn_subscribe(
    app: tauri::AppHandle,
    state: State<'_, MidiLearnState>,
) -> Result<(), String> {
    use tauri::Emitter;

    let mut rx = state.0.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let event_data = match event {
                LearnEvent::MappingCreated(m) => {
                    serde_json::json!({"type": "created", "mapping": m})
                },
                LearnEvent::MappingDeleted(id) => {
                    serde_json::json!({"type": "deleted", "id": id})
                },
                LearnEvent::MappingUpdated(m) => {
                    serde_json::json!({"type": "updated", "mapping": m})
                },
                LearnEvent::ParameterChanged { path, value } => {
                    serde_json::json!({"type": "param", "path": path, "value": value})
                },
                LearnEvent::TransportTriggered(action) => {
                    serde_json::json!({"type": "transport", "action": format!("{:?}", action)})
                },
                LearnEvent::LearnModeStarted { target } => {
                    serde_json::json!({"type": "learn_started", "target": format!("{:?}", target)})
                },
                LearnEvent::LearnModeCompleted { mapping } => {
                    serde_json::json!({"type": "learn_completed", "mapping": mapping})
                },
                LearnEvent::LearnModeCancelled => {
                    serde_json::json!({"type": "learn_cancelled"})
                },
            };

            if app.emit("midi-learn-event", event_data).is_err() {
                break;
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_scaling() {
        let mode = ScalingMode::Linear { min: 0.0, max: 100.0 };
        assert_eq!(mode.scale(0, None), 0.0);
        assert_eq!(mode.scale(127, None), 100.0);
        assert!((mode.scale(64, None) - 50.4).abs() < 0.5);
    }

    #[test]
    fn test_toggle_scaling() {
        let mode = ScalingMode::Toggle;
        assert_eq!(mode.scale(0, None), 0.0);
        assert_eq!(mode.scale(63, None), 0.0);
        assert_eq!(mode.scale(64, None), 1.0);
        assert_eq!(mode.scale(127, None), 1.0);
    }

    #[test]
    fn test_stepped_scaling() {
        let mode = ScalingMode::Stepped { values: vec![0.0, 0.5, 1.0] };
        assert_eq!(mode.scale(0, None), 0.0);
        assert_eq!(mode.scale(64, None), 0.5);
        assert_eq!(mode.scale(127, None), 1.0);
    }
}
