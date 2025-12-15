// Controller Editor State and Commands
//
// Handles MIDI CC (Continuous Controller) automation editing.
// Architecture:
// - ControllerEditorState: Trusty Module (pure state)
// - Implementation functions: Grown-up Script
// - Tauri commands: Task-O-Matic

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CCError {
    #[error("Invalid CC value: {0} (must be 0-127)")]
    InvalidCCValue(u8),
    #[error("Invalid CC number: {0} (must be 0-127)")]
    InvalidCCNumber(u8),
    #[error("Invalid tick position: {0} (must be non-negative)")]
    InvalidTick(i32),
    #[error("Automation point not found: {0}")]
    PointNotFound(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationPoint {
    pub id: i32,
    pub tick: i32,
    pub value: u8,
}

impl AutomationPoint {
    pub fn new(id: i32, tick: i32, value: u8) -> Result<Self, CCError> {
        if tick < 0 {
            return Err(CCError::InvalidTick(tick));
        }
        if value > 127 {
            return Err(CCError::InvalidCCValue(value));
        }
        Ok(Self { id, tick, value })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerEditorState {
    pub cc_number: u8,
    pub automation_points: HashMap<i32, AutomationPoint>,
    pub next_point_id: i32,
}

impl ControllerEditorState {
    pub fn new(cc_number: u8) -> Self {
        Self { cc_number, automation_points: HashMap::new(), next_point_id: 1 }
    }

    pub fn add_cc_point_impl(&mut self, tick: i32, value: u8) -> Result<i32, CCError> {
        let point = AutomationPoint::new(self.next_point_id, tick, value)?;
        let point_id = point.id;
        self.automation_points.insert(point_id, point);
        self.next_point_id += 1;
        Ok(point_id)
    }

    pub fn delete_cc_point_impl(&mut self, point_id: i32) -> Result<AutomationPoint, CCError> {
        self.automation_points.remove(&point_id).ok_or(CCError::PointNotFound(point_id))
    }

    pub fn update_cc_point_impl(
        &mut self,
        point_id: i32,
        tick: i32,
        value: u8,
    ) -> Result<(), CCError> {
        if tick < 0 {
            return Err(CCError::InvalidTick(tick));
        }
        if value > 127 {
            return Err(CCError::InvalidCCValue(value));
        }

        let point = self
            .automation_points
            .get_mut(&point_id)
            .ok_or(CCError::PointNotFound(point_id))?;

        point.tick = tick;
        point.value = value;
        Ok(())
    }

    pub fn get_point(&self, point_id: i32) -> Option<&AutomationPoint> {
        self.automation_points.get(&point_id)
    }

    pub fn get_points_in_range(&self, start_tick: i32, end_tick: i32) -> Vec<&AutomationPoint> {
        self.automation_points
            .values()
            .filter(|p| p.tick >= start_tick && p.tick < end_tick)
            .collect()
    }

    pub fn get_value_at_tick(&self, tick: i32) -> u8 {
        // Find the automation point at or before this tick
        let mut points: Vec<_> =
            self.automation_points.values().filter(|p| p.tick <= tick).collect();

        if points.is_empty() {
            return 64; // Default center value
        }

        points.sort_by_key(|p| p.tick);
        points.last().unwrap().value
    }

    pub fn interpolate_value_at_tick(&self, tick: i32) -> u8 {
        let mut points: Vec<_> = self.automation_points.values().collect();
        points.sort_by_key(|p| p.tick);

        if points.is_empty() {
            return 64; // Default center value
        }

        // Find surrounding points
        let before = points.iter().rev().find(|p| p.tick <= tick);
        let after = points.iter().find(|p| p.tick > tick);

        match (before, after) {
            (Some(b), Some(a)) => {
                // Linear interpolation
                let range = (a.tick - b.tick) as f32;
                let position = (tick - b.tick) as f32;
                let ratio = position / range;
                let value = b.value as f32 + (a.value as f32 - b.value as f32) * ratio;
                value.round() as u8
            },
            (Some(b), None) => b.value,
            (None, Some(a)) => a.value,
            (None, None) => 64,
        }
    }

    pub fn clear(&mut self) {
        self.automation_points.clear();
    }

    pub fn point_count(&self) -> usize {
        self.automation_points.len()
    }
}

// Tauri Command Handlers (Task-O-Matic)
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub async fn add_cc_point(
    state: State<'_, Mutex<ControllerEditorState>>,
    tick: i32,
    value: u8,
) -> Result<i32, String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .add_cc_point_impl(tick, value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_cc_point(
    state: State<'_, Mutex<ControllerEditorState>>,
    point_id: i32,
) -> Result<(), String> {
    state
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?
        .delete_cc_point_impl(point_id)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automation_point_new() {
        let point = AutomationPoint::new(1, 0, 64).unwrap();
        assert_eq!(point.id, 1);
        assert_eq!(point.tick, 0);
        assert_eq!(point.value, 64);
    }

    #[test]
    fn test_automation_point_invalid_tick() {
        let result = AutomationPoint::new(1, -1, 64);
        assert!(matches!(result, Err(CCError::InvalidTick(-1))));
    }

    #[test]
    fn test_automation_point_invalid_value() {
        let result = AutomationPoint::new(1, 0, 128);
        assert!(matches!(result, Err(CCError::InvalidCCValue(128))));
    }

    #[test]
    fn test_controller_editor_state_new() {
        let state = ControllerEditorState::new(1);
        assert_eq!(state.cc_number, 1);
        assert_eq!(state.automation_points.len(), 0);
        assert_eq!(state.next_point_id, 1);
    }

    #[test]
    fn test_add_cc_point() {
        let mut state = ControllerEditorState::new(1);
        let point_id = state.add_cc_point_impl(0, 64).unwrap();
        assert_eq!(point_id, 1);
        assert_eq!(state.automation_points.len(), 1);
        assert_eq!(state.next_point_id, 2);
    }

    #[test]
    fn test_add_multiple_cc_points() {
        let mut state = ControllerEditorState::new(1);
        let id1 = state.add_cc_point_impl(0, 64).unwrap();
        let id2 = state.add_cc_point_impl(480, 80).unwrap();
        let id3 = state.add_cc_point_impl(960, 100).unwrap();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        assert_eq!(state.automation_points.len(), 3);
    }

    #[test]
    fn test_add_cc_point_invalid_tick() {
        let mut state = ControllerEditorState::new(1);
        let result = state.add_cc_point_impl(-100, 64);
        assert!(matches!(result, Err(CCError::InvalidTick(-100))));
    }

    #[test]
    fn test_add_cc_point_invalid_value() {
        let mut state = ControllerEditorState::new(1);
        let result = state.add_cc_point_impl(0, 128);
        assert!(matches!(result, Err(CCError::InvalidCCValue(128))));
    }

    #[test]
    fn test_delete_cc_point() {
        let mut state = ControllerEditorState::new(1);
        let point_id = state.add_cc_point_impl(0, 64).unwrap();
        let deleted = state.delete_cc_point_impl(point_id).unwrap();
        assert_eq!(deleted.value, 64);
        assert_eq!(state.automation_points.len(), 0);
    }

    #[test]
    fn test_delete_cc_point_not_found() {
        let mut state = ControllerEditorState::new(1);
        let result = state.delete_cc_point_impl(999);
        assert!(matches!(result, Err(CCError::PointNotFound(999))));
    }

    #[test]
    fn test_update_cc_point() {
        let mut state = ControllerEditorState::new(1);
        let point_id = state.add_cc_point_impl(0, 64).unwrap();

        state.update_cc_point_impl(point_id, 480, 100).unwrap();
        let point = state.get_point(point_id).unwrap();
        assert_eq!(point.tick, 480);
        assert_eq!(point.value, 100);
    }

    #[test]
    fn test_update_cc_point_invalid_tick() {
        let mut state = ControllerEditorState::new(1);
        let point_id = state.add_cc_point_impl(0, 64).unwrap();
        let result = state.update_cc_point_impl(point_id, -100, 64);
        assert!(matches!(result, Err(CCError::InvalidTick(-100))));
    }

    #[test]
    fn test_update_cc_point_invalid_value() {
        let mut state = ControllerEditorState::new(1);
        let point_id = state.add_cc_point_impl(0, 64).unwrap();
        let result = state.update_cc_point_impl(point_id, 480, 128);
        assert!(matches!(result, Err(CCError::InvalidCCValue(128))));
    }

    #[test]
    fn test_update_cc_point_not_found() {
        let mut state = ControllerEditorState::new(1);
        let result = state.update_cc_point_impl(999, 480, 64);
        assert!(matches!(result, Err(CCError::PointNotFound(999))));
    }

    #[test]
    fn test_get_point() {
        let mut state = ControllerEditorState::new(1);
        let point_id = state.add_cc_point_impl(0, 64).unwrap();
        let point = state.get_point(point_id).unwrap();
        assert_eq!(point.tick, 0);
        assert_eq!(point.value, 64);
    }

    #[test]
    fn test_get_point_not_found() {
        let state = ControllerEditorState::new(1);
        assert!(state.get_point(999).is_none());
    }

    #[test]
    fn test_get_points_in_range() {
        let mut state = ControllerEditorState::new(1);
        state.add_cc_point_impl(0, 64).unwrap();
        state.add_cc_point_impl(480, 80).unwrap();
        state.add_cc_point_impl(960, 100).unwrap();
        state.add_cc_point_impl(1440, 120).unwrap();

        let points = state.get_points_in_range(400, 1000);
        assert_eq!(points.len(), 2); // Points at 480 and 960
    }

    #[test]
    fn test_get_points_in_range_empty() {
        let mut state = ControllerEditorState::new(1);
        state.add_cc_point_impl(0, 64).unwrap();

        let points = state.get_points_in_range(1000, 2000);
        assert_eq!(points.len(), 0);
    }

    #[test]
    fn test_get_value_at_tick() {
        let mut state = ControllerEditorState::new(1);
        state.add_cc_point_impl(0, 50).unwrap();
        state.add_cc_point_impl(480, 100).unwrap();

        assert_eq!(state.get_value_at_tick(0), 50);
        assert_eq!(state.get_value_at_tick(240), 50);
        assert_eq!(state.get_value_at_tick(480), 100);
        assert_eq!(state.get_value_at_tick(960), 100);
    }

    #[test]
    fn test_get_value_at_tick_no_points() {
        let state = ControllerEditorState::new(1);
        assert_eq!(state.get_value_at_tick(0), 64); // Default center value
    }

    #[test]
    fn test_interpolate_value_at_tick() {
        let mut state = ControllerEditorState::new(1);
        state.add_cc_point_impl(0, 0).unwrap();
        state.add_cc_point_impl(480, 127).unwrap();

        // Linear interpolation
        assert_eq!(state.interpolate_value_at_tick(0), 0);
        assert_eq!(state.interpolate_value_at_tick(240), 64); // Midpoint
        assert_eq!(state.interpolate_value_at_tick(480), 127);
    }

    #[test]
    fn test_interpolate_value_at_tick_before_first() {
        let mut state = ControllerEditorState::new(1);
        state.add_cc_point_impl(480, 100).unwrap();

        assert_eq!(state.interpolate_value_at_tick(0), 100);
        assert_eq!(state.interpolate_value_at_tick(240), 100);
    }

    #[test]
    fn test_interpolate_value_at_tick_after_last() {
        let mut state = ControllerEditorState::new(1);
        state.add_cc_point_impl(0, 50).unwrap();

        assert_eq!(state.interpolate_value_at_tick(480), 50);
        assert_eq!(state.interpolate_value_at_tick(960), 50);
    }

    #[test]
    fn test_interpolate_value_at_tick_no_points() {
        let state = ControllerEditorState::new(1);
        assert_eq!(state.interpolate_value_at_tick(0), 64);
    }

    #[test]
    fn test_clear() {
        let mut state = ControllerEditorState::new(1);
        state.add_cc_point_impl(0, 64).unwrap();
        state.add_cc_point_impl(480, 80).unwrap();

        state.clear();
        assert_eq!(state.automation_points.len(), 0);
    }

    #[test]
    fn test_point_count() {
        let mut state = ControllerEditorState::new(1);
        assert_eq!(state.point_count(), 0);

        state.add_cc_point_impl(0, 64).unwrap();
        assert_eq!(state.point_count(), 1);

        state.add_cc_point_impl(480, 80).unwrap();
        assert_eq!(state.point_count(), 2);
    }
}
