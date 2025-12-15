/// Auto-tagging functionality wrapper
///
/// This is a simplified wrapper for the shared library.
/// The full implementation lives in the Pipeline component at
/// `pipeline/src-tauri/src/core/analysis/auto_tagger.rs`
///
/// Generates tags based on MIDI file content analysis including:
/// - Instrument detection from GM program changes
/// - Note density and patterns
/// - Tempo characteristics (if BPM detected)
/// - Channel usage patterns
///
/// # Arguments
/// * `midi_file` - Parsed MIDI file structure
///
/// # Returns
/// * `Vec<String>` - List of detected tags (e.g., "drums", "piano", "fast", "melodic")
///
/// # Examples
///
/// ```ignore
/// use midi_app::core::midi::MidiFile;
/// use midi_app::core::analysis::auto_tagger::generate_tags;
///
/// let midi_file = MidiFile::parse(&data)?;
/// let tags = generate_tags(&midi_file);
/// println!("Tags: {:?}", tags);
/// ```
pub fn generate_tags(midi_file: &crate::core::midi::MidiFile) -> Vec<String> {
    let mut tags = Vec::new();

    // Track instruments detected from MIDI program changes
    let mut instruments_seen = std::collections::HashSet::new();
    let mut note_count = 0u32;

    // Analyze all tracks
    for track in midi_file.tracks.iter() {
        for timed_event in &track.events {
            match &timed_event.event {
                crate::core::midi::Event::ProgramChange { channel, program } => {
                    // Check if this is the drum channel (channel 10 in GM)
                    if *channel == 9 {
                        instruments_seen.insert("drums");
                    } else {
                        // Map GM program numbers to instrument names
                        let instrument = map_gm_program_to_instrument(*program);
                        instruments_seen.insert(instrument);
                    }
                },
                crate::core::midi::Event::NoteOn { channel, .. } => {
                    // Channel 10 (9 in 0-indexed) is drums in GM
                    if *channel == 9 {
                        instruments_seen.insert("drums");
                    }
                    note_count = note_count.saturating_add(1);
                },
                crate::core::midi::Event::Text { text_type: _, text } => {
                    // Extract genre hints from track text in any track
                    let text_lower = text.to_lowercase();

                    // Extract genre hints from track text
                    if text_lower.contains("rock") {
                        tags.push("rock".to_string());
                    }
                    if text_lower.contains("jazz") {
                        tags.push("jazz".to_string());
                    }
                    if text_lower.contains("classical") {
                        tags.push("classical".to_string());
                    }
                    if text_lower.contains("electronic") {
                        tags.push("electronic".to_string());
                    }
                },
                _ => {},
            }
        }
    }

    // Add instrument tags
    for instrument in instruments_seen {
        tags.push(instrument.to_string());
    }

    // Add complexity tags based on note density
    if note_count > 1000 {
        tags.push("dense".to_string());
    } else if note_count > 500 {
        tags.push("moderate".to_string());
    } else if note_count > 0 {
        tags.push("sparse".to_string());
    }

    // Add track count tags
    let track_count = midi_file.tracks.len();
    if track_count > 10 {
        tags.push("multi-track".to_string());
    } else if track_count > 1 {
        tags.push("layered".to_string());
    } else {
        tags.push("single-track".to_string());
    }

    // Add tempo tags if we can determine BPM
    // Note: This is simplified - full BPM detection is in Pipeline
    if let Some(tempo) = detect_average_tempo(midi_file) {
        if tempo > 140.0 {
            tags.push("fast".to_string());
        } else if tempo > 100.0 {
            tags.push("moderate-tempo".to_string());
        } else if tempo > 60.0 {
            tags.push("slow".to_string());
        }
    }

    tags
}

/// Map GM program numbers to instrument names
fn map_gm_program_to_instrument(program: u8) -> &'static str {
    match program {
        0..=7 => "piano",
        8..=15 => "chromatic-percussion",
        16..=23 => "organ",
        24..=31 => "guitar",
        32..=39 => "bass",
        40..=47 => "strings",
        48..=55 => "ensemble",
        56..=63 => "brass",
        64..=71 => "reed",
        72..=79 => "pipe",
        80..=87 => "synth-lead",
        88..=95 => "synth-pad",
        96..=103 => "synth-effects",
        104..=111 => "ethnic",
        112..=119 => "percussive",
        120..=127 => "sound-effects",
        _ => "unknown",
    }
}

/// Detect average tempo from MIDI tempo events
fn detect_average_tempo(midi_file: &crate::core::midi::MidiFile) -> Option<f64> {
    let mut tempo_sum = 0.0;
    let mut tempo_count = 0;

    for track in &midi_file.tracks {
        for timed_event in &track.events {
            if let crate::core::midi::Event::TempoChange { microseconds_per_quarter } =
                &timed_event.event
            {
                // Convert microseconds per quarter note to BPM
                let bpm = 60_000_000.0 / (*microseconds_per_quarter as f64);
                tempo_sum += bpm;
                tempo_count += 1;
            }
        }
    }

    if tempo_count > 0 {
        Some(tempo_sum / tempo_count as f64)
    } else {
        None
    }
}
