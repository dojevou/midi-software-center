// Piano Roll Editor State and Commands
//
// Handles note selection, editing, quantization, and grid snapping for the piano roll view.
// Architecture:
// - PianoRollState: Trusty Module (pure state)
// - Implementation functions: Grown-up Script
// - Tauri commands: Task-O-Matic

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PianoRollError {
    #[error("Invalid pitch value: {0} (must be 0-127)")]
    InvalidPitch(u8),
    #[error("Invalid velocity value: {0} (must be 1-127)")]
    InvalidVelocity(u8),
    #[error("Note not found: {0}")]
    NoteNotFound(i32),
    #[error("Invalid duration: {0} (must be positive)")]
    InvalidDuration(i32),
    #[error("Invalid tick position: {0} (must be non-negative)")]
    InvalidTick(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridSnap {
    None,
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    QuarterTriplet,
    EighthTriplet,
    SixteenthTriplet,
}

impl GridSnap {
    /// Returns the tick divisor for this grid snap setting
    /// Assumes 480 ticks per quarter note (standard MIDI resolution)
    pub fn tick_divisor(&self) -> i32 {
        match self {
            GridSnap::None => 1,
            GridSnap::Whole => 1920,          // 4 beats
            GridSnap::Half => 960,            // 2 beats
            GridSnap::Quarter => 480,         // 1 beat
            GridSnap::Eighth => 240,          // 1/2 beat
            GridSnap::Sixteenth => 120,       // 1/4 beat
            GridSnap::ThirtySecond => 60,     // 1/8 beat
            GridSnap::QuarterTriplet => 320,  // 1/3 beat
            GridSnap::EighthTriplet => 160,   // 1/6 beat
            GridSnap::SixteenthTriplet => 80, // 1/12 beat
        }
    }

    /// Quantize a tick position to the nearest grid line
    pub fn quantize_tick(&self, tick: i32) -> i32 {
        if *self == GridSnap::None {
            return tick;
        }
        let divisor = self.tick_divisor();
        ((tick + divisor / 2) / divisor) * divisor
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiNote {
    pub id: i32,
    pub pitch: u8,
    pub velocity: u8,
    pub start_tick: i32,
    pub duration: i32,
    pub channel: u8,
}

impl MidiNote {
    pub fn new(
        id: i32,
        pitch: u8,
        velocity: u8,
        start_tick: i32,
        duration: i32,
    ) -> Result<Self, PianoRollError> {
        if pitch > 127 {
            return Err(PianoRollError::InvalidPitch(pitch));
        }
        if velocity == 0 || velocity > 127 {
            return Err(PianoRollError::InvalidVelocity(velocity));
        }
        if start_tick < 0 {
            return Err(PianoRollError::InvalidTick(start_tick));
        }
        if duration <= 0 {
            return Err(PianoRollError::InvalidDuration(duration));
        }

        Ok(Self { id, pitch, velocity, start_tick, duration, channel: 0 })
    }

    pub fn end_tick(&self) -> i32 {
        self.start_tick + self.duration
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSelection {
    pub selected_notes: HashSet<i32>,
}

impl NoteSelection {
    pub fn new() -> Self {
        Self { selected_notes: HashSet::new() }
    }

    pub fn select(&mut self, note_id: i32, extend: bool) {
        if !extend {
            self.selected_notes.clear();
        }
        self.selected_notes.insert(note_id);
    }

    pub fn deselect(&mut self, note_id: i32) {
        self.selected_notes.remove(&note_id);
    }

    pub fn clear(&mut self) {
        self.selected_notes.clear();
    }

    pub fn is_selected(&self, note_id: i32) -> bool {
        self.selected_notes.contains(&note_id)
    }

    pub fn count(&self) -> usize {
        self.selected_notes.len()
    }
}

impl Default for NoteSelection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PianoRollState {
    pub notes: HashMap<i32, MidiNote>,
    pub selected_notes: HashSet<i32>,
    pub grid_snap: GridSnap,
    pub next_note_id: i32,
    pub quantize_enabled: bool,
}

impl PianoRollState {
    pub fn new() -> Self {
        Self {
            notes: HashMap::new(),
            selected_notes: HashSet::new(),
            grid_snap: GridSnap::Sixteenth,
            next_note_id: 1,
            quantize_enabled: true,
        }
    }

    pub fn add_note_impl(
        &mut self,
        pitch: u8,
        velocity: u8,
        start_tick: i32,
        duration: i32,
    ) -> Result<i32, PianoRollError> {
        let quantized_tick = if self.quantize_enabled {
            self.grid_snap.quantize_tick(start_tick)
        } else {
            start_tick
        };

        let note = MidiNote::new(self.next_note_id, pitch, velocity, quantized_tick, duration)?;
        let note_id = note.id;
        self.notes.insert(note_id, note);
        self.next_note_id += 1;
        Ok(note_id)
    }

    pub fn delete_note_impl(&mut self, note_id: i32) -> Result<MidiNote, PianoRollError> {
        self.notes.remove(&note_id).ok_or(PianoRollError::NoteNotFound(note_id))
    }

    pub fn move_note_impl(&mut self, note_id: i32, new_tick: i32) -> Result<(), PianoRollError> {
        if new_tick < 0 {
            return Err(PianoRollError::InvalidTick(new_tick));
        }

        let note = self.notes.get_mut(&note_id).ok_or(PianoRollError::NoteNotFound(note_id))?;

        let quantized_tick = if self.quantize_enabled {
            self.grid_snap.quantize_tick(new_tick)
        } else {
            new_tick
        };

        note.start_tick = quantized_tick;
        Ok(())
    }

    pub fn select_note_impl(&mut self, note_id: i32, extend: bool) -> Result<(), PianoRollError> {
        if !self.notes.contains_key(&note_id) {
            return Err(PianoRollError::NoteNotFound(note_id));
        }

        if !extend {
            self.selected_notes.clear();
        }
        self.selected_notes.insert(note_id);
        Ok(())
    }

    pub fn set_quantize_grid_impl(&mut self, grid: GridSnap) {
        self.grid_snap = grid;
    }

    pub fn get_note(&self, note_id: i32) -> Option<&MidiNote> {
        self.notes.get(&note_id)
    }

    pub fn get_notes_in_range(&self, start_tick: i32, end_tick: i32) -> Vec<&MidiNote> {
        self.notes
            .values()
            .filter(|note| note.start_tick < end_tick && note.end_tick() > start_tick)
            .collect()
    }
}

impl Default for PianoRollState {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri Command Handlers (Task-O-Matic)
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub async fn select_note(
    state: State<'_, Mutex<PianoRollState>>,
    note_id: i32,
    extend: bool,
) -> Result<(), String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .select_note_impl(note_id, extend)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_note(
    state: State<'_, Mutex<PianoRollState>>,
    pitch: u8,
    velocity: u8,
    start_tick: i32,
    duration: i32,
) -> Result<i32, String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .add_note_impl(pitch, velocity, start_tick, duration)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_note(
    state: State<'_, Mutex<PianoRollState>>,
    note_id: i32,
) -> Result<(), String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .delete_note_impl(note_id)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn move_note(
    state: State<'_, Mutex<PianoRollState>>,
    note_id: i32,
    new_tick: i32,
) -> Result<(), String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .move_note_impl(note_id, new_tick)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_quantize_grid(
    state: State<'_, Mutex<PianoRollState>>,
    grid: GridSnap,
) -> Result<(), String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .set_quantize_grid_impl(grid);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_snap_tick_divisor() {
        assert_eq!(GridSnap::Quarter.tick_divisor(), 480);
        assert_eq!(GridSnap::Eighth.tick_divisor(), 240);
        assert_eq!(GridSnap::Sixteenth.tick_divisor(), 120);
    }

    #[test]
    fn test_grid_snap_quantize_tick() {
        let grid = GridSnap::Quarter;
        assert_eq!(grid.quantize_tick(0), 0);
        assert_eq!(grid.quantize_tick(100), 0);
        assert_eq!(grid.quantize_tick(240), 480);
        assert_eq!(grid.quantize_tick(600), 480);
        assert_eq!(grid.quantize_tick(720), 960);
    }

    #[test]
    fn test_grid_snap_none() {
        let grid = GridSnap::None;
        assert_eq!(grid.quantize_tick(123), 123);
        assert_eq!(grid.quantize_tick(456), 456);
    }

    #[test]
    fn test_midi_note_creation() {
        let note = MidiNote::new(1, 60, 100, 0, 480).unwrap();
        assert_eq!(note.pitch, 60);
        assert_eq!(note.velocity, 100);
        assert_eq!(note.start_tick, 0);
        assert_eq!(note.duration, 480);
        assert_eq!(note.end_tick(), 480);
    }

    #[test]
    fn test_midi_note_invalid_pitch() {
        let result = MidiNote::new(1, 128, 100, 0, 480);
        assert!(matches!(result, Err(PianoRollError::InvalidPitch(128))));
    }

    #[test]
    fn test_midi_note_invalid_velocity() {
        let result = MidiNote::new(1, 60, 0, 0, 480);
        assert!(matches!(result, Err(PianoRollError::InvalidVelocity(0))));

        let result = MidiNote::new(1, 60, 128, 0, 480);
        assert!(matches!(result, Err(PianoRollError::InvalidVelocity(128))));
    }

    #[test]
    fn test_midi_note_invalid_tick() {
        let result = MidiNote::new(1, 60, 100, -1, 480);
        assert!(matches!(result, Err(PianoRollError::InvalidTick(-1))));
    }

    #[test]
    fn test_midi_note_invalid_duration() {
        let result = MidiNote::new(1, 60, 100, 0, 0);
        assert!(matches!(result, Err(PianoRollError::InvalidDuration(0))));

        let result = MidiNote::new(1, 60, 100, 0, -10);
        assert!(matches!(result, Err(PianoRollError::InvalidDuration(-10))));
    }

    #[test]
    fn test_note_selection_new() {
        let selection = NoteSelection::new();
        assert_eq!(selection.count(), 0);
    }

    #[test]
    fn test_note_selection_select() {
        let mut selection = NoteSelection::new();
        selection.select(1, false);
        assert!(selection.is_selected(1));
        assert_eq!(selection.count(), 1);
    }

    #[test]
    fn test_note_selection_extend() {
        let mut selection = NoteSelection::new();
        selection.select(1, false);
        selection.select(2, true);
        assert!(selection.is_selected(1));
        assert!(selection.is_selected(2));
        assert_eq!(selection.count(), 2);
    }

    #[test]
    fn test_note_selection_replace() {
        let mut selection = NoteSelection::new();
        selection.select(1, false);
        selection.select(2, false);
        assert!(!selection.is_selected(1));
        assert!(selection.is_selected(2));
        assert_eq!(selection.count(), 1);
    }

    #[test]
    fn test_note_selection_deselect() {
        let mut selection = NoteSelection::new();
        selection.select(1, false);
        selection.deselect(1);
        assert!(!selection.is_selected(1));
        assert_eq!(selection.count(), 0);
    }

    #[test]
    fn test_note_selection_clear() {
        let mut selection = NoteSelection::new();
        selection.select(1, false);
        selection.select(2, true);
        selection.clear();
        assert_eq!(selection.count(), 0);
    }

    #[test]
    fn test_piano_roll_state_new() {
        let state = PianoRollState::new();
        assert_eq!(state.notes.len(), 0);
        assert_eq!(state.selected_notes.len(), 0);
        assert_eq!(state.next_note_id, 1);
    }

    #[test]
    fn test_piano_roll_add_note() {
        let mut state = PianoRollState::new();
        let note_id = state.add_note_impl(60, 100, 0, 480).unwrap();
        assert_eq!(note_id, 1);
        assert_eq!(state.notes.len(), 1);
        assert_eq!(state.next_note_id, 2);
    }

    #[test]
    fn test_piano_roll_add_note_quantized() {
        let mut state = PianoRollState::new();
        state.grid_snap = GridSnap::Quarter;
        state.quantize_enabled = true;

        let note_id = state.add_note_impl(60, 100, 100, 480).unwrap();
        let note = state.get_note(note_id).unwrap();
        assert_eq!(note.start_tick, 0); // Quantized to nearest quarter
    }

    #[test]
    fn test_piano_roll_add_note_unquantized() {
        let mut state = PianoRollState::new();
        state.quantize_enabled = false;

        let note_id = state.add_note_impl(60, 100, 100, 480).unwrap();
        let note = state.get_note(note_id).unwrap();
        assert_eq!(note.start_tick, 100); // Not quantized
    }

    #[test]
    fn test_piano_roll_delete_note() {
        let mut state = PianoRollState::new();
        let note_id = state.add_note_impl(60, 100, 0, 480).unwrap();
        let deleted = state.delete_note_impl(note_id).unwrap();
        assert_eq!(deleted.pitch, 60);
        assert_eq!(state.notes.len(), 0);
    }

    #[test]
    fn test_piano_roll_delete_note_not_found() {
        let mut state = PianoRollState::new();
        let result = state.delete_note_impl(999);
        assert!(matches!(result, Err(PianoRollError::NoteNotFound(999))));
    }

    #[test]
    fn test_piano_roll_move_note() {
        let mut state = PianoRollState::new();
        let note_id = state.add_note_impl(60, 100, 0, 480).unwrap();
        state.move_note_impl(note_id, 960).unwrap();
        let note = state.get_note(note_id).unwrap();
        assert_eq!(note.start_tick, 960);
    }

    #[test]
    fn test_piano_roll_move_note_quantized() {
        let mut state = PianoRollState::new();
        state.grid_snap = GridSnap::Quarter;
        state.quantize_enabled = true;

        let note_id = state.add_note_impl(60, 100, 0, 480).unwrap();
        state.move_note_impl(note_id, 600).unwrap();
        let note = state.get_note(note_id).unwrap();
        assert_eq!(note.start_tick, 480); // Quantized
    }

    #[test]
    fn test_piano_roll_move_note_invalid_tick() {
        let mut state = PianoRollState::new();
        let note_id = state.add_note_impl(60, 100, 0, 480).unwrap();
        let result = state.move_note_impl(note_id, -100);
        assert!(matches!(result, Err(PianoRollError::InvalidTick(-100))));
    }

    #[test]
    fn test_piano_roll_move_note_not_found() {
        let mut state = PianoRollState::new();
        let result = state.move_note_impl(999, 960);
        assert!(matches!(result, Err(PianoRollError::NoteNotFound(999))));
    }

    #[test]
    fn test_piano_roll_select_note() {
        let mut state = PianoRollState::new();
        let note_id = state.add_note_impl(60, 100, 0, 480).unwrap();
        state.select_note_impl(note_id, false).unwrap();
        assert!(state.selected_notes.contains(&note_id));
    }

    #[test]
    fn test_piano_roll_select_note_extend() {
        let mut state = PianoRollState::new();
        let note1 = state.add_note_impl(60, 100, 0, 480).unwrap();
        let note2 = state.add_note_impl(62, 100, 480, 480).unwrap();

        state.select_note_impl(note1, false).unwrap();
        state.select_note_impl(note2, true).unwrap();

        assert!(state.selected_notes.contains(&note1));
        assert!(state.selected_notes.contains(&note2));
    }

    #[test]
    fn test_piano_roll_select_note_replace() {
        let mut state = PianoRollState::new();
        let note1 = state.add_note_impl(60, 100, 0, 480).unwrap();
        let note2 = state.add_note_impl(62, 100, 480, 480).unwrap();

        state.select_note_impl(note1, false).unwrap();
        state.select_note_impl(note2, false).unwrap();

        assert!(!state.selected_notes.contains(&note1));
        assert!(state.selected_notes.contains(&note2));
    }

    #[test]
    fn test_piano_roll_select_note_not_found() {
        let mut state = PianoRollState::new();
        let result = state.select_note_impl(999, false);
        assert!(matches!(result, Err(PianoRollError::NoteNotFound(999))));
    }

    #[test]
    fn test_piano_roll_set_quantize_grid() {
        let mut state = PianoRollState::new();
        state.set_quantize_grid_impl(GridSnap::Eighth);
        assert_eq!(state.grid_snap, GridSnap::Eighth);
    }

    #[test]
    fn test_piano_roll_get_notes_in_range() {
        let mut state = PianoRollState::new();
        state.add_note_impl(60, 100, 0, 480).unwrap();
        state.add_note_impl(62, 100, 480, 480).unwrap();
        state.add_note_impl(64, 100, 960, 480).unwrap();

        let notes = state.get_notes_in_range(400, 600);
        assert_eq!(notes.len(), 2); // First and second notes overlap this range
    }

    #[test]
    fn test_piano_roll_get_notes_in_range_empty() {
        let mut state = PianoRollState::new();
        state.add_note_impl(60, 100, 0, 480).unwrap();

        let notes = state.get_notes_in_range(1000, 2000);
        assert_eq!(notes.len(), 0);
    }
}
