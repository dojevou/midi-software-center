// Tempo Editor State and Commands
//
// Handles tempo changes and tempo automation for the DAW.
// Architecture:
// - TempoEditorState: Trusty Module (pure state)
// - Implementation functions: Grown-up Script
// - Tauri commands: Task-O-Matic

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TempoError {
    #[error("Invalid tempo: {0} (must be 20-300 BPM)")]
    InvalidTempo(f32),
    #[error("Invalid bar position: {0} (must be non-negative)")]
    InvalidBar(i32),
    #[error("Tempo point not found at bar: {0}")]
    TempoPointNotFound(i32),
    #[error("Invalid duration: {0} (must be positive)")]
    InvalidDuration(i32),
}

const MIN_TEMPO: f32 = 20.0;
const MAX_TEMPO: f32 = 300.0;
const DEFAULT_TEMPO: f32 = 120.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TempoPoint {
    pub bar: i32,
    pub bpm: f32,
}

impl TempoPoint {
    pub fn new(bar: i32, bpm: f32) -> Result<Self, TempoError> {
        if bar < 0 {
            return Err(TempoError::InvalidBar(bar));
        }
        if !(MIN_TEMPO..=MAX_TEMPO).contains(&bpm) {
            return Err(TempoError::InvalidTempo(bpm));
        }
        Ok(Self { bar, bpm })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempoEditorState {
    pub tempo_points: HashMap<i32, TempoPoint>, // Key is bar number
}

impl TempoEditorState {
    pub fn new() -> Self {
        let mut tempo_points = HashMap::new();
        tempo_points.insert(0, TempoPoint { bar: 0, bpm: DEFAULT_TEMPO });
        Self { tempo_points }
    }

    pub fn set_tempo_impl(&mut self, bar: i32, bpm: f32) -> Result<(), TempoError> {
        let point = TempoPoint::new(bar, bpm)?;
        self.tempo_points.insert(bar, point);
        Ok(())
    }

    pub fn add_tempo_ramp_impl(
        &mut self,
        start_bar: i32,
        end_bar: i32,
        start_bpm: f32,
        end_bpm: f32,
    ) -> Result<Vec<i32>, TempoError> {
        if start_bar < 0 {
            return Err(TempoError::InvalidBar(start_bar));
        }
        if end_bar < 0 {
            return Err(TempoError::InvalidBar(end_bar));
        }
        if end_bar <= start_bar {
            return Err(TempoError::InvalidDuration(end_bar - start_bar));
        }
        if !(MIN_TEMPO..=MAX_TEMPO).contains(&start_bpm) {
            return Err(TempoError::InvalidTempo(start_bpm));
        }
        if !(MIN_TEMPO..=MAX_TEMPO).contains(&end_bpm) {
            return Err(TempoError::InvalidTempo(end_bpm));
        }

        let mut added_bars = Vec::new();
        let duration = end_bar - start_bar;

        for bar in start_bar..=end_bar {
            let position = (bar - start_bar) as f32 / duration as f32;
            let bpm = start_bpm + (end_bpm - start_bpm) * position;
            self.tempo_points.insert(bar, TempoPoint { bar, bpm });
            added_bars.push(bar);
        }

        Ok(added_bars)
    }

    pub fn delete_tempo_point_impl(&mut self, bar: i32) -> Result<TempoPoint, TempoError> {
        if bar == 0 {
            return Err(TempoError::TempoPointNotFound(bar)); // Cannot delete initial tempo
        }
        self.tempo_points
            .remove(&bar)
            .ok_or(TempoError::TempoPointNotFound(bar))
    }

    pub fn get_tempo_at_bar(&self, bar: i32) -> f32 {
        // Find the tempo point at or before this bar
        let mut bars: Vec<_> = self.tempo_points.keys().copied().collect();
        bars.sort_unstable();

        for &b in bars.iter().rev() {
            if b <= bar {
                return self.tempo_points.get(&b).unwrap().bpm;
            }
        }

        DEFAULT_TEMPO
    }

    pub fn get_tempo_point(&self, bar: i32) -> Option<&TempoPoint> {
        self.tempo_points.get(&bar)
    }

    pub fn get_all_tempo_points(&self) -> Vec<&TempoPoint> {
        let mut points: Vec<_> = self.tempo_points.values().collect();
        points.sort_by_key(|p| p.bar);
        points
    }

    pub fn clear_tempo_points(&mut self) {
        self.tempo_points.clear();
        self.tempo_points.insert(0, TempoPoint { bar: 0, bpm: DEFAULT_TEMPO });
    }

    pub fn point_count(&self) -> usize {
        self.tempo_points.len()
    }

    pub fn has_tempo_changes(&self) -> bool {
        self.tempo_points.len() > 1
    }
}

impl Default for TempoEditorState {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri Command Handlers (Task-O-Matic)
use tauri::State;
use std::sync::Mutex;

#[tauri::command]
pub async fn tempo_editor_set_tempo(
    state: State<'_, Mutex<TempoEditorState>>,
    bar: i32,
    bpm: f32,
) -> Result<(), String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .set_tempo_impl(bar, bpm)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tempo_editor_add_tempo_ramp(
    state: State<'_, Mutex<TempoEditorState>>,
    start_bar: i32,
    end_bar: i32,
    start_bpm: f32,
    end_bpm: f32,
) -> Result<Vec<i32>, String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .add_tempo_ramp_impl(start_bar, end_bar, start_bpm, end_bpm)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempo_point_new() {
        let point = TempoPoint::new(0, 120.0).unwrap();
        assert_eq!(point.bar, 0);
        assert_eq!(point.bpm, 120.0);
    }

    #[test]
    fn test_tempo_point_invalid_bar() {
        let result = TempoPoint::new(-1, 120.0);
        assert!(matches!(result, Err(TempoError::InvalidBar(-1))));
    }

    #[test]
    fn test_tempo_point_invalid_tempo_low() {
        let result = TempoPoint::new(0, 10.0);
        assert!(matches!(result, Err(TempoError::InvalidTempo(_))));
    }

    #[test]
    fn test_tempo_point_invalid_tempo_high() {
        let result = TempoPoint::new(0, 400.0);
        assert!(matches!(result, Err(TempoError::InvalidTempo(_))));
    }

    #[test]
    fn test_tempo_point_boundary_valid() {
        let min = TempoPoint::new(0, MIN_TEMPO).unwrap();
        assert_eq!(min.bpm, MIN_TEMPO);

        let max = TempoPoint::new(0, MAX_TEMPO).unwrap();
        assert_eq!(max.bpm, MAX_TEMPO);
    }

    #[test]
    fn test_tempo_editor_state_new() {
        let state = TempoEditorState::new();
        assert_eq!(state.tempo_points.len(), 1);
        assert_eq!(state.get_tempo_at_bar(0), DEFAULT_TEMPO);
    }

    #[test]
    fn test_set_tempo() {
        let mut state = TempoEditorState::new();
        state.set_tempo_impl(4, 140.0).unwrap();
        assert_eq!(state.get_tempo_at_bar(4), 140.0);
    }

    #[test]
    fn test_set_tempo_overwrites_existing() {
        let mut state = TempoEditorState::new();
        state.set_tempo_impl(4, 140.0).unwrap();
        state.set_tempo_impl(4, 160.0).unwrap();
        assert_eq!(state.get_tempo_at_bar(4), 160.0);
        assert_eq!(state.point_count(), 2); // Initial + bar 4
    }

    #[test]
    fn test_set_tempo_invalid() {
        let mut state = TempoEditorState::new();
        let result = state.set_tempo_impl(4, 10.0);
        assert!(matches!(result, Err(TempoError::InvalidTempo(_))));
    }

    #[test]
    fn test_add_tempo_ramp() {
        let mut state = TempoEditorState::new();
        let bars = state.add_tempo_ramp_impl(0, 4, 100.0, 140.0).unwrap();
        assert_eq!(bars.len(), 5); // 0, 1, 2, 3, 4

        assert_eq!(state.get_tempo_at_bar(0), 100.0);
        assert_eq!(state.get_tempo_at_bar(2), 120.0);
        assert_eq!(state.get_tempo_at_bar(4), 140.0);
    }

    #[test]
    fn test_add_tempo_ramp_decreasing() {
        let mut state = TempoEditorState::new();
        let bars = state.add_tempo_ramp_impl(0, 4, 140.0, 100.0).unwrap();
        assert_eq!(bars.len(), 5);

        assert_eq!(state.get_tempo_at_bar(0), 140.0);
        assert_eq!(state.get_tempo_at_bar(2), 120.0);
        assert_eq!(state.get_tempo_at_bar(4), 100.0);
    }

    #[test]
    fn test_add_tempo_ramp_invalid_start_bar() {
        let mut state = TempoEditorState::new();
        let result = state.add_tempo_ramp_impl(-1, 4, 100.0, 140.0);
        assert!(matches!(result, Err(TempoError::InvalidBar(-1))));
    }

    #[test]
    fn test_add_tempo_ramp_invalid_end_bar() {
        let mut state = TempoEditorState::new();
        let result = state.add_tempo_ramp_impl(0, -1, 100.0, 140.0);
        assert!(matches!(result, Err(TempoError::InvalidBar(-1))));
    }

    #[test]
    fn test_add_tempo_ramp_invalid_duration() {
        let mut state = TempoEditorState::new();
        let result = state.add_tempo_ramp_impl(4, 4, 100.0, 140.0);
        assert!(matches!(result, Err(TempoError::InvalidDuration(_))));

        let result = state.add_tempo_ramp_impl(4, 2, 100.0, 140.0);
        assert!(matches!(result, Err(TempoError::InvalidDuration(_))));
    }

    #[test]
    fn test_add_tempo_ramp_invalid_start_tempo() {
        let mut state = TempoEditorState::new();
        let result = state.add_tempo_ramp_impl(0, 4, 10.0, 140.0);
        assert!(matches!(result, Err(TempoError::InvalidTempo(_))));
    }

    #[test]
    fn test_add_tempo_ramp_invalid_end_tempo() {
        let mut state = TempoEditorState::new();
        let result = state.add_tempo_ramp_impl(0, 4, 100.0, 400.0);
        assert!(matches!(result, Err(TempoError::InvalidTempo(_))));
    }

    #[test]
    fn test_delete_tempo_point() {
        let mut state = TempoEditorState::new();
        state.set_tempo_impl(4, 140.0).unwrap();

        let deleted = state.delete_tempo_point_impl(4).unwrap();
        assert_eq!(deleted.bpm, 140.0);
        assert_eq!(state.point_count(), 1); // Only initial tempo remains
    }

    #[test]
    fn test_delete_tempo_point_initial() {
        let mut state = TempoEditorState::new();
        let result = state.delete_tempo_point_impl(0);
        assert!(matches!(result, Err(TempoError::TempoPointNotFound(0))));
    }

    #[test]
    fn test_delete_tempo_point_not_found() {
        let mut state = TempoEditorState::new();
        let result = state.delete_tempo_point_impl(99);
        assert!(matches!(result, Err(TempoError::TempoPointNotFound(99))));
    }

    #[test]
    fn test_get_tempo_at_bar() {
        let mut state = TempoEditorState::new();
        state.set_tempo_impl(0, 100.0).unwrap();
        state.set_tempo_impl(4, 140.0).unwrap();
        state.set_tempo_impl(8, 160.0).unwrap();

        assert_eq!(state.get_tempo_at_bar(0), 100.0);
        assert_eq!(state.get_tempo_at_bar(2), 100.0);
        assert_eq!(state.get_tempo_at_bar(4), 140.0);
        assert_eq!(state.get_tempo_at_bar(6), 140.0);
        assert_eq!(state.get_tempo_at_bar(8), 160.0);
        assert_eq!(state.get_tempo_at_bar(10), 160.0);
    }

    #[test]
    fn test_get_tempo_point() {
        let mut state = TempoEditorState::new();
        state.set_tempo_impl(4, 140.0).unwrap();

        let point = state.get_tempo_point(4).unwrap();
        assert_eq!(point.bar, 4);
        assert_eq!(point.bpm, 140.0);
    }

    #[test]
    fn test_get_tempo_point_not_found() {
        let state = TempoEditorState::new();
        assert!(state.get_tempo_point(99).is_none());
    }

    #[test]
    fn test_get_all_tempo_points() {
        let mut state = TempoEditorState::new();
        state.set_tempo_impl(4, 140.0).unwrap();
        state.set_tempo_impl(8, 160.0).unwrap();

        let points = state.get_all_tempo_points();
        assert_eq!(points.len(), 3); // Initial + 2 added
        assert_eq!(points[0].bar, 0);
        assert_eq!(points[1].bar, 4);
        assert_eq!(points[2].bar, 8);
    }

    #[test]
    fn test_clear_tempo_points() {
        let mut state = TempoEditorState::new();
        state.set_tempo_impl(4, 140.0).unwrap();
        state.set_tempo_impl(8, 160.0).unwrap();

        state.clear_tempo_points();
        assert_eq!(state.point_count(), 1); // Only initial tempo
        assert_eq!(state.get_tempo_at_bar(0), DEFAULT_TEMPO);
    }

    #[test]
    fn test_point_count() {
        let mut state = TempoEditorState::new();
        assert_eq!(state.point_count(), 1);

        state.set_tempo_impl(4, 140.0).unwrap();
        assert_eq!(state.point_count(), 2);

        state.set_tempo_impl(8, 160.0).unwrap();
        assert_eq!(state.point_count(), 3);
    }

    #[test]
    fn test_has_tempo_changes() {
        let mut state = TempoEditorState::new();
        assert!(!state.has_tempo_changes());

        state.set_tempo_impl(4, 140.0).unwrap();
        assert!(state.has_tempo_changes());
    }
}
