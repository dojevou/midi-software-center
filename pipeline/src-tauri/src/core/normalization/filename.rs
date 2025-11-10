
/// Filename normalization utilities for MIDI files.
///
/// This module provides pure functions for normalizing MIDI filenames,
/// specifically converting `.midi` extensions to `.mid` and replacing
/// spaces with underscores.
///
/// # Archetype: Trusty Module
///
/// This is a pure logic module with NO side effects:
/// - ❌ NO file I/O
/// - ❌ NO database access
/// - ❌ NO printing/logging
/// - ✅ Pure functions only
/// - ✅ Comprehensive tests
///
/// # Examples
///
/// ```
/// use pipeline::core::normalization::filename::normalize_midi_filename;
///
/// // Extension normalization
/// let normalized = normalize_midi_filename("song.midi");
/// assert_eq!(normalized, "song.mid");
///
/// // Space replacement
/// let normalized = normalize_midi_filename("my song.mid");
/// assert_eq!(normalized, "my_song.mid");
///
/// // Both transformations
/// let normalized = normalize_midi_filename("Cool Track.midi");
/// assert_eq!(normalized, "Cool_Track.mid");
/// ```
/// Normalize a MIDI filename by converting `.midi` extension to `.mid`
/// and replacing all spaces with underscores.
///
/// This function performs two normalizations:
/// 1. Converts `.midi` extension to `.mid` (case-insensitive)
/// 2. Replaces all spaces ` ` with underscores `_`
///
/// # Arguments
///
/// * `filename` - The filename to normalize (without path, just the filename)
///
/// # Returns
///
/// A new `String` with the normalized filename.
///
/// # Examples
///
/// ```
/// # use pipeline::core::normalization::filename::normalize_midi_filename;
/// // Convert .midi to .mid
/// assert_eq!(normalize_midi_filename("song.midi"), "song.mid");
///
/// // Replace spaces with underscores
/// assert_eq!(normalize_midi_filename("my song.mid"), "my_song.mid");
///
/// // Both transformations
/// assert_eq!(normalize_midi_filename("Cool Track.midi"), "Cool_Track.mid");
///
/// // Case insensitive extension
/// assert_eq!(normalize_midi_filename("song.MIDI"), "song.mid");
///
/// // Multiple spaces become multiple underscores
/// assert_eq!(normalize_midi_filename("jazz  blues.midi"), "jazz__blues.mid");
///
/// // Already normalized - unchanged
/// assert_eq!(normalize_midi_filename("song.mid"), "song.mid");
///
/// // Preserve other special characters
/// assert_eq!(normalize_midi_filename("my-song_123.midi"), "my-song_123.mid");
/// ```
pub fn normalize_midi_filename(filename: &str) -> String {
    // Step 1: Normalize extension (.midi -> .mid)
    let extension_normalized = if let Some(name_without_ext) = strip_midi_extension(filename) {
        format!("{}.mid", name_without_ext)
    } else {
        filename.to_string()
    };

    // Step 2: Replace all spaces with underscores
    extension_normalized.replace(' ', "_")
}

/// Check if a filename needs normalization.
///
/// Returns `true` if the filename:
/// - Has a `.midi` extension (case-insensitive) that should be converted to `.mid`, OR
/// - Contains spaces that should be replaced with underscores
///
/// # Arguments
///
/// * `filename` - The filename to check
///
/// # Returns
///
/// `true` if the filename needs normalization, `false` otherwise.
///
/// # Examples
///
/// ```
/// # use pipeline::core::normalization::filename::needs_normalization;
/// // Has .midi extension
/// assert!(needs_normalization("song.midi"));
/// assert!(needs_normalization("song.MIDI"));
///
/// // Has spaces
/// assert!(needs_normalization("my song.mid"));
/// assert!(needs_normalization("Cool Track.mp3"));
///
/// // Has both
/// assert!(needs_normalization("my song.midi"));
///
/// // Already normalized
/// assert!(!needs_normalization("song.mid"));
/// assert!(!needs_normalization("my_song.mid"));
/// ```
pub fn needs_normalization(filename: &str) -> bool {
    // Check if has .midi extension OR contains spaces
    strip_midi_extension(filename).is_some() || filename.contains(' ')
}

/// Strip `.midi` extension from filename if present (case-insensitive).
///
/// This is an internal helper function that checks if a filename ends with
/// `.midi` (in any case combination) and returns the filename without the
/// extension if found.
///
/// # Arguments
///
/// * `filename` - The filename to process
///
/// # Returns
///
/// `Some(&str)` containing the filename without `.midi` extension if found,
/// `None` otherwise.
fn strip_midi_extension(filename: &str) -> Option<&str> {
    // Check for .midi extension (case-insensitive)
    // We need to check the last 5 characters (.midi)
    if filename.len() > 5 {
        let extension_start = filename.len() - 5;
        let potential_ext = &filename[extension_start..];

        if potential_ext.eq_ignore_ascii_case(".midi") {
            return Some(&filename[..extension_start]);
        }
    } else if filename.len() == 5 {
        // Special case: filename is exactly ".midi"
        if filename.eq_ignore_ascii_case(".midi") {
            return Some("");
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic functionality tests
    #[test]
    fn test_normalize_midi_to_mid() {
        assert_eq!(normalize_midi_filename("song.midi"), "song.mid");
    }

    #[test]
    fn test_already_mid_unchanged() {
        assert_eq!(normalize_midi_filename("song.mid"), "song.mid");
    }

    #[test]
    fn test_uppercase_midi() {
        assert_eq!(normalize_midi_filename("song.MIDI"), "song.mid");
    }

    #[test]
    fn test_mixed_case_midi() {
        assert_eq!(normalize_midi_filename("song.MiDi"), "song.mid");
        assert_eq!(normalize_midi_filename("song.mIdI"), "song.mid");
        assert_eq!(normalize_midi_filename("song.Midi"), "song.mid");
    }

    #[test]
    fn test_uppercase_mid_unchanged() {
        // .MID is already the correct length, just different case
        // We should preserve it as-is since it's not .midi
        assert_eq!(normalize_midi_filename("song.MID"), "song.MID");
    }

    // Space replacement tests
    #[test]
    fn test_space_replacement() {
        assert_eq!(normalize_midi_filename("my song.mid"), "my_song.mid");
    }

    #[test]
    fn test_multiple_spaces() {
        assert_eq!(normalize_midi_filename("my  song.midi"), "my__song.mid");
    }

    #[test]
    fn test_space_and_midi_extension() {
        assert_eq!(normalize_midi_filename("Cool Track.midi"), "Cool_Track.mid");
    }

    #[test]
    fn test_no_spaces_no_change() {
        assert_eq!(normalize_midi_filename("song.mid"), "song.mid");
    }

    #[test]
    fn test_spaces_in_complex_filename() {
        assert_eq!(
            normalize_midi_filename("120bpm Cmaj Scale.midi"),
            "120bpm_Cmaj_Scale.mid"
        );
    }

    #[test]
    fn test_needs_normalization_with_spaces() {
        assert!(needs_normalization("my song.mid"));
        assert!(needs_normalization("song.midi"));
        assert!(needs_normalization("my song.midi"));
        assert!(!needs_normalization("song.mid"));
        assert!(!needs_normalization("my_song.mid"));
    }

    // Special characters and spaces
    #[test]
    fn test_filename_with_spaces() {
        assert_eq!(
            normalize_midi_filename("my song name.midi"),
            "my_song_name.mid"
        );
    }

    #[test]
    fn test_filename_with_special_chars() {
        assert_eq!(
            normalize_midi_filename("song-123_test@example.midi"),
            "song-123_test@example.mid"
        );
    }

    #[test]
    fn test_filename_with_unicode() {
        assert_eq!(normalize_midi_filename("歌曲.midi"), "歌曲.mid");
        assert_eq!(normalize_midi_filename("café-song.midi"), "café-song.mid");
    }

    #[test]
    fn test_spaces_with_unicode() {
        assert_eq!(normalize_midi_filename("café song.midi"), "café_song.mid");
    }

    // Multiple dots
    #[test]
    fn test_multiple_dots_in_filename() {
        assert_eq!(
            normalize_midi_filename("my.song.title.midi"),
            "my.song.title.mid"
        );
    }

    #[test]
    fn test_dots_preserved() {
        assert_eq!(
            normalize_midi_filename("song.v2.final.midi"),
            "song.v2.final.mid"
        );
    }

    #[test]
    fn test_dots_and_spaces() {
        assert_eq!(normalize_midi_filename("my.song v2.midi"), "my.song_v2.mid");
    }

    // Edge cases
    #[test]
    fn test_no_extension() {
        assert_eq!(normalize_midi_filename("song"), "song");
    }

    #[test]
    fn test_no_extension_with_spaces() {
        assert_eq!(normalize_midi_filename("my song"), "my_song");
    }

    #[test]
    fn test_wrong_extension() {
        assert_eq!(normalize_midi_filename("song.mp3"), "song.mp3");
        assert_eq!(normalize_midi_filename("song.wav"), "song.wav");
        assert_eq!(normalize_midi_filename("song.txt"), "song.txt");
    }

    #[test]
    fn test_wrong_extension_with_spaces() {
        assert_eq!(normalize_midi_filename("my song.mp3"), "my_song.mp3");
    }

    #[test]
    fn test_midi_not_at_end() {
        // .midi in the middle of filename shouldn't be changed
        assert_eq!(
            normalize_midi_filename("song.midi.backup"),
            "song.midi.backup"
        );
        assert_eq!(normalize_midi_filename("midi.txt"), "midi.txt");
    }

    #[test]
    fn test_just_extension() {
        // Edge case: filename is just ".midi"
        assert_eq!(normalize_midi_filename(".midi"), ".mid");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(normalize_midi_filename(""), "");
    }

    #[test]
    fn test_very_short_filename() {
        assert_eq!(normalize_midi_filename("a.midi"), "a.mid");
        assert_eq!(normalize_midi_filename("ab.midi"), "ab.mid");
    }

    #[test]
    fn test_very_short_filename_with_space() {
        assert_eq!(normalize_midi_filename("a b.midi"), "a_b.mid");
    }

    #[test]
    fn test_only_spaces() {
        assert_eq!(normalize_midi_filename("   .midi"), "___.mid");
        assert_eq!(normalize_midi_filename(" "), "_");
    }

    #[test]
    fn test_leading_trailing_spaces() {
        assert_eq!(normalize_midi_filename(" song.midi"), "_song.mid");
        assert_eq!(normalize_midi_filename("song .midi"), "song_.mid");
        assert_eq!(normalize_midi_filename(" song .midi"), "_song_.mid");
    }

    // needs_normalization tests
    #[test]
    fn test_needs_normalization_true() {
        // Has .midi extension
        assert!(needs_normalization("song.midi"));
        assert!(needs_normalization("song.MIDI"));
        assert!(needs_normalization("song.MiDi"));
        assert!(needs_normalization("my.song.midi"));

        // Has spaces
        assert!(needs_normalization("my song.mid"));
        assert!(needs_normalization("Cool Track.mp3"));

        // Has both
        assert!(needs_normalization("my song.midi"));
        assert!(needs_normalization("Cool Track.MIDI"));
    }

    #[test]
    fn test_needs_normalization_false() {
        assert!(!needs_normalization("song.mid"));
        assert!(!needs_normalization("song.MID"));
        assert!(!needs_normalization("song.mp3"));
        assert!(!needs_normalization("song"));
        assert!(!needs_normalization(""));
        assert!(!needs_normalization("song.midi.backup"));
        assert!(!needs_normalization("my_song.mid"));
        assert!(!needs_normalization("my-song.mid"));
    }

    // strip_midi_extension tests
    #[test]
    fn test_strip_midi_extension() {
        assert_eq!(strip_midi_extension("song.midi"), Some("song"));
        assert_eq!(strip_midi_extension("song.MIDI"), Some("song"));
        assert_eq!(strip_midi_extension("my.song.midi"), Some("my.song"));
        assert_eq!(strip_midi_extension(".midi"), Some(""));
        assert_eq!(strip_midi_extension("my song.midi"), Some("my song"));
    }

    #[test]
    fn test_strip_midi_extension_none() {
        assert_eq!(strip_midi_extension("song.mid"), None);
        assert_eq!(strip_midi_extension("song"), None);
        assert_eq!(strip_midi_extension(""), None);
        assert_eq!(strip_midi_extension("song.mp3"), None);
        assert_eq!(strip_midi_extension("my song.mid"), None);
    }

    // Property-based style tests
    #[test]
    fn test_normalization_idempotent() {
        // Normalizing twice should give same result
        let filename = "my song.midi";
        let normalized_once = normalize_midi_filename(filename);
        let normalized_twice = normalize_midi_filename(&normalized_once);
        assert_eq!(normalized_once, normalized_twice);
        assert_eq!(normalized_once, "my_song.mid");
    }

    #[test]
    fn test_normalized_files_dont_need_normalization() {
        // After normalization, needs_normalization should return false
        let filename = "my song.midi";
        let normalized = normalize_midi_filename(filename);
        assert!(!needs_normalization(&normalized));
        assert_eq!(normalized, "my_song.mid");
    }

    #[test]
    fn test_long_filename() {
        let long_name = "a".repeat(200) + ".midi";
        let normalized = normalize_midi_filename(&long_name);
        assert_eq!(normalized, "a".repeat(200) + ".mid");
    }

    #[test]
    fn test_long_filename_with_spaces() {
        let long_name = "a b ".repeat(50) + "c.midi";
        let normalized = normalize_midi_filename(&long_name);
        assert!(normalized.contains('_'));
        assert!(!normalized.contains(' '));
        assert!(normalized.ends_with(".mid"));
    }

    #[test]
    fn test_real_world_examples() {
        // Common real-world filename patterns
        assert_eq!(
            normalize_midi_filename("Drum Loop 120 BPM.midi"),
            "Drum_Loop_120_BPM.mid"
        );
        assert_eq!(
            normalize_midi_filename("Bass Line - C Minor.MIDI"),
            "Bass_Line_-_C_Minor.mid"
        );
        assert_eq!(
            normalize_midi_filename("Synth Pad (Ambient).midi"),
            "Synth_Pad_(Ambient).mid"
        );
        assert_eq!(
            normalize_midi_filename("Track 01 Intro.midi"),
            "Track_01_Intro.mid"
        );
    }

    #[test]
    fn test_consecutive_spaces() {
        assert_eq!(
            normalize_midi_filename("song     name.midi"),
            "song_____name.mid"
        );
    }

    #[test]
    fn test_space_preservation_in_count() {
        // Each space becomes exactly one underscore
        let input = "a b c d.midi";
        let output = normalize_midi_filename(input);
        assert_eq!(output, "a_b_c_d.mid");
        assert_eq!(output.matches('_').count(), 3);
    }
}
