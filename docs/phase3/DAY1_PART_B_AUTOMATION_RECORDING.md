# Day 1, Part 1B: Automation Recording

**Duration:** 2 hours
**Prerequisites:** Part 1A complete (automation models)
**Files to create:** 2

---

## Overview

Build automation recording system:
1. AutomationRecorder component
2. Recording modes (write, latch, touch)
3. Parameter change capture
4. Tauri commands for automation control

---

## Step 1: Automation Recorder (45 min)

Create `app/src-tauri/src/daw/automation/recorder.rs`:

```rust
use super::models::{AutomationLane, AutomationPoint, AutomationMode};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Automation recorder state
pub struct AutomationRecorder {
    /// All automation lanes (key: lane_id)
    lanes: RwLock<HashMap<String, AutomationLane>>,

    /// Currently recording lanes (key: lane_id, value: is_touching)
    recording_state: RwLock<HashMap<String, bool>>,

    /// Current playhead position in ticks
    current_position: RwLock<u64>,
}

impl AutomationRecorder {
    pub fn new() -> Self {
        Self {
            lanes: RwLock::new(HashMap::new()),
            recording_state: RwLock::new(HashMap::new()),
            current_position: RwLock::new(0),
        }
    }

    /// Add or update automation lane
    pub async fn add_lane(&self, lane: AutomationLane) {
        let mut lanes = self.lanes.write().await;
        lanes.insert(lane.id.clone(), lane);
    }

    /// Get automation lane
    pub async fn get_lane(&self, lane_id: &str) -> Option<AutomationLane> {
        let lanes = self.lanes.read().await;
        lanes.get(lane_id).cloned()
    }

    /// Get all lanes for a track
    pub async fn get_track_lanes(&self, track_id: u32) -> Vec<AutomationLane> {
        let lanes = self.lanes.read().await;
        lanes
            .values()
            .filter(|lane| lane.track_id == track_id)
            .cloned()
            .collect()
    }

    /// Remove automation lane
    pub async fn remove_lane(&self, lane_id: &str) -> Option<AutomationLane> {
        let mut lanes = self.lanes.write().await;
        let mut recording = self.recording_state.write().await;
        recording.remove(lane_id);
        lanes.remove(lane_id)
    }

    /// Update playhead position
    pub async fn set_position(&self, position_ticks: u64) {
        let mut pos = self.current_position.write().await;
        *pos = position_ticks;
    }

    /// Get current position
    pub async fn get_position(&self) -> u64 {
        *self.current_position.read().await
    }

    /// Record parameter change
    pub async fn record_parameter(
        &self,
        lane_id: &str,
        value: f32,
        is_touching: bool,
    ) -> Result<(), String> {
        let position = self.get_position().await;
        let mut lanes = self.lanes.write().await;
        let mut recording = self.recording_state.write().await;

        let lane = lanes
            .get_mut(lane_id)
            .ok_or_else(|| format!("Lane {} not found", lane_id))?;

        if !lane.enabled {
            return Ok(());
        }

        match lane.mode {
            AutomationMode::Read | AutomationMode::Off => {
                // Not recording
                Ok(())
            }

            AutomationMode::Write => {
                // Always record
                let point = AutomationPoint::new(position, value);
                lane.add_point(point);
                Ok(())
            }

            AutomationMode::Latch => {
                // Start recording on first change, continue until stopped
                let is_recording = recording.get(lane_id).copied().unwrap_or(false);

                if is_touching || is_recording {
                    recording.insert(lane_id.to_string(), true);
                    let point = AutomationPoint::new(position, value);
                    lane.add_point(point);
                }
                Ok(())
            }

            AutomationMode::Touch => {
                // Record only while touching
                if is_touching {
                    recording.insert(lane_id.to_string(), true);
                    let point = AutomationPoint::new(position, value);
                    lane.add_point(point);
                } else {
                    recording.insert(lane_id.to_string(), false);
                }
                Ok(())
            }
        }
    }

    /// Stop recording on a lane (for latch mode)
    pub async fn stop_recording(&self, lane_id: &str) {
        let mut recording = self.recording_state.write().await;
        recording.insert(lane_id.to_string(), false);
    }

    /// Clear all automation points in a lane
    pub async fn clear_lane(&self, lane_id: &str) -> Result<(), String> {
        let mut lanes = self.lanes.write().await;
        let lane = lanes
            .get_mut(lane_id)
            .ok_or_else(|| format!("Lane {} not found", lane_id))?;

        lane.clear();
        Ok(())
    }

    /// Set automation mode for a lane
    pub async fn set_lane_mode(&self, lane_id: &str, mode: AutomationMode) -> Result<(), String> {
        let mut lanes = self.lanes.write().await;
        let lane = lanes
            .get_mut(lane_id)
            .ok_or_else(|| format!("Lane {} not found", lane_id))?;

        lane.mode = mode;

        // Reset recording state when changing mode
        let mut recording = self.recording_state.write().await;
        recording.insert(lane_id.to_string(), false);

        Ok(())
    }

    /// Enable/disable automation lane
    pub async fn set_lane_enabled(&self, lane_id: &str, enabled: bool) -> Result<(), String> {
        let mut lanes = self.lanes.write().await;
        let lane = lanes
            .get_mut(lane_id)
            .ok_or_else(|| format!("Lane {} not found", lane_id))?;

        lane.enabled = enabled;
        Ok(())
    }

    /// Get all lanes
    pub async fn get_all_lanes(&self) -> Vec<AutomationLane> {
        let lanes = self.lanes.read().await;
        lanes.values().cloned().collect()
    }
}

impl Default for AutomationRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_mode() {
        let recorder = AutomationRecorder::new();
        let mut lane = AutomationLane::new(
            "auto1".to_string(),
            1,
            "gain".to_string(),
            "Gain".to_string(),
        );
        lane.mode = AutomationMode::Write;

        recorder.add_lane(lane).await;
        recorder.set_position(100).await;

        recorder.record_parameter("auto1", 0.5, false).await.unwrap();

        let lane = recorder.get_lane("auto1").await.unwrap();
        assert_eq!(lane.points.len(), 1);
        assert_eq!(lane.points[0].time_ticks, 100);
        assert_eq!(lane.points[0].value, 0.5);
    }

    #[tokio::test]
    async fn test_touch_mode() {
        let recorder = AutomationRecorder::new();
        let mut lane = AutomationLane::new(
            "auto1".to_string(),
            1,
            "gain".to_string(),
            "Gain".to_string(),
        );
        lane.mode = AutomationMode::Touch;

        recorder.add_lane(lane).await;
        recorder.set_position(100).await;

        // Not touching - should not record
        recorder.record_parameter("auto1", 0.5, false).await.unwrap();
        let lane = recorder.get_lane("auto1").await.unwrap();
        assert_eq!(lane.points.len(), 0);

        // Touching - should record
        recorder.record_parameter("auto1", 0.5, true).await.unwrap();
        let lane = recorder.get_lane("auto1").await.unwrap();
        assert_eq!(lane.points.len(), 1);
    }

    #[tokio::test]
    async fn test_latch_mode() {
        let recorder = AutomationRecorder::new();
        let mut lane = AutomationLane::new(
            "auto1".to_string(),
            1,
            "gain".to_string(),
            "Gain".to_string(),
        );
        lane.mode = AutomationMode::Latch;

        recorder.add_lane(lane).await;
        recorder.set_position(100).await;

        // First touch starts recording
        recorder.record_parameter("auto1", 0.5, true).await.unwrap();

        recorder.set_position(200).await;
        // Continue recording even after release
        recorder.record_parameter("auto1", 0.8, false).await.unwrap();

        let lane = recorder.get_lane("auto1").await.unwrap();
        assert_eq!(lane.points.len(), 2);

        // Stop recording
        recorder.stop_recording("auto1").await;

        recorder.set_position(300).await;
        recorder.record_parameter("auto1", 1.0, false).await.unwrap();

        let lane = recorder.get_lane("auto1").await.unwrap();
        assert_eq!(lane.points.len(), 2); // Should not add point
    }
}
```

---

## Step 2: Integration with SequencerEngine (30 min)

Update `app/src-tauri/src/daw/sequencer/engine.rs`:

```rust
use crate::daw::automation::{AutomationRecorder, AutomationLane, AutomationMode};
use std::sync::Arc;

pub struct SequencerEngine {
    // ... existing fields
    automation_recorder: Arc<AutomationRecorder>,  // NEW
}

impl SequencerEngine {
    pub fn new() -> Self {
        Self {
            // ... existing fields
            automation_recorder: Arc::new(AutomationRecorder::new()),
        }
    }

    // Automation methods
    pub fn get_automation_recorder(&self) -> Arc<AutomationRecorder> {
        Arc::clone(&self.automation_recorder)
    }

    pub async fn add_automation_lane(&self, lane: AutomationLane) {
        self.automation_recorder.add_lane(lane).await;
    }

    pub async fn record_automation(
        &self,
        lane_id: &str,
        value: f32,
        is_touching: bool,
    ) -> Result<(), String> {
        self.automation_recorder
            .record_parameter(lane_id, value, is_touching)
            .await
    }

    pub async fn set_automation_mode(
        &self,
        lane_id: &str,
        mode: AutomationMode,
    ) -> Result<(), String> {
        self.automation_recorder.set_lane_mode(lane_id, mode).await
    }

    pub async fn clear_automation_lane(&self, lane_id: &str) -> Result<(), String> {
        self.automation_recorder.clear_lane(lane_id).await
    }

    pub async fn get_automation_lane(&self, lane_id: &str) -> Option<AutomationLane> {
        self.automation_recorder.get_lane(lane_id).await
    }

    pub async fn get_track_automation_lanes(&self, track_id: u32) -> Vec<AutomationLane> {
        self.automation_recorder.get_track_lanes(track_id).await
    }

    pub async fn remove_automation_lane(&self, lane_id: &str) -> Option<AutomationLane> {
        self.automation_recorder.remove_lane(lane_id).await
    }
}
```

---

## Step 3: Tauri Commands (30 min)

Create `app/src-tauri/src/commands/daw/automation_commands.rs`:

```rust
use crate::daw::automation::{AutomationLane, AutomationMode, AutomationPoint, CurveType};
use crate::AppState;
use tauri::State;

/// Create automation lane
#[tauri::command]
pub async fn create_automation_lane(
    track_id: u32,
    parameter_id: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<AutomationLane, String> {
    log::info!(
        "Creating automation lane for track {} parameter {}",
        track_id,
        parameter_id
    );

    let lane_id = format!("auto_{}_{}", track_id, parameter_id);
    let lane = AutomationLane::new(lane_id, track_id, parameter_id, name);

    let sequencer = state.sequencer.lock().await;
    sequencer.add_automation_lane(lane.clone()).await;

    Ok(lane)
}

/// Get automation lane
#[tauri::command]
pub async fn get_automation_lane(
    lane_id: String,
    state: State<'_, AppState>,
) -> Result<Option<AutomationLane>, String> {
    let sequencer = state.sequencer.lock().await;
    Ok(sequencer.get_automation_lane(&lane_id).await)
}

/// Get all automation lanes for a track
#[tauri::command]
pub async fn get_track_automation_lanes(
    track_id: u32,
    state: State<'_, AppState>,
) -> Result<Vec<AutomationLane>, String> {
    let sequencer = state.sequencer.lock().await;
    Ok(sequencer.get_track_automation_lanes(track_id).await)
}

/// Set automation mode
#[tauri::command]
pub async fn set_automation_mode(
    lane_id: String,
    mode: AutomationMode,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting automation mode for {}: {:?}", lane_id, mode);

    let sequencer = state.sequencer.lock().await;
    sequencer.set_automation_mode(&lane_id, mode).await
}

/// Record automation point
#[tauri::command]
pub async fn record_automation(
    lane_id: String,
    value: f32,
    is_touching: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let sequencer = state.sequencer.lock().await;
    sequencer
        .record_automation(&lane_id, value, is_touching)
        .await
}

/// Add automation point manually
#[tauri::command]
pub async fn add_automation_point(
    lane_id: String,
    time_ticks: u64,
    value: f32,
    curve_type: CurveType,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Adding automation point to {} at {} = {}",
        lane_id,
        time_ticks,
        value
    );

    let sequencer = state.sequencer.lock().await;
    let recorder = sequencer.get_automation_recorder();

    let point = AutomationPoint::with_curve(time_ticks, value, curve_type);

    let mut lanes = recorder.lanes.write().await;
    let lane = lanes
        .get_mut(&lane_id)
        .ok_or_else(|| format!("Lane {} not found", lane_id))?;

    lane.add_point(point);
    Ok(())
}

/// Remove automation point
#[tauri::command]
pub async fn remove_automation_point(
    lane_id: String,
    time_ticks: u64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Removing automation point from {} at {}", lane_id, time_ticks);

    let sequencer = state.sequencer.lock().await;
    let recorder = sequencer.get_automation_recorder();

    let mut lanes = recorder.lanes.write().await;
    let lane = lanes
        .get_mut(&lane_id)
        .ok_or_else(|| format!("Lane {} not found", lane_id))?;

    lane.remove_point(time_ticks)
        .ok_or_else(|| format!("Point at {} not found", time_ticks))?;

    Ok(())
}

/// Clear automation lane
#[tauri::command]
pub async fn clear_automation_lane(
    lane_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Clearing automation lane {}", lane_id);

    let sequencer = state.sequencer.lock().await;
    sequencer.clear_automation_lane(&lane_id).await
}

/// Delete automation lane
#[tauri::command]
pub async fn delete_automation_lane(
    lane_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Deleting automation lane {}", lane_id);

    let sequencer = state.sequencer.lock().await;
    sequencer.remove_automation_lane(&lane_id).await;

    Ok(())
}
```

Register in `app/src-tauri/src/commands/daw/mod.rs`:

```rust
pub mod automation_commands;
pub use automation_commands::*;
```

Register in `app/src-tauri/src/main.rs`:

```rust
use midi_app::commands::daw::{
    // ... existing commands
    create_automation_lane,
    get_automation_lane,
    get_track_automation_lanes,
    set_automation_mode,
    record_automation,
    add_automation_point,
    remove_automation_point,
    clear_automation_lane,
    delete_automation_lane,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing handlers
            create_automation_lane,
            get_automation_lane,
            get_track_automation_lanes,
            set_automation_mode,
            record_automation,
            add_automation_point,
            remove_automation_point,
            clear_automation_lane,
            delete_automation_lane,
        ])
        // ...
}
```

---

## Step 4: Update automation module (15 min)

Update `app/src-tauri/src/daw/automation/mod.rs`:

```rust
pub mod models;
pub mod recorder;

pub use models::{AutomationPoint, AutomationLane, AutomationMode, CurveType};
pub use recorder::AutomationRecorder;
```

---

## Verification (15 min)

```bash
cd app/src-tauri
cargo check
cargo test --lib automation
```

Test in browser console:

```javascript
// Create automation lane for track 1 gain
const lane = await window.__TAURI__.invoke('create_automation_lane', {
  trackId: 1,
  parameterId: 'gain',
  name: 'Gain'
});
console.log('Created lane:', lane);

// Set to write mode
await window.__TAURI__.invoke('set_automation_mode', {
  laneId: lane.id,
  mode: 'Write'
});

// Record some points
await window.__TAURI__.invoke('record_automation', {
  laneId: lane.id,
  value: 0.5,
  isTouching: true
});

// Get lane with points
const updated = await window.__TAURI__.invoke('get_automation_lane', {
  laneId: lane.id
});
console.log('Lane with points:', updated);

// Add manual point
await window.__TAURI__.invoke('add_automation_point', {
  laneId: lane.id,
  timeTicks: 500,
  value: 0.8,
  curveType: 'Linear'
});

// Clear lane
await window.__TAURI__.invoke('clear_automation_lane', {
  laneId: lane.id
});
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Recording not working | Check lane mode is not Read or Off |
| Latch mode not latching | Ensure first touch with is_touching=true |
| Touch mode always recording | Verify is_touching parameter is correct |
| Points not sorted | AutomationLane.add_point maintains sort order |

---

## What's Next?

✅ **You've completed:**
- AutomationRecorder with write/latch/touch modes
- Integration with SequencerEngine
- 9 Tauri commands for automation control
- Manual point add/remove functionality
- 3 unit tests for recording modes

**Next:** [Part 2A: Automation Playback](./DAY2_PART_A_AUTOMATION_PLAYBACK.md)
- Automation player component
- Apply automation during playback
- Real-time value interpolation
- Automation enable/disable per track
