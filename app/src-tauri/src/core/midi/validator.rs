/// MIDI message validation
///
/// Trusty Module: Pure validation functions for MIDI data.
/// No I/O operations - all functions are deterministic and testable.
use super::playback_types::{MidiEventType, MidiMessage};

/// Validate MIDI channel (0-15)
///
/// MIDI channels are 0-indexed (0-15 represent MIDI channels 1-16).
pub fn validate_channel(channel: u8) -> Result<u8, String> {
    if channel > 15 {
        Err(format!("Invalid MIDI channel: {}. Must be 0-15", channel))
    } else {
        Ok(channel)
    }
}

/// Validate MIDI note (0-127)
///
/// MIDI note numbers range from 0 to 127 (C-1 to G9).
pub fn validate_note(note: u8) -> Result<u8, String> {
    if note > 127 {
        Err(format!("Invalid MIDI note: {}. Must be 0-127", note))
    } else {
        Ok(note)
    }
}

/// Validate MIDI velocity (0-127)
///
/// Velocity 0 is treated as note off in some contexts.
pub fn validate_velocity(velocity: u8) -> Result<u8, String> {
    if velocity > 127 {
        Err(format!("Invalid velocity: {}. Must be 0-127", velocity))
    } else {
        Ok(velocity)
    }
}

/// Validate MIDI control value (0-127)
///
/// Used for control change messages and other data values.
pub fn validate_control_value(value: u8) -> Result<u8, String> {
    if value > 127 {
        Err(format!("Invalid control value: {}. Must be 0-127", value))
    } else {
        Ok(value)
    }
}

/// Validate complete MIDI message
///
/// Performs comprehensive validation of all message fields.
pub fn validate_message(msg: &MidiMessage) -> Result<(), String> {
    validate_channel(msg.channel)?;

    match msg.event_type {
        MidiEventType::NoteOn | MidiEventType::NoteOff => {
            validate_note(msg.data1)?;
            validate_velocity(msg.data2)?;
        },
        MidiEventType::ControlChange => {
            validate_control_value(msg.data1)?;
            validate_control_value(msg.data2)?;
        },
        MidiEventType::ProgramChange => {
            validate_control_value(msg.data1)?;
        },
        MidiEventType::Aftertouch => {
            validate_control_value(msg.data1)?;
        },
        MidiEventType::PitchBend => {
            // Pitch bend uses 14-bit value split across data1 and data2
            // Each byte is 7-bit (0-127), so no additional validation needed
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_channel() {
        assert!(validate_channel(0).is_ok());
        assert!(validate_channel(15).is_ok());
        assert!(validate_channel(16).is_err());
        assert!(validate_channel(255).is_err());
    }

    #[test]
    fn test_validate_note() {
        assert!(validate_note(0).is_ok());
        assert!(validate_note(127).is_ok());
        assert!(validate_note(128).is_err());
        assert!(validate_note(255).is_err());
    }

    #[test]
    fn test_validate_velocity() {
        assert!(validate_velocity(0).is_ok());
        assert!(validate_velocity(127).is_ok());
        assert!(validate_velocity(128).is_err());
    }

    #[test]
    fn test_validate_control_value() {
        assert!(validate_control_value(0).is_ok());
        assert!(validate_control_value(127).is_ok());
        assert!(validate_control_value(128).is_err());
    }

    #[test]
    fn test_validate_message_note_on() {
        let msg = MidiMessage {
            event_type: MidiEventType::NoteOn,
            channel: 0,
            data1: 60,
            data2: 100,
            timestamp: 0,
        };

        assert!(validate_message(&msg).is_ok());
    }

    #[test]
    fn test_validate_message_invalid_channel() {
        let msg = MidiMessage {
            event_type: MidiEventType::NoteOn,
            channel: 16,
            data1: 60,
            data2: 100,
            timestamp: 0,
        };

        assert!(validate_message(&msg).is_err());
    }

    #[test]
    fn test_validate_message_invalid_note() {
        let msg = MidiMessage {
            event_type: MidiEventType::NoteOn,
            channel: 0,
            data1: 128,
            data2: 100,
            timestamp: 0,
        };

        assert!(validate_message(&msg).is_err());
    }
}
