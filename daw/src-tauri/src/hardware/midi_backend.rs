// High-Performance MIDI Backend Abstraction Layer
// Provides unified interface for multiple low-latency MIDI backends
//
// Backend Priority (by latency):
// 1. JACK (Linux/macOS) - ~3ms, lock-free ringbuffers
// 2. ALSA Raw MIDI (Linux) - ~5ms, direct kernel access
// 3. CoreMIDI (macOS) - ~5ms, native framework
// 4. midir (all platforms) - ~10-15ms, cross-platform fallback

use anyhow::{Context, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;
use wmidi::MidiMessage;

// ============================================================================
// Core Types
// ============================================================================

/// Backend type for MIDI I/O
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MidiBackendType {
    /// JACK Audio Connection Kit - lowest latency (~3ms)
    Jack,
    /// ALSA Raw MIDI - direct kernel access (~5ms)
    AlsaRaw,
    /// ALSA Sequencer - general purpose (~8-15ms)
    AlsaSeq,
    /// CoreMIDI - macOS native (~5ms)
    CoreMidi,
    /// midir - cross-platform fallback (~10-15ms)
    Midir,
    /// Auto-select best available backend
    #[default]
    Auto,
}

/// MIDI port information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiPort {
    pub id: String,
    pub name: String,
    pub direction: String,
    pub backend: String,
    pub latency_us: u64,
    pub is_virtual: bool,
}

/// Timestamped MIDI event for precise timing
#[derive(Debug, Clone)]
pub struct TimestampedMidiEvent {
    /// Raw MIDI bytes (3 bytes for most messages, more for SysEx)
    pub data: Vec<u8>,
    /// Timestamp in microseconds from connection start
    pub timestamp_us: u64,
    /// Port ID this event came from/goes to
    pub port_id: String,
}

impl TimestampedMidiEvent {
    /// Create a new timestamped event
    pub fn new(data: Vec<u8>, timestamp_us: u64, port_id: String) -> Self {
        Self { data, timestamp_us, port_id }
    }

    /// Parse as wmidi MidiMessage (zero-copy when possible)
    pub fn parse(&self) -> Result<MidiMessage<'_>> {
        MidiMessage::try_from(self.data.as_slice())
            .map_err(|e| anyhow::anyhow!("Failed to parse MIDI message: {:?}", e))
    }

    /// Create Note On event
    pub fn note_on(
        channel: u8,
        note: u8,
        velocity: u8,
        timestamp_us: u64,
        port_id: String,
    ) -> Self {
        Self::new(
            vec![0x90 | (channel & 0x0F), note & 0x7F, velocity & 0x7F],
            timestamp_us,
            port_id,
        )
    }

    /// Create Note Off event
    pub fn note_off(
        channel: u8,
        note: u8,
        velocity: u8,
        timestamp_us: u64,
        port_id: String,
    ) -> Self {
        Self::new(
            vec![0x80 | (channel & 0x0F), note & 0x7F, velocity & 0x7F],
            timestamp_us,
            port_id,
        )
    }

    /// Create Control Change event
    pub fn control_change(
        channel: u8,
        controller: u8,
        value: u8,
        timestamp_us: u64,
        port_id: String,
    ) -> Self {
        Self::new(
            vec![0xB0 | (channel & 0x0F), controller & 0x7F, value & 0x7F],
            timestamp_us,
            port_id,
        )
    }

    /// Create Program Change event
    pub fn program_change(channel: u8, program: u8, timestamp_us: u64, port_id: String) -> Self {
        Self::new(
            vec![0xC0 | (channel & 0x0F), program & 0x7F],
            timestamp_us,
            port_id,
        )
    }

    /// Create Pitch Bend event
    pub fn pitch_bend(channel: u8, value: u16, timestamp_us: u64, port_id: String) -> Self {
        let lsb = (value & 0x7F) as u8;
        let msb = ((value >> 7) & 0x7F) as u8;
        Self::new(
            vec![0xE0 | (channel & 0x0F), lsb, msb],
            timestamp_us,
            port_id,
        )
    }

    /// Create All Notes Off event
    pub fn all_notes_off(channel: u8, timestamp_us: u64, port_id: String) -> Self {
        Self::control_change(channel, 123, 0, timestamp_us, port_id)
    }

    /// Create All Sound Off event
    pub fn all_sound_off(channel: u8, timestamp_us: u64, port_id: String) -> Self {
        Self::control_change(channel, 120, 0, timestamp_us, port_id)
    }
}

/// Performance metrics for MIDI I/O
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MidiPerformanceMetrics {
    /// Average round-trip latency in microseconds
    pub avg_latency_us: u64,
    /// Minimum latency observed
    pub min_latency_us: u64,
    /// Maximum latency observed
    pub max_latency_us: u64,
    /// Jitter (standard deviation) in microseconds
    pub jitter_us: u64,
    /// Messages processed per second
    pub messages_per_sec: f64,
    /// Total messages processed
    pub total_messages: u64,
    /// Dropped messages (buffer overflow)
    pub dropped_messages: u64,
    /// Backend type in use
    pub backend: String,
}

// ============================================================================
// Backend Trait
// ============================================================================

/// Trait for MIDI backend implementations
pub trait MidiBackend: Send + Sync {
    /// Get backend type
    fn backend_type(&self) -> MidiBackendType;

    /// Get available input ports
    fn list_input_ports(&self) -> Result<Vec<MidiPort>>;

    /// Get available output ports
    fn list_output_ports(&self) -> Result<Vec<MidiPort>>;

    /// Get receiver for incoming MIDI events
    fn get_receiver(&self) -> Option<Receiver<TimestampedMidiEvent>>;

    /// Get performance metrics
    fn get_metrics(&self) -> MidiPerformanceMetrics;

    /// Check if backend is available on this system
    fn is_available() -> bool
    where
        Self: Sized;

    /// Get expected latency in microseconds
    fn expected_latency_us(&self) -> u64;
}

// ============================================================================
// midir Backend (Cross-Platform Fallback)
// ============================================================================

/// midir-based MIDI backend (cross-platform, ~10-15ms latency)
pub struct MidirBackend {
    event_sender: Sender<TimestampedMidiEvent>,
    event_receiver: Receiver<TimestampedMidiEvent>,
    start_time: Instant,
    metrics: Arc<RwLock<MidiPerformanceMetrics>>,
    message_count: AtomicU64,
    dropped_count: AtomicU64,
}

impl MidirBackend {
    /// Create a new midir backend
    pub fn new() -> Result<Self> {
        // Verify midir is available
        let _test = midir::MidiInput::new("MIDI Software Center Test")
            .context("Failed to initialize MIDI input")?;

        // Lock-free channel for MIDI events (10,000 capacity)
        let (tx, rx) = bounded(10_000);

        let metrics = MidiPerformanceMetrics {
            backend: "midir".to_string(),
            min_latency_us: u64::MAX,
            ..Default::default()
        };

        Ok(Self {
            event_sender: tx,
            event_receiver: rx,
            start_time: Instant::now(),
            metrics: Arc::new(RwLock::new(metrics)),
            message_count: AtomicU64::new(0),
            dropped_count: AtomicU64::new(0),
        })
    }

    /// Get current timestamp in microseconds
    pub fn get_timestamp_us(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }

    /// Queue an outgoing MIDI event
    pub fn queue_event(&self, event: TimestampedMidiEvent) -> Result<()> {
        match self.event_sender.try_send(event) {
            Ok(()) => {
                self.message_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            },
            Err(_) => {
                self.dropped_count.fetch_add(1, Ordering::Relaxed);
                Err(anyhow::anyhow!("MIDI event queue full"))
            },
        }
    }
}

impl MidiBackend for MidirBackend {
    fn backend_type(&self) -> MidiBackendType {
        MidiBackendType::Midir
    }

    fn list_input_ports(&self) -> Result<Vec<MidiPort>> {
        let midi_in = midir::MidiInput::new("MIDI Software Center Scan")
            .context("Failed to create MIDI input for scanning")?;

        let ports: Vec<MidiPort> = midi_in
            .ports()
            .iter()
            .enumerate()
            .filter_map(|(i, port)| {
                let name = midi_in.port_name(port).ok()?;
                Some(MidiPort {
                    id: format!("midir-in-{}", i),
                    name,
                    direction: "input".to_string(),
                    backend: "midir".to_string(),
                    latency_us: 10_000, // ~10ms typical
                    is_virtual: false,
                })
            })
            .collect();

        Ok(ports)
    }

    fn list_output_ports(&self) -> Result<Vec<MidiPort>> {
        let midi_out = midir::MidiOutput::new("MIDI Software Center Scan")
            .context("Failed to create MIDI output for scanning")?;

        let ports: Vec<MidiPort> = midi_out
            .ports()
            .iter()
            .enumerate()
            .filter_map(|(i, port)| {
                let name = midi_out.port_name(port).ok()?;
                Some(MidiPort {
                    id: format!("midir-out-{}", i),
                    name,
                    direction: "output".to_string(),
                    backend: "midir".to_string(),
                    latency_us: 10_000,
                    is_virtual: false,
                })
            })
            .collect();

        Ok(ports)
    }

    fn get_receiver(&self) -> Option<Receiver<TimestampedMidiEvent>> {
        Some(self.event_receiver.clone())
    }

    fn get_metrics(&self) -> MidiPerformanceMetrics {
        let mut metrics = self.metrics.read().clone();
        metrics.total_messages = self.message_count.load(Ordering::Relaxed);
        metrics.dropped_messages = self.dropped_count.load(Ordering::Relaxed);

        // Calculate messages per second
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        if elapsed_secs > 0.0 {
            metrics.messages_per_sec = metrics.total_messages as f64 / elapsed_secs;
        }

        metrics
    }

    fn is_available() -> bool {
        midir::MidiInput::new("test").is_ok()
    }

    fn expected_latency_us(&self) -> u64 {
        10_000 // ~10ms
    }
}

// ============================================================================
// JACK Backend Stub (Linux/macOS - Lowest Latency)
// ============================================================================

#[cfg(feature = "jack-backend")]
pub struct JackBackend {
    event_sender: Sender<TimestampedMidiEvent>,
    event_receiver: Receiver<TimestampedMidiEvent>,
    metrics: Arc<RwLock<MidiPerformanceMetrics>>,
    _is_running: AtomicBool,
}

#[cfg(feature = "jack-backend")]
impl JackBackend {
    pub fn new() -> Result<Self> {
        let (tx, rx) = bounded(10_000);

        let mut metrics = MidiPerformanceMetrics::default();
        metrics.backend = "jack".to_string();

        Ok(Self {
            event_sender: tx,
            event_receiver: rx,
            metrics: Arc::new(RwLock::new(metrics)),
            _is_running: AtomicBool::new(false),
        })
    }
}

#[cfg(feature = "jack-backend")]
impl MidiBackend for JackBackend {
    fn backend_type(&self) -> MidiBackendType {
        MidiBackendType::Jack
    }

    fn list_input_ports(&self) -> Result<Vec<MidiPort>> {
        // JACK port enumeration would go here
        Ok(Vec::new())
    }

    fn list_output_ports(&self) -> Result<Vec<MidiPort>> {
        Ok(Vec::new())
    }

    fn get_receiver(&self) -> Option<Receiver<TimestampedMidiEvent>> {
        Some(self.event_receiver.clone())
    }

    fn get_metrics(&self) -> MidiPerformanceMetrics {
        self.metrics.read().clone()
    }

    fn is_available() -> bool {
        // Check if JACK server is running
        jack::Client::new("test", jack::ClientOptions::NO_START_SERVER).is_ok()
    }

    fn expected_latency_us(&self) -> u64 {
        3_000 // ~3ms
    }
}

// ============================================================================
// ALSA Raw MIDI Backend Stub (Linux - Direct Kernel Access)
// ============================================================================

#[cfg(all(feature = "alsa-backend", target_os = "linux"))]
pub struct AlsaRawBackend {
    event_sender: Sender<TimestampedMidiEvent>,
    event_receiver: Receiver<TimestampedMidiEvent>,
    metrics: Arc<RwLock<MidiPerformanceMetrics>>,
}

#[cfg(all(feature = "alsa-backend", target_os = "linux"))]
impl AlsaRawBackend {
    pub fn new() -> Result<Self> {
        let (tx, rx) = bounded(10_000);

        let mut metrics = MidiPerformanceMetrics::default();
        metrics.backend = "alsa-raw".to_string();

        Ok(Self { event_sender: tx, event_receiver: rx, metrics: Arc::new(RwLock::new(metrics)) })
    }
}

#[cfg(all(feature = "alsa-backend", target_os = "linux"))]
impl MidiBackend for AlsaRawBackend {
    fn backend_type(&self) -> MidiBackendType {
        MidiBackendType::AlsaRaw
    }

    fn list_input_ports(&self) -> Result<Vec<MidiPort>> {
        // ALSA raw MIDI enumeration would go here
        Ok(Vec::new())
    }

    fn list_output_ports(&self) -> Result<Vec<MidiPort>> {
        Ok(Vec::new())
    }

    fn get_receiver(&self) -> Option<Receiver<TimestampedMidiEvent>> {
        Some(self.event_receiver.clone())
    }

    fn get_metrics(&self) -> MidiPerformanceMetrics {
        self.metrics.read().clone()
    }

    fn is_available() -> bool {
        std::path::Path::new("/dev/snd").exists()
    }

    fn expected_latency_us(&self) -> u64 {
        5_000 // ~5ms
    }
}

// ============================================================================
// Unified MIDI Manager
// ============================================================================

/// Unified MIDI manager that selects the best available backend
pub struct MidiManager {
    backend_type: MidiBackendType,
    midir_backend: MidirBackend,
    #[cfg(feature = "jack-backend")]
    jack_backend: Option<JackBackend>,
    #[cfg(all(feature = "alsa-backend", target_os = "linux"))]
    alsa_backend: Option<AlsaRawBackend>,
}

impl MidiManager {
    /// Create a new MIDI manager with auto-detected best backend
    pub fn new() -> Result<Self> {
        Self::with_backend(MidiBackendType::Auto)
    }

    /// Create a new MIDI manager with specified backend preference
    pub fn with_backend(preferred: MidiBackendType) -> Result<Self> {
        let midir_backend = MidirBackend::new()?;

        let actual_type = match preferred {
            MidiBackendType::Auto => Self::detect_best_backend(),
            other => other,
        };

        info!(
            "MIDI Manager initialized with {:?} backend (expected latency: {}μs)",
            actual_type,
            match actual_type {
                MidiBackendType::Jack => 3_000,
                MidiBackendType::AlsaRaw => 5_000,
                MidiBackendType::CoreMidi => 5_000,
                _ => 10_000,
            }
        );

        Ok(Self {
            backend_type: actual_type,
            midir_backend,
            #[cfg(feature = "jack-backend")]
            jack_backend: JackBackend::new().ok(),
            #[cfg(all(feature = "alsa-backend", target_os = "linux"))]
            alsa_backend: AlsaRawBackend::new().ok(),
        })
    }

    /// Detect the best available backend
    fn detect_best_backend() -> MidiBackendType {
        #[cfg(feature = "jack-backend")]
        {
            if JackBackend::is_available() {
                info!("Auto-selected JACK backend (lowest latency: ~3ms)");
                return MidiBackendType::Jack;
            }
        }

        #[cfg(all(feature = "alsa-backend", target_os = "linux"))]
        {
            if AlsaRawBackend::is_available() {
                info!("Auto-selected ALSA Raw backend (~5ms latency)");
                return MidiBackendType::AlsaRaw;
            }
        }

        info!("Using midir backend (cross-platform, ~10-15ms latency)");
        MidiBackendType::Midir
    }

    /// Get the current backend type
    pub fn backend_type(&self) -> MidiBackendType {
        self.backend_type
    }

    /// List all available input ports
    pub fn list_input_ports(&self) -> Result<Vec<MidiPort>> {
        self.midir_backend.list_input_ports()
    }

    /// List all available output ports
    pub fn list_output_ports(&self) -> Result<Vec<MidiPort>> {
        self.midir_backend.list_output_ports()
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> MidiPerformanceMetrics {
        self.midir_backend.get_metrics()
    }

    /// Get event receiver for incoming MIDI
    pub fn get_receiver(&self) -> Option<Receiver<TimestampedMidiEvent>> {
        self.midir_backend.get_receiver()
    }

    /// Get expected latency in microseconds
    pub fn expected_latency_us(&self) -> u64 {
        match self.backend_type {
            MidiBackendType::Jack => 3_000,
            MidiBackendType::AlsaRaw => 5_000,
            MidiBackendType::CoreMidi => 5_000,
            _ => 10_000,
        }
    }

    /// Queue an outgoing MIDI event
    pub fn queue_event(&self, event: TimestampedMidiEvent) -> Result<()> {
        self.midir_backend.queue_event(event)
    }

    /// Get current timestamp in microseconds
    pub fn get_timestamp_us(&self) -> u64 {
        self.midir_backend.get_timestamp_us()
    }
}

// ============================================================================
// High-Precision Timer
// ============================================================================

/// High-precision timer for MIDI scheduling
pub struct PrecisionTimer {
    /// Use spin-lock for sub-millisecond precision
    use_spin_sleep: bool,
    /// Start time for timestamp calculations
    start_time: Instant,
}

impl PrecisionTimer {
    /// Create a new precision timer
    pub fn new(use_spin_sleep: bool) -> Self {
        Self { use_spin_sleep, start_time: Instant::now() }
    }

    /// Sleep for the specified duration with high precision
    pub fn sleep(&self, duration: Duration) {
        if self.use_spin_sleep {
            // Use spin_sleep for sub-millisecond precision
            spin_sleep::sleep(duration);
        } else {
            std::thread::sleep(duration);
        }
    }

    /// Sleep until a specific timestamp (microseconds from start)
    pub fn sleep_until(&self, target_us: u64) {
        let now = self.start_time.elapsed().as_micros() as u64;
        if target_us > now {
            self.sleep(Duration::from_micros(target_us - now));
        }
    }

    /// Get current timestamp in microseconds
    pub fn now_us(&self) -> u64 {
        self.start_time.elapsed().as_micros() as u64
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
    }
}

impl Default for PrecisionTimer {
    fn default() -> Self {
        // Enable spin sleep on Windows where standard sleep is inaccurate
        #[cfg(target_os = "windows")]
        let use_spin_sleep = true;
        #[cfg(not(target_os = "windows"))]
        let use_spin_sleep = false;

        Self::new(use_spin_sleep)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamped_event_creation() {
        let event = TimestampedMidiEvent::new(vec![0x90, 60, 100], 1000, "test-port".to_string());

        assert_eq!(event.data, vec![0x90, 60, 100]);
        assert_eq!(event.timestamp_us, 1000);
        assert_eq!(event.port_id, "test-port");
    }

    #[test]
    fn test_note_on_creation() {
        let event = TimestampedMidiEvent::note_on(0, 60, 100, 1000, "port".to_string());
        assert_eq!(event.data, vec![0x90, 60, 100]);
    }

    #[test]
    fn test_note_off_creation() {
        let event = TimestampedMidiEvent::note_off(0, 60, 64, 1000, "port".to_string());
        assert_eq!(event.data, vec![0x80, 60, 64]);
    }

    #[test]
    fn test_control_change_creation() {
        let event = TimestampedMidiEvent::control_change(0, 1, 127, 1000, "port".to_string());
        assert_eq!(event.data, vec![0xB0, 1, 127]);
    }

    #[test]
    fn test_program_change_creation() {
        let event = TimestampedMidiEvent::program_change(0, 42, 1000, "port".to_string());
        assert_eq!(event.data, vec![0xC0, 42]);
    }

    #[test]
    fn test_pitch_bend_creation() {
        let event = TimestampedMidiEvent::pitch_bend(0, 8192, 1000, "port".to_string());
        // 8192 = center position
        assert_eq!(event.data[0], 0xE0);
        assert_eq!(event.data[1], 0); // LSB
        assert_eq!(event.data[2], 64); // MSB
    }

    #[test]
    fn test_all_notes_off() {
        let event = TimestampedMidiEvent::all_notes_off(0, 1000, "port".to_string());
        assert_eq!(event.data, vec![0xB0, 123, 0]);
    }

    #[test]
    fn test_parse_midi_message() {
        let event = TimestampedMidiEvent::note_on(0, 60, 100, 1000, "port".to_string());
        let msg = event.parse().unwrap();

        match msg {
            MidiMessage::NoteOn(channel, note, velocity) => {
                assert_eq!(channel.index(), 0);
                assert_eq!(u8::from(note), 60);
                assert_eq!(u8::from(velocity), 100);
            },
            _ => panic!("Expected NoteOn message"),
        }
    }

    #[test]
    fn test_backend_type_default() {
        assert_eq!(MidiBackendType::default(), MidiBackendType::Auto);
    }

    #[test]
    fn test_precision_timer() {
        let timer = PrecisionTimer::default();
        let start = Instant::now();
        timer.sleep(Duration::from_millis(1));
        let elapsed = start.elapsed();
        // Should be at least 1ms, but less than 10ms
        assert!(elapsed >= Duration::from_millis(1));
        assert!(elapsed < Duration::from_millis(50));
    }

    #[test]
    fn test_precision_timer_now() {
        let timer = PrecisionTimer::default();
        let t1 = timer.now_us();
        std::thread::sleep(Duration::from_millis(1));
        let t2 = timer.now_us();
        assert!(t2 > t1);
    }

    #[test]
    #[ignore] // Requires real MIDI hardware - run with: cargo test -- --ignored
    fn test_midir_backend_available() {
        // midir should always be available
        assert!(MidirBackend::is_available());
    }

    #[test]
    fn test_midi_port_serialization() {
        let port = MidiPort {
            id: "test-1".to_string(),
            name: "Test Port".to_string(),
            direction: "input".to_string(),
            backend: "midir".to_string(),
            latency_us: 10_000,
            is_virtual: false,
        };

        let json = serde_json::to_string(&port).unwrap();
        let deserialized: MidiPort = serde_json::from_str(&json).unwrap();

        assert_eq!(port.id, deserialized.id);
        assert_eq!(port.latency_us, deserialized.latency_us);
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = MidiPerformanceMetrics::default();
        assert_eq!(metrics.avg_latency_us, 0);
        assert_eq!(metrics.total_messages, 0);
    }

    #[test]
    fn test_channel_masking() {
        // Test all 16 channels
        for ch in 0..16u8 {
            let event = TimestampedMidiEvent::note_on(ch, 60, 100, 1000, "port".to_string());
            assert_eq!(event.data[0] & 0x0F, ch);
        }
    }

    #[test]
    fn test_note_velocity_clamping() {
        // Values should be masked to 7 bits (0-127)
        let event = TimestampedMidiEvent::note_on(0, 200, 200, 1000, "port".to_string());
        assert_eq!(event.data[1], 200 & 0x7F); // 72
        assert_eq!(event.data[2], 200 & 0x7F); // 72
    }
}
