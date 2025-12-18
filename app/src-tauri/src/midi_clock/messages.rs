#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// MIDI Real-Time Messages for clock synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiClockMessage {
    /// Timing Clock (0xF8) - sent 24 times per quarter note
    TimingClock,
    /// Start (0xFA) - start playback from beginning
    Start,
    /// Continue (0xFB) - continue from current position
    Continue,
    /// Stop (0xFC) - stop playback
    Stop,
    /// Active Sensing (0xFE) - keepalive, sent every 300ms
    ActiveSensing,
    /// System Reset (0xFF) - reset to power-on state
    SystemReset,
}

impl MidiClockMessage {
    pub fn to_byte(self) -> u8 {
        match self {
            Self::TimingClock => 0xF8,
            Self::Start => 0xFA,
            Self::Continue => 0xFB,
            Self::Stop => 0xFC,
            Self::ActiveSensing => 0xFE,
            Self::SystemReset => 0xFF,
        }
    }

    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0xF8 => Some(Self::TimingClock),
            0xFA => Some(Self::Start),
            0xFB => Some(Self::Continue),
            0xFC => Some(Self::Stop),
            0xFE => Some(Self::ActiveSensing),
            0xFF => Some(Self::SystemReset),
            _ => None,
        }
    }
}

/// MIDI Time Code (MTC) Quarter Frame message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiTimecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub frame_rate: FrameRate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameRate {
    Fps24,
    Fps25,
    Fps30Drop,
    Fps30,
}

impl FrameRate {
    pub fn frames_per_second(&self) -> f64 {
        match self {
            Self::Fps24 => 24.0,
            Self::Fps25 => 25.0,
            Self::Fps30Drop => 29.97,
            Self::Fps30 => 30.0,
        }
    }

    pub fn to_mtc_type(self) -> u8 {
        match self {
            Self::Fps24 => 0,
            Self::Fps25 => 1,
            Self::Fps30Drop => 2,
            Self::Fps30 => 3,
        }
    }
}

impl MidiTimecode {
    pub fn new(frame_rate: FrameRate) -> Self {
        Self { hours: 0, minutes: 0, seconds: 0, frames: 0, frame_rate }
    }

    pub fn from_millis(millis: u64, frame_rate: FrameRate) -> Self {
        let fps = frame_rate.frames_per_second();
        let total_frames = (millis as f64 / 1000.0 * fps) as u64;

        let frames = (total_frames % fps as u64) as u8;
        let total_seconds = total_frames / fps as u64;
        let seconds = (total_seconds % 60) as u8;
        let total_minutes = total_seconds / 60;
        let minutes = (total_minutes % 60) as u8;
        let hours = (total_minutes / 60) as u8;

        Self { hours, minutes, seconds, frames, frame_rate }
    }

    pub fn to_millis(self) -> u64 {
        let fps = self.frame_rate.frames_per_second();
        let total_seconds =
            self.hours as u64 * 3600 + self.minutes as u64 * 60 + self.seconds as u64;
        let frame_millis = (self.frames as f64 / fps * 1000.0) as u64;

        total_seconds * 1000 + frame_millis
    }

    /// Generate MTC Quarter Frame messages
    pub fn to_quarter_frames(self) -> [u8; 8] {
        let frame_type = self.frame_rate.to_mtc_type();
        [
            self.frames & 0x0F,                                    // Frame number low nibble
            0x10 | ((self.frames >> 4) & 0x01),                    // Frame number high nibble
            0x20 | (self.seconds & 0x0F),                          // Seconds low nibble
            0x30 | ((self.seconds >> 4) & 0x03),                   // Seconds high nibble
            0x40 | (self.minutes & 0x0F),                          // Minutes low nibble
            0x50 | ((self.minutes >> 4) & 0x03),                   // Minutes high nibble
            0x60 | (self.hours & 0x0F),                            // Hours low nibble
            0x70 | ((self.hours >> 4) & 0x01) | (frame_type << 1), // Hours high + type
        ]
    }
}

/// Song Position Pointer (SPP) - position in MIDI beats (1/16th notes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SongPosition {
    pub beats: u16, // 14-bit value (0-16383)
}

impl SongPosition {
    pub fn new(beats: u16) -> Self {
        Self { beats: beats.min(16383) }
    }

    pub fn from_bars(bars: u16, beats_per_bar: u8) -> Self {
        let total_16ths = bars * beats_per_bar as u16 * 4;
        Self::new(total_16ths)
    }

    pub fn to_bytes(self) -> [u8; 2] {
        [
            (self.beats & 0x7F) as u8,        // LSB (7 bits)
            ((self.beats >> 7) & 0x7F) as u8, // MSB (7 bits)
        ]
    }

    pub fn from_bytes(lsb: u8, msb: u8) -> Self {
        Self { beats: (lsb as u16 & 0x7F) | ((msb as u16 & 0x7F) << 7) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // MidiClockMessage Tests
    // ==========================================================================

    #[test]
    fn test_midi_clock_message_to_byte() {
        assert_eq!(MidiClockMessage::TimingClock.to_byte(), 0xF8);
        assert_eq!(MidiClockMessage::Start.to_byte(), 0xFA);
        assert_eq!(MidiClockMessage::Continue.to_byte(), 0xFB);
        assert_eq!(MidiClockMessage::Stop.to_byte(), 0xFC);
        assert_eq!(MidiClockMessage::ActiveSensing.to_byte(), 0xFE);
        assert_eq!(MidiClockMessage::SystemReset.to_byte(), 0xFF);
    }

    #[test]
    fn test_midi_clock_message_from_byte() {
        assert_eq!(
            MidiClockMessage::from_byte(0xF8),
            Some(MidiClockMessage::TimingClock)
        );
        assert_eq!(
            MidiClockMessage::from_byte(0xFA),
            Some(MidiClockMessage::Start)
        );
        assert_eq!(
            MidiClockMessage::from_byte(0xFB),
            Some(MidiClockMessage::Continue)
        );
        assert_eq!(
            MidiClockMessage::from_byte(0xFC),
            Some(MidiClockMessage::Stop)
        );
        assert_eq!(
            MidiClockMessage::from_byte(0xFE),
            Some(MidiClockMessage::ActiveSensing)
        );
        assert_eq!(
            MidiClockMessage::from_byte(0xFF),
            Some(MidiClockMessage::SystemReset)
        );
    }

    #[test]
    fn test_midi_clock_message_from_byte_invalid() {
        assert_eq!(MidiClockMessage::from_byte(0x00), None);
        assert_eq!(MidiClockMessage::from_byte(0x90), None);
        assert_eq!(MidiClockMessage::from_byte(0xF7), None);
        assert_eq!(MidiClockMessage::from_byte(0xF9), None);
        assert_eq!(MidiClockMessage::from_byte(0xFD), None);
    }

    #[test]
    fn test_midi_clock_message_roundtrip() {
        let messages = [
            MidiClockMessage::TimingClock,
            MidiClockMessage::Start,
            MidiClockMessage::Continue,
            MidiClockMessage::Stop,
            MidiClockMessage::ActiveSensing,
            MidiClockMessage::SystemReset,
        ];

        for msg in messages {
            let byte = msg.to_byte();
            let recovered = MidiClockMessage::from_byte(byte);
            assert_eq!(recovered, Some(msg));
        }
    }

    // ==========================================================================
    // FrameRate Tests
    // ==========================================================================

    #[test]
    fn test_frame_rate_fps() {
        assert!((FrameRate::Fps24.frames_per_second() - 24.0).abs() < 0.001);
        assert!((FrameRate::Fps25.frames_per_second() - 25.0).abs() < 0.001);
        assert!((FrameRate::Fps30Drop.frames_per_second() - 29.97).abs() < 0.001);
        assert!((FrameRate::Fps30.frames_per_second() - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_frame_rate_to_mtc_type() {
        assert_eq!(FrameRate::Fps24.to_mtc_type(), 0);
        assert_eq!(FrameRate::Fps25.to_mtc_type(), 1);
        assert_eq!(FrameRate::Fps30Drop.to_mtc_type(), 2);
        assert_eq!(FrameRate::Fps30.to_mtc_type(), 3);
    }

    // ==========================================================================
    // MidiTimecode Tests
    // ==========================================================================

    #[test]
    fn test_midi_timecode_new() {
        let tc = MidiTimecode::new(FrameRate::Fps30);
        assert_eq!(tc.hours, 0);
        assert_eq!(tc.minutes, 0);
        assert_eq!(tc.seconds, 0);
        assert_eq!(tc.frames, 0);
        assert_eq!(tc.frame_rate, FrameRate::Fps30);
    }

    #[test]
    fn test_midi_timecode_from_millis_zero() {
        let tc = MidiTimecode::from_millis(0, FrameRate::Fps30);
        assert_eq!(tc.hours, 0);
        assert_eq!(tc.minutes, 0);
        assert_eq!(tc.seconds, 0);
        assert_eq!(tc.frames, 0);
    }

    #[test]
    fn test_midi_timecode_from_millis_one_second() {
        let tc = MidiTimecode::from_millis(1000, FrameRate::Fps30);
        assert_eq!(tc.hours, 0);
        assert_eq!(tc.minutes, 0);
        assert_eq!(tc.seconds, 1);
        assert_eq!(tc.frames, 0);
    }

    #[test]
    fn test_midi_timecode_from_millis_one_minute() {
        let tc = MidiTimecode::from_millis(60_000, FrameRate::Fps30);
        assert_eq!(tc.hours, 0);
        assert_eq!(tc.minutes, 1);
        assert_eq!(tc.seconds, 0);
    }

    #[test]
    fn test_midi_timecode_from_millis_one_hour() {
        let tc = MidiTimecode::from_millis(3_600_000, FrameRate::Fps30);
        assert_eq!(tc.hours, 1);
        assert_eq!(tc.minutes, 0);
        assert_eq!(tc.seconds, 0);
    }

    #[test]
    fn test_midi_timecode_from_millis_complex() {
        // 1 hour, 23 minutes, 45 seconds, some frames at 30fps
        let millis = 3_600_000 + (23 * 60_000) + (45 * 1000);
        let tc = MidiTimecode::from_millis(millis, FrameRate::Fps30);
        assert_eq!(tc.hours, 1);
        assert_eq!(tc.minutes, 23);
        assert_eq!(tc.seconds, 45);
    }

    #[test]
    fn test_midi_timecode_to_millis() {
        let tc = MidiTimecode {
            hours: 1,
            minutes: 30,
            seconds: 45,
            frames: 15,
            frame_rate: FrameRate::Fps30,
        };

        let millis = tc.to_millis();
        // 1 hour = 3,600,000ms
        // 30 min = 1,800,000ms
        // 45 sec = 45,000ms
        // 15 frames at 30fps = 500ms
        // Total = 5,445,500ms
        assert_eq!(millis, 5_445_500);
    }

    #[test]
    fn test_midi_timecode_roundtrip() {
        let original_millis = 5_445_500u64;
        let tc = MidiTimecode::from_millis(original_millis, FrameRate::Fps30);
        let recovered_millis = tc.to_millis();

        // Allow small rounding error due to frame quantization
        assert!(
            (original_millis as i64 - recovered_millis as i64).abs() < 34,
            "Expected ~{}, got {}",
            original_millis,
            recovered_millis
        );
    }

    #[test]
    fn test_midi_timecode_quarter_frames_structure() {
        let tc = MidiTimecode {
            hours: 1,
            minutes: 23,
            seconds: 45,
            frames: 15,
            frame_rate: FrameRate::Fps30,
        };

        let qf = tc.to_quarter_frames();
        assert_eq!(qf.len(), 8);

        // Check that the quarter frames follow MTC spec
        // Low nibbles of each type should be in sequence
        assert_eq!(qf[0] & 0xF0, 0x00); // Frame low
        assert_eq!(qf[1] & 0xF0, 0x10); // Frame high
        assert_eq!(qf[2] & 0xF0, 0x20); // Seconds low
        assert_eq!(qf[3] & 0xF0, 0x30); // Seconds high
        assert_eq!(qf[4] & 0xF0, 0x40); // Minutes low
        assert_eq!(qf[5] & 0xF0, 0x50); // Minutes high
        assert_eq!(qf[6] & 0xF0, 0x60); // Hours low
        assert_eq!(qf[7] & 0xF0, 0x70); // Hours high + frame rate
    }

    #[test]
    fn test_midi_timecode_quarter_frames_values() {
        let tc = MidiTimecode {
            hours: 0,
            minutes: 0,
            seconds: 0,
            frames: 15,
            frame_rate: FrameRate::Fps30,
        };

        let qf = tc.to_quarter_frames();

        // Frames = 15 = 0x0F
        // Low nibble = 0x0F & 0x0F = 0x0F
        assert_eq!(qf[0] & 0x0F, 0x0F);
        // High nibble = (0x0F >> 4) & 0x01 = 0x00
        assert_eq!(qf[1] & 0x01, 0x00);
    }

    #[test]
    fn test_midi_timecode_serialization() {
        let tc = MidiTimecode {
            hours: 1,
            minutes: 23,
            seconds: 45,
            frames: 15,
            frame_rate: FrameRate::Fps30,
        };

        let json = serde_json::to_string(&tc).unwrap();
        let deserialized: MidiTimecode = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.hours, tc.hours);
        assert_eq!(deserialized.minutes, tc.minutes);
        assert_eq!(deserialized.seconds, tc.seconds);
        assert_eq!(deserialized.frames, tc.frames);
        assert_eq!(deserialized.frame_rate, tc.frame_rate);
    }

    // ==========================================================================
    // SongPosition Tests
    // ==========================================================================

    #[test]
    fn test_song_position_new() {
        let sp = SongPosition::new(100);
        assert_eq!(sp.beats, 100);
    }

    #[test]
    fn test_song_position_new_clamped() {
        // Max is 16383 (14-bit value)
        let sp = SongPosition::new(20000);
        assert_eq!(sp.beats, 16383);
    }

    #[test]
    fn test_song_position_from_bars() {
        // 4 bars in 4/4 time = 4 * 4 * 4 = 64 sixteenth notes
        let sp = SongPosition::from_bars(4, 4);
        assert_eq!(sp.beats, 64);
    }

    #[test]
    fn test_song_position_from_bars_different_time_sig() {
        // 2 bars in 3/4 time = 2 * 3 * 4 = 24 sixteenth notes
        let sp = SongPosition::from_bars(2, 3);
        assert_eq!(sp.beats, 24);
    }

    #[test]
    fn test_song_position_to_bytes() {
        let sp = SongPosition::new(1234);
        let bytes = sp.to_bytes();

        // 1234 = 0x04D2
        // LSB (7 bits) = 1234 & 0x7F = 82 (0x52)
        // MSB (7 bits) = (1234 >> 7) & 0x7F = 9 (0x09)
        assert_eq!(bytes[0], 82);
        assert_eq!(bytes[1], 9);
    }

    #[test]
    fn test_song_position_from_bytes() {
        let sp = SongPosition::from_bytes(82, 9);
        assert_eq!(sp.beats, 1234);
    }

    #[test]
    fn test_song_position_bytes_roundtrip() {
        let original = SongPosition::new(8000);
        let bytes = original.to_bytes();
        let recovered = SongPosition::from_bytes(bytes[0], bytes[1]);
        assert_eq!(original.beats, recovered.beats);
    }

    #[test]
    fn test_song_position_bytes_max_value() {
        let sp = SongPosition::new(16383); // Max 14-bit value
        let bytes = sp.to_bytes();
        let recovered = SongPosition::from_bytes(bytes[0], bytes[1]);
        assert_eq!(recovered.beats, 16383);
    }

    #[test]
    fn test_song_position_serialization() {
        let sp = SongPosition::new(5000);

        let json = serde_json::to_string(&sp).unwrap();
        let deserialized: SongPosition = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.beats, sp.beats);
    }
}
