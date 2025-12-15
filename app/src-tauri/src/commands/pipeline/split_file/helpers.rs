//! Helper functions for track splitting

use crate::core::splitting::SplitTrack;

/// Generate filename for a split track: `{base}_track_{num:02}_{instrument}.mid`
pub fn generate_split_filename(base_filename: &str, split_track: &SplitTrack) -> String {
    let base = sanitize_filename(base_filename);
    let track_num = format!("{:02}", split_track.track_number);

    // Build suffix: prefer instrument, fall back to track name, then just number
    let suffix = if let Some(ref instrument) = split_track.instrument {
        sanitize_filename(instrument)
    } else if let Some(ref track_name) = split_track.track_name {
        sanitize_filename(track_name)
    } else {
        String::new()
    };

    if suffix.is_empty() {
        format!("{}_track_{}.mid", base, track_num)
    } else {
        format!("{}_track_{}_{}.mid", base, track_num, suffix)
    }
}

/// Sanitize string for use in filenames (replaces unsafe chars, collapses underscores).
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            // Replace spaces with underscores
            ' ' => '_',
            // Remove problematic characters
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            // Keep alphanumeric, underscore, hyphen, period, parentheses
            c if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '(' || c == ')' => {
                c
            }
            // Replace everything else with underscore
            _ => '_',
        })
        .collect::<String>()
        // Collapse multiple underscores
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

/// Extract time signature from MIDI events (e.g., "4-4" for 4/4 time).
pub fn extract_time_signature_from_midi(
    midi: &crate::core::midi::types::MidiFile,
) -> Option<String> {
    use crate::core::midi::types::Event;

    // Search all tracks for TimeSignature event
    for track in &midi.tracks {
        for timed_event in &track.events {
            if let Event::TimeSignature { numerator, denominator, .. } = &timed_event.event {
                // Convert denominator from power-of-2 format (e.g., 2 = quarter note = 4)
                let denom_value = 2_u8.pow(*denominator as u32);
                return Some(format!("{}-{}", numerator, denom_value));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_spaces() {
        assert_eq!(sanitize_filename("Piano Track"), "Piano_Track");
        assert_eq!(sanitize_filename("My Song Name"), "My_Song_Name");
    }

    #[test]
    fn test_sanitize_filename_special_chars() {
        assert_eq!(sanitize_filename("Track: 1"), "Track_1");
        assert_eq!(sanitize_filename("Bass/Guitar"), "Bass_Guitar");
        assert_eq!(sanitize_filename("Lead (Synth)"), "Lead_(Synth)");
        assert_eq!(sanitize_filename("File*Name?"), "File_Name");
        assert_eq!(sanitize_filename("Path\\To\\File"), "Path_To_File");
    }

    #[test]
    fn test_sanitize_filename_multiple_underscores() {
        assert_eq!(sanitize_filename("Track___1"), "Track_1");
        assert_eq!(sanitize_filename("__Piano__"), "Piano");
        assert_eq!(sanitize_filename("A___B___C"), "A_B_C");
    }

    #[test]
    fn test_sanitize_filename_edge_cases() {
        assert_eq!(sanitize_filename(""), "");
        assert_eq!(sanitize_filename("   "), "");
        assert_eq!(sanitize_filename("___"), "");
        assert_eq!(sanitize_filename("ValidName123"), "ValidName123");
    }

    #[test]
    fn test_sanitize_filename_unicode() {
        // Keep alphanumeric Unicode (includes accented characters)
        assert_eq!(sanitize_filename("Café"), "Café");
        assert_eq!(sanitize_filename("Track♪1"), "Track_1");
    }

    #[test]
    fn test_generate_split_filename_with_instrument() {
        let track = SplitTrack {
            track_number: 1,
            track_name: Some("Piano Part".to_string()),
            channel: Some(0),
            instrument: Some("Acoustic Grand Piano".to_string()),
            note_count: 100,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("my_song", &track);
        assert_eq!(filename, "my_song_track_01_Acoustic_Grand_Piano.mid");
    }

    #[test]
    fn test_generate_split_filename_with_track_name_only() {
        let track = SplitTrack {
            track_number: 2,
            track_name: Some("Bass Line".to_string()),
            channel: Some(1),
            instrument: None,
            note_count: 50,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("song", &track);
        assert_eq!(filename, "song_track_02_Bass_Line.mid");
    }

    #[test]
    fn test_generate_split_filename_no_metadata() {
        let track = SplitTrack {
            track_number: 0,
            track_name: None,
            channel: None,
            instrument: None,
            note_count: 10,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("minimal", &track);
        assert_eq!(filename, "minimal_track_00.mid");
    }

    #[test]
    fn test_generate_split_filename_sanitizes_base() {
        let track = SplitTrack {
            track_number: 5,
            track_name: None,
            channel: None,
            instrument: Some("Guitar".to_string()),
            note_count: 75,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("My/Bad\\Filename:1", &track);
        assert_eq!(filename, "My_Bad_Filename_1_track_05_Guitar.mid");
    }

    #[test]
    fn test_generate_split_filename_sanitizes_instrument() {
        let track = SplitTrack {
            track_number: 3,
            track_name: None,
            channel: None,
            instrument: Some("Electric Piano (DX7)".to_string()),
            note_count: 80,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("track", &track);
        assert_eq!(filename, "track_track_03_Electric_Piano_(DX7).mid");
    }

    #[test]
    fn test_generate_split_filename_high_track_numbers() {
        let track = SplitTrack {
            track_number: 99,
            track_name: None,
            channel: None,
            instrument: Some("Drums".to_string()),
            note_count: 200,
            midi_bytes: vec![],
        };

        let filename = generate_split_filename("orchestra", &track);
        assert_eq!(filename, "orchestra_track_99_Drums.mid");
    }
}
