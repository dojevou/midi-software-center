//! Sequencer timing calculations
//!
//! Trusty Module: Pure timing functions for MIDI sequencing.
//! All functions are deterministic and thoroughly tested.

/// Bar position structure
///
/// Represents a position in musical time as bar:beat:tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarPosition {
    pub bar: u32,
    pub beat: u32,
    pub tick: u64,
}

/// Calculate microseconds per tick at given BPM
///
/// Used for high-precision timing in MIDI playback.
///
/// # Arguments
/// * `bpm` - Beats per minute
/// * `ticks_per_quarter` - MIDI resolution (typically 480)
///
/// # Returns
/// Microseconds per tick as f64
pub fn microseconds_per_tick(bpm: f32, ticks_per_quarter: u16) -> f64 {
    let microseconds_per_minute = 60_000_000.0;
    let microseconds_per_beat = microseconds_per_minute / bpm as f64;
    microseconds_per_beat / ticks_per_quarter as f64
}

/// Calculate bar and beat from tick position
///
/// Converts absolute tick position to musical bar:beat notation.
///
/// # Arguments
/// * `tick` - Absolute tick position
/// * `ticks_per_quarter` - MIDI resolution
/// * `beats_per_bar` - Time signature numerator (typically 4)
///
/// # Returns
/// Tuple of (bar, beat) as (u32, u32)
pub fn tick_to_bar_beat(
    tick: u64,
    ticks_per_quarter: u16,
    beats_per_bar: u8,
) -> (u32, u32) {
    let ticks_per_beat = ticks_per_quarter as u64;
    let ticks_per_bar = ticks_per_beat * beats_per_bar as u64;

    let bar = (tick / ticks_per_bar) as u32;
    let beat = ((tick % ticks_per_bar) / ticks_per_beat) as u32;

    (bar, beat)
}

/// Calculate tick position from bar and beat
///
/// Converts musical bar:beat notation to absolute tick position.
///
/// # Arguments
/// * `bar` - Bar number (0-indexed)
/// * `beat` - Beat within bar (0-indexed)
/// * `ticks_per_quarter` - MIDI resolution
/// * `beats_per_bar` - Time signature numerator
///
/// # Returns
/// Absolute tick position as u64
pub fn bar_beat_to_tick(
    bar: u32,
    beat: u32,
    ticks_per_quarter: u16,
    beats_per_bar: u8,
) -> u64 {
    let ticks_per_beat = ticks_per_quarter as u64;
    let ticks_per_bar = ticks_per_beat * beats_per_bar as u64;

    (bar as u64 * ticks_per_bar) + (beat as u64 * ticks_per_beat)
}

/// Calculate seconds from ticks
///
/// Converts MIDI ticks to real time in seconds.
///
/// # Arguments
/// * `tick` - Absolute tick position
/// * `bpm` - Current tempo
/// * `ticks_per_quarter` - MIDI resolution
///
/// # Returns
/// Time in seconds as f64
pub fn ticks_to_seconds(tick: u64, bpm: f32, ticks_per_quarter: u16) -> f64 {
    let us_per_tick = microseconds_per_tick(bpm, ticks_per_quarter);
    (tick as f64 * us_per_tick) / 1_000_000.0
}

/// Calculate ticks from seconds
///
/// Converts real time in seconds to MIDI ticks.
///
/// # Arguments
/// * `seconds` - Time in seconds
/// * `bpm` - Current tempo
/// * `ticks_per_quarter` - MIDI resolution
///
/// # Returns
/// Absolute tick position as u64
pub fn seconds_to_ticks(seconds: f64, bpm: f32, ticks_per_quarter: u16) -> u64 {
    let us_per_tick = microseconds_per_tick(bpm, ticks_per_quarter);
    ((seconds * 1_000_000.0) / us_per_tick) as u64
}

/// Convert ticks to microseconds
///
/// Converts MIDI ticks to microseconds for precise timing.
///
/// # Arguments
/// * `ticks` - Number of ticks
/// * `tpqn` - Ticks per quarter note
/// * `bpm` - Beats per minute
///
/// # Returns
/// Time in microseconds as u64
pub fn ticks_to_microseconds(ticks: u64, tpqn: u16, bpm: f32) -> u64 {
    let us_per_tick = microseconds_per_tick(bpm, tpqn);
    (ticks as f64 * us_per_tick) as u64
}

/// Convert microseconds to ticks
///
/// Converts microseconds to MIDI ticks.
///
/// # Arguments
/// * `micros` - Time in microseconds
/// * `tpqn` - Ticks per quarter note
/// * `bpm` - Beats per minute
///
/// # Returns
/// Number of ticks as u64
pub fn microseconds_to_ticks(micros: u64, tpqn: u16, bpm: f32) -> u64 {
    let us_per_tick = microseconds_per_tick(bpm, tpqn);
    (micros as f64 / us_per_tick) as u64
}

/// Calculate bar position from tick
///
/// Converts absolute tick to bar:beat:tick position.
///
/// # Arguments
/// * `tick` - Absolute tick position
/// * `tpqn` - Ticks per quarter note
/// * `time_sig_num` - Time signature numerator (e.g., 4 in 4/4)
/// * `time_sig_denom` - Time signature denominator (e.g., 4 in 4/4)
///
/// # Returns
/// BarPosition with bar, beat, and tick within beat
pub fn calculate_bar_position(
    tick: u64,
    tpqn: u16,
    time_sig_num: u8,
    time_sig_denom: u8,
) -> BarPosition {
    let ticks_per_beat = (tpqn as u64 * 4) / time_sig_denom as u64;
    let ticks_per_bar = ticks_per_beat * time_sig_num as u64;

    let bar = (tick / ticks_per_bar) as u32;
    let tick_in_bar = tick % ticks_per_bar;
    let beat = (tick_in_bar / ticks_per_beat) as u32;
    let tick_in_beat = tick_in_bar % ticks_per_beat;

    BarPosition {
        bar,
        beat,
        tick: tick_in_beat,
    }
}

/// Calculate ticks per bar
///
/// Calculates the number of ticks in one bar based on time signature.
///
/// # Arguments
/// * `tpqn` - Ticks per quarter note
/// * `time_sig_num` - Time signature numerator (e.g., 4 in 4/4)
/// * `time_sig_denom` - Time signature denominator (e.g., 4 in 4/4)
///
/// # Returns
/// Number of ticks per bar as u64
///
/// # Examples
/// - 4/4 time with 480 TPQN: 1920 ticks per bar (4 quarter notes)
/// - 3/4 time with 480 TPQN: 1440 ticks per bar (3 quarter notes)
/// - 6/8 time with 480 TPQN: 1440 ticks per bar (2 dotted quarter notes)
pub fn ticks_per_bar(tpqn: u16, time_sig_num: u8, time_sig_denom: u8) -> u64 {
    let ticks_per_beat = (tpqn as u64 * 4) / time_sig_denom as u64;
    ticks_per_beat * time_sig_num as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microseconds_per_tick() {
        // At 120 BPM with 480 TPQN:
        // 60,000,000 / 120 = 500,000 µs per beat
        // 500,000 / 480 = 1041.67 µs per tick
        let result = microseconds_per_tick(120.0, 480);
        assert!((result - 1041.67).abs() < 0.01);
    }

    #[test]
    fn test_microseconds_per_tick_different_bpm() {
        // At 90 BPM:
        // 60,000,000 / 90 = 666,666.67 µs per beat
        // 666,666.67 / 480 = 1388.89 µs per tick
        let result = microseconds_per_tick(90.0, 480);
        assert!((result - 1388.89).abs() < 0.01);
    }

    #[test]
    fn test_tick_to_bar_beat_start() {
        // Bar 0, Beat 0 = tick 0
        assert_eq!(tick_to_bar_beat(0, 480, 4), (0, 0));
    }

    #[test]
    fn test_tick_to_bar_beat_second_beat() {
        // Bar 0, Beat 1 = tick 480
        assert_eq!(tick_to_bar_beat(480, 480, 4), (0, 1));
    }

    #[test]
    fn test_tick_to_bar_beat_second_bar() {
        // Bar 1, Beat 0 = tick 1920 (480 * 4)
        assert_eq!(tick_to_bar_beat(1920, 480, 4), (1, 0));
    }

    #[test]
    fn test_tick_to_bar_beat_complex() {
        // Bar 2, Beat 3 = tick 4560 (480 * 4 * 2 + 480 * 3)
        assert_eq!(tick_to_bar_beat(5280, 480, 4), (2, 3));
    }

    #[test]
    fn test_bar_beat_to_tick_start() {
        assert_eq!(bar_beat_to_tick(0, 0, 480, 4), 0);
    }

    #[test]
    fn test_bar_beat_to_tick_second_beat() {
        assert_eq!(bar_beat_to_tick(0, 1, 480, 4), 480);
    }

    #[test]
    fn test_bar_beat_to_tick_second_bar() {
        assert_eq!(bar_beat_to_tick(1, 0, 480, 4), 1920);
    }

    #[test]
    fn test_bar_beat_round_trip() {
        // Test round-trip conversion
        let original_tick = 5280u64;
        let (bar, beat) = tick_to_bar_beat(original_tick, 480, 4);
        let converted_tick = bar_beat_to_tick(bar, beat, 480, 4);
        assert_eq!(original_tick, converted_tick);
    }

    #[test]
    fn test_ticks_to_seconds() {
        // At 120 BPM, 480 TPQN:
        // 1920 ticks = 1 bar = 4 beats = 2 seconds
        let seconds = ticks_to_seconds(1920, 120.0, 480);
        assert!((seconds - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_seconds_to_ticks() {
        // At 120 BPM, 480 TPQN:
        // 2 seconds = 4 beats = 1920 ticks
        let ticks = seconds_to_ticks(2.0, 120.0, 480);
        // Allow for floating point rounding (within 1 tick)
        assert!((ticks as i64 - 1920).abs() <= 1);
    }

    #[test]
    fn test_time_conversion_round_trip() {
        let original_ticks = 3840u64;
        let seconds = ticks_to_seconds(original_ticks, 120.0, 480);
        let converted_ticks = seconds_to_ticks(seconds, 120.0, 480);
        assert_eq!(original_ticks, converted_ticks);
    }

    #[test]
    fn test_different_time_signatures() {
        // 3/4 time (3 beats per bar)
        assert_eq!(tick_to_bar_beat(1440, 480, 3), (1, 0)); // Bar 1 in 3/4
        assert_eq!(bar_beat_to_tick(1, 0, 480, 3), 1440);
    }
}
