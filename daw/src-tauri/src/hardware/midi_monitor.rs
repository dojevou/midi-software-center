// MIDI Monitor - Real-time MIDI event monitoring and recording
// GROWN-UP SCRIPT - Event handling with proper state management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// MIDI message types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MidiMessageType {
    NoteOn,
    NoteOff,
    ControlChange,
    ProgramChange,
    PitchBend,
    Aftertouch,
    ChannelPressure,
    SystemExclusive,
    Unknown,
}

impl MidiMessageType {
    /// Parse message type from status byte
    pub fn from_status(status: u8) -> Self {
        match status & 0xF0 {
            0x80 => MidiMessageType::NoteOff,
            0x90 => MidiMessageType::NoteOn,
            0xA0 => MidiMessageType::Aftertouch,
            0xB0 => MidiMessageType::ControlChange,
            0xC0 => MidiMessageType::ProgramChange,
            0xD0 => MidiMessageType::ChannelPressure,
            0xE0 => MidiMessageType::PitchBend,
            0xF0 => MidiMessageType::SystemExclusive,
            _ => MidiMessageType::Unknown,
        }
    }
}

/// MIDI event captured by the monitor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MidiEvent {
    pub timestamp: f64,
    pub message_type: MidiMessageType,
    pub channel: u8,
    pub data1: u8,
    pub data2: u8,
    pub device_id: String,
}

impl MidiEvent {
    /// Create a new MIDI event
    pub fn new(
        timestamp: f64,
        status: u8,
        data1: u8,
        data2: u8,
        device_id: String,
    ) -> Self {
        let message_type = MidiMessageType::from_status(status);
        let channel = status & 0x0F;

        Self {
            timestamp,
            message_type,
            channel,
            data1,
            data2,
            device_id,
        }
    }

    /// Create a note on event
    pub fn note_on(timestamp: f64, channel: u8, note: u8, velocity: u8, device_id: String) -> Self {
        Self {
            timestamp,
            message_type: MidiMessageType::NoteOn,
            channel,
            data1: note,
            data2: velocity,
            device_id,
        }
    }

    /// Create a note off event
    pub fn note_off(timestamp: f64, channel: u8, note: u8, velocity: u8, device_id: String) -> Self {
        Self {
            timestamp,
            message_type: MidiMessageType::NoteOff,
            channel,
            data1: note,
            data2: velocity,
            device_id,
        }
    }

    /// Create a control change event
    pub fn control_change(timestamp: f64, channel: u8, controller: u8, value: u8, device_id: String) -> Self {
        Self {
            timestamp,
            message_type: MidiMessageType::ControlChange,
            channel,
            data1: controller,
            data2: value,
            device_id,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> String {
        match self.message_type {
            MidiMessageType::NoteOn => {
                format!("Note On: {} velocity {}", self.data1, self.data2)
            }
            MidiMessageType::NoteOff => {
                format!("Note Off: {} velocity {}", self.data1, self.data2)
            }
            MidiMessageType::ControlChange => {
                format!("CC {}: {}", self.data1, self.data2)
            }
            MidiMessageType::ProgramChange => {
                format!("Program Change: {}", self.data1)
            }
            MidiMessageType::PitchBend => {
                let bend_value = (self.data2 as u16) << 7 | (self.data1 as u16);
                format!("Pitch Bend: {}", bend_value)
            }
            MidiMessageType::Aftertouch => {
                format!("Aftertouch: note {} pressure {}", self.data1, self.data2)
            }
            MidiMessageType::ChannelPressure => {
                format!("Channel Pressure: {}", self.data1)
            }
            MidiMessageType::SystemExclusive => "System Exclusive".to_string(),
            MidiMessageType::Unknown => "Unknown".to_string(),
        }
    }
}

/// State for MIDI monitoring
#[derive(Debug, Clone)]
pub struct MidiMonitorState {
    pub is_recording: bool,
    pub event_history: Vec<MidiEvent>,
    pub max_history_size: usize,
}

impl MidiMonitorState {
    /// Create new monitor state
    pub fn new() -> Self {
        Self {
            is_recording: false,
            event_history: Vec::new(),
            max_history_size: 1000,
        }
    }

    /// Start monitoring/recording events
    pub fn start(&mut self) -> Result<()> {
        if self.is_recording {
            return Err(anyhow::anyhow!("Monitor is already recording"));
        }
        self.is_recording = true;
        Ok(())
    }

    /// Stop monitoring/recording events
    pub fn stop(&mut self) -> Result<()> {
        if !self.is_recording {
            return Err(anyhow::anyhow!("Monitor is not recording"));
        }
        self.is_recording = false;
        Ok(())
    }

    /// Add event to history
    pub fn add_event(&mut self, event: MidiEvent) {
        if !self.is_recording {
            return;
        }

        self.event_history.push(event);

        // Trim history if it exceeds max size
        if self.event_history.len() > self.max_history_size {
            let excess = self.event_history.len() - self.max_history_size;
            self.event_history.drain(0..excess);
        }
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.event_history.clear();
    }

    /// Get events with optional limit
    pub fn get_events(&self, limit: Option<usize>) -> Vec<MidiEvent> {
        if let Some(limit) = limit {
            let start = if self.event_history.len() > limit {
                self.event_history.len() - limit
            } else {
                0
            };
            self.event_history[start..].to_vec()
        } else {
            self.event_history.clone()
        }
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.event_history.len()
    }

    /// Set maximum history size
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;

        // Trim existing history if needed
        if self.event_history.len() > size {
            let excess = self.event_history.len() - size;
            self.event_history.drain(0..excess);
        }
    }
}

impl Default for MidiMonitorState {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri Commands

/// Start MIDI monitoring
#[tauri::command]
pub async fn start_monitoring(
    state: State<'_, Arc<RwLock<MidiMonitorState>>>,
) -> Result<(), String> {
    start_monitoring_impl(&state)
        .await
        .map_err(|e| e.to_string())
}

async fn start_monitoring_impl(
    state: &State<'_, Arc<RwLock<MidiMonitorState>>>,
) -> Result<()> {
    let mut monitor_state = state.write().await;
    monitor_state
        .start()
        .context("Failed to start monitoring")?;
    Ok(())
}

/// Stop MIDI monitoring
#[tauri::command]
pub async fn stop_monitoring(
    state: State<'_, Arc<RwLock<MidiMonitorState>>>,
) -> Result<(), String> {
    stop_monitoring_impl(&state)
        .await
        .map_err(|e| e.to_string())
}

async fn stop_monitoring_impl(
    state: &State<'_, Arc<RwLock<MidiMonitorState>>>,
) -> Result<()> {
    let mut monitor_state = state.write().await;
    monitor_state.stop().context("Failed to stop monitoring")?;
    Ok(())
}

/// Clear all recorded events
#[tauri::command]
pub async fn clear_events(
    state: State<'_, Arc<RwLock<MidiMonitorState>>>,
) -> Result<(), String> {
    clear_events_impl(&state).await.map_err(|e| e.to_string())
}

async fn clear_events_impl(
    state: &State<'_, Arc<RwLock<MidiMonitorState>>>,
) -> Result<()> {
    let mut monitor_state = state.write().await;
    monitor_state.clear();
    Ok(())
}

/// Get recorded events with optional limit
#[tauri::command]
pub async fn get_events(
    limit: i32,
    state: State<'_, Arc<RwLock<MidiMonitorState>>>,
) -> Result<Vec<MidiEvent>, String> {
    get_events_impl(limit, &state)
        .await
        .map_err(|e| e.to_string())
}

async fn get_events_impl(
    limit: i32,
    state: &State<'_, Arc<RwLock<MidiMonitorState>>>,
) -> Result<Vec<MidiEvent>> {
    let monitor_state = state.read().await;

    let limit_opt = if limit > 0 {
        Some(limit as usize)
    } else {
        None
    };

    Ok(monitor_state.get_events(limit_opt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_from_status() {
        assert_eq!(
            MidiMessageType::from_status(0x90),
            MidiMessageType::NoteOn
        );
        assert_eq!(
            MidiMessageType::from_status(0x80),
            MidiMessageType::NoteOff
        );
        assert_eq!(
            MidiMessageType::from_status(0xB0),
            MidiMessageType::ControlChange
        );
        assert_eq!(
            MidiMessageType::from_status(0xC0),
            MidiMessageType::ProgramChange
        );
        assert_eq!(
            MidiMessageType::from_status(0xE0),
            MidiMessageType::PitchBend
        );
    }

    #[test]
    fn test_midi_event_creation() {
        let event = MidiEvent::new(1.0, 0x90, 60, 100, "device-1".to_string());

        assert_eq!(event.timestamp, 1.0);
        assert_eq!(event.message_type, MidiMessageType::NoteOn);
        assert_eq!(event.channel, 0);
        assert_eq!(event.data1, 60);
        assert_eq!(event.data2, 100);
        assert_eq!(event.device_id, "device-1");
    }

    #[test]
    fn test_midi_event_note_on() {
        let event = MidiEvent::note_on(1.0, 0, 60, 100, "device-1".to_string());

        assert_eq!(event.message_type, MidiMessageType::NoteOn);
        assert_eq!(event.data1, 60);
        assert_eq!(event.data2, 100);
    }

    #[test]
    fn test_midi_event_note_off() {
        let event = MidiEvent::note_off(1.0, 0, 60, 0, "device-1".to_string());

        assert_eq!(event.message_type, MidiMessageType::NoteOff);
        assert_eq!(event.data1, 60);
        assert_eq!(event.data2, 0);
    }

    #[test]
    fn test_midi_event_control_change() {
        let event = MidiEvent::control_change(1.0, 0, 1, 64, "device-1".to_string());

        assert_eq!(event.message_type, MidiMessageType::ControlChange);
        assert_eq!(event.data1, 1);
        assert_eq!(event.data2, 64);
    }

    #[test]
    fn test_midi_event_description() {
        let note_on = MidiEvent::note_on(1.0, 0, 60, 100, "device-1".to_string());
        assert_eq!(note_on.description(), "Note On: 60 velocity 100");

        let cc = MidiEvent::control_change(1.0, 0, 1, 64, "device-1".to_string());
        assert_eq!(cc.description(), "CC 1: 64");
    }

    #[test]
    fn test_monitor_state_creation() {
        let state = MidiMonitorState::new();

        assert!(!state.is_recording);
        assert!(state.event_history.is_empty());
        assert_eq!(state.max_history_size, 1000);
    }

    #[test]
    fn test_start_monitoring() {
        let mut state = MidiMonitorState::new();

        let result = state.start();
        assert!(result.is_ok());
        assert!(state.is_recording);
    }

    #[test]
    fn test_start_already_recording() {
        let mut state = MidiMonitorState::new();
        state.start().unwrap();

        let result = state.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_stop_monitoring() {
        let mut state = MidiMonitorState::new();
        state.start().unwrap();

        let result = state.stop();
        assert!(result.is_ok());
        assert!(!state.is_recording);
    }

    #[test]
    fn test_stop_not_recording() {
        let mut state = MidiMonitorState::new();

        let result = state.stop();
        assert!(result.is_err());
    }

    #[test]
    fn test_add_event() {
        let mut state = MidiMonitorState::new();
        state.start().unwrap();

        let event = MidiEvent::note_on(1.0, 0, 60, 100, "device-1".to_string());
        state.add_event(event);

        assert_eq!(state.event_count(), 1);
    }

    #[test]
    fn test_add_event_not_recording() {
        let mut state = MidiMonitorState::new();

        let event = MidiEvent::note_on(1.0, 0, 60, 100, "device-1".to_string());
        state.add_event(event);

        assert_eq!(state.event_count(), 0);
    }

    #[test]
    fn test_clear_events() {
        let mut state = MidiMonitorState::new();
        state.start().unwrap();

        let event = MidiEvent::note_on(1.0, 0, 60, 100, "device-1".to_string());
        state.add_event(event);

        state.clear();
        assert_eq!(state.event_count(), 0);
    }

    #[test]
    fn test_get_events() {
        let mut state = MidiMonitorState::new();
        state.start().unwrap();

        for i in 0..5 {
            let event = MidiEvent::note_on(i as f64, 0, 60, 100, "device-1".to_string());
            state.add_event(event);
        }

        let events = state.get_events(None);
        assert_eq!(events.len(), 5);

        let events = state.get_events(Some(3));
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_max_history_size() {
        let mut state = MidiMonitorState::new();
        state.set_max_history_size(5);
        state.start().unwrap();

        for i in 0..10 {
            let event = MidiEvent::note_on(i as f64, 0, 60, 100, "device-1".to_string());
            state.add_event(event);
        }

        assert_eq!(state.event_count(), 5);
        let events = state.get_events(None);
        assert_eq!(events[0].timestamp, 5.0); // First event should be timestamp 5
    }

    #[test]
    fn test_set_max_history_size_trims_existing() {
        let mut state = MidiMonitorState::new();
        state.start().unwrap();

        for i in 0..10 {
            let event = MidiEvent::note_on(i as f64, 0, 60, 100, "device-1".to_string());
            state.add_event(event);
        }

        state.set_max_history_size(5);
        assert_eq!(state.event_count(), 5);
    }

    #[tokio::test]
    async fn test_start_monitoring_impl() {
        let state = Arc::new(RwLock::new(MidiMonitorState::new()));
        let tauri_state = State::from(&state);

        let result = start_monitoring_impl(&tauri_state).await;
        assert!(result.is_ok());

        let s = state.read().await;
        assert!(s.is_recording);
    }

    #[tokio::test]
    async fn test_stop_monitoring_impl() {
        let state = Arc::new(RwLock::new(MidiMonitorState::new()));
        {
            let mut s = state.write().await;
            s.start().unwrap();
        }

        let tauri_state = State::from(&state);
        let result = stop_monitoring_impl(&tauri_state).await;
        assert!(result.is_ok());

        let s = state.read().await;
        assert!(!s.is_recording);
    }

    #[tokio::test]
    async fn test_clear_events_impl() {
        let state = Arc::new(RwLock::new(MidiMonitorState::new()));
        {
            let mut s = state.write().await;
            s.start().unwrap();
            let event = MidiEvent::note_on(1.0, 0, 60, 100, "device-1".to_string());
            s.add_event(event);
        }

        let tauri_state = State::from(&state);
        let result = clear_events_impl(&tauri_state).await;
        assert!(result.is_ok());

        let s = state.read().await;
        assert_eq!(s.event_count(), 0);
    }

    #[tokio::test]
    async fn test_get_events_impl() {
        let state = Arc::new(RwLock::new(MidiMonitorState::new()));
        {
            let mut s = state.write().await;
            s.start().unwrap();
            for i in 0..5 {
                let event = MidiEvent::note_on(i as f64, 0, 60, 100, "device-1".to_string());
                s.add_event(event);
            }
        }

        let tauri_state = State::from(&state);
        let events = get_events_impl(-1, &tauri_state).await;
        assert!(events.is_ok());
        assert_eq!(events.unwrap().len(), 5);

        let events = get_events_impl(3, &tauri_state).await;
        assert!(events.is_ok());
        assert_eq!(events.unwrap().len(), 3);
    }
}
