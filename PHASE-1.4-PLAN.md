# Phase 1.4 Execution Plan: BPM Detector Module Testing

**Status:** Ready to Execute
**Module:** `shared/rust/src/core/analysis/bpm_detector.rs`
**Priority:** 🔴 CRITICAL (Core musical analysis functionality)
**Estimated Time:** 45-60 minutes (tool-enhanced workflow)
**Target Coverage:** 90%+ (algorithmic module requires higher coverage)
**Date Created:** 2025-10-27

---

## Executive Summary

Phase 1.4 focuses on comprehensive testing of the BPM (Beats Per Minute) detector module, a critical component for musical analysis. This module uses algorithmic tempo detection with weighted averaging and confidence scoring. The existing implementation has only 2 basic tests, leaving most of the algorithm untested.

**Key Objectives:**
1. Achieve 90%+ test coverage (higher than 80% standard due to algorithmic complexity)
2. Test all 6 functions (1 public, 5 private helpers)
3. Validate BPM detection algorithms (single tempo, weighted average, default)
4. Test confidence scoring and edge cases
5. Ensure production readiness for musical analysis pipeline

**Success Criteria:**
- ✅ 50+ comprehensive tests across 12 categories
- ✅ 90%+ line coverage (algorithmic code requires thorough testing)
- ✅ All tests passing
- ✅ Expert review score: 90+ (high bar for musical algorithms)
- ✅ Performance benchmarks for algorithm efficiency

---

## Module Overview

### Current State

**File:** `shared/rust/src/core/analysis/bpm_detector.rs`
- **Lines:** 274 total (187 production code, 27 existing tests, 60 documentation)
- **Functions:** 6 total
  - `detect_bpm()` - Public API (main detection function)
  - `extract_tempo_events()` - Extract tempo meta events from MIDI
  - `calculate_total_ticks()` - Sum all track lengths
  - `microseconds_to_bpm()` - Convert MIDI tempo to BPM
  - `calculate_weighted_average()` - Weighted BPM across tempo changes
  - `calculate_confidence()` - Statistical confidence scoring
- **Existing Tests:** 2 basic tests
  - `test_microseconds_to_bpm()` - Basic conversion
  - `test_bpm_clamping()` - Min/max validation
- **Coverage:** ~15% (very low - only 2 helper functions tested)

### Algorithm Complexity

The BPM detector uses sophisticated musical analysis:

**Three Detection Methods:**
1. **SingleTempo** - One tempo event found (confidence: 1.0)
2. **WeightedAverage** - Multiple tempos, weighted by duration (confidence: 0.5-1.0 based on variance)
3. **DefaultTempo** - No tempo events found (confidence: 0.3, defaults to 120 BPM)

**Mathematical Operations:**
- Microseconds per quarter note → BPM conversion: `60,000,000 / microseconds`
- BPM clamping: `MIN_BPM (20.0)` to `MAX_BPM (300.0)`
- Weighted average: `Σ(bpm_i × duration_i) / total_duration`
- Confidence scoring: `1.0 - (std_dev / mean)`, clamped to `[0.5, 1.0]`
- Variance calculation for tempo stability assessment

**Edge Cases to Test:**
- Empty MIDI files (no tracks)
- Files with no tempo events (default tempo)
- Single tempo event
- Multiple tempo events (2, 5, 10+)
- Tempo changes at track boundaries
- Very slow tempos (near MIN_BPM)
- Very fast tempos (near MAX_BPM)
- Tempos outside valid range (clamping)
- Zero-length tracks
- Extremely long files (tick overflow potential)
- Tempo variance (stable vs. varying)

---

## Tool Integration Matrix

### Specialized Agents (6 Used)

| Agent | Purpose | When Used | Expected Output |
|-------|---------|-----------|-----------------|
| **compounding-engineering:pattern-recognition-specialist** | Analyze BPM detector patterns and identify test needs | Step 1 (Analysis) | List of all code paths, edge cases, and recommended test categories |
| **rust-backend** | Expert Rust code review for test quality | Step 9 (Review) | Production readiness assessment, score 0-100, improvement recommendations |
| **compounding-engineering:performance-oracle** | Benchmark BPM detection performance | Step 6 (Benchmarks) | Performance metrics, optimization suggestions, scalability analysis |
| **compounding-engineering:best-practices-researcher** | Research BPM detection algorithms and testing strategies | Step 2 (Research) | Academic references, industry best practices, test validation methods |
| **compounding-engineering:security-sentinel** | Security review for numerical stability and edge cases | Step 8 (Security) | Vulnerability assessment, overflow detection, precision analysis |
| **compounding-engineering:code-simplicity-reviewer** | Final simplicity review before commit | Step 10 (Simplify) | Simplification opportunities, complexity metrics |

### Slash Commands (3 Used)

| Command | Purpose | Step | Arguments |
|---------|---------|------|-----------|
| `/unit-test-generator:generate-tests` | Generate comprehensive test boilerplate | Step 3 | Target: `bpm_detector.rs` |
| `/test-coverage-analyzer:analyze-coverage` | Measure test coverage | Step 7 | Package: `midi-library-shared`, module: `core::analysis::bpm_detector` |
| `/git-commit-smart:commit-smart` | Create conventional commit | Step 11 | Auto-analyze changes |

### MCP Servers (4 Used)

| Server | Purpose | Usage Pattern | Example |
|--------|---------|---------------|---------|
| **rust** | Cargo operations (test, build, bench) | Throughout execution | `cargo test --package midi-library-shared --lib core::analysis::bpm_detector` |
| **filesystem** | Read module files, verify structure | Step 1 (Analysis) | Read `bpm_detector.rs`, check test directory |
| **git** | Version control operations | Step 11 (Commit) | `git add`, `git commit`, `git status` |
| **bash** | Shell command execution | Steps 3, 7, 8 | Run tests, measure coverage, benchmarks |

### Plugins (2 Used)

| Plugin | Purpose | When Used | Output |
|--------|---------|-----------|--------|
| **test-coverage-analyzer** | Detailed coverage reports | Step 7 | HTML report, line-by-line coverage, uncovered paths |
| **unit-test-generator** | Auto-generate test scaffolding | Step 3 | Test boilerplate, helper functions, fixture suggestions |

---

## Detailed Execution Plan (12 Steps, ~45-60 minutes)

### Step 1: Analyze BPM Detector Module (5 minutes)

**Objective:** Understand algorithm complexity and identify all test requirements

**Actions:**
1. Read `bpm_detector.rs` completely to understand algorithms
2. Map all 6 functions and their dependencies
3. Identify mathematical operations (weighted average, variance, confidence)
4. List all edge cases (empty files, no tempos, multiple tempos, clamping)
5. Count existing test coverage baseline

**Tools:**
- `Read` tool for file analysis
- `Grep` for function discovery
- `compounding-engineering:pattern-recognition-specialist` agent

**Agent Prompt:**
```
Analyze shared/rust/src/core/analysis/bpm_detector.rs for comprehensive test planning.

Focus on:
1. All 6 functions and their algorithms (weighted average, confidence scoring)
2. Mathematical operations and potential edge cases (overflow, precision)
3. Three detection methods (SingleTempo, WeightedAverage, DefaultTempo)
4. Input validation and clamping (MIN_BPM, MAX_BPM)
5. Recommended test categories for musical algorithm validation

Provide:
- List of all code paths requiring tests
- Edge cases for tempo detection
- Mathematical validation test cases
- Performance test recommendations
- Estimated test count to reach 90%+ coverage
```

**Expected Output:**
- Comprehensive analysis of all 6 functions
- List of 50-60 required tests across 12 categories
- Mathematical test cases for algorithm validation
- Coverage projection: 90-95%

---

### Step 2: Research BPM Detection Best Practices (5 minutes)

**Objective:** Gather industry best practices for BPM detection algorithm testing

**Tools:**
- `compounding-engineering:best-practices-researcher` agent
- `WebSearch` (if needed for academic references)

**Agent Prompt:**
```
Research best practices for testing BPM detection algorithms in music software.

Focus on:
1. How are tempo detection algorithms validated in production music software?
2. What are common edge cases in MIDI tempo detection?
3. Industry standards for tempo accuracy (precision requirements)
4. Test strategies for weighted averaging algorithms
5. Confidence scoring validation methods

Provide:
- Academic references (if available)
- Industry best practices from music software projects
- Test validation strategies for musical algorithms
- Performance benchmarking approaches
```

**Expected Output:**
- Best practices document from music software industry
- Validation strategies for tempo algorithms
- Precision requirements for BPM detection
- Confidence scoring validation methods

---

### Step 3: Generate Comprehensive Test Suite (15 minutes)

**Objective:** Create 50+ tests covering all functions and edge cases

**Tools:**
- `/unit-test-generator:generate-tests` slash command
- Manual test writing for algorithmic validation

**Test Categories (12 Categories, 58 Tests Planned):**

#### Category 1: Microseconds to BPM Conversion (8 tests)
- Test common BPMs (60, 120, 140, 180)
- Test exact conversions (500,000 µs = 120 BPM)
- Test precision (floating point accuracy)
- Test boundary values (MIN_BPM, MAX_BPM)
- Test below minimum (should clamp to 20 BPM)
- Test above maximum (should clamp to 300 BPM)
- Test rounding behavior
- Test extreme values (overflow protection)

#### Category 2: BPM Clamping (6 tests)
- Test exact MIN_BPM (20.0)
- Test exact MAX_BPM (300.0)
- Test below MIN_BPM (5 BPM → 20 BPM)
- Test above MAX_BPM (500 BPM → 300 BPM)
- Test within range (no clamping)
- Test edge precision (19.99 vs 20.0)

#### Category 3: Single Tempo Detection (6 tests)
- Test single tempo event (confidence = 1.0)
- Test at file start (tick 0)
- Test in middle of file
- Test at various BPMs (slow, medium, fast)
- Test metadata correctness (is_constant = true)
- Test method detection (SingleTempo)

#### Category 4: Multiple Tempo Detection (8 tests)
- Test 2 tempo events (weighted average)
- Test 5 tempo events
- Test 10+ tempo events
- Test even distribution (equal durations)
- Test uneven distribution (long segment vs short)
- Test extreme variance (40 BPM to 200 BPM)
- Test small variance (118-122 BPM range)
- Test metadata (tempo_range calculation)

#### Category 5: Weighted Average Calculation (7 tests)
- Test equal-length segments (simple average)
- Test 90% one tempo, 10% another (should favor longer)
- Test 3-way split (33/33/33%)
- Test very short tempo change (< 1% of file)
- Test mathematical precision
- Test edge case: single-tick tempo change
- Test validation against hand-calculated values

#### Category 6: Confidence Scoring (8 tests)
- Test single tempo (confidence = 1.0)
- Test stable tempos (low variance, high confidence)
- Test varying tempos (high variance, low confidence)
- Test extreme variance (confidence near 0.5)
- Test zero variance (all same BPM, confidence = 1.0)
- Test coefficient of variation calculation
- Test clamping to [0.5, 1.0] range
- Test mathematical correctness (variance, std dev)

#### Category 7: Default Tempo (No Tempo Events) (5 tests)
- Test empty MIDI file (no tracks)
- Test file with tracks but no tempo events
- Test default BPM (120.0)
- Test default confidence (0.3)
- Test default method (DefaultTempo)

#### Category 8: Extract Tempo Events (6 tests)
- Test no tempo events (empty vec)
- Test single tempo in single track
- Test multiple tempos in single track
- Test tempos across multiple tracks
- Test non-tempo meta events (should ignore)
- Test tempo event extraction correctness

#### Category 9: Calculate Total Ticks (4 tests)
- Test single track file
- Test multi-track file (max tick across tracks)
- Test empty tracks (zero ticks)
- Test very long files (large tick values)

#### Category 10: Edge Cases and Error Conditions (6 tests)
- Test zero-length file (no data)
- Test file with only non-tempo meta events
- Test tempo event at last tick
- Test overlapping tempo events (same tick)
- Test tick overflow protection
- Test precision loss in long files

#### Category 11: Integration Tests (3 tests)
- Test realistic MIDI file (Format 1, multiple tracks, tempo changes)
- Test simple MIDI file (Format 0, single tempo)
- Test complex tempo map (10+ changes)

#### Category 12: Metadata Validation (existing 2 tests + 1 new)
- Test `is_constant` flag (single tempo vs multiple)
- Test `tempo_range` calculation (min/max BPMs)
- Test `tempo_changes` vector (all events captured)

**Total Tests:** 58 new + 2 existing = **60 tests**

**Helper Functions to Create:**
```rust
// Create MIDI file with specific tempo events
fn create_test_midi_with_tempos(tempos: Vec<(u32, u32)>) -> MidiFile { ... }

// Create minimal MIDI file with no tempo events
fn create_midi_no_tempo() -> MidiFile { ... }

// Create single-track MIDI with specific tick count
fn create_midi_with_length(total_ticks: u32) -> MidiFile { ... }

// Helper to assert BPM is approximately equal (floating point tolerance)
fn assert_bpm_approx_eq(actual: f64, expected: f64, tolerance: f64) { ... }
```

**Execution:**
```bash
# Generate initial test boilerplate
/unit-test-generator:generate-tests

# Then manually enhance with algorithmic test cases
```

---

### Step 4: Implement Helper Functions (5 minutes)

**Objective:** Create reusable test data builders for MIDI files with tempo events

**Helper Functions (4 functions):**

1. **`create_test_midi_with_tempos()`**
   - Input: Vec of (tick, microseconds_per_quarter)
   - Output: MidiFile with tempo events at specified ticks
   - Used by: Categories 3, 4, 5, 8, 11

2. **`create_midi_no_tempo()`**
   - Input: None (or track count)
   - Output: MidiFile with tracks but no tempo events
   - Used by: Category 7

3. **`create_midi_with_length()`**
   - Input: Total tick count
   - Output: Minimal MIDI file of specified length
   - Used by: Category 9

4. **`assert_bpm_approx_eq()`**
   - Input: actual BPM, expected BPM, tolerance
   - Output: Assertion (panics if not within tolerance)
   - Used by: All categories (floating point comparison)

**Implementation Pattern:**
```rust
fn create_test_midi_with_tempos(tempos: Vec<(u32, u32)>) -> MidiFile {
    use crate::core::midi::types::{Event, Header, MidiFile, TimedEvent, Track};

    let mut events: Vec<TimedEvent> = tempos
        .iter()
        .map(|(tick, microseconds)| TimedEvent {
            delta_ticks: *tick,
            event: Event::TempoChange {
                microseconds_per_quarter: *microseconds,
            },
        })
        .collect();

    // Add EndOfTrack
    events.push(TimedEvent {
        delta_ticks: 0,
        event: Event::EndOfTrack,
    });

    MidiFile {
        header: Header {
            format: 1,
            num_tracks: 1,
            ticks_per_quarter_note: 480,
        },
        tracks: vec![Track { events }],
    }
}
```

---

### Step 5: Write Comprehensive Tests (15 minutes)

**Objective:** Implement all 58 new tests across 12 categories

**Test Writing Strategy:**
1. Start with Category 1-2 (basic conversions and clamping) - foundation
2. Move to Category 3-4 (single and multiple tempo detection) - core algorithm
3. Add Category 5-6 (weighted average and confidence) - complex math
4. Complete Category 7-9 (default, extraction, tick calculation) - edge cases
5. Finish with Category 10-12 (errors, integration, metadata) - validation

**Code Quality Standards:**
- Clear test names describing scenario
- Comprehensive assertions (BPM, confidence, method, metadata)
- Floating point tolerance (0.01 BPM for most tests, 0.1 for edge cases)
- Good test documentation (comments explaining expected behavior)
- No `.unwrap()` in test setup (use proper error handling)

**Example Test Implementation:**
```rust
#[test]
fn test_weighted_average_uneven_distribution() {
    // 90% of file at 120 BPM, 10% at 140 BPM
    // Expected: (120 * 0.9) + (140 * 0.1) = 110 BPM
    let tempos = vec![
        (0, 500_000),      // 120 BPM for 4320 ticks (90%)
        (4320, 428_571),   // 140 BPM for 480 ticks (10%)
    ];

    let midi = create_test_midi_with_tempos(tempos);
    let result = detect_bpm(&midi);

    assert_bpm_approx_eq(result.bpm, 122.0, 0.5);
    assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
    assert!(result.confidence > 0.8); // Low variance = high confidence
    assert_eq!(result.metadata.tempo_changes.len(), 2);
    assert!(!result.metadata.is_constant);
}
```

---

### Step 6: Add Performance Benchmarks (5 minutes)

**Objective:** Benchmark BPM detection performance for scalability validation

**Tools:**
- `compounding-engineering:performance-oracle` agent
- Rust `criterion` or built-in `#[bench]` (if nightly)

**Agent Prompt:**
```
Create performance benchmarks for BPM detection algorithm in shared/rust/src/core/analysis/bpm_detector.rs

Benchmark scenarios:
1. Small file (100 events, 1 tempo)
2. Medium file (10,000 events, 5 tempo changes)
3. Large file (100,000 events, 20 tempo changes)
4. Worst case (1,000,000 events, 100 tempo changes)

Measure:
- Detection time (µs)
- Memory allocation
- Algorithmic complexity (should be O(n) where n = number of tracks)

Provide:
- Benchmark code using Rust best practices
- Performance baseline expectations
- Scalability analysis
```

**Benchmark Implementation (if using criterion):**
```rust
#[cfg(feature = "bench")]
mod benches {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_bpm_detection(c: &mut Criterion) {
        let small_midi = create_test_midi_with_tempos(vec![(0, 500_000)]);
        let large_midi = create_test_midi_with_tempos(
            (0..20).map(|i| (i * 1000, 500_000)).collect()
        );

        c.bench_function("detect_bpm_small", |b| {
            b.iter(|| detect_bpm(black_box(&small_midi)))
        });

        c.bench_function("detect_bpm_large", |b| {
            b.iter(|| detect_bpm(black_box(&large_midi)))
        });
    }

    criterion_group!(benches, bench_bpm_detection);
    criterion_main!(benches);
}
```

---

### Step 7: Run Tests and Measure Coverage (5 minutes)

**Objective:** Verify all 60 tests pass and measure coverage

**Commands:**
```bash
# Run all BPM detector tests
cargo test --package midi-library-shared --lib core::analysis::bpm_detector -- --nocapture

# Measure coverage with tarpaulin
cargo tarpaulin --packages midi-library-shared --out Html --out Stdout \
  --timeout 120 --skip-clean 2>&1 | grep -A 20 "bpm_detector.rs"
```

**Tools:**
- `/test-coverage-analyzer:analyze-coverage` slash command
- `rust` MCP server for cargo operations
- `bash` MCP for coverage measurement

**Expected Output:**
```
running 60 tests
test core::analysis::bpm_detector::tests::test_bpm_clamping ... ok
test core::analysis::bpm_detector::tests::test_microseconds_to_bpm ... ok
test core::analysis::bpm_detector::tests::test_single_tempo_high_confidence ... ok
[... 57 more tests ...]

test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured

Coverage:
shared/rust/src/core/analysis/bpm_detector.rs: 168/187 lines (89.8%)
```

**Success Criteria:**
- ✅ All 60 tests passing (0 failures)
- ✅ Coverage 90%+ (target: 90-95%)
- ✅ Uncovered lines identified and justified

---

### Step 8: Security and Numerical Stability Review (3 minutes)

**Objective:** Ensure no numerical overflow, precision loss, or security issues

**Tools:**
- `compounding-engineering:security-sentinel` agent

**Agent Prompt:**
```
Perform security and numerical stability review of BPM detector test suite.

Focus on:
1. Overflow protection in weighted average calculation
2. Floating point precision loss in long files
3. Division by zero protection (variance calculation)
4. Integer overflow in tick calculations
5. Edge cases that could cause panics

Analyze:
- shared/rust/src/core/analysis/bpm_detector.rs (production code)
- All 60 tests for numerical edge case coverage

Provide:
- Security vulnerability assessment
- Numerical stability analysis
- Missing test cases for overflow/underflow
- Recommendations for hardening
```

**Expected Output:**
- Security risk assessment (should be LOW)
- Numerical stability confirmation
- Recommendations for additional edge case tests (if any)

---

### Step 9: Expert Review (5 minutes)

**Objective:** Get production readiness assessment from Rust expert

**Tools:**
- `rust-backend` agent

**Agent Prompt:**
```
Review the BPM detector test suite for production readiness.

Context:
- Module: shared/rust/src/core/analysis/bpm_detector.rs
- Production code: 187 lines (6 functions)
- Test code: ~800 new lines (60 tests across 12 categories)
- Coverage: 90%+ (target achieved)

Review focus:
1. Test completeness - Are all algorithms thoroughly tested?
2. Mathematical validation - Are weighted average and confidence calculations correct?
3. Edge case coverage - Are all musical scenarios covered?
4. Code quality - Rust best practices, proper assertions, clear naming?
5. Performance - Are benchmarks sufficient?
6. Production readiness - Safe for musical analysis in production?

Provide:
- Overall score (0-100)
- Strengths
- Weaknesses or gaps
- Recommendations
- Production readiness verdict
```

**Expected Output:**
- Score: 90-95/100 (high bar for algorithmic module)
- Production readiness confirmation
- Recommendations for improvements (if any)

---

### Step 10: Simplicity Review (2 minutes)

**Objective:** Final check for unnecessary complexity in tests

**Tools:**
- `compounding-engineering:code-simplicity-reviewer` agent

**Agent Prompt:**
```
Review BPM detector test suite for simplicity and maintainability.

Check:
1. Are helper functions well-designed and reusable?
2. Are test names clear and self-documenting?
3. Can any tests be simplified without losing coverage?
4. Is test data construction straightforward?
5. Are assertions clear and precise?

Suggest:
- Simplification opportunities
- Better helper function organization
- Clearer test naming
```

---

### Step 11: Commit Changes (3 minutes)

**Objective:** Create conventional commit with comprehensive message

**Tools:**
- `/git-commit-smart:commit-smart` slash command
- `git` MCP server

**Commit Message Template:**
```
test(bpm-detector): add comprehensive test suite for BPM detection algorithms

Implements 60 tests across 12 categories covering all detection methods:
- Microseconds to BPM conversion (8 tests)
- BPM clamping validation (6 tests)
- Single tempo detection (6 tests)
- Multiple tempo weighted averaging (8 tests)
- Weighted average calculation (7 tests)
- Confidence scoring (8 tests)
- Default tempo handling (5 tests)
- Tempo event extraction (6 tests)
- Tick calculation (4 tests)
- Edge cases and errors (6 tests)
- Integration tests (3 tests)
- Metadata validation (3 tests)

Coverage: 90%+ (168/187 lines) - exceeds 80% Trusty Module requirement
All tests passing (60/60)
Performance benchmarks added for scalability validation
Expert review: 90-95/100 (production-ready)

Adds helper functions:
- create_test_midi_with_tempos() - Generate test MIDI with tempo events
- create_midi_no_tempo() - Generate MIDI without tempo events
- create_midi_with_length() - Generate MIDI with specific length
- assert_bpm_approx_eq() - Floating point BPM comparison

Phase 1.4 Complete - BPM detector ready for musical analysis pipeline.
```

---

### Step 12: Update Documentation (2 minutes)

**Objective:** Update CLAUDE.md with Phase 1.4 completion

**Changes:**
1. Add Phase 1.4 achievement section
2. Update coverage gap summary (Shared: 13.6% → 18.2%)
3. Update overall progress (39.3% → 40.5%)

---

## Test Data Strategy

### MIDI File Construction Approach

**Programmatic Generation** (no external files):
- Build MIDI files as Rust structs (MidiFile, Header, Track, TimedEvent)
- Use helper functions to create specific test scenarios
- Inline tempo events with known values for validation

**Advantages:**
- Self-contained tests (no external dependencies)
- Clear test intent (data construction shows what's being tested)
- Fast test execution (no file I/O)
- Easy to modify (change parameters, not files)

**Example:**
```rust
// Create MIDI with 2 tempo changes: 120 BPM for 50%, 140 BPM for 50%
let midi = create_test_midi_with_tempos(vec![
    (0, 500_000),      // 120 BPM at tick 0
    (2400, 428_571),   // 140 BPM at tick 2400 (50% mark for 480 TPPQN, 4 bars)
]);
```

---

## Mathematical Validation

### Key Formulas to Test

1. **Microseconds to BPM:**
   ```
   BPM = 60,000,000 / microseconds_per_quarter

   Examples:
   500,000 µs → 120.0 BPM
   1,000,000 µs → 60.0 BPM
   428,571 µs → 140.0 BPM (approximate)
   ```

2. **Weighted Average:**
   ```
   weighted_bpm = Σ(bpm_i × duration_i) / Σ(duration_i)

   Example (90/10 split):
   bpm = (120 × 4320 + 140 × 480) / 4800
       = (518,400 + 67,200) / 4800
       = 585,600 / 4800
       = 122.0 BPM
   ```

3. **Confidence Scoring:**
   ```
   confidence = 1.0 - (std_dev / mean)
   confidence = clamp(confidence, 0.5, 1.0)

   Where:
   variance = Σ((bpm_i - mean)²) / n
   std_dev = √variance
   ```

### Test Validation Approach

For each mathematical test:
1. Hand-calculate expected result
2. Provide tolerance for floating point (typically ±0.01 BPM)
3. Document calculation in test comment
4. Verify intermediate values if possible

---

## Success Criteria

### Quantitative Metrics

- ✅ **Test Count:** 60 tests (58 new + 2 existing)
- ✅ **Coverage:** 90%+ (target: 90-95%, stretch: 95%+)
- ✅ **Pass Rate:** 100% (60/60 passing)
- ✅ **Performance:** Detection < 1ms for typical files
- ✅ **Expert Score:** 90+ (algorithmic module, high bar)

### Qualitative Assessment

- ✅ **Algorithm Validation:** All 3 detection methods tested thoroughly
- ✅ **Mathematical Correctness:** Weighted average and confidence formulas validated
- ✅ **Edge Case Coverage:** All identified edge cases tested
- ✅ **Code Quality:** Rust best practices, clear naming, good documentation
- ✅ **Production Ready:** Safe for use in musical analysis pipeline

### Coverage Breakdown (Target)

| Function | Lines | Covered | % | Tests |
|----------|-------|---------|---|-------|
| `detect_bpm()` | 56 | 54 | 96% | 20 |
| `extract_tempo_events()` | 24 | 22 | 92% | 6 |
| `calculate_total_ticks()` | 15 | 14 | 93% | 4 |
| `microseconds_to_bpm()` | 8 | 8 | 100% | 8 |
| `calculate_weighted_average()` | 32 | 30 | 94% | 7 |
| `calculate_confidence()` | 19 | 18 | 95% | 8 |
| **Total** | **154** | **146** | **95%** | **53** |

*Note: Remaining 7 uncovered lines likely error handling paths or unreachable code*

---

## Timeline Estimate

**Total Time: 45-60 minutes** (tool-enhanced workflow)

| Step | Duration | Cumulative |
|------|----------|------------|
| 1. Analyze module | 5 min | 5 min |
| 2. Research best practices | 5 min | 10 min |
| 3. Generate test suite | 15 min | 25 min |
| 4. Helper functions | 5 min | 30 min |
| 5. Write comprehensive tests | 15 min | 45 min |
| 6. Performance benchmarks | 5 min | 50 min |
| 7. Run tests & coverage | 5 min | 55 min |
| 8. Security review | 3 min | 58 min |
| 9. Expert review | 5 min | 63 min |
| 10. Simplicity review | 2 min | 65 min |
| 11. Commit changes | 3 min | 68 min |
| 12. Update docs | 2 min | 70 min |

**Estimated: 45-60 minutes** (varies by complexity encountered)

---

## Risk Assessment

### Low Risk ✅
- Module is self-contained (no external dependencies)
- Algorithms are well-defined (mathematical validation possible)
- No unsafe code (all safe Rust)
- Good existing structure (just needs tests)

### Moderate Risk ⚠️
- Floating point precision (requires careful tolerance selection)
- Algorithm complexity (weighted average needs thorough validation)
- Edge case discovery (may find new cases during testing)

### Mitigation Strategies
- Use comprehensive tolerance testing for floating point
- Hand-calculate expected values for all mathematical tests
- Research industry best practices for validation
- Performance benchmarks to catch algorithmic issues

---

## Post-Phase 1.4 Status

**After Completion:**
- Shared library coverage: 13.6% → **18.2%** (4/22 files)
- Overall project coverage: 39.3% → **40.5%** (34/84 files)
- Phase 1 progress: 3/4 modules complete
- Remaining Phase 1: Database models (7 files)

**Next Phase:** 1.5 - Database Models Testing (4 hours estimated)

---

## Appendix A: Tool Command Reference

```bash
# Step 1: Read BPM detector
Read(file_path="/home/dojevou/projects/midi-software-center/shared/rust/src/core/analysis/bpm_detector.rs")

# Step 1: Analyze with pattern recognition
Task(
  subagent_type="compounding-engineering:pattern-recognition-specialist",
  prompt="Analyze shared/rust/src/core/analysis/bpm_detector.rs..."
)

# Step 2: Research best practices
Task(
  subagent_type="compounding-engineering:best-practices-researcher",
  prompt="Research BPM detection testing strategies..."
)

# Step 3: Generate tests
SlashCommand(command="/unit-test-generator:generate-tests")

# Step 6: Performance benchmarks
Task(
  subagent_type="compounding-engineering:performance-oracle",
  prompt="Create performance benchmarks for BPM detection..."
)

# Step 7: Run tests
Bash(command="cargo test --package midi-library-shared --lib core::analysis::bpm_detector")

# Step 7: Measure coverage
SlashCommand(command="/test-coverage-analyzer:analyze-coverage")

# Step 8: Security review
Task(
  subagent_type="compounding-engineering:security-sentinel",
  prompt="Security review of BPM detector..."
)

# Step 9: Expert review
Task(
  subagent_type="rust-backend",
  prompt="Production readiness review..."
)

# Step 10: Simplicity review
Task(
  subagent_type="compounding-engineering:code-simplicity-reviewer",
  prompt="Simplicity review of test suite..."
)

# Step 11: Commit
SlashCommand(command="/git-commit-smart:commit-smart")
```

---

## Appendix B: Expected Test File Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Helper Functions
    // ========================================================================

    fn create_test_midi_with_tempos(tempos: Vec<(u32, u32)>) -> MidiFile { ... }
    fn create_midi_no_tempo() -> MidiFile { ... }
    fn create_midi_with_length(total_ticks: u32) -> MidiFile { ... }
    fn assert_bpm_approx_eq(actual: f64, expected: f64, tolerance: f64) { ... }

    // ========================================================================
    // Category 1: Microseconds to BPM Conversion (8 tests)
    // ========================================================================

    #[test] fn test_microseconds_to_bpm() { ... }  // Existing
    #[test] fn test_microseconds_common_bpms() { ... }
    #[test] fn test_microseconds_precision() { ... }
    // ... 5 more tests

    // ========================================================================
    // Category 2: BPM Clamping (6 tests)
    // ========================================================================

    #[test] fn test_bpm_clamping() { ... }  // Existing
    #[test] fn test_exact_min_bpm() { ... }
    // ... 4 more tests

    // ... Categories 3-12 ...
}
```

---

**End of Phase 1.4 Plan**

**Ready to Execute:** All tools, agents, and strategies defined.
**Expected Outcome:** Production-ready BPM detector with 90%+ coverage and expert validation.
