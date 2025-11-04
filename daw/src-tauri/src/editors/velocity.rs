// Velocity Editor State and Commands
//
// Handles velocity editing and humanization for MIDI notes.
// Architecture:
// - VelocityEditorState: Trusty Module (pure state)
// - Implementation functions: Grown-up Script
// - Tauri commands: Task-O-Matic

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VelocityError {
    #[error("Invalid velocity value: {0} (must be 1-127)")]
    InvalidVelocity(u8),
    #[error("Note not found: {0}")]
    NoteNotFound(i32),
    #[error("Invalid humanization amount: {0} (must be 0.0-1.0)")]
    InvalidHumanizationAmount(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelocityEditorState {
    pub note_velocities: HashMap<i32, u8>,
    pub modified_notes: HashMap<i32, u8>, // Original velocities for undo
    pub default_velocity: u8,
}

impl VelocityEditorState {
    pub fn new() -> Self {
        Self {
            note_velocities: HashMap::new(),
            modified_notes: HashMap::new(),
            default_velocity: 100,
        }
    }

    pub fn set_velocity_impl(&mut self, note_id: i32, velocity: u8) -> Result<(), VelocityError> {
        if velocity == 0 || velocity > 127 {
            return Err(VelocityError::InvalidVelocity(velocity));
        }

        // Store original velocity if first modification
        if !self.modified_notes.contains_key(&note_id) {
            if let Some(&original) = self.note_velocities.get(&note_id) {
                self.modified_notes.insert(note_id, original);
            }
        }

        self.note_velocities.insert(note_id, velocity);
        Ok(())
    }

    pub fn humanize_velocities_impl(&mut self, amount: f32) -> Result<Vec<i32>, VelocityError> {
        if !(0.0..=1.0).contains(&amount) {
            return Err(VelocityError::InvalidHumanizationAmount(amount));
        }

        let mut modified_notes = Vec::new();
        let mut updates = Vec::new();

        // Collect all updates first
        for (&note_id, &velocity) in self.note_velocities.iter() {
            let original = if !self.modified_notes.contains_key(&note_id) {
                velocity
            } else {
                velocity
            };

            // Simple pseudo-random variation based on note_id
            let variation = ((note_id as f32 * 12345.0).sin() * amount * 20.0) as i32;
            let new_velocity = (original as i32 + variation).clamp(1, 127) as u8;

            updates.push((note_id, velocity, new_velocity));
            modified_notes.push(note_id);
        }

        // Apply updates
        for (note_id, original_velocity, new_velocity) in updates {
            if !self.modified_notes.contains_key(&note_id) {
                self.modified_notes.insert(note_id, original_velocity);
            }
            self.note_velocities.insert(note_id, new_velocity);
        }

        Ok(modified_notes)
    }

    pub fn get_velocity(&self, note_id: i32) -> Option<u8> {
        self.note_velocities.get(&note_id).copied()
    }

    pub fn add_note(&mut self, note_id: i32, velocity: u8) -> Result<(), VelocityError> {
        if velocity == 0 || velocity > 127 {
            return Err(VelocityError::InvalidVelocity(velocity));
        }
        self.note_velocities.insert(note_id, velocity);
        Ok(())
    }

    pub fn remove_note(&mut self, note_id: i32) {
        self.note_velocities.remove(&note_id);
        self.modified_notes.remove(&note_id);
    }

    pub fn reset_note(&mut self, note_id: i32) -> Result<(), VelocityError> {
        if let Some(&original) = self.modified_notes.get(&note_id) {
            self.note_velocities.insert(note_id, original);
            self.modified_notes.remove(&note_id);
            Ok(())
        } else {
            Err(VelocityError::NoteNotFound(note_id))
        }
    }

    pub fn reset_all(&mut self) {
        for (&note_id, &original_velocity) in self.modified_notes.iter() {
            self.note_velocities.insert(note_id, original_velocity);
        }
        self.modified_notes.clear();
    }

    pub fn has_modifications(&self) -> bool {
        !self.modified_notes.is_empty()
    }

    pub fn modification_count(&self) -> usize {
        self.modified_notes.len()
    }
}

impl Default for VelocityEditorState {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri Command Handlers (Task-O-Matic)
use tauri::State;
use std::sync::Mutex;

#[tauri::command]
pub async fn set_velocity(
    state: State<'_, Mutex<VelocityEditorState>>,
    note_id: i32,
    velocity: u8,
) -> Result<(), String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .set_velocity_impl(note_id, velocity)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn humanize_velocities(
    state: State<'_, Mutex<VelocityEditorState>>,
    amount: f32,
) -> Result<Vec<i32>, String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .humanize_velocities_impl(amount)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velocity_editor_state_new() {
        let state = VelocityEditorState::new();
        assert_eq!(state.note_velocities.len(), 0);
        assert_eq!(state.modified_notes.len(), 0);
        assert_eq!(state.default_velocity, 100);
    }

    #[test]
    fn test_set_velocity() {
        let mut state = VelocityEditorState::new();
        state.add_note(1, 80).unwrap();

        state.set_velocity_impl(1, 100).unwrap();
        assert_eq!(state.get_velocity(1), Some(100));
    }

    #[test]
    fn test_set_velocity_stores_original() {
        let mut state = VelocityEditorState::new();
        state.add_note(1, 80).unwrap();

        state.set_velocity_impl(1, 100).unwrap();
        assert_eq!(state.modified_notes.get(&1), Some(&80));
    }

    #[test]
    fn test_set_velocity_invalid_zero() {
        let mut state = VelocityEditorState::new();
        let result = state.set_velocity_impl(1, 0);
        assert!(matches!(result, Err(VelocityError::InvalidVelocity(0))));
    }

    #[test]
    fn test_set_velocity_invalid_high() {
        let mut state = VelocityEditorState::new();
        let result = state.set_velocity_impl(1, 128);
        assert!(matches!(result, Err(VelocityError::InvalidVelocity(128))));
    }

    #[test]
    fn test_set_velocity_boundary_valid() {
        let mut state = VelocityEditorState::new();
        state.add_note(1, 64).unwrap();

        state.set_velocity_impl(1, 1).unwrap();
        assert_eq!(state.get_velocity(1), Some(1));

        state.set_velocity_impl(1, 127).unwrap();
        assert_eq!(state.get_velocity(1), Some(127));
    }

    #[test]
    fn test_humanize_velocities() {
        let mut state = VelocityEditorState::new();
        state.add_note(1, 100).unwrap();
        state.add_note(2, 100).unwrap();
        state.add_note(3, 100).unwrap();

        let modified = state.humanize_velocities_impl(0.5).unwrap();
        assert_eq!(modified.len(), 3);

        // Velocities should be different after humanization
        let v1 = state.get_velocity(1).unwrap();
        let v2 = state.get_velocity(2).unwrap();
        let v3 = state.get_velocity(3).unwrap();

        // At least one should be different (deterministic variation)
        assert!(v1 != v2 || v2 != v3 || v1 != v3);
    }

    #[test]
    fn test_humanize_velocities_zero_amount() {
        let mut state = VelocityEditorState::new();
        state.add_note(1, 100).unwrap();

        let modified = state.humanize_velocities_impl(0.0).unwrap();
        assert_eq!(modified.len(), 1);
        assert_eq!(state.get_velocity(1), Some(100)); // Should remain unchanged
    }

    #[test]
    fn test_humanize_velocities_invalid_amount_negative() {
        let mut state = VelocityEditorState::new();
        let result = state.humanize_velocities_impl(-0.1);
        assert!(matches!(
            result,
            Err(VelocityError::InvalidHumanizationAmount(_))
        ));
    }

    #[test]
    fn test_humanize_velocities_invalid_amount_high() {
        let mut state = VelocityEditorState::new();
        let result = state.humanize_velocities_impl(1.1);
        assert!(matches!(
            result,
            Err(VelocityError::InvalidHumanizationAmount(_))
        ));
    }

    #[test]
    fn test_humanize_velocities_clamping() {
        let mut state = VelocityEditorState::new();
        state.add_note(1, 1).unwrap(); // Very low velocity
        state.add_note(2, 127).unwrap(); // Very high velocity

        state.humanize_velocities_impl(1.0).unwrap();

        let v1 = state.get_velocity(1).unwrap();
        let v2 = state.get_velocity(2).unwrap();

        // Should be clamped to valid range
        assert!(v1 >= 1 && v1 <= 127);
        assert!(v2 >= 1 && v2 <= 127);
    }

    #[test]
    fn test_add_note() {
        let mut state = VelocityEditorState::new();
        state.add_note(1, 100).unwrap();
        assert_eq!(state.get_velocity(1), Some(100));
    }

    #[test]
    fn test_add_note_invalid_velocity() {
        let mut state = VelocityEditorState::new();
        let result = state.add_note(1, 0);
        assert!(matches!(result, Err(VelocityError::InvalidVelocity(0))));
    }

    #[test]
    fn test_remove_note() {
        let mut state = VelocityEditorState::new();
        state.add_note(1, 100).unwrap();
        state.set_velocity_impl(1, 110).unwrap();

        state.remove_note(1);
        assert_eq!(state.get_velocity(1), None);
        assert!(!state.modified_notes.contains_key(&1));
    }

    #[test]
    fn test_reset_note() {
        let mut state = VelocityEditorState::new();
        state.add_note(1, 80).unwrap();
        state.set_velocity_impl(1, 100).unwrap();

        state.reset_note(1).unwrap();
        assert_eq!(state.get_velocity(1), Some(80));
        assert!(!state.modified_notes.contains_key(&1));
    }

    #[test]
    fn test_reset_note_not_modified() {
        let mut state = VelocityEditorState::new();
        state.add_note(1, 80).unwrap();

        let result = state.reset_note(1);
        assert!(matches!(result, Err(VelocityError::NoteNotFound(1))));
    }

    #[test]
    fn test_reset_all() {
        let mut state = VelocityEditorState::new();
        state.add_note(1, 80).unwrap();
        state.add_note(2, 90).unwrap();
        state.add_note(3, 70).unwrap();

        state.set_velocity_impl(1, 100).unwrap();
        state.set_velocity_impl(2, 110).unwrap();
        state.set_velocity_impl(3, 120).unwrap();

        state.reset_all();

        assert_eq!(state.get_velocity(1), Some(80));
        assert_eq!(state.get_velocity(2), Some(90));
        assert_eq!(state.get_velocity(3), Some(70));
        assert_eq!(state.modification_count(), 0);
    }

    #[test]
    fn test_has_modifications() {
        let mut state = VelocityEditorState::new();
        assert!(!state.has_modifications());

        state.add_note(1, 80).unwrap();
        state.set_velocity_impl(1, 100).unwrap();
        assert!(state.has_modifications());

        state.reset_all();
        assert!(!state.has_modifications());
    }

    #[test]
    fn test_modification_count() {
        let mut state = VelocityEditorState::new();
        assert_eq!(state.modification_count(), 0);

        state.add_note(1, 80).unwrap();
        state.add_note(2, 90).unwrap();

        state.set_velocity_impl(1, 100).unwrap();
        assert_eq!(state.modification_count(), 1);

        state.set_velocity_impl(2, 110).unwrap();
        assert_eq!(state.modification_count(), 2);
    }
}
