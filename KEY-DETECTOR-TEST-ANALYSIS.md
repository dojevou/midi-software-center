# Key Detector Module - Comprehensive Test Coverage Analysis

**File:** `pipeline/src-tauri/src/core/analysis/key_detector.rs` (318 lines)
**Algorithm:** Krumhansl-Schmuckler key-finding
**Current Coverage:** ~20% (6 tests)
**Target Coverage:** 90%+
**Estimated Tests Required:** 70-75 tests

---

## 1. Complete Code Path Analysis

### 1.1 `detect_key()` - Main Entry Point (Lines 83-138)

**Code Paths:**
1. **Line 85:** Call `build_pitch_class_histogram()` → depends on MIDI content
2. **Line 88:** Call `normalize_histogram()` → depends on histogram values
3. **Lines 93-107:** Loop over 24 key profiles (12 pitch classes × 2 scale types)
   - Path A: Major key correlation (lines 95-99)
   - Path B: Minor key correlation (lines 102-106)
4. **Line 111:** Sort correlations with NaN handling (`.unwrap_or()` branch)
   - Path A: Normal sort (all finite values)
   - Path B: NaN handling (partial_cmp returns None)
5. **Line 114:** Index into correlations[0] - **assumes non-empty**
6. **Line 116:** Call `format_key_name()`
7. **Line 119:** Call `calculate_confidence()`
8. **Lines 122-129:** Slice correlations[1..4] - **assumes >= 4 elements**
9. **Lines 131-137:** Construct result

**Critical Assumptions:**
- Correlations vec always has >= 4 elements (safe: always 24 elements from loop)
- No bounds checking needed for correlations[0] or [1..4]

**Edge Cases:**
- Empty MIDI file → histogram all zeros → distribution all zeros
- Single note → histogram with one non-zero entry
- All notes same pitch class → histogram with one non-zero entry
- NaN in correlations (from divide-by-zero or other numerical issues)

### 1.2 `build_pitch_class_histogram()` - Lines 141-156

**Code Paths:**
1. **Line 144:** Loop over tracks (0 to N tracks)
   - Path A: File has no tracks → loop doesn't execute → returns [0; 12]
   - Path B: File has 1+ tracks → processes all tracks
2. **Line 145:** Loop over events in each track
   - Path A: Track has no events → loop doesn't execute
   - Path B: Track has 1+ events → processes all events
3. **Lines 146-151:** Match on Event enum
   - Path A: Event is NoteOn → check velocity
     - Path A1: velocity > 0 → count note (line 147-150)
     - Path A2: velocity == 0 → skip (NoteOff as NoteOn with velocity 0)
   - Path B: Event is NOT NoteOn → skip
4. **Line 148:** Calculate pitch class: `(note % 12) as usize`
   - All MIDI notes (0-127) map to pitch classes 0-11
5. **Line 149:** Increment histogram counter (u32, can't overflow with realistic MIDI files)

**Edge Cases:**
- Empty MIDI file (no tracks) → returns [0; 12]
- MIDI file with no NoteOn events → returns [0; 12]
- MIDI file with NoteOn velocity=0 (should be treated as NoteOff) → skipped correctly
- Very low notes (note < 12) → pitch_class = note
- Very high notes (note > 120) → pitch_class = note % 12
- Extremely large MIDI file → u32 counter could theoretically overflow (unlikely)

### 1.3 `normalize_histogram()` - Lines 159-172

**Code Paths:**
1. **Line 160:** Sum all histogram values
2. **Lines 162-164:** Check if sum is zero
   - Path A: total == 0 → return [0.0; 12] (early return)
   - Path B: total > 0 → continue to normalization
3. **Lines 166-169:** Normalize each element
   - Division: histogram[i] / total (both u32, converted to f64)

**Edge Cases:**
- Empty histogram [0; 12] → returns [0.0; 12] (handled)
- Single non-zero entry → returns distribution with one 1.0, rest 0.0
- All entries equal → returns [1/12.0; 12] ≈ [0.0833; 12]
- Very large counts → no overflow (u32 converts to f64)

**Numerical Properties:**
- Sum of output should always equal 1.0 (within floating-point precision)
- All values should be in range [0.0, 1.0]

### 1.4 `rotate_profile()` - Lines 179-187

**Code Paths:**
1. **Lines 182-184:** Loop over 12 elements
   - Formula: `(i + 12 - rotation) % 12`
   - Works for rotation values 0-11 (and wraps for higher values)

**Edge Cases:**
- rotation = 0 → identity (no rotation)
- rotation = 12 → identity (full circle)
- rotation > 12 → wraps around (modulo arithmetic)
- Negative rotation (not possible with usize, but conceptually)

**Mathematical Properties:**
- Rotation by 0 should return original profile
- Rotation by 12 should return original profile
- Double rotation should be equivalent to single rotation by sum
- Rotation should preserve all values (bijection)

### 1.5 `calculate_correlation()` - Lines 190-218

**Code Paths:**
1. **Lines 192-193:** Calculate means (sum / 12.0)
2. **Lines 196-198:** Initialize accumulators for covariance and variances
3. **Lines 200-207:** Calculate covariance and variances in single pass
4. **Lines 210-211:** Calculate standard deviations (sqrt)
5. **Lines 213-215:** Check for zero standard deviation
   - Path A: std_dist == 0.0 OR std_prof == 0.0 → return 0.0
   - Path B: Both non-zero → continue
6. **Line 217:** Calculate final correlation (covariance / (std_dist * std_prof))

**Edge Cases:**
- Both distributions all zeros → means = 0, variances = 0 → return 0.0 (handled)
- One distribution all zeros → variance = 0 → return 0.0 (handled)
- Identical distributions → correlation = 1.0
- Perfectly anti-correlated → correlation = -1.0
- Uncorrelated (flat vs. varied) → correlation ≈ 0.0
- Constant distribution (all same value) → variance = 0 → return 0.0
- NaN or infinity propagation (if input contains special values)

**Numerical Stability:**
- Uses single-pass algorithm (numerically stable)
- Division by zero protected (line 213-215)
- Potential for NaN if sqrt of negative variance (shouldn't happen mathematically)

### 1.6 `calculate_confidence()` - Lines 221-238

**Code Paths:**
1. **Lines 222-224:** Check if correlations has < 2 elements
   - Path A: len < 2 → return 0.5 (fallback)
   - Path B: len >= 2 → continue
2. **Lines 226-227:** Extract best and second-best correlations
3. **Line 230:** Calculate gap (best - second_best)
4. **Line 235:** Map gap to confidence score
   - Formula: `0.5 + (gap * 2.5).min(0.5)`
   - Gap of 0.0 → confidence = 0.5
   - Gap of 0.2 → confidence = 0.5 + 0.5 = 1.0
   - Gap > 0.2 → confidence = 1.0 (capped)
5. **Line 237:** Clamp to [0.5, 1.0]

**Edge Cases:**
- correlations.len() == 0 → fallback to 0.5 (should never happen in practice)
- correlations.len() == 1 → fallback to 0.5 (should never happen: always 24)
- best == second_best → gap = 0 → confidence = 0.5
- best < second_best → gap < 0 → confidence < 0.5 → clamped to 0.5
- Large gap (> 0.2) → confidence = 1.0
- NaN in correlations → gap could be NaN → need to test

**Numerical Properties:**
- Output should always be in [0.5, 1.0]
- Monotonic: larger gap → higher confidence
- Saturates at 1.0 for gap >= 0.2

### 1.7 `format_key_name()` - Lines 241-248

**Code Paths:**
1. **Line 242:** Call `pitch_class_to_key_name()`
2. **Lines 244-247:** Match on scale_type
   - Path A: Major → return base name as-is
   - Path B: Minor → append "m" suffix

**Edge Cases:**
- pitch_class out of range 0-11 → handled by `pitch_class_to_key_name()` (returns "UNKNOWN")
- Major keys: "C", "C#", "D", ..., "B"
- Minor keys: "Cm", "C#m", "Dm", ..., "Bm"

### 1.8 `pitch_class_to_key_name()` - key_profiles.rs Lines 45-61

**Code Paths:**
1. **Lines 47-59:** Match on pitch_class (12 branches, 0-11)
2. **Line 59:** Default case → "UNKNOWN" (for pitch_class > 11)

**Edge Cases:**
- pitch_class 0-11 → standard key names
- pitch_class 12+ → "UNKNOWN"

---

## 2. Edge Case Catalog

### 2.1 Input Data Edge Cases

| Category | Description | Expected Behavior |
|----------|-------------|-------------------|
| **Empty MIDI** | No tracks | histogram all zeros → distribution all zeros → correlation 0.0 → default key (likely C) |
| **No Note Events** | Tracks exist but no NoteOn events | Same as empty MIDI |
| **Single Note** | Only one note in entire file | Histogram has one entry → distribution [0,0,0,1.0,0,...] → poor correlation |
| **Single Pitch Class** | All notes same pitch class | Histogram [N,0,0,0,0,0,0,0,0,0,0,0] → distribution [1.0,0,...] → constant distribution |
| **Two Notes** | Minimal melodic content | May have ambiguous key detection |
| **All Pitch Classes** | Chromatic (all 12 pitch classes equally) | Uniform distribution [1/12, ...] → low correlations across all keys |
| **Velocity Zero** | NoteOn with velocity=0 (NoteOff) | Should be ignored (line 147) |
| **Extreme Note Ranges** | Notes 0-127 (entire MIDI range) | All map correctly to pitch classes 0-11 |

### 2.2 Numerical Edge Cases

| Category | Description | Potential Issue | Line |
|----------|-------------|-----------------|------|
| **Zero Variance** | Constant distribution | Division by zero in correlation | 213-215 |
| **NaN Propagation** | Invalid floating-point operations | NaN in correlations | 217 |
| **Infinity** | Extreme values in correlation | Inf values | 217 |
| **Sorting NaNs** | Sorting with NaN values | partial_cmp returns None | 111 |
| **Negative Gap** | best < second_best (shouldn't happen) | Confidence clamping | 237 |
| **Large Histogram** | Overflow in u32 counters | Very unlikely (would need 4B+ notes) | 149 |
| **Precision Loss** | f64 arithmetic precision | Minor errors in sum (< 1e-10) | Throughout |

### 2.3 Musical Edge Cases

| Category | Description | Expected Key Detection |
|----------|-------------|------------------------|
| **C Major Scale** | C-D-E-F-G-A-B | C major (high confidence) |
| **A Minor Scale** | A-B-C-D-E-F-G | A minor (high confidence) |
| **Relative Keys** | C major vs. A minor (same notes) | Ambiguous (low confidence) |
| **Parallel Keys** | C major vs. C minor | Different (Eb distinguishes) |
| **Modal Music** | Dorian, Mixolydian, etc. | May misidentify as major/minor |
| **Atonal/12-tone** | All pitch classes equal | Very low confidence, arbitrary key |
| **Pentatonic** | 5-note scales | May detect as subset of major/minor |
| **Blues Scale** | Includes flat 5th | May confuse major/minor |
| **Whole Tone** | C-D-E-F#-G#-A# | Ambiguous (no tonic) |
| **Octatonic** | Alternating half/whole steps | Ambiguous |

### 2.4 Algorithm Edge Cases

| Category | Description | Test Needed |
|----------|-------------|-------------|
| **Profile Rotation** | Rotation by 0, 6, 12, 24 | Identity checks |
| **Correlation Bounds** | Should be in [-1.0, 1.0] | Verify bounds |
| **Confidence Bounds** | Should be in [0.5, 1.0] | Verify bounds |
| **Alternative Keys** | Top 4 alternatives correctly extracted | Verify ordering |
| **Ties** | Multiple keys with same correlation | Sort stability |

---

## 3. Test Scenario Categories

### Category 1: Basic Unit Tests (Functions in Isolation)
**Estimated Tests:** 12

| Test Name | Function | Description |
|-----------|----------|-------------|
| `test_normalize_histogram_basic` | normalize_histogram | Normal histogram with varied values |
| `test_normalize_histogram_empty` | normalize_histogram | All zeros (existing ✓) |
| `test_normalize_histogram_single_entry` | normalize_histogram | Only one non-zero entry |
| `test_normalize_histogram_uniform` | normalize_histogram | All entries equal |
| `test_normalize_histogram_sum` | normalize_histogram | Verify sum equals 1.0 |
| `test_rotate_profile_identity` | rotate_profile | Rotation by 0 |
| `test_rotate_profile_full_circle` | rotate_profile | Rotation by 12 |
| `test_rotate_profile_arbitrary` | rotate_profile | Rotation by 3 (existing ✓) |
| `test_rotate_profile_double` | rotate_profile | Two rotations equal sum |
| `test_format_key_name_major` | format_key_name | All 12 major keys |
| `test_format_key_name_minor` | format_key_name | All 12 minor keys |
| `test_format_key_name_invalid` | format_key_name | pitch_class > 11 |

### Category 2: Correlation Function Tests
**Estimated Tests:** 10

| Test Name | Description |
|-----------|-------------|
| `test_correlation_identical` | Identical distributions = 1.0 (existing ✓) |
| `test_correlation_zero_distribution` | Zero distribution (existing ✓) |
| `test_correlation_zero_profile` | Zero profile |
| `test_correlation_both_zero` | Both zero |
| `test_correlation_constant_distribution` | All same values |
| `test_correlation_constant_profile` | Profile all same |
| `test_correlation_anticorrelated` | Perfect negative correlation (-1.0) |
| `test_correlation_uncorrelated` | Random distributions (~0.0) |
| `test_correlation_bounds` | Result in [-1.0, 1.0] |
| `test_correlation_commutative` | corr(A,B) == corr(B,A) |

### Category 3: Histogram Building Tests
**Estimated Tests:** 8

| Test Name | Description |
|-----------|-------------|
| `test_histogram_empty_midi` | No tracks → all zeros |
| `test_histogram_no_notes` | Tracks but no NoteOn → all zeros |
| `test_histogram_single_note` | One note → one entry |
| `test_histogram_velocity_zero` | NoteOn velocity=0 ignored |
| `test_histogram_pitch_class_mapping` | Notes 0-127 map correctly |
| `test_histogram_multiple_octaves` | Same pitch class, different octaves |
| `test_histogram_chromatic` | All 12 pitch classes |
| `test_histogram_multiple_tracks` | Sum across all tracks |

### Category 4: Confidence Calculation Tests
**Estimated Tests:** 8

| Test Name | Description |
|-----------|-------------|
| `test_confidence_empty_correlations` | len=0 → 0.5 |
| `test_confidence_single_correlation` | len=1 → 0.5 |
| `test_confidence_zero_gap` | best == second → 0.5 |
| `test_confidence_small_gap` | gap=0.1 → ~0.75 |
| `test_confidence_large_gap` | gap=0.2+ → 1.0 |
| `test_confidence_negative_gap` | best < second → clamped to 0.5 |
| `test_confidence_bounds` | Always in [0.5, 1.0] |
| `test_confidence_with_nans` | NaN correlations |

### Category 5: Integration Tests - Known Musical Content
**Estimated Tests:** 15

| Test Name | Key/Scale | Expected Result |
|-----------|-----------|-----------------|
| `test_detect_c_major_scale` | C-D-E-F-G-A-B | C major (high conf) |
| `test_detect_g_major_scale` | G-A-B-C-D-E-F# | G major (high conf) |
| `test_detect_f_major_scale` | F-G-A-Bb-C-D-E | F major (high conf) |
| `test_detect_a_minor_natural` | A-B-C-D-E-F-G | A minor (high conf) |
| `test_detect_e_minor_natural` | E-F#-G-A-B-C-D | E minor (high conf) |
| `test_detect_d_minor_natural` | D-E-F-G-A-Bb-C | D minor (high conf) |
| `test_detect_c_sharp_major` | C#-D#-E#-F#-G#-A#-B# | C# major |
| `test_detect_f_sharp_minor` | F#-G#-A-B-C#-D-E | F# minor |
| `test_detect_all_sharps` | Test all 5 sharp keys |
| `test_detect_all_flats` | Test all 5 flat keys |
| `test_detect_relative_keys` | C major vs. A minor (ambiguous) |
| `test_detect_parallel_keys` | C major vs. C minor (clear) |
| `test_detect_c_major_chord_progression` | I-IV-V-I in C | C major |
| `test_detect_a_minor_chord_progression` | i-iv-V-i in Am | A minor |
| `test_detect_mixed_major_minor` | Borrowed chords | Depends on dominant |

### Category 6: Integration Tests - Edge Cases
**Estimated Tests:** 8

| Test Name | Description |
|-----------|-------------|
| `test_detect_empty_midi` | No notes → default key |
| `test_detect_single_note` | One note → low confidence |
| `test_detect_two_notes` | Minimal content |
| `test_detect_chromatic` | All 12 pitch classes equal → low conf |
| `test_detect_pentatonic` | 5-note scale → may detect subset |
| `test_detect_whole_tone` | 6-note scale → ambiguous |
| `test_detect_atonal` | Random notes → very low confidence |
| `test_detect_repeated_note` | Same note 100 times → single pitch class |

### Category 7: Numerical Stability Tests
**Estimated Tests:** 6

| Test Name | Description |
|-----------|-------------|
| `test_correlation_nan_handling` | NaN values in distribution |
| `test_correlation_infinity_handling` | Inf values |
| `test_sorting_with_nans` | Sort correlations with NaN |
| `test_large_histogram_values` | Near u32::MAX counts |
| `test_precision_normalized_sum` | Sum exactly 1.0 |
| `test_correlation_near_zero_variance` | Very small variance |

### Category 8: Property-Based Tests
**Estimated Tests:** 5

| Test Name | Property |
|-----------|----------|
| `test_normalize_always_sums_to_one` | ∑ normalized[i] = 1.0 |
| `test_correlation_bounded` | -1.0 ≤ corr ≤ 1.0 |
| `test_confidence_bounded` | 0.5 ≤ conf ≤ 1.0 |
| `test_rotate_preserves_values` | All profile values present after rotation |
| `test_histogram_sum_equals_note_count` | ∑ histogram = total NoteOn events |

---

## 4. Mathematical Validation Requirements

### 4.1 Hand-Calculated Test Cases

| Test | Input | Expected Output | Calculation |
|------|-------|-----------------|-------------|
| **Normalize [10,0,5,0,3,0,0,7,0,2,0,3]** | Sum=30 | [0.333, 0, 0.167, 0, 0.1, 0, 0, 0.233, 0, 0.067, 0, 0.1] | Each / 30 |
| **Rotate profile by 3** | [1,2,3,4,5,6,7,8,9,10,11,12], rot=3 | [10,11,12,1,2,3,4,5,6,7,8,9] | Shift right 3 |
| **Correlation: identical** | dist=[0.1, 0.2, ...], same | 1.0 | Pearson formula |
| **Correlation: anti** | [1,0,1,0,...], [0,1,0,1,...] | -1.0 | Perfect negative |
| **Confidence: gap=0.1** | best=0.8, second=0.7 | 0.75 | 0.5 + 0.1*2.5 = 0.75 |
| **Confidence: gap=0.2** | best=0.9, second=0.7 | 1.0 | 0.5 + 0.5 = 1.0 (capped) |

### 4.2 Pearson Correlation Edge Cases

**Formula:** `r = Cov(X,Y) / (σ_X * σ_Y)`

| Case | X | Y | r | Reasoning |
|------|---|---|---|-----------|
| Identical | [a,b,c,...] | [a,b,c,...] | 1.0 | Perfect positive |
| Scaled | [1,2,3] | [2,4,6] | 1.0 | Linear relationship |
| Shifted | [1,2,3] | [11,12,13] | 1.0 | Shift doesn't affect correlation |
| Anti-correlated | [1,2,3] | [3,2,1] | -1.0 | Perfect negative |
| Constant X | [5,5,5] | [1,2,3] | 0.0 | Zero variance in X |
| Constant Y | [1,2,3] | [5,5,5] | 0.0 | Zero variance in Y |
| Uncorrelated | [1,2,3] | [3,1,2] | ~0.0 | No linear relationship |

### 4.3 Profile Rotation Correctness

**Original Profile (C major):**
```
Index:  0    1    2    3    4    5    6    7    8    9   10   11
Note:   C   C#    D   D#    E    F   F#    G   G#    A   A#    B
Value: 6.35 2.23 3.48 2.33 4.38 4.09 2.52 5.19 2.39 3.66 2.29 2.88
```

**Rotated to D major (rotation=2):**
```
Index:  0    1    2    3    4    5    6    7    8    9   10   11
Note:   C   C#    D   D#    E    F   F#    G   G#    A   A#    B
Value: 3.48 2.33 6.35 2.23 3.48 2.33 4.38 4.09 2.52 5.19 2.39 3.66
        ^D's profile should have 6.35 (tonic weight) at index 2
```

**Verification:**
- Tonic weight (6.35) appears at target pitch class
- Perfect 5th weight (5.19) appears 7 semitones up
- Major 3rd weight (4.38) appears 4 semitones up

---

## 5. Coverage Projection

### 5.1 Current Coverage (~20%)

**Covered:**
- normalize_histogram: 2 tests (basic + empty)
- rotate_profile: 1 test (basic rotation)
- calculate_correlation: 2 tests (identical + zero)
- format_key_name: 1 test (basic cases)

**Not Covered:**
- detect_key: 0 integration tests
- build_pitch_class_histogram: 0 tests
- calculate_confidence: 0 tests
- Edge cases: 0 tests
- Musical validation: 0 tests

### 5.2 Projected Coverage with Full Test Suite

| Category | Tests | Lines Covered | Coverage % |
|----------|-------|---------------|------------|
| **Existing Tests** | 6 | ~60 | 20% |
| **Category 1: Basic Units** | 12 | +20 | +6% |
| **Category 2: Correlation** | 10 | +15 | +5% |
| **Category 3: Histogram** | 8 | +20 | +6% |
| **Category 4: Confidence** | 8 | +18 | +6% |
| **Category 5: Musical** | 15 | +40 | +13% |
| **Category 6: Edge Cases** | 8 | +30 | +9% |
| **Category 7: Numerical** | 6 | +15 | +5% |
| **Category 8: Properties** | 5 | +20 | +6% |
| **Total** | **72** | **238** | **~95%** |

### 5.3 Unreachable/Uncoverable Code

**None identified.** All code paths appear reachable with appropriate test inputs.

**Potential gaps:**
- Error handling: Module has no error types (uses panics for out-of-bounds)
- The `pitch_class_to_key_name()` default case ("UNKNOWN") is reachable but unlikely in production
- Some numerical edge cases (NaN propagation) may be difficult to trigger naturally

---

## 6. Test Implementation Priority

### Phase 1: Foundation (15 tests)
1. All basic unit tests (Category 1)
2. Core correlation tests (Category 2, first 5 tests)

### Phase 2: Core Algorithm (25 tests)
1. Histogram building (Category 3)
2. Confidence calculation (Category 4)
3. Remaining correlation tests (Category 2, last 5 tests)

### Phase 3: Musical Validation (15 tests)
1. All integration tests for known scales (Category 5)

### Phase 4: Edge Cases & Robustness (17 tests)
1. Edge case integration tests (Category 6)
2. Numerical stability tests (Category 7)
3. Property-based tests (Category 8)

---

## 7. Test Data Requirements

### 7.1 MIDI File Test Fixtures Needed

| Fixture | Description | Notes Count | Pitch Classes |
|---------|-------------|-------------|---------------|
| `empty.mid` | No tracks | 0 | None |
| `no_notes.mid` | Tracks with no NoteOn events | 0 | None |
| `single_note.mid` | One note (C4) | 1 | C |
| `c_major_scale.mid` | C-D-E-F-G-A-B-C | 8 | 7 unique |
| `a_minor_scale.mid` | A-B-C-D-E-F-G-A | 8 | 7 unique |
| `chromatic.mid` | All 12 pitch classes | 12 | 12 |
| `c_major_chord_prog.mid` | I-IV-V-I progression | ~16 | 7 |
| `pentatonic.mid` | C-D-E-G-A | 5 | 5 |
| `whole_tone.mid` | C-D-E-F#-G#-A# | 6 | 6 |
| `velocity_zero.mid` | NoteOn with velocity=0 | 0 (effective) | None |

### 7.2 Helper Functions for Test Data Creation

```rust
fn create_midi_with_notes(notes: &[u8]) -> MidiFile
fn create_empty_midi() -> MidiFile
fn create_scale_midi(root: u8, intervals: &[u8]) -> MidiFile
fn create_chord_progression(chords: &[&[u8]]) -> MidiFile
```

---

## 8. Expected Test Count Summary

**Total Estimated Tests:** 72

**Breakdown:**
- Basic Unit Tests: 12
- Correlation Tests: 10
- Histogram Tests: 8
- Confidence Tests: 8
- Musical Integration Tests: 15
- Edge Case Tests: 8
- Numerical Stability Tests: 6
- Property-Based Tests: 5

**Coverage Target:** 95% (exceeds 90% goal)

**Estimated Implementation Time:**
- Test data fixture creation: 2 hours
- Test implementation: 8-10 hours
- Debugging and refinement: 2 hours
- **Total:** ~12-14 hours

---

## 9. Recommendations

### 9.1 Immediate Actions
1. Create MIDI test fixture helper functions
2. Implement Category 1 (Basic Units) - quickest wins
3. Implement Category 3 (Histogram) - covers critical untested function
4. Implement Category 5 (Musical Integration) - validates algorithm correctness

### 9.2 Code Improvements (Post-Testing)
1. Add explicit error handling instead of panics
2. Document numerical stability considerations
3. Consider adding validation for pitch_class bounds
4. Add debug logging for correlation values (useful for troubleshooting)

### 9.3 Testing Infrastructure
1. Create a `test_helpers.rs` module for MIDI fixture creation
2. Consider property-based testing framework (e.g., proptest) for Categories 8
3. Document expected outputs for musical test cases (reference values)
4. Create visual inspection tools for correlation matrices (debugging aid)

---

## 10. Code Path Line Number Reference

### Complete Line-by-Line Coverage Map

| Lines | Function | Critical Paths | Tested? |
|-------|----------|----------------|---------|
| 83-138 | detect_key | Main algorithm flow | ❌ |
| 85 | detect_key | build_pitch_class_histogram call | ❌ |
| 88 | detect_key | normalize_histogram call | ✓ (indirectly) |
| 93-107 | detect_key | 24-key correlation loop | ❌ |
| 111 | detect_key | Sort with NaN handling | ❌ |
| 114 | detect_key | Extract best key | ❌ |
| 119 | detect_key | Calculate confidence | ❌ |
| 122-129 | detect_key | Extract alternatives | ❌ |
| 141-156 | build_pitch_class_histogram | Histogram building | ❌ |
| 144 | build_pitch_class_histogram | Track loop | ❌ |
| 145 | build_pitch_class_histogram | Event loop | ❌ |
| 146-151 | build_pitch_class_histogram | NoteOn matching + velocity check | ❌ |
| 159-172 | normalize_histogram | Normalization | ✓ |
| 162-164 | normalize_histogram | Zero check | ✓ |
| 179-187 | rotate_profile | Profile rotation | ✓ |
| 190-218 | calculate_correlation | Pearson correlation | ✓ (partial) |
| 213-215 | calculate_correlation | Zero variance check | ✓ |
| 221-238 | calculate_confidence | Confidence calculation | ❌ |
| 222-224 | calculate_confidence | Length check | ❌ |
| 237 | calculate_confidence | Clamping | ❌ |
| 241-248 | format_key_name | Key name formatting | ✓ |

**Coverage Summary:**
- ✓ Tested: ~60 lines (20%)
- ❌ Not Tested: ~240 lines (80%)
- **Target with full test suite:** ~285 lines (95%)

---

## Conclusion

This analysis identified **72 test scenarios** across 8 categories that will achieve **95% code coverage** for the key detector module. The test suite prioritizes:

1. **Correctness**: Mathematical validation with hand-calculated expected values
2. **Robustness**: Comprehensive edge case coverage (empty inputs, numerical stability)
3. **Musical Accuracy**: Integration tests with known scales and progressions
4. **Algorithmic Integrity**: Property-based tests for mathematical invariants

All code paths have been identified, and no unreachable code was found. The implementation priority ensures that critical untested functions (histogram building, confidence calculation) are covered early, followed by comprehensive musical validation tests.
