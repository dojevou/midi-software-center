# Key Detector Test Implementation - Quick Reference

**Use this guide during test implementation. See KEY-DETECTOR-TEST-ANALYSIS.md for complete details.**

---

## Test Helper Functions to Create

```rust
// In test_helpers.rs or in #[cfg(test)] mod

/// Create empty MIDI file (no tracks)
fn create_empty_midi() -> MidiFile {
    MidiFile {
        header: Header {
            format: 0,
            num_tracks: 0,
            ticks_per_quarter_note: 480,
        },
        tracks: vec![],
    }
}

/// Create MIDI file with specified notes
fn create_midi_with_notes(notes: &[u8]) -> MidiFile {
    let events: Vec<TimedEvent> = notes
        .iter()
        .map(|&note| TimedEvent {
            delta_ticks: 480,
            event: Event::NoteOn {
                channel: 0,
                note,
                velocity: 64,
            },
        })
        .collect();

    MidiFile {
        header: Header {
            format: 0,
            num_tracks: 1,
            ticks_per_quarter_note: 480,
        },
        tracks: vec![Track { events }],
    }
}

/// Create MIDI file for a musical scale
fn create_scale_midi(root: u8, intervals: &[u8]) -> MidiFile {
    let mut notes = vec![root];
    let mut current = root;
    for &interval in intervals {
        current += interval;
        notes.push(current);
    }
    create_midi_with_notes(&notes)
}

/// Create C major scale MIDI
fn create_c_major_scale() -> MidiFile {
    // C-D-E-F-G-A-B-C
    create_scale_midi(60, &[2, 2, 1, 2, 2, 2, 1])
}

/// Create A minor natural scale MIDI
fn create_a_minor_scale() -> MidiFile {
    // A-B-C-D-E-F-G-A
    create_scale_midi(69, &[2, 1, 2, 2, 1, 2, 2])
}

/// Create chromatic MIDI (all 12 pitch classes)
fn create_chromatic_midi() -> MidiFile {
    let notes: Vec<u8> = (60..72).collect(); // C4 to B4
    create_midi_with_notes(&notes)
}

/// Create MIDI with single note repeated multiple times
fn create_repeated_note_midi(note: u8, count: usize) -> MidiFile {
    let notes: Vec<u8> = vec![note; count];
    create_midi_with_notes(&notes)
}
```

---

## Phase 1: Foundation (15 tests, ~3 hours)

### Category 1: Basic Unit Tests (12 tests)

```rust
#[test]
fn test_normalize_histogram_basic() {
    let histogram = [10, 0, 5, 0, 3, 0, 0, 7, 0, 2, 0, 3];
    let normalized = normalize_histogram(&histogram);
    let total: f64 = normalized.iter().sum();
    assert!((total - 1.0).abs() < 1e-10);
    assert!((normalized[0] - 0.333333).abs() < 1e-5);
}

#[test]
fn test_normalize_histogram_single_entry() {
    let histogram = [0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 0];
    let normalized = normalize_histogram(&histogram);
    assert_eq!(normalized[3], 1.0);
    assert_eq!(normalized[0], 0.0);
}

#[test]
fn test_normalize_histogram_uniform() {
    let histogram = [10; 12];
    let normalized = normalize_histogram(&histogram);
    for &val in &normalized {
        assert!((val - 1.0/12.0).abs() < 1e-10);
    }
}

#[test]
fn test_rotate_profile_identity() {
    let profile = MAJOR_PROFILE;
    let rotated = rotate_profile(&profile, 0);
    assert_eq!(rotated, profile);
}

#[test]
fn test_rotate_profile_full_circle() {
    let profile = MAJOR_PROFILE;
    let rotated = rotate_profile(&profile, 12);
    assert_eq!(rotated, profile);
}

#[test]
fn test_rotate_profile_double() {
    let profile = MAJOR_PROFILE;
    let rotated_3 = rotate_profile(&profile, 3);
    let rotated_5 = rotate_profile(&profile, 5);
    let rotated_8 = rotate_profile(&profile, 8);
    assert_eq!(rotate_profile(&rotated_3, 5), rotated_8);
    assert_eq!(rotate_profile(&rotated_5, 3), rotated_8);
}

#[test]
fn test_format_key_name_all_major() {
    let expected = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    for (pc, &expected_name) in expected.iter().enumerate() {
        assert_eq!(format_key_name(pc, ScaleType::Major), expected_name);
    }
}

#[test]
fn test_format_key_name_all_minor() {
    let expected = ["Cm", "C#m", "Dm", "D#m", "Em", "Fm", "F#m", "Gm", "G#m", "Am", "A#m", "Bm"];
    for (pc, &expected_name) in expected.iter().enumerate() {
        assert_eq!(format_key_name(pc, ScaleType::Minor), expected_name);
    }
}

#[test]
fn test_format_key_name_invalid_pitch_class() {
    // pitch_class_to_key_name returns "UNKNOWN" for pc > 11
    let result = format_key_name(100, ScaleType::Major);
    assert_eq!(result, "UNKNOWN");
}
```

### Category 2: Core Correlation Tests (First 5 tests)

```rust
#[test]
fn test_correlation_zero_profile() {
    let dist = [0.1, 0.2, 0.1, 0.05, 0.15, 0.1, 0.05, 0.15, 0.03, 0.02, 0.03, 0.02];
    let profile = [0.0; 12];
    let corr = calculate_correlation(&dist, &profile);
    assert_eq!(corr, 0.0);
}

#[test]
fn test_correlation_both_zero() {
    let dist = [0.0; 12];
    let profile = [0.0; 12];
    let corr = calculate_correlation(&dist, &profile);
    assert_eq!(corr, 0.0);
}

#[test]
fn test_correlation_constant_distribution() {
    let dist = [0.0833333; 12]; // uniform distribution
    let profile = MAJOR_PROFILE;
    let corr = calculate_correlation(&dist, &profile);
    // Constant distribution should have correlation ~0 with any non-constant profile
    assert!(corr.abs() < 0.1);
}

#[test]
fn test_correlation_anticorrelated() {
    // Create two distributions that are negatively correlated
    let dist1 = [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0];
    let dist2 = [0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0];
    let normalized_dist1 = normalize_histogram(&[6; 12]);
    let normalized_dist2 = normalize_histogram(&[6; 12]);
    // Need to construct actual anti-correlated distributions
    let corr = calculate_correlation(&dist1, &dist2);
    assert!(corr < 0.0); // negative correlation
}

#[test]
fn test_correlation_bounds() {
    // Test that correlation is always in [-1, 1]
    for _ in 0..100 {
        // Use various random-like distributions
        let dist = normalize_histogram(&[10, 5, 8, 3, 12, 7, 4, 9, 6, 11, 2, 13]);
        let profile = MAJOR_PROFILE;
        let corr = calculate_correlation(&dist, &profile);
        assert!(corr >= -1.0 && corr <= 1.0);
    }
}
```

---

## Phase 2: Core Algorithm (25 tests, ~5 hours)

### Category 3: Histogram Building Tests (8 tests)

```rust
#[test]
fn test_histogram_empty_midi() {
    let midi = create_empty_midi();
    let histogram = build_pitch_class_histogram(&midi);
    assert_eq!(histogram, [0; 12]);
}

#[test]
fn test_histogram_no_notes() {
    let midi = MidiFile {
        header: Header { format: 0, num_tracks: 1, ticks_per_quarter_note: 480 },
        tracks: vec![Track { events: vec![] }],
    };
    let histogram = build_pitch_class_histogram(&midi);
    assert_eq!(histogram, [0; 12]);
}

#[test]
fn test_histogram_single_note() {
    let midi = create_midi_with_notes(&[60]); // C4
    let histogram = build_pitch_class_histogram(&midi);
    assert_eq!(histogram[0], 1); // C (pitch class 0)
    assert_eq!(histogram.iter().sum::<u32>(), 1);
}

#[test]
fn test_histogram_velocity_zero() {
    let midi = MidiFile {
        header: Header { format: 0, num_tracks: 1, ticks_per_quarter_note: 480 },
        tracks: vec![Track {
            events: vec![
                TimedEvent {
                    delta_ticks: 0,
                    event: Event::NoteOn { channel: 0, note: 60, velocity: 0 },
                },
            ],
        }],
    };
    let histogram = build_pitch_class_histogram(&midi);
    assert_eq!(histogram, [0; 12]); // velocity=0 should be ignored
}

#[test]
fn test_histogram_pitch_class_mapping() {
    // Test notes across full MIDI range (0-127)
    let notes = vec![0, 12, 24, 36, 48, 60, 72, 84, 96, 108, 120]; // All C notes
    let midi = create_midi_with_notes(&notes);
    let histogram = build_pitch_class_histogram(&midi);
    assert_eq!(histogram[0], 11); // All map to pitch class 0 (C)
    assert_eq!(histogram.iter().sum::<u32>(), 11);
}

#[test]
fn test_histogram_multiple_octaves() {
    let notes = vec![60, 72, 84]; // C4, C5, C6
    let midi = create_midi_with_notes(&notes);
    let histogram = build_pitch_class_histogram(&midi);
    assert_eq!(histogram[0], 3); // All are pitch class 0
}

#[test]
fn test_histogram_chromatic() {
    let midi = create_chromatic_midi();
    let histogram = build_pitch_class_histogram(&midi);
    for i in 0..12 {
        assert_eq!(histogram[i], 1); // Each pitch class appears once
    }
}

#[test]
fn test_histogram_multiple_tracks() {
    let midi = MidiFile {
        header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
        tracks: vec![
            Track {
                events: vec![
                    TimedEvent { delta_ticks: 0, event: Event::NoteOn { channel: 0, note: 60, velocity: 64 } },
                ],
            },
            Track {
                events: vec![
                    TimedEvent { delta_ticks: 0, event: Event::NoteOn { channel: 0, note: 60, velocity: 64 } },
                ],
            },
        ],
    };
    let histogram = build_pitch_class_histogram(&midi);
    assert_eq!(histogram[0], 2); // C appears in both tracks
}
```

### Category 4: Confidence Calculation Tests (8 tests)

```rust
#[test]
fn test_confidence_empty_correlations() {
    let correlations: Vec<(usize, ScaleType, f64)> = vec![];
    let confidence = calculate_confidence(&correlations);
    assert_eq!(confidence, 0.5);
}

#[test]
fn test_confidence_single_correlation() {
    let correlations = vec![(0, ScaleType::Major, 0.8)];
    let confidence = calculate_confidence(&correlations);
    assert_eq!(confidence, 0.5);
}

#[test]
fn test_confidence_zero_gap() {
    let correlations = vec![
        (0, ScaleType::Major, 0.7),
        (9, ScaleType::Minor, 0.7),
    ];
    let confidence = calculate_confidence(&correlations);
    assert_eq!(confidence, 0.5);
}

#[test]
fn test_confidence_small_gap() {
    let correlations = vec![
        (0, ScaleType::Major, 0.8),
        (9, ScaleType::Minor, 0.7),
    ];
    let confidence = calculate_confidence(&correlations);
    // gap = 0.1 → confidence = 0.5 + 0.1*2.5 = 0.75
    assert!((confidence - 0.75).abs() < 1e-10);
}

#[test]
fn test_confidence_large_gap() {
    let correlations = vec![
        (0, ScaleType::Major, 0.9),
        (9, ScaleType::Minor, 0.6),
    ];
    let confidence = calculate_confidence(&correlations);
    // gap = 0.3 → should saturate at 1.0
    assert_eq!(confidence, 1.0);
}

#[test]
fn test_confidence_negative_gap() {
    // Shouldn't happen in practice, but test clamping
    let correlations = vec![
        (0, ScaleType::Major, 0.6),
        (9, ScaleType::Minor, 0.8), // second is higher!
    ];
    let confidence = calculate_confidence(&correlations);
    assert_eq!(confidence, 0.5); // clamped to minimum
}

#[test]
fn test_confidence_bounds() {
    // Test various gaps to ensure always in [0.5, 1.0]
    let gaps = [0.0, 0.05, 0.1, 0.15, 0.2, 0.3, 0.5];
    for &gap in &gaps {
        let correlations = vec![
            (0, ScaleType::Major, 0.8),
            (9, ScaleType::Minor, 0.8 - gap),
        ];
        let confidence = calculate_confidence(&correlations);
        assert!(confidence >= 0.5 && confidence <= 1.0);
    }
}

#[test]
fn test_confidence_with_nans() {
    let correlations = vec![
        (0, ScaleType::Major, f64::NAN),
        (9, ScaleType::Minor, 0.7),
    ];
    let confidence = calculate_confidence(&correlations);
    // Should handle NaN gracefully (gap will be NaN)
    assert!(confidence.is_nan() || (confidence >= 0.5 && confidence <= 1.0));
}
```

---

## Phase 3: Musical Validation (15 tests, ~3 hours)

### Category 5: Musical Integration Tests (15 tests)

```rust
#[test]
fn test_detect_c_major_scale() {
    let midi = create_c_major_scale();
    let result = detect_key(&midi);
    assert_eq!(result.key, "C");
    assert_eq!(result.scale_type, ScaleType::Major);
    assert!(result.confidence > 0.7); // high confidence
}

#[test]
fn test_detect_a_minor_scale() {
    let midi = create_a_minor_scale();
    let result = detect_key(&midi);
    assert_eq!(result.key, "Am");
    assert_eq!(result.scale_type, ScaleType::Minor);
    assert!(result.confidence > 0.7);
}

#[test]
fn test_detect_g_major_scale() {
    // G-A-B-C-D-E-F#-G
    let midi = create_scale_midi(67, &[2, 2, 1, 2, 2, 2, 1]);
    let result = detect_key(&midi);
    assert_eq!(result.key, "G");
    assert_eq!(result.scale_type, ScaleType::Major);
}

#[test]
fn test_detect_f_major_scale() {
    // F-G-A-Bb-C-D-E-F
    let midi = create_scale_midi(65, &[2, 2, 1, 2, 2, 2, 1]);
    let result = detect_key(&midi);
    assert_eq!(result.key, "F");
    assert_eq!(result.scale_type, ScaleType::Major);
}

#[test]
fn test_detect_relative_keys() {
    // C major and A minor have the same notes
    let c_major = create_c_major_scale();
    let a_minor = create_a_minor_scale();

    let result_c = detect_key(&c_major);
    let result_a = detect_key(&a_minor);

    // Both should have reasonable confidence but not super high
    // (since they're ambiguous)
    assert!(result_c.confidence < 0.9);
    assert!(result_a.confidence < 0.9);
}

#[test]
fn test_detect_parallel_keys() {
    // C major vs C minor - should be distinguishable
    let c_major = create_c_major_scale();
    // C minor: C-D-Eb-F-G-Ab-Bb-C
    let c_minor = create_scale_midi(60, &[2, 1, 2, 2, 1, 2, 2]);

    let result_major = detect_key(&c_major);
    let result_minor = detect_key(&c_minor);

    assert_eq!(result_major.scale_type, ScaleType::Major);
    assert_eq!(result_minor.scale_type, ScaleType::Minor);
}

// Add 9 more tests for different keys, progressions, etc.
// See full analysis document for complete list
```

---

## Phase 4: Robustness (17 tests, ~3 hours)

### Category 6: Edge Case Integration Tests (8 tests)

```rust
#[test]
fn test_detect_empty_midi() {
    let midi = create_empty_midi();
    let result = detect_key(&midi);
    // Should return some default key with low confidence
    assert!(result.confidence <= 0.5);
}

#[test]
fn test_detect_single_note() {
    let midi = create_midi_with_notes(&[60]);
    let result = detect_key(&midi);
    // Low confidence due to minimal data
    assert!(result.confidence < 0.7);
}

#[test]
fn test_detect_chromatic() {
    let midi = create_chromatic_midi();
    let result = detect_key(&midi);
    // Very low confidence - all pitch classes equal
    assert!(result.confidence < 0.6);
}

#[test]
fn test_detect_repeated_note() {
    let midi = create_repeated_note_midi(60, 100);
    let result = detect_key(&midi);
    // Single pitch class - low confidence
    assert!(result.confidence < 0.6);
}

// Add 4 more edge case tests
```

---

## Quick Test Checklist

**After implementing all tests, verify:**

- [ ] All 72 tests pass
- [ ] Coverage >= 95% (run `cargo tllvm-cov` or similar)
- [ ] No warnings or clippy errors in test code
- [ ] Test fixtures are reusable and well-documented
- [ ] Hand-calculated expected values are documented in comments
- [ ] Edge cases all have assertions
- [ ] Property-based tests use appropriate ranges

**Files to create/modify:**
1. `pipeline/src-tauri/src/core/analysis/key_detector.rs` - add tests in `#[cfg(test)] mod tests`
2. Optionally create `test_helpers.rs` for reusable fixture functions
3. Update coverage baseline in TEST-COVERAGE-PLAN.md after completion

**Estimated time per phase:**
- Phase 1: 3 hours (foundation)
- Phase 2: 5 hours (core algorithm)
- Phase 3: 3 hours (musical validation)
- Phase 4: 3 hours (robustness)
- **Total: 14 hours**
