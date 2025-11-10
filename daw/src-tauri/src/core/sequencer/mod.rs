/// Sequencer Core - Trusty Modules
///
/// Pure functions for timing calculations and sequencer logic.
/// NO I/O operations - all functions are deterministic and testable.
pub mod timing;

#[allow(unused_imports)]
pub use timing::{
    bar_beat_to_tick, calculate_bar_position, microseconds_per_tick, microseconds_to_ticks,
    seconds_to_ticks, tick_to_bar_beat, ticks_per_bar, ticks_to_microseconds, ticks_to_seconds,
    BarPosition,
};
