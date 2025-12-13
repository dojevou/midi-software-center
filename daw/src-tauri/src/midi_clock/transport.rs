#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportState {
    Stopped,
    Playing,
    Recording,
    Paused,
}

impl Default for TransportState {
    fn default() -> Self {
        Self::Stopped
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self {
            numerator: 4,
            denominator: 4,
        }
    }
}

impl TimeSignature {
    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub fn beats_per_bar(&self) -> u8 {
        self.numerator
    }

    /// Beat value relative to quarter note
    pub fn beat_value(&self) -> f64 {
        4.0 / self.denominator as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transport {
    pub state: TransportState,
    pub bpm: f64,
    pub time_signature: TimeSignature,
    pub position_ticks: u64,
    pub position_millis: u64,
    pub bar: u32,
    pub beat: u32,
    pub tick: u32,
    pub loop_enabled: bool,
    pub loop_start_ticks: u64,
    pub loop_end_ticks: u64,
    pub metronome_enabled: bool,
    pub count_in_enabled: bool,
    pub count_in_bars: u8,
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            state: TransportState::Stopped,
            bpm: 120.0,
            time_signature: TimeSignature::default(),
            position_ticks: 0,
            position_millis: 0,
            bar: 1,
            beat: 1,
            tick: 0,
            loop_enabled: false,
            loop_start_ticks: 0,
            loop_end_ticks: 0,
            metronome_enabled: false,
            count_in_enabled: false,
            count_in_bars: 1,
        }
    }
}
