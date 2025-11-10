//! Drum-specific MIDI analysis and tagging
//!
//! Analyzes drum MIDI files for:
//! - GM drum channel detection (channel 10)
//! - GM drum note mapping (kick=35/36, snare=38/40, etc.)
//! - Pattern type detection (groove, fill, intro, ending)
//! - Rhythmic feel detection (straight, swing, shuffle, triplet)
//! - Time signature extraction from filename/path
//! - BPM extraction from filename patterns
//! - Cymbal type detection based on GM notes
//! - Technique detection (ghost notes, linear, double-bass)
//! - Song structure section detection (verse, chorus, bridge)
//!
//! **Archetype: Trusty Module** (Pure functions, no I/O, 80%+ test coverage)

use crate::core::analysis::auto_tagger::Tag;
use midi_library_shared::core::midi::types::{Event, MidiFile};
use std::collections::HashMap;

// ============================================================================
// CORE DATA STRUCTURES
// ============================================================================

/// GM Drum Note Mapping (General MIDI Standard)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrumNote {
    // Kick drums
    AcousticBassDrum = 35,
    BassDrum1 = 36,

    // Snares
    SideStick = 37,
    AcousticSnare = 38,
    HandClap = 39,
    ElectricSnare = 40,

    // Toms
    LowFloorTom = 41,
    HighFloorTom = 43,
    LowTom = 45,
    LowMidTom = 47,
    HighMidTom = 48,
    HighTom = 50,

    // Hi-hats
    ClosedHiHat = 42,
    PedalHiHat = 44,
    OpenHiHat = 46,

    // Cymbals
    CrashCymbal1 = 49,
    RideCymbal1 = 51,
    ChineseCymbal = 52,
    RideBell = 53,
    SplashCymbal = 55,
    CrashCymbal2 = 57,
    RideCymbal2 = 59,

    // Latin percussion
    Tambourine = 54,
    Cowbell = 56,
    HighBongo = 60,
    LowBongo = 61,
    MuteHighConga = 62,
    OpenHighConga = 63,
    LowConga = 64,
    HighTimbale = 65,
    LowTimbale = 66,
    HighAgogo = 67,
    LowAgogo = 68,
    Cabasa = 69,
    Maracas = 70,
    ShortWhistle = 71,
    LongWhistle = 72,
    ShortGuiro = 73,
    LongGuiro = 74,
    Claves = 75,
    HighWoodBlock = 76,
    LowWoodBlock = 77,
    MuteCuica = 78,
    OpenCuica = 79,
    MuteTriangle = 80,
    OpenTriangle = 81,
}

/// Drum pattern types (from collection analysis)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternType {
    Groove,     // Main beat pattern
    Fill,       // Transitional fill
    Intro,      // Song intro
    Ending,     // Song ending/outro
    Breakdown,  // Breakdown section
    Turnaround, // Turnaround pattern
    Sequence,   // Sequenced pattern
    OneShot,    // Single hit
}

/// Rhythmic feel classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RhythmicFeel {
    Straight, // Straight 8ths or 16ths
    Swing,    // Swing feel (jazz)
    Shuffle,  // Shuffle feel (blues/rock)
    Triplet,  // Triplet-based
    Half,     // Half-time feel
    Double,   // Double-time feel
    Pocket,   // Laid-back pocket
}

/// Drum technique classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrumTechnique {
    GhostNotes, // Low-velocity grace notes
    Linear,     // Linear drumming (no simultaneous notes)
    DoubleBass, // Double bass drum
    BlastBeat,  // Extreme metal blast beat
    Paradiddle, // Rudiment pattern
    Flam,       // Flam rudiment
    Roll,       // Drum roll
}

/// Time signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

/// Cymbal types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CymbalType {
    ClosedHat,
    PedalHat,
    OpenHat,
    Ride,
    RideBell,
    Crash,
    China,
    Splash,
}

/// Song structure sections
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SongStructure {
    Verse,
    Chorus,
    Bridge,
    Intro,
    Outro,
    PreChorus,
    Breakdown,
    Turnaround,
    MiddleEight,
}

/// Complete drum analysis results
#[derive(Debug, Clone)]
pub struct DrumAnalysis {
    pub is_drum_file: bool,
    pub drum_channel_detected: bool,
    pub drum_notes: HashMap<DrumNote, usize>,
    pub pattern_type: Option<PatternType>,
    pub rhythmic_feel: Option<RhythmicFeel>,
    pub time_signature: Option<TimeSignature>,
    pub bpm: Option<f64>,
    pub cymbal_types: Vec<CymbalType>,
    pub techniques: Vec<DrumTechnique>,
    pub song_structure: Option<SongStructure>,
}

// ============================================================================
// CORE ANALYSIS FUNCTIONS (Trusty Module - Pure, Testable)
// ============================================================================

/// Analyze MIDI file for drum-specific characteristics
///
/// **Trusty Module**: Pure function, no I/O, no side effects
pub fn analyze_drum_midi(midi_file: &MidiFile) -> DrumAnalysis {
    let drum_channel_detected = has_drum_channel(midi_file);
    let drum_notes = extract_drum_notes(midi_file);
    let cymbal_types = detect_cymbal_types(&drum_notes);
    let time_signature = extract_time_signature_from_meta(midi_file);
    let techniques = detect_techniques(midi_file, &drum_notes);

    DrumAnalysis {
        is_drum_file: drum_channel_detected || !drum_notes.is_empty(),
        drum_channel_detected,
        drum_notes,
        pattern_type: None,  // Set from filename analysis
        rhythmic_feel: None, // Set from filename analysis
        time_signature,
        bpm: None, // Set from filename analysis
        cymbal_types,
        techniques,
        song_structure: None, // Set from filename analysis
    }
}

/// Check if MIDI file uses channel 10 (GM drum channel)
///
/// **Trusty Module**: Pure function
pub fn has_drum_channel(midi_file: &MidiFile) -> bool {
    for track in &midi_file.tracks {
        for timed_event in &track.events {
            match timed_event.event {
                Event::NoteOn { channel, .. } | Event::NoteOff { channel, .. } => {
                    if channel == 9 {
                        // MIDI channel 10 = index 9
                        return true;
                    }
                },
                _ => {},
            }
        }
    }
    false
}

/// Extract all GM drum notes and their frequencies
///
/// **Trusty Module**: Pure function
pub fn extract_drum_notes(midi_file: &MidiFile) -> HashMap<DrumNote, usize> {
    let mut note_counts = HashMap::new();

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let Event::NoteOn { channel, note, velocity } = timed_event.event {
                // Channel 10 (index 9) or any note in GM drum range (35-81)
                if (channel == 9 || (35..=81).contains(&note)) && velocity > 0 {
                    if let Some(drum_note) = note_to_drum_type(note) {
                        *note_counts.entry(drum_note).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    note_counts
}

/// Map MIDI note number to GM drum type
///
/// **Trusty Module**: Pure function
pub fn note_to_drum_type(note: u8) -> Option<DrumNote> {
    match note {
        35 => Some(DrumNote::AcousticBassDrum),
        36 => Some(DrumNote::BassDrum1),
        37 => Some(DrumNote::SideStick),
        38 => Some(DrumNote::AcousticSnare),
        39 => Some(DrumNote::HandClap),
        40 => Some(DrumNote::ElectricSnare),
        41 => Some(DrumNote::LowFloorTom),
        42 => Some(DrumNote::ClosedHiHat),
        43 => Some(DrumNote::HighFloorTom),
        44 => Some(DrumNote::PedalHiHat),
        45 => Some(DrumNote::LowTom),
        46 => Some(DrumNote::OpenHiHat),
        47 => Some(DrumNote::LowMidTom),
        48 => Some(DrumNote::HighMidTom),
        49 => Some(DrumNote::CrashCymbal1),
        50 => Some(DrumNote::HighTom),
        51 => Some(DrumNote::RideCymbal1),
        52 => Some(DrumNote::ChineseCymbal),
        53 => Some(DrumNote::RideBell),
        54 => Some(DrumNote::Tambourine),
        55 => Some(DrumNote::SplashCymbal),
        56 => Some(DrumNote::Cowbell),
        57 => Some(DrumNote::CrashCymbal2),
        59 => Some(DrumNote::RideCymbal2),
        60 => Some(DrumNote::HighBongo),
        61 => Some(DrumNote::LowBongo),
        62 => Some(DrumNote::MuteHighConga),
        63 => Some(DrumNote::OpenHighConga),
        64 => Some(DrumNote::LowConga),
        65 => Some(DrumNote::HighTimbale),
        66 => Some(DrumNote::LowTimbale),
        67 => Some(DrumNote::HighAgogo),
        68 => Some(DrumNote::LowAgogo),
        69 => Some(DrumNote::Cabasa),
        70 => Some(DrumNote::Maracas),
        71 => Some(DrumNote::ShortWhistle),
        72 => Some(DrumNote::LongWhistle),
        73 => Some(DrumNote::ShortGuiro),
        74 => Some(DrumNote::LongGuiro),
        75 => Some(DrumNote::Claves),
        76 => Some(DrumNote::HighWoodBlock),
        77 => Some(DrumNote::LowWoodBlock),
        78 => Some(DrumNote::MuteCuica),
        79 => Some(DrumNote::OpenCuica),
        80 => Some(DrumNote::MuteTriangle),
        81 => Some(DrumNote::OpenTriangle),
        _ => None,
    }
}

/// Detect cymbal types from drum notes
///
/// **Trusty Module**: Pure function
pub fn detect_cymbal_types(drum_notes: &HashMap<DrumNote, usize>) -> Vec<CymbalType> {
    let mut cymbals = Vec::new();

    if drum_notes.contains_key(&DrumNote::ClosedHiHat) {
        cymbals.push(CymbalType::ClosedHat);
    }
    if drum_notes.contains_key(&DrumNote::PedalHiHat) {
        cymbals.push(CymbalType::PedalHat);
    }
    if drum_notes.contains_key(&DrumNote::OpenHiHat) {
        cymbals.push(CymbalType::OpenHat);
    }
    if drum_notes.contains_key(&DrumNote::RideCymbal1)
        || drum_notes.contains_key(&DrumNote::RideCymbal2)
    {
        cymbals.push(CymbalType::Ride);
    }
    if drum_notes.contains_key(&DrumNote::RideBell) {
        cymbals.push(CymbalType::RideBell);
    }
    if drum_notes.contains_key(&DrumNote::CrashCymbal1)
        || drum_notes.contains_key(&DrumNote::CrashCymbal2)
    {
        cymbals.push(CymbalType::Crash);
    }
    if drum_notes.contains_key(&DrumNote::ChineseCymbal) {
        cymbals.push(CymbalType::China);
    }
    if drum_notes.contains_key(&DrumNote::SplashCymbal) {
        cymbals.push(CymbalType::Splash);
    }

    cymbals
}

/// Extract time signature from MIDI meta events
///
/// **Trusty Module**: Pure function
pub fn extract_time_signature_from_meta(midi_file: &MidiFile) -> Option<TimeSignature> {
    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let Event::TimeSignature { numerator, denominator, .. } = timed_event.event {
                // MIDI stores denominator as power of 2 (e.g., 3 means 2^3 = 8)
                // Use saturating_pow to prevent overflow in debug mode
                return Some(TimeSignature {
                    numerator,
                    denominator: 2u8.saturating_pow(denominator as u32),
                });
            }
        }
    }
    None
}

/// Detect drum techniques from note patterns
///
/// **Trusty Module**: Pure function
pub fn detect_techniques(
    midi_file: &MidiFile,
    drum_notes: &HashMap<DrumNote, usize>,
) -> Vec<DrumTechnique> {
    let mut techniques = Vec::new();

    // Ghost notes: Check for many low-velocity snare hits
    if has_ghost_notes(midi_file) {
        techniques.push(DrumTechnique::GhostNotes);
    }

    // Double bass: High count of kick notes (combine both kick types)
    let kick_count_1 = drum_notes.get(&DrumNote::BassDrum1).copied().unwrap_or(0);
    let kick_count_2 = drum_notes.get(&DrumNote::AcousticBassDrum).copied().unwrap_or(0);
    let kick_count = kick_count_1 + kick_count_2;

    if kick_count > 100 {
        // Threshold for double-bass
        techniques.push(DrumTechnique::DoubleBass);
    }

    techniques
}

/// Check for ghost notes (low-velocity snare hits)
///
/// **Trusty Module**: Pure function
fn has_ghost_notes(midi_file: &MidiFile) -> bool {
    let mut ghost_count = 0;
    let mut total_snare = 0;

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let Event::NoteOn { note, velocity, .. } = timed_event.event {
                if note == 38 || note == 40 {
                    // Snare notes
                    total_snare += 1;
                    if velocity > 0 && velocity < 40 {
                        // Ghost note threshold
                        ghost_count += 1;
                    }
                }
            }
        }
    }

    total_snare > 0 && (ghost_count as f64 / total_snare as f64) >= 0.3
}

// ============================================================================
// FILENAME/PATH METADATA EXTRACTION (Trusty Module)
// ============================================================================

/// Extract time signature from filename or path
///
/// **Trusty Module**: Pure function
///
/// Patterns:
/// - "9-8 Straight Kick.mid"
/// - "Jazz Parts 2/12-8 Swing/..."
/// - "6-8_groove.mid"
pub fn extract_time_signature_from_path(file_path: &str, file_name: &str) -> Option<TimeSignature> {
    let combined = format!("{}/{}", file_path, file_name);

    // Common time signature patterns
    let patterns = [
        ("12-8", (12, 8)),
        ("12/8", (12, 8)),
        ("9-8", (9, 8)),
        ("9/8", (9, 8)),
        ("6-8", (6, 8)),
        ("6/8", (6, 8)),
        ("7-8", (7, 8)),
        ("7/8", (7, 8)),
        ("11-8", (11, 8)),
        ("11/8", (11, 8)),
        ("15-8", (15, 8)),
        ("15/8", (15, 8)),
        ("7-4", (7, 4)),
        ("7/4", (7, 4)),
        ("5-4", (5, 4)),
        ("5/4", (5, 4)),
        ("3-4", (3, 4)),
        ("3/4", (3, 4)),
        ("4-4", (4, 4)),
        ("4/4", (4, 4)),
        ("2-4", (2, 4)),
        ("2/4", (2, 4)),
    ];

    for (pattern, (num, denom)) in &patterns {
        if combined.contains(pattern) {
            return Some(TimeSignature { numerator: *num, denominator: *denom });
        }
    }

    None
}

/// Extract BPM from filename
///
/// **Trusty Module**: Pure function
///
/// Patterns:
/// - "174_Gmin_Bass.mid"
/// - "140bpm_Kick.mid"
/// - "120 BPM Groove.mid"
/// - "jazz_136_swing.mid"
pub fn extract_bpm_from_filename(file_name: &str) -> Option<f64> {
    let name_lower = file_name.to_lowercase();

    // Pattern 1: "XXXbpm" or "XXX bpm"
    if let Some(pos) = name_lower.find("bpm") {
        let before_bpm = &name_lower[..pos].trim();
        if let Some(num_start) = before_bpm.rfind(|c: char| !c.is_ascii_digit()) {
            if let Ok(bpm) = before_bpm[num_start + 1..].parse::<f64>() {
                if (30.0..=300.0).contains(&bpm) {
                    return Some(bpm);
                }
            }
        }
    }

    // Pattern 2: "XXX_" at start
    if file_name.len() >= 4 && file_name.chars().nth(3) == Some('_') {
        if let Ok(bpm) = file_name[..3].parse::<f64>() {
            if (30.0..=300.0).contains(&bpm) {
                return Some(bpm);
            }
        }
    }

    // Pattern 3: "XXX " at start (space after number)
    if file_name.len() >= 4 && file_name.chars().nth(3) == Some(' ') {
        if let Ok(bpm) = file_name[..3].parse::<f64>() {
            if (30.0..=300.0).contains(&bpm) {
                return Some(bpm);
            }
        }
    }

    // Pattern 4: "_XXX_" anywhere in filename (e.g., "jazz_136_swing.mid")
    // Split by underscore and check each segment
    for segment in file_name.split('_') {
        // Check if segment is exactly 3 digits
        if segment.len() == 3 && segment.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(bpm) = segment.parse::<f64>() {
                if (30.0..=300.0).contains(&bpm) {
                    return Some(bpm);
                }
            }
        }
    }

    None
}

/// Extract pattern type from filename/path
///
/// **Trusty Module**: Pure function
pub fn extract_pattern_type(file_path: &str, file_name: &str) -> Option<PatternType> {
    let combined = format!("{}/{}", file_path, file_name).to_lowercase();

    if combined.contains("groove") || combined.contains(" gr ") {
        Some(PatternType::Groove)
    } else if combined.contains("fill") {
        Some(PatternType::Fill)
    } else if combined.contains("intro") {
        Some(PatternType::Intro)
    } else if combined.contains("ending") || combined.contains("outro") {
        Some(PatternType::Ending)
    } else if combined.contains("breakdown") || combined.contains("bkdn") {
        Some(PatternType::Breakdown)
    } else if combined.contains("turnaround") || combined.contains(" ta ") {
        Some(PatternType::Turnaround)
    } else if combined.contains("sequence") || combined.contains(" seq ") {
        Some(PatternType::Sequence)
    } else if combined.contains("one-shot") || combined.contains("oneshot") {
        Some(PatternType::OneShot)
    } else {
        None
    }
}

/// Extract rhythmic feel from filename/path
///
/// **Trusty Module**: Pure function
pub fn extract_rhythmic_feel(file_path: &str, file_name: &str) -> Option<RhythmicFeel> {
    let combined = format!("{}/{}", file_path, file_name).to_lowercase();

    if combined.contains("swing") {
        Some(RhythmicFeel::Swing)
    } else if combined.contains("shuffle") {
        Some(RhythmicFeel::Shuffle)
    } else if combined.contains("straight") {
        Some(RhythmicFeel::Straight)
    } else if combined.contains("triplet") {
        Some(RhythmicFeel::Triplet)
    } else if combined.contains("half-time") || combined.contains("halftime") {
        Some(RhythmicFeel::Half)
    } else if combined.contains("double-time") || combined.contains("doubletime") {
        Some(RhythmicFeel::Double)
    } else if combined.contains("pocket") {
        Some(RhythmicFeel::Pocket)
    } else {
        None
    }
}

/// Extract song structure from filename/path
///
/// **Trusty Module**: Pure function
pub fn extract_song_structure(file_path: &str, file_name: &str) -> Option<SongStructure> {
    let combined = format!("{}/{}", file_path, file_name).to_lowercase();

    if combined.contains("verse") {
        Some(SongStructure::Verse)
    } else if combined.contains("chorus") {
        Some(SongStructure::Chorus)
    } else if combined.contains("bridge") {
        Some(SongStructure::Bridge)
    } else if combined.contains("intro") {
        Some(SongStructure::Intro)
    } else if combined.contains("outro") {
        Some(SongStructure::Outro)
    } else if combined.contains("pre-chorus") || combined.contains("prechorus") {
        Some(SongStructure::PreChorus)
    } else if combined.contains("breakdown") {
        Some(SongStructure::Breakdown)
    } else if combined.contains("turnaround") {
        Some(SongStructure::Turnaround)
    } else if combined.contains("middle-8") || combined.contains("mid-8") {
        Some(SongStructure::MiddleEight)
    } else {
        None
    }
}

// Helper conversion functions for tag generation
fn drum_note_to_tag(drum_note: DrumNote) -> (&'static str, &'static str) {
    match drum_note {
        DrumNote::AcousticBassDrum | DrumNote::BassDrum1 => ("kick", "instrument"),
        DrumNote::AcousticSnare | DrumNote::ElectricSnare => ("snare", "instrument"),
        DrumNote::ClosedHiHat | DrumNote::OpenHiHat | DrumNote::PedalHiHat => {
            ("hihat", "instrument")
        },
        DrumNote::LowFloorTom
        | DrumNote::HighFloorTom
        | DrumNote::LowTom
        | DrumNote::LowMidTom
        | DrumNote::HighMidTom
        | DrumNote::HighTom => ("toms", "instrument"),
        DrumNote::CrashCymbal1 | DrumNote::CrashCymbal2 => ("crash", "instrument"),
        DrumNote::RideCymbal1 | DrumNote::RideCymbal2 => ("ride", "instrument"),
        DrumNote::ChineseCymbal => ("china", "instrument"),
        DrumNote::SplashCymbal => ("splash", "instrument"),
        DrumNote::RideBell => ("ride-bell", "instrument"),
        DrumNote::SideStick => ("sidestick", "instrument"),
        DrumNote::HandClap => ("clap", "instrument"),
        DrumNote::Cowbell => ("cowbell", "instrument"),
        DrumNote::Tambourine => ("tambourine", "instrument"),
        DrumNote::HighBongo | DrumNote::LowBongo => ("bongo", "instrument"),
        DrumNote::MuteHighConga | DrumNote::OpenHighConga | DrumNote::LowConga => {
            ("conga", "instrument")
        },
        _ => ("percussion", "instrument"),
    }
}

fn cymbal_to_tag_name(cymbal: &CymbalType) -> &'static str {
    match cymbal {
        CymbalType::ClosedHat => "closed-hat",
        CymbalType::PedalHat => "pedal-hat",
        CymbalType::OpenHat => "open-hat",
        CymbalType::Ride => "ride",
        CymbalType::RideBell => "ride-bell",
        CymbalType::Crash => "crash",
        CymbalType::China => "china",
        CymbalType::Splash => "splash",
    }
}

fn pattern_type_to_tag(pattern: &PatternType) -> &'static str {
    match pattern {
        PatternType::Groove => "groove",
        PatternType::Fill => "fill",
        PatternType::Intro => "intro",
        PatternType::Ending => "ending",
        PatternType::Breakdown => "breakdown",
        PatternType::Turnaround => "turnaround",
        PatternType::Sequence => "sequence",
        PatternType::OneShot => "one-shot",
    }
}

fn rhythmic_feel_to_tag(feel: &RhythmicFeel) -> &'static str {
    match feel {
        RhythmicFeel::Straight => "straight",
        RhythmicFeel::Swing => "swing",
        RhythmicFeel::Shuffle => "shuffle",
        RhythmicFeel::Triplet => "triplet",
        RhythmicFeel::Half => "half-time",
        RhythmicFeel::Double => "double-time",
        RhythmicFeel::Pocket => "pocket",
    }
}

fn song_structure_to_tag(structure: &SongStructure) -> &'static str {
    match structure {
        SongStructure::Verse => "verse",
        SongStructure::Chorus => "chorus",
        SongStructure::Bridge => "bridge",
        SongStructure::Intro => "intro",
        SongStructure::Outro => "outro",
        SongStructure::PreChorus => "pre-chorus",
        SongStructure::Breakdown => "breakdown",
        SongStructure::Turnaround => "turnaround",
        SongStructure::MiddleEight => "middle-8",
    }
}

fn technique_to_tag_name(technique: &DrumTechnique) -> &'static str {
    match technique {
        DrumTechnique::GhostNotes => "ghost-notes",
        DrumTechnique::Linear => "linear",
        DrumTechnique::DoubleBass => "double-bass",
        DrumTechnique::BlastBeat => "blast-beat",
        DrumTechnique::Paradiddle => "paradiddle",
        DrumTechnique::Flam => "flam",
        DrumTechnique::Roll => "roll",
    }
}

/// Generate drum-specific tags from analysis results
///
/// **Trusty Module**: Pure function
///
/// Returns tags compatible with AutoTagger Tag structure
pub fn generate_drum_tags(analysis: &DrumAnalysis, file_path: &str, file_name: &str) -> Vec<Tag> {
    let mut tags = Vec::new();

    // 1. Drum detection tag
    if analysis.is_drum_file {
        tags.push(Tag::with_metadata(
            "drums".to_string(),
            Some("instrument".to_string()),
            0.90,
            20,
            "midi_channel_10".to_string(),
        ));
    }

    // 2. Specific drum instrument tags
    for (drum_note, count) in &analysis.drum_notes {
        if *count > 5 {
            // Threshold for significant presence
            let (tag_name, category) = drum_note_to_tag(*drum_note);
            tags.push(Tag::with_metadata(
                tag_name.to_string(),
                Some(category.to_string()),
                0.85,
                20,
                "midi_drum_notes".to_string(),
            ));
        }
    }

    // 3. Cymbal type tags
    for cymbal in &analysis.cymbal_types {
        tags.push(Tag::with_metadata(
            cymbal_to_tag_name(cymbal).to_string(),
            Some("cymbal-type".to_string()),
            0.85,
            25,
            "midi_drum_notes".to_string(),
        ));
    }

    // 4. Time signature tags
    if let Some(ref time_sig) = analysis.time_signature {
        let ts_tag = format!("{}-{}", time_sig.numerator, time_sig.denominator);
        tags.push(Tag::with_metadata(
            ts_tag,
            Some("time-signature".to_string()),
            0.90,
            35,
            "midi_meta_event".to_string(),
        ));

        // Add meter category tags
        if [6, 9, 12].contains(&time_sig.numerator) && time_sig.denominator == 8 {
            tags.push(Tag::with_metadata(
                "compound-meter".to_string(),
                Some("rhythm-style".to_string()),
                0.80,
                40,
                "time_sig_derived".to_string(),
            ));
        }
    }

    // 5. Pattern type tags (from filename)
    if let Some(pattern) = extract_pattern_type(file_path, file_name) {
        tags.push(Tag::with_metadata(
            pattern_type_to_tag(&pattern).to_string(),
            Some("pattern-type".to_string()),
            0.85,
            30,
            "filename_exact".to_string(),
        ));
    }

    // 6. Rhythmic feel tags (from filename)
    if let Some(feel) = extract_rhythmic_feel(file_path, file_name) {
        tags.push(Tag::with_metadata(
            rhythmic_feel_to_tag(&feel).to_string(),
            Some("rhythm-feel".to_string()),
            0.85,
            40,
            "filename_exact".to_string(),
        ));
    }

    // 7. Song structure tags (from filename)
    if let Some(structure) = extract_song_structure(file_path, file_name) {
        tags.push(Tag::with_metadata(
            song_structure_to_tag(&structure).to_string(),
            Some("structure".to_string()),
            0.85,
            80,
            "filename_exact".to_string(),
        ));
    }

    // 8. BPM tags (from filename)
    if let Some(bpm) = extract_bpm_from_filename(file_name) {
        let bpm_rounded = bpm.round() as i32;
        tags.push(Tag::with_metadata(
            bpm_rounded.to_string(),
            Some("tempo".to_string()),
            0.85,
            50,
            "filename_bpm".to_string(),
        ));
    }

    // 9. Technique tags
    for technique in &analysis.techniques {
        tags.push(Tag::with_metadata(
            technique_to_tag_name(technique).to_string(),
            Some("technique".to_string()),
            0.75,
            45,
            "midi_pattern_analysis".to_string(),
        ));
    }

    tags
}
