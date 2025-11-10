// ARCHETYPE: MANAGER (Grown-up Script)
// Purpose: Track import progress and emit real-time updates to frontend
// Side effects: Emits Tauri events, manages mutable state

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

/// Progress state for file import operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressState {
    pub current_file: String,
    pub current_index: usize,
    pub total_files: usize,
    pub percentage: f64,
    pub phase: String,
    pub files_per_second: f64,
    pub errors_count: usize,
    pub duplicates_found: usize,
    pub estimated_time_remaining: f64,
}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            current_file: String::new(),
            current_index: 0,
            total_files: 0,
            percentage: 0.0,
            phase: "idle".to_string(),
            files_per_second: 0.0,
            errors_count: 0,
            duplicates_found: 0,
            estimated_time_remaining: 0.0,
        }
    }
}

/// Thread-safe progress tracker
#[derive(Clone)]
pub struct ProgressTracker {
    state: Arc<Mutex<ProgressState>>,
    start_time: Arc<Mutex<Option<std::time::Instant>>>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ProgressState::default())),
            start_time: Arc::new(Mutex::new(None)),
        }
    }

    /// Get current progress state
    pub fn get_state(&self) -> ProgressState {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()
    }

    /// Update state and return the new state
    fn update_state<F>(&self, updater: F) -> ProgressState
    where
        F: FnOnce(&mut ProgressState),
    {
        let mut state = self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        updater(&mut state);
        state.clone()
    }

    /// Calculate metrics based on current progress
    fn calculate_metrics(&self, current_index: usize, total_files: usize) -> (f64, f64) {
        let start_time = self.start_time.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(start) = *start_time {
            let elapsed = start.elapsed().as_secs_f64();

            if elapsed > 0.0 && current_index > 0 {
                let files_per_second = current_index as f64 / elapsed;
                let remaining_files = total_files.saturating_sub(current_index);
                let estimated_time_remaining = if files_per_second > 0.0 {
                    remaining_files as f64 / files_per_second
                } else {
                    0.0
                };

                return (files_per_second, estimated_time_remaining);
            }
        }

        (0.0, 0.0)
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Start progress tracking for a new import operation
#[tauri::command]
pub async fn start_progress_tracking(
    total: usize,
    tracker: State<'_, ProgressTracker>,
    app: AppHandle,
) -> Result<(), String> {
    // Reset start time
    *tracker.start_time.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) =
        Some(std::time::Instant::now());

    // Initialize state
    let state = tracker.update_state(|s| {
        s.current_file = String::new();
        s.current_index = 0;
        s.total_files = total;
        s.percentage = 0.0;
        s.phase = "scanning".to_string();
        s.files_per_second = 0.0;
        s.errors_count = 0;
        s.duplicates_found = 0;
        s.estimated_time_remaining = 0.0;
    });

    // Emit initial state
    app.emit("import-progress", &state)
        .map_err(|e| format!("Failed to emit progress: {}", e))?;

    Ok(())
}

/// Update progress with current file and phase
#[tauri::command]
pub async fn update_progress(
    current: usize,
    file: String,
    phase: String,
    tracker: State<'_, ProgressTracker>,
    app: AppHandle,
) -> Result<(), String> {
    let total = {
        let state = tracker.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        state.total_files
    };

    // Calculate metrics
    let (files_per_second, estimated_time_remaining) = tracker.calculate_metrics(current, total);

    // Update state
    let state = tracker.update_state(|s| {
        s.current_file = file;
        s.current_index = current;
        s.phase = phase;
        s.percentage = if total > 0 {
            (current as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        s.files_per_second = files_per_second;
        s.estimated_time_remaining = estimated_time_remaining;
    });

    // Emit updated state
    app.emit("import-progress", &state)
        .map_err(|e| format!("Failed to emit progress: {}", e))?;

    Ok(())
}

/// Mark an error during processing
#[tauri::command]
pub async fn increment_error_count(
    tracker: State<'_, ProgressTracker>,
    app: AppHandle,
) -> Result<(), String> {
    let state = tracker.update_state(|s| {
        s.errors_count += 1;
    });

    // Emit updated state
    app.emit("import-progress", &state)
        .map_err(|e| format!("Failed to emit progress: {}", e))?;

    Ok(())
}

/// Mark a duplicate file found
#[tauri::command]
pub async fn increment_duplicate_count(
    tracker: State<'_, ProgressTracker>,
    app: AppHandle,
) -> Result<(), String> {
    let state = tracker.update_state(|s| {
        s.duplicates_found += 1;
    });

    // Emit updated state
    app.emit("import-progress", &state)
        .map_err(|e| format!("Failed to emit progress: {}", e))?;

    Ok(())
}

/// Complete progress tracking
#[tauri::command]
pub async fn complete_progress(
    tracker: State<'_, ProgressTracker>,
    app: AppHandle,
) -> Result<(), String> {
    let state = tracker.update_state(|s| {
        s.percentage = 100.0;
        s.phase = "complete".to_string();
        s.estimated_time_remaining = 0.0;
    });

    // Emit final state
    app.emit("import-progress", &state)
        .map_err(|e| format!("Failed to emit progress: {}", e))?;

    // Reset start time
    *tracker.start_time.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = None;

    Ok(())
}

/// Get current progress state (for polling if needed)
#[tauri::command]
pub async fn get_current_progress(
    tracker: State<'_, ProgressTracker>,
) -> Result<ProgressState, String> {
    Ok(tracker.get_state())
}

/// Reset progress to idle state
#[tauri::command]
pub async fn reset_progress(
    tracker: State<'_, ProgressTracker>,
    app: AppHandle,
) -> Result<(), String> {
    *tracker.start_time.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = None;

    let state = tracker.update_state(|s| {
        *s = ProgressState::default();
    });

    // Emit reset state
    app.emit("import-progress", &state)
        .map_err(|e| format!("Failed to emit progress: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_state_default() {
        let state = ProgressState::default();
        assert_eq!(state.current_index, 0);
        assert_eq!(state.total_files, 0);
        assert_eq!(state.percentage, 0.0);
        assert_eq!(state.phase, "idle");
    }

    #[test]
    fn test_progress_tracker_new() {
        let tracker = ProgressTracker::new();
        let state = tracker.get_state();
        assert_eq!(state.current_index, 0);
        assert_eq!(state.total_files, 0);
    }

    #[test]
    fn test_progress_tracker_update() {
        let tracker = ProgressTracker::new();

        // Simulate starting tracking
        *tracker.start_time.lock().unwrap() = Some(std::time::Instant::now());

        let state = tracker.update_state(|s| {
            s.total_files = 100;
            s.current_index = 50;
            s.phase = "analyzing".to_string();
            s.percentage = 50.0;
        });

        assert_eq!(state.total_files, 100);
        assert_eq!(state.current_index, 50);
        assert_eq!(state.phase, "analyzing");
        assert_eq!(state.percentage, 50.0);
    }

    #[test]
    fn test_calculate_metrics() {
        let tracker = ProgressTracker::new();

        // Before start time is set
        let (fps, eta) = tracker.calculate_metrics(10, 100);
        assert_eq!(fps, 0.0);
        assert_eq!(eta, 0.0);

        // After start time is set
        *tracker.start_time.lock().unwrap() = Some(std::time::Instant::now());
        std::thread::sleep(std::time::Duration::from_millis(100));

        let (fps, eta) = tracker.calculate_metrics(10, 100);
        assert!(fps > 0.0); // Should be processing files
        assert!(eta > 0.0); // Should have estimated time
    }
}
