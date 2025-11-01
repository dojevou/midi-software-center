# DAW MIDI Parser Analysis (Medium Thoroughness)

**File:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/core/midi/parser.rs`
**Lines of Code:** 330 lines
**Classification:** Trusty Module (pure functions, no I/O)

---

## 1. ARCHITECTURE OVERVIEW

The DAW parser is a **simplified, playback-focused parser** designed for real-time sequencing, not archival/analysis.

### Key Differences from Shared Parser

| Aspect | DAW Parser | Shared Parser |
|--------|-----------|---------------|
| **Purpose** | Real-time playback | Archival & analysis |
| **Meta Events** | Skip most (not stored) | Full parsing & storage |
| **SysEx Events** | Skip | Full parsing & storage |
| **Delta Time Handling** | Accumulates to absolute ticks | Stores as relative delta_ticks |
| **Track Merging** | Merges all tracks into single event stream | Keeps tracks separate |
| **Error Handling** | Simple (4 variants) | Rich (7+ variants) |
| **VLQ Implementation** | Custom read_variable_length() | Separate read_var_len() |
| **Return Type** | MidiPattern (simplified) | MidiFile (full metadata) |

---

## 2. PUBLIC API FUNCTIONS

### Main Entry Point
```rust
pub fn parse_midi(data: &[u8]) -> Result<MidiPattern, ParseError>
```

**Purpose:** Parse complete MIDI file bytes into MidiPattern
**Input:** Raw MIDI file bytes
**Output:** MidiPattern with all events merged and sorted by tick
**Complexity:** O(n) where n = file size
**Error Handling:** 
- Empty file → IncompleteData(0)
- Invalid header → propagates from parse_header
- Invalid track → propagates from parse_track

**Algorithm:**
1. Check data not empty
2. Create MidiReader wrapper
3. Parse header (validates magic, format, track count)
4. Loop through tracks, parse each, accumulate events
5. Sort all events by tick (stable sort - O(n log n))
6. Calculate total_ticks from last event
7. Return MidiPattern

---

## 3. PRIVATE FUNCTIONS (Helper Functions)

### parse_header()
```rust
fn parse_header(reader: &mut MidiReader) -> Result<MidiHeader, ParseError>
```

**Purpose:** Parse MThd chunk (14 bytes total)
**Input:** MidiReader positioned at start
**Output:** MidiHeader with num_tracks and ticks_per_quarter_note
**Complexity:** O(1) - fixed 14 bytes

**Steps:**
1. Read 4 bytes "MThd" magic → error if mismatch
2. Read 4-byte header length → error if != 6
3. Read 2-byte format (0, 1, or 2) → error if > 2
4. Read 2-byte num_tracks
5. Read 2-byte ticks_per_quarter_note

**Edge Cases:**
- Zero tracks (legal but unusual)
- Very small TPPQN values (< 24)
- Large TPPQN values (> 960)

### parse_track()
```rust
fn parse_track(reader: &mut MidiReader) -> Result<Vec<MidiEvent>, ParseError>
```

**Purpose:** Parse MTrk chunk
**Input:** MidiReader positioned at MTrk header
**Output:** Vec of MidiEvents from this track
**Complexity:** O(m) where m = number of events in track

**Steps:**
1. Read 4 bytes "MTrk" → error if mismatch
2. Read 4-byte track length
3. Calculate track_end position
4. Loop until position >= track_end:
   - Read delta time (VLQ) → accumulate to current_tick
   - Peek at next byte to check for running status
   - If MSB clear (< 0x80): use running status
   - If MSB set: read status byte, update running_status
   - Route based on status byte (0xFF, 0xF0, 0xF7, or channel)

**Running Status Handling:**
- Meta events (0xFF) → don't update running status
- SysEx events (0xF0, 0xF7) → don't update running status
- Channel events (0x80-0xEF) → update running status

**Event Types Captured:**
- 0x90: Note On → includes note, velocity
- 0x80: Note Off → includes note, sets velocity=0
- 0xB0: Control Change → includes controller, value
- 0xC0: Program Change → includes program
- 0xE0: Pitch Bend → raw bytes captured but not decoded
- 0xD0: Aftertouch → raw byte captured

**Events SKIPPED (not stored):**
- 0xFF meta events (except not explicitly forbidden - code just skips with skip())
- 0xF0, 0xF7 SysEx events

**Sorting:** Events sorted by tick after all tracks merged (in parse_midi)

---

## 4. DATA STRUCTURES

### MidiHeader (Private)
```rust
#[derive(Debug)]
struct MidiHeader {
    num_tracks: u16,
    ticks_per_quarter_note: u16,
}
```
Simple struct for internal use only.

### MidiReader (Helper Struct)
```rust
struct MidiReader<'a> {
    data: &'a [u8],
    pos: usize,
}
```

**Methods:**
| Method | Signature | Purpose |
|--------|-----------|---------|
| new() | `fn new(data: &'a [u8])` | Create new reader |
| position() | `fn position(&self) -> usize` | Get current position |
| read_u8() | `fn read_u8(&mut self) -> Result<u8>` | Read 1 byte, advance pos |
| peek_u8() | `fn peek_u8(&self) -> Result<u8>` | Read without advancing |
| read_u16() | `fn read_u16(&mut self) -> Result<u16>` | Read 2 bytes big-endian |
| read_u32() | `fn read_u32(&mut self) -> Result<u32>` | Read 4 bytes big-endian |
| read_bytes() | `fn read_bytes(count: usize) -> Result<&'a [u8]>` | Read slice |
| read_variable_length() | `fn read_variable_length(&mut self) -> Result<u64>` | Read VLQ (0-8 bytes) |
| skip() | `fn skip(count: usize) -> Result<()>` | Skip bytes without storing |

**Complexity:**
- All methods: O(1) except read_variable_length() which is O(1) typically but up to O(8) for VLQ
- Bounds checking on every read

### MidiPattern (Public Output)
```rust
pub struct MidiPattern {
    pub events: Vec<MidiEvent>,
    pub ticks_per_quarter_note: u16,
    pub total_ticks: u64,
}
```

### MidiEvent (Public Output)
```rust
pub struct MidiEvent {
    pub event_type: MidiEventType,
    pub tick: u64,              // Absolute tick position (not delta)
    pub channel: u8,
    pub note: Option<u8>,       // For NoteOn/NoteOff
    pub velocity: Option<u8>,   // For NoteOn/NoteOff
    pub controller: Option<u8>, // For ControlChange
    pub value: Option<u8>,      // For ControlChange or Aftertouch
    pub program: Option<u8>,    // For ProgramChange
}
```

---

## 5. ERROR TYPES & VARIANTS

### ParseError Enum (4 variants)
```rust
pub enum ParseError {
    #[error("Invalid MIDI file format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported MIDI format: {0}")]
    UnsupportedFormat(String),

    #[error("Incomplete data at position {0}")]
    IncompleteData(usize),

    #[error("Invalid track data: {0}")]
    InvalidTrack(String),
}
```

**Usage Map:**
| Variant | Triggered By |
|---------|--------------|
| InvalidFormat | Bad MThd/MTrk magic, wrong header length |
| UnsupportedFormat | MIDI format > 2 |
| IncompleteData | End of buffer before expected (all MidiReader bounds errors) |
| InvalidTrack | MTrk parsing errors, running status without prior status |

**Simpler Than Shared Parser:**
- Shared has 7+ variants including Io, Utf8, InvalidVarLen
- DAW has only 4 - combines many cases into generic strings

---

## 6. VLQ (VARIABLE LENGTH QUANTITY) PARSING

### read_variable_length() Implementation
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

**Algorithm:**
1. Read first byte
2. Extract lower 7 bits as initial value
3. While high bit (0x80) is set:
   - Read next byte
   - Shift value left 7 bits
   - OR in lower 7 bits of new byte
4. Return accumulated value

**Characteristics:**
- No explicit max length check (could theoretically read up to 8 bytes for u64)
- Returns u64 (unlimited range)
- Each byte contributes 7 bits
- High bit (continuation flag) is not stored in value

**Test Cases:**
- 1 byte: 0x00-0x7F → values 0-127
- 2 bytes: 0x81 0x00 → value 128
- 2 bytes: 0xFF 0x7F → value 16383
- Larger values accumulate correctly

**Potential Issues:**
- CRITICAL: No maximum length enforcement (could read 8+ bytes)
- Shared parser has `if i >= 4: return None` check
- DAW parser will happily read 8 bytes for u64, but MIDI spec allows up to 4 bytes
- Edge case: Malformed file with infinite VLQ (all bytes have 0x80 set) will eventually fail on IncompleteData

---

## 7. RUNNING STATUS HANDLING

### How It Works:
```rust
let status_byte = reader.peek_u8()?;

let status = if status_byte & 0x80 == 0 {
    // Data byte without status → use running status
    running_status.ok_or(ParseError::InvalidTrack(...))?
} else {
    // Status byte
    let s = reader.read_u8()?;
    if s != 0xFF && s != 0xF0 && s != 0xF7 {
        // Save as running status (but not meta/sysex)
        running_status = Some(s);
    }
    s
};
```

**Rules:**
1. If next byte < 0x80 (MSB clear): it's a data byte, use previous status
2. If next byte >= 0x80 (MSB set): it's a new status byte, read it
3. Running status only for channel events (0x80-0xEF)
4. Meta (0xFF) and SysEx (0xF0, 0xF7) clear running status (don't update it)

**Error Case:**
- Data byte (< 0x80) encountered with no prior running status → InvalidTrack error

---

## 8. COMPLEXITY ANALYSIS

| Function | Time | Space | Notes |
|----------|------|-------|-------|
| parse_midi() | O(n log n) | O(n) | n = # events; sort is O(n log n) |
| parse_header() | O(1) | O(1) | Fixed 14 bytes |
| parse_track() | O(m) | O(m) | m = events in track |
| MidiReader::read_u8() | O(1) | O(1) | Bounds check + index |
| MidiReader::read_u16() | O(1) | O(1) | 2x read_u8 |
| MidiReader::read_u32() | O(1) | O(1) | 4x read_u8 |
| MidiReader::read_variable_length() | O(1) avg, O(k) worst | O(1) | k ≤ 8 for u64 |
| MidiReader::skip() | O(1) | O(1) | Just advances position |

---

## 9. EDGE CASES NEEDING TESTS

### 1. Header Parsing
- [ ] Zero tracks (valid format 0)
- [ ] Very large track count (65535)
- [ ] TPPQN = 0 (invalid but unchecked)
- [ ] TPPQN = 1 (edge case)
- [ ] TPPQN = 32767 (near max)
- [ ] Format 0 with 1 track
- [ ] Format 1 with many tracks
- [ ] Format 2 (rarely used)

### 2. Delta Time / VLQ
- [ ] Zero delta time (simultaneous events)
- [ ] Maximum 1-byte VLQ: 0x7F (127)
- [ ] Minimum 2-byte VLQ: 0x81 0x00 (128)
- [ ] Maximum 2-byte VLQ: 0xFF 0x7F (16383)
- [ ] Large values: 0x81 0x80 0x00 (8192, 3 bytes)
- [ ] Boundary: 0x80 = continuation, not 128
- [ ] Large delta followed by events
- [ ] Consecutive zero-delta events

### 3. Note Events
- [ ] Note on velocity = 0 (sometimes treated as note off)
- [ ] Note on velocity = 127 (maximum)
- [ ] Note = 0 (lowest)
- [ ] Note = 127 (highest)
- [ ] Channel 0-15 (all channels)
- [ ] Note on then note off same note same channel
- [ ] Overlapping notes (poly)

### 4. Running Status
- [ ] Single status byte, multiple data bytes
- [ ] Running status across 10+ events
- [ ] Running status cleared by meta event
- [ ] Running status cleared by sysex
- [ ] Running status error: data without prior status
- [ ] Meta event doesn't update running status
- [ ] SysEx doesn't update running status

### 5. Track Boundaries
- [ ] Track data exactly fills declared length
- [ ] Track data shorter than declared (should error)
- [ ] Multiple tracks concatenated correctly
- [ ] Empty track (just EndOfTrack)
- [ ] Track without EndOfTrack (ends at track_end)

### 6. Event Types
- [ ] Control Change: all 128 controllers
- [ ] Program Change: all 128 programs
- [ ] Pitch Bend: center (0), max up, max down
- [ ] Aftertouch: channel + note variants
- [ ] SysEx: empty, single byte, many bytes
- [ ] Meta events: skipped correctly

### 7. Buffer Boundaries
- [ ] File exactly 14 bytes (empty single track)
- [ ] File one byte short (error)
- [ ] Track header incomplete (error)
- [ ] Event incomplete (error)
- [ ] VLQ incomplete (error)

### 8. Unusual but Valid
- [ ] All events on same tick
- [ ] Extreme tick values (u64 max approaches)
- [ ] Mixed meta + channel events
- [ ] Multiple format changes in file
- [ ] Deeply nested tracks

### 9. Error Cases
- [ ] Wrong MThd magic: "MThx", "MThd" with typo
- [ ] Wrong MTrk magic
- [ ] Format > 2
- [ ] Header length != 6
- [ ] Track length = 0
- [ ] VLQ never terminates (tests if read_u8 returns error)

### 10. Sorting & Merging
- [ ] Events from multiple tracks sorted by tick
- [ ] Stable sort preserves order for same-tick events
- [ ] Total_ticks correctly calculated
- [ ] Total_ticks with empty event list

---

## 10. COMPARATIVE DIFFERENCES: DAW vs SHARED PARSER

### Data Representation

**Shared Parser (Complex):**
```rust
pub struct MidiFile {
    pub header: Header,
    pub tracks: Vec<Track>,  // Tracks kept separate
}

pub struct Track {
    pub events: Vec<TimedEvent>,
}

pub struct TimedEvent {
    pub delta_ticks: u32,    // RELATIVE delta time
    pub event: Event,         // Full Event enum
}

pub enum Event {
    NoteOn { channel, note, velocity },
    NoteOff { ... },
    TempoChange { microseconds_per_quarter },
    TimeSignature { ... },
    KeySignature { ... },
    Text { text_type, text },
    SysEx { data },
    Unknown { status, data },
    // ... and more
}
```

**DAW Parser (Simplified):**
```rust
pub struct MidiPattern {
    pub events: Vec<MidiEvent>,      // All tracks merged
    pub ticks_per_quarter_note: u16,
    pub total_ticks: u64,
}

pub struct MidiEvent {
    pub event_type: MidiEventType,
    pub tick: u64,                   // ABSOLUTE tick
    pub channel: u8,
    pub note: Option<u8>,
    pub velocity: Option<u8>,
    pub controller: Option<u8>,
    pub value: Option<u8>,
    pub program: Option<u8>,
}
```

### Key Differences:

1. **Track Handling:**
   - Shared: Keeps tracks separate (format compatibility)
   - DAW: Merges all tracks into one list (simpler for playback)

2. **Delta Time:**
   - Shared: Stores relative delta_ticks (standard MIDI)
   - DAW: Converts to absolute tick positions (easier for sequencer)

3. **Meta Events:**
   - Shared: Fully parses (Tempo, TimeSignature, KeySignature, Text)
   - DAW: Completely skips (not needed for playback)

4. **SysEx Events:**
   - Shared: Fully parses and stores data
   - DAW: Completely skips

5. **Error Types:**
   - Shared: 7+ variants with position info, UTF8 errors, IO errors
   - DAW: 4 variants, simpler error messages

6. **VLQ Parsing:**
   - Shared: Separate read_var_len() function returning Option, max 4 bytes
   - DAW: Inline read_variable_length() method, no explicit limit for u64

7. **Event Enum:**
   - Shared: Large enum with many variants and nested fields
   - DAW: Small enum (6 types) in MidiEventType, flattened to struct with Options

---

## 11. FUNCTION SIGNATURES SUMMARY

### Public
```rust
pub fn parse_midi(data: &[u8]) -> Result<MidiPattern, ParseError>
```

### Private
```rust
fn parse_header(reader: &mut MidiReader) -> Result<MidiHeader, ParseError>
fn parse_track(reader: &mut MidiReader) -> Result<Vec<MidiEvent>, ParseError>
```

### MidiReader Methods
```rust
impl<'a> MidiReader<'a> {
    fn new(data: &'a [u8]) -> Self
    fn position(&self) -> usize
    fn read_u8(&mut self) -> Result<u8, ParseError>
    fn peek_u8(&self) -> Result<u8, ParseError>
    fn read_u16(&mut self) -> Result<u16, ParseError>
    fn read_u32(&mut self) -> Result<u32, ParseError>
    fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], ParseError>
    fn read_variable_length(&mut self) -> Result<u64, ParseError>
    fn skip(&mut self, count: usize) -> Result<(), ParseError>
}
```

---

## 12. TESTING STRATEGY RECOMMENDATIONS

### Unit Test Organization (Proposed)

```
Tests for parse_midi (main entry)
├── Empty/minimal files
├── Format variants (0, 1, 2)
├── Multi-track merging
└── Error propagation

Tests for parse_header
├── Valid headers (format 0, 1, 2)
├── Invalid magic
├── Invalid length
├── Unsupported format
└── Edge case TPPQN values

Tests for parse_track
├── Running status (continuation)
├── Running status (error)
├── All 6 channel event types
├── Skip meta events
├── Skip sysex events
├── Track boundaries
└── Event accumulation

Tests for MidiReader
├── read_u8 (bounds checking)
├── read_u16 (big-endian)
├── read_u32 (big-endian)
├── read_bytes (slicing)
├── read_variable_length (VLQ tests)
├── skip (position update)
└── Boundary conditions

Tests for edge cases
├── Zero delta times
├── Large delta times
├── All 16 MIDI channels
├── All note values (0-127)
├── All velocity values (0-127)
├── Pitch bend extremes
└── Track count extremes
```

### Critical Test Coverage

**Must Test (High Priority):**
- Header parsing with all 3 valid formats
- Running status continuation and error
- VLQ parsing for 1, 2, 3, 4+ byte values
- Note On/Off with all channels
- Track merging and sorting
- Incomplete data errors

**Should Test (Medium Priority):**
- Control Change and Program Change
- All optional fields in MidiEvent
- Empty/zero-length tracks
- Events with zero delta time
- Pitch bend min/center/max values

**Can Test (Lower Priority):**
- Aftertouch pressure values
- Controller number ranges
- Program number ranges
- Extreme TPPQN values
- Format 2 files (rarely used)

---

## 13. KNOWN ISSUES / IMPROVEMENTS

### Issue 1: No VLQ Max Length Check
**Severity:** Medium
**Location:** read_variable_length() line 308-320
**Problem:** Will read up to 8 bytes for u64, but MIDI spec allows max 4 bytes
**Impact:** Malformed files could cause unexpected behavior
**Fix:** Add `if byte_count >= 4 { return Err(...) }`

### Issue 2: Meta Events Skipped Silently
**Severity:** Low (by design)
**Location:** parse_track() lines 139-143
**Problem:** All meta events are skipped with no record
**Impact:** Tempo changes, time signatures not captured
**Workaround:** By design - DAW doesn't need this info for playback

### Issue 3: SysEx Events Skipped Silently
**Severity:** Low (by design)
**Problem:** All SysEx events are skipped
**Impact:** Instrument-specific MIDI not captured
**Workaround:** By design - DAW can't replay custom SysEx anyway

### Issue 4: Pitch Bend Not Decoded
**Severity:** Medium
**Location:** parse_track() lines 213-227
**Problem:** LSB/MSB raw bytes captured but not converted to signed value
**Impact:** Pitch bend events stored but value not meaningful
**Fix:** Decode as signed 14-bit value (add decode logic)

### Issue 5: No Unwrap/Panic
**Severity:** None (Good!)
**Status:** Confirmed - zero unwrap() calls found
**Quality:** Excellent error handling

---

## 14. SUMMARY CHECKLIST FOR TEST WRITING

- [ ] **Verify all 4 error types are testable**
  - InvalidFormat (bad magic)
  - UnsupportedFormat (format > 2)
  - IncompleteData (buffer too short)
  - InvalidTrack (running status error)

- [ ] **Test all 6 channel event types captured correctly**
  - NoteOn with channel & velocity
  - NoteOff with channel
  - ControlChange with controller & value
  - ProgramChange with program
  - PitchBend raw bytes
  - Aftertouch raw byte

- [ ] **Test VLQ edge cases**
  - 1-byte VLQ (0x7F)
  - 2-byte VLQ boundary (0x81 0x00)
  - 3-byte and 4-byte VLQs
  - Maximum valid VLQ for u64

- [ ] **Test running status**
  - Continuation across multiple events
  - Clearing by meta/sysex
  - Error when no prior status

- [ ] **Test track and file handling**
  - Merging multiple tracks
  - Sorting by tick
  - All 3 format types
  - Track count variants

- [ ] **Test data structure outputs**
  - MidiPattern.events is populated
  - MidiPattern.ticks_per_quarter_note preserved
  - MidiPattern.total_ticks calculated
  - MidiEvent fields correctly populated

- [ ] **Test boundary conditions**
  - Empty files
  - Files with one event
  - Very large tick values
  - All 16 MIDI channels
  - Note range 0-127
  - Velocity range 0-127

