   /// MIDI types and message encoding/decoding
   ///
   /// Trusty Module: Pure data structures and conversion functions.
   /// No I/O operations - all functions are pure and deterministic.

// Re-export from models for convenience
pub use crate::models::MidiEventType;

/// MIDI message with raw data
///
/// Internal representation used for encoding/decoding MIDI messages.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MidiMessage {
    pub event_type: MidiEventType,
    pub channel: u8,
    pub data1: u8,
    pub data2: u8,
    pub timestamp: u64,
}

impl MidiMessage {
    /// Convert to raw MIDI bytes
    ///
    /// Pure function - converts MIDI message to bytes for transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self.event_type {
            MidiEventType::NoteOn => {
                vec![
                    0x90 | (self.channel & 0x0F),
                    self.data1 & 0x7F,
                    self.data2 & 0x7F,
                ]
            }
            MidiEventType::NoteOff => {
                vec![
                    0x80 | (self.channel & 0x0F),
                    self.data1 & 0x7F,
                    0x00,
                ]
            }
            MidiEventType::ControlChange => {
                vec![
                    0xB0 | (self.channel & 0x0F),
                    self.data1 & 0x7F,
                    self.data2 & 0x7F,
                ]
            }
            MidiEventType::ProgramChange => {
                vec![
                    0xC0 | (self.channel & 0x0F),
                    self.data1 & 0x7F,
                ]
            }
            MidiEventType::PitchBend => {
                let value = ((self.data2 as u16) << 7) | (self.data1 as u16);
                vec![
                    0xE0 | (self.channel & 0x0F),
                    (value & 0x7F) as u8,
                    ((value >> 7) & 0x7F) as u8,
                ]
            }
            MidiEventType::Aftertouch => {
                vec![
                    0xD0 | (self.channel & 0x0F),
                    self.data1 & 0x7F,
                ]
            }
        }
    }

    /// Parse from raw MIDI bytes
    ///
    /// Pure function - converts raw bytes to MIDI message.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Empty MIDI message".to_string());
        }

        let status = bytes[0];
        let channel = status & 0x0F;
        let command = status & 0xF0;

        let (event_type, data1, data2) = match command {
            0x90 => {
                if bytes.len() < 3 {
                    return Err("Incomplete Note On message".to_string());
                }
                (MidiEventType::NoteOn, bytes[1], bytes[2])
            }
            0x80 => {
                if bytes.len() < 3 {
                    return Err("Incomplete Note Off message".to_string());
                }
                (MidiEventType::NoteOff, bytes[1], bytes[2])
            }
            0xB0 => {
                if bytes.len() < 3 {
                    return Err("Incomplete Control Change message".to_string());
                }
                (MidiEventType::ControlChange, bytes[1], bytes[2])
            }
            0xC0 => {
                if bytes.len() < 2 {
                    return Err("Incomplete Program Change message".to_string());
                }
                (MidiEventType::ProgramChange, bytes[1], 0)
            }
            0xE0 => {
                if bytes.len() < 3 {
                    return Err("Incomplete Pitch Bend message".to_string());
                }
                (MidiEventType::PitchBend, bytes[1], bytes[2])
            }
            0xD0 => {
                if bytes.len() < 2 {
                    return Err("Incomplete Aftertouch message".to_string());
                }
                (MidiEventType::Aftertouch, bytes[1], 0)
            }
            _ => return Err(format!("Unknown MIDI command: {:#X}", command)),
        };

        Ok(MidiMessage {
            event_type,
            channel,
            data1,
            data2,
            timestamp: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_on_encoding() {
        let msg = MidiMessage {
            event_type: MidiEventType::NoteOn,
            channel: 0,
            data1: 60, // Middle C
            data2: 100, // Velocity
            timestamp: 0,
        };

        let bytes = msg.to_bytes();
        assert_eq!(bytes, vec![0x90, 60, 100]);
    }

    #[test]
    fn test_note_off_encoding() {
        let msg = MidiMessage {
            event_type: MidiEventType::NoteOff,
            channel: 0,
            data1: 60,
            data2: 0,
            timestamp: 0,
        };

        let bytes = msg.to_bytes();
        assert_eq!(bytes, vec![0x80, 60, 0]);
    }

    #[test]
    fn test_message_parsing() {
        let bytes = vec![0x90, 60, 100];
        let msg = MidiMessage::from_bytes(&bytes).unwrap();

        assert_eq!(msg.event_type, MidiEventType::NoteOn);
        assert_eq!(msg.channel, 0);
        assert_eq!(msg.data1, 60);
        assert_eq!(msg.data2, 100);
    }

    #[test]
    fn test_invalid_message() {
        let bytes = vec![];
        let result = MidiMessage::from_bytes(&bytes);
        assert!(result.is_err());
    }
}
