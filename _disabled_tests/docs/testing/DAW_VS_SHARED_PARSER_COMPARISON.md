# DAW Parser vs Shared Parser - Detailed Comparison

## Executive Summary

The **DAW parser** is a simplified, playback-focused implementation for real-time sequencing.
The **Shared parser** is a comprehensive, archival-focused implementation for analysis and storage.

**They serve different purposes** and are not interchangeable, but both are correct for their use cases.

---

## 1. ARCHITECTURAL PURPOSE

### DAW Parser (daw/src-tauri/src/core/midi/parser.rs)
- **Purpose:** Real-time MIDI sequencing and playback
- **Use Case:** Load MIDI file → play it back in DAW
- **Optimization:** Speed, simplicity, direct usability
- **What it Discards:** Metadata, tempos, key signatures, SysEx
- **What it Produces:** Single merged event stream ready for sequencer

### Shared Parser (shared/rust/src/core/midi/parser.rs)
- **Purpose:** MIDI archival, analysis, and metadata extraction
- **Use Case:** Import → analyze → store in database
- **Optimization:** Completeness, accuracy, preserving all data
- **What it Preserves:** All metadata, all event types, structure
- **What it Produces:** Complete MidiFile with separate tracks

---

## 2. CODE ORGANIZATION

### DAW Parser
```
parse_midi()               [PUBLIC - main entry]
  ├─ parse_header()        [private]
  ├─ parse_track()         [private]
  └─ MidiReader helper     [private]

330 lines total
4 error variants
8 MidiReader methods
```

### Shared Parser
```
parse_midi_file()          [PUBLIC - main entry]
  ├─ parse_header()        [private]
  ├─ parse_track()         [private]
  ├─ parse_track_events()  [private]
  ├─ parse_event()         [private]
  ├─ parse_meta_or_sysex() [private]
  └─ read_var_len()        [private function - no struct]

~1,370 lines total (code + tests)
7+ error variants
Separate read_var_len() function
Test suite with 100+ test cases
```

---

## 3. DATA FLOW & TRANSFORMATION

### DAW Parser Flow
```
Raw bytes
    ↓
[parse_midi()]
    ├─ Read header (14 bytes)
    │   └─ Extract: format, num_tracks, TPPQN
    │
    ├─ For each track:
    │   └─ [parse_track()] reads events
    │       ├─ Accumulate absolute ticks (sum delta times)
    │       ├─ Route by status byte
    │       ├─ Extract minimal fields (note, velocity, etc)
    │       └─ Skip meta/SysEx entirely
    │
    ├─ Merge all events into single vec
    ├─ Sort by tick (O(n log n))
    └─ Calculate total_ticks
    ↓
MidiPattern {
    events: Vec<MidiEvent>,      // All merged, sorted, absolute ticks
    ticks_per_quarter_note: u16,
    total_ticks: u64,
}
```

### Shared Parser Flow
```
Raw bytes
    ↓
[parse_midi_file()]
    ├─ Read header (14 bytes)
    │   └─ Extract & validate: format, num_tracks, TPPQN
    │
    ├─ For each track:
    │   └─ [parse_track()] → returns Track, bytes_consumed
    │       └─ [parse_track_events()]
    │           ├─ Position-based parsing (not accumulated)
    │           ├─ [parse_event()] for each event
    │           │   ├─ Channel events (NoteOn, NoteOff, CC, PC, etc.)
    │           │   └─ Running status handling
    │           │
    │           └─ [parse_meta_or_sysex()]
    │               ├─ Meta: Tempo, TimeSignature, KeySignature, Text
    │               └─ SysEx: Full data preservation
    │
    └─ Keep tracks separate
    ↓
MidiFile {
    header: Header,
    tracks: Vec<Track>,    // Tracks kept separate
}

Where Track {
    events: Vec<TimedEvent> {
        delta_ticks: u32,  // RELATIVE (not accumulated)
        event: Event,      // Full enum with all variants
    }
}
```

---

## 4. EVENT TYPE HANDLING

### DAW Parser

**Parsed (6 types):**
```
MidiEventType enum {
    NoteOn      → stores note, velocity
    NoteOff     → stores note (velocity always 0)
    ControlChange → stores controller, value
    ProgramChange → stores program
    PitchBend   → stores raw LSB/MSB (not decoded!)
    Aftertouch  → stores raw value
}
```

**Skipped:**
- Meta events (0xFF) - all of them
- SysEx (0xF0, 0xF7) - all of them

**Storage:**
```rust
struct MidiEvent {
    event_type: MidiEventType,
    tick: u64,                // ABSOLUTE
    channel: u8,
    note: Option<u8>,
    velocity: Option<u8>,
    controller: Option<u8>,
    value: Option<u8>,
    program: Option<u8>,
}
```

### Shared Parser

**Parsed (14+ types):**
```
enum Event {
    // Channel events (7)
    NoteOn { channel, note, velocity }
    NoteOff { channel, note, velocity }
    Aftertouch { channel, note, pressure }
    ControlChange { channel, controller, value }
    ProgramChange { channel, program }
    ChannelAftertouch { channel, pressure }
    PitchBend { channel, value }        // DECODED as signed i16!
    
    // Meta events (5)
    TempoChange { microseconds_per_quarter }
    TimeSignature { numerator, denominator, clocks_per_click, thirty_seconds_per_quarter }
    KeySignature { sharps_flats: i8, is_minor: bool }
    Text { text_type: TextType, text: String }
    EndOfTrack
    
    // System events (2)
    SysEx { data: Vec<u8> }
    Unknown { status: u8, data: Vec<u8> }
}

enum TextType {
    Text, Copyright, TrackName, InstrumentName, Lyric, Marker, CuePoint
}
```

**Storage:**
```rust
struct TimedEvent {
    delta_ticks: u32,      // RELATIVE (not accumulated)
    event: Event,
}
```

---

## 5. TIMING REPRESENTATION

### DAW Parser: Absolute Ticks

**Advantage:** Direct position in sequencer
**Disadvantage:** Must recalculate if file structure needed

```
File:  MThd...MTrk... 0 NoteOn, 100 NoteOn, 50 NoteOff...

Result:
Event 1: tick = 0 (absolute)
Event 2: tick = 100 (absolute)
Event 3: tick = 150 (absolute)
```

### Shared Parser: Relative Delta Ticks

**Advantage:** Standard MIDI representation, preserves original file
**Disadvantage:** Must accumulate to get absolute positions

```
File:  MThd...MTrk... 0 NoteOn, 100 NoteOn, 50 NoteOff...

Result:
Event 1: delta_ticks = 0
Event 2: delta_ticks = 100
Event 3: delta_ticks = 50
(Actual position = 0, 100, 150 when accumulated)
```

---

## 6. ERROR HANDLING

### DAW Parser (4 variants)

```rust
pub enum ParseError {
    #[error("Invalid MIDI file format: {0}")]
    InvalidFormat(String),          // Bad magic, wrong header length
    
    #[error("Unsupported MIDI format: {0}")]
    UnsupportedFormat(String),      // Format > 2
    
    #[error("Incomplete data at position {0}")]
    IncompleteData(usize),          // Buffer too short
    
    #[error("Invalid track data: {0}")]
    InvalidTrack(String),           // Track parsing error
}
```

**Strategy:** Simple, generic, combines related errors

### Shared Parser (7+ variants)

```rust
pub enum MidiParseError {
    #[error("Invalid MIDI header: {0}")]
    InvalidHeader(String),
    
    #[error("Invalid track data at byte {position}: {reason}")]
    InvalidTrack { position: usize, reason: String },
    
    #[error("Unsupported MIDI format: {0}")]
    UnsupportedFormat(u16),
    
    #[error("Invalid event at byte {position}: {reason}")]
    InvalidEvent { position: usize, reason: String },
    
    #[error("Incomplete data: expected {expected} bytes, got {actual}")]
    IncompleteData { expected: usize, actual: usize },
    
    #[error("Invalid variable-length quantity at byte {0}")]
    InvalidVarLen(usize),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
```

**Strategy:** Detailed, specific, includes position info and external errors

---

## 7. VLQ (VARIABLE LENGTH QUANTITY) PARSING

### DAW Parser Implementation

```rust
fn read_variable_length(&mut self) -> Result<u64, ParseError> {
    let mut value: u64;
    let mut byte = self.read_u8()?;

    value = (byte & 0x7F) as u64;

    while byte & 0x80 != 0 {
        byte = self.read_u8()?;
        value = (value << 7) | ((byte & 0x7F) as u64);
    }

    Ok(value)
}
```

**Characteristics:**
- Returns u64 (supports very large values)
- No explicit max length check
- Could read up to 8 bytes (MIDI spec: max 4)
- Simple, inline implementation
- **Potential issue:** Malformed files could read more bytes than MIDI spec allows

### Shared Parser Implementation

```rust
fn read_var_len(data: &[u8]) -> Option<(u32, usize)> {
    let mut value = 0u32;
    let mut bytes_read = 0;

    for (i, &byte) in data.iter().enumerate() {
        if i >= 4 {
            return None;  // Max 4 bytes enforcement
        }

        value = (value << 7) | (byte & 0x7F) as u32;
        bytes_read += 1;

        if byte & 0x80 == 0 {
            return Some((value, bytes_read));
        }
    }

    None  // Ran out of data
}
```

**Characteristics:**
- Returns Option<(u32, usize)> (clearer about bytes consumed)
- Explicit max 4 bytes check
- Separate function, not a method
- Returns bytes_consumed explicitly
- MIDI spec compliant
- Used in shared parser's comprehensive test suite

---

## 8. RUNNING STATUS HANDLING

### Both Parsers Implement Correctly

Both handle running status similarly, but with different error types:

```
Logic (same in both):
1. Peek next byte
2. If < 0x80 (MSB clear): it's a data byte, use previous status
3. If >= 0x80 (MSB set): it's a new status, read and save it
4. Meta (0xFF) and SysEx (0xF0, 0xF7): don't save as running status
5. Error if data byte without prior running status
```

**DAW Error:** `InvalidTrack("No running status available")`
**Shared Error:** `InvalidEvent { position, reason: "Data byte without running status" }`

---

## 9. TEST COVERAGE

### DAW Parser
- **Tests:** None in current code (but 330-line module)
- **Coverage:** 0%
- **Recommendation:** Need comprehensive test suite (80%+ target)

### Shared Parser
- **Tests:** 100+ tests in file
- **Coverage:** ~91.97% (126/137 lines)
- **Status:** Production-ready (Phase 1.3 complete)

---

## 10. COMPLEXITY ANALYSIS

### DAW Parser
```
parse_midi():        O(n log n)    [n = # events, dominated by sort]
parse_header():      O(1)          [always 14 bytes]
parse_track():       O(m)          [m = events in track]
read_u8():           O(1)
read_u16():          O(1)
read_u32():          O(1)
read_bytes():        O(k)          [k = bytes to read]
read_variable_length(): O(1) avg, O(8) worst
skip():              O(1)
```

### Shared Parser
Same complexity characteristics, but with more event parsing variations.

---

## 11. CODE QUALITY

### DAW Parser
- **Unwraps:** 0
- **Panics:** 0
- **Expects:** 0
- **Quality:** Excellent - no dangerous patterns
- **Optimization:** Good - minimal allocations

### Shared Parser
- **Unwraps:** 0
- **Panics:** 0
- **Expects:** 0
- **Quality:** Excellent - production-ready
- **Optimization:** Good - with proper error contexts

---

## 12. WHEN TO USE EACH

### Use DAW Parser When:
- Loading MIDI file for playback only
- You don't need metadata (tempos, key signatures)
- You want simplicity and speed
- Events need to be in a single sorted stream
- Tracks don't need to be kept separate

### Use Shared Parser When:
- Importing MIDI for database storage
- You need complete metadata extraction
- You need to analyze the file
- Track structure needs to be preserved
- Building analysis features (BPM, key detection)
- Serializing to JSON/database

### Should NOT Reuse:
- These parsers are not interchangeable
- Different output types (MidiPattern vs MidiFile)
- Different error handling strategies
- Different use cases (playback vs archival)

---

## 13. POTENTIAL IMPROVEMENTS

### DAW Parser
1. Add VLQ max length check (4 bytes per MIDI spec)
2. Decode pitch bend value to signed integer
3. Add comprehensive test suite (80%+ coverage)
4. Consider optional meta event parsing for advanced features

### Shared Parser
1. Already excellent - minimal improvements needed
2. Could add more exotic meta events
3. Could add SMTPE support (if needed)

---

## 14. SUMMARY TABLE

| Aspect | DAW | Shared |
|--------|-----|--------|
| **Lines** | 330 | ~1,370 |
| **Tests** | 0 | 100+ |
| **Coverage** | 0% | 91.97% |
| **Error variants** | 4 | 7+ |
| **Event types** | 6 | 14+ |
| **VLQ max check** | ❌ | ✅ |
| **Meta events** | ❌ Skip | ✅ Parse |
| **SysEx events** | ❌ Skip | ✅ Parse |
| **Track structure** | 🔄 Merged | 📋 Separate |
| **Timing** | Absolute | Relative |
| **Pitch bend decode** | ❌ Raw | ✅ Signed i16 |
| **Status** | Basic | Production-ready |
| **Use case** | Playback | Archival/Analysis |

---

## 15. CODE SNIPPETS FOR REFERENCE

### Key Difference: Event Struct

**DAW** - Flat with Options:
```rust
pub struct MidiEvent {
    pub event_type: MidiEventType,
    pub tick: u64,
    pub channel: u8,
    pub note: Option<u8>,
    pub velocity: Option<u8>,
    pub controller: Option<u8>,
    pub value: Option<u8>,
    pub program: Option<u8>,
}
```

**Shared** - Rich enum:
```rust
pub enum Event {
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8, velocity: u8 },
    TempoChange { microseconds_per_quarter: u32 },
    TimeSignature { numerator: u8, denominator: u8, ... },
    KeySignature { sharps_flats: i8, is_minor: bool },
    // ... many more
}
```

---

## Conclusion

Both parsers are **correct and well-implemented** for their respective use cases. They represent a deliberate architectural choice to optimize for different scenarios:

- **DAW:** Optimized for speed and simplicity in playback
- **Shared:** Optimized for completeness and analysis

This is a good architectural decision - there's no need to unify them.

