//! Ableton Link tests

use midi_software_center_daw::midi::link::{AbletonLink, LinkState};

#[test]
fn test_link_creation() {
    let link = AbletonLink::new(120.0);
    assert_eq!(link.tempo(), 120.0);
    assert!(!link.is_enabled());
}

#[test]
fn test_tempo_change() {
    let link = AbletonLink::new(120.0);
    link.set_tempo(140.0);
    assert_eq!(link.tempo(), 140.0);
}

#[test]
fn test_tempo_clamping() {
    let link = AbletonLink::new(120.0);
    link.set_tempo(10.0); // Below min (20.0)
    assert_eq!(link.tempo(), 20.0);
    link.set_tempo(1000.0); // Above max (999.0)
    assert_eq!(link.tempo(), 999.0);
}

#[test]
fn test_quantum() {
    let link = AbletonLink::new(120.0);
    assert_eq!(link.quantum(), 4.0); // Default
    link.set_quantum(8.0);
    assert_eq!(link.quantum(), 8.0);
}

#[test]
fn test_quantum_clamping() {
    let link = AbletonLink::new(120.0);
    link.set_quantum(0.5); // Below min (1.0)
    assert_eq!(link.quantum(), 1.0);
    link.set_quantum(20.0); // Above max (16.0)
    assert_eq!(link.quantum(), 16.0);
}

#[test]
fn test_start_stop() {
    let link = AbletonLink::new(120.0);
    assert!(!link.is_playing());
    link.start();
    assert!(link.is_playing());
    link.stop();
    assert!(!link.is_playing());
}

#[test]
fn test_link_state() {
    let link = AbletonLink::new(120.0);
    let state = link.state();
    assert!(!state.enabled);
    assert_eq!(state.tempo, 120.0);
    assert_eq!(state.num_peers, 0);
    assert_eq!(state.quantum, 4.0);
    assert!(!state.is_playing);
}

#[test]
fn test_enable_disable() {
    let link = AbletonLink::new(120.0);
    assert!(!link.is_enabled());
    link.set_enabled(true);
    assert!(link.is_enabled());
    link.set_enabled(false);
    assert!(!link.is_enabled());
}

#[test]
fn test_num_peers_initial() {
    let link = AbletonLink::new(120.0);
    assert_eq!(link.num_peers(), 0);
}

#[test]
fn test_beat_and_phase() {
    let link = AbletonLink::new(120.0);
    assert_eq!(link.beat(), 0.0);
    assert_eq!(link.phase(), 0.0);
}

#[test]
fn test_event_subscription() {
    let link = AbletonLink::new(120.0);
    let _rx = link.subscribe();
    // Just verify subscription works without panic
}

#[test]
fn test_start_stop_sync() {
    let link = AbletonLink::new(120.0);
    // Default is true
    link.set_start_stop_sync(false);
    // With start_stop_sync disabled, start/stop should not change playing state
    // (implementation returns early if start_stop_sync is false)
    link.start();
    assert!(!link.is_playing()); // Should NOT be playing because sync is off
}
