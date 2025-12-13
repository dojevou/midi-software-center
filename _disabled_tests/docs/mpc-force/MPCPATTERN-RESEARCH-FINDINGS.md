# .mpcpattern Format Research Findings

**Date:** November 22, 2025
**Status:** In-depth investigation ongoing

---

## 🔍 What We've Discovered

### 1. Commercial .mpcpattern Analysis

Analyzed commercial patterns from Deep House expansion pack. Key findings:

#### Pattern Structure:
```json
{
  "pattern": {
    "length": 9223372036854775807,
    "events": [ /* 634 events */ ]
  }
}
```

#### Unusual Pattern at Start:
**Commercial patterns begin with many "initialization" events at time 0:**

```
Total events: 634
Events at time 0: 44 events
Events at time 0 with velocity 0: 28 events

Breakdown by note:
- Note 0:  7 events (mixed velocities)
- Note 42: 9 events (all velocity 0)
- Note 43: 8 events (all velocity 0)
- Note 44: 8 events (all velocity 0)
- Note 45: 4 events (mixed velocities)
- Note 46: 3 events (all velocity 0)
- Note 55: 5 events (all velocity 0)
```

**Theory:** These are initialization/setup events that tell MPC which pads will be used in the pattern. Velocity 0 may be placeholders or "mute" states.

### 2. Event Format (Confirmed)

Type 1 events are correct:
```json
{
  "type": 1,
  "time": 480,         // MIDI ticks from pattern start
  "len": 0,            // Always 0 for Type 1
  "1": 131,            // Status byte (0x83 = note off, channel 3)
  "2": 0.488,          // Velocity (normalized 0.0-1.0)
  "3": 36,             // MIDI note number
  "mod": 0,            // Modulation type
  "modVal": 0.0        // Modulation value
}
```

### 3. Why Our Converter Might Still Fail

**Hypothesis:** We're missing the initialization events!

Commercial patterns have:
1. **Initialization phase** - Multiple events at time 0 for each note that will be used
2. **Actual pattern events** - Starting from time 0 onwards with real timing

Our converter only creates the pattern events, skipping initialization.

---

## 🛠️ Available Tools

### Midian (Free, Web-based)
- **URL:** http://www.fentonia.com/catnip/midianmpc/index.html
- **Processing:** Client-side (browser), no upload
- **Limitations:** Only converts Note On/Note Off events
- **Advantage:** Free, produces working .mpcpattern files

### MIDI2MPC (Commercial)
- **URL:** https://midi2mpc.com/
- **Cost:** Paid
- **Features:** Track splitting, batch conversion
- **For:** MPC 3.6+

### MIDI-MPC (Gumroad)
- **URL:** https://artur-brahms.gumroad.com/l/yRMHZ
- **Cost:** Paid
- **Note:** Multitrack not yet supported

---

## 🎯 Next Steps to Fix Our Converter

### Step 1: Use Midian as Reference
1. Take one of our test MIDI files: `/media/dojevou/RYXSTR/Expansions/Test_Fixed_5_Patterns/MIDI/Disco_Groove_BD-HH_01.mid`
2. Convert using Midian: http://www.fentonia.com/catnip/midianmpc/index.html
3. Download the .mpcpattern output
4. Compare with our generated output

### Step 2: Analyze Differences
Compare Midian output vs our output:
- Count of events at time 0
- Pattern of velocity 0 events
- Note number distribution
- Total event count
- Event ordering

### Step 3: Reverse Engineer Initialization Logic

Questions to answer:
1. How many "initialization" events per note?
2. What determines the velocity values?
3. Is there a pattern to the ordering?
4. Are notes sorted in any way?
5. Do we need one velocity 0 event per note used?

### Step 4: Update Rust Converter

Based on Midian analysis, implement:
```rust
fn convert_midi_to_mpcpattern(midi_path: &Path) -> Result<MpcPattern> {
    // Step 1: Scan all notes used
    let notes_used = scan_all_notes_in_midi(&midi_file);

    // Step 2: Create initialization events (time 0)
    let mut mpc_events = Vec::new();
    for note in notes_used {
        // Add initialization events (velocity 0?)
        mpc_events.push(create_init_event(note));
    }

    // Step 3: Add actual pattern events
    for event in midi_events {
        mpc_events.push(convert_event(event));
    }

    // Step 4: Sort (by time, then note?)
    mpc_events.sort_by_key(|e| (e.time, e.field3));

    Ok(MpcPattern { pattern: Pattern { length: i64::MAX, events: mpc_events }})
}
```

---

## 📊 Comparison Table

| Aspect | Commercial Pattern | Our Generated Pattern | Issue |
|--------|-------------------|----------------------|-------|
| Total events | 634 | 32 | ✗ Missing many events |
| Time 0 events | 44 | 0 | ✗ No initialization |
| Velocity 0 events | 28 | 0 | ✗ No placeholders |
| Event type | 1 | 1 | ✓ Correct |
| JSON fields | "mod" | "mod" | ✓ Correct |
| Field structure | 1=131, 2=vel, 3=note | 1=131, 2=vel, 3=note | ✓ Correct |

---

## 🧪 Test Plan

### Phase 1: Reference Collection
1. ✓ Analyze commercial .mpcpattern files
2. ✗ Convert test MIDI with Midian
3. ✗ Compare outputs side-by-side

### Phase 2: Implementation
1. Implement initialization event creation
2. Determine velocity 0 event rules
3. Test ordering/sorting logic

### Phase 3: Validation
1. Convert test MIDI with updated converter
2. Load on Force
3. Verify multiple markers appear
4. Verify playback works

---

## 💡 Key Insights

### Insight 1: Initialization is Critical
MPC appears to require "setup" events at time 0 before playing pattern events. Without these, Force may only read the first event.

### Insight 2: Velocity 0 Has Meaning
28 out of 44 initialization events have velocity 0. This is intentional, not an error.

### Insight 3: Note 0 is Special
Note 0 (not a standard MIDI note) appears in commercial patterns with velocity 1. May be a control/marker note.

### Insight 4: Multiple Events per Note
Notes like 42, 43, 44 have 8-9 initialization events EACH. This is not redundant - it's part of the format.

---

## 🚀 Immediate Action Required

**Convert test file with Midian:**
1. Go to: http://www.fentonia.com/catnip/midianmpc/index.html
2. Load: `/media/dojevou/RYXSTR/Expansions/Test_Fixed_5_Patterns/MIDI/Disco_Groove_BD-HH_01.mid`
3. Click "+ MPC" button
4. Download output
5. Save to: `/tmp/midian_disco_01.mpcpattern`

Then we can compare:
```bash
# Our output
cat "/media/dojevou/RYXSTR/Expansions/Test_Fixed_5_Patterns/Patterns/Disco_Groove_BD-HH_01.mpcpattern"

# Midian output
cat "/tmp/midian_disco_01.mpcpattern"

# Difference
diff <(python3 -m json.tool "/media/dojevou/RYXSTR/Expansions/Test_Fixed_5_Patterns/Patterns/Disco_Groove_BD-HH_01.mpcpattern") \
     <(python3 -m json.tool "/tmp/midian_disco_01.mpcpattern")
```

---

## 📚 Resources

### Documentation Found:
- ✗ No official .mpcpattern specification exists
- ✓ Format is JSON-based (confirmed)
- ✓ Type 1 events are standard (confirmed)
- ✗ Initialization logic is undocumented

### Tools Found:
- Midian (free web tool) - **Use this for reference**
- MIDI2MPC (commercial)
- MIDI-MPC (Gumroad)

### Code Found:
- GitHub p3r7/mpc-tools - Has MPC2000XL format, not .mpcpattern
- No open source .mpcpattern converters

---

**Next Action:** Use Midian to convert test MIDI file and analyze output to understand initialization pattern.
