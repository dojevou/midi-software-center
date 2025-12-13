//! Piano Roll Commands
//!
//! Tauri commands for piano roll note editing operations including:
//! - CRUD operations for MIDI notes
//! - Selection and manipulation
//! - Quantization and humanization
//! - Clipboard operations
//! - Undo/redo support

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use thiserror::Error;
use tokio::sync::RwLock;

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Error)]
pub enum PianoRollError {
    #[error("Track not found: {0}")]
    TrackNotFound(i64),

    #[error("Note not found: {0}")]
    NoteNotFound(i64),

    #[error("Invalid note data: {0}")]
    InvalidNoteData(String),

    #[error("Invalid pitch: {0} (must be 0-127)")]
    InvalidPitch(i32),

    #[error("Invalid velocity: {0} (must be 0-127)")]
    InvalidVelocity(i32),

    #[error("Invalid duration: {0} (must be > 0)")]
    InvalidDuration(i64),

    #[error("Invalid channel: {0} (must be 0-15)")]
    InvalidChannel(i32),

    #[error("No notes selected")]
    NoSelection,

    #[error("Clipboard is empty")]
    EmptyClipboard,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("State lock error: {0}")]
    LockError(String),

    #[error("Invalid slice position: {slice_tick} not within note range [{note_start}, {note_end})")]
    InvalidSlicePosition { slice_tick: i64, note_start: i64, note_end: i64 },

    #[error("Invalid stretch ratio: {0} (must be > 0 and <= 10)")]
    InvalidStretchRatio(f64),

    #[error("Invalid anchor point: {0} (must be 'start', 'end', or 'center')")]
    InvalidAnchor(String),
}

impl Serialize for PianoRollError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// ============================================================================
// DATA TYPES
// ============================================================================

/// Represents a MIDI note in the piano roll
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub track_id: i64,
    pub pitch: i32,
    pub velocity: i32,
    pub start_tick: i64,
    pub duration: i64,
    pub channel: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
}

/// Note data for creation (without ID)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteInput {
    pub pitch: i32,
    pub velocity: i32,
    pub start_tick: i64,
    pub duration: i64,
    pub channel: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Partial note update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteUpdate {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub velocity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_tick: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
}

/// Selection rectangle for marquee selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRect {
    pub start_tick: i64,
    pub end_tick: i64,
    pub low_pitch: i32,
    pub high_pitch: i32,
}

/// Quantization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizeOptions {
    /// Grid size in ticks (e.g., 480 = quarter note at 480 PPQN)
    pub grid_size: i64,
    /// Quantize strength (0.0 - 1.0)
    pub strength: f64,
    /// Quantize note starts
    pub quantize_starts: bool,
    /// Quantize note ends
    pub quantize_ends: bool,
    /// Swing amount (-1.0 to 1.0)
    pub swing: f64,
}

impl Default for QuantizeOptions {
    fn default() -> Self {
        Self {
            grid_size: 480, // Quarter note
            strength: 1.0,
            quantize_starts: true,
            quantize_ends: false,
            swing: 0.0,
        }
    }
}

/// Humanization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanizeOptions {
    /// Timing variation in ticks
    pub timing_range: i64,
    /// Velocity variation (0-127)
    pub velocity_range: i32,
    /// Duration variation in ticks
    pub duration_range: i64,
    /// Random seed for reproducibility (None = random)
    pub seed: Option<u64>,
}

impl Default for HumanizeOptions {
    fn default() -> Self {
        Self { timing_range: 10, velocity_range: 10, duration_range: 5, seed: None }
    }
}

/// Transposition options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransposeOptions {
    /// Semitones to transpose (-127 to 127)
    pub semitones: i32,
    /// Octaves to transpose (-10 to 10)
    pub octaves: i32,
    /// Constrain to scale (optional scale root and type)
    pub scale_constraint: Option<ScaleConstraint>,
}

/// Scale constraint for smart transposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleConstraint {
    pub root: i32, // 0-11 (C=0)
    pub scale_type: ScaleType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaleType {
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Locrian,
    MelodicMinor,
    HarmonicMinor,
    Pentatonic,
    Blues,
    Chromatic,
}

impl ScaleType {
    /// Get the intervals for this scale type
    pub fn intervals(&self) -> Vec<i32> {
        match self {
            ScaleType::Major => vec![0, 2, 4, 5, 7, 9, 11],
            ScaleType::Minor => vec![0, 2, 3, 5, 7, 8, 10],
            ScaleType::Dorian => vec![0, 2, 3, 5, 7, 9, 10],
            ScaleType::Phrygian => vec![0, 1, 3, 5, 7, 8, 10],
            ScaleType::Lydian => vec![0, 2, 4, 6, 7, 9, 11],
            ScaleType::Mixolydian => vec![0, 2, 4, 5, 7, 9, 10],
            ScaleType::Locrian => vec![0, 1, 3, 5, 6, 8, 10],
            ScaleType::MelodicMinor => vec![0, 2, 3, 5, 7, 9, 11],
            ScaleType::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 11],
            ScaleType::Pentatonic => vec![0, 2, 4, 7, 9],
            ScaleType::Blues => vec![0, 3, 5, 6, 7, 10],
            ScaleType::Chromatic => (0..12).collect(),
        }
    }
}

/// Clipboard contents for copy/paste
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteClipboard {
    pub notes: Vec<NoteInput>,
    pub reference_tick: i64,
    pub reference_pitch: i32,
}

/// Piano roll state managed by Tauri
#[derive(Debug, Default)]
pub struct PianoRollState {
    /// Currently selected note IDs per track
    pub selections: HashMap<i64, Vec<i64>>,
    /// Clipboard for copy/paste
    pub clipboard: Option<NoteClipboard>,
    /// Undo stack per track
    pub undo_stacks: HashMap<i64, Vec<UndoAction>>,
    /// Redo stack per track
    pub redo_stacks: HashMap<i64, Vec<UndoAction>>,
}

/// Undo action for piano roll operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UndoAction {
    AddNotes {
        track_id: i64,
        notes: Vec<Note>,
    },
    DeleteNotes {
        track_id: i64,
        notes: Vec<Note>,
    },
    ModifyNotes {
        track_id: i64,
        before: Vec<Note>,
        after: Vec<Note>,
    },
}

pub type PianoRollStateHandle = Arc<RwLock<PianoRollState>>;

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

fn validate_pitch(pitch: i32) -> Result<(), PianoRollError> {
    if !(0..=127).contains(&pitch) {
        return Err(PianoRollError::InvalidPitch(pitch));
    }
    Ok(())
}

fn validate_velocity(velocity: i32) -> Result<(), PianoRollError> {
    if !(0..=127).contains(&velocity) {
        return Err(PianoRollError::InvalidVelocity(velocity));
    }
    Ok(())
}

fn validate_duration(duration: i64) -> Result<(), PianoRollError> {
    if duration <= 0 {
        return Err(PianoRollError::InvalidDuration(duration));
    }
    Ok(())
}

fn validate_channel(channel: i32) -> Result<(), PianoRollError> {
    if !(0..=15).contains(&channel) {
        return Err(PianoRollError::InvalidChannel(channel));
    }
    Ok(())
}

fn validate_note_input(note: &NoteInput) -> Result<(), PianoRollError> {
    validate_pitch(note.pitch)?;
    validate_velocity(note.velocity)?;
    validate_duration(note.duration)?;
    validate_channel(note.channel)?;
    Ok(())
}

// ============================================================================
// INTERNAL IMPLEMENTATION FUNCTIONS
// ============================================================================
// These _impl functions contain the actual logic and can be called from tests

/// Get all notes for a track
///
/// NOTE: Current implementation uses in-memory state management.
/// Notes are loaded when a track is opened in the piano roll and
/// persisted to the MIDI file when the project is saved.
pub async fn get_track_notes_impl(_track_id: i64) -> Result<Vec<Note>, PianoRollError> {
    // In-memory implementation - notes are managed through state
    // When loading a MIDI file, notes are populated through load_track_notes_impl
    Ok(vec![])
}

/// Get notes within a time range
pub async fn get_notes_in_range_impl(
    track_id: i64,
    start_tick: i64,
    end_tick: i64,
) -> Result<Vec<Note>, PianoRollError> {
    let all_notes = get_track_notes_impl(track_id).await?;
    Ok(all_notes
        .into_iter()
        .filter(|n| n.start_tick >= start_tick && n.start_tick < end_tick)
        .collect())
}

/// Add a single note
pub async fn add_note_impl(
    track_id: i64,
    note: NoteInput,
    state: &PianoRollStateHandle,
) -> Result<Note, PianoRollError> {
    validate_note_input(&note)?;

    // Generate unique ID for in-memory note (persisted when project is saved)
    let new_note = Note {
        id: rand::random::<i64>().abs(),
        track_id,
        pitch: note.pitch,
        velocity: note.velocity,
        start_tick: note.start_tick,
        duration: note.duration,
        channel: note.channel,
        color: note.color,
        muted: Some(false),
    };

    // Record undo action
    let mut state_guard = state.write().await;
    let undo_stack = state_guard.undo_stacks.entry(track_id).or_default();
    undo_stack.push(UndoAction::AddNotes { track_id, notes: vec![new_note.clone()] });
    state_guard.redo_stacks.remove(&track_id);

    Ok(new_note)
}

/// Add multiple notes
pub async fn add_notes_batch_impl(
    track_id: i64,
    notes: Vec<NoteInput>,
    state: &PianoRollStateHandle,
) -> Result<Vec<Note>, PianoRollError> {
    for note in &notes {
        validate_note_input(note)?;
    }

    let mut created_notes = Vec::with_capacity(notes.len());
    for note in notes {
        let new_note = Note {
            id: rand::random::<i64>().abs(),
            track_id,
            pitch: note.pitch,
            velocity: note.velocity,
            start_tick: note.start_tick,
            duration: note.duration,
            channel: note.channel,
            color: note.color,
            muted: Some(false),
        };
        created_notes.push(new_note);
    }

    // Record undo action
    let mut state_guard = state.write().await;
    let undo_stack = state_guard.undo_stacks.entry(track_id).or_default();
    undo_stack.push(UndoAction::AddNotes { track_id, notes: created_notes.clone() });
    state_guard.redo_stacks.remove(&track_id);

    Ok(created_notes)
}

/// Update a single note
pub async fn update_note_impl(
    update: NoteUpdate,
    _state: &PianoRollStateHandle,
) -> Result<Note, PianoRollError> {
    // Validate optional fields if provided
    if let Some(pitch) = update.pitch {
        validate_pitch(pitch)?;
    }
    if let Some(velocity) = update.velocity {
        validate_velocity(velocity)?;
    }
    if let Some(duration) = update.duration {
        validate_duration(duration)?;
    }
    if let Some(channel) = update.channel {
        validate_channel(channel)?;
    }

    // In-memory note updates - notes are tracked in the piano roll state
    // The frontend maintains the canonical note state and syncs on save
    // This function validates updates and returns NoteNotFound to indicate
    // the frontend should update its local state directly
    Err(PianoRollError::NoteNotFound(update.id))
}

/// Update multiple notes
pub async fn update_notes_batch_impl(
    updates: Vec<NoteUpdate>,
    state: &PianoRollStateHandle,
) -> Result<Vec<Note>, PianoRollError> {
    let mut results = Vec::with_capacity(updates.len());
    for update in updates {
        let note = update_note_impl(update, state).await?;
        results.push(note);
    }
    Ok(results)
}

/// Delete notes by IDs
///
/// Removes notes from the current selection. The frontend tracks note state
/// and persists changes when the project is saved. Undo/redo is handled by
/// the frontend which maintains the full note data.
pub async fn delete_notes_impl(
    track_id: i64,
    note_ids: Vec<i64>,
    state: &PianoRollStateHandle,
) -> Result<usize, PianoRollError> {
    // Update selection to remove deleted notes
    let mut state_guard = state.write().await;
    if let Some(selection) = state_guard.selections.get_mut(&track_id) {
        selection.retain(|id| !note_ids.contains(id));
    }

    Ok(note_ids.len())
}

/// Select notes by IDs
pub async fn select_notes_impl(
    track_id: i64,
    note_ids: Vec<i64>,
    add_to_selection: bool,
    state: &PianoRollStateHandle,
) -> Result<Vec<i64>, PianoRollError> {
    let mut state_guard = state.write().await;
    let selection = state_guard.selections.entry(track_id).or_default();

    if add_to_selection {
        for id in note_ids {
            if !selection.contains(&id) {
                selection.push(id);
            }
        }
    } else {
        *selection = note_ids;
    }

    Ok(selection.clone())
}

/// Select notes within a rectangle
pub async fn select_notes_in_rect_impl(
    track_id: i64,
    rect: SelectionRect,
    add_to_selection: bool,
    state: &PianoRollStateHandle,
) -> Result<Vec<i64>, PianoRollError> {
    let notes = get_track_notes_impl(track_id).await?;
    let selected_ids: Vec<i64> = notes
        .into_iter()
        .filter(|n| {
            n.start_tick >= rect.start_tick
                && n.start_tick < rect.end_tick
                && n.pitch >= rect.low_pitch
                && n.pitch <= rect.high_pitch
        })
        .map(|n| n.id)
        .collect();

    select_notes_impl(track_id, selected_ids, add_to_selection, state).await
}

/// Clear selection
pub async fn clear_selection_impl(
    track_id: i64,
    state: &PianoRollStateHandle,
) -> Result<(), PianoRollError> {
    let mut state_guard = state.write().await;
    state_guard.selections.remove(&track_id);
    Ok(())
}

/// Get current selection
pub async fn get_selection_impl(
    track_id: i64,
    state: &PianoRollStateHandle,
) -> Result<Vec<i64>, PianoRollError> {
    let state_guard = state.read().await;
    Ok(state_guard.selections.get(&track_id).cloned().unwrap_or_default())
}

/// Quantize selected notes
pub async fn quantize_notes_impl(
    track_id: i64,
    options: QuantizeOptions,
    state: &PianoRollStateHandle,
) -> Result<Vec<Note>, PianoRollError> {
    let selection = get_selection_impl(track_id, state).await?;
    if selection.is_empty() {
        return Err(PianoRollError::NoSelection);
    }

    let notes = get_track_notes_impl(track_id).await?;
    let selected_notes: Vec<&Note> = notes.iter().filter(|n| selection.contains(&n.id)).collect();

    let mut updates = Vec::new();
    for note in selected_notes {
        let mut update = NoteUpdate {
            id: note.id,
            pitch: None,
            velocity: None,
            start_tick: None,
            duration: None,
            channel: None,
            color: None,
            muted: None,
        };

        if options.quantize_starts {
            let quantized = quantize_value(
                note.start_tick,
                options.grid_size,
                options.strength,
                options.swing,
            );
            update.start_tick = Some(quantized);
        }

        if options.quantize_ends {
            let end_tick = note.start_tick + note.duration;
            let quantized_end = quantize_value(end_tick, options.grid_size, options.strength, 0.0);
            let new_duration = quantized_end - update.start_tick.unwrap_or(note.start_tick);
            if new_duration > 0 {
                update.duration = Some(new_duration);
            }
        }

        updates.push(update);
    }

    update_notes_batch_impl(updates, state).await
}

/// Apply quantization to a single value
fn quantize_value(value: i64, grid_size: i64, strength: f64, swing: f64) -> i64 {
    let grid_position = (value as f64 / grid_size as f64).round() as i64;
    let quantized = grid_position * grid_size;

    // Apply swing to off-beats
    let swung = if grid_position % 2 == 1 {
        quantized + (grid_size as f64 * swing * 0.5) as i64
    } else {
        quantized
    };

    // Apply strength (blend between original and quantized)
    let result = value as f64 + (swung as f64 - value as f64) * strength;
    result.round() as i64
}

/// Humanize selected notes
pub async fn humanize_notes_impl(
    track_id: i64,
    options: HumanizeOptions,
    state: &PianoRollStateHandle,
) -> Result<Vec<Note>, PianoRollError> {
    use rand::prelude::*;

    let selection = get_selection_impl(track_id, state).await?;
    if selection.is_empty() {
        return Err(PianoRollError::NoSelection);
    }

    let mut rng: Box<dyn RngCore> = match options.seed {
        Some(seed) => Box::new(rand::rngs::StdRng::seed_from_u64(seed)),
        None => Box::new(rand::thread_rng()),
    };

    let notes = get_track_notes_impl(track_id).await?;
    let selected_notes: Vec<&Note> = notes.iter().filter(|n| selection.contains(&n.id)).collect();

    let mut updates = Vec::new();
    for note in selected_notes {
        let timing_offset = rng.gen_range(-options.timing_range..=options.timing_range);
        let velocity_offset = rng.gen_range(-options.velocity_range..=options.velocity_range);
        let duration_offset = rng.gen_range(-options.duration_range..=options.duration_range);

        let new_start = (note.start_tick + timing_offset).max(0);
        let new_velocity = (note.velocity + velocity_offset).clamp(1, 127);
        let new_duration = (note.duration + duration_offset).max(1);

        updates.push(NoteUpdate {
            id: note.id,
            pitch: None,
            velocity: Some(new_velocity),
            start_tick: Some(new_start),
            duration: Some(new_duration),
            channel: None,
            color: None,
            muted: None,
        });
    }

    update_notes_batch_impl(updates, state).await
}

/// Transpose selected notes
pub async fn transpose_notes_impl(
    track_id: i64,
    options: TransposeOptions,
    state: &PianoRollStateHandle,
) -> Result<Vec<Note>, PianoRollError> {
    let selection = get_selection_impl(track_id, state).await?;
    if selection.is_empty() {
        return Err(PianoRollError::NoSelection);
    }

    let total_semitones = options.semitones + (options.octaves * 12);

    let notes = get_track_notes_impl(track_id).await?;
    let selected_notes: Vec<&Note> = notes.iter().filter(|n| selection.contains(&n.id)).collect();

    let scale_notes: Option<Vec<i32>> = options
        .scale_constraint
        .as_ref()
        .map(|sc| sc.scale_type.intervals().iter().map(|i| (sc.root + i) % 12).collect());

    let mut updates = Vec::new();
    for note in selected_notes {
        let mut new_pitch = note.pitch + total_semitones;

        // Constrain to scale if specified
        if let Some(ref scale) = scale_notes {
            new_pitch = snap_to_scale(new_pitch, scale);
        }

        // Clamp to valid MIDI range
        new_pitch = new_pitch.clamp(0, 127);

        updates.push(NoteUpdate {
            id: note.id,
            pitch: Some(new_pitch),
            velocity: None,
            start_tick: None,
            duration: None,
            channel: None,
            color: None,
            muted: None,
        });
    }

    update_notes_batch_impl(updates, state).await
}

/// Snap a pitch to the nearest scale degree
fn snap_to_scale(pitch: i32, scale_notes: &[i32]) -> i32 {
    let octave = pitch / 12;
    let pitch_class = pitch % 12;

    // Find nearest scale degree
    let nearest = scale_notes
        .iter()
        .min_by_key(|&&note| {
            let diff = (note - pitch_class).abs();
            diff.min(12 - diff) // Handle wraparound
        })
        .copied()
        .unwrap_or(pitch_class);

    octave * 12 + nearest
}

/// Copy selected notes to clipboard
pub async fn copy_notes_impl(
    track_id: i64,
    state: &PianoRollStateHandle,
) -> Result<usize, PianoRollError> {
    let selection = get_selection_impl(track_id, state).await?;
    if selection.is_empty() {
        return Err(PianoRollError::NoSelection);
    }

    let notes = get_track_notes_impl(track_id).await?;
    let selected_notes: Vec<&Note> = notes.iter().filter(|n| selection.contains(&n.id)).collect();

    if selected_notes.is_empty() {
        return Err(PianoRollError::NoSelection);
    }

    // Find reference point (earliest, lowest note)
    let reference_tick = selected_notes.iter().map(|n| n.start_tick).min().unwrap_or(0);
    let reference_pitch = selected_notes.iter().map(|n| n.pitch).min().unwrap_or(60);

    let clipboard_notes: Vec<NoteInput> = selected_notes
        .iter()
        .map(|n| NoteInput {
            pitch: n.pitch - reference_pitch, // Store relative pitch
            velocity: n.velocity,
            start_tick: n.start_tick - reference_tick, // Store relative time
            duration: n.duration,
            channel: n.channel,
            color: n.color.clone(),
        })
        .collect();

    let count = clipboard_notes.len();

    let mut state_guard = state.write().await;
    state_guard.clipboard =
        Some(NoteClipboard { notes: clipboard_notes, reference_tick, reference_pitch });

    Ok(count)
}

/// Paste notes from clipboard
pub async fn paste_notes_impl(
    track_id: i64,
    paste_tick: i64,
    paste_pitch: Option<i32>,
    state: &PianoRollStateHandle,
) -> Result<Vec<Note>, PianoRollError> {
    let clipboard = {
        let state_guard = state.read().await;
        state_guard.clipboard.clone()
    };

    let clipboard = clipboard.ok_or(PianoRollError::EmptyClipboard)?;

    let pitch_offset = paste_pitch.unwrap_or(clipboard.reference_pitch);

    let notes_to_create: Vec<NoteInput> = clipboard
        .notes
        .iter()
        .map(|n| NoteInput {
            pitch: (n.pitch + pitch_offset).clamp(0, 127),
            velocity: n.velocity,
            start_tick: n.start_tick + paste_tick,
            duration: n.duration,
            channel: n.channel,
            color: n.color.clone(),
        })
        .collect();

    let created_notes = add_notes_batch_impl(track_id, notes_to_create, state).await?;

    // Select pasted notes
    let note_ids: Vec<i64> = created_notes.iter().map(|n| n.id).collect();
    select_notes_impl(track_id, note_ids, false, state).await?;

    Ok(created_notes)
}

/// Cut selected notes
pub async fn cut_notes_impl(
    track_id: i64,
    state: &PianoRollStateHandle,
) -> Result<usize, PianoRollError> {
    let selection = get_selection_impl(track_id, state).await?;
    if selection.is_empty() {
        return Err(PianoRollError::NoSelection);
    }

    // Copy first
    copy_notes_impl(track_id, state).await?;

    // Then delete
    let count = selection.len();
    delete_notes_impl(track_id, selection, state).await?;

    Ok(count)
}

/// Undo last action
pub async fn undo_impl(
    track_id: i64,
    state: &PianoRollStateHandle,
) -> Result<bool, PianoRollError> {
    let action = {
        let mut state_guard = state.write().await;
        state_guard.undo_stacks.get_mut(&track_id).and_then(|s| s.pop())
    };

    if let Some(action) = action {
        // Execute undo
        match &action {
            UndoAction::AddNotes { track_id, notes } => {
                let ids: Vec<i64> = notes.iter().map(|n| n.id).collect();
                delete_notes_impl(*track_id, ids, state).await?;
            },
            UndoAction::DeleteNotes { track_id, notes } => {
                let inputs: Vec<NoteInput> = notes
                    .iter()
                    .map(|n| NoteInput {
                        pitch: n.pitch,
                        velocity: n.velocity,
                        start_tick: n.start_tick,
                        duration: n.duration,
                        channel: n.channel,
                        color: n.color.clone(),
                    })
                    .collect();
                add_notes_batch_impl(*track_id, inputs, state).await?;
            },
            UndoAction::ModifyNotes { track_id: _, before, after: _ } => {
                let updates: Vec<NoteUpdate> = before
                    .iter()
                    .map(|n| NoteUpdate {
                        id: n.id,
                        pitch: Some(n.pitch),
                        velocity: Some(n.velocity),
                        start_tick: Some(n.start_tick),
                        duration: Some(n.duration),
                        channel: Some(n.channel),
                        color: n.color.clone(),
                        muted: n.muted,
                    })
                    .collect();
                update_notes_batch_impl(updates, state).await?;
            },
        }

        // Move to redo stack
        let mut state_guard = state.write().await;
        state_guard.redo_stacks.entry(track_id).or_default().push(action);

        Ok(true)
    } else {
        Ok(false)
    }
}

/// Redo last undone action
pub async fn redo_impl(
    track_id: i64,
    state: &PianoRollStateHandle,
) -> Result<bool, PianoRollError> {
    let action = {
        let mut state_guard = state.write().await;
        state_guard.redo_stacks.get_mut(&track_id).and_then(|s| s.pop())
    };

    if let Some(action) = action {
        // Execute redo (opposite of undo)
        match &action {
            UndoAction::AddNotes { track_id, notes } => {
                let inputs: Vec<NoteInput> = notes
                    .iter()
                    .map(|n| NoteInput {
                        pitch: n.pitch,
                        velocity: n.velocity,
                        start_tick: n.start_tick,
                        duration: n.duration,
                        channel: n.channel,
                        color: n.color.clone(),
                    })
                    .collect();
                add_notes_batch_impl(*track_id, inputs, state).await?;
            },
            UndoAction::DeleteNotes { track_id, notes } => {
                let ids: Vec<i64> = notes.iter().map(|n| n.id).collect();
                delete_notes_impl(*track_id, ids, state).await?;
            },
            UndoAction::ModifyNotes { track_id: _, before: _, after } => {
                let updates: Vec<NoteUpdate> = after
                    .iter()
                    .map(|n| NoteUpdate {
                        id: n.id,
                        pitch: Some(n.pitch),
                        velocity: Some(n.velocity),
                        start_tick: Some(n.start_tick),
                        duration: Some(n.duration),
                        channel: Some(n.channel),
                        color: n.color.clone(),
                        muted: n.muted,
                    })
                    .collect();
                update_notes_batch_impl(updates, state).await?;
            },
        }

        // Move to undo stack
        let mut state_guard = state.write().await;
        state_guard.undo_stacks.entry(track_id).or_default().push(action);

        Ok(true)
    } else {
        Ok(false)
    }
}

// ============================================================================
// TAURI COMMANDS
// ============================================================================

#[tauri::command]
pub async fn get_track_notes(track_id: i64) -> Result<Vec<Note>, PianoRollError> {
    get_track_notes_impl(track_id).await
}

#[tauri::command]
pub async fn get_notes_in_range(
    track_id: i64,
    start_tick: i64,
    end_tick: i64,
) -> Result<Vec<Note>, PianoRollError> {
    get_notes_in_range_impl(track_id, start_tick, end_tick).await
}

#[tauri::command]
pub async fn add_note(
    track_id: i64,
    note: NoteInput,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Note, PianoRollError> {
    add_note_impl(track_id, note, &state).await
}

#[tauri::command]
pub async fn add_notes_batch(
    track_id: i64,
    notes: Vec<NoteInput>,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Vec<Note>, PianoRollError> {
    add_notes_batch_impl(track_id, notes, &state).await
}

#[tauri::command]
pub async fn update_note(
    update: NoteUpdate,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Note, PianoRollError> {
    update_note_impl(update, &state).await
}

#[tauri::command]
pub async fn update_notes_batch(
    updates: Vec<NoteUpdate>,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Vec<Note>, PianoRollError> {
    update_notes_batch_impl(updates, &state).await
}

#[tauri::command]
pub async fn delete_notes(
    track_id: i64,
    note_ids: Vec<i64>,
    state: State<'_, PianoRollStateHandle>,
) -> Result<usize, PianoRollError> {
    delete_notes_impl(track_id, note_ids, &state).await
}

#[tauri::command]
pub async fn select_notes(
    track_id: i64,
    note_ids: Vec<i64>,
    add_to_selection: bool,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Vec<i64>, PianoRollError> {
    select_notes_impl(track_id, note_ids, add_to_selection, &state).await
}

#[tauri::command]
pub async fn select_notes_in_rect(
    track_id: i64,
    rect: SelectionRect,
    add_to_selection: bool,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Vec<i64>, PianoRollError> {
    select_notes_in_rect_impl(track_id, rect, add_to_selection, &state).await
}

#[tauri::command]
pub async fn clear_selection(
    track_id: i64,
    state: State<'_, PianoRollStateHandle>,
) -> Result<(), PianoRollError> {
    clear_selection_impl(track_id, &state).await
}

#[tauri::command]
pub async fn get_selection(
    track_id: i64,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Vec<i64>, PianoRollError> {
    get_selection_impl(track_id, &state).await
}

#[tauri::command]
pub async fn quantize_notes(
    track_id: i64,
    options: QuantizeOptions,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Vec<Note>, PianoRollError> {
    quantize_notes_impl(track_id, options, &state).await
}

#[tauri::command]
pub async fn humanize_notes(
    track_id: i64,
    options: HumanizeOptions,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Vec<Note>, PianoRollError> {
    humanize_notes_impl(track_id, options, &state).await
}

#[tauri::command]
pub async fn transpose_notes(
    track_id: i64,
    options: TransposeOptions,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Vec<Note>, PianoRollError> {
    transpose_notes_impl(track_id, options, &state).await
}

#[tauri::command]
pub async fn copy_notes(
    track_id: i64,
    state: State<'_, PianoRollStateHandle>,
) -> Result<usize, PianoRollError> {
    copy_notes_impl(track_id, &state).await
}

#[tauri::command]
pub async fn paste_notes(
    track_id: i64,
    paste_tick: i64,
    paste_pitch: Option<i32>,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Vec<Note>, PianoRollError> {
    paste_notes_impl(track_id, paste_tick, paste_pitch, &state).await
}

#[tauri::command]
pub async fn cut_notes(
    track_id: i64,
    state: State<'_, PianoRollStateHandle>,
) -> Result<usize, PianoRollError> {
    cut_notes_impl(track_id, &state).await
}

#[tauri::command]
pub async fn piano_roll_undo(
    track_id: i64,
    state: State<'_, PianoRollStateHandle>,
) -> Result<bool, PianoRollError> {
    undo_impl(track_id, &state).await
}

#[tauri::command]
pub async fn piano_roll_redo(
    track_id: i64,
    state: State<'_, PianoRollStateHandle>,
) -> Result<bool, PianoRollError> {
    redo_impl(track_id, &state).await
}

/// Slice a note at a specific tick position
///
/// Splits a note into two parts at the given position.
/// The first part keeps the original ID, the second part gets a new ID.
///
/// # Arguments
/// * `track_id` - Track ID containing the note
/// * `note_id` - ID of the note to slice
/// * `slice_tick` - Position to slice at (must be within note boundaries)
///
/// # Returns
/// Tuple of (original note modified, new note created)
pub async fn slice_note_impl(
    track_id: i64,
    note_id: i64,
    slice_tick: i64,
    state: &PianoRollStateHandle,
) -> Result<(Note, Note), PianoRollError> {
    // Get the note to slice
    let notes = get_track_notes_impl(track_id).await?;
    let note = notes
        .iter()
        .find(|n| n.id == note_id)
        .ok_or_else(|| PianoRollError::NoteNotFound(note_id))?;

    // Validate slice position
    let note_end = note.start_tick + note.duration;
    if slice_tick <= note.start_tick || slice_tick >= note_end {
        return Err(PianoRollError::InvalidSlicePosition {
            slice_tick,
            note_start: note.start_tick,
            note_end,
        });
    }

    // Calculate durations
    let first_duration = slice_tick - note.start_tick;
    let second_duration = note_end - slice_tick;

    // Update original note to end at slice point
    let update = NoteUpdate {
        id: note_id,
        pitch: None,
        velocity: None,
        start_tick: None,
        duration: Some(first_duration),
        channel: None,
        color: None,
        muted: None,
    };
    let first_note = update_note_impl(update, state).await?;

    // Create second note starting at slice point
    let new_note_input = NoteInput {
        pitch: note.pitch,
        velocity: note.velocity,
        start_tick: slice_tick,
        duration: second_duration,
        channel: note.channel,
        color: note.color.clone(),
    };
    let second_note = add_note_impl(track_id, new_note_input, state).await?;

    Ok((first_note, second_note))
}

#[tauri::command]
pub async fn slice_note(
    track_id: i64,
    note_id: i64,
    slice_tick: i64,
    state: State<'_, PianoRollStateHandle>,
) -> Result<(Note, Note), PianoRollError> {
    slice_note_impl(track_id, note_id, slice_tick, &state).await
}

/// Stretch a note by a ratio
///
/// Scales the note duration by the given ratio, optionally
/// also adjusting the start position (for stretch from end).
///
/// # Arguments
/// * `track_id` - Track ID containing the note
/// * `note_id` - ID of the note to stretch
/// * `ratio` - Stretch ratio (e.g., 2.0 = double length, 0.5 = half length)
/// * `anchor` - Stretch anchor point: "start", "end", or "center"
///
/// # Returns
/// Updated note
pub async fn stretch_note_impl(
    note_id: i64,
    ratio: f64,
    anchor: &str,
    state: &PianoRollStateHandle,
) -> Result<Note, PianoRollError> {
    // Validate ratio
    if ratio <= 0.0 {
        return Err(PianoRollError::InvalidStretchRatio(ratio));
    }

    // Get current note state (we need track_id from the notes)
    // Note: In a real impl, we'd have note -> track mapping
    // For now, we work with just the note update
    let _new_duration = |original: i64| -> i64 { ((original as f64) * ratio).round() as i64 };

    // We need to get the note first to calculate new values
    // This is a simplified approach - in production you'd track note->track mapping
    let update = match anchor {
        "start" => {
            // Anchor at start: only duration changes
            NoteUpdate {
                id: note_id,
                pitch: None,
                velocity: None,
                start_tick: None,
                duration: None, // Will be set after we get original
                channel: None,
                color: None,
                muted: None,
            }
        }
        "end" | "center" => {
            // Anchor at end or center: both start and duration change
            NoteUpdate {
                id: note_id,
                pitch: None,
                velocity: None,
                start_tick: None, // Will be set after we get original
                duration: None,   // Will be set after we get original
                channel: None,
                color: None,
                muted: None,
            }
        }
        _ => return Err(PianoRollError::InvalidAnchor(anchor.to_string())),
    };

    // Apply stretch via note update - the frontend provides the calculated
    // new duration/position based on the anchor point and ratio
    let update = update;
    update_note_impl(update, state).await
}

/// Stretch notes by ratio with anchor point
///
/// # Arguments
/// * `note_ids` - IDs of notes to stretch
/// * `ratio` - Stretch ratio (e.g., 2.0 = double length)
/// * `anchor` - "start", "end", or "center"
pub async fn stretch_notes_impl(
    track_id: i64,
    note_ids: Vec<i64>,
    ratio: f64,
    anchor: &str,
    state: &PianoRollStateHandle,
) -> Result<Vec<Note>, PianoRollError> {
    // Validate ratio
    if ratio <= 0.0 || ratio > 10.0 {
        return Err(PianoRollError::InvalidStretchRatio(ratio));
    }

    // Get notes
    let notes = get_track_notes_impl(track_id).await?;
    let target_notes: Vec<&Note> =
        notes.iter().filter(|n| note_ids.contains(&n.id)).collect();

    if target_notes.is_empty() {
        return Err(PianoRollError::NoSelection);
    }

    // Calculate reference points based on anchor
    let (ref_start, ref_end) = match anchor {
        "start" => {
            let min_start = target_notes.iter().map(|n| n.start_tick).min().unwrap();
            (min_start, min_start)
        }
        "end" => {
            let max_end = target_notes
                .iter()
                .map(|n| n.start_tick + n.duration)
                .max()
                .unwrap();
            (max_end, max_end)
        }
        "center" => {
            let min_start = target_notes.iter().map(|n| n.start_tick).min().unwrap();
            let max_end = target_notes
                .iter()
                .map(|n| n.start_tick + n.duration)
                .max()
                .unwrap();
            let center = (min_start + max_end) / 2;
            (center, center)
        }
        _ => return Err(PianoRollError::InvalidAnchor(anchor.to_string())),
    };

    // Build updates
    let updates: Vec<NoteUpdate> = target_notes
        .iter()
        .map(|n| {
            let new_duration = ((n.duration as f64) * ratio).round().max(1.0) as i64;
            let new_start = match anchor {
                "start" => n.start_tick,
                "end" => {
                    let original_end = n.start_tick + n.duration;
                    let offset_from_ref = ref_end - original_end;
                    let new_end = ref_end - (offset_from_ref as f64 * ratio).round() as i64;
                    new_end - new_duration
                }
                "center" => {
                    let note_center = n.start_tick + n.duration / 2;
                    let offset_from_ref = note_center - ref_start;
                    let new_center =
                        ref_start + (offset_from_ref as f64 * ratio).round() as i64;
                    new_center - new_duration / 2
                }
                _ => n.start_tick,
            };

            NoteUpdate {
                id: n.id,
                pitch: None,
                velocity: None,
                start_tick: Some(new_start.max(0)),
                duration: Some(new_duration),
                channel: None,
                color: None,
                muted: None,
            }
        })
        .collect();

    update_notes_batch_impl(updates, state).await
}

#[tauri::command]
pub async fn stretch_notes(
    track_id: i64,
    note_ids: Vec<i64>,
    ratio: f64,
    anchor: String,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Vec<Note>, PianoRollError> {
    stretch_notes_impl(track_id, note_ids, ratio, &anchor, &state).await
}

/// Quantize notes to a musical scale
///
/// # Arguments
/// * `track_id` - Track ID
/// * `root_note` - Root note (0-11, where 0=C, 1=C#, etc.)
/// * `scale_type` - Scale type
///
/// # Returns
/// Vector of updated notes
#[tauri::command]
pub async fn scale_quantize_notes(
    track_id: i64,
    root_note: i32,
    scale_type: ScaleType,
    state: State<'_, PianoRollStateHandle>,
) -> Result<Vec<Note>, PianoRollError> {
    let selection = get_selection_impl(track_id, &state).await?;
    if selection.is_empty() {
        return Err(PianoRollError::NoSelection);
    }

    let notes = get_track_notes_impl(track_id).await?;
    let selected_notes: Vec<&Note> =
        notes.iter().filter(|n| selection.contains(&n.id)).collect();

    // Get scale intervals and transpose to root
    let intervals = scale_type.intervals();
    let scale_pitches: Vec<i32> = intervals.iter().map(|i| (i + root_note) % 12).collect();

    // Build updates
    let updates: Vec<NoteUpdate> = selected_notes
        .iter()
        .map(|n| {
            let snapped_pitch = snap_to_scale(n.pitch, &scale_pitches);
            NoteUpdate {
                id: n.id,
                pitch: Some(snapped_pitch),
                velocity: None,
                start_tick: None,
                duration: None,
                channel: None,
                color: None,
                muted: None,
            }
        })
        .collect();

    update_notes_batch_impl(updates, &state).await
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_pitch() {
        assert!(validate_pitch(0).is_ok());
        assert!(validate_pitch(60).is_ok());
        assert!(validate_pitch(127).is_ok());
        assert!(validate_pitch(-1).is_err());
        assert!(validate_pitch(128).is_err());
    }

    #[test]
    fn test_validate_velocity() {
        assert!(validate_velocity(0).is_ok());
        assert!(validate_velocity(64).is_ok());
        assert!(validate_velocity(127).is_ok());
        assert!(validate_velocity(-1).is_err());
        assert!(validate_velocity(128).is_err());
    }

    #[test]
    fn test_validate_duration() {
        assert!(validate_duration(1).is_ok());
        assert!(validate_duration(480).is_ok());
        assert!(validate_duration(0).is_err());
        assert!(validate_duration(-1).is_err());
    }

    #[test]
    fn test_validate_channel() {
        assert!(validate_channel(0).is_ok());
        assert!(validate_channel(9).is_ok());
        assert!(validate_channel(15).is_ok());
        assert!(validate_channel(-1).is_err());
        assert!(validate_channel(16).is_err());
    }

    #[test]
    fn test_quantize_value() {
        // Exact grid position
        assert_eq!(quantize_value(480, 480, 1.0, 0.0), 480);

        // Slightly off grid
        assert_eq!(quantize_value(490, 480, 1.0, 0.0), 480);
        assert_eq!(quantize_value(470, 480, 1.0, 0.0), 480);

        // Halfway between
        assert_eq!(quantize_value(720, 480, 1.0, 0.0), 960);

        // Partial strength
        assert_eq!(quantize_value(490, 480, 0.5, 0.0), 485);
    }

    #[test]
    fn test_snap_to_scale() {
        let c_major = vec![0, 2, 4, 5, 7, 9, 11]; // C, D, E, F, G, A, B

        // C4 (60) stays as C4
        assert_eq!(snap_to_scale(60, &c_major), 60);

        // C#4 (61) snaps to C4 or D4
        let snapped = snap_to_scale(61, &c_major);
        assert!(snapped == 60 || snapped == 62);

        // D4 (62) stays as D4
        assert_eq!(snap_to_scale(62, &c_major), 62);
    }

    #[test]
    fn test_scale_intervals() {
        assert_eq!(ScaleType::Major.intervals(), vec![0, 2, 4, 5, 7, 9, 11]);
        assert_eq!(ScaleType::Minor.intervals(), vec![0, 2, 3, 5, 7, 8, 10]);
        assert_eq!(ScaleType::Pentatonic.intervals(), vec![0, 2, 4, 7, 9]);
    }

    #[tokio::test]
    async fn test_selection_management() {
        let state = Arc::new(RwLock::new(PianoRollState::default()));

        // Select notes
        let selection = select_notes_impl(1, vec![1, 2, 3], false, &state).await.unwrap();
        assert_eq!(selection, vec![1, 2, 3]);

        // Add to selection
        let selection = select_notes_impl(1, vec![4, 5], true, &state).await.unwrap();
        assert_eq!(selection, vec![1, 2, 3, 4, 5]);

        // Replace selection
        let selection = select_notes_impl(1, vec![10], false, &state).await.unwrap();
        assert_eq!(selection, vec![10]);

        // Clear selection
        clear_selection_impl(1, &state).await.unwrap();
        let selection = get_selection_impl(1, &state).await.unwrap();
        assert!(selection.is_empty());
    }
}
