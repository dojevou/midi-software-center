# DAW MIDI Parser - Analysis Index

This directory contains comprehensive analysis of the DAW's MIDI parser, designed to help with test writing and understanding the codebase.

## Documents

### 1. **DAW_PARSER_ANALYSIS.md** (20 KB)
**Comprehensive medium-thoroughness analysis**

14 sections covering:
- Architecture overview and comparison to shared parser
- All public and private functions with signatures
- Complete error type catalog
- Data structure definitions (MidiPattern, MidiEvent, MidiReader)
- VLQ parsing algorithm explained
- Running status handling with examples
- Complexity analysis (time/space)
- 40+ edge cases that need testing
- Detailed differences between DAW and shared parsers
- Known issues and improvements
- Testing checklist

**Best for:** Understanding the full implementation, finding edge cases, learning how to test

### 2. **DAW_PARSER_QUICK_REFERENCE.md** (5 KB)
**Condensed reference guide**

Covers:
- Public API (1 function)
- Error types (4 variants)
- Data structures (input/output)
- Key behaviors (what's parsed vs skipped)
- VLQ parsing summary
- Running status rules
- MidiReader methods (all 8)
- Complexity summary
- Test priorities (must/should/can)
- Known issues (3 main ones)
- Comparison with shared parser

**Best for:** Quick lookup, getting oriented, reference while coding tests

### 3. **DAW_VS_SHARED_PARSER_COMPARISON.md** (15 KB)
**Side-by-side detailed comparison**

15 sections covering:
- Executive summary
- Architectural purpose of each
- Code organization
- Data flow and transformation diagrams
- Event type handling comparison
- Timing representation (absolute vs relative)
- Error handling strategies
- VLQ implementation differences
- Running status handling (both correct)
- Test coverage status
- Complexity analysis
- Code quality assessment
- When to use each
- Potential improvements
- Summary comparison table

**Best for:** Understanding design choices, knowing why they're different, architectural decisions

---

## Quick Navigation

### If you want to...

**Write comprehensive tests:**
→ Read `DAW_PARSER_ANALYSIS.md` Section 9 (Edge Cases) and Section 12 (Testing Strategy)

**Understand the main functions:**
→ Read `DAW_PARSER_QUICK_REFERENCE.md` or `DAW_PARSER_ANALYSIS.md` Sections 2-3

**Understand VLQ parsing:**
→ Read `DAW_PARSER_ANALYSIS.md` Section 6 or `DAW_PARSER_QUICK_REFERENCE.md`

**Understand running status:**
→ Read `DAW_PARSER_ANALYSIS.md` Section 7 or `DAW_PARSER_QUICK_REFERENCE.md`

**Know why it's different from shared parser:**
→ Read `DAW_VS_SHARED_PARSER_COMPARISON.md` (entire document)

**Get quick reference while coding:**
→ Use `DAW_PARSER_QUICK_REFERENCE.md`

**Understand data structures:**
→ Read `DAW_PARSER_ANALYSIS.md` Section 4 or `DAW_PARSER_QUICK_REFERENCE.md`

**Find error handling patterns:**
→ Read `DAW_PARSER_ANALYSIS.md` Section 5

**Identify potential bugs:**
→ Read `DAW_PARSER_ANALYSIS.md` Section 13 (Known Issues)

---

## File Being Analyzed

**Location:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/core/midi/parser.rs`
**Size:** 330 lines
**Type:** Trusty Module (pure functions, no I/O)
**Classification:** Playback-focused MIDI parser

### What it Does
Parses MIDI file bytes into MidiPattern structure with all tracks merged and sorted by absolute tick position. Designed for real-time sequencer playback, not archival.

### Key Characteristics
- 1 public function: `parse_midi()`
- 2 private functions: `parse_header()`, `parse_track()`
- 1 helper struct: `MidiReader` with 8 methods
- 4 error variants
- Parses 6 event types (Note On/Off, Control Change, Program Change, Pitch Bend, Aftertouch)
- Skips meta events and SysEx
- Converts relative delta times to absolute ticks
- Sorts events by tick position

---

## Key Facts Summary

| Aspect | Value |
|--------|-------|
| **Public Functions** | 1 (parse_midi) |
| **Error Variants** | 4 |
| **Event Types** | 6 |
| **Data Structure** | MidiPattern + MidiEvent |
| **Unwrap Calls** | 0 ✅ |
| **Panic Calls** | 0 ✅ |
| **VLQ Max Check** | ❌ (Potential issue) |
| **Meta Events** | Skipped |
| **Track Handling** | Merged + sorted |
| **Timing** | Absolute ticks |
| **Complexity** | O(n log n) |

---

## Related Files

For comparison and context:

### Shared Library Parser
**Location:** `/home/dojevou/projects/midi-software-center/shared/rust/src/core/midi/parser.rs`
- 921 lines + 400+ lines of tests
- Comprehensive, archival-focused
- Parses all meta events and SysEx
- 100+ test cases
- 91.97% coverage

### DAW MIDI Types
**Location:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/core/midi/types.rs`
- MidiMessage struct and encoding/decoding
- Separate from parser

### DAW MIDI Models
**Location:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/models/midi.rs`
- MidiEvent, MidiPattern, MidiEventType definitions
- Used by parser as output types

---

## Testing Recommendations

### Minimum Coverage (Must Test)
1. Header parsing (all 3 MIDI formats)
2. Running status (continuation and error cases)
3. VLQ edge cases (1, 2, 3, 4 bytes)
4. Note On/Off on all channels (0-15)
5. Track merging and sorting
6. All error conditions (4 error types)

### Recommended Coverage (Should Test)
- Control Change and Program Change
- All MidiEvent optional fields
- Empty and zero-length tracks
- Zero delta time handling
- Pitch bend extremes

### Nice to Have (Can Test)
- All 128 MIDI controller numbers
- All 128 MIDI program numbers
- All note values (0-127)
- All velocity values (0-127)
- Extreme TPPQN values
- Format 2 files (rarely used)

**Overall Goal:** 80%+ code coverage

---

## Analysis Methodology

These documents were created using medium-thoroughness analysis covering:

1. **Code Reading** - Complete file review
2. **Function Analysis** - Every public and private function
3. **Data Flow** - How data transforms through parser
4. **Error Handling** - All error paths
5. **Complexity** - Time and space analysis
6. **Edge Cases** - 40+ potential test cases identified
7. **Comparison** - Detailed comparison with shared parser
8. **Quality Assessment** - Code quality, safety, optimizations

---

## Document Statistics

| Document | Size | Lines | Sections |
|----------|------|-------|----------|
| DAW_PARSER_ANALYSIS.md | 20 KB | 600+ | 14 |
| DAW_PARSER_QUICK_REFERENCE.md | 5 KB | 150+ | 12 |
| DAW_VS_SHARED_PARSER_COMPARISON.md | 15 KB | 500+ | 15 |
| **Total** | **40 KB** | **1250+** | **41** |

---

## Version Information

- **Analysis Date:** 2025-10-27
- **Code Version:** Current main branch
- **Codebase:** MIDI Software Center (Post-Migration)
- **DAW Parser Status:** 0% test coverage (needs improvement)
- **Shared Parser Status:** 91.97% coverage (production-ready)

---

## Next Steps

1. **Write Tests** - Use Section 9 from DAW_PARSER_ANALYSIS.md for test cases
2. **Add VLQ Check** - Implement 4-byte max limit per MIDI spec
3. **Decode Pitch Bend** - Convert raw LSB/MSB to signed 14-bit value
4. **Integration** - Ensure parser works with sequencer engine

---

## Notes

- All documents are markdown format
- No code modifications suggested (only improvements)
- Parser code has zero unwrap/panic calls (excellent)
- Architecture is sound - DAW and shared parsers are correctly optimized for different purposes
- No security issues identified

