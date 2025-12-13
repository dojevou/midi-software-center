use serde::Serialize;
/// Tauri and MIDI mocking framework for testing IPC commands
/// Captures event emissions and provides MIDI device simulation
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Events emitted during command execution
#[derive(Debug, Clone, PartialEq)]
pub struct EmittedEvent {
    pub event_name: String,
    pub payload: String, // JSON serialized
}

/// Mock Tauri Window for testing event emission
#[derive(Clone)]
pub struct MockWindow {
    emitted_events: Arc<Mutex<Vec<EmittedEvent>>>,
}

impl MockWindow {
    pub fn new() -> Self {
        Self { emitted_events: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Mock emit method (matches Tauri signature)
    pub async fn emit<S: Serialize>(&self, event: &str, payload: S) -> Result<(), String> {
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| format!("Failed to serialize payload: {}", e))?;

        self.emitted_events
            .lock()
            .await
            .push(EmittedEvent { event_name: event.to_string(), payload: payload_json });

        Ok(())
    }

    /// Get all emitted events (for assertions)
    pub async fn get_emitted_events(&self) -> Vec<EmittedEvent> {
        self.emitted_events.lock().await.clone()
    }

    /// Assert event was emitted
    pub async fn assert_event_emitted(&self, event_name: &str) {
        let events = self.get_emitted_events().await;
        assert!(
            events.iter().any(|e| e.event_name == event_name),
            "Expected event '{}' not emitted. Emitted events: {:?}",
            event_name,
            events.iter().map(|e| &e.event_name).collect::<Vec<_>>()
        );
    }

    /// Assert event count
    pub async fn assert_event_count(&self, event_name: &str, expected_count: usize) {
        let events = self.get_emitted_events().await;
        let actual_count = events.iter().filter(|e| e.event_name == event_name).count();
        assert_eq!(
            actual_count, expected_count,
            "Expected {} '{}' events, got {}",
            expected_count, event_name, actual_count
        );
    }

    /// Clear all events (for reset between tests)
    pub async fn clear_events(&self) {
        self.emitted_events.lock().await.clear();
    }
}

impl Default for MockWindow {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock AppHandle (minimal, can be extended)
pub struct MockAppHandle {
    pub window: MockWindow,
}

impl MockAppHandle {
    pub fn new() -> Self {
        Self { window: MockWindow::new() }
    }
}

impl Default for MockAppHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// MIDI message sent to mock device
#[derive(Debug, Clone, PartialEq)]
pub struct MidiMessage {
    pub channel: u8,
    pub message_type: MidiMessageType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MidiMessageType {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
    ControlChange { controller: u8, value: u8 },
    PitchBend { value: u16 },
    ProgramChange { program: u8 },
}

/// Mock MIDI device for testing without hardware
pub struct MockMidiDevice {
    connected: Arc<RwLock<bool>>,
    device_name: Arc<RwLock<Option<String>>>,
    sent_messages: Arc<Mutex<Vec<MidiMessage>>>,
    latency_ms: Arc<RwLock<u32>>,
}

impl MockMidiDevice {
    pub fn new() -> Self {
        Self {
            connected: Arc::new(RwLock::new(false)),
            device_name: Arc::new(RwLock::new(None)),
            sent_messages: Arc::new(Mutex::new(Vec::new())),
            latency_ms: Arc::new(RwLock::new(5)),
        }
    }

    pub async fn connect(&self, device_name: &str) {
        *self.connected.write().await = true;
        *self.device_name.write().await = Some(device_name.to_string());
    }

    pub async fn disconnect(&self) {
        *self.connected.write().await = false;
        *self.device_name.write().await = None;
        self.sent_messages.lock().await.clear();
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    pub async fn current_device(&self) -> Option<String> {
        self.device_name.read().await.clone()
    }

    pub async fn send_note_on(&self, channel: u8, note: u8, velocity: u8) {
        if *self.connected.read().await {
            self.sent_messages.lock().await.push(MidiMessage {
                channel,
                message_type: MidiMessageType::NoteOn { note, velocity },
            });
        }
    }

    pub async fn send_note_off(&self, channel: u8, note: u8) {
        if *self.connected.read().await {
            self.sent_messages
                .lock()
                .await
                .push(MidiMessage { channel, message_type: MidiMessageType::NoteOff { note } });
        }
    }

    pub async fn send_control_change(&self, channel: u8, controller: u8, value: u8) {
        if *self.connected.read().await {
            self.sent_messages.lock().await.push(MidiMessage {
                channel,
                message_type: MidiMessageType::ControlChange { controller, value },
            });
        }
    }

    pub async fn send_pitch_bend(&self, channel: u8, value: u16) {
        if *self.connected.read().await {
            self.sent_messages
                .lock()
                .await
                .push(MidiMessage { channel, message_type: MidiMessageType::PitchBend { value } });
        }
    }

    pub async fn send_program_change(&self, channel: u8, program: u8) {
        if *self.connected.read().await {
            self.sent_messages.lock().await.push(MidiMessage {
                channel,
                message_type: MidiMessageType::ProgramChange { program },
            });
        }
    }

    pub async fn get_sent_messages(&self) -> Vec<MidiMessage> {
        self.sent_messages.lock().await.clone()
    }

    pub async fn clear_messages(&self) {
        self.sent_messages.lock().await.clear();
    }

    pub async fn set_latency(&self, latency_ms: u32) {
        *self.latency_ms.write().await = latency_ms;
    }

    pub async fn get_latency(&self) -> u32 {
        *self.latency_ms.read().await
    }

    /// Assert specific message was sent
    pub async fn assert_message_sent(&self, expected: MidiMessage) {
        let messages = self.get_sent_messages().await;
        assert!(
            messages.contains(&expected),
            "Expected message {:?} not found in sent messages: {:?}",
            expected,
            messages
        );
    }

    /// Assert message count
    pub async fn assert_message_count(&self, expected_count: usize) {
        let count = self.sent_messages.lock().await.len();
        assert_eq!(
            count, expected_count,
            "Expected {} messages, got {}",
            expected_count, count
        );
    }
}

impl Default for MockMidiDevice {
    fn default() -> Self {
        Self::new()
    }
}
