/// MIDI Library Shared Code
///
/// This crate contains all shared functionality used by:
/// - Pipeline (import, process, analyze)
/// - DAW (playback, sequence, MIDI out)
///
/// ## Structure
///
/// - `core::midi` - MIDI parsing and types
/// - `core::analysis` - Musical analysis (BPM, key detection, etc.)
/// - `db::models` - Database model types
/// - `db::repositories` - Database access layer
/// - `health` - Service health monitoring
pub mod core;
pub mod db;
pub mod health;
pub mod logging;

// Re-export top-level modules for convenience
pub use core::analysis;
pub use core::midi;
pub use db::{models, repositories};
#[cfg(feature = "database")]
pub use health::HealthChecker;
pub use health::{HealthStatus, ServiceHealth, SystemHealth};
pub use logging::{init_logging, LogConfig, LogContext, LogEvent, PerformanceMetrics};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
