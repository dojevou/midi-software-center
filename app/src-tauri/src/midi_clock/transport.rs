#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportState {
    #[default]
    Stopped,
    Playing,
    Recording,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self { numerator: 4, denominator: 4 }
    }
}

impl TimeSignature {
    pub fn new(numerator: u8, denominator: u8) -> Self {
        Self { numerator, denominator }
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

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // TransportState Tests
    // ==========================================================================

    #[test]
    fn test_transport_state_default() {
        let state = TransportState::default();
        assert_eq!(state, TransportState::Stopped);
    }

    #[test]
    fn test_transport_state_variants() {
        // Ensure all variants exist
        let _stopped = TransportState::Stopped;
        let _playing = TransportState::Playing;
        let _recording = TransportState::Recording;
        let _paused = TransportState::Paused;
    }

    #[test]
    fn test_transport_state_serialization() {
        let states = [
            TransportState::Stopped,
            TransportState::Playing,
            TransportState::Recording,
            TransportState::Paused,
        ];

        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: TransportState = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, state);
        }
    }

    #[test]
    fn test_transport_state_equality() {
        assert_eq!(TransportState::Stopped, TransportState::Stopped);
        assert_ne!(TransportState::Stopped, TransportState::Playing);
        assert_ne!(TransportState::Playing, TransportState::Recording);
        assert_ne!(TransportState::Recording, TransportState::Paused);
    }

    // ==========================================================================
    // TimeSignature Tests
    // ==========================================================================

    #[test]
    fn test_time_signature_default() {
        let ts = TimeSignature::default();
        assert_eq!(ts.numerator, 4);
        assert_eq!(ts.denominator, 4);
    }

    #[test]
    fn test_time_signature_new() {
        let ts = TimeSignature::new(3, 4);
        assert_eq!(ts.numerator, 3);
        assert_eq!(ts.denominator, 4);
    }

    #[test]
    fn test_time_signature_new_various() {
        let ts_6_8 = TimeSignature::new(6, 8);
        assert_eq!(ts_6_8.numerator, 6);
        assert_eq!(ts_6_8.denominator, 8);

        let ts_7_8 = TimeSignature::new(7, 8);
        assert_eq!(ts_7_8.numerator, 7);
        assert_eq!(ts_7_8.denominator, 8);

        let ts_5_4 = TimeSignature::new(5, 4);
        assert_eq!(ts_5_4.numerator, 5);
        assert_eq!(ts_5_4.denominator, 4);
    }

    #[test]
    fn test_time_signature_beats_per_bar() {
        assert_eq!(TimeSignature::new(4, 4).beats_per_bar(), 4);
        assert_eq!(TimeSignature::new(3, 4).beats_per_bar(), 3);
        assert_eq!(TimeSignature::new(6, 8).beats_per_bar(), 6);
        assert_eq!(TimeSignature::new(12, 8).beats_per_bar(), 12);
    }

    #[test]
    fn test_time_signature_beat_value() {
        // 4/4 - quarter note = 1 quarter note
        assert!((TimeSignature::new(4, 4).beat_value() - 1.0).abs() < 0.001);

        // 6/8 - eighth note = 0.5 quarter notes
        assert!((TimeSignature::new(6, 8).beat_value() - 0.5).abs() < 0.001);

        // 2/2 - half note = 2 quarter notes
        assert!((TimeSignature::new(2, 2).beat_value() - 2.0).abs() < 0.001);

        // 3/16 - sixteenth note = 0.25 quarter notes
        assert!((TimeSignature::new(3, 16).beat_value() - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_time_signature_serialization() {
        let ts = TimeSignature::new(7, 8);

        let json = serde_json::to_string(&ts).unwrap();
        let deserialized: TimeSignature = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.numerator, ts.numerator);
        assert_eq!(deserialized.denominator, ts.denominator);
    }

    #[test]
    fn test_time_signature_equality() {
        let ts1 = TimeSignature::new(4, 4);
        let ts2 = TimeSignature::new(4, 4);
        let ts3 = TimeSignature::new(3, 4);

        assert_eq!(ts1, ts2);
        assert_ne!(ts1, ts3);
    }

    // ==========================================================================
    // Transport Tests
    // ==========================================================================

    #[test]
    fn test_transport_default() {
        let transport = Transport::default();

        assert_eq!(transport.state, TransportState::Stopped);
        assert!((transport.bpm - 120.0).abs() < 0.001);
        assert_eq!(transport.time_signature, TimeSignature::default());
        assert_eq!(transport.position_ticks, 0);
        assert_eq!(transport.position_millis, 0);
        assert_eq!(transport.bar, 1);
        assert_eq!(transport.beat, 1);
        assert_eq!(transport.tick, 0);
        assert!(!transport.loop_enabled);
        assert_eq!(transport.loop_start_ticks, 0);
        assert_eq!(transport.loop_end_ticks, 0);
        assert!(!transport.metronome_enabled);
        assert!(!transport.count_in_enabled);
        assert_eq!(transport.count_in_bars, 1);
    }

    #[test]
    fn test_transport_serialization() {
        let transport = Transport {
            state: TransportState::Playing,
            bpm: 140.0,
            time_signature: TimeSignature::new(6, 8),
            position_ticks: 1920,
            position_millis: 1000,
            bar: 2,
            beat: 3,
            tick: 240,
            loop_enabled: true,
            loop_start_ticks: 0,
            loop_end_ticks: 7680,
            metronome_enabled: true,
            count_in_enabled: true,
            count_in_bars: 2,
        };

        let json = serde_json::to_string(&transport).unwrap();
        let deserialized: Transport = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.state, transport.state);
        assert!((deserialized.bpm - transport.bpm).abs() < 0.001);
        assert_eq!(deserialized.time_signature, transport.time_signature);
        assert_eq!(deserialized.position_ticks, transport.position_ticks);
        assert_eq!(deserialized.position_millis, transport.position_millis);
        assert_eq!(deserialized.bar, transport.bar);
        assert_eq!(deserialized.beat, transport.beat);
        assert_eq!(deserialized.tick, transport.tick);
        assert_eq!(deserialized.loop_enabled, transport.loop_enabled);
        assert_eq!(deserialized.loop_start_ticks, transport.loop_start_ticks);
        assert_eq!(deserialized.loop_end_ticks, transport.loop_end_ticks);
        assert_eq!(deserialized.metronome_enabled, transport.metronome_enabled);
        assert_eq!(deserialized.count_in_enabled, transport.count_in_enabled);
        assert_eq!(deserialized.count_in_bars, transport.count_in_bars);
    }

    #[test]
    fn test_transport_clone() {
        let transport = Transport {
            state: TransportState::Recording,
            bpm: 180.0,
            ..Transport::default()
        };

        let cloned = transport.clone();

        assert_eq!(cloned.state, transport.state);
        assert!((cloned.bpm - transport.bpm).abs() < 0.001);
    }
}
