# Phase 1.6 Execution Plan: Auto-Tagger Module

**Module:** `shared/rust/src/core/analysis/auto_tagger.rs`
**Target Coverage:** 85%+ (business logic complexity)
**Estimated Time:** 50-60 minutes
**Complexity:** MEDIUM-HIGH (rule-based logic, pattern matching)

---

## 1. Module Analysis

**File Location:** `shared/rust/src/core/analysis/auto_tagger.rs`

**Purpose:** Automatically generates tags/labels for MIDI files based on musical characteristics (genre, mood, style, instrumentation).

**Expected Characteristics:**
- **Lines of Code:** ~250-350
- **Functions:** 8-12 (tag generation, rule matching, confidence scoring, etc.)
- **Algorithms:** Rule-based classification, pattern matching, heuristic scoring
- **Complexity:** Medium-High (business logic, multiple classification rules)
- **Edge Cases:** Empty files, ambiguous characteristics, missing data

**Existing Tests:** Likely 2-3 basic tests (~10-15% coverage)

---

## 2. Tool Integration Matrix

### MCP Servers (3 servers)
| Server | Purpose | Usage |
|--------|---------|-------|
| **filesystem** | Read module file | `Read auto_tagger.rs` |
| **rust** | Compile/test | `cargo test auto_tagger` |
| **bash** | Run tests, check coverage | `cargo tarpaulin` |

### Claude Agents (5 agents)
| Agent | Phase | Purpose |
|-------|-------|---------|
| **pattern-recognition-specialist** | Analysis | Identify all tagging rules, patterns, test scenarios |
| **security-sentinel** | Security | Check for edge cases, null checks, array bounds |
| **rust-backend** | Expert Review | Final production readiness assessment |
| **code-simplicity-reviewer** | Optional | Suggest rule simplifications |
| **best-practices-researcher** | Optional | Research MIDI genre classification techniques |

### Slash Commands (2 commands)
| Command | Phase | Purpose |
|---------|-------|---------|
| `/test-coverage-analyzer:analyze-coverage` | Coverage | Measure line/branch coverage |
| `/git-commit-smart:commit-smart` | Commit | Generate conventional commit |

### Plugins/Skills (1 plugin)
| Plugin | Phase | Purpose |
|--------|-------|---------|
| **test-coverage-analyzer** | Coverage | Detailed coverage analysis |

---

## 3. Expected Algorithm Structure

### Auto-Tagging Pipeline

**Step 1: Extract Features**
```rust
fn extract_features(midi_file: &MidiFile) -> MusicFeatures
```
- BPM (from bpm_detector)
- Key (from key_detector)
- Note density
- Instrument program numbers
- Duration/length
- Dynamic range (velocity)
- Rhythmic complexity

**Step 2: Apply Tagging Rules**
```rust
fn apply_tagging_rules(features: &MusicFeatures) -> Vec<Tag>
```
- Genre rules (BPM + instruments → genre)
- Mood rules (key + dynamics → mood)
- Style rules (rhythmic patterns → style)
- Instrumentation rules (programs → instruments)

**Step 3: Score and Filter Tags**
```rust
fn score_tags(tags: Vec<Tag>) -> Vec<ScoredTag>
```
- Calculate confidence per tag
- Filter low-confidence tags
- Rank by confidence

**Step 4: Return Tagged Result**
```rust
pub fn auto_tag(midi_file: &MidiFile) -> TaggingResult
```
- Primary tags (high confidence)
- Secondary tags (medium confidence)
- Metadata (features used)

### Expected Tag Categories

1. **Genre Tags:**
   - Rock, Pop, Jazz, Classical, Electronic, Hip-Hop
   - BPM ranges, instrument combinations

2. **Mood Tags:**
   - Happy, Sad, Energetic, Calm, Aggressive, Peaceful
   - Key (major/minor), dynamics, tempo

3. **Style Tags:**
   - Ballad, Uptempo, Groove, Ambient, Orchestral
   - Rhythmic complexity, instrument layering

4. **Instrumentation Tags:**
   - Piano, Guitar, Drums, Strings, Synth, Bass
   - MIDI program numbers

### Expected Functions
1. `auto_tag(&MidiFile) -> TaggingResult` (public API)
2. `extract_features(&MidiFile) -> MusicFeatures`
3. `apply_genre_rules(&MusicFeatures) -> Vec<Tag>`
4. `apply_mood_rules(&MusicFeatures) -> Vec<Tag>`
5. `apply_style_rules(&MusicFeatures) -> Vec<Tag>`
6. `apply_instrumentation_rules(&MusicFeatures) -> Vec<Tag>`
7. `score_tag(&Tag, &MusicFeatures) -> f64`
8. `filter_tags(Vec<ScoredTag>, threshold: f64) -> Vec<Tag>`

---

## 4. Test Strategy (Estimated 55-65 tests)

### Category 1: Feature Extraction (10 tests)
- Empty MIDI → default features
- Simple melody → basic features
- Full orchestral → complex features
- Extract BPM (integration with bpm_detector)
- Extract key (integration with key_detector)
- Extract instrument programs
- Calculate note density
- Calculate dynamic range (velocity)
- Calculate duration
- Edge case: percussion-only file

### Category 2: Genre Classification (12 tests)
- Fast BPM (140+) + rock instruments → Rock
- Slow BPM (60-80) + piano → Ballad/Classical
- Medium BPM (100-120) + 4/4 time → Pop
- Fast BPM (120+) + synths → Electronic
- Complex rhythms + brass → Jazz
- Piano solo → Classical/Piano
- Ambiguous features → multiple genre tags
- No clear genre → default tags
- Genre confidence scoring
- Genre boundary cases (BPM thresholds)
- Multi-genre detection
- Genre exclusion rules

### Category 3: Mood Classification (10 tests)
- Major key + high velocity → Happy/Energetic
- Minor key + low velocity → Sad/Calm
- Major key + low velocity → Peaceful
- Minor key + high velocity → Aggressive/Intense
- Neutral (no strong indicators) → Neutral tag
- Ambiguous mood → multiple tags
- Mood confidence scoring
- Mood combinations (happy + energetic)
- Edge cases (chromatic = ambiguous mood)
- Mood from tempo (fast = energetic)

### Category 4: Style Classification (8 tests)
- Slow + piano → Ballad
- Fast + driving rhythm → Uptempo
- Low note density + long notes → Ambient
- High note density + strings → Orchestral
- Rhythmic patterns → Groove
- Syncopation detection
- Style confidence scoring
- Multiple style tags

### Category 5: Instrumentation Detection (9 tests)
- MIDI program 0 → Piano
- Program 24-31 → Guitar
- Program 40-47 → Strings
- Program 80-87 → Synth
- Channel 9/10 → Drums/Percussion
- Multiple instruments → ensemble tags
- Dominant instrument detection
- Instrumentation confidence
- Unknown program numbers

### Category 6: Tag Scoring & Filtering (8 tests)
- High confidence tag → included
- Low confidence tag → filtered out
- Threshold tuning (0.5, 0.7, 0.9)
- Multiple tags with similar confidence
- Tag ranking by confidence
- Minimum tag count (always return at least 1 tag)
- Maximum tag count (limit to top N)
- Score normalization

### Category 7: Edge Cases (5 tests)
- Empty MIDI file → default tags
- Single note → minimal tags
- Percussion only → drumming/percussion tags
- Very short file (< 1 second) → low confidence
- Very long file (> 10 minutes) → normal confidence

### Category 8: Integration & Metadata (8 tests)
- Full tagging pipeline
- Metadata includes features used
- Metadata includes all tag candidates (not just top)
- Primary vs. secondary tag distinction
- Tag deduplication
- Realistic MIDI file (multi-track, multi-instrument)
- Tag consistency across similar files
- Regression test (known file → expected tags)

**Total Estimated Tests:** 70 tests

---

## 5. Security & Stability Checks

### Priority 1 (Critical)
1. **Null/Empty Checks:**
   - Empty MIDI files
   - Missing BPM/key data
   - No instruments detected

2. **Array Bounds:**
   - MIDI program number range (0-127)
   - Channel number range (0-15)
   - Tag list indexing

3. **Confidence Score Validity:**
   - Scores in [0.0, 1.0] range
   - No NaN or infinity values
   - Proper normalization

### Priority 2 (Important)
4. **Rule Coverage:**
   - All BPM ranges covered
   - All program numbers mapped
   - No undefined behavior for edge cases

5. **Tag Consistency:**
   - No contradictory tags (happy + sad)
   - Logical tag combinations
   - Proper tag filtering

---

## 6. Expected Coverage Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| **Line Coverage** | 85%+ | Business logic complexity |
| **Branch Coverage** | 80%+ | Multiple conditional rules |
| **Function Coverage** | 100% | All functions must be tested |

---

## 7. Workflow Steps (12 steps)

### Step 1: Read and Analyze Module (5 min)
```bash
Read shared/rust/src/core/analysis/auto_tagger.rs
```
- Understand tagging rules
- Identify all functions
- Note existing tests

### Step 2: Launch Pattern Recognition Analysis (8 min)
```bash
Task(pattern-recognition-specialist) → analyze auto_tagger.rs
```
- Identify all tagging rules
- List edge cases
- Estimate test count

### Step 3: Create Test Plan Document (5 min)
- Document test categories (8 categories)
- Estimate test count (70 tests)
- Define coverage targets (85%+)

### Step 4: Generate Helper Functions (8 min)
```rust
// Helper to create test MIDI with specific instruments
fn create_test_midi_with_instruments(programs: Vec<u8>) -> MidiFile

// Helper to create MIDI with specific BPM and key
fn create_test_midi_with_features(bpm: f64, key: Key, ...) -> MidiFile

// Assert tag present
fn assert_tag_present(result: &TaggingResult, tag: &str)

// Assert tag confidence
fn assert_tag_confidence(tag: &ScoredTag, min: f64, max: f64)
```

### Step 5-12: Generate Tests by Category (30-35 min)
- Category 1: Feature extraction (10 tests)
- Category 2: Genre classification (12 tests)
- Category 3: Mood classification (10 tests)
- Category 4: Style classification (8 tests)
- Category 5: Instrumentation (9 tests)
- Category 6: Tag scoring (8 tests)
- Category 7: Edge cases (5 tests)
- Category 8: Integration (8 tests)

### Step 13: Run Tests and Fix Failures (5 min)
```bash
cargo test --package midi-library-shared --lib core::analysis::auto_tagger
```

### Step 14: Measure Coverage (3 min)
```bash
/test-coverage-analyzer:analyze-coverage
```

### Step 15: Security Review (5 min)
```bash
Task(security-sentinel) → review auto_tagger.rs
```

### Step 16: Apply Fixes (3 min)
- Add null checks
- Clamp confidence scores
- Validate array bounds

### Step 17: Expert Rust Review (5 min)
```bash
Task(rust-backend) → review auto_tagger.rs
```

### Step 18: Commit Changes (2 min)
```bash
/git-commit-smart:commit-smart
```

---

## 8. Expected Challenges

1. **Tagging Rule Complexity:**
   - Many rules to test
   - Rule interactions
   - Edge case coverage

2. **Ambiguous Cases:**
   - Files that fit multiple genres
   - Neutral mood files
   - Mixed instrumentation

3. **Confidence Calibration:**
   - Setting appropriate thresholds
   - Balancing precision vs. recall
   - Avoiding over-tagging or under-tagging

4. **Test Data:**
   - Creating representative MIDI files
   - Covering all genre/mood/style combinations
   - Realistic instrument combinations

---

## 9. Success Criteria

- ✅ All 70+ tests passing
- ✅ Coverage ≥ 85% (target: 88-92%)
- ✅ Security rating ≥ 8.5/10
- ✅ Expert review ≥ 9.0/10
- ✅ Zero unwrap/expect/panic in production code
- ✅ Comprehensive documentation
- ✅ Production-ready commit

---

## 10. Timeline

| Phase | Duration | Cumulative |
|-------|----------|------------|
| Analysis | 5 min | 5 min |
| Pattern recognition | 8 min | 13 min |
| Test planning | 5 min | 18 min |
| Helper functions | 8 min | 26 min |
| Test generation (8 categories) | 35 min | 61 min |
| Run tests + fixes | 5 min | 66 min |
| Coverage analysis | 3 min | 69 min |
| Security review + fixes | 8 min | 77 min |
| Expert review | 5 min | 82 min |
| Commit | 2 min | 84 min |

**Total Estimated Time:** 50-60 minutes (actual may vary)

---

## 11. References

**MIDI Tagging:**
- Genre classification from MIDI
- Mood detection algorithms
- Instrumentation mapping (MIDI program numbers to instruments)

**Machine Learning (if applicable):**
- Rule-based classification
- Confidence scoring
- Tag filtering strategies

**Similar Work:**
- Phase 1.4 (BPM detector) - good template for numerical algorithms
- Phase 1.5 (Key detector) - music theory integration
- Existing auto-tagging systems (music21, Essentia)
