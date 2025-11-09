   /// MIDI Analyzer - Trusty Module
   ///
   /// Extracts complete musical metadata from MIDI files

use midly::{Smf, Timing, MetaMessage, TrackEventKind, MidiMessage};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MidiAnalysis {
    pub bpm: Option<f64>,
    pub bpm_confidence: f32,
    pub has_tempo_changes: bool,

    pub key_signature: Option<String>,
    pub key_confidence: f32,

    pub time_sig_num: i16,
    pub time_sig_den: i16,

    pub duration_seconds: f64,
    pub duration_ticks: i64,

    pub total_notes: i32,
    pub unique_pitches: i32,
    pub pitch_range_min: i16,
    pub pitch_range_max: i16,

    pub avg_velocity: f64,
    pub polyphony_max: i16,
    pub polyphony_avg: f64,

    pub is_monophonic: bool,
    pub is_polyphonic: bool,
    pub is_percussive: bool,
    pub has_chords: bool,

    pub note_density: f64,
    pub instruments: Vec<String>,
}

impl Default for MidiAnalysis {
    fn default() -> Self {
        Self {
            bpm: None,
            bpm_confidence: 0.0,
            has_tempo_changes: false,
            key_signature: None,
            key_confidence: 0.0,
            time_sig_num: 4,
            time_sig_den: 4,
            duration_seconds: 0.0,
            duration_ticks: 0,
            total_notes: 0,
            unique_pitches: 0,
            pitch_range_min: 127,
            pitch_range_max: 0,
            avg_velocity: 0.0,
            polyphony_max: 0,
            polyphony_avg: 0.0,
            is_monophonic: false,
            is_polyphonic: false,
            is_percussive: false,
            has_chords: false,
            note_density: 0.0,
            instruments: Vec::new(),
        }
    }
}

pub fn analyze_midi(data: &[u8]) -> Result<MidiAnalysis, String> {
    let smf = Smf::parse(data).map_err(|e| format!("Parse error: {}", e))?;

    let mut analysis = MidiAnalysis::default();

    // Get PPQ
    let ppq = match smf.header.timing {
        Timing::Metrical(tpb) => tpb.as_int() as i64,
        Timing::Timecode(_, _) => 480,
    };

    // Extract tempo
    let tempo_us = extract_tempo(&smf);
    if tempo_us > 0 {
        analysis.bpm = Some(60_000_000.0 / tempo_us as f64);
        analysis.bpm_confidence = 1.0;
    }

    // Extract time signature
    let (num, den) = extract_time_signature(&smf);
    analysis.time_sig_num = num;
    analysis.time_sig_den = den;

    // Collect all notes
    let notes = collect_notes(&smf);
    analysis.total_notes = notes.len() as i32;

    if !notes.is_empty() {
        // Pitch analysis
        let unique_pitches: std::collections::HashSet<u8> =
            notes.iter().map(|(_, pitch, _)| *pitch).collect();
        analysis.unique_pitches = unique_pitches.len() as i32;

        analysis.pitch_range_min = notes.iter().map(|(_, p, _)| *p).min().unwrap_or(127) as i16;
        analysis.pitch_range_max = notes.iter().map(|(_, p, _)| *p).max().unwrap_or(0) as i16;

        // Velocity analysis
        let total_vel: u32 = notes.iter().map(|(_, _, v)| *v as u32).sum();
        analysis.avg_velocity = total_vel as f64 / notes.len() as f64;

        // Polyphony analysis
        analysis.polyphony_max = calculate_max_polyphony(&notes);
        analysis.polyphony_avg = calculate_avg_polyphony(&notes);

        analysis.is_monophonic = analysis.polyphony_max <= 1;
        analysis.is_polyphonic = analysis.polyphony_max >= 3;
        analysis.has_chords = analysis.polyphony_max >= 3;

        // Check for percussion (channel 10 or low pitch range)
        analysis.is_percussive = notes.iter().any(|(_, pitch, _)| *pitch < 36);

        // Duration
        let max_tick = notes.iter().map(|(tick, _, _)| *tick).max().unwrap_or(0);
        analysis.duration_ticks = max_tick as i64;

        if let Some(bpm) = analysis.bpm {
            if bpm > 0.0 {
                analysis.duration_seconds = (max_tick as f64 / ppq as f64) * (60.0 / bpm);
            }
        }

        // Note density
        if analysis.duration_seconds > 0.0 {
            analysis.note_density = analysis.total_notes as f64 / analysis.duration_seconds;
        }

        // Key detection
        if let Some((key, confidence)) = detect_key(&notes) {
            analysis.key_signature = Some(key);
            analysis.key_confidence = confidence;
        }

        // Instruments
        analysis.instruments = extract_instruments(&smf);
    }

    Ok(analysis)
}

fn extract_tempo(smf: &Smf) -> u32 {
    for track in &smf.tracks {
        for event in track {
            if let TrackEventKind::Meta(MetaMessage::Tempo(tempo)) = event.kind {
                return tempo.as_int();
            }
        }
    }
    500_000 // Default 120 BPM
}

fn extract_time_signature(smf: &Smf) -> (i16, i16) {
    for track in &smf.tracks {
        for event in track {
            if let TrackEventKind::Meta(MetaMessage::TimeSignature(num, den, _, _)) = event.kind {
                return (num as i16, 2_i16.pow(den as u32));
            }
        }
    }
    (4, 4)
}

fn collect_notes(smf: &Smf) -> Vec<(u32, u8, u8)> {
    let mut notes = Vec::new();

    for track in &smf.tracks {
        let mut tick = 0u32;
        for event in track {
            tick += event.delta.as_int();

            if let TrackEventKind::Midi { message, .. } = event.kind {
                if let MidiMessage::NoteOn { key, vel } = message {
                    if vel.as_int() > 0 {
                        notes.push((tick, key.as_int(), vel.as_int()));
                    }
                }
            }
        }
    }

    notes
}

fn calculate_max_polyphony(notes: &[(u32, u8, u8)]) -> i16 {
    let mut note_states: HashMap<u32, Vec<u8>> = HashMap::new();

    for (tick, pitch, _) in notes {
        note_states.entry(*tick).or_insert_with(Vec::new).push(*pitch);
    }

    note_states.values()
        .map(|pitches| {
            pitches.iter().collect::<std::collections::HashSet<_>>().len() as i16
        })
        .max()
        .unwrap_or(0)
}

fn calculate_avg_polyphony(notes: &[(u32, u8, u8)]) -> f64 {
    if notes.is_empty() {
        return 0.0;
    }

    let max = calculate_max_polyphony(notes);
    max as f64 * 0.6 // Rough estimate
}

fn detect_key(notes: &[(u32, u8, u8)]) -> Option<(String, f32)> {
    if notes.is_empty() {
        return None;
    }

    // Krumhansl-Schmuckler algorithm
    const MAJOR_PROFILE: [f32; 12] = [
        6.35, 2.23, 3.48, 2.33, 4.38, 4.09,
        2.52, 5.19, 2.39, 3.66, 2.29, 2.88
    ];

    const MINOR_PROFILE: [f32; 12] = [
        6.33, 2.68, 3.52, 5.38, 2.60, 3.53,
        2.54, 4.75, 3.98, 2.69, 3.34, 3.17
    ];

    const NOTE_NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F",
        "F#", "G", "G#", "A", "A#", "B"
    ];

    // Count pitch class occurrences
    let mut pitch_class_counts = [0u32; 12];
    for (_, pitch, _) in notes {
        pitch_class_counts[(pitch % 12) as usize] += 1;
    }

    // Normalize
    let total: u32 = pitch_class_counts.iter().sum();
    if total == 0 {
        return None;
    }

    let distribution: Vec<f32> = pitch_class_counts
        .iter()
        .map(|&c| c as f32 / total as f32)
        .collect();

    // Find best correlation
    let mut best_key = String::new();
    let mut best_corr = -1.0f32;

    for tonic in 0..12 {
        // Test major
        let major_corr = correlate(&distribution, &rotate_profile(&MAJOR_PROFILE, tonic));
        if major_corr > best_corr {
            best_corr = major_corr;
            best_key = NOTE_NAMES[tonic].to_string();
        }

        // Test minor
        let minor_corr = correlate(&distribution, &rotate_profile(&MINOR_PROFILE, tonic));
        if minor_corr > best_corr {
            best_corr = minor_corr;
            best_key = format!("{}m", NOTE_NAMES[tonic]);
        }
    }

    Some((best_key, best_corr.max(0.0).min(1.0)))
}

fn correlate(a: &[f32], b: &[f32]) -> f32 {
    let mean_a: f32 = a.iter().sum::<f32>() / a.len() as f32;
    let mean_b: f32 = b.iter().sum::<f32>() / b.len() as f32;

    let mut numerator = 0.0f32;
    let mut denom_a = 0.0f32;
    let mut denom_b = 0.0f32;

    for i in 0..a.len() {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        numerator += da * db;
        denom_a += da * da;
        denom_b += db * db;
    }

    if denom_a == 0.0 || denom_b == 0.0 {
        return 0.0;
    }

    numerator / (denom_a * denom_b).sqrt()
}

fn rotate_profile(profile: &[f32; 12], steps: usize) -> [f32; 12] {
    let mut rotated = [0.0f32; 12];
    for i in 0..12 {
        rotated[i] = profile[(i + steps) % 12];
    }
    rotated
}

fn extract_instruments(smf: &Smf) -> Vec<String> {
    let mut instruments = Vec::new();

    for track in &smf.tracks {
        for event in track {
            if let TrackEventKind::Midi { message, .. } = event.kind {
                if let MidiMessage::ProgramChange { program } = message {
                    let instrument = get_instrument_name(program.as_int());
                    if !instruments.contains(&instrument) {
                        instruments.push(instrument);
                    }
                }
            }
        }
    }

    instruments
}

fn get_instrument_name(program: u8) -> String {
    match program {
        0..=7 => "Piano".to_string(),
        8..=15 => "Chromatic Percussion".to_string(),
        16..=23 => "Organ".to_string(),
        24..=31 => "Guitar".to_string(),
        32..=39 => "Bass".to_string(),
        40..=47 => "Strings".to_string(),
        48..=55 => "Ensemble".to_string(),
        56..=63 => "Brass".to_string(),
        64..=71 => "Reed".to_string(),
        72..=79 => "Pipe".to_string(),
        80..=87 => "Synth Lead".to_string(),
        88..=95 => "Synth Pad".to_string(),
        96..=103 => "Synth Effects".to_string(),
        104..=111 => "Ethnic".to_string(),
        112..=119 => "Percussive".to_string(),
        120..=127 => "Sound Effects".to_string(),
        _ => "Unknown".to_string(),
    }
}
