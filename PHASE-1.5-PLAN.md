# Phase 1.5 Execution Plan: Key Detector Module

**Module:** `shared/rust/src/core/analysis/key_detector.rs`
**Target Coverage:** 90%+ (algorithmic complexity requires high coverage)
**Estimated Time:** 60-75 minutes
**Complexity:** HIGH (music theory algorithms, statistical analysis)

---

## 1. Module Analysis

**File Location:** `shared/rust/src/core/analysis/key_detector.rs`

**Purpose:** Detects musical key (e.g., C major, A minor) from MIDI files using statistical analysis of note frequencies and music theory heuristics.

**Expected Characteristics:**
- **Lines of Code:** ~300-400 (similar to BPM detector)
- **Functions:** 6-8 (note extraction, key scoring, profile matching, etc.)
- **Algorithms:** Krumhansl-Schmuckler key-finding algorithm, correlation-based scoring
- **Complexity:** High (statistical analysis, music theory, pitch class profiles)
- **Edge Cases:** Empty files, single notes, atonal music, chromatic passages

**Existing Tests:** Likely 2-3 basic tests (~10-15% coverage)

---

## 2. Tool Integration Matrix

### MCP Servers (4 servers)
| Server | Purpose | Usage |
|--------|---------|-------|
| **postgres** | N/A (no DB for this module) | Not needed |
| **filesystem** | Read module file | `Read key_detector.rs` |
| **rust** | Compile/test | `cargo test key_detector` |
| **bash** | Run tests, check coverage | `cargo tarpaulin` |

### Claude Agents (6 agents)
| Agent | Phase | Purpose |
|-------|-------|---------|
| **pattern-recognition-specialist** | Analysis | Identify all code paths, edge cases, test scenarios |
| **security-sentinel** | Security | Check for numerical stability, division by zero, overflow |
| **rust-backend** | Expert Review | Final production readiness assessment |
| **code-simplicity-reviewer** | Optional | Suggest simplifications after implementation |
| **performance-oracle** | Optional | Analyze algorithm efficiency |
| **best-practices-researcher** | Optional | Research Krumhansl-Schmuckler algorithm details |

### Slash Commands (2 commands)
| Command | Phase | Purpose |
|---------|-------|---------|
| `/test-coverage-analyzer:analyze-coverage` | Coverage | Measure line/branch coverage |
| `/git-commit-smart:commit-smart` | Commit | Generate conventional commit |

### Plugins/Skills (2 plugins)
| Plugin | Phase | Purpose |
|--------|-------|---------|
| **test-coverage-analyzer** | Coverage | Detailed coverage analysis |
| **unit-test-generator** | Optional | Suggest test patterns |

---

## 3. Expected Algorithm Structure

### Key Detection Algorithm (Krumhansl-Schmuckler)

**Step 1: Extract Note Events**
```rust
fn extract_notes(midi_file: &MidiFile) -> Vec<Note>
```
- Parse all NoteOn events across all tracks
- Ignore NoteOff, ControlChange, etc.
- Return Vec of note pitches (0-127)

**Step 2: Build Pitch Class Profile**
```rust
fn build_pitch_class_profile(notes: &[Note]) -> [f64; 12]
```
- Map MIDI notes to pitch classes (C=0, C#=1, ..., B=11)
- Count frequency of each pitch class
- Normalize to probabilities (sum to 1.0)

**Step 3: Correlate with Key Profiles**
```rust
fn correlate_with_profiles(profile: &[f64; 12], key_profiles: &KeyProfiles) -> Vec<KeyScore>
```
- Compare against 24 key profiles (12 major, 12 minor)
- Use Pearson correlation coefficient
- Return sorted list of (key, correlation_score)

**Step 4: Select Best Key**
```rust
pub fn detect_key(midi_file: &MidiFile) -> KeyDetectionResult
```
- Get highest correlation score
- Apply confidence threshold
- Return key with metadata

### Expected Functions
1. `detect_key(&MidiFile) -> KeyDetectionResult` (public API)
2. `extract_notes(&MidiFile) -> Vec<Note>`
3. `build_pitch_class_profile(&[Note]) -> [f64; 12]`
4. `correlate_with_profiles(&[f64; 12], &KeyProfiles) -> Vec<KeyScore>`
5. `pearson_correlation(&[f64], &[f64]) -> f64`
6. `normalize_profile(&mut [f64; 12])`

### Key Profiles (Constants)
```rust
// Krumhansl-Schmuckler major key profile
const MAJOR_PROFILE: [f64; 12] = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88];

// Minor key profile
const MINOR_PROFILE: [f64; 12] = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17];
```

---

## 4. Test Strategy (Estimated 65-75 tests)

### Category 1: Note Extraction (8 tests)
- Empty MIDI file → empty vec
- Single note event → vec with 1 note
- Multiple tracks with notes → combined vec
- Non-note events filtered out
- Duplicate notes at same tick
- Notes across full MIDI range (0-127)
- Tracks with no note events
- Large file with 1000+ notes

### Category 2: Pitch Class Mapping (6 tests)
- MIDI note 60 (C4) → pitch class 0
- MIDI note 61 (C#4) → pitch class 1
- Note 127 (G9) → pitch class 7
- Full chromatic scale (12 notes) → all pitch classes
- Octave equivalence (C3, C4, C5 all → 0)
- Edge cases (note 0 → 0, note 11 → 11)

### Category 3: Pitch Class Profile (9 tests)
- Empty notes → all zeros or uniform?
- Single note → profile with 1.0 at that pitch class
- All C notes → [1.0, 0, 0, ..., 0]
- C major scale notes → higher weights for scale degrees
- Chromatic scale → uniform distribution [1/12, 1/12, ...]
- Normalization check (sum = 1.0)
- Profile with various note counts
- Weighted by duration vs. count
- Large profile (1000+ notes)

### Category 4: Pearson Correlation (8 tests)
- Perfect correlation (same profile) → 1.0
- Perfect negative correlation → -1.0
- No correlation (orthogonal) → 0.0
- C major scale vs. C major profile → high positive
- C major vs. C# major profile → lower correlation
- Empty profiles → 0.0 or error handling
- Single-element profiles
- Profiles with zero variance

### Category 5: Key Profile Matching (10 tests)
- C major scale notes → detect C major
- A minor scale notes → detect A minor
- All C notes → ambiguous (C major or A minor)
- Chromatic passage → low confidence
- C major + F# note → G major (dominant key)
- Parallel keys (C major vs. C minor)
- Relative keys (C major vs. A minor)
- Enharmonic equivalents (C# major vs. Db major)
- Empty file → default or error
- Single note → low confidence

### Category 6: Confidence Scoring (7 tests)
- High correlation (>0.9) → high confidence
- Medium correlation (0.5-0.9) → medium confidence
- Low correlation (<0.5) → low confidence
- Ambiguous keys (two keys with similar scores) → lower confidence
- Clear tonal center → high confidence
- Atonal/chromatic music → very low confidence
- Confidence clamping to [0.0, 1.0]

### Category 7: Edge Cases (8 tests)
- Empty MIDI file
- File with only percussion (channel 9/10)
- Single repeated note
- Two-note interval (ambiguous)
- Microtonal/non-standard tuning
- Very short file (< 10 notes)
- Very long file (10,000+ notes)
- All notes in one octave vs. spread across octaves

### Category 8: Musical Examples (9 tests)
- C major arpeggio → C major
- A natural minor scale → A minor
- A harmonic minor scale → A minor
- Blues scale in E → E minor or E major?
- Pentatonic scale → ambiguous
- Whole tone scale → very low confidence
- Diminished scale → ambiguous
- Modal music (Dorian, Mixolydian) → related major/minor
- Modulation (starts C major, ends G major)

### Category 9: Integration & Metadata (6 tests)
- Result includes top 3 key candidates
- Metadata contains pitch class profile
- Metadata contains note count
- Metadata contains key alternatives
- Method selection (statistical vs. heuristic)
- Full integration test with realistic MIDI

**Total Estimated Tests:** 71 tests

---

## 5. Security & Numerical Stability Checks

### Priority 1 (Critical)
1. **Division by Zero:**
   - Profile normalization when sum = 0
   - Pearson correlation with zero variance
   - Empty note lists

2. **Overflow/Underflow:**
   - Pitch class counting (use saturating add)
   - Correlation calculations with large values

3. **Floating Point Precision:**
   - Profile normalization (sum to 1.0)
   - Correlation calculations
   - Confidence score calculations

### Priority 2 (Important)
4. **Array Bounds:**
   - Pitch class indexing (0-11)
   - MIDI note range (0-127)
   - Key profile indexing (0-23)

5. **Mathematical Correctness:**
   - Pearson correlation formula
   - Profile normalization
   - Confidence calculation

---

## 6. Expected Coverage Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| **Line Coverage** | 90%+ | High algorithmic complexity |
| **Branch Coverage** | 85%+ | Multiple conditional paths |
| **Function Coverage** | 100% | All functions must be tested |

---

## 7. Workflow Steps (12 steps)

### Step 1: Read and Analyze Module (5 min)
```bash
Read shared/rust/src/core/analysis/key_detector.rs
```
- Understand current implementation
- Identify all functions
- Note existing tests

### Step 2: Launch Pattern Recognition Analysis (10 min)
```bash
Task(pattern-recognition-specialist) → analyze key_detector.rs
```
- Identify all code paths
- List edge cases
- Estimate test count

### Step 3: Create Test Plan Document (5 min)
- Document test categories (9 categories)
- Estimate test count (71 tests)
- Define coverage targets (90%+)

### Step 4: Generate Helper Functions (10 min)
```rust
// Helper to create test MIDI with specific notes
fn create_test_midi_with_notes(notes: Vec<u8>) -> MidiFile

// Helper to create scale (major, minor, chromatic, etc.)
fn create_scale(root: u8, scale_type: ScaleType) -> Vec<u8>

// Assert key matches with tolerance
fn assert_key_eq(result: &Key, expected: &Key)

// Assert confidence in range
fn assert_confidence_range(conf: f64, min: f64, max: f64)
```

### Step 5-12: Generate Tests by Category (35-40 min)
- Category 1: Note extraction (8 tests)
- Category 2: Pitch class mapping (6 tests)
- Category 3: Profile building (9 tests)
- Category 4: Pearson correlation (8 tests)
- Category 5: Key matching (10 tests)
- Category 6: Confidence (7 tests)
- Category 7: Edge cases (8 tests)
- Category 8: Musical examples (9 tests)
- Category 9: Integration (6 tests)

### Step 13: Run Tests and Fix Failures (5 min)
```bash
cargo test --package midi-library-shared --lib core::analysis::key_detector
```

### Step 14: Measure Coverage (3 min)
```bash
/test-coverage-analyzer:analyze-coverage
```

### Step 15: Security Review (8 min)
```bash
Task(security-sentinel) → review key_detector.rs
```
- Check division by zero
- Verify numerical stability
- Validate array bounds

### Step 16: Apply Security Fixes (3 min)
- Add explicit zero checks
- Use saturating arithmetic
- Clamp confidence scores

### Step 17: Expert Rust Review (5 min)
```bash
Task(rust-backend) → review key_detector.rs
```

### Step 18: Commit Changes (2 min)
```bash
/git-commit-smart:commit-smart
```

---

## 8. Expected Challenges

1. **Music Theory Complexity:**
   - Krumhansl-Schmuckler algorithm details
   - Key profile constants (may need research)
   - Handling modal music

2. **Ambiguous Cases:**
   - Relative keys (C major vs. A minor)
   - Parallel keys (C major vs. C minor)
   - Atonal music

3. **Statistical Calculations:**
   - Pearson correlation implementation
   - Profile normalization edge cases
   - Confidence score calibration

4. **Test Data Generation:**
   - Creating realistic musical examples
   - Generating scales and arpeggios
   - Simulating modulations

---

## 9. Success Criteria

- ✅ All 71+ tests passing
- ✅ Coverage ≥ 90% (target: 92-95%)
- ✅ Security rating ≥ 9.0/10
- ✅ Expert review ≥ 9.0/10
- ✅ Zero unwrap/expect/panic in production code
- ✅ Comprehensive documentation
- ✅ Production-ready commit

---

## 10. Timeline

| Phase | Duration | Cumulative |
|-------|----------|------------|
| Analysis | 5 min | 5 min |
| Pattern recognition | 10 min | 15 min |
| Test planning | 5 min | 20 min |
| Helper functions | 10 min | 30 min |
| Test generation (9 categories) | 40 min | 70 min |
| Run tests + fixes | 5 min | 75 min |
| Coverage analysis | 3 min | 78 min |
| Security review + fixes | 11 min | 89 min |
| Expert review | 5 min | 94 min |
| Commit | 2 min | 96 min |

**Total Estimated Time:** 60-75 minutes (actual may vary)

---

## 11. References

**Music Theory:**
- Krumhansl-Schmuckler key-finding algorithm
- Pitch class theory
- Major and minor key profiles

**Statistical Methods:**
- Pearson correlation coefficient
- Profile normalization techniques

**Similar Work:**
- Phase 1.4 (BPM detector) - similar complexity, good template
- Existing key detection libraries (music21, librosa)
