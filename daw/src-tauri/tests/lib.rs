mod commands;
/// Integration tests for DAW Tauri commands
///
/// Test organization:
/// - tests/common/ - Shared test infrastructure (TestDatabase, mocks, fixtures)
/// - tests/commands/ - Command-specific integration tests
/// - tests/integration/ - Cross-command workflow tests
/// - tests/link_test.rs - Ableton Link integration tests
/// - tests/learn_test.rs - MIDI Learn tests
/// - tests/notation_test.rs - Score rendering tests
///
/// NOTE: Phase 5-8 generated tests disabled temporarily (_disabled_tests/)
/// These tests will be remediated in Phase 9.5
mod common;
mod link_test;
mod learn_test;
mod notation_test;

// Integration tests go here
pub mod integration {
    // Placeholder for integration tests (Phase 5.5)
}
