#![allow(dead_code)]
// =============================================================================
// MIDI FILE EXPORT
// =============================================================================
// Export MIDI data to Standard MIDI File (SMF) format.
//
// CLAUDE CODE INSTRUCTIONS:
// 1. Location: daw/src-tauri/src/midi/export.rs
// 2. Supports SMF Type 0 (single track) and Type 1 (multi-track)
// 3. Includes tempo, time signature, key signature meta events
// 4. Track naming and text events support
//
// FEATURES:
// - Export to SMF Type 0 or Type 1
// - Configurable tempo and time signature
// - Key signature support
// - Track naming and text events
// - SMPTE timing support
// =============================================================================

use crate::models::midi::{MidiEvent, MidiEventType};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, TrackEvent, TrackEventKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tauri::command;
use tracing::{debug, info};

// =============================================================================
// TYPES AND CONFIGURATION
// =============================================================================

/// SMF format type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SmfFormat {
    /// Type 0: All events in a single track
    #[default]
    Type0,
    /// Type 1: Multiple parallel tracks
    Type1,
}

impl From<SmfFormat> for Format {
    fn from(format: SmfFormat) -> Self {
        match format {
            SmfFormat::Type0 => Format::SingleTrack,
            SmfFormat::Type1 => Format::Parallel,
        }
    }
}

/// Key signature mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum KeyMode {
    #[default]
    Major,
    Minor,
}

/// Key signature for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeySignature {
    /// Number of sharps (positive) or flats (negative), -7 to 7
    pub key: i8,
    /// Major or minor
    pub mode: KeyMode,
}

impl Default for KeySignature {
    fn default() -> Self {
        Self {
            key: 0, // C
            mode: KeyMode::Major,
        }
    }
}

/// Time signature for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSignature {
    /// Numerator (beats per measure)
    pub numerator: u8,
    /// Denominator as power of 2 (4 = quarter note, 8 = eighth note)
    pub denominator: u8,
    /// MIDI clocks per metronome tick (usually 24)
    pub clocks_per_click: u8,
    /// Number of 32nd notes per quarter note (usually 8)
    pub thirty_seconds_per_quarter: u8,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self {
            numerator: 4,
            denominator: 4,
            clocks_per_click: 24,
            thirty_seconds_per_quarter: 8,
        }
    }
}

/// Track metadata for export
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrackMetadata {
    /// Track name
    pub name: Option<String>,
    /// Instrument name
    pub instrument: Option<String>,
    /// Copyright notice (usually only on track 0)
    pub copyright: Option<String>,
    /// Text annotations
    pub text: Vec<String>,
    /// MIDI channel (0-15)
    pub channel: u8,
}

/// Export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Output format (Type 0 or Type 1)
    pub format: SmfFormat,
    /// Ticks per quarter note (resolution)
    pub ticks_per_quarter: u16,
    /// Tempo in BPM
    pub tempo_bpm: f64,
    /// Time signature
    pub time_signature: TimeSignature,
    /// Key signature (optional)
    pub key_signature: Option<KeySignature>,
    /// Include track names
    pub include_track_names: bool,
    /// Include text events
    pub include_text_events: bool,
    /// Track metadata (indexed by track number)
    pub track_metadata: HashMap<u8, TrackMetadata>,
    /// Global copyright notice
    pub copyright: Option<String>,
    /// Sequence name
    pub sequence_name: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: SmfFormat::Type0,
            ticks_per_quarter: 480,
            tempo_bpm: 120.0,
            time_signature: TimeSignature::default(),
            key_signature: None,
            include_track_names: true,
            include_text_events: true,
            track_metadata: HashMap::new(),
            copyright: None,
            sequence_name: None,
        }
    }
}

/// Export result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// Path to exported file
    pub path: String,
    /// Number of bytes written
    pub bytes_written: usize,
    /// Number of tracks exported
    pub num_tracks: u16,
    /// Number of events exported
    pub num_events: usize,
}

/// Export error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportError {
    pub message: String,
    pub kind: ExportErrorKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportErrorKind {
    InvalidPath,
    IoError,
    InvalidData,
    EncodingError,
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for ExportError {}

// =============================================================================
// MIDI EXPORTER
// =============================================================================

/// MIDI file exporter
pub struct MidiExporter {
    options: ExportOptions,
}

impl MidiExporter {
    /// Create a new exporter with default options
    pub fn new() -> Self {
        Self {
            options: ExportOptions::default(),
        }
    }

    /// Create a new exporter with custom options
    pub fn with_options(options: ExportOptions) -> Self {
        Self { options }
    }

    /// Set export format
    pub fn set_format(&mut self, format: SmfFormat) {
        self.options.format = format;
    }

    /// Set tempo in BPM
    pub fn set_tempo(&mut self, bpm: f64) {
        self.options.tempo_bpm = bpm;
    }

    /// Set time signature
    pub fn set_time_signature(&mut self, time_sig: TimeSignature) {
        self.options.time_signature = time_sig;
    }

    /// Set key signature
    pub fn set_key_signature(&mut self, key_sig: Option<KeySignature>) {
        self.options.key_signature = key_sig;
    }

    /// Set track metadata
    pub fn set_track_metadata(&mut self, track: u8, metadata: TrackMetadata) {
        self.options.track_metadata.insert(track, metadata);
    }

    /// Get current options
    pub fn options(&self) -> &ExportOptions {
        &self.options
    }

    /// Convert BPM to microseconds per quarter note
    fn bpm_to_tempo(bpm: f64) -> u32 {
        if bpm <= 0.0 {
            500_000 // Default to 120 BPM
        } else {
            (60_000_000.0 / bpm).round() as u32
        }
    }

    /// Convert denominator to power of 2
    fn denominator_to_power(denom: u8) -> u8 {
        match denom {
            1 => 0,
            2 => 1,
            4 => 2,
            8 => 3,
            16 => 4,
            32 => 5,
            64 => 6,
            _ => 2, // Default to quarter note
        }
    }

    /// Convert a string to a leaked static byte slice for midly
    fn leak_str(s: &str) -> &'static [u8] {
        Box::leak(s.to_string().into_bytes().into_boxed_slice())
    }

    /// Build meta events for the conductor track
    fn build_conductor_events(&self) -> Vec<TrackEvent<'static>> {
        let mut events = Vec::new();
        let mut delta = 0u32;

        // Sequence name
        if let Some(ref name) = self.options.sequence_name {
            events.push(TrackEvent {
                delta: delta.into(),
                kind: TrackEventKind::Meta(MetaMessage::TrackName(Self::leak_str(name))),
            });
            delta = 0;
        }

        // Copyright
        if let Some(ref copyright) = self.options.copyright {
            events.push(TrackEvent {
                delta: delta.into(),
                kind: TrackEventKind::Meta(MetaMessage::Copyright(Self::leak_str(copyright))),
            });
            delta = 0;
        }

        // Tempo
        let tempo_us = Self::bpm_to_tempo(self.options.tempo_bpm);
        events.push(TrackEvent {
            delta: delta.into(),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(tempo_us.into())),
        });
        delta = 0;

        // Time signature
        let ts = &self.options.time_signature;
        events.push(TrackEvent {
            delta: delta.into(),
            kind: TrackEventKind::Meta(MetaMessage::TimeSignature(
                ts.numerator,
                Self::denominator_to_power(ts.denominator),
                ts.clocks_per_click,
                ts.thirty_seconds_per_quarter,
            )),
        });

        // Key signature
        if let Some(ref ks) = self.options.key_signature {
            events.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::KeySignature(
                    ks.key,
                    ks.mode == KeyMode::Minor,
                )),
            });
        }

        events
    }

    /// Build track header events (name, instrument, text)
    fn build_track_header(&self, track_idx: u8) -> Vec<TrackEvent<'static>> {
        let mut events = Vec::new();

        if let Some(metadata) = self.options.track_metadata.get(&track_idx) {
            // Track name
            if self.options.include_track_names {
                if let Some(ref name) = metadata.name {
                    events.push(TrackEvent {
                        delta: 0.into(),
                        kind: TrackEventKind::Meta(MetaMessage::TrackName(Self::leak_str(name))),
                    });
                }
            }

            // Instrument name
            if let Some(ref instrument) = metadata.instrument {
                events.push(TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Meta(MetaMessage::InstrumentName(Self::leak_str(instrument))),
                });
            }

            // Text events
            if self.options.include_text_events {
                for text in &metadata.text {
                    events.push(TrackEvent {
                        delta: 0.into(),
                        kind: TrackEventKind::Meta(MetaMessage::Text(Self::leak_str(text))),
                    });
                }
            }
        }

        events
    }

    /// Convert a MidiEvent to a midly TrackEvent
    fn convert_event(&self, event: &MidiEvent, channel: u8) -> Option<TrackEventKind<'static>> {
        let ch = midly::num::u4::new(channel & 0x0F);

        match event.event_type {
            MidiEventType::NoteOn => {
                let note = event.note?;
                let vel = event.velocity.unwrap_or(64);
                Some(TrackEventKind::Midi {
                    channel: ch,
                    message: MidiMessage::NoteOn {
                        key: midly::num::u7::new(note & 0x7F),
                        vel: midly::num::u7::new(vel & 0x7F),
                    },
                })
            }
            MidiEventType::NoteOff => {
                let note = event.note?;
                let vel = event.velocity.unwrap_or(64);
                Some(TrackEventKind::Midi {
                    channel: ch,
                    message: MidiMessage::NoteOff {
                        key: midly::num::u7::new(note & 0x7F),
                        vel: midly::num::u7::new(vel & 0x7F),
                    },
                })
            }
            MidiEventType::ControlChange => {
                let controller = event.controller?;
                let value = event.value.unwrap_or(0);
                Some(TrackEventKind::Midi {
                    channel: ch,
                    message: MidiMessage::Controller {
                        controller: midly::num::u7::new(controller & 0x7F),
                        value: midly::num::u7::new(value & 0x7F),
                    },
                })
            }
            MidiEventType::ProgramChange => {
                let program = event.program?;
                Some(TrackEventKind::Midi {
                    channel: ch,
                    message: MidiMessage::ProgramChange {
                        program: midly::num::u7::new(program & 0x7F),
                    },
                })
            }
            MidiEventType::PitchBend => {
                // Value is stored as 14-bit value (0-16383, center at 8192)
                let bend_value = event.value.map(|v| (v as u16) << 7).unwrap_or(8192);
                Some(TrackEventKind::Midi {
                    channel: ch,
                    message: MidiMessage::PitchBend {
                        bend: midly::PitchBend(midly::num::u14::new(bend_value)),
                    },
                })
            }
            MidiEventType::Aftertouch => {
                let pressure = event.value.unwrap_or(0);
                Some(TrackEventKind::Midi {
                    channel: ch,
                    message: MidiMessage::ChannelAftertouch {
                        vel: midly::num::u7::new(pressure & 0x7F),
                    },
                })
            }
        }
    }

    /// Export events to SMF Type 0 (single track)
    fn export_type0(&self, events: &[MidiEvent]) -> Result<Smf<'static>, ExportError> {
        let mut track_events = self.build_conductor_events();

        // Add track header for track 0
        track_events.extend(self.build_track_header(0));

        // Sort events by tick
        let mut sorted_events: Vec<_> = events.iter().collect();
        sorted_events.sort_by_key(|e| e.tick);

        // Convert events with delta times
        let mut last_tick: u64 = 0;
        for event in sorted_events {
            let delta = (event.tick.saturating_sub(last_tick)) as u32;
            if let Some(kind) = self.convert_event(event, event.channel) {
                track_events.push(TrackEvent {
                    delta: delta.into(),
                    kind,
                });
                last_tick = event.tick;
            }
        }

        // End of track
        track_events.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        let header = Header::new(
            Format::SingleTrack,
            midly::Timing::Metrical(midly::num::u15::new(self.options.ticks_per_quarter)),
        );

        Ok(Smf {
            header,
            tracks: vec![track_events],
        })
    }

    /// Export events to SMF Type 1 (multi-track)
    fn export_type1(&self, events: &[MidiEvent]) -> Result<Smf<'static>, ExportError> {
        // Group events by channel
        let mut channel_events: HashMap<u8, Vec<&MidiEvent>> = HashMap::new();
        for event in events {
            channel_events.entry(event.channel).or_default().push(event);
        }

        let mut tracks = Vec::new();

        // Track 0: Conductor track with tempo and time signature
        let mut conductor_track = self.build_conductor_events();
        conductor_track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });
        tracks.push(conductor_track);

        // Create a track for each channel
        let mut channels: Vec<_> = channel_events.keys().copied().collect();
        channels.sort();

        for (track_idx, channel) in channels.iter().enumerate() {
            let mut track_events = self.build_track_header(track_idx as u8 + 1);

            // Sort events by tick
            let mut events = channel_events.get(channel).unwrap().clone();
            events.sort_by_key(|e| e.tick);

            // Convert events with delta times
            let mut last_tick: u64 = 0;
            for event in events {
                let delta = (event.tick.saturating_sub(last_tick)) as u32;
                if let Some(kind) = self.convert_event(event, *channel) {
                    track_events.push(TrackEvent {
                        delta: delta.into(),
                        kind,
                    });
                    last_tick = event.tick;
                }
            }

            // End of track
            track_events.push(TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
            });

            tracks.push(track_events);
        }

        let header = Header::new(
            Format::Parallel,
            midly::Timing::Metrical(midly::num::u15::new(self.options.ticks_per_quarter)),
        );

        Ok(Smf {
            header,
            tracks,
        })
    }

    /// Export MIDI events to a file
    pub fn export_to_file(
        &self,
        events: &[MidiEvent],
        path: &Path,
    ) -> Result<ExportResult, ExportError> {
        info!("Exporting {} events to {:?}", events.len(), path);

        // Build SMF based on format
        let smf = match self.options.format {
            SmfFormat::Type0 => self.export_type0(events)?,
            SmfFormat::Type1 => self.export_type1(events)?,
        };

        // Serialize to bytes
        let mut buffer = Vec::new();
        smf.write(&mut buffer).map_err(|e| ExportError {
            message: format!("Failed to serialize MIDI: {}", e),
            kind: ExportErrorKind::EncodingError,
        })?;

        // Write to file
        let mut file = File::create(path).map_err(|e| ExportError {
            message: format!("Failed to create file: {}", e),
            kind: ExportErrorKind::IoError,
        })?;

        file.write_all(&buffer).map_err(|e| ExportError {
            message: format!("Failed to write file: {}", e),
            kind: ExportErrorKind::IoError,
        })?;

        let num_tracks = smf.tracks.len() as u16;
        let num_events = smf.tracks.iter().map(|t| t.len()).sum();

        debug!(
            "Exported {} tracks, {} events, {} bytes",
            num_tracks,
            num_events,
            buffer.len()
        );

        Ok(ExportResult {
            path: path.to_string_lossy().to_string(),
            bytes_written: buffer.len(),
            num_tracks,
            num_events,
        })
    }

    /// Export MIDI events to bytes (in-memory)
    pub fn export_to_bytes(&self, events: &[MidiEvent]) -> Result<Vec<u8>, ExportError> {
        let smf = match self.options.format {
            SmfFormat::Type0 => self.export_type0(events)?,
            SmfFormat::Type1 => self.export_type1(events)?,
        };

        let mut buffer = Vec::new();
        smf.write(&mut buffer).map_err(|e| ExportError {
            message: format!("Failed to serialize MIDI: {}", e),
            kind: ExportErrorKind::EncodingError,
        })?;

        Ok(buffer)
    }
}

impl Default for MidiExporter {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// GLOBAL STATE FOR TAURI COMMANDS
// =============================================================================

use parking_lot::RwLock;
use std::sync::LazyLock;

static EXPORT_OPTIONS: LazyLock<RwLock<ExportOptions>> =
    LazyLock::new(|| RwLock::new(ExportOptions::default()));

// =============================================================================
// TAURI COMMANDS
// =============================================================================

/// Export MIDI events to a file
///
/// # Arguments
/// * `path` - Output file path
/// * `events` - MIDI events to export
/// * `options` - Optional export options (uses global options if not provided)
#[command]
pub async fn export_midi_file(
    path: String,
    events: Vec<MidiEvent>,
    options: Option<ExportOptions>,
) -> Result<ExportResult, String> {
    let opts = options.unwrap_or_else(|| EXPORT_OPTIONS.read().clone());
    let exporter = MidiExporter::with_options(opts);

    let path = Path::new(&path);
    exporter
        .export_to_file(&events, path)
        .map_err(|e| e.to_string())
}

/// Set the global export format
///
/// # Arguments
/// * `format` - SMF format (Type0 or Type1)
#[command]
pub async fn set_export_format(format: SmfFormat) -> Result<(), String> {
    let mut opts = EXPORT_OPTIONS.write();
    opts.format = format;
    info!("Export format set to {:?}", format);
    Ok(())
}

/// Get current export options
#[command]
pub async fn get_export_options() -> Result<ExportOptions, String> {
    Ok(EXPORT_OPTIONS.read().clone())
}

/// Set export tempo
#[command]
pub async fn set_export_tempo(bpm: f64) -> Result<(), String> {
    if bpm <= 0.0 || bpm > 999.0 {
        return Err("Invalid tempo: must be between 0 and 999 BPM".to_string());
    }
    let mut opts = EXPORT_OPTIONS.write();
    opts.tempo_bpm = bpm;
    Ok(())
}

/// Set export time signature
#[command]
pub async fn set_export_time_signature(
    numerator: u8,
    denominator: u8,
) -> Result<(), String> {
    if numerator == 0 || numerator > 32 {
        return Err("Invalid numerator: must be between 1 and 32".to_string());
    }
    if !matches!(denominator, 1 | 2 | 4 | 8 | 16 | 32 | 64) {
        return Err("Invalid denominator: must be a power of 2 (1, 2, 4, 8, 16, 32, 64)".to_string());
    }

    let mut opts = EXPORT_OPTIONS.write();
    opts.time_signature.numerator = numerator;
    opts.time_signature.denominator = denominator;
    Ok(())
}

/// Set export key signature
#[command]
pub async fn set_export_key_signature(
    key: i8,
    minor: bool,
) -> Result<(), String> {
    if !(-7..=7).contains(&key) {
        return Err("Invalid key: must be between -7 and 7".to_string());
    }

    let mut opts = EXPORT_OPTIONS.write();
    opts.key_signature = Some(KeySignature {
        key,
        mode: if minor { KeyMode::Minor } else { KeyMode::Major },
    });
    Ok(())
}

/// Set track metadata for export
#[command]
pub async fn set_export_track_metadata(
    track: u8,
    name: Option<String>,
    instrument: Option<String>,
) -> Result<(), String> {
    let mut opts = EXPORT_OPTIONS.write();
    let metadata = opts.track_metadata.entry(track).or_default();
    metadata.name = name;
    metadata.instrument = instrument;
    Ok(())
}

/// Reset export options to defaults
#[command]
pub async fn reset_export_options() -> Result<(), String> {
    let mut opts = EXPORT_OPTIONS.write();
    *opts = ExportOptions::default();
    info!("Export options reset to defaults");
    Ok(())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_events() -> Vec<MidiEvent> {
        vec![
            MidiEvent {
                event_type: MidiEventType::NoteOn,
                tick: 0,
                channel: 0,
                note: Some(60),
                velocity: Some(100),
                controller: None,
                value: None,
                program: None,
            },
            MidiEvent {
                event_type: MidiEventType::NoteOff,
                tick: 480,
                channel: 0,
                note: Some(60),
                velocity: Some(64),
                controller: None,
                value: None,
                program: None,
            },
            MidiEvent {
                event_type: MidiEventType::NoteOn,
                tick: 480,
                channel: 1,
                note: Some(64),
                velocity: Some(80),
                controller: None,
                value: None,
                program: None,
            },
            MidiEvent {
                event_type: MidiEventType::NoteOff,
                tick: 960,
                channel: 1,
                note: Some(64),
                velocity: Some(64),
                controller: None,
                value: None,
                program: None,
            },
        ]
    }

    #[test]
    fn test_default_options() {
        let opts = ExportOptions::default();
        assert_eq!(opts.format, SmfFormat::Type0);
        assert_eq!(opts.ticks_per_quarter, 480);
        assert_eq!(opts.tempo_bpm, 120.0);
        assert_eq!(opts.time_signature.numerator, 4);
        assert_eq!(opts.time_signature.denominator, 4);
    }

    #[test]
    fn test_bpm_to_tempo() {
        assert_eq!(MidiExporter::bpm_to_tempo(120.0), 500_000);
        assert_eq!(MidiExporter::bpm_to_tempo(60.0), 1_000_000);
        assert_eq!(MidiExporter::bpm_to_tempo(240.0), 250_000);
    }

    #[test]
    fn test_denominator_to_power() {
        assert_eq!(MidiExporter::denominator_to_power(1), 0);
        assert_eq!(MidiExporter::denominator_to_power(2), 1);
        assert_eq!(MidiExporter::denominator_to_power(4), 2);
        assert_eq!(MidiExporter::denominator_to_power(8), 3);
        assert_eq!(MidiExporter::denominator_to_power(16), 4);
    }

    #[test]
    fn test_export_type0() {
        let exporter = MidiExporter::new();
        let events = create_test_events();
        let result = exporter.export_to_bytes(&events);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        // Check MIDI header magic bytes
        assert_eq!(&bytes[0..4], b"MThd");
    }

    #[test]
    fn test_export_type1() {
        let mut exporter = MidiExporter::new();
        exporter.set_format(SmfFormat::Type1);

        let events = create_test_events();
        let result = exporter.export_to_bytes(&events);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert_eq!(&bytes[0..4], b"MThd");
    }

    #[test]
    fn test_export_with_metadata() {
        let mut options = ExportOptions::default();
        options.sequence_name = Some("Test Sequence".to_string());
        options.copyright = Some("Copyright 2025".to_string());
        options.key_signature = Some(KeySignature {
            key: -2, // Bb
            mode: KeyMode::Major,
        });

        let exporter = MidiExporter::with_options(options);
        let events = create_test_events();
        let result = exporter.export_to_bytes(&events);
        assert!(result.is_ok());
    }

    #[test]
    fn test_track_metadata() {
        let mut exporter = MidiExporter::new();
        exporter.set_format(SmfFormat::Type1);
        exporter.set_track_metadata(
            1,
            TrackMetadata {
                name: Some("Piano".to_string()),
                instrument: Some("Acoustic Grand Piano".to_string()),
                copyright: None,
                text: vec!["Main melody".to_string()],
                channel: 0,
            },
        );

        let events = create_test_events();
        let result = exporter.export_to_bytes(&events);
        assert!(result.is_ok());
    }
}
