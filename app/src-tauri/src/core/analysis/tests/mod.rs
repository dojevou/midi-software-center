//! Test modules for core analysis functions
//!
//! This directory contains unit tests for all analysis modules:
//! - drum_analyzer_test.rs - Drum-specific MIDI analysis tests (Phases 1-5)
//! - real_world_validation_test.rs - Real-world validation with actual drum files (Phase 6)
//! - phase2_validation_test.rs - Phase 2 filename metadata extraction validation
//! - chord_analyzer_extended_test.rs - Extended chord analysis tests (complex chords, inversions, edge cases)

mod chord_analyzer_extended_test;
mod drum_analyzer_test;
mod phase2_validation_test;
mod real_world_validation_test;
