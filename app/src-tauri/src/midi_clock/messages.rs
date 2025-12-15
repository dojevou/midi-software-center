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
