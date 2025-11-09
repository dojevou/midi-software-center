   /// Piano Roll Undo/Redo Commands - Trusty Module
   ///
   /// Commands for piano roll operations: add/delete/move notes, set velocity, quantize, transpose.
   /// All commands are pure data transformations without I/O.

use super::core::{Command, UndoRedoError, UndoRedoResult};
use crate::editors::piano_roll::{MidiNote, PianoRollState};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Add a note to the piano roll
#[derive(Debug)]
pub struct AddNoteCommand {
    state: Arc<Mutex<PianoRollState>>,
    pitch: u8,
    velocity: u8,
    start_tick: i32,
    duration: i32,
    note_id: Option<i32>,
}

impl AddNoteCommand {
    pub fn new(
        state: Arc<Mutex<PianoRollState>>,
        pitch: u8,
        velocity: u8,
        start_tick: i32,
        duration: i32,
    ) -> Self {
        Self {
            state,
            pitch,
            velocity,
            start_tick,
            duration,
            note_id: None,
        }
    }
}

impl Command for AddNoteCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::ExecutionFailed(format!("Lock error: {}", e)))?;

        let note_id = state
            .add_note_impl(self.pitch, self.velocity, self.start_tick, self.duration)
            .map_err(|e| UndoRedoError::ExecutionFailed(e.to_string()))?;

        self.note_id = Some(note_id);
        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        let note_id = self
            .note_id
            .ok_or_else(|| UndoRedoError::UndoFailed("Note ID not set".to_string()))?;

        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::UndoFailed(format!("Lock error: {}", e)))?;

        state
            .delete_note_impl(note_id)
            .map_err(|e| UndoRedoError::UndoFailed(e.to_string()))?;

        Ok(())
    }

    fn description(&self) -> String {
        format!("Add Note (pitch {})", self.pitch)
    }

    fn memory_usage(&self) -> usize {
        64 // Approximate size of command data
    }
}

/// Delete a note from the piano roll
#[derive(Debug)]
pub struct DeleteNoteCommand {
    state: Arc<Mutex<PianoRollState>>,
    note_id: i32,
    deleted_note: Option<MidiNote>,
}

impl DeleteNoteCommand {
    pub fn new(state: Arc<Mutex<PianoRollState>>, note_id: i32) -> Self {
        Self {
            state,
            note_id,
            deleted_note: None,
        }
    }
}

impl Command for DeleteNoteCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::ExecutionFailed(format!("Lock error: {}", e)))?;

        let note = state
            .delete_note_impl(self.note_id)
            .map_err(|e| UndoRedoError::ExecutionFailed(e.to_string()))?;

        self.deleted_note = Some(note);
        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        let note = self
            .deleted_note
            .as_ref()
            .ok_or_else(|| UndoRedoError::UndoFailed("Deleted note not stored".to_string()))?;

        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::UndoFailed(format!("Lock error: {}", e)))?;

        // Re-add the note with original ID
        state.notes.insert(note.id, note.clone());
        Ok(())
    }

    fn description(&self) -> String {
        format!("Delete Note (ID {})", self.note_id)
    }

    fn memory_usage(&self) -> usize {
        64 + self.deleted_note.as_ref().map(|_| 48).unwrap_or(0)
    }
}

/// Move a note to a new position
#[derive(Debug)]
pub struct MoveNoteCommand {
    state: Arc<Mutex<PianoRollState>>,
    note_id: i32,
    new_tick: i32,
    new_pitch: Option<u8>,
    old_tick: Option<i32>,
    old_pitch: Option<u8>,
}

impl MoveNoteCommand {
    pub fn new(state: Arc<Mutex<PianoRollState>>, note_id: i32, new_tick: i32) -> Self {
        Self {
            state,
            note_id,
            new_tick,
            new_pitch: None,
            old_tick: None,
            old_pitch: None,
        }
    }

    pub fn with_pitch(mut self, new_pitch: u8) -> Self {
        self.new_pitch = Some(new_pitch);
        self
    }
}

impl Command for MoveNoteCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::ExecutionFailed(format!("Lock error: {}", e)))?;

        let note = state
            .notes
            .get(&self.note_id)
            .ok_or_else(|| UndoRedoError::ExecutionFailed("Note not found".to_string()))?;

        // Store old values
        self.old_tick = Some(note.start_tick);
        self.old_pitch = Some(note.pitch);

        // Apply new position
        state
            .move_note_impl(self.note_id, self.new_tick)
            .map_err(|e| UndoRedoError::ExecutionFailed(e.to_string()))?;

        // Apply new pitch if specified
        if let Some(new_pitch) = self.new_pitch {
            if let Some(note) = state.notes.get_mut(&self.note_id) {
                note.pitch = new_pitch;
            }
        }

        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        let old_tick = self
            .old_tick
            .ok_or_else(|| UndoRedoError::UndoFailed("Old tick not stored".to_string()))?;

        let old_pitch = self
            .old_pitch
            .ok_or_else(|| UndoRedoError::UndoFailed("Old pitch not stored".to_string()))?;

        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::UndoFailed(format!("Lock error: {}", e)))?;

        // Restore old position
        state
            .move_note_impl(self.note_id, old_tick)
            .map_err(|e| UndoRedoError::UndoFailed(e.to_string()))?;

        // Restore old pitch
        if let Some(note) = state.notes.get_mut(&self.note_id) {
            note.pitch = old_pitch;
        }

        Ok(())
    }

    fn description(&self) -> String {
        format!("Move Note (ID {})", self.note_id)
    }

    fn memory_usage(&self) -> usize {
        48
    }
}

/// Set velocity for a note
#[derive(Debug)]
pub struct SetVelocityCommand {
    state: Arc<Mutex<PianoRollState>>,
    note_id: i32,
    new_velocity: u8,
    old_velocity: Option<u8>,
}

impl SetVelocityCommand {
    pub fn new(state: Arc<Mutex<PianoRollState>>, note_id: i32, new_velocity: u8) -> Self {
        Self {
            state,
            note_id,
            new_velocity,
            old_velocity: None,
        }
    }
}

impl Command for SetVelocityCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::ExecutionFailed(format!("Lock error: {}", e)))?;

        let note = state
            .notes
            .get_mut(&self.note_id)
            .ok_or_else(|| UndoRedoError::ExecutionFailed("Note not found".to_string()))?;

        self.old_velocity = Some(note.velocity);
        note.velocity = self.new_velocity;

        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        let old_velocity = self
            .old_velocity
            .ok_or_else(|| UndoRedoError::UndoFailed("Old velocity not stored".to_string()))?;

        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::UndoFailed(format!("Lock error: {}", e)))?;

        let note = state
            .notes
            .get_mut(&self.note_id)
            .ok_or_else(|| UndoRedoError::UndoFailed("Note not found".to_string()))?;

        note.velocity = old_velocity;
        Ok(())
    }

    fn description(&self) -> String {
        format!("Set Velocity (note {})", self.note_id)
    }

    fn memory_usage(&self) -> usize {
        32
    }
}

/// Quantize notes to grid
#[derive(Debug)]
pub struct QuantizeNotesCommand {
    state: Arc<Mutex<PianoRollState>>,
    note_ids: Vec<i32>,
    old_positions: HashMap<i32, i32>,
}

impl QuantizeNotesCommand {
    pub fn new(state: Arc<Mutex<PianoRollState>>, note_ids: Vec<i32>) -> Self {
        Self {
            state,
            note_ids,
            old_positions: HashMap::new(),
        }
    }
}

impl Command for QuantizeNotesCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::ExecutionFailed(format!("Lock error: {}", e)))?;

        for &note_id in &self.note_ids {
            if let Some(note) = state.notes.get(&note_id) {
                let old_tick = note.start_tick;
                let new_tick = state.grid_snap.quantize_tick(old_tick);

                self.old_positions.insert(note_id, old_tick);

                // Update note position
                if let Some(note) = state.notes.get_mut(&note_id) {
                    note.start_tick = new_tick;
                }
            }
        }

        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::UndoFailed(format!("Lock error: {}", e)))?;

        for (note_id, old_tick) in &self.old_positions {
            if let Some(note) = state.notes.get_mut(note_id) {
                note.start_tick = *old_tick;
            }
        }

        Ok(())
    }

    fn description(&self) -> String {
        format!("Quantize {} notes", self.note_ids.len())
    }

    fn memory_usage(&self) -> usize {
        48 + self.note_ids.len() * 8 + self.old_positions.len() * 16
    }
}

/// Transpose notes by semitones
#[derive(Debug)]
pub struct TransposeCommand {
    state: Arc<Mutex<PianoRollState>>,
    note_ids: Vec<i32>,
    semitones: i8,
    old_pitches: HashMap<i32, u8>,
}

impl TransposeCommand {
    pub fn new(state: Arc<Mutex<PianoRollState>>, note_ids: Vec<i32>, semitones: i8) -> Self {
        Self {
            state,
            note_ids,
            semitones,
            old_pitches: HashMap::new(),
        }
    }
}

impl Command for TransposeCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::ExecutionFailed(format!("Lock error: {}", e)))?;

        for &note_id in &self.note_ids {
            if let Some(note) = state.notes.get_mut(&note_id) {
                self.old_pitches.insert(note_id, note.pitch);

                let new_pitch = (note.pitch as i16 + self.semitones as i16).clamp(0, 127) as u8;
                note.pitch = new_pitch;
            }
        }

        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|e| UndoRedoError::UndoFailed(format!("Lock error: {}", e)))?;

        for (note_id, old_pitch) in &self.old_pitches {
            if let Some(note) = state.notes.get_mut(note_id) {
                note.pitch = *old_pitch;
            }
        }

        Ok(())
    }

    fn description(&self) -> String {
        format!(
            "Transpose {} notes by {} semitones",
            self.note_ids.len(),
            self.semitones
        )
    }

    fn memory_usage(&self) -> usize {
        48 + self.note_ids.len() * 8 + self.old_pitches.len() * 12
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editors::piano_roll::GridSnap;

    fn create_test_state() -> Arc<Mutex<PianoRollState>> {
        Arc::new(Mutex::new(PianoRollState::new()))
    }

    #[test]
    fn test_add_note_command() {
        let state = create_test_state();
        let mut cmd = AddNoteCommand::new(state.clone(), 60, 100, 0, 480);

        // Execute
        cmd.execute().unwrap();
        assert_eq!(state.lock().unwrap().notes.len(), 1);
        assert!(cmd.note_id.is_some());

        // Undo
        cmd.undo().unwrap();
        assert_eq!(state.lock().unwrap().notes.len(), 0);

        // Redo
        cmd.redo().unwrap();
        assert_eq!(state.lock().unwrap().notes.len(), 1);
    }

    #[test]
    fn test_add_note_command_description() {
        let state = create_test_state();
        let cmd = AddNoteCommand::new(state, 60, 100, 0, 480);
        assert_eq!(cmd.description(), "Add Note (pitch 60)");
    }

    #[test]
    fn test_delete_note_command() {
        let state = create_test_state();

        // Add a note first
        let note_id = {
            let mut s = state.lock().unwrap();
            s.add_note_impl(60, 100, 0, 480).unwrap()
        };

        let mut cmd = DeleteNoteCommand::new(state.clone(), note_id);

        // Execute
        cmd.execute().unwrap();
        assert_eq!(state.lock().unwrap().notes.len(), 0);
        assert!(cmd.deleted_note.is_some());

        // Undo
        cmd.undo().unwrap();
        assert_eq!(state.lock().unwrap().notes.len(), 1);

        // Redo
        cmd.redo().unwrap();
        assert_eq!(state.lock().unwrap().notes.len(), 0);
    }

    #[test]
    fn test_delete_note_command_nonexistent() {
        let state = create_test_state();
        let mut cmd = DeleteNoteCommand::new(state.clone(), 999);

        let result = cmd.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_move_note_command() {
        let state = create_test_state();

        let note_id = {
            let mut s = state.lock().unwrap();
            s.add_note_impl(60, 100, 0, 480).unwrap()
        };

        let mut cmd = MoveNoteCommand::new(state.clone(), note_id, 960);

        // Execute
        cmd.execute().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_id).unwrap().start_tick, 960);
        }

        // Undo
        cmd.undo().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_id).unwrap().start_tick, 0);
        }

        // Redo
        cmd.redo().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_id).unwrap().start_tick, 960);
        }
    }

    #[test]
    fn test_move_note_command_with_pitch() {
        let state = create_test_state();

        let note_id = {
            let mut s = state.lock().unwrap();
            s.add_note_impl(60, 100, 0, 480).unwrap()
        };

        let mut cmd = MoveNoteCommand::new(state.clone(), note_id, 960).with_pitch(64);

        cmd.execute().unwrap();
        {
            let s = state.lock().unwrap();
            let note = s.notes.get(&note_id).unwrap();
            assert_eq!(note.start_tick, 960);
            assert_eq!(note.pitch, 64);
        }

        cmd.undo().unwrap();
        {
            let s = state.lock().unwrap();
            let note = s.notes.get(&note_id).unwrap();
            assert_eq!(note.start_tick, 0);
            assert_eq!(note.pitch, 60);
        }
    }

    #[test]
    fn test_set_velocity_command() {
        let state = create_test_state();

        let note_id = {
            let mut s = state.lock().unwrap();
            s.add_note_impl(60, 100, 0, 480).unwrap()
        };

        let mut cmd = SetVelocityCommand::new(state.clone(), note_id, 64);

        // Execute
        cmd.execute().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_id).unwrap().velocity, 64);
        }

        // Undo
        cmd.undo().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_id).unwrap().velocity, 100);
        }

        // Redo
        cmd.redo().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_id).unwrap().velocity, 64);
        }
    }

    #[test]
    fn test_quantize_notes_command() {
        let state = create_test_state();

        let note_ids = {
            let mut s = state.lock().unwrap();
            s.grid_snap = GridSnap::Quarter; // 480 ticks
            s.quantize_enabled = false; // Disable auto-quantize
            vec![
                s.add_note_impl(60, 100, 100, 480).unwrap(),
                s.add_note_impl(62, 100, 600, 480).unwrap(),
            ]
        };

        let mut cmd = QuantizeNotesCommand::new(state.clone(), note_ids.clone());

        // Execute
        cmd.execute().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_ids[0]).unwrap().start_tick, 0); // Quantized to 0
            assert_eq!(s.notes.get(&note_ids[1]).unwrap().start_tick, 480); // Quantized to 480
        }

        // Undo
        cmd.undo().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_ids[0]).unwrap().start_tick, 100);
            assert_eq!(s.notes.get(&note_ids[1]).unwrap().start_tick, 600);
        }
    }

    #[test]
    fn test_transpose_command() {
        let state = create_test_state();

        let note_ids = {
            let mut s = state.lock().unwrap();
            vec![
                s.add_note_impl(60, 100, 0, 480).unwrap(),
                s.add_note_impl(62, 100, 480, 480).unwrap(),
            ]
        };

        let mut cmd = TransposeCommand::new(state.clone(), note_ids.clone(), 5);

        // Execute
        cmd.execute().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_ids[0]).unwrap().pitch, 65);
            assert_eq!(s.notes.get(&note_ids[1]).unwrap().pitch, 67);
        }

        // Undo
        cmd.undo().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_ids[0]).unwrap().pitch, 60);
            assert_eq!(s.notes.get(&note_ids[1]).unwrap().pitch, 62);
        }

        // Redo
        cmd.redo().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_ids[0]).unwrap().pitch, 65);
            assert_eq!(s.notes.get(&note_ids[1]).unwrap().pitch, 67);
        }
    }

    #[test]
    fn test_transpose_command_clamping() {
        let state = create_test_state();

        let note_id = {
            let mut s = state.lock().unwrap();
            s.add_note_impl(120, 100, 0, 480).unwrap()
        };

        let mut cmd = TransposeCommand::new(state.clone(), vec![note_id], 10);

        cmd.execute().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_id).unwrap().pitch, 127); // Clamped
        }
    }

    #[test]
    fn test_transpose_command_negative() {
        let state = create_test_state();

        let note_id = {
            let mut s = state.lock().unwrap();
            s.add_note_impl(60, 100, 0, 480).unwrap()
        };

        let mut cmd = TransposeCommand::new(state.clone(), vec![note_id], -12);

        cmd.execute().unwrap();
        {
            let s = state.lock().unwrap();
            assert_eq!(s.notes.get(&note_id).unwrap().pitch, 48);
        }
    }

    #[test]
    fn test_command_memory_usage() {
        let state = create_test_state();

        let add_cmd = AddNoteCommand::new(state.clone(), 60, 100, 0, 480);
        assert!(add_cmd.memory_usage() > 0);

        let delete_cmd = DeleteNoteCommand::new(state.clone(), 1);
        assert!(delete_cmd.memory_usage() > 0);

        let move_cmd = MoveNoteCommand::new(state.clone(), 1, 960);
        assert!(move_cmd.memory_usage() > 0);

        let velocity_cmd = SetVelocityCommand::new(state.clone(), 1, 64);
        assert!(velocity_cmd.memory_usage() > 0);

        let quantize_cmd = QuantizeNotesCommand::new(state.clone(), vec![1, 2, 3]);
        assert!(quantize_cmd.memory_usage() > 0);

        let transpose_cmd = TransposeCommand::new(state.clone(), vec![1, 2, 3], 5);
        assert!(transpose_cmd.memory_usage() > 0);
    }
}
