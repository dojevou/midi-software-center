//! Automation commands for Tauri frontend
//!
//! Grown-up Script: Tauri commands with side effects for automation management.

use crate::automation::{
    AutomationLane, AutomationManager, CurveType, ParameterType,
};
use std::sync::Mutex;
use tauri::State;

/// Global automation manager state
pub struct AutomationState {
    manager: Mutex<AutomationManager>,
}

impl AutomationState {
    pub fn new() -> Self {
        Self {
            manager: Mutex::new(AutomationManager::new()),
        }
    }
}

impl Default for AutomationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create automation lane
///
/// # Arguments
/// * `track_id` - Parent track ID
/// * `parameter_type` - Parameter to automate
///
/// # Returns
/// Lane ID, or error message
#[tauri::command]
pub fn create_automation_lane(
    track_id: i32,
    parameter_type: ParameterType,
    state: State<'_, AutomationState>,
) -> Result<i32, String> {
    let mut manager = state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.create_lane(track_id, parameter_type)
}

/// Delete automation lane
///
/// # Arguments
/// * `track_id` - Parent track ID
/// * `parameter_type` - Parameter type
///
/// # Returns
/// Ok or error message
#[tauri::command]
pub fn delete_automation_lane(
    track_id: i32,
    parameter_type: ParameterType,
    state: State<'_, AutomationState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.delete_lane(track_id, parameter_type)
}

/// Add automation point
///
/// # Arguments
/// * `track_id` - Parent track ID
/// * `parameter_type` - Parameter type
/// * `time` - Time in ticks
/// * `value` - Normalized value (0.0-1.0)
///
/// # Returns
/// Point ID, or error message
#[tauri::command]
pub fn add_automation_point(
    track_id: i32,
    parameter_type: ParameterType,
    time: u64,
    value: f64,
    state: State<'_, AutomationState>,
) -> Result<i32, String> {
    let mut manager = state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.add_point(track_id, parameter_type, time, value)
}

/// Remove automation point
///
/// # Arguments
/// * `track_id` - Parent track ID
/// * `parameter_type` - Parameter type
/// * `point_id` - Point ID to remove
///
/// # Returns
/// Ok or error message
#[tauri::command]
pub fn remove_automation_point(
    track_id: i32,
    parameter_type: ParameterType,
    point_id: i32,
    state: State<'_, AutomationState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.remove_point(track_id, parameter_type, point_id)
}

/// Move automation point
///
/// # Arguments
/// * `track_id` - Parent track ID
/// * `parameter_type` - Parameter type
/// * `point_id` - Point ID to move
/// * `new_time` - New time position
/// * `new_value` - New normalized value
///
/// # Returns
/// Ok or error message
#[tauri::command]
pub fn move_automation_point(
    track_id: i32,
    parameter_type: ParameterType,
    point_id: i32,
    new_time: u64,
    new_value: f64,
    state: State<'_, AutomationState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.move_point(track_id, parameter_type, point_id, new_time, new_value)
}

/// Set automation curve type
///
/// # Arguments
/// * `track_id` - Parent track ID
/// * `parameter_type` - Parameter type
/// * `curve_type` - New curve type
///
/// # Returns
/// Ok or error message
#[tauri::command]
pub fn set_automation_curve_type(
    track_id: i32,
    parameter_type: ParameterType,
    curve_type: CurveType,
    state: State<'_, AutomationState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.set_curve_type(track_id, parameter_type, curve_type)
}

/// Get automation lane
///
/// # Arguments
/// * `track_id` - Parent track ID
/// * `parameter_type` - Parameter type
///
/// # Returns
/// Automation lane, or error message
#[tauri::command]
pub fn get_automation_lane(
    track_id: i32,
    parameter_type: ParameterType,
    state: State<'_, AutomationState>,
) -> Result<AutomationLane, String> {
    let manager = state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.get_lane(track_id, parameter_type)
}

/// Get all automation lanes for track
///
/// # Arguments
/// * `track_id` - Track ID
///
/// # Returns
/// Vector of all lanes for track
#[tauri::command]
pub fn get_track_automation(
    track_id: i32,
    state: State<'_, AutomationState>,
) -> Result<Vec<AutomationLane>, String> {
    let manager = state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    Ok(manager.get_track_lanes(track_id))
}

/// Get automation value at specific time
///
/// # Arguments
/// * `track_id` - Track ID
/// * `parameter_type` - Parameter type
/// * `time` - Time in ticks
///
/// # Returns
/// Interpolated value, or None if no automation
#[tauri::command]
pub fn get_automation_value(
    track_id: i32,
    parameter_type: ParameterType,
    time: u64,
    state: State<'_, AutomationState>,
) -> Result<Option<f64>, String> {
    let manager = state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    Ok(manager.get_value_at(track_id, parameter_type, time))
}

/// Clear all automation for track
///
/// # Arguments
/// * `track_id` - Track ID
///
/// # Returns
/// Ok or error message
#[tauri::command]
pub fn clear_track_automation(
    track_id: i32,
    state: State<'_, AutomationState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.remove_track(track_id);
    Ok(())
}

/// Clear all automation
///
/// # Returns
/// Ok or error message
#[tauri::command]
pub fn clear_all_automation(
    state: State<'_, AutomationState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock()
        .map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.clear_all();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::AutomationManager;

    // Test the automation manager directly (not Tauri commands)
    #[test]
    fn test_create_automation_lane() {
        let mut manager = AutomationManager::new();
        let result = manager.create_lane(1, ParameterType::Volume);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_create_duplicate_lane_fails() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();

        let result = manager.create_lane(1, ParameterType::Volume);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_automation_lane() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();

        let result = manager.delete_lane(1, ParameterType::Volume);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_nonexistent_lane_fails() {
        let mut manager = AutomationManager::new();
        let result = manager.delete_lane(1, ParameterType::Volume);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_automation_point() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();

        let result = manager.add_point(1, ParameterType::Volume, 100, 0.5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_add_point_to_nonexistent_lane_fails() {
        let mut manager = AutomationManager::new();
        let result = manager.add_point(1, ParameterType::Volume, 100, 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_automation_point() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();
        let point_id = manager.add_point(1, ParameterType::Volume, 100, 0.5).unwrap();

        let result = manager.remove_point(1, ParameterType::Volume, point_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_nonexistent_point_fails() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();

        let result = manager.remove_point(1, ParameterType::Volume, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_move_automation_point() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();
        let point_id = manager.add_point(1, ParameterType::Volume, 100, 0.5).unwrap();

        let result = manager.move_point(1, ParameterType::Volume, point_id, 200, 0.7);
        assert!(result.is_ok());
    }

    #[test]
    fn test_move_point_clamps_value() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();
        let point_id = manager.add_point(1, ParameterType::Volume, 100, 0.5).unwrap();

        // Try to move to out-of-range value
        let result = manager.move_point(1, ParameterType::Volume, point_id, 200, 2.0);
        assert!(result.is_ok());

        // Verify value was clamped
        let value = manager.get_value_at(1, ParameterType::Volume, 200);
        assert_eq!(value, Some(1.0));
    }

    #[test]
    fn test_set_automation_curve_type() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();

        let result = manager.set_curve_type(1, ParameterType::Volume, CurveType::Bezier);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_automation_lane() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();

        let result = manager.get_lane(1, ParameterType::Volume);
        assert!(result.is_ok());
        let lane = result.unwrap();
        assert_eq!(lane.track_id, 1);
        assert_eq!(lane.parameter_type, ParameterType::Volume);
    }

    #[test]
    fn test_get_nonexistent_lane_fails() {
        let manager = AutomationManager::new();
        let result = manager.get_lane(1, ParameterType::Volume);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_track_automation() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();
        manager.create_lane(1, ParameterType::Pan).unwrap();

        let lanes = manager.get_track_lanes(1);
        assert_eq!(lanes.len(), 2);
    }

    #[test]
    fn test_get_track_automation_empty() {
        let manager = AutomationManager::new();
        let lanes = manager.get_track_lanes(999);
        assert_eq!(lanes.len(), 0);
    }

    #[test]
    fn test_get_automation_value() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();
        manager.add_point(1, ParameterType::Volume, 100, 0.5).unwrap();

        let value = manager.get_value_at(1, ParameterType::Volume, 100);
        assert_eq!(value, Some(0.5));
    }

    #[test]
    fn test_get_automation_value_interpolated() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();
        manager.add_point(1, ParameterType::Volume, 0, 0.0).unwrap();
        manager.add_point(1, ParameterType::Volume, 100, 1.0).unwrap();

        // Midpoint should be ~0.5 for linear interpolation
        let value = manager.get_value_at(1, ParameterType::Volume, 50).unwrap();
        assert!((value - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_get_automation_value_no_points() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();

        let value = manager.get_value_at(1, ParameterType::Volume, 100);
        assert_eq!(value, None);
    }

    #[test]
    fn test_clear_track_automation() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();

        manager.remove_track(1);

        let lanes = manager.get_track_lanes(1);
        assert_eq!(lanes.len(), 0);
    }

    #[test]
    fn test_clear_all_automation() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();
        manager.create_lane(2, ParameterType::Pan).unwrap();

        manager.clear_all();

        let lanes1 = manager.get_track_lanes(1);
        let lanes2 = manager.get_track_lanes(2);
        assert_eq!(lanes1.len(), 0);
        assert_eq!(lanes2.len(), 0);
    }

    #[test]
    fn test_multiple_tracks() {
        let mut manager = AutomationManager::new();

        manager.create_lane(1, ParameterType::Volume).unwrap();
        manager.create_lane(2, ParameterType::Volume).unwrap();

        manager.add_point(1, ParameterType::Volume, 100, 0.3).unwrap();
        manager.add_point(2, ParameterType::Volume, 100, 0.7).unwrap();

        let value1 = manager.get_value_at(1, ParameterType::Volume, 100);
        let value2 = manager.get_value_at(2, ParameterType::Volume, 100);

        assert_eq!(value1, Some(0.3));
        assert_eq!(value2, Some(0.7));
    }

    #[test]
    fn test_multiple_parameter_types() {
        let mut manager = AutomationManager::new();

        manager.create_lane(1, ParameterType::Volume).unwrap();
        manager.create_lane(1, ParameterType::Pan).unwrap();
        manager.create_lane(1, ParameterType::CC(7)).unwrap();

        let lanes = manager.get_track_lanes(1);
        assert_eq!(lanes.len(), 3);
    }

    #[test]
    fn test_workflow_create_add_move_remove() {
        let mut manager = AutomationManager::new();

        // Create lane
        manager.create_lane(1, ParameterType::Volume).unwrap();

        // Add points
        let p1 = manager.add_point(1, ParameterType::Volume, 0, 0.0).unwrap();
        let p2 = manager.add_point(1, ParameterType::Volume, 100, 1.0).unwrap();

        // Move point
        manager.move_point(1, ParameterType::Volume, p1, 50, 0.3).unwrap();

        // Verify value
        let value = manager.get_value_at(1, ParameterType::Volume, 50);
        assert_eq!(value, Some(0.3));

        // Remove point
        manager.remove_point(1, ParameterType::Volume, p2).unwrap();

        // Verify only one point remains
        let lane = manager.get_lane(1, ParameterType::Volume).unwrap();
        assert_eq!(lane.curve.point_count(), 1);
    }
}
