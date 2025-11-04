//! Window Management System for DAW
//!
//! Provides window state management, specialized for DAW windows including
//! the main DAW window, Mixer, and integration with Database/Pipeline windows.
//!
//! # Architecture
//!
//! - `state`: DAW-specific window state structures (playback, tracks, transport)
//! - Commands are in `commands::window` module
//!
//! # Window Types
//!
//! - **DAW Window**: Main sequencer/piano roll window with transport controls
//! - **Mixer Window**: Channel strip mixer with routing and effects
//! - **Database Window**: File browser and search (shared from Pipeline)
//! - **Pipeline Window**: Batch import and analysis (shared from Pipeline)

pub mod state;

pub use state::{
    DAWWindowState, PlaybackState, TrackInfo, TransportInfo,
    MixerWindowState, MixerChannel, ChannelType,
    PlaybackPosition,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_state_transitions() {
        let state = PlaybackState::Stopped;
        assert_eq!(state, PlaybackState::Stopped);
    }

    #[test]
    fn test_track_info_creation() {
        let track = TrackInfo::new(1, "Piano");
        assert_eq!(track.id, 1);
        assert_eq!(track.label, "Piano");
        assert!(track.visible);
        assert!(!track.muted);
        assert!(!track.soloed);
    }

    #[test]
    fn test_transport_info_defaults() {
        let transport = TransportInfo::default();
        assert_eq!(transport.bpm, 120.0);
        assert_eq!(transport.time_signature_numerator, 4);
        assert_eq!(transport.time_signature_denominator, 4);
        assert_eq!(transport.key_signature, "C");
    }
}
