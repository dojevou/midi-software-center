# Phase 1.1 Instructions: Test Shared MIDI Types

**Goal:** Add comprehensive tests to shared library MIDI core modules
**Time Estimate:** 4 hours
**Coverage Target:** 95%+
**Priority:** 🔴 CRITICAL

---

## Files to Test (3 files)

1. ✅ `shared/rust/src/core/midi/parser.rs` - Already has 6 tests, expand to 15+
2. 🔴 `shared/rust/src/core/midi/types.rs` - **START HERE** - No tests (CRITICAL)
3. 🔴 `shared/rust/src/core/midi/error.rs` - No tests (HIGH)

---

## Part 1: Test `types.rs` (90 minutes)

### File Location
```
shared/rust/src/core/midi/types.rs
```

### Current State
- **Lines:** ~200 lines (estimate)
- **Public Types:** MidiFile, Track, TimedEvent, Event, Header, etc.
- **Current Tests:** 0
- **Dependencies:** midly crate

### What to Test

#### 1. MidiFile Structure (15 tests)

**Basic Construction:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_file_creation() {
        let header = Header {
            format: 1,
            num_tracks: 2,
            ticks_per_quarter_note: 480,
        };

        let file = MidiFile {
            header,
            tracks: vec![],
        };

        assert_eq!(file.header.format, 1);
        assert_eq!(file.header.num_tracks, 2);
        assert_eq!(file.header.ticks_per_quarter_note, 480);
        assert_eq!(file.tracks.len(), 0);
    }

    #[test]
    fn test_midi_file_with_tracks() {
        let header = Header {
            format: 1,
            num_tracks: 1,
            ticks_per_quarter_note: 480,
        };

        let track = Track {
            events: vec![],
        };

        let file = MidiFile {
            header,
            tracks: vec![track],
        };

        assert_eq!(file.tracks.len(), 1);
        assert_eq!(file.tracks[0].events.len(), 0);
    }

    #[test]
    fn test_format_0_single_track() {
        // Format 0 = single track
        let header = Header {
            format: 0,
            num_tracks: 1,
            ticks_per_quarter_note: 480,
        };

        let file = MidiFile {
            header,
            tracks: vec![Track { events: vec![] }],
        };

        assert_eq!(file.header.format, 0);
        assert_eq!(file.tracks.len(), 1);
    }

    #[test]
    fn test_format_1_multi_track() {
        // Format 1 = multiple tracks, synchronous
        let header = Header {
            format: 1,
            num_tracks: 3,
            ticks_per_quarter_note: 480,
        };

        let file = MidiFile {
            header,
            tracks: vec![
                Track { events: vec![] },
                Track { events: vec![] },
                Track { events: vec![] },
            ],
        };

        assert_eq!(file.header.format, 1);
        assert_eq!(file.tracks.len(), 3);
    }

    #[test]
    fn test_ticks_per_quarter_note_variations() {
        // Test common TPQN values
        for tpqn in [96, 120, 240, 480, 960, 1920] {
            let header = Header {
                format: 1,
                num_tracks: 1,
                ticks_per_quarter_note: tpqn,
            };

            assert_eq!(header.ticks_per_quarter_note, tpqn);
        }
    }
}
```

#### 2. Event Types (20 tests)

**NoteOn/NoteOff:**
```rust
#[test]
fn test_note_on_event() {
    let event = Event::NoteOn {
        channel: 0,
        note: 60, // Middle C
        velocity: 64,
    };

    match event {
        Event::NoteOn { channel, note, velocity } => {
            assert_eq!(channel, 0);
            assert_eq!(note, 60);
            assert_eq!(velocity, 64);
        }
        _ => panic!("Expected NoteOn event"),
    }
}

#[test]
fn test_note_off_event() {
    let event = Event::NoteOff {
        channel: 0,
        note: 60,
        velocity: 0,
    };

    match event {
        Event::NoteOff { channel, note, velocity } => {
            assert_eq!(channel, 0);
            assert_eq!(note, 60);
            assert_eq!(velocity, 0);
        }
        _ => panic!("Expected NoteOff event"),
    }
}

#[test]
fn test_note_velocity_range() {
    // MIDI velocity is 0-127
    for velocity in [0, 64, 127] {
        let event = Event::NoteOn {
            channel: 0,
            note: 60,
            velocity,
        };

        match event {
            Event::NoteOn { velocity: v, .. } => assert_eq!(v, velocity),
            _ => panic!(),
        }
    }
}

#[test]
fn test_note_pitch_range() {
    // MIDI notes are 0-127
    for note in [0, 21, 60, 108, 127] {
        let event = Event::NoteOn {
            channel: 0,
            note,
            velocity: 64,
        };

        match event {
            Event::NoteOn { note: n, .. } => assert_eq!(n, note),
            _ => panic!(),
        }
    }
}

#[test]
fn test_channel_range() {
    // MIDI channels are 0-15 (1-16 in user-facing)
    for channel in 0..16 {
        let event = Event::NoteOn {
            channel,
            note: 60,
            velocity: 64,
        };

        match event {
            Event::NoteOn { channel: ch, .. } => assert_eq!(ch, channel),
            _ => panic!(),
        }
    }
}
```

**Control Change:**
```rust
#[test]
fn test_control_change_event() {
    let event = Event::ControlChange {
        channel: 0,
        controller: 7, // Volume
        value: 100,
    };

    match event {
        Event::ControlChange { channel, controller, value } => {
            assert_eq!(channel, 0);
            assert_eq!(controller, 7);
            assert_eq!(value, 100);
        }
        _ => panic!("Expected ControlChange"),
    }
}

#[test]
fn test_common_cc_controllers() {
    // Test common MIDI CC numbers
    let controllers = [
        (1, "Modulation"),
        (7, "Volume"),
        (10, "Pan"),
        (11, "Expression"),
        (64, "Sustain"),
    ];

    for (cc_num, _name) in controllers {
        let event = Event::ControlChange {
            channel: 0,
            controller: cc_num,
            value: 64,
        };

        match event {
            Event::ControlChange { controller, .. } => {
                assert_eq!(controller, cc_num);
            }
            _ => panic!(),
        }
    }
}
```

**Tempo/Time Signature:**
```rust
#[test]
fn test_tempo_event() {
    let event = Event::Tempo {
        microseconds_per_quarter_note: 500_000, // 120 BPM
    };

    match event {
        Event::Tempo { microseconds_per_quarter_note } => {
            assert_eq!(microseconds_per_quarter_note, 500_000);
        }
        _ => panic!(),
    }
}

#[test]
fn test_time_signature_event() {
    let event = Event::TimeSignature {
        numerator: 4,
        denominator: 2, // Represents 4/4 (2^2 = 4)
        midi_clocks: 24,
        thirty_seconds: 8,
    };

    match event {
        Event::TimeSignature { numerator, denominator, .. } => {
            assert_eq!(numerator, 4);
            assert_eq!(denominator, 2);
        }
        _ => panic!(),
    }
}
```

#### 3. TimedEvent Structure (5 tests)

```rust
#[test]
fn test_timed_event_creation() {
    let event = TimedEvent {
        delta_ticks: 480,
        event: Event::NoteOn {
            channel: 0,
            note: 60,
            velocity: 64,
        },
    };

    assert_eq!(event.delta_ticks, 480);
}

#[test]
fn test_timed_event_zero_delta() {
    // Simultaneous events have delta_ticks = 0
    let event = TimedEvent {
        delta_ticks: 0,
        event: Event::NoteOn {
            channel: 0,
            note: 64,
            velocity: 64,
        },
    };

    assert_eq!(event.delta_ticks, 0);
}

#[test]
fn test_timed_event_large_delta() {
    // Test large delta times
    let event = TimedEvent {
        delta_ticks: 100_000,
        event: Event::NoteOff {
            channel: 0,
            note: 60,
            velocity: 0,
        },
    };

    assert_eq!(event.delta_ticks, 100_000);
}
```

#### 4. Track Structure (5 tests)

```rust
#[test]
fn test_empty_track() {
    let track = Track {
        events: vec![],
    };

    assert_eq!(track.events.len(), 0);
}

#[test]
fn test_track_with_events() {
    let events = vec![
        TimedEvent {
            delta_ticks: 0,
            event: Event::NoteOn { channel: 0, note: 60, velocity: 64 },
        },
        TimedEvent {
            delta_ticks: 480,
            event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
        },
    ];

    let track = Track { events };

    assert_eq!(track.events.len(), 2);
}

#[test]
fn test_track_total_ticks() {
    let events = vec![
        TimedEvent {
            delta_ticks: 100,
            event: Event::NoteOn { channel: 0, note: 60, velocity: 64 },
        },
        TimedEvent {
            delta_ticks: 200,
            event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
        },
        TimedEvent {
            delta_ticks: 300,
            event: Event::NoteOn { channel: 0, note: 64, velocity: 64 },
        },
    ];

    let track = Track { events };

    let total_ticks: u32 = track.events.iter()
        .map(|e| e.delta_ticks)
        .sum();

    assert_eq!(total_ticks, 600);
}
```

---

## Part 2: Test `error.rs` (60 minutes)

### File Location
```
shared/rust/src/core/midi/error.rs
```

### What to Test

#### 1. Error Enum Variants (10 tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_creation() {
        let error = MidiError::ParseError("Invalid header".to_string());

        match error {
            MidiError::ParseError(msg) => {
                assert_eq!(msg, "Invalid header");
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_io_error_conversion() {
        use std::io;

        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let midi_error: MidiError = io_error.into();

        match midi_error {
            MidiError::IoError(_) => {}, // Success
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_invalid_format_error() {
        let error = MidiError::InvalidFormat {
            expected: 1,
            found: 2,
        };

        match error {
            MidiError::InvalidFormat { expected, found } => {
                assert_eq!(expected, 1);
                assert_eq!(found, 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_error_display() {
        let error = MidiError::ParseError("Test error".to_string());
        let display_string = format!("{}", error);

        assert!(display_string.contains("Test error"));
    }

    #[test]
    fn test_error_debug() {
        let error = MidiError::ParseError("Debug test".to_string());
        let debug_string = format!("{:?}", error);

        assert!(debug_string.contains("Debug test"));
    }
}
```

#### 2. Error Conversion Tests (5 tests)

```rust
#[test]
fn test_from_midly_error() {
    // If using midly::Error
    // Test conversion from midly errors
}

#[test]
fn test_error_chain() {
    // Test nested errors if applicable
}
```

---

## Part 3: Expand `parser.rs` Tests (90 minutes)

### Current Tests (6)
- ✅ test_parse_header
- ✅ test_parse_invalid_header_magic
- ✅ test_parse_minimal_file
- ✅ test_parse_note_on
- ✅ test_read_var_len
- ✅ test_channels_used
- ✅ test_total_notes

### Additional Tests Needed (9+)

```rust
#[test]
fn test_parse_format_0_file() {
    // Single track MIDI file
    let data = create_test_midi_format_0();
    let result = parse_midi_file(&data);
    assert!(result.is_ok());

    let midi = result.unwrap();
    assert_eq!(midi.header.format, 0);
    assert_eq!(midi.tracks.len(), 1);
}

#[test]
fn test_parse_format_1_file() {
    // Multi-track MIDI file
    let data = create_test_midi_format_1();
    let result = parse_midi_file(&data);
    assert!(result.is_ok());

    let midi = result.unwrap();
    assert_eq!(midi.header.format, 1);
    assert!(midi.tracks.len() > 1);
}

#[test]
fn test_parse_empty_file() {
    let data = vec![];
    let result = parse_midi_file(&data);
    assert!(result.is_err());
}

#[test]
fn test_parse_truncated_file() {
    // File that ends abruptly
    let data = vec![0x4D, 0x54, 0x68, 0x64]; // "MThd" header only
    let result = parse_midi_file(&data);
    assert!(result.is_err());
}

#[test]
fn test_parse_corrupt_track_data() {
    // Malformed track chunk
    let data = create_corrupt_track_data();
    let result = parse_midi_file(&data);
    assert!(result.is_err());
}

#[test]
fn test_parse_multiple_tempo_events() {
    // File with tempo changes
    let data = create_multi_tempo_midi();
    let result = parse_midi_file(&data);
    assert!(result.is_ok());

    let midi = result.unwrap();
    let tempo_count = midi.tracks[0].events.iter()
        .filter(|e| matches!(e.event, Event::Tempo { .. }))
        .count();

    assert!(tempo_count > 1);
}

#[test]
fn test_parse_key_signature_event() {
    // File with key signature
    let data = create_keysig_midi();
    let result = parse_midi_file(&data);
    assert!(result.is_ok());
}

#[test]
fn test_parse_program_change() {
    // Instrument changes
    let data = create_program_change_midi();
    let result = parse_midi_file(&data);
    assert!(result.is_ok());

    let midi = result.unwrap();
    let has_program_change = midi.tracks[0].events.iter()
        .any(|e| matches!(e.event, Event::ProgramChange { .. }));

    assert!(has_program_change);
}

#[test]
fn test_parse_pitch_bend() {
    // Pitch bend events
    let data = create_pitchbend_midi();
    let result = parse_midi_file(&data);
    assert!(result.is_ok());
}
```

---

## Test Helpers

### Create Test Data Functions

```rust
#[cfg(test)]
mod test_helpers {
    use super::*;

    pub fn create_minimal_midi_header() -> Vec<u8> {
        vec![
            // "MThd" header
            0x4D, 0x54, 0x68, 0x64,
            // Header length: 6 bytes
            0x00, 0x00, 0x00, 0x06,
            // Format: 1
            0x00, 0x01,
            // Number of tracks: 1
            0x00, 0x01,
            // Ticks per quarter note: 480
            0x01, 0xE0,
        ]
    }

    pub fn create_empty_track() -> Vec<u8> {
        vec![
            // "MTrk" header
            0x4D, 0x54, 0x72, 0x6B,
            // Track length: 4 (just end-of-track)
            0x00, 0x00, 0x00, 0x04,
            // End of track: delta=0, meta event FF 2F 00
            0x00, 0xFF, 0x2F, 0x00,
        ]
    }

    pub fn create_test_midi_format_0() -> Vec<u8> {
        let mut data = create_minimal_midi_header();
        data.extend(create_empty_track());
        data
    }
}
```

---

## Success Criteria

After completing Phase 1.1, you should have:

- ✅ **40+ new tests** (15 types + 20 events + 5 timed + 5 track + 10 error + 9 parser)
- ✅ **95%+ coverage** of types.rs
- ✅ **90%+ coverage** of error.rs
- ✅ **95%+ coverage** of parser.rs
- ✅ **All tests passing**: `cargo test --lib`
- ✅ **No warnings** from rustc or clippy

---

## Running Tests

```bash
# Run all shared library tests
cd shared/rust
cargo test

# Run specific module tests
cargo test core::midi::types
cargo test core::midi::error
cargo test core::midi::parser

# Run with output
cargo test -- --nocapture

# Check coverage (if tarpaulin works)
cargo tarpaulin --lib --out Stdout
```

---

## Time Breakdown

| Task | Time |
|------|------|
| Test types.rs | 90 min |
| Test error.rs | 60 min |
| Expand parser.rs tests | 90 min |
| **Total** | **240 min (4 hours)** |

---

## Next Phase

After completing Phase 1.1, proceed to:
- **Phase 1.2:** Test analysis modules (auto_tagger, key_detector)
- **Estimated:** 12 hours

---

*Ready to begin? Start with `test_midi_file_creation()` in types.rs*
