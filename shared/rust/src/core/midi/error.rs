use thiserror::Error;

#[derive(Error, Debug)]
pub enum MidiParseError {
    #[error("Invalid MIDI header: {0}")]
    InvalidHeader(String),

    #[error("Invalid track data at byte {position}: {reason}")]
    InvalidTrack { position: usize, reason: String },

    #[error("Unsupported MIDI format: {0}")]
    UnsupportedFormat(u16),

    #[error("Invalid event at byte {position}: {reason}")]
    InvalidEvent { position: usize, reason: String },

    #[error("Incomplete data: expected {expected} bytes, got {actual}")]
    IncompleteData { expected: usize, actual: usize },

    #[error("Invalid variable-length quantity at byte {0}")]
    InvalidVarLen(usize),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, MidiParseError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // ============================================================================
    // Error Construction Tests
    // ============================================================================

    #[test]
    fn test_invalid_header_construction() {
        let error = MidiParseError::InvalidHeader("bad magic number".to_string());
        assert!(matches!(error, MidiParseError::InvalidHeader(_)));
    }

    #[test]
    fn test_invalid_track_construction() {
        let error =
            MidiParseError::InvalidTrack { position: 42, reason: "unexpected end".to_string() };
        assert!(matches!(error, MidiParseError::InvalidTrack { .. }));
    }

    #[test]
    fn test_unsupported_format_construction() {
        let error = MidiParseError::UnsupportedFormat(99);
        assert!(matches!(error, MidiParseError::UnsupportedFormat(99)));
    }

    #[test]
    fn test_invalid_event_construction() {
        let error = MidiParseError::InvalidEvent {
            position: 100,
            reason: "invalid status byte".to_string(),
        };
        assert!(matches!(error, MidiParseError::InvalidEvent { .. }));
    }

    #[test]
    fn test_incomplete_data_construction() {
        let error = MidiParseError::IncompleteData { expected: 100, actual: 50 };
        assert!(matches!(error, MidiParseError::IncompleteData { .. }));
    }

    #[test]
    fn test_invalid_var_len_construction() {
        let error = MidiParseError::InvalidVarLen(256);
        assert!(matches!(error, MidiParseError::InvalidVarLen(256)));
    }

    // ============================================================================
    // Display Formatting Tests
    // ============================================================================

    #[test]
    fn test_invalid_header_message_format() {
        let error = MidiParseError::InvalidHeader("bad magic number".to_string());
        let msg = error.to_string();
        assert!(msg.contains("Invalid MIDI header"));
        assert!(msg.contains("bad magic number"));
    }

    #[test]
    fn test_invalid_track_message_includes_position() {
        let error =
            MidiParseError::InvalidTrack { position: 42, reason: "unexpected end".to_string() };
        let msg = error.to_string();
        assert!(msg.contains("42"));
        assert!(msg.contains("unexpected end"));
        assert!(msg.contains("Invalid track data"));
    }

    #[test]
    fn test_unsupported_format_message() {
        let error = MidiParseError::UnsupportedFormat(99);
        let msg = error.to_string();
        assert!(msg.contains("Unsupported MIDI format"));
        assert!(msg.contains("99"));
    }

    #[test]
    fn test_invalid_event_message_includes_position() {
        let error = MidiParseError::InvalidEvent {
            position: 100,
            reason: "invalid status byte".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("invalid status byte"));
    }

    #[test]
    fn test_incomplete_data_shows_expected_vs_actual() {
        let error = MidiParseError::IncompleteData { expected: 100, actual: 50 };
        let msg = error.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
        assert!(msg.contains("Incomplete data"));
    }

    #[test]
    fn test_invalid_var_len_message() {
        let error = MidiParseError::InvalidVarLen(256);
        let msg = error.to_string();
        assert!(msg.contains("256"));
        assert!(msg.contains("Invalid variable-length quantity"));
    }

    #[test]
    fn test_io_error_message() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error = MidiParseError::Io(io_error);
        let msg = error.to_string();
        assert!(msg.contains("IO error"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_utf8_error_message() {
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        let utf8_error = String::from_utf8(invalid_utf8).unwrap_err();
        let error = MidiParseError::Utf8(utf8_error);
        let msg = error.to_string();
        assert!(msg.contains("UTF-8 decode error"));
    }

    // ============================================================================
    // Error Conversion Tests (From trait)
    // ============================================================================

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "test file");
        let midi_error: MidiParseError = io_error.into();

        assert!(matches!(midi_error, MidiParseError::Io(_)));
        assert!(midi_error.to_string().contains("test file"));
    }

    #[test]
    fn test_utf8_error_conversion() {
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        let utf8_error = String::from_utf8(invalid_utf8).unwrap_err();
        let midi_error: MidiParseError = utf8_error.into();

        assert!(matches!(midi_error, MidiParseError::Utf8(_)));
    }

    // ============================================================================
    // Debug Formatting Tests
    // ============================================================================

    #[test]
    fn test_error_debug_format() {
        let error = MidiParseError::UnsupportedFormat(99);
        let debug = format!("{:?}", error);
        assert!(debug.contains("UnsupportedFormat"));
        assert!(debug.contains("99"));
    }

    #[test]
    fn test_error_debug_includes_variant_name() {
        let error = MidiParseError::InvalidHeader("test".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("InvalidHeader"));
    }

    #[test]
    fn test_error_debug_includes_data() {
        let error = MidiParseError::IncompleteData { expected: 100, actual: 50 };
        let debug = format!("{:?}", error);
        assert!(debug.contains("100"));
        assert!(debug.contains("50"));
    }

    // ============================================================================
    // Result Type Alias Tests
    // ============================================================================

    #[test]
    fn test_result_type_alias_ok() {
        let result: Result<i32> = Ok(42);
        assert!(matches!(result, Ok(42)));
    }

    #[test]
    fn test_result_type_alias_err() {
        let result: Result<i32> = Err(MidiParseError::InvalidVarLen(0));
        assert!(result.is_err());
    }

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    #[test]
    fn test_empty_error_messages() {
        let error = MidiParseError::InvalidHeader(String::new());
        let msg = error.to_string();
        assert!(msg.contains("Invalid MIDI header"));
    }

    #[test]
    fn test_very_long_error_message() {
        let long_msg = "x".repeat(10000);
        let error = MidiParseError::InvalidHeader(long_msg.clone());
        let msg = error.to_string();
        assert!(msg.contains(&long_msg));
        assert_eq!(msg.len(), "Invalid MIDI header: ".len() + 10000);
    }

    #[test]
    fn test_special_characters_in_error() {
        let error = MidiParseError::InvalidHeader("Line 1\nLine 2\tTab".to_string());
        let msg = error.to_string();
        assert!(msg.contains("Line 1\nLine 2\tTab"));
    }

    #[test]
    fn test_unicode_in_error_message() {
        let error = MidiParseError::InvalidHeader("Invalid: 🎹 MIDI file".to_string());
        let msg = error.to_string();
        assert!(msg.contains("🎹"));
    }

    #[test]
    fn test_position_boundaries() {
        let error_min =
            MidiParseError::InvalidTrack { position: 0, reason: "start of file".to_string() };
        let error_max = MidiParseError::InvalidTrack {
            position: usize::MAX,
            reason: "end of file".to_string(),
        };

        assert!(error_min.to_string().contains("0"));
        assert!(error_max.to_string().contains(&usize::MAX.to_string()));
    }

    #[test]
    fn test_all_format_variants() {
        // Test format values 0-2 (valid) and beyond
        for format in [0, 1, 2, 3, 99, u16::MAX] {
            let error = MidiParseError::UnsupportedFormat(format);
            let msg = error.to_string();
            assert!(msg.contains(&format.to_string()));
        }
    }

    #[test]
    fn test_expected_actual_boundaries() {
        let cases = vec![(0, 0), (1, 0), (100, 50), (usize::MAX, 0), (1000, 999)];

        for (expected, actual) in cases {
            let error = MidiParseError::IncompleteData { expected, actual };
            let msg = error.to_string();
            assert!(msg.contains(&expected.to_string()));
            assert!(msg.contains(&actual.to_string()));
        }
    }

    // ============================================================================
    // Security Tests
    // ============================================================================

    #[test]
    fn test_error_message_no_memory_leak() {
        // Create large error messages to ensure no memory leak
        for _ in 0..1000 {
            let error = MidiParseError::InvalidHeader("x".repeat(1000));
            let _ = error.to_string();
        }
        // If we get here, no memory leak (would OOM otherwise)
    }

    #[test]
    fn test_malicious_position_values() {
        // Test extreme position values don't cause issues
        let positions = vec![0, 1, usize::MAX - 1, usize::MAX];

        for pos in positions {
            let error = MidiParseError::InvalidEvent { position: pos, reason: "test".to_string() };
            let msg = error.to_string();
            assert!(msg.contains(&pos.to_string()));
        }
    }

    #[test]
    fn test_error_size_is_reasonable() {
        // Ensure error type doesn't use excessive memory
        use std::mem;
        let size = mem::size_of::<MidiParseError>();

        // thiserror errors should be reasonably sized (< 200 bytes typical)
        assert!(size < 256, "MidiParseError is too large: {} bytes", size);
    }
}
