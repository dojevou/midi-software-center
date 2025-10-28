# DAW Parser Quick Reference

## File Location
`/home/dojevou/projects/midi-software-center/daw/src-tauri/src/core/midi/parser.rs` (330 lines)

## Classification
**Trusty Module** - Pure functions, no I/O, deterministic

## Public API (1 Function)
```rust
pub fn parse_midi(data: &[u8]) -> Result<MidiPattern, ParseError>
```

## Error Types (4 Variants)
1. **InvalidFormat(String)** - Bad magic bytes or wrong header length
2. **UnsupportedFormat(String)** - MIDI format > 2
3. **IncompleteData(usize)** - Buffer too short
4. **InvalidTrack(String)** - Track parsing error or running status error

## Data Structures

### Input
- Raw MIDI file bytes (`&[u8]`)

### Output
```rust
MidiPattern {
    events: Vec<MidiEvent>,           // All tracks merged, sorted by absolute tick
    ticks_per_quarter_note: u16,      // From header
    total_ticks: u64,                 // Calculated from last event
}

MidiEvent {
    event_type: MidiEventType,        // NoteOn, NoteOff, ControlChange, ProgramChange, PitchBend, Aftertouch
    tick: u64,                        // ABSOLUTE tick (not relative delta)
    channel: u8,                      // 0-15
    note: Option<u8>,                 // For NoteOn/NoteOff (0-127)
    velocity: Option<u8>,             // For NoteOn/NoteOff (0-127)
    controller: Option<u8>,           // For ControlChange (0-127)
    value: Option<u8>,                // For ControlChange (0-127)
    program: Option<u8>,              // For ProgramChange (0-127)
}
```

## Key Behaviors

### What Gets Parsed
- Note On (0x90) - note + velocity stored
- Note Off (0x80) - note stored, velocity set to 0
- Control Change (0xB0) - controller + value
- Program Change (0xC0) - program number
- Pitch Bend (0xE0) - raw LSB/MSB captured (not decoded)
- Aftertouch (0xD0) - raw value captured

### What Gets SKIPPED
- Meta events (0xFF) - all of them
- SysEx events (0xF0, 0xF7) - all of them

### Unique Behaviors
1. **All tracks merged** into single event list
2. **Delta times converted** to absolute tick positions
3. **Events sorted** by tick (O(n log n))
4. **Running status** properly handled (can error if no prior status)

## VLQ (Variable-Length Quantity) Parsing
```rust
fn read_variable_length(&mut self) -> Result<u64, ParseError>
```

- Accumulates 7-bit chunks from bytes with continuation bit (0x80)
- Returns u64 (could read up to 8 bytes)
- **No explicit max length check** (MIDI spec allows max 4 bytes) - potential issue

**Examples:**
- 0x7F → 127 (1 byte)
- 0x81 0x00 → 128 (2 bytes)
- 0xFF 0x7F → 16383 (2 bytes)

## Running Status
- If next byte < 0x80: use previous status (data byte)
- If next byte ≥ 0x80: read as new status byte
- Channel events (0x80-0xEF): update running status
- Meta/SysEx (0xFF/0xF0/0xF7): don't update running status
- Error if data byte seen without prior status

## MidiReader Helper (8 methods)
```rust
new(data)              // Create reader
position()             // Get current position
read_u8()              // Read 1 byte big-endian
peek_u8()              // Peek 1 byte without advancing
read_u16()             // Read 2 bytes big-endian
read_u32()             // Read 4 bytes big-endian
read_bytes(count)      // Read slice of N bytes
read_variable_length() // Read VLQ
skip(count)            // Advance position without reading
```

## Complexity
- **parse_midi():** O(n log n) - file size n, dominated by sort
- **parse_header():** O(1) - always 14 bytes
- **parse_track():** O(m) - m = events in track
- **read_variable_length():** O(1) avg, O(k) worst where k ≤ 8 for u64

## Test Coverage Priorities

### Must Test
1. Header parsing (all 3 formats)
2. Running status continuation & error
3. VLQ edge cases (1, 2, 3, 4 bytes)
4. Note On/Off all channels
5. Track merging & sorting
6. Incomplete data errors

### Should Test
1. Control Change / Program Change
2. Optional fields in MidiEvent
3. Zero delta times
4. Pitch bend extremes
5. Empty tracks

### Can Test
1. All controller numbers
2. Extreme TPPQN values
3. Format 2 files
4. Large tick values

## Key Differences vs Shared Parser

| Aspect | DAW | Shared |
|--------|-----|--------|
| Purpose | Real-time playback | Archival & analysis |
| Track handling | Merged | Separate |
| Delta time | Absolute ticks | Relative delta_ticks |
| Meta events | Skipped | Fully parsed |
| Error variants | 4 | 7+ |
| VLQ max check | None | 4 bytes |

## Known Issues

1. **No VLQ max length check** (Medium severity)
   - Could read > 4 bytes (MIDI spec violation)
   - Malformed files might cause unexpected behavior

2. **Pitch Bend not decoded** (Medium severity)
   - LSB/MSB stored raw, not converted to signed value
   - Events exist but value not meaningful

3. **Meta/SysEx silently skipped** (Low severity - by design)
   - No record of these events
   - OK for playback-only use

## Code Quality
- ✅ Zero unwrap/expect/panic calls
- ✅ All errors propagated properly
- ✅ Bounds checking on all reads
- ✅ Consistent error handling

