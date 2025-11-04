// DAW Editor Modules
//
// This module contains all editor implementations for the DAW application.
// Each editor follows the Three Archetypes pattern:
// - Pure state structures (Trusty Module)
// - Implementation logic (Grown-up Script)
// - Tauri command handlers (Task-O-Matic)

pub mod controller;
pub mod piano_roll;
pub mod tempo;
pub mod velocity;

// Re-export common types for convenience
pub use controller::{
    add_cc_point, delete_cc_point, AutomationPoint, ControllerEditorState, CCError,
};
pub use piano_roll::{
    add_note, delete_note, move_note, select_note, set_quantize_grid, GridSnap, NoteSelection,
    PianoRollError, PianoRollState,
};
pub use tempo::{
    tempo_editor_add_tempo_ramp, tempo_editor_set_tempo, TempoEditorState, TempoError, TempoPoint,
};
pub use velocity::{
    humanize_velocities, set_velocity, VelocityEditorState, VelocityError,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports_piano_roll() {
        let state = PianoRollState::default();
        assert_eq!(state.selected_notes.len(), 0);
    }

    #[test]
    fn test_module_exports_velocity() {
        let state = VelocityEditorState::default();
        assert!(state.modified_notes.is_empty());
    }

    #[test]
    fn test_module_exports_controller() {
        let state = ControllerEditorState::new(1);
        assert_eq!(state.cc_number, 1);
    }

    #[test]
    fn test_module_exports_tempo() {
        let state = TempoEditorState::default();
        assert_eq!(state.tempo_points.len(), 1); // Default tempo point
    }

    #[test]
    fn test_grid_snap_serialization() {
        let grid = GridSnap::Eighth;
        let json = serde_json::to_string(&grid).unwrap();
        let deserialized: GridSnap = serde_json::from_str(&json).unwrap();
        assert_eq!(grid, deserialized);
    }
}
