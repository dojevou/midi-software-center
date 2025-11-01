# Phase 1.3: MIDI Parser Module Testing Plan

**Module:** `shared/rust/src/core/midi/parser.rs`
**Status:** 🟡 Planning
**Started:** 2025-10-27
**Target:** 80%+ coverage (Trusty Module requirement)
**Complexity:** HIGH - Most complex module in Phase 1

---

## Module Overview

### File Details
- **Location:** `shared/rust/src/core/midi/parser.rs`
- **Total Lines:** 921 lines (production code ~400-500 lines)
- **Current Coverage:** 0/137 lines (0% - completely untested)
- **Functions:** ~20 parsing functions
- **Error Handling:** Uses MidiParseError (tested in Phase 1.2)

### Key Components

**1. Core Parsing Functions:**
- `parse_midi_file()` - Main entry point (reads entire MIDI file)
- `parse_header()` - Parses MIDI header chunk (MThd)
- `parse_track()` - Parses track chunk (MTrk)
- `parse_event()` - Parses individual MIDI events
- `parse_meta_event()` - Parses meta events (tempo, time signature, etc.)
- `parse_sysex_event()` - Parses system exclusive events

**2. Utility Functions:**
- `read_var_len()` - Variable-length quantity parser (CRITICAL)
- `read_bytes()` - Safe byte reading with bounds checking
- `read_u16()`, `read_u32()` - Primitive type readers

**3. Data Structures (from types.rs - tested in Phase 1.1):**
- `MidiFile` - Top-level structure
- `Track` - MIDI track
- `MidiEvent` - Individual events
- `MidiMessage` - Encoded messages

### Dependencies
- ✅ `error.rs` - Phase 1.2 complete (31 tests, 100% coverage)
- ✅ `types.rs` - Phase 1.1 complete (50 tests, 85% coverage)
- ⏳ Test fixtures needed (sample MIDI files)

---

## Complexity Assessment

### Why This Is the Most Complex Module

**1. Parsing Logic Complexity:**
- Binary format parsing (byte-level operations)
- Variable-length quantities (7-bit encoding)
- Multiple event types (note, meta, sysex)
- Running status handling (MIDI optimization)
- Temporal ordering and delta times

**2. Error Handling:**
- 30+ error sites in parser.rs
- Boundary checking on all reads
- Malformed data detection
- Corruption recovery

**3. Test Requirements:**
- Valid MIDI files (multiple formats: 0, 1, 2)
- Malformed files (corrupted headers, truncated data)
- Edge cases (empty tracks, zero delta times)
- Large files (memory safety)
- All event types (note on/off, CC, meta, sysex)

**4. Coverage Challenges:**
- Many conditional branches
- Error paths need explicit testing
- Edge cases in binary parsing
- Format variants (Format 0, 1, 2)

---

## Tool-Enhanced Workflow

This phase uses Claude Code's full toolkit for maximum efficiency and quality.

### Slash Commands (4 used)

| Command | When | Purpose |
|---------|------|---------|
| `/unit-test-generator:generate-tests` | Step 3 | Generate comprehensive test boilerplate |
| `/test-coverage-analyzer:analyze-coverage` | Step 8 | Measure line/branch coverage, identify gaps |
| `/database-test-manager:db-test` | Step 4 (optional) | If parser needs DB integration tests |
| `/git-commit-smart:commit-smart` | Step 10 | Generate semantic commit message |

### Specialized Agents (5 used)

| Agent | When | Purpose |
|-------|------|---------|
| `rust-backend` | Step 9 | Expert review of parser tests and implementation |
| `midi-hardware` | Step 5 | Review MIDI-specific parsing logic and standards |
| `security-sentinel` | Step 6 | Audit for buffer overflows, memory safety issues |
| `pattern-recognition-specialist` | Step 2 | Analyze parser patterns and test requirements |
| `best-practices-researcher` | Step 4 | Research MIDI parsing test strategies |

### MCP Servers (5 active)

| Server | Usage | Examples |
|--------|-------|----------|
| `rust` (stdio) | Cargo operations, testing, analysis | `cargo test`, `cargo tarpaulin` |
| `filesystem` (stdio) | Create test fixtures, read MIDI files | Copy sample MIDIs, create test data |
| `git` (stdio) | Version control, commit tracking | Track progress, create commits |
| `web-search` (stdio) | Research MIDI format specs | Look up MIDI 1.0 spec, SMF format |
| `postgres` (stdio) | N/A for parser (pure logic) | Not used in this phase |

### Plugins (2 active)

| Plugin | Usage | Notes |
|--------|-------|-------|
| `unit-test-generator` | Generate test boilerplate | Saves 30-60 minutes of manual test writing |
| `git-commit-smart` | Semantic commit messages | Conventional commit format |

---

## Test Categories (60+ tests planned)

### 1. Valid MIDI File Parsing (15 tests)

**Format 0 (Single Track):**
- Simple melody (4 notes)
- Polyphonic music (chords)
- Full song with tempo/time signature changes

**Format 1 (Multiple Tracks):**
- 2 tracks (melody + bass)
- 4 tracks (drums + bass + chords + melody)
- 16 tracks (full arrangement)

**Format 2 (Multiple Sequences):**
- 2 independent sequences
- Pattern-based arrangement

**Event Coverage:**
- Note On/Off
- Control Changes (CC 1, 7, 10, 11, 64, 91, 93)
- Program Changes
- Pitch Bend
- Aftertouch
- System Exclusive

**Meta Events:**
- Tempo changes
- Time signature changes
- Key signature
- Track names
- Lyrics
- Markers
- End of Track

### 2. Header Parsing (8 tests)

**Valid Headers:**
- Format 0 header
- Format 1 header
- Format 2 header
- Different divisions (480, 960, 192, 96 TPQN)

**Invalid Headers:**
- Wrong magic number ("MThX" instead of "MThd")
- Incorrect chunk size
- Invalid format number (3, 99, 255)
- Truncated header

### 3. Track Parsing (10 tests)

**Valid Tracks:**
- Empty track (just End of Track meta)
- Single note track
- Complex track (100+ events)
- Track with all meta events

**Invalid Tracks:**
- Wrong magic number ("MTRK" instead of "MTrk")
- Incorrect chunk size
- Missing End of Track
- Truncated events
- Corrupted delta times

### 4. Event Parsing (12 tests)

**MIDI Channel Events:**
- Note On (velocity > 0)
- Note On (velocity = 0, treated as Note Off)
- Note Off
- Polyphonic Aftertouch
- Control Change
- Program Change
- Channel Aftertouch
- Pitch Bend

**Running Status:**
- Consecutive notes with running status
- Mixed events with running status
- Running status interrupted by meta event

**System Common/Realtime:**
- System Exclusive (F0...F7)
- Meta events (FF)

### 5. Meta Event Parsing (8 tests)

- Sequence Number (FF 00)
- Text Events (FF 01-07)
- MIDI Channel Prefix (FF 20)
- End of Track (FF 2F) - CRITICAL
- Set Tempo (FF 51)
- SMPTE Offset (FF 54)
- Time Signature (FF 58)
- Key Signature (FF 59)

### 6. Variable-Length Quantity (VLQ) Parsing (5 tests)

**CRITICAL FUNCTION - Many bugs hide here:**
- Single byte (0x00 - 0x7F)
- Two bytes (0x80 0x00 - 0xFF 0x7F)
- Three bytes
- Four bytes (maximum: 0x0FFFFFFF)
- Invalid VLQ (continuation bit errors)

### 7. Error Handling (8 tests)

**File-Level Errors:**
- Empty file
- File too small (< 14 bytes)
- Non-MIDI file (wrong magic)

**Parsing Errors:**
- Unexpected EOF (truncated file)
- Invalid event status byte
- Malformed meta event
- Corrupted VLQ
- Invalid track count

### 8. Edge Cases (6 tests)

**Boundary Conditions:**
- Zero-length tracks
- Maximum track count (65535)
- Very long delta times
- Zero delta times (simultaneous events)
- Very large files (> 10MB)
- Minimal valid MIDI (14-byte header + empty track)

### 9. Security Tests (5 tests)

**Memory Safety:**
- Buffer overflow attempts (read past end)
- Integer overflow in chunk sizes
- Malicious VLQ (crafted to overflow)
- Extremely large file claims (DOS prevention)
- Recursive/nested structure limits

### 10. Round-Trip Tests (3 tests)

**Parsing → Serialization → Parsing:**
- Parse MIDI → serialize → parse again → compare
- Ensures lossless round-trip
- Uses writer.rs (if available, or defer to Phase 1.4)

---

## Execution Plan

### Phase 1.3 Timeline: 180 minutes (3 hours)

| Step | Task | Time | Tools Used |
|------|------|------|------------|
| 1 | Analyze parser module | 20 min | Read, Grep, pattern-recognition-specialist agent |
| 2 | Create test fixtures (sample MIDIs) | 30 min | filesystem MCP, web-search MCP, best-practices-researcher |
| 3 | Generate test boilerplate | 20 min | /unit-test-generator slash command |
| 4 | Enhance tests (valid parsing) | 30 min | rust MCP, Edit tool |
| 5 | Add MIDI-specific tests | 20 min | midi-hardware agent |
| 6 | Add security tests | 15 min | security-sentinel agent |
| 7 | Run all tests | 10 min | rust MCP (cargo test) |
| 8 | Measure coverage | 15 min | /test-coverage-analyzer |
| 9 | Expert review | 10 min | rust-backend agent |
| 10 | Commit changes | 10 min | /git-commit-smart, git MCP |

**Total:** 180 minutes (3 hours)

---

## Step-by-Step Execution

### Step 1: Analyze Parser Module (20 minutes)

**Objectives:**
- Understand parser structure and functions
- Identify all parsing functions
- Map error handling sites
- Determine test fixture needs

**Commands:**
```bash
# Read the parser module
Read shared/rust/src/core/midi/parser.rs

# Find all public functions
grep "pub fn" shared/rust/src/core/midi/parser.rs

# Count error sites
grep "MidiParseError" shared/rust/src/core/midi/parser.rs | wc -l

# Identify dependencies
grep "use " shared/rust/src/core/midi/parser.rs
```

**Agent:**
```bash
# Launch pattern-recognition-specialist to analyze parser patterns
Task agent:
  subagent_type: pattern-recognition-specialist
  prompt: "Analyze shared/rust/src/core/midi/parser.rs and identify:
           1. All parsing functions and their responsibilities
           2. Common error patterns
           3. Test coverage requirements
           4. Suggested test categories

           Focus on binary parsing patterns, error handling, and
           boundary conditions."
```

**Deliverable:** Analysis document with function list, error sites, test requirements

---

### Step 2: Create Test Fixtures (30 minutes)

**Objectives:**
- Create minimal valid MIDI files for each format (0, 1, 2)
- Create malformed MIDI files for error testing
- Document fixture structure

**MIDI File Fixtures Needed:**

**Valid Files:**
1. `format0_simple.mid` - Single track, 4 notes, 120 BPM
2. `format1_two_tracks.mid` - 2 tracks (melody + bass)
3. `format2_sequences.mid` - 2 independent sequences
4. `all_events.mid` - All MIDI event types
5. `meta_events.mid` - All meta event types

**Malformed Files:**
1. `invalid_magic.mid` - Wrong header magic ("MThX")
2. `truncated_header.mid` - Incomplete header
3. `truncated_track.mid` - Incomplete track data
4. `invalid_format.mid` - Format number 99
5. `corrupted_vlq.mid` - Invalid variable-length quantity
6. `missing_eot.mid` - Track without End of Track

**Test Fixture Location:**
```
shared/rust/tests/fixtures/midi/
├── valid/
│   ├── format0_simple.mid
│   ├── format1_two_tracks.mid
│   ├── format2_sequences.mid
│   ├── all_events.mid
│   └── meta_events.mid
└── invalid/
    ├── invalid_magic.mid
    ├── truncated_header.mid
    ├── truncated_track.mid
    ├── invalid_format.mid
    ├── corrupted_vlq.mid
    └── missing_eot.mid
```

**Commands:**
```bash
# Create fixture directories
mkdir -p shared/rust/tests/fixtures/midi/{valid,invalid}

# Option 1: Generate fixtures programmatically (Rust)
# Create a test helper that builds MIDI bytes

# Option 2: Use existing MIDI files (if available)
# Search for sample MIDIs online (public domain)

# Option 3: Hand-craft minimal MIDI files
# Use hex editor or Rust to write byte arrays
```

**Agent:**
```bash
# Research MIDI test strategies
Task agent:
  subagent_type: best-practices-researcher
  prompt: "Research best practices for testing MIDI file parsers:
           1. How to create minimal test fixtures
           2. Common edge cases in MIDI parsing
           3. Examples from other MIDI libraries (e.g., midly, rimd)
           4. Security considerations for binary parsers

           Find specific test cases from successful Rust MIDI projects."
```

**Deliverable:** 11 MIDI test fixtures + documentation

---

### Step 3: Generate Test Boilerplate (20 minutes)

**Objectives:**
- Use /unit-test-generator to create initial test structure
- Generate tests for all major parsing functions
- Organize tests by category

**Command:**
```bash
/unit-test-generator:generate-tests
```

**Prompt for Generator:**
```
Generate comprehensive unit tests for shared/rust/src/core/midi/parser.rs

Context:
- MIDI file parser (binary format)
- ~20 parsing functions (header, track, events, VLQ)
- Uses MidiParseError for all errors
- Needs test fixtures (11 MIDI files created in Step 2)

Test Categories Needed:
1. Valid MIDI parsing (15 tests - format 0/1/2, all events)
2. Header parsing (8 tests - valid/invalid headers)
3. Track parsing (10 tests - valid/invalid tracks)
4. Event parsing (12 tests - all event types)
5. Meta event parsing (8 tests - all meta types)
6. VLQ parsing (5 tests - single to 4-byte VLQs)
7. Error handling (8 tests - truncation, corruption)
8. Edge cases (6 tests - boundaries, large files)
9. Security (5 tests - overflow, malicious input)

Focus on:
- Using test fixtures from shared/rust/tests/fixtures/midi/
- Testing all public functions
- Covering error paths
- Binary parsing edge cases
```

**Expected Output:**
- 60-80 test function stubs
- Organized in modules (valid_parsing, error_handling, edge_cases, security)
- Assert statements for expected behavior
- Fixture loading helpers

**Deliverable:** Test boilerplate with 60-80 test stubs

---

### Step 4: Enhance Tests - Valid Parsing (30 minutes)

**Objectives:**
- Implement valid MIDI file parsing tests
- Verify correct parsing of all MIDI formats
- Test all event types

**Implementation:**

**Format Tests:**
```rust
#[test]
fn test_parse_format0_simple() {
    let data = include_bytes!("../../tests/fixtures/midi/valid/format0_simple.mid");
    let midi = parse_midi_file(data).expect("Should parse valid format 0 MIDI");

    assert_eq!(midi.format, 0);
    assert_eq!(midi.tracks.len(), 1);
    assert_eq!(midi.division, 480);

    // Verify events
    let track = &midi.tracks[0];
    assert_eq!(track.events.len(), 9); // 4 note-ons + 4 note-offs + EOT
}

#[test]
fn test_parse_format1_two_tracks() {
    let data = include_bytes!("../../tests/fixtures/midi/valid/format1_two_tracks.mid");
    let midi = parse_midi_file(data).expect("Should parse valid format 1 MIDI");

    assert_eq!(midi.format, 1);
    assert_eq!(midi.tracks.len(), 2);

    // Track 0: Tempo map
    // Track 1: Melody
    // Track 2: Bass
}
```

**Event Tests:**
```rust
#[test]
fn test_parse_all_event_types() {
    let data = include_bytes!("../../tests/fixtures/midi/valid/all_events.mid");
    let midi = parse_midi_file(data).unwrap();

    let events = &midi.tracks[0].events;

    // Verify we have each event type
    assert!(events.iter().any(|e| matches!(e.event_type, MidiEventType::NoteOn)));
    assert!(events.iter().any(|e| matches!(e.event_type, MidiEventType::NoteOff)));
    assert!(events.iter().any(|e| matches!(e.event_type, MidiEventType::ControlChange)));
    assert!(events.iter().any(|e| matches!(e.event_type, MidiEventType::ProgramChange)));
    assert!(events.iter().any(|e| matches!(e.event_type, MidiEventType::PitchBend)));
}
```

**VLQ Tests:**
```rust
#[test]
fn test_read_var_len_single_byte() {
    let data = &[0x00];
    let (value, bytes_read) = read_var_len(data).unwrap();
    assert_eq!(value, 0);
    assert_eq!(bytes_read, 1);

    let data = &[0x7F];
    let (value, bytes_read) = read_var_len(data).unwrap();
    assert_eq!(value, 127);
    assert_eq!(bytes_read, 1);
}

#[test]
fn test_read_var_len_two_bytes() {
    let data = &[0x81, 0x00];
    let (value, bytes_read) = read_var_len(data).unwrap();
    assert_eq!(value, 128);
    assert_eq!(bytes_read, 2);
}

#[test]
fn test_read_var_len_maximum() {
    // Maximum VLQ: 0x0FFFFFFF (4 bytes)
    let data = &[0xFF, 0xFF, 0xFF, 0x7F];
    let (value, bytes_read) = read_var_len(data).unwrap();
    assert_eq!(value, 0x0FFFFFFF);
    assert_eq!(bytes_read, 4);
}
```

**Deliverable:** 30-40 working tests for valid parsing

---

### Step 5: Add MIDI-Specific Tests (20 minutes)

**Objectives:**
- Validate MIDI standard compliance
- Test running status behavior
- Verify meta event handling

**Agent:**
```bash
Task agent:
  subagent_type: midi-hardware
  prompt: "Review the MIDI parser tests in shared/rust/src/core/midi/parser.rs
           and ensure compliance with MIDI 1.0 Standard File Format spec.

           Specifically check:
           1. Running status implementation (consecutive notes)
           2. Meta event parsing (all FF types)
           3. Delta time handling (0 delta = simultaneous)
           4. Tempo and time signature parsing
           5. SMPTE time code support

           Identify any missing test cases for MIDI compliance."
```

**Implementation:**
```rust
#[test]
fn test_running_status() {
    // Running status: First note has full status byte,
    // subsequent notes omit it (optimization)
    let data = &[
        // Header (14 bytes)
        b'M', b'T', b'h', b'd',
        0, 0, 0, 6,  // chunk size
        0, 0,        // format 0
        0, 1,        // 1 track
        0, 0x60,     // 96 TPQN

        // Track (with running status)
        b'M', b'T', b'r', b'k',
        0, 0, 0, 13, // chunk size

        0x00,        // delta time
        0x90, 60, 64, // Note On C4, velocity 64

        0x10,        // delta time (16 ticks)
        61, 64,      // Note On C#4 (running status - no 0x90)

        0x10,        // delta time
        62, 64,      // Note On D4 (running status)

        0x00,        // delta time
        0xFF, 0x2F, 0x00, // End of Track
    ];

    let midi = parse_midi_file(data).unwrap();
    let events = &midi.tracks[0].events;

    // Should have 3 note-on events
    let note_ons: Vec<_> = events.iter()
        .filter(|e| matches!(e.event_type, MidiEventType::NoteOn))
        .collect();

    assert_eq!(note_ons.len(), 3);
    assert_eq!(note_ons[0].data1, 60); // C4
    assert_eq!(note_ons[1].data1, 61); // C#4
    assert_eq!(note_ons[2].data1, 62); // D4
}

#[test]
fn test_tempo_meta_event() {
    // FF 51 03 tt tt tt (Set Tempo)
    let data = &[0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]; // 500,000 µs/quarter

    let event = parse_meta_event(data, 0).unwrap();

    match event.event_type {
        MetaEventType::SetTempo(tempo) => {
            assert_eq!(tempo, 500_000); // 120 BPM
        }
        _ => panic!("Expected SetTempo meta event"),
    }
}
```

**Deliverable:** 10-15 MIDI-compliance tests

---

### Step 6: Add Security Tests (15 minutes)

**Objectives:**
- Test buffer overflow prevention
- Test integer overflow handling
- Test malicious input rejection

**Agent:**
```bash
Task agent:
  subagent_type: security-sentinel
  prompt: "Review shared/rust/src/core/midi/parser.rs for security vulnerabilities.

           Focus on:
           1. Buffer overflow risks (reading past end of data)
           2. Integer overflow in chunk sizes
           3. Malicious VLQ values (crafted to overflow)
           4. DOS via extremely large files
           5. Memory exhaustion attacks

           Suggest specific test cases for security validation."
```

**Implementation:**
```rust
#[test]
fn test_buffer_overflow_prevention() {
    // Claim 100 bytes but only provide 10
    let data = &[
        b'M', b'T', b'h', b'd',
        0, 0, 0, 6,
        0, 0, 0, 1, 0, 0x60,

        b'M', b'T', b'r', b'k',
        0, 0, 0, 100,  // Claims 100 bytes
        // Only 10 bytes follow
        0x00, 0x90, 60, 64,
        0x00, 0x80, 60, 64,
        0xFF, 0x2F,
    ];

    let result = parse_midi_file(data);
    assert!(result.is_err());

    match result.unwrap_err() {
        MidiParseError::IncompleteData { expected, actual } => {
            assert_eq!(expected, 100);
            assert!(actual < 100);
        }
        _ => panic!("Expected IncompleteData error"),
    }
}

#[test]
fn test_integer_overflow_in_chunk_size() {
    let data = &[
        b'M', b'T', b'h', b'd',
        0xFF, 0xFF, 0xFF, 0xFF,  // Maximum u32 chunk size
        0, 0, 0, 1, 0, 0x60,
    ];

    let result = parse_midi_file(data);
    assert!(result.is_err());
}

#[test]
fn test_malicious_vlq() {
    // VLQ with all continuation bits set (would overflow)
    let data = &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

    let result = read_var_len(data);
    assert!(result.is_err());

    match result.unwrap_err() {
        MidiParseError::InvalidVarLen(_) => {},
        _ => panic!("Expected InvalidVarLen error"),
    }
}

#[test]
fn test_dos_prevention_large_track_count() {
    let data = &[
        b'M', b'T', b'h', b'd',
        0, 0, 0, 6,
        0, 1,        // Format 1
        0xFF, 0xFF,  // 65535 tracks (would allocate massive Vec)
        0, 0x60,
    ];

    // Should reject or handle gracefully
    let result = parse_midi_file(data);
    // Either error or allocate reasonably
    if let Ok(midi) = result {
        assert!(midi.tracks.len() < 10000, "Should limit track allocation");
    }
}

#[test]
fn test_memory_exhaustion_prevention() {
    // Test that parser doesn't allocate based on unvalidated input
    let data = &[
        b'M', b'T', b'h', b'd',
        0, 0, 0, 6,
        0, 0, 0, 1, 0, 0x60,

        b'M', b'T', b'r', b'k',
        0xFF, 0xFF, 0xFF, 0xFE,  // Claims 4GB track
        0x00,
    ];

    let result = parse_midi_file(data);
    assert!(result.is_err(), "Should reject unreasonably large chunks");
}
```

**Deliverable:** 5-8 security tests

---

### Step 7: Run All Tests (10 minutes)

**Objectives:**
- Execute full test suite
- Verify all tests pass
- Fix any failing tests

**Commands:**
```bash
# Run parser tests specifically
cargo test --package midi-library-shared --lib core::midi::parser -- --nocapture

# Run with verbose output
cargo test --package midi-library-shared --lib core::midi::parser -- --nocapture --show-output

# Check test count
cargo test --package midi-library-shared --lib core::midi::parser -- --list | wc -l
```

**Expected Output:**
```
running 65 tests
test core::midi::parser::tests::valid_parsing::test_parse_format0_simple ... ok
test core::midi::parser::tests::valid_parsing::test_parse_format1_two_tracks ... ok
test core::midi::parser::tests::header::test_parse_valid_header ... ok
test core::midi::parser::tests::vlq::test_read_var_len_single_byte ... ok
[... 61 more tests ...]

test result: ok. 65 passed; 0 failed; 0 ignored; 0 measured
```

**If Tests Fail:**
1. Read error messages carefully
2. Check test fixture data
3. Verify parser logic matches MIDI spec
4. Fix implementation bugs (not tests)
5. Re-run until all pass

**Deliverable:** All 65+ tests passing

---

### Step 8: Measure Coverage (15 minutes)

**Objectives:**
- Run cargo tarpaulin to measure line/branch coverage
- Identify uncovered code paths
- Add tests for gaps

**Command:**
```bash
/test-coverage-analyzer:analyze-coverage

# Or manually:
cargo tarpaulin --package midi-library-shared \
  --lib --skip-clean --timeout 120 \
  --out Html --output-dir ./coverage_parser \
  -- core::midi::parser::tests
```

**Analysis:**
```bash
# View coverage report
firefox coverage_parser/index.html

# Check specific coverage
grep "parser.rs" coverage_parser/cobertura.xml
```

**Target:**
- **Line Coverage:** 80%+ (Trusty Module requirement)
- **Branch Coverage:** 70%+ (good for binary parser)
- **Function Coverage:** 90%+ (all public functions)

**Gap Analysis:**
If coverage < 80%:
1. Identify uncovered lines (tarpaulin report)
2. Determine if lines are:
   - Error paths (add error tests)
   - Edge cases (add boundary tests)
   - Dead code (remove or document)
3. Add missing tests
4. Re-measure

**Expected Coverage:**
```
Coverage Report:
==================
Overall: 82.3%

By Function:
  parse_midi_file()    95.2%  ✅
  parse_header()       100.0% ✅
  parse_track()        88.4%  ✅
  parse_event()        79.1%  ⚠️
  parse_meta_event()   85.0%  ✅
  read_var_len()       100.0% ✅

Uncovered Lines:
  parser.rs:234-236 (rare meta event type)
  parser.rs:401-403 (sysex escape sequence)
  parser.rs:512 (unreachable error branch)
```

**Deliverable:** Coverage report showing 80%+ line coverage

---

### Step 9: Expert Review (10 minutes)

**Objectives:**
- Get rust-backend agent review of parser tests
- Verify MIDI compliance
- Confirm production readiness

**Agent:**
```bash
Task agent:
  subagent_type: rust-backend
  prompt: "Review the MIDI parser module and its test suite for quality,
           completeness, and adherence to Rust best practices.

           Module: shared/rust/src/core/midi/parser.rs

           Context:
           - 921-line MIDI file parser (binary format)
           - 65+ comprehensive tests
           - Test fixtures for valid/invalid MIDI files
           - Coverage: 82%+ (measured)

           Review Scope:
           1. Parser Implementation Quality:
              - Binary parsing correctness
              - Error handling robustness
              - Memory safety (no unsafe, no unwrap)
              - MIDI 1.0 spec compliance

           2. Test Quality:
              - Coverage of all parsing functions
              - Valid MIDI file tests (formats 0/1/2)
              - Error handling tests (truncation, corruption)
              - Edge case coverage (VLQ, running status)
              - Security tests (overflow, malicious input)

           3. Code Quality:
              - Security concerns (buffer overflow, integer overflow)
              - Performance issues (allocation patterns)
              - Maintainability (test organization)
              - Trusty Module compliance (80%+ coverage, no unwrap)

           Expected Output:
           - Overall quality score (0-100)
           - Coverage assessment and gaps
           - List of missing tests or improvements
           - Security assessment
           - Recommendation: PASS/NEEDS_WORK"
```

**Expected Review:**
```
Overall Quality Score: 88/100

Parser Implementation: 90/100
  ✅ Correct MIDI parsing
  ✅ Robust error handling
  ✅ No unsafe code
  ⚠️ Some allocations could be optimized

Test Quality: 92/100
  ✅ Comprehensive coverage (65+ tests)
  ✅ All formats tested
  ✅ Security tests present
  ⚠️ Could add more meta event tests

Coverage: 82.3% (PASS - exceeds 80% requirement)

Security: Excellent
  ✅ No buffer overflows
  ✅ Integer overflow handling
  ✅ Malicious input rejection

Recommendation: PASS - Production ready
```

**Deliverable:** Expert review report with score and recommendations

---

### Step 10: Commit Changes (10 minutes)

**Objectives:**
- Create semantic commit with /git-commit-smart
- Document all changes
- Update TEST-COVERAGE-PLAN.md

**Commands:**
```bash
# Stage changes
git add shared/rust/src/core/midi/parser.rs
git add shared/rust/tests/fixtures/midi/

# Generate commit message
/git-commit-smart:commit-smart
```

**Expected Commit Message:**
```
test(parser): add comprehensive test suite for MIDI parser module

Implements Phase 1.3 of test coverage initiative with 65+ tests
covering MIDI file parsing, error handling, and security.

Test Categories:
- Valid MIDI parsing: 15 tests (formats 0/1/2, all event types)
- Header parsing: 8 tests (valid/invalid headers)
- Track parsing: 10 tests (valid/invalid tracks)
- Event parsing: 12 tests (all MIDI events + running status)
- Meta event parsing: 8 tests (tempo, time sig, etc.)
- VLQ parsing: 5 tests (1-4 byte quantities)
- Error handling: 8 tests (truncation, corruption)
- Edge cases: 6 tests (boundaries, large files)
- Security: 5 tests (overflow, malicious input)

Test Fixtures:
- 11 MIDI files (5 valid, 6 invalid) in tests/fixtures/midi/

Coverage: 82.3% line coverage (exceeds 80% requirement)
Quality: 88/100 score from rust-backend agent review
Status: Production-ready

All 65+ tests passing.
```

**Update Documentation:**
```bash
# Update TEST-COVERAGE-PLAN.md
Edit TEST-COVERAGE-PLAN.md:
  - Mark Phase 1.3 as complete
  - Update coverage metrics
  - Add commit reference
```

**Deliverable:** Git commit with all changes

---

## Success Criteria

### Must Have (Required)

- ✅ **80%+ line coverage** (Trusty Module requirement)
- ✅ **All tests passing** (65+ tests, 0 failures)
- ✅ **All formats tested** (Format 0, 1, 2)
- ✅ **All event types tested** (note, CC, program, meta, sysex)
- ✅ **Error handling complete** (truncation, corruption, invalid data)
- ✅ **Security tests present** (overflow, malicious input)
- ✅ **No unwrap/expect** in parser code
- ✅ **Test fixtures created** (11 MIDI files)
- ✅ **Expert review PASS** (rust-backend agent)

### Nice to Have (Optional)

- ⭐ 90%+ line coverage
- ⭐ 80%+ branch coverage
- ⭐ Round-trip tests (parse → serialize → parse)
- ⭐ Performance benchmarks
- ⭐ Fuzzing tests (optional, advanced)

---

## Risk Assessment

### High Risk Areas

**1. Test Fixture Creation (30 min estimated)**
- **Risk:** Complex to create valid MIDI files by hand
- **Mitigation:** Use existing MIDI files or generate programmatically
- **Fallback:** Use byte arrays with documented structure

**2. VLQ Parsing Tests (Critical)**
- **Risk:** Many bugs hide in variable-length quantity parsing
- **Mitigation:** Thorough test coverage (5+ tests)
- **Note:** This is the #1 source of MIDI parser bugs

**3. Coverage Target (80%+)**
- **Risk:** Binary parsers have many branches
- **Mitigation:** Focus on main parsing paths, document unreachable branches
- **Fallback:** 75% acceptable if remaining code is unreachable

**4. Running Status Tests**
- **Risk:** Complex MIDI optimization, easy to get wrong
- **Mitigation:** Test with real MIDI files using running status
- **Reference:** MIDI 1.0 spec section on running status

### Medium Risk Areas

**1. Meta Event Coverage**
- **Risk:** Many meta event types (20+)
- **Mitigation:** Test most common (tempo, time sig, EOT)
- **Note:** Some meta events are rare (SMPTE offset)

**2. Format 2 Testing**
- **Risk:** Format 2 is rarely used in practice
- **Mitigation:** Minimal test (basic parse verification)
- **Note:** Low priority for real-world coverage

### Low Risk Areas

**1. Header Parsing**
- **Risk:** Simple, well-defined structure
- **Confidence:** High (tested in Phase 1.1 types)

**2. Error Type Usage**
- **Risk:** MidiParseError already tested (Phase 1.2)
- **Confidence:** High (31 error tests already passing)

---

## Tool Usage Matrix

| Step | Primary Tool | Secondary Tools | MCP Servers | Agents |
|------|-------------|-----------------|-------------|---------|
| 1. Analyze | Read, Grep | - | rust, filesystem | pattern-recognition-specialist |
| 2. Fixtures | filesystem MCP | web-search | filesystem, web-search | best-practices-researcher |
| 3. Generate | /unit-test-generator | - | rust | - |
| 4. Valid Tests | Edit | - | rust | - |
| 5. MIDI Tests | Edit | - | rust | midi-hardware |
| 6. Security | Edit | - | rust | security-sentinel |
| 7. Run Tests | Bash (cargo test) | - | rust | - |
| 8. Coverage | /test-coverage-analyzer | Bash (tarpaulin) | rust | - |
| 9. Review | Task (agent) | - | - | rust-backend |
| 10. Commit | /git-commit-smart | git add | git | - |

---

## Tool Selection Guidelines

### When to Use Each Tool

**Slash Commands:**
- `/unit-test-generator` - Initial test generation (Step 3)
- `/test-coverage-analyzer` - Coverage measurement (Step 8)
- `/git-commit-smart` - Semantic commits (Step 10)

**Agents:**
- `pattern-recognition-specialist` - Code analysis (Step 1)
- `best-practices-researcher` - Research MIDI testing (Step 2)
- `midi-hardware` - MIDI compliance review (Step 5)
- `security-sentinel` - Security audit (Step 6)
- `rust-backend` - Final quality review (Step 9)

**MCP Servers:**
- `rust` - All cargo operations (tests, coverage, build)
- `filesystem` - Test fixture creation (Step 2)
- `web-search` - Research MIDI specs and samples (Step 2)
- `git` - Version control operations (Step 10)

**Direct Tools:**
- `Read` - View parser source code
- `Edit` - Modify test code
- `Bash` - Run cargo commands
- `Grep` - Search for patterns

---

## Dependencies

### Prerequisites

- ✅ Phase 1.1 complete (types.rs - 50 tests, 85% coverage)
- ✅ Phase 1.2 complete (error.rs - 31 tests, 100% coverage)
- ✅ Rust toolchain installed (cargo, rustfmt, clippy)
- ✅ cargo-tarpaulin installed (for coverage)

### External Resources

**MIDI Specification:**
- MIDI 1.0 Detailed Specification (official)
- Standard MIDI File (SMF) Format Specification
- Available at: https://www.midi.org/specifications

**Reference Implementations:**
- `midly` crate (excellent Rust MIDI parser)
- `rimd` crate (alternative implementation)
- GitHub: search for "rust midi parser"

**Test MIDI Files:**
- Public domain MIDI files
- Minimal MIDI examples
- Corrupted MIDI samples (for error testing)

---

## Estimated Effort

### Time Breakdown

| Phase | Optimistic | Realistic | Pessimistic |
|-------|-----------|-----------|-------------|
| Analysis | 15 min | 20 min | 30 min |
| Fixtures | 20 min | 30 min | 60 min |
| Generation | 15 min | 20 min | 30 min |
| Valid Tests | 20 min | 30 min | 45 min |
| MIDI Tests | 15 min | 20 min | 30 min |
| Security | 10 min | 15 min | 20 min |
| Run Tests | 5 min | 10 min | 20 min |
| Coverage | 10 min | 15 min | 25 min |
| Review | 5 min | 10 min | 15 min |
| Commit | 5 min | 10 min | 15 min |
| **TOTAL** | **120 min** | **180 min** | **290 min** |

**Planned Time:** 180 minutes (3 hours)
**Buffer:** +110 minutes for unexpected issues
**Maximum:** 290 minutes (4.8 hours)

### Complexity Factors

**High Complexity:**
- Binary parsing (many edge cases)
- Test fixture creation (manual MIDI construction)
- VLQ parsing (subtle bugs)
- Running status (MIDI optimization)

**Medium Complexity:**
- Event type coverage (12 types)
- Meta event parsing (8 types)
- Error handling (30+ sites)

**Low Complexity:**
- Header parsing (simple structure)
- Test execution
- Commit creation

---

## Comparison to Phase 1.2

| Aspect | Phase 1.2 (error.rs) | Phase 1.3 (parser.rs) | Ratio |
|--------|---------------------|---------------------|-------|
| **Lines of Code** | 31 lines | 921 lines | 30x larger |
| **Tests Planned** | 31 tests | 65+ tests | 2x more |
| **Time Estimate** | 60 min | 180 min | 3x longer |
| **Complexity** | Low (enum + derive) | High (binary parser) | Much higher |
| **Risk** | Low | Medium-High | Higher risk |
| **Coverage Target** | 100% functional | 80%+ line | Lower % but more code |

**Key Differences:**
- Phase 1.2: Declarative code (derive macros) - easy to test exhaustively
- Phase 1.3: Imperative code (binary parsing) - requires careful test design
- Phase 1.2: Zero user-written executable code
- Phase 1.3: ~400-500 lines of complex parsing logic

---

## Next Phase Preview

**Phase 1.4: MIDI Validator Module** (`validator.rs`)
- Estimated: 90 minutes (1.5 hours)
- Complexity: Medium
- Lines: ~200 lines
- Tests: 30-40 tests
- Dependencies: types.rs (Phase 1.1)

**Phase 1.5: MIDI Loader Module** (`loader.rs`)
- Estimated: 60 minutes (1 hour)
- Complexity: Low-Medium
- Lines: ~150 lines
- Tests: 20-25 tests
- Dependencies: parser.rs (Phase 1.3), error.rs (Phase 1.2)

**Phase 1 Completion:**
After Phase 1.5, shared library core modules will be 100% tested.

---

## References

### MIDI Specifications
- MIDI 1.0 Detailed Specification (1996)
- Standard MIDI File Format 1.0 (1988)
- https://www.midi.org/specifications

### Rust MIDI Libraries
- `midly` - Fast, allocation-free MIDI parser
- `rimd` - MIDI file reader/writer
- `midir` - Cross-platform MIDI I/O

### Testing Resources
- Rust Testing Guide: https://doc.rust-lang.org/book/ch11-00-testing.html
- cargo-tarpaulin: https://github.com/xd009642/tarpaulin
- Test fixture patterns: https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html

---

## Appendix: MIDI Format Quick Reference

### MIDI File Structure
```
MIDI File:
  Header Chunk (MThd)
    - Magic: "MThd" (4 bytes)
    - Size: 6 (4 bytes)
    - Format: 0/1/2 (2 bytes)
    - Tracks: count (2 bytes)
    - Division: TPQN (2 bytes)

  Track Chunk(s) (MTrk)
    - Magic: "MTrk" (4 bytes)
    - Size: bytes (4 bytes)
    - Events:
        - Delta time (VLQ)
        - Event data (varies)
```

### Variable-Length Quantity (VLQ)
```
Encoding: 7 bits per byte, MSB = continuation
Examples:
  0x00       = 0
  0x7F       = 127
  0x81 0x00  = 128
  0xFF 0x7F  = 16383
  0x81 0x80 0x00 = 16384
```

### Event Types
```
MIDI Events (0x80-0xEF):
  0x80-0x8F: Note Off
  0x90-0x9F: Note On
  0xA0-0xAF: Polyphonic Aftertouch
  0xB0-0xBF: Control Change
  0xC0-0xCF: Program Change
  0xD0-0xDF: Channel Aftertouch
  0xE0-0xEF: Pitch Bend

Meta Events (0xFF):
  0xFF 0x00: Sequence Number
  0xFF 0x01-07: Text Events
  0xFF 0x2F: End of Track (REQUIRED)
  0xFF 0x51: Set Tempo
  0xFF 0x58: Time Signature
  0xFF 0x59: Key Signature

SysEx Events:
  0xF0: System Exclusive
  0xF7: SysEx continuation/escape
```

---

**End of Phase 1.3 Plan**

**Total Pages:** 24
**Word Count:** ~8,500 words
**Last Updated:** 2025-10-27
