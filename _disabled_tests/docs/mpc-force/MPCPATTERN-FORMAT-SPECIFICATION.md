# .mpcpattern File Format Specification

**Reverse Engineered:** November 22, 2025
**Source:** Akai Force/MPC commercial expansion packs
**Format:** Plain JSON

---

## Overview

The `.mpcpattern` file format is Akai's proprietary MIDI pattern format used in MPC/Force hardware and software. **It's simply a JSON file** containing MIDI-like event data.

## File Structure

```json
{
  "pattern": {
    "length": 9223372036854775807,
    "events": [ ... ]
  }
}
```

### Top-Level Fields

| Field | Type | Description |
|-------|------|-------------|
| `pattern.length` | integer | Pattern length (typically max int64: 9223372036854775807) |
| `pattern.events` | array | Array of MIDI-like events |

---

## Event Types

### Type 1: Note Off / Automation

Represents note-off messages or automation data.

```json
{
  "type": 1,
  "time": 0,
  "len": 0,
  "1": 131,
  "2": 0.0,
  "3": 42,
  "mod": 0,
  "modVal": 0.0
}
```

**Fields:**

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `type` | integer | `1` | Event type identifier (note off) |
| `time` | integer | `0-∞` | Timestamp in MIDI ticks |
| `len` | integer | `0` | Always 0 for Type 1 |
| `"1"` | integer | `128-143` | MIDI status byte (0x80-0x8F = Note Off) |
| `"2"` | float | `0.0-1.0` | Normalized velocity (0.0 = vel 0, 1.0 = vel 127) |
| `"3"` | integer | `0-127` | MIDI note number |
| `mod` | integer | `0` | Modulation parameter (usually 0) |
| `modVal` | float | `0.0` | Modulation value (usually 0) |

**Notes:**
- Field `"1"` = 131 (0x83) = Note Off on Channel 3
- Most Type 1 events have `"2"` = 0 (zero velocity note-off)
- Field `"3"` contains the note number to turn off

---

### Type 2: Note On

Represents note-on messages with duration.

```json
{
  "type": 2,
  "time": 240,
  "len": 180,
  "1": 44,
  "2": 0.7086614370346069,
  "3": 0,
  "mod": 0,
  "modVal": 0.0
}
```

**Fields:**

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `type` | integer | `2` | Event type identifier (note on) |
| `time` | integer | `0-∞` | Timestamp in MIDI ticks from pattern start |
| `len` | integer | `0-∞` | Note duration in MIDI ticks |
| `"1"` | integer | `0-127` | MIDI note number |
| `"2"` | float | `0.0-1.0` | Normalized velocity (0.0 = vel 0, 1.0 = vel 127) |
| `"3"` | integer | `0` | Usually 0 (possibly channel/track) |
| `mod` | integer | `0` | Modulation parameter |
| `modVal` | float | varies | Modulation value (possibly swing/humanization) |

**Notes:**
- `time` is absolute from pattern start
- `len` determines note duration (no explicit note-off needed)
- Velocity is normalized: `midi_velocity = int(velocity_float * 127)`
- `modVal` varies and may control timing humanization or swing

---

## MIDI Tick Timing

- **Default PPQN:** 480 ticks per quarter note (common in MPC)
- **Time = 0:** First beat of pattern
- **Time = 480:** One quarter note later
- **Time = 240:** One eighth note later
- **Time = 120:** One sixteenth note later

---

## Common MIDI Note Numbers (GM Drums)

From analyzed patterns, common drum notes:

| Note | Name |
|------|------|
| 36 | Bass Drum 1 |
| 37 | Side Stick |
| 38 | Acoustic Snare |
| 42 | Closed Hi-Hat |
| 43 | Low Floor Tom |
| 44 | Pedal Hi-Hat |
| 45 | Low Tom |
| 46 | Open Hi-Hat |
| 49 | Crash Cymbal 1 |
| 51 | Ride Cymbal 1 |
| 53 | Ride Bell |
| 55 | Splash Cymbal |

---

## Example Pattern Analysis

**Source:** `DeepHouse-Kit-DH Kit 02 122-Pattern.mpcpattern`

- **Total Events:** 634
- **Type 1 (Note Off):** 482 events
- **Type 2 (Note On):** 152 events
- **Instruments:** Side Stick, Floor Tom, Pedal Hi-Hat, Low Tom, Ride, Ride Bell, Splash
- **Velocity Range:** 0.4173 to 1.0 (53-127 in MIDI velocity)

---

## Converting MIDI to .mpcpattern

### Algorithm:

1. Parse MIDI file
2. Extract note on/off events
3. Calculate note durations (note_off_time - note_on_time)
4. Normalize velocities (velocity / 127.0)
5. Create Type 2 events for each note
6. Optionally create Type 1 events for note-offs
7. Save as JSON

### Minimal Example:

```json
{
  "pattern": {
    "length": 9223372036854775807,
    "events": [
      {
        "type": 2,
        "time": 0,
        "len": 480,
        "1": 36,
        "2": 1.0,
        "3": 0,
        "mod": 0,
        "modVal": 0
      }
    ]
  }
}
```

This creates a single kick drum (note 36) at the start, full velocity, lasting one quarter note.

---

## Key Insights

1. ✅ **Format is JSON** - No binary parsing needed!
2. ✅ **Simple structure** - Just pattern + events array
3. ✅ **Type 2 events only** - Note-offs are implicit via `len` field
4. ✅ **Type 1 events optional** - Explicit note-offs (rarely needed)
5. ✅ **Normalized velocities** - 0.0-1.0 instead of 0-127
6. ✅ **Duration-based** - No need for paired note on/off

---

## Advantages Over Standard MIDI

- **Human-readable** - JSON format
- **Simpler note model** - Duration instead of note-on/off pairs
- **Easy to edit** - Text editor or JSON tools
- **No binary parsing** - Standard JSON libraries work
- **Normalized values** - Cleaner 0.0-1.0 range

---

## Conversion Strategy

**For this project:**

1. Query PostgreSQL for best MIDI files
2. Parse with `midly` crate (already in codebase)
3. Convert to .mpcpattern JSON structure
4. Save to Force drive

**No external tools needed!** We can build our own converter using existing Rust MIDI parser.

---

## Next Steps

1. Build Rust converter: `midi_to_mpcpattern.rs`
2. Test with sample MIDI files
3. Batch convert 5,000-10,000 patterns
4. Copy to Force drive
5. Test on hardware

---

**Bottom Line:** The `.mpcpattern` format is surprisingly simple - just JSON with MIDI note data. We can easily create our own converter instead of relying on external tools.
