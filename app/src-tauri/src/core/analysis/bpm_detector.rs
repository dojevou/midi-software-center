/// BPM Detection Module
///
/// This module provides BPM (Beats Per Minute) detection for MIDI files.
/// It analyzes tempo change events and provides confidence scores.
///
/// # Archetype: Trusty Module
/// - Pure functions with no side effects
/// - No I/O operations
/// - Highly testable
/// - Reusable across the application
use crate::core::midi::types::{Event, MidiFile};

/// Default BPM when no tempo events are found
const DEFAULT_BPM: f64 = 120.0;

/// Minimum valid BPM
const MIN_BPM: f64 = 20.0;

/// Maximum valid BPM
const MAX_BPM: f64 = 300.0;

/// Result of BPM detection
#[derive(Debug, Clone, PartialEq)]
pub struct BpmDetectionResult {
    /// Detected BPM (beats per minute)
    pub bpm: f64,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,

    /// Detection method used
    pub method: BpmDetectionMethod,

    /// Additional metadata
    pub metadata: BpmMetadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BpmDetectionMethod {
    /// Single tempo event found
    SingleTempo,

    /// Multiple tempo events, used weighted average
    WeightedAverage,

    /// No tempo events, used default
    DefaultTempo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BpmMetadata {
    /// All tempo changes in the file
    pub tempo_changes: Vec<TempoChange>,

    /// Whether tempo is constant throughout
    pub is_constant: bool,

    /// Tempo range (min, max) if multiple tempos
    pub tempo_range: Option<(f64, f64)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TempoChange {
    pub tick: u32,
    pub bpm: f64,
}

/// Detects BPM from a parsed MIDI file
///
/// # Arguments
/// * `midi_file` - Parsed MIDI file structure
///
/// # Returns
/// * `BpmDetectionResult` - Detection result with confidence and metadata
///
/// # Examples
/// ```ignore
/// use midi_library_shared::core::analysis::bpm_detector::detect_bpm;
/// use midi_library_shared::core::midi::types::MidiFile;
///
/// let result = detect_bpm(&midi_file);
/// println!("Detected BPM: {:.2}", result.bpm);
/// ```
pub fn detect_bpm(midi_file: &MidiFile) -> BpmDetectionResult {
    // Extract all tempo events from all tracks
    let tempo_events = extract_tempo_events(midi_file);

    if tempo_events.is_empty() {
        return BpmDetectionResult {
            bpm: DEFAULT_BPM,
            confidence: 0.3, // Low confidence for default tempo
            method: BpmDetectionMethod::DefaultTempo,
            metadata: BpmMetadata { tempo_changes: vec![], is_constant: true, tempo_range: None },
        };
    }

    // Convert tempo changes to BPM values
    let tempo_changes: Vec<TempoChange> = tempo_events
        .into_iter()
        .map(|(tick, microseconds_per_quarter)| TempoChange {
            tick,
            bpm: microseconds_to_bpm(microseconds_per_quarter),
        })
        .collect();

    // Calculate statistics
    let is_constant = tempo_changes.len() == 1;
    let bpms: Vec<f64> = tempo_changes.iter().map(|tc| tc.bpm).collect();
    let total_ticks = calculate_total_ticks(midi_file);
    let avg_bpm = calculate_weighted_average(&tempo_changes, total_ticks);

    let tempo_range = if tempo_changes.len() > 1 {
        let min = bpms.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = bpms.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        Some((min, max))
    } else {
        None
    };

    // Determine confidence based on consistency
    let confidence = calculate_confidence(&tempo_changes);

    let method = if tempo_changes.len() == 1 {
        BpmDetectionMethod::SingleTempo
    } else {
        BpmDetectionMethod::WeightedAverage
    };

    BpmDetectionResult {
        bpm: avg_bpm,
        confidence,
        method,
        metadata: BpmMetadata { tempo_changes, is_constant, tempo_range },
    }
}

/// Extracts tempo events from all tracks in the MIDI file
fn extract_tempo_events(midi_file: &MidiFile) -> Vec<(u32, u32)> {
    let mut tempo_events = Vec::new();

    for track in &midi_file.tracks {
        let mut current_tick = 0u32;

        for timed_event in &track.events {
            current_tick = current_tick.saturating_add(timed_event.delta_ticks);

            if let Event::TempoChange { microseconds_per_quarter } = timed_event.event {
                tempo_events.push((current_tick, microseconds_per_quarter));
            }
        }
    }

    // Sort by tick position
    tempo_events.sort_by_key(|(tick, _)| *tick);
    tempo_events
}

/// Calculates the total number of ticks in the MIDI file
fn calculate_total_ticks(midi_file: &MidiFile) -> u32 {
    let mut max_ticks = 0u32;

    for track in &midi_file.tracks {
        let mut track_ticks = 0u32;
        for timed_event in &track.events {
            track_ticks = track_ticks.saturating_add(timed_event.delta_ticks);
        }
        max_ticks = max_ticks.max(track_ticks);
    }

    max_ticks
}

/// Converts microseconds per quarter note to BPM
fn microseconds_to_bpm(microseconds_per_quarter: u32) -> f64 {
    // Explicitly handle zero to avoid division by zero
    if microseconds_per_quarter == 0 {
        return DEFAULT_BPM;
    }

    let bpm = 60_000_000.0 / microseconds_per_quarter as f64;

    // Clamp to valid range
    bpm.clamp(MIN_BPM, MAX_BPM)
}

/// Calculates weighted average BPM based on duration each tempo is active
fn calculate_weighted_average(tempo_changes: &[TempoChange], total_ticks: u32) -> f64 {
    if tempo_changes.is_empty() {
        return DEFAULT_BPM;
    }

    if tempo_changes.len() == 1 {
        return tempo_changes[0].bpm;
    }

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for (i, tempo_change) in tempo_changes.iter().enumerate() {
        let duration = if i + 1 < tempo_changes.len() {
            tempo_changes[i + 1].tick - tempo_change.tick
        } else {
            total_ticks.saturating_sub(tempo_change.tick)
        };

        let weight = duration as f64;
        weighted_sum += tempo_change.bpm * weight;
        total_weight += weight;
    }

    if total_weight > 0.0 {
        weighted_sum / total_weight
    } else {
        tempo_changes[0].bpm
    }
}

/// Calculates confidence score based on tempo consistency
fn calculate_confidence(tempo_changes: &[TempoChange]) -> f64 {
    if tempo_changes.is_empty() {
        return 0.3; // Low confidence for default
    }

    if tempo_changes.len() == 1 {
        return 1.0; // High confidence for single tempo
    }

    // Calculate variance in BPM values
    let bpms: Vec<f64> = tempo_changes.iter().map(|tc| tc.bpm).collect();
    let mean = bpms.iter().sum::<f64>() / bpms.len() as f64;
    let variance = bpms.iter().map(|bpm| (bpm - mean).powi(2)).sum::<f64>() / bpms.len() as f64;
    let std_dev = variance.sqrt();

    // Lower variance = higher confidence
    // Scale confidence based on coefficient of variation
    let cv = std_dev / mean;
    (1.0 - cv).clamp(0.5, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::midi::types::{Header, MidiFile, TimedEvent, Track};

    // ============================================================================
    // Helper Functions for Building Test MIDI Files
    // ============================================================================

    /// Create a MIDI file with specific tempo events
    /// Each tuple is (tick, microseconds_per_quarter)
    /// If total_ticks is Some(n), extends file to n ticks with EndOfTrack
    fn create_test_midi_with_tempos(tempos: Vec<(u32, u32)>, total_ticks: Option<u32>) -> MidiFile {
        let mut events: Vec<TimedEvent> = Vec::new();
        let mut last_tick = 0u32;

        for (tick, microseconds) in tempos {
            let delta = tick - last_tick;
            events.push(TimedEvent {
                delta_ticks: delta,
                event: Event::TempoChange { microseconds_per_quarter: microseconds },
            });
            last_tick = tick;
        }

        // Add EndOfTrack (optionally extending to total_ticks)
        let eot_delta = if let Some(total) = total_ticks {
            total.saturating_sub(last_tick)
        } else {
            0
        };

        events.push(TimedEvent { delta_ticks: eot_delta, event: Event::EndOfTrack });

        MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events }],
        }
    }

    /// Create a MIDI file with no tempo events
    fn create_midi_no_tempo(num_tracks: u16) -> MidiFile {
        let mut tracks = Vec::new();

        for _ in 0..num_tracks {
            tracks.push(Track {
                events: vec![TimedEvent { delta_ticks: 1000, event: Event::EndOfTrack }],
            });
        }

        MidiFile { header: Header { format: 1, num_tracks, ticks_per_quarter_note: 480 }, tracks }
    }

    /// Create a MIDI file with specific length (total ticks)
    fn create_midi_with_length(total_ticks: u32) -> MidiFile {
        MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![TimedEvent { delta_ticks: total_ticks, event: Event::EndOfTrack }],
            }],
        }
    }

    /// Assert BPM is approximately equal (floating point tolerance)
    fn assert_bpm_approx_eq(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() < tolerance,
            "BPM mismatch: expected {}, got {} (tolerance: {})",
            expected,
            actual,
            tolerance
        );
    }

    // ============================================================================
    // Category 1: microseconds_to_bpm() Conversion (12 tests)
    // ============================================================================

    #[test]
    fn test_microseconds_to_bpm() {
        // 120 BPM = 500,000 microseconds per quarter note
        assert_eq!(microseconds_to_bpm(500_000), 120.0);

        // 60 BPM = 1,000,000 microseconds
        assert_eq!(microseconds_to_bpm(1_000_000), 60.0);

        // 140 BPM ≈ 428,571 microseconds
        let bpm = microseconds_to_bpm(428_571);
        assert!((bpm - 140.0).abs() < 0.1);
    }

    #[test]
    fn test_microseconds_common_bpms() {
        // 90 BPM
        assert_bpm_approx_eq(microseconds_to_bpm(666_667), 90.0, 0.1);

        // 180 BPM
        assert_bpm_approx_eq(microseconds_to_bpm(333_333), 180.0, 0.1);

        // 200 BPM
        assert_eq!(microseconds_to_bpm(300_000), 200.0);
    }

    #[test]
    fn test_microseconds_exact_boundaries() {
        // Exact MIN_BPM (20 BPM = 3,000,000 µs)
        assert_eq!(microseconds_to_bpm(3_000_000), 20.0);

        // Exact MAX_BPM (300 BPM = 200,000 µs)
        assert_eq!(microseconds_to_bpm(200_000), 300.0);
    }

    #[test]
    fn test_bpm_clamping() {
        // Test minimum clamping
        let too_slow = microseconds_to_bpm(5_000_000); // Would be 12 BPM
        assert_eq!(too_slow, MIN_BPM);

        // Test maximum clamping
        let too_fast = microseconds_to_bpm(100_000); // Would be 600 BPM
        assert_eq!(too_fast, MAX_BPM);
    }

    #[test]
    fn test_microseconds_extreme_slow() {
        // 6 BPM (10,000,000 µs) → clamps to 20
        assert_eq!(microseconds_to_bpm(10_000_000), 20.0);

        // Near u32::MAX
        assert_eq!(microseconds_to_bpm(4_000_000_000), 20.0);
    }

    #[test]
    fn test_microseconds_extreme_fast() {
        // 1200 BPM (50,000 µs) → clamps to 300
        assert_eq!(microseconds_to_bpm(50_000), 300.0);

        // 6000 BPM (10,000 µs) → clamps to 300
        assert_eq!(microseconds_to_bpm(10_000), 300.0);

        // 60000 BPM (1,000 µs) → clamps to 300
        assert_eq!(microseconds_to_bpm(1_000), 300.0);
    }

    #[test]
    fn test_microseconds_precision() {
        // Test floating point precision for common tempos
        let bpm_128 = microseconds_to_bpm(468_750); // 128 BPM
        assert_bpm_approx_eq(bpm_128, 128.0, 0.01);

        let bpm_174 = microseconds_to_bpm(344_828); // 174 BPM (dnb tempo)
        assert_bpm_approx_eq(bpm_174, 174.0, 0.1);
    }

    // ============================================================================
    // Category 2: BPM Clamping Edge Cases (2 additional tests)
    // ============================================================================

    #[test]
    fn test_clamping_just_outside_bounds() {
        // Just below minimum: 19.99 BPM → clamps to 20
        assert_eq!(microseconds_to_bpm(3_001_501), 20.0);

        // Just above maximum: 300.1 BPM → clamps to 300
        assert_eq!(microseconds_to_bpm(199_900), 300.0);
    }

    #[test]
    fn test_clamping_within_bounds() {
        // Just above minimum: 20.1 BPM → no clamping
        let bpm = microseconds_to_bpm(2_985_075);
        assert!(bpm > 20.0 && bpm < 21.0);

        // Just below maximum: 299 BPM → no clamping
        let bpm = microseconds_to_bpm(200_669);
        assert!(bpm > 298.0 && bpm < 300.0);
    }

    // ============================================================================
    // Category 3: extract_tempo_events() - Empty Inputs (4 tests)
    // ============================================================================

    #[test]
    fn test_extract_tempo_events_empty_midi() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 0, ticks_per_quarter_note: 480 },
            tracks: vec![],
        };

        let events = extract_tempo_events(&midi);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_extract_tempo_events_empty_tracks() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events: vec![] }, Track { events: vec![] }],
        };

        let events = extract_tempo_events(&midi);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_extract_tempo_events_no_tempo_events() {
        let midi = create_midi_no_tempo(1);
        let events = extract_tempo_events(&midi);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_extract_tempo_events_non_tempo_only() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 480,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            }],
        };

        let events = extract_tempo_events(&midi);
        assert_eq!(events.len(), 0);
    }

    // ============================================================================
    // Category 4: extract_tempo_events() - Single Track (6 tests)
    // ============================================================================

    #[test]
    fn test_extract_tempo_single_event_at_zero() {
        let midi = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (0, 500_000));
    }

    #[test]
    fn test_extract_tempo_single_event_at_tick() {
        let midi = create_test_midi_with_tempos(vec![(1000, 428_571)], None);
        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (1000, 428_571));
    }

    #[test]
    fn test_extract_tempo_multiple_events_single_track() {
        let midi = create_test_midi_with_tempos(
            vec![(0, 500_000), (1000, 428_571), (2000, 333_333)],
            None,
        );

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 3);
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (1000, 428_571));
        assert_eq!(events[2], (2000, 333_333));
    }

    #[test]
    fn test_extract_tempo_mixed_events() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                    },
                    TimedEvent {
                        delta_ticks: 480,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 480,
                        event: Event::TempoChange { microseconds_per_quarter: 428_571 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            }],
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (960, 428_571)); // 480 + 480
    }

    #[test]
    fn test_extract_tempo_large_delta_ticks() {
        let midi = create_test_midi_with_tempos(vec![(0, 500_000), (1_000_000, 428_571)], None);

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (1_000_000, 428_571));
    }

    #[test]
    fn test_extract_tempo_consecutive_events() {
        // Multiple tempo changes with no gap
        let midi =
            create_test_midi_with_tempos(vec![(0, 500_000), (0, 428_571), (0, 333_333)], None);

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 3);
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (0, 428_571));
        assert_eq!(events[2], (0, 333_333));
    }

    // ============================================================================
    // Category 5: extract_tempo_events() - Multiple Tracks (5 tests)
    // ============================================================================

    #[test]
    fn test_extract_tempo_multiple_tracks_one_each() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::TempoChange { microseconds_per_quarter: 428_571 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
            ],
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 2);
        // Should be sorted by tick
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (1000, 428_571));
    }

    #[test]
    fn test_extract_tempo_sorting_required() {
        // Tempo events from different tracks requiring sorting
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 2000,
                            event: Event::TempoChange { microseconds_per_quarter: 333_333 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::TempoChange { microseconds_per_quarter: 428_571 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
            ],
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 3);
        // Verify sorted order
        assert_eq!(events[0], (0, 500_000));
        assert_eq!(events[1], (1000, 428_571));
        assert_eq!(events[2], (2000, 333_333));
    }

    #[test]
    fn test_extract_tempo_same_tick_different_tracks() {
        // Tempo events at same tick from different tracks
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::TempoChange { microseconds_per_quarter: 428_571 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
            ],
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, 1000);
        assert_eq!(events[1].0, 1000);
    }

    #[test]
    fn test_extract_tempo_some_tracks_empty() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track { events: vec![TimedEvent { delta_ticks: 0, event: Event::EndOfTrack }] },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track { events: vec![TimedEvent { delta_ticks: 0, event: Event::EndOfTrack }] },
            ],
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (0, 500_000));
    }

    #[test]
    fn test_extract_tempo_many_tracks() {
        // Test with 10 tracks, each with one tempo event
        let mut tracks = Vec::new();
        for i in 0..10 {
            tracks.push(Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: i * 100,
                        event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            });
        }

        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 10, ticks_per_quarter_note: 480 },
            tracks,
        };

        let events = extract_tempo_events(&midi);

        assert_eq!(events.len(), 10);
        // Verify sorted
        for i in 0..10 {
            assert_eq!(events[i as usize].0, i * 100);
        }
    }

    // ============================================================================
    // Category 6: calculate_total_ticks() (8 tests)
    // ============================================================================

    #[test]
    fn test_total_ticks_empty_midi() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 0, ticks_per_quarter_note: 480 },
            tracks: vec![],
        };

        assert_eq!(calculate_total_ticks(&midi), 0);
    }

    #[test]
    fn test_total_ticks_empty_tracks() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![Track { events: vec![] }, Track { events: vec![] }],
        };

        assert_eq!(calculate_total_ticks(&midi), 0);
    }

    #[test]
    fn test_total_ticks_single_track() {
        let midi = create_midi_with_length(1920); // 1 bar at 480 TPPQN
        assert_eq!(calculate_total_ticks(&midi), 1920);
    }

    #[test]
    fn test_total_ticks_multiple_tracks_same_length() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track { events: vec![TimedEvent { delta_ticks: 1000, event: Event::EndOfTrack }] },
                Track { events: vec![TimedEvent { delta_ticks: 1000, event: Event::EndOfTrack }] },
                Track { events: vec![TimedEvent { delta_ticks: 1000, event: Event::EndOfTrack }] },
            ],
        };

        assert_eq!(calculate_total_ticks(&midi), 1000);
    }

    #[test]
    fn test_total_ticks_multiple_tracks_different_lengths() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track { events: vec![TimedEvent { delta_ticks: 500, event: Event::EndOfTrack }] },
                Track { events: vec![TimedEvent { delta_ticks: 2000, event: Event::EndOfTrack }] },
                Track { events: vec![TimedEvent { delta_ticks: 1000, event: Event::EndOfTrack }] },
            ],
        };

        // Should return max (2000)
        assert_eq!(calculate_total_ticks(&midi), 2000);
    }

    #[test]
    fn test_total_ticks_one_long_track() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track { events: vec![TimedEvent { delta_ticks: 100, event: Event::EndOfTrack }] },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                        },
                        TimedEvent {
                            delta_ticks: 1000,
                            event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                        },
                        TimedEvent { delta_ticks: 8000, event: Event::EndOfTrack },
                    ],
                },
                Track { events: vec![TimedEvent { delta_ticks: 200, event: Event::EndOfTrack }] },
            ],
        };

        // Long track: 1000 + 1000 + 8000 = 10000
        assert_eq!(calculate_total_ticks(&midi), 10_000);
    }

    #[test]
    fn test_total_ticks_zero_delta() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            }],
        };

        assert_eq!(calculate_total_ticks(&midi), 0);
    }

    #[test]
    fn test_total_ticks_large_value() {
        let midi = create_midi_with_length(100_000_000);
        assert_eq!(calculate_total_ticks(&midi), 100_000_000);
    }

    // ============================================================================
    // Category 7: calculate_weighted_average() - Edge Cases (8 tests)
    // ============================================================================

    #[test]
    fn test_weighted_average_empty() {
        let tempo_changes = vec![];
        let total_ticks = 1000;

        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            DEFAULT_BPM
        );
    }

    #[test]
    fn test_weighted_average_single_tempo() {
        let tempo_changes = vec![TempoChange { tick: 0, bpm: 140.0 }];
        let total_ticks = 1000;

        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            140.0
        );
    }

    #[test]
    fn test_weighted_average_single_tempo_mid_file() {
        let tempo_changes = vec![TempoChange { tick: 500, bpm: 150.0 }];
        let total_ticks = 1000;

        // Duration: 1000 - 500 = 500 ticks
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            150.0
        );
    }

    #[test]
    fn test_weighted_average_last_tempo_at_end() {
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 120.0 }, TempoChange { tick: 1000, bpm: 140.0 }];
        let total_ticks = 1000;

        // Second tempo has zero duration (at end)
        // weighted_sum = 120 * 1000 + 140 * 0 = 120000
        // total_weight = 1000
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            120.0
        );
    }

    #[test]
    fn test_weighted_average_all_zero_duration() {
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 100.0 },
            TempoChange { tick: 0, bpm: 120.0 },
            TempoChange { tick: 0, bpm: 140.0 },
        ];
        let total_ticks = 0;

        // All durations are zero, should return first BPM
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            100.0
        );
    }

    #[test]
    fn test_weighted_average_total_ticks_zero() {
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 120.0 }, TempoChange { tick: 500, bpm: 140.0 }];
        let total_ticks = 0;

        // total_ticks < last tick, saturating_sub gives 0 for last tempo
        // Only first tempo has duration (500 - 0 = 500)
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            120.0
        );
    }

    #[test]
    fn test_weighted_average_total_less_than_last_tick() {
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 100.0 }, TempoChange { tick: 1000, bpm: 140.0 }];
        let total_ticks = 500; // Less than last tempo tick

        // First tempo: 1000 - 0 = 1000
        // Second tempo: 500.saturating_sub(1000) = 0
        // weighted_sum = 100 * 1000 + 140 * 0 = 100000
        // total_weight = 1000
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            100.0
        );
    }

    #[test]
    fn test_weighted_average_saturation_behavior() {
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 120.0 },
            TempoChange { tick: u32::MAX / 2, bpm: 140.0 },
        ];
        let total_ticks = u32::MAX;

        // First: u32::MAX/2 - 0 = u32::MAX/2
        // Second: u32::MAX - u32::MAX/2 ≈ u32::MAX/2
        // Both have roughly equal duration, average ≈ 130
        let result = calculate_weighted_average(&tempo_changes, total_ticks);
        assert_bpm_approx_eq(result, 130.0, 1.0);
    }

    // ============================================================================
    // Category 8: calculate_weighted_average() - Mathematical Correctness (7 tests)
    // ============================================================================

    #[test]
    fn test_weighted_average_equal_duration_50_50() {
        // 50% at 100 BPM, 50% at 120 BPM → average = 110
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 100.0 }, TempoChange { tick: 500, bpm: 120.0 }];
        let total_ticks = 1000;

        // (100 * 500 + 120 * 500) / 1000 = (50000 + 60000) / 1000 = 110
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            110.0
        );
    }

    #[test]
    fn test_weighted_average_90_10_split() {
        // 90% at 100 BPM, 10% at 120 BPM
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 100.0 }, TempoChange { tick: 900, bpm: 120.0 }];
        let total_ticks = 1000;

        // (100 * 900 + 120 * 100) / 1000 = (90000 + 12000) / 1000 = 102
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            102.0
        );
    }

    #[test]
    fn test_weighted_average_10_90_split() {
        // 10% at 100 BPM, 90% at 120 BPM
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 100.0 }, TempoChange { tick: 100, bpm: 120.0 }];
        let total_ticks = 1000;

        // (100 * 100 + 120 * 900) / 1000 = (10000 + 108000) / 1000 = 118
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            118.0
        );
    }

    #[test]
    fn test_weighted_average_three_equal_durations() {
        // 33.3% each at 80, 120, 160 BPM → average = 120
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 80.0 },
            TempoChange { tick: 300, bpm: 120.0 },
            TempoChange { tick: 700, bpm: 160.0 },
        ];
        let total_ticks = 1000;

        // (80 * 300 + 120 * 400 + 160 * 300) / 1000
        // = (24000 + 48000 + 48000) / 1000 = 120
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            120.0
        );
    }

    #[test]
    fn test_weighted_average_three_unequal_50_30_20() {
        // 50% at 100, 30% at 120, 20% at 140
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 100.0 },
            TempoChange { tick: 500, bpm: 120.0 },
            TempoChange { tick: 800, bpm: 140.0 },
        ];
        let total_ticks = 1000;

        // (100 * 500 + 120 * 300 + 140 * 200) / 1000
        // = (50000 + 36000 + 28000) / 1000 = 114
        assert_eq!(
            calculate_weighted_average(&tempo_changes, total_ticks),
            114.0
        );
    }

    #[test]
    fn test_weighted_average_five_exponential_decay() {
        // Durations: 500, 250, 125, 62, 63 (exponentially decreasing)
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 100.0 },
            TempoChange { tick: 500, bpm: 110.0 },
            TempoChange { tick: 750, bpm: 120.0 },
            TempoChange { tick: 875, bpm: 130.0 },
            TempoChange { tick: 937, bpm: 140.0 },
        ];
        let total_ticks = 1000;

        // (100*500 + 110*250 + 120*125 + 130*62 + 140*63) / 1000
        // = (50000 + 27500 + 15000 + 8060 + 8820) / 1000 = 109.38
        let result = calculate_weighted_average(&tempo_changes, total_ticks);
        assert_bpm_approx_eq(result, 109.38, 0.01);
    }

    #[test]
    fn test_weighted_average_precision_check() {
        // Verify floating point precision is maintained
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 123.456 },
            TempoChange { tick: 333, bpm: 156.789 },
            TempoChange { tick: 667, bpm: 178.901 },
        ];
        let total_ticks = 1000;

        // (123.456*333 + 156.789*334 + 178.901*333) / 1000
        // = (41110.848 + 52367.526 + 59574.033) / 1000 = 153.052407
        let result = calculate_weighted_average(&tempo_changes, total_ticks);
        assert_bpm_approx_eq(result, 153.052, 0.01);
    }

    // ============================================================================
    // Category 9: calculate_confidence() - Edge Cases (5 tests)
    // ============================================================================

    #[test]
    fn test_confidence_empty() {
        assert_eq!(calculate_confidence(&[]), 0.3);
    }

    #[test]
    fn test_confidence_single_tempo() {
        let tempo_changes = vec![TempoChange { tick: 0, bpm: 120.0 }];

        assert_eq!(calculate_confidence(&tempo_changes), 1.0);
    }

    #[test]
    fn test_confidence_two_identical() {
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 120.0 }, TempoChange { tick: 500, bpm: 120.0 }];

        // Variance = 0, cv = 0, confidence = 1.0
        assert_eq!(calculate_confidence(&tempo_changes), 1.0);
    }

    #[test]
    fn test_confidence_many_identical() {
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 140.0 },
            TempoChange { tick: 100, bpm: 140.0 },
            TempoChange { tick: 200, bpm: 140.0 },
            TempoChange { tick: 300, bpm: 140.0 },
            TempoChange { tick: 400, bpm: 140.0 },
        ];

        assert_eq!(calculate_confidence(&tempo_changes), 1.0);
    }

    #[test]
    fn test_confidence_extreme_variance_clamped() {
        // Extreme variance should clamp to minimum confidence (0.5)
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 20.0 }, TempoChange { tick: 500, bpm: 300.0 }];

        // Mean = 160, variance = 19600, std_dev = 140, cv = 0.875
        // confidence = 1.0 - 0.875 = 0.125 → clamped to 0.5
        assert_eq!(calculate_confidence(&tempo_changes), 0.5);
    }

    // ============================================================================
    // Category 10: calculate_confidence() - Variance Testing (7 tests)
    // ============================================================================

    #[test]
    fn test_confidence_very_low_variance() {
        // BPMs differ by only 1
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 120.0 }, TempoChange { tick: 500, bpm: 121.0 }];

        // Mean = 120.5, variance = 0.25, std_dev = 0.5, cv ≈ 0.00415
        // confidence ≈ 0.996
        let conf = calculate_confidence(&tempo_changes);
        assert!(conf > 0.99 && conf <= 1.0);
    }

    #[test]
    fn test_confidence_low_variance() {
        // 120, 121, 122 (±1 BPM around mean)
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 120.0 },
            TempoChange { tick: 333, bpm: 121.0 },
            TempoChange { tick: 667, bpm: 122.0 },
        ];

        // Mean = 121, variance ≈ 0.667, std_dev ≈ 0.816, cv ≈ 0.00675
        // confidence ≈ 0.993
        let conf = calculate_confidence(&tempo_changes);
        assert_bpm_approx_eq(conf, 0.993, 0.01);
    }

    #[test]
    fn test_confidence_medium_variance() {
        // 100, 140 (±20 from mean of 120)
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 100.0 }, TempoChange { tick: 500, bpm: 140.0 }];

        // Mean = 120, variance = 400, std_dev = 20, cv ≈ 0.167
        // confidence ≈ 0.833
        let conf = calculate_confidence(&tempo_changes);
        assert_bpm_approx_eq(conf, 0.833, 0.01);
    }

    #[test]
    fn test_confidence_high_variance_at_clamp() {
        // 60, 180 (±60 from mean of 120)
        let tempo_changes =
            vec![TempoChange { tick: 0, bpm: 60.0 }, TempoChange { tick: 500, bpm: 180.0 }];

        // Mean = 120, variance = 3600, std_dev = 60, cv = 0.5
        // confidence = 1.0 - 0.5 = 0.5 (exactly at clamp boundary)
        assert_eq!(calculate_confidence(&tempo_changes), 0.5);
    }

    #[test]
    fn test_confidence_five_low_variance() {
        // 115, 118, 120, 122, 125 (±5 around mean of 120)
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 115.0 },
            TempoChange { tick: 200, bpm: 118.0 },
            TempoChange { tick: 400, bpm: 120.0 },
            TempoChange { tick: 600, bpm: 122.0 },
            TempoChange { tick: 800, bpm: 125.0 },
        ];

        // Mean = 120, variance = 11.2, std_dev ≈ 3.35, cv ≈ 0.0279
        // confidence ≈ 0.972
        let conf = calculate_confidence(&tempo_changes);
        assert_bpm_approx_eq(conf, 0.972, 0.01);
    }

    #[test]
    fn test_confidence_five_high_variance() {
        // Wide range: 80, 100, 120, 140, 160
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 80.0 },
            TempoChange { tick: 200, bpm: 100.0 },
            TempoChange { tick: 400, bpm: 120.0 },
            TempoChange { tick: 600, bpm: 140.0 },
            TempoChange { tick: 800, bpm: 160.0 },
        ];

        // Mean = 120, variance = 800, std_dev ≈ 28.28, cv ≈ 0.236
        // confidence ≈ 0.764
        let conf = calculate_confidence(&tempo_changes);
        assert_bpm_approx_eq(conf, 0.764, 0.01);
    }

    #[test]
    fn test_confidence_mathematical_validation() {
        // Hand-calculated test case
        // BPMs: [100, 110, 120]
        // Mean = 110
        // Variance = ((100-110)² + (110-110)² + (120-110)²) / 3 = (100 + 0 + 100) / 3 = 66.67
        // Std dev = √66.67 ≈ 8.165
        // CV = 8.165 / 110 ≈ 0.0742
        // Confidence = 1.0 - 0.0742 ≈ 0.926
        let tempo_changes = vec![
            TempoChange { tick: 0, bpm: 100.0 },
            TempoChange { tick: 333, bpm: 110.0 },
            TempoChange { tick: 667, bpm: 120.0 },
        ];

        let conf = calculate_confidence(&tempo_changes);
        assert_bpm_approx_eq(conf, 0.926, 0.01);
    }

    // ============================================================================
    // Category 11: detect_bpm() - DefaultTempo Method (3 tests)
    // ============================================================================

    #[test]
    fn test_detect_bpm_empty_midi() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 0, ticks_per_quarter_note: 480 },
            tracks: vec![],
        };

        let result = detect_bpm(&midi);

        assert_eq!(result.bpm, DEFAULT_BPM);
        assert_eq!(result.confidence, 0.3);
        assert_eq!(result.method, BpmDetectionMethod::DefaultTempo);
        assert!(result.metadata.tempo_changes.is_empty());
        assert!(result.metadata.is_constant);
        assert_eq!(result.metadata.tempo_range, None);
    }

    #[test]
    fn test_detect_bpm_no_tempo_events() {
        let midi = create_midi_no_tempo(2);
        let result = detect_bpm(&midi);

        assert_eq!(result.bpm, 120.0);
        assert_eq!(result.confidence, 0.3);
        assert_eq!(result.method, BpmDetectionMethod::DefaultTempo);
        assert!(result.metadata.tempo_changes.is_empty());
        assert!(result.metadata.is_constant);
        assert_eq!(result.metadata.tempo_range, None);
    }

    #[test]
    fn test_detect_bpm_only_non_tempo_events() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 0,
                        event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                    },
                    TimedEvent {
                        delta_ticks: 480,
                        event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            }],
        };

        let result = detect_bpm(&midi);

        assert_eq!(result.bpm, 120.0);
        assert_eq!(result.method, BpmDetectionMethod::DefaultTempo);
        assert_eq!(result.confidence, 0.3);
    }

    // ============================================================================
    // Category 12: detect_bpm() - SingleTempo Method (4 tests)
    // ============================================================================

    #[test]
    fn test_detect_bpm_single_tempo_at_zero() {
        let midi = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        let result = detect_bpm(&midi);

        assert_eq!(result.bpm, 120.0);
        assert_eq!(result.confidence, 1.0);
        assert_eq!(result.method, BpmDetectionMethod::SingleTempo);
        assert_eq!(result.metadata.tempo_changes.len(), 1);
        assert!(result.metadata.is_constant);
        assert_eq!(result.metadata.tempo_range, None);
    }

    #[test]
    fn test_detect_bpm_single_tempo_mid_file() {
        let midi = create_test_midi_with_tempos(vec![(1000, 428_571)], None);
        let result = detect_bpm(&midi);

        assert_bpm_approx_eq(result.bpm, 140.0, 0.1);
        assert_eq!(result.confidence, 1.0);
        assert_eq!(result.method, BpmDetectionMethod::SingleTempo);
        assert!(result.metadata.is_constant);
    }

    #[test]
    fn test_detect_bpm_single_tempo_end_of_file() {
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 1, ticks_per_quarter_note: 480 },
            tracks: vec![Track {
                events: vec![
                    TimedEvent {
                        delta_ticks: 1920, // 1 bar
                        event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                    },
                    TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                ],
            }],
        };

        let result = detect_bpm(&midi);

        assert_eq!(result.bpm, 120.0);
        assert_eq!(result.method, BpmDetectionMethod::SingleTempo);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_detect_bpm_single_tempo_multiple_tracks() {
        // Same tempo event in multiple tracks (should dedupe)
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 2, ticks_per_quarter_note: 480 },
            tracks: vec![
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange { microseconds_per_quarter: 500_000 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
            ],
        };

        let result = detect_bpm(&midi);

        // Both tracks have same tempo at tick 0, so we get 2 tempo changes
        // but they're identical, so high confidence
        assert_eq!(result.bpm, 120.0);
        assert_eq!(result.metadata.tempo_changes.len(), 2);
        assert_eq!(result.confidence, 1.0); // Identical tempos
    }

    // ============================================================================
    // Category 13: detect_bpm() - WeightedAverage Method (6 tests)
    // ============================================================================

    #[test]
    fn test_detect_bpm_two_tempos_equal_duration() {
        let midi = create_test_midi_with_tempos(vec![(0, 500_000), (500, 428_571)], Some(1000));

        let result = detect_bpm(&midi);

        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
        assert_eq!(result.metadata.tempo_changes.len(), 2);
        assert!(!result.metadata.is_constant);

        // Should be approximately 130 BPM (midpoint of 120 and 140)
        assert_bpm_approx_eq(result.bpm, 130.0, 1.0);

        // Confidence should be good (similar tempos)
        assert!(result.confidence > 0.8);

        // Tempo range should be (120, 140)
        let (min, max) = result.metadata.tempo_range.unwrap();
        assert_bpm_approx_eq(min, 120.0, 0.1);
        assert_bpm_approx_eq(max, 140.0, 0.1);
    }

    #[test]
    fn test_detect_bpm_five_tempos() {
        let midi = create_test_midi_with_tempos(
            vec![
                (0, 500_000),   // 120 BPM
                (200, 468_750), // 128 BPM
                (400, 500_000), // 120 BPM
                (600, 468_750), // 128 BPM
                (800, 500_000), // 120 BPM
            ],
            None,
        );

        let result = detect_bpm(&midi);

        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
        assert_eq!(result.metadata.tempo_changes.len(), 5);

        // Should be close to average of 120 and 128 (≈124)
        assert_bpm_approx_eq(result.bpm, 124.0, 1.0);

        // Confidence should be high (low variance)
        assert!(result.confidence > 0.95);
    }

    #[test]
    fn test_detect_bpm_ten_tempos() {
        let tempos: Vec<(u32, u32)> = (0..10)
            .map(|i| (i * 100, 500_000)) // All 120 BPM at different ticks
            .collect();

        let midi = create_test_midi_with_tempos(tempos, None);
        let result = detect_bpm(&midi);

        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
        assert_eq!(result.metadata.tempo_changes.len(), 10);

        // All identical tempos → average = 120
        assert_eq!(result.bpm, 120.0);

        // All identical → maximum confidence
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_detect_bpm_weighted_average_90_10() {
        // Create MIDI with 90% at 100 BPM, 10% at 120 BPM
        let midi = create_test_midi_with_tempos(vec![(0, 600_000), (900, 500_000)], Some(1000));

        let result = detect_bpm(&midi);

        // weighted = (100 * 900 + 120 * 100) / 1000 = 102
        assert_bpm_approx_eq(result.bpm, 102.0, 0.5);
        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
    }

    #[test]
    fn test_detect_bpm_complex_real_world() {
        // Simulate a real-world MIDI with tempo changes
        // Classical piece with ritardando: 120 → 115 → 110 → 105
        let midi = create_test_midi_with_tempos(
            vec![
                (0, 500_000),    // 120 BPM (40% of file)
                (4000, 521_739), // 115 BPM (30%)
                (7000, 545_455), // 110 BPM (20%)
                (9000, 571_429), // 105 BPM (10%)
            ],
            Some(10000),
        );

        let result = detect_bpm(&midi);

        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
        assert_eq!(result.metadata.tempo_changes.len(), 4);

        // Weighted average: (120*4000 + 115*3000 + 110*2000 + 105*1000) / 10000
        // = (480000 + 345000 + 220000 + 105000) / 10000 = 115
        assert_bpm_approx_eq(result.bpm, 115.0, 0.5);

        // Good confidence (gradual tempo change)
        assert!(result.confidence > 0.9);

        // Tempo range
        let (min, max) = result.metadata.tempo_range.unwrap();
        assert_bpm_approx_eq(min, 105.0, 1.0);
        assert_bpm_approx_eq(max, 120.0, 1.0);
    }

    #[test]
    fn test_detect_bpm_high_variance_low_confidence() {
        // Extreme tempo changes: 60 → 180
        let midi = create_test_midi_with_tempos(
            vec![
                (0, 1_000_000), // 60 BPM
                (500, 333_333), // 180 BPM
            ],
            None,
        );

        let result = detect_bpm(&midi);

        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);

        // Should have low confidence due to high variance
        assert_eq!(result.confidence, 0.5); // Clamped to minimum

        let (min, max) = result.metadata.tempo_range.unwrap();
        assert_bpm_approx_eq(min, 60.0, 1.0);
        assert_bpm_approx_eq(max, 180.0, 1.0);
    }

    // ============================================================================
    // Category 14: detect_bpm() - Integration & Metadata (6 tests)
    // ============================================================================

    #[test]
    fn test_detect_bpm_metadata_tempo_changes_populated() {
        let midi = create_test_midi_with_tempos(
            vec![(0, 500_000), (1000, 428_571), (2000, 333_333)],
            None,
        );

        let result = detect_bpm(&midi);

        assert_eq!(result.metadata.tempo_changes.len(), 3);
        assert_bpm_approx_eq(result.metadata.tempo_changes[0].bpm, 120.0, 0.1);
        assert_bpm_approx_eq(result.metadata.tempo_changes[1].bpm, 140.0, 0.1);
        assert_bpm_approx_eq(result.metadata.tempo_changes[2].bpm, 180.0, 0.1);
        assert_eq!(result.metadata.tempo_changes[0].tick, 0);
        assert_eq!(result.metadata.tempo_changes[1].tick, 1000);
        assert_eq!(result.metadata.tempo_changes[2].tick, 2000);
    }

    #[test]
    fn test_detect_bpm_metadata_is_constant_flag() {
        // Single tempo → is_constant = true
        let midi_single = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        let result_single = detect_bpm(&midi_single);
        assert!(result_single.metadata.is_constant);

        // Multiple tempos → is_constant = false
        let midi_multiple = create_test_midi_with_tempos(vec![(0, 500_000), (1000, 428_571)], None);
        let result_multiple = detect_bpm(&midi_multiple);
        assert!(!result_multiple.metadata.is_constant);

        // No tempos (default) → is_constant = true
        let midi_none = create_midi_no_tempo(1);
        let result_none = detect_bpm(&midi_none);
        assert!(result_none.metadata.is_constant);
    }

    #[test]
    fn test_detect_bpm_metadata_tempo_range() {
        // Single tempo → tempo_range = None
        let midi_single = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        let result_single = detect_bpm(&midi_single);
        assert_eq!(result_single.metadata.tempo_range, None);

        // Multiple tempos → tempo_range = Some((min, max))
        let midi_multiple = create_test_midi_with_tempos(
            vec![
                (0, 600_000),    // 100 BPM
                (500, 500_000),  // 120 BPM
                (1000, 428_571), // 140 BPM
            ],
            None,
        );
        let result_multiple = detect_bpm(&midi_multiple);
        let (min, max) = result_multiple.metadata.tempo_range.unwrap();
        assert_bpm_approx_eq(min, 100.0, 0.1);
        assert_bpm_approx_eq(max, 140.0, 0.1);
    }

    #[test]
    fn test_detect_bpm_confidence_score_ranges() {
        // Default tempo → confidence = 0.3
        let midi_default = create_midi_no_tempo(1);
        assert_eq!(detect_bpm(&midi_default).confidence, 0.3);

        // Single tempo → confidence = 1.0
        let midi_single = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        assert_eq!(detect_bpm(&midi_single).confidence, 1.0);

        // Multiple identical → confidence = 1.0
        let midi_identical =
            create_test_midi_with_tempos(vec![(0, 500_000), (500, 500_000), (1000, 500_000)], None);
        assert_eq!(detect_bpm(&midi_identical).confidence, 1.0);

        // High variance → confidence = 0.5 (clamped)
        let midi_variance = create_test_midi_with_tempos(
            vec![
                (0, 1_000_000), // 60 BPM
                (500, 333_333), // 180 BPM
            ],
            None,
        );
        assert_eq!(detect_bpm(&midi_variance).confidence, 0.5);
    }

    #[test]
    fn test_detect_bpm_method_selection_logic() {
        // No tempos → DefaultTempo
        let midi_default = create_midi_no_tempo(1);
        assert_eq!(
            detect_bpm(&midi_default).method,
            BpmDetectionMethod::DefaultTempo
        );

        // One tempo → SingleTempo
        let midi_single = create_test_midi_with_tempos(vec![(0, 500_000)], None);
        assert_eq!(
            detect_bpm(&midi_single).method,
            BpmDetectionMethod::SingleTempo
        );

        // Two or more tempos → WeightedAverage
        let midi_multiple = create_test_midi_with_tempos(vec![(0, 500_000), (1000, 428_571)], None);
        assert_eq!(
            detect_bpm(&midi_multiple).method,
            BpmDetectionMethod::WeightedAverage
        );
    }

    #[test]
    fn test_detect_bpm_full_integration() {
        // Real-world integration test with realistic MIDI structure
        let midi = MidiFile {
            header: Header { format: 1, num_tracks: 3, ticks_per_quarter_note: 480 },
            tracks: vec![
                // Tempo track
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::TempoChange {
                                microseconds_per_quarter: 500_000, // 120 BPM
                            },
                        },
                        TimedEvent {
                            delta_ticks: 1920, // 1 bar
                            event: Event::TempoChange {
                                microseconds_per_quarter: 468_750, // 128 BPM
                            },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                // Melody track
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 0, note: 60, velocity: 100 },
                        },
                        TimedEvent {
                            delta_ticks: 480,
                            event: Event::NoteOff { channel: 0, note: 60, velocity: 0 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
                // Bass track
                Track {
                    events: vec![
                        TimedEvent {
                            delta_ticks: 0,
                            event: Event::NoteOn { channel: 1, note: 48, velocity: 80 },
                        },
                        TimedEvent {
                            delta_ticks: 960,
                            event: Event::NoteOff { channel: 1, note: 48, velocity: 0 },
                        },
                        TimedEvent { delta_ticks: 0, event: Event::EndOfTrack },
                    ],
                },
            ],
        };

        let result = detect_bpm(&midi);

        // Verify all aspects
        assert_eq!(result.method, BpmDetectionMethod::WeightedAverage);
        assert_eq!(result.metadata.tempo_changes.len(), 2);
        assert!(!result.metadata.is_constant);
        assert!(result.metadata.tempo_range.is_some());

        // Weighted average: (120 * 1920 + 128 * 0) / 1920 = 120 (last tempo at end has no duration)
        assert_bpm_approx_eq(result.bpm, 120.0, 0.1);

        // Very similar tempos → high confidence
        assert!(result.confidence > 0.95);
    }
}
