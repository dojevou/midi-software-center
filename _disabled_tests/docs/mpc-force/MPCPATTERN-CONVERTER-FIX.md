# .mpcpattern Converter Fix - November 22, 2025

## 🐛 Problem Identified

User tested converted .mpcpattern files on Akai Force and reported:
- **Only 1 MIDI marker showing** per pattern
- Patterns not playing correctly

## 🔍 Root Cause Analysis

Analyzed commercial .mpcpattern files from working expansion packs and discovered **we were using the wrong event format entirely**.

### Original (Broken) Format - Type 2 Events
```json
{
  "type": 2,           // ❌ WRONG - Type 2 not standard for MPC
  "time": 960,
  "len": 86,           // ❌ Duration-based (note-on + note-off)
  "1": 72,             // ❌ Note number in field1
  "2": 0.078,          // ✓ Velocity correct
  "3": 0,              // ❌ Field3 unused
  "mod_field": 0,      // ❌ Wrong field name
  "modVal": 0.0
}
```

### Correct (Fixed) Format - Type 1 Events
```json
{
  "type": 1,           // ✓ Type 1 = MPC standard
  "time": 960,
  "len": 0,            // ✓ Always 0 for Type 1
  "1": 131,            // ✓ Status byte (0x83 = note off channel 3)
  "2": 0.488,          // ✓ Velocity (normalized 0.0-1.0)
  "3": 36,             // ✓ MIDI note number
  "mod": 0,            // ✓ Correct field name
  "modVal": 0.0
}
```

## 🔧 Changes Made

### 1. Event Type Changed
- **Before:** Type 2 (note-on with duration)
- **After:** Type 1 (standard MPC format)

### 2. Event Structure Fixed
- **field1:** Changed from note number → status byte (131)
- **field2:** Velocity (unchanged, correct)
- **field3:** Changed from 0 → note number
- **len:** Changed from duration → 0

### 3. JSON Field Name
- **Before:** `"mod_field": 0`
- **After:** `"mod": 0`

### 4. Conversion Logic
- **Before:** Wait for note-off, calculate duration, create Type 2 event
- **After:** Create Type 1 event immediately on note-on

### Code Changes

**File:** `pipeline/src-tauri/src/bin/midi_to_mpcpattern.rs`

#### Before (Lines 29-44):
```rust
#[derive(Debug, Serialize, Deserialize)]
struct MpcEvent {
    #[serde(rename = "type")]
    event_type: u8,
    time: i64,
    len: i64,
    #[serde(rename = "1")]
    field1: i32,              // Was: note number
    #[serde(rename = "2")]
    field2: f64,
    #[serde(rename = "3")]
    field3: i32,              // Was: 0
    mod_field: i32,           // Wrong field name
    #[serde(rename = "modVal")]
    mod_val: f64,
}
```

#### After (Lines 29-44):
```rust
#[derive(Debug, Serialize, Deserialize)]
struct MpcEvent {
    #[serde(rename = "type")]
    event_type: u8,
    time: i64,
    len: i64,
    #[serde(rename = "1")]
    field1: i32,              // Now: 131 (status byte)
    #[serde(rename = "2")]
    field2: f64,
    #[serde(rename = "3")]
    field3: i32,              // Now: note number
    #[serde(rename = "mod")]  // Fixed field name
    mod_field: i32,
    #[serde(rename = "modVal")]
    mod_val: f64,
}
```

#### Event Creation Before:
```rust
fn note_on(time: i64, duration: i64, note: u8, velocity: f64) -> Self {
    Self {
        event_type: 2,        // Type 2
        time,
        len: duration,        // Had duration
        field1: note as i32,  // Note in field1
        field2: velocity,
        field3: 0,            // Field3 unused
        mod_field: 0,
        mod_val: 0.0,
    }
}
```

#### Event Creation After:
```rust
fn note_on(time: i64, note: u8, velocity: f64) -> Self {
    Self {
        event_type: 1,        // Type 1
        time,
        len: 0,               // Always 0
        field1: 131,          // Status byte
        field2: velocity,
        field3: note as i32,  // Note in field3
        mod_field: 0,
        mod_val: 0.0,
    }
}
```

## ✅ Test Files Created

**Location:** `/media/dojevou/RYXSTR/Expansions/Test_Fixed_5_Patterns`

### Structure:
```
Test_Fixed_5_Patterns/
├── MIDI/                       # Original MIDI files
│   ├── Disco_Groove_BD-HH_01.mid
│   ├── Disco_Groove_BD-HH_02.mid
│   ├── Disco_Groove_BD-HH_03.mid
│   ├── Disco_Groove_BD-HH_04.mid
│   └── Electro_Groove_BD-HH_01.mid
├── Patterns/                   # Converted .mpcpattern files
│   ├── Disco_Groove_BD-HH_01.mpcpattern (32 events)
│   ├── Disco_Groove_BD-HH_02.mpcpattern (35 events)
│   ├── Disco_Groove_BD-HH_03.mpcpattern (39 events)
│   ├── Disco_Groove_BD-HH_04.mpcpattern (33 events)
│   └── Electro_Groove_BD-HH_01.mpcpattern (39 events)
└── README.txt
```

### Test File Details:
- **5 drum groove patterns** (120 BPM disco/electro)
- **32-39 events each** (multiple MIDI markers)
- **Both .mid and .mpcpattern** included for comparison
- **Same BPM/style** for easy testing

## 📊 Verification

### Pattern Structure Comparison

**Commercial .mpcpattern** (Deep House pack):
```json
{
  "pattern": {
    "length": 9223372036854775807,
    "events": [
      {
        "type": 1,
        "time": 0,
        "len": 0,
        "1": 131,
        "2": 0.976,
        "3": 42,
        "mod": 0,
        "modVal": 0
      }
    ]
  }
}
```

**Our Generated .mpcpattern** (Test files):
```json
{
  "pattern": {
    "length": 9223372036854775807,
    "events": [
      {
        "type": 1,
        "time": 0,
        "len": 0,
        "1": 131,
        "2": 0.566,
        "3": 36,
        "mod": 0,
        "modVal": 0.0
      }
    ]
  }
}
```

**Match:** ✅ Perfect structural match

## 🚀 Next Steps

1. **Test on Force hardware**
   - Load Test_Fixed_5_Patterns expansion
   - Open patterns in sequencer
   - Verify multiple MIDI markers appear
   - Play with drum kit to verify playback

2. **If tests pass:**
   - Convert larger batch (100+ patterns)
   - Build production expansion packs
   - Automate batch conversion

3. **If tests fail:**
   - Check Force firmware version
   - Analyze Force error logs
   - Compare timing values with commercial packs

## 📝 Technical Notes

### Why Type 1 vs Type 2?

Looking at commercial expansion packs:
- **All patterns use Type 1 events**
- Type 2 appears to be unsupported or legacy
- MPC documentation doesn't clearly specify this
- Only discovered by reverse-engineering real files

### Status Byte (field1 = 131)

`131` decimal = `0x83` hex = **Note Off, Channel 3**

MPC uses note-off events (not note-on) because:
- Duration is handled by pattern sequencer
- Note-off triggers sample playback
- Consistent with GM drum mapping

### Timing

- All times are absolute MIDI ticks from pattern start
- Time 0 = beginning of pattern
- Commercial patterns use same timing format

## 🎯 Expected Results

After fix, patterns should:
1. Show **multiple MIDI markers** in Force grid view
2. **Play correctly** with drum kits
3. **Match timing** of original MIDI files
4. Work identically to commercial expansion packs

---

**Fixed:** November 22, 2025, 19:02
**Test files:** Ready on Force drive
**Status:** Awaiting hardware verification
