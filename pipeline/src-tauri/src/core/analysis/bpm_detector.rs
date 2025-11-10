
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
use midi_library_shared::core::midi::types::{Event, MidiFile};

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
/// ```no_run
/// use pipeline::core::analysis::bpm_detector::detect_bpm;
/// use pipeline::core::midi::types::MidiFile;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # let midi_file = MidiFile {
/// #     header: pipeline::core::midi::types::Header {
/// #         format: 1,
/// #         num_tracks: 1,
/// #         ticks_per_quarter_note: 480,
/// #     },
/// #     tracks: vec![],
/// # };
/// let result = detect_bpm(&midi_file);
/// println!("Detected BPM: {:.2}", result.bpm);
/// # Ok(())
/// # }
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
            current_tick += timed_event.delta_ticks;

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
            track_ticks += timed_event.delta_ticks;
        }
        max_ticks = max_ticks.max(track_ticks);
    }

    max_ticks
}

/// Converts microseconds per quarter note to BPM
fn microseconds_to_bpm(microseconds_per_quarter: u32) -> f64 {
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
    fn test_bpm_clamping() {
        // Test minimum clamping
        let too_slow = microseconds_to_bpm(5_000_000); // Would be 12 BPM
        assert_eq!(too_slow, MIN_BPM);

        // Test maximum clamping
        let too_fast = microseconds_to_bpm(100_000); // Would be 600 BPM
        assert_eq!(too_fast, MAX_BPM);
    }
}
