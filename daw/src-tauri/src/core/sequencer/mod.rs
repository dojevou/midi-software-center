   /// Sequencer Core - Trusty Modules
   ///
   /// Pure functions for timing calculations and sequencer logic.
   /// NO I/O operations - all functions are deterministic and testable.

pub mod timing;

#[allow(unused_imports)]
pub use timing::{
    BarPosition, microseconds_per_tick, tick_to_bar_beat, bar_beat_to_tick,
    ticks_to_seconds, seconds_to_ticks, ticks_to_microseconds,
    microseconds_to_ticks, calculate_bar_position, ticks_per_bar,
};
