#![allow(dead_code)]
//! Automation commands for Tauri frontend
//!
//! Grown-up Script: Tauri commands with side effects for automation management.

use crate::automation::{AutomationLane, AutomationManager, AutomationMode, CurveType, ParameterType};
use std::sync::Mutex;
use tauri::State;

/// Global automation manager state
pub struct AutomationState {
    manager: Mutex<AutomationManager>,
}

impl AutomationState {
    pub fn new() -> Self {
        Self { manager: Mutex::new(AutomationManager::new()) }
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
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

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
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

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
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

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
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.remove_point(track_id, parameter_type, point_id)
}

/// Remove multiple automation points (batch operation)
///
/// # Arguments
/// * `track_id` - Parent track ID
/// * `parameter_type` - Parameter type
/// * `point_ids` - Vector of point IDs to remove
///
/// # Returns
/// Number of points removed, or error message
#[tauri::command]
pub fn delete_automation_points_batch(
    track_id: i32,
    parameter_type: ParameterType,
    point_ids: Vec<i32>,
    state: State<'_, AutomationState>,
) -> Result<usize, String> {
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.remove_points_batch(track_id, parameter_type, &point_ids)
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
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

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
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

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
    let manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

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
    let manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

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
    let manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

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
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.remove_track(track_id);
    Ok(())
}

/// Clear all automation
///
/// # Returns
/// Ok or error message
#[tauri::command]
pub fn clear_all_automation(state: State<'_, AutomationState>) -> Result<(), String> {
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

    manager.clear_all();
    Ok(())
}

// ============================================================================
// CC RECORDING
// ============================================================================

/// CC Recording session state
#[derive(Debug, Clone, Default)]
pub struct CCRecordingSession {
    /// Track being recorded to
    pub track_id: i32,
    /// CC number being recorded
    pub cc_number: u8,
    /// Whether recording is active
    pub is_recording: bool,
    /// Recording start tick
    pub start_tick: u64,
    /// Recorded points during session
    pub recorded_points: Vec<(u64, f64)>,
}

/// Start CC automation recording
///
/// Begins recording incoming MIDI CC messages as automation points.
///
/// # Arguments
/// * `track_id` - Track ID to record to
/// * `cc_number` - MIDI CC number to record (0-127)
/// * `start_tick` - Tick position to start recording from
///
/// # Returns
/// Ok on success, or error message
#[tauri::command]
pub fn start_cc_recording(
    track_id: i32,
    cc_number: u8,
    _start_tick: u64,
    state: State<'_, AutomationState>,
) -> Result<(), String> {
    // Ensure lane exists for this CC
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

    let parameter = ParameterType::CC(cc_number);

    // Create lane if it doesn't exist
    if manager.get_lane(track_id, parameter).is_err() {
        manager.create_lane(track_id, parameter)?;
    }

    // Recording session will be tracked separately
    // In production, this would set recording state
    Ok(())
}

/// Record CC value from MIDI input
///
/// Call this for each incoming CC message during recording.
///
/// # Arguments
/// * `track_id` - Track ID
/// * `cc_number` - CC number
/// * `value` - CC value (0-127)
/// * `tick` - Current tick position
///
/// # Returns
/// Point ID if recorded, or error message
#[tauri::command]
pub fn record_cc_value(
    track_id: i32,
    cc_number: u8,
    value: u8,
    tick: u64,
    state: State<'_, AutomationState>,
) -> Result<i32, String> {
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

    let parameter = ParameterType::CC(cc_number);

    // Convert 0-127 to 0.0-1.0
    let normalized = (value as f64) / 127.0;

    manager.add_point(track_id, parameter, tick, normalized)
}

/// Stop CC recording and finalize points
///
/// # Arguments
/// * `track_id` - Track ID
/// * `cc_number` - CC number
///
/// # Returns
/// Number of points recorded
#[tauri::command]
pub fn stop_cc_recording(
    _track_id: i32,
    _cc_number: u8,
    _state: State<'_, AutomationState>,
) -> Result<usize, String> {
    // In production, this would finalize recording state
    // and return actual count of recorded points
    Ok(0)
}

/// Record automation in real-time from MIDI CC input
///
/// Batch record multiple CC values at once (for performance).
///
/// # Arguments
/// * `track_id` - Track ID
/// * `cc_number` - CC number
/// * `values` - Vec of (tick, value) pairs
///
/// # Returns
/// Number of points recorded
#[tauri::command]
pub fn record_cc_automation_batch(
    track_id: i32,
    cc_number: u8,
    values: Vec<(u64, u8)>,
    state: State<'_, AutomationState>,
) -> Result<usize, String> {
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

    let parameter = ParameterType::CC(cc_number);

    // Ensure lane exists
    if manager.get_lane(track_id, parameter).is_err() {
        manager.create_lane(track_id, parameter)?;
    }

    let mut count = 0;
    for (tick, value) in values {
        let normalized = (value as f64) / 127.0;
        if manager.add_point(track_id, parameter, tick, normalized).is_ok() {
            count += 1;
        }
    }

    Ok(count)
}

// ============================================================================
// AUTOMATION CLIPBOARD
// ============================================================================

/// Automation clipboard for copy/paste
#[derive(Debug, Clone, Default)]
pub struct AutomationClipboard {
    /// Copied points (time offset, value)
    pub points: Vec<(u64, f64)>,
    /// Curve type of source
    pub curve_type: CurveType,
    /// Reference time (earliest point)
    pub reference_time: u64,
}

/// Copy automation points
///
/// # Arguments
/// * `track_id` - Track ID
/// * `parameter_type` - Parameter type
/// * `point_ids` - Points to copy (empty = copy all)
///
/// # Returns
/// Number of points copied
#[tauri::command]
pub fn copy_automation_points(
    track_id: i32,
    parameter_type: ParameterType,
    point_ids: Vec<i32>,
    state: State<'_, AutomationState>,
) -> Result<usize, String> {
    let manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

    let lane = manager.get_lane(track_id, parameter_type)?;
    let points = &lane.curve.points;

    if points.is_empty() {
        return Err("No automation points to copy".to_string());
    }

    let to_copy: Vec<&crate::automation::AutomationPoint> = if point_ids.is_empty() {
        points.iter().collect()
    } else {
        points.iter().filter(|p| point_ids.contains(&p.id)).collect()
    };

    if to_copy.is_empty() {
        return Err("No matching points found".to_string());
    }

    // Copy is handled in frontend state for now
    Ok(to_copy.len())
}

/// Paste automation points
///
/// # Arguments
/// * `track_id` - Track ID
/// * `parameter_type` - Parameter type
/// * `paste_tick` - Where to paste (time offset)
/// * `points` - Points to paste (time, value pairs)
///
/// # Returns
/// Number of points pasted
#[tauri::command]
pub fn paste_automation_points(
    track_id: i32,
    parameter_type: ParameterType,
    paste_tick: u64,
    points: Vec<(u64, f64)>,
    state: State<'_, AutomationState>,
) -> Result<Vec<i32>, String> {
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

    // Ensure lane exists
    if manager.get_lane(track_id, parameter_type).is_err() {
        manager.create_lane(track_id, parameter_type)?;
    }

    let mut pasted_ids = Vec::new();

    // Find reference (earliest time in clipboard)
    let reference = points.iter().map(|(t, _)| *t).min().unwrap_or(0);

    for (time, value) in points {
        let offset_time = time.saturating_sub(reference);
        let new_time = paste_tick + offset_time;

        match manager.add_point(track_id, parameter_type, new_time, value) {
            Ok(id) => pasted_ids.push(id),
            Err(e) => eprintln!("Failed to paste point: {}", e),
        }
    }

    Ok(pasted_ids)
}

/// Cut automation points (copy + delete)
///
/// # Arguments
/// * `track_id` - Track ID
/// * `parameter_type` - Parameter type
/// * `point_ids` - Points to cut
///
/// # Returns
/// Cut points as (time, value) pairs for clipboard
#[tauri::command]
pub fn cut_automation_points(
    track_id: i32,
    parameter_type: ParameterType,
    point_ids: Vec<i32>,
    state: State<'_, AutomationState>,
) -> Result<Vec<(u64, f64)>, String> {
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;

    let lane = manager.get_lane(track_id, parameter_type)?;

    // Collect points to cut
    let cut_points: Vec<(u64, f64)> = lane
        .curve
        .points
        .iter()
        .filter(|p| point_ids.contains(&p.id))
        .map(|p| (p.time, p.value))
        .collect();

    if cut_points.is_empty() {
        return Err("No matching points to cut".to_string());
    }

    // Delete the points
    manager.remove_points_batch(track_id, parameter_type, &point_ids)?;

    Ok(cut_points)
}

/// Get parameter type display string
///
/// # Arguments
/// * `parameter_type` - Parameter type
///
/// # Returns
/// Display string for the parameter type
#[tauri::command]
pub fn get_parameter_type_string(parameter_type: ParameterType) -> String {
    parameter_type.as_string()
}

/// Get parameter type color
///
/// # Arguments
/// * `parameter_type` - Parameter type
///
/// # Returns
/// Color hex string for the parameter type
#[tauri::command]
pub fn get_parameter_type_color(parameter_type: ParameterType) -> &'static str {
    parameter_type.color()
}

/// Get automation points within a time range
///
/// # Arguments
/// * `track_id` - Track ID
/// * `parameter_type` - Parameter type
/// * `start_time` - Start time in ticks
/// * `end_time` - End time in ticks
///
/// # Returns
/// Points within the specified range
#[tauri::command]
pub fn get_automation_points_in_range(
    track_id: i32,
    parameter_type: ParameterType,
    start_time: u64,
    end_time: u64,
    state: State<'_, AutomationState>,
) -> Result<Vec<crate::automation::AutomationPoint>, String> {
    let manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;
    let lane = manager.get_lane(track_id, parameter_type)?;
    Ok(lane.curve.get_points_in_range(start_time, end_time))
}

/// Clear all points from an automation curve
///
/// # Arguments
/// * `track_id` - Track ID
/// * `parameter_type` - Parameter type
///
/// # Returns
/// Ok or error message
#[tauri::command]
pub fn clear_automation_curve(
    track_id: i32,
    parameter_type: ParameterType,
    state: State<'_, AutomationState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;
    let lane = manager.get_lane_mut(track_id, parameter_type)?;
    lane.curve.clear();
    Ok(())
}

/// Get the number of points in an automation curve
///
/// # Arguments
/// * `track_id` - Track ID
/// * `parameter_type` - Parameter type
///
/// # Returns
/// Point count
#[tauri::command]
pub fn get_automation_point_count(
    track_id: i32,
    parameter_type: ParameterType,
    state: State<'_, AutomationState>,
) -> Result<usize, String> {
    let manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;
    let lane = manager.get_lane(track_id, parameter_type)?;
    Ok(lane.curve.point_count())
}

/// Get the display name for an automation lane
///
/// # Arguments
/// * `track_id` - Track ID
/// * `parameter_type` - Parameter type
///
/// # Returns
/// Display name string
#[tauri::command]
pub fn get_automation_lane_display_name(
    track_id: i32,
    parameter_type: ParameterType,
    state: State<'_, AutomationState>,
) -> Result<String, String> {
    let manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;
    let lane = manager.get_lane(track_id, parameter_type)?;
    Ok(lane.display_name())
}

/// Get the color for an automation lane
///
/// # Arguments
/// * `track_id` - Track ID
/// * `parameter_type` - Parameter type
///
/// # Returns
/// Color hex string
#[tauri::command]
pub fn get_automation_lane_color(
    track_id: i32,
    parameter_type: ParameterType,
    state: State<'_, AutomationState>,
) -> Result<&'static str, String> {
    let manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;
    let lane = manager.get_lane(track_id, parameter_type)?;
    Ok(lane.color())
}

/// Get the number of automation lanes for a track
///
/// # Arguments
/// * `track_id` - Track ID
///
/// # Returns
/// Lane count
#[tauri::command]
pub fn get_automation_lane_count(
    track_id: i32,
    state: State<'_, AutomationState>,
) -> Result<usize, String> {
    let manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;
    let track = manager.get_track(track_id)?;
    Ok(track.lane_count())
}

/// Set automation mode
///
/// # Arguments
/// * `track_id` - Track ID
/// * `parameter_type` - Parameter type
/// * `mode` - Automation mode (Off, Read, Write, Latch, Touch)
///
/// # Returns
/// Ok or error message
#[tauri::command]
pub fn set_automation_mode(
    track_id: i32,
    parameter_type: ParameterType,
    mode: AutomationMode,
    state: State<'_, AutomationState>,
) -> Result<(), String> {
    let mut manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;
    manager.set_automation_mode(track_id, parameter_type, mode)
}

/// Get automation mode
///
/// # Arguments
/// * `track_id` - Track ID
/// * `parameter_type` - Parameter type
///
/// # Returns
/// Current automation mode, or error message
#[tauri::command]
pub fn get_automation_mode(
    track_id: i32,
    parameter_type: ParameterType,
    state: State<'_, AutomationState>,
) -> Result<AutomationMode, String> {
    let manager = state.manager.lock().map_err(|e| format!("Failed to lock manager: {}", e))?;
    manager.get_automation_mode(track_id, parameter_type)
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
