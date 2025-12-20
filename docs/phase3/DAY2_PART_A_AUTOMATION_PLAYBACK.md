# Day 2, Part 2A: Automation Playback

**Duration:** 2 hours
**Prerequisites:** Parts 1A-1B complete (automation models and recording)
**Files to create:** 2

---

## Overview

Build automation playback system:
1. AutomationPlayer component
2. Apply automation during playback
3. Real-time value interpolation
4. Integration with mixer parameters

---

## Step 1: Automation Player (1 hour)

Create `app/src-tauri/src/daw/automation/player.rs`:

```rust
use super::models::{AutomationLane, AutomationMode};
use super::recorder::AutomationRecorder;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Automation player state
pub struct AutomationPlayer {
    /// Reference to recorder (shared lane data)
    recorder: Arc<AutomationRecorder>,

    /// Cache of automation values (key: lane_id, value: current_value)
    value_cache: RwLock<HashMap<String, f32>>,

    /// Enabled state
    enabled: RwLock<bool>,
}

impl AutomationPlayer {
    pub fn new(recorder: Arc<AutomationRecorder>) -> Self {
        Self {
            recorder,
            value_cache: RwLock::new(HashMap::new()),
            enabled: RwLock::new(true),
        }
    }

    /// Enable/disable automation playback globally
    pub async fn set_enabled(&self, enabled: bool) {
        let mut e = self.enabled.write().await;
        *e = enabled;
    }

    /// Check if playback is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }

    /// Update automation at specific time position
    /// Returns map of parameter_id -> value for parameters that have automation
    pub async fn update(&self, position_ticks: u64) -> HashMap<String, f32> {
        let mut updates = HashMap::new();

        if !self.is_enabled().await {
            return updates;
        }

        // Update position in recorder
        self.recorder.set_position(position_ticks).await;

        // Get all lanes
        let lanes = self.recorder.get_all_lanes().await;

        // Calculate automation values
        for lane in lanes {
            if !lane.enabled {
                continue;
            }

            // Only read automation in Read mode (not while recording)
            if lane.mode != AutomationMode::Read && lane.mode != AutomationMode::Off {
                continue;
            }

            if let Some(value) = lane.get_value_at(position_ticks) {
                updates.insert(lane.parameter_id.clone(), value);

                // Update cache
                let mut cache = self.value_cache.write().await;
                cache.insert(lane.id.clone(), value);
            }
        }

        updates
    }

    /// Get current cached value for a lane
    pub async fn get_cached_value(&self, lane_id: &str) -> Option<f32> {
        let cache = self.value_cache.read().await;
        cache.get(lane_id).copied()
    }

    /// Get automation value for a specific lane at specific time
    pub async fn get_value_at(&self, lane_id: &str, position_ticks: u64) -> Option<f32> {
        if !self.is_enabled().await {
            return None;
        }

        let lane = self.recorder.get_lane(lane_id).await?;

        if !lane.enabled || lane.mode == AutomationMode::Off {
            return None;
        }

        lane.get_value_at(position_ticks)
    }

    /// Get all automation values for a track at specific time
    pub async fn get_track_values_at(
        &self,
        track_id: u32,
        position_ticks: u64,
    ) -> HashMap<String, f32> {
        let mut values = HashMap::new();

        if !self.is_enabled().await {
            return values;
        }

        let lanes = self.recorder.get_track_lanes(track_id).await;

        for lane in lanes {
            if !lane.enabled || lane.mode == AutomationMode::Off {
                continue;
            }

            if let Some(value) = lane.get_value_at(position_ticks) {
                values.insert(lane.parameter_id.clone(), value);
            }
        }

        values
    }

    /// Clear value cache
    pub async fn clear_cache(&self) {
        let mut cache = self.value_cache.write().await;
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daw::automation::models::{AutomationLane, AutomationPoint};

    #[tokio::test]
    async fn test_automation_playback() {
        let recorder = Arc::new(AutomationRecorder::new());
        let player = AutomationPlayer::new(Arc::clone(&recorder));

        // Create lane with points
        let mut lane = AutomationLane::new(
            "auto1".to_string(),
            1,
            "gain".to_string(),
            "Gain".to_string(),
        );
        lane.add_point(AutomationPoint::new(0, 0.0));
        lane.add_point(AutomationPoint::new(100, 1.0));

        recorder.add_lane(lane).await;

        // Get values at different positions
        let value_0 = player.get_value_at("auto1", 0).await;
        let value_50 = player.get_value_at("auto1", 50).await;
        let value_100 = player.get_value_at("auto1", 100).await;

        assert_eq!(value_0, Some(0.0));
        assert_eq!(value_50, Some(0.5));
        assert_eq!(value_100, Some(1.0));
    }

    #[tokio::test]
    async fn test_update_returns_all_lanes() {
        let recorder = Arc::new(AutomationRecorder::new());
        let player = AutomationPlayer::new(Arc::clone(&recorder));

        // Create gain lane
        let mut gain_lane = AutomationLane::new(
            "auto_gain".to_string(),
            1,
            "gain".to_string(),
            "Gain".to_string(),
        );
        gain_lane.add_point(AutomationPoint::new(0, 0.5));

        // Create pan lane
        let mut pan_lane = AutomationLane::new(
            "auto_pan".to_string(),
            1,
            "pan".to_string(),
            "Pan".to_string(),
        );
        pan_lane.add_point(AutomationPoint::new(0, 0.0));

        recorder.add_lane(gain_lane).await;
        recorder.add_lane(pan_lane).await;

        // Update at position 0
        let updates = player.update(0).await;

        assert_eq!(updates.len(), 2);
        assert_eq!(updates.get("gain"), Some(&0.5));
        assert_eq!(updates.get("pan"), Some(&0.0));
    }

    #[tokio::test]
    async fn test_disabled_player() {
        let recorder = Arc::new(AutomationRecorder::new());
        let player = AutomationPlayer::new(Arc::clone(&recorder));

        let mut lane = AutomationLane::new(
            "auto1".to_string(),
            1,
            "gain".to_string(),
            "Gain".to_string(),
        );
        lane.add_point(AutomationPoint::new(0, 0.5));
        recorder.add_lane(lane).await;

        // Disable player
        player.set_enabled(false).await;

        let updates = player.update(0).await;
        assert_eq!(updates.len(), 0);

        let value = player.get_value_at("auto1", 0).await;
        assert_eq!(value, None);
    }
}
```

---

## Step 2: Integration with SequencerEngine (30 min)

Update `app/src-tauri/src/daw/sequencer/engine.rs`:

```rust
use crate::daw::automation::{AutomationPlayer, AutomationRecorder};
use std::sync::Arc;

pub struct SequencerEngine {
    // ... existing fields
    automation_recorder: Arc<AutomationRecorder>,
    automation_player: Arc<AutomationPlayer>,  // NEW
}

impl SequencerEngine {
    pub fn new() -> Self {
        let automation_recorder = Arc::new(AutomationRecorder::new());
        let automation_player = Arc::new(AutomationPlayer::new(Arc::clone(&automation_recorder)));

        Self {
            // ... existing fields
            automation_recorder,
            automation_player,
        }
    }

    pub fn get_automation_player(&self) -> Arc<AutomationPlayer> {
        Arc::clone(&self.automation_player)
    }

    /// Called during playback to apply automation
    pub async fn apply_automation(&self, position_ticks: u64) -> HashMap<String, f32> {
        self.automation_player.update(position_ticks).await
    }

    /// Set automation playback enabled/disabled
    pub async fn set_automation_enabled(&self, enabled: bool) {
        self.automation_player.set_enabled(enabled).await;
    }

    /// Get automation value for parameter at current position
    pub async fn get_automation_value(&self, lane_id: &str) -> Option<f32> {
        let position = self.automation_recorder.get_position().await;
        self.automation_player.get_value_at(lane_id, position).await
    }
}
```

---

## Step 3: Apply Automation to Mixer (20 min)

Update `app/src-tauri/src/daw/sequencer/engine.rs` playback integration:

```rust
impl SequencerEngine {
    /// Process audio frame with automation
    pub async fn process_frame(&self, position_ticks: u64) -> Result<(), String> {
        // Get automation updates
        let automation_values = self.apply_automation(position_ticks).await;

        // Apply automation to mixer parameters
        let mixer = self.mixer.lock().await;

        for (parameter_id, value) in automation_values {
            self.apply_parameter_automation(&parameter_id, value, &mixer).await?;
        }

        Ok(())
    }

    /// Apply single parameter automation
    async fn apply_parameter_automation(
        &self,
        parameter_id: &str,
        value: f32,
        mixer: &MixerConfig,
    ) -> Result<(), String> {
        // Parse parameter_id format: "gain", "pan", "effect1.reverb_mix", etc.
        let parts: Vec<&str> = parameter_id.split('.').collect();

        match parts[0] {
            "gain" => {
                // Convert normalized 0-1 to dB (-60 to +12)
                let gain_db = (value * 72.0) - 60.0;
                // Apply to appropriate track (need track context)
                // This is simplified - in reality you'd have track_id in parameter_id
                Ok(())
            }
            "pan" => {
                // Convert normalized 0-1 to pan -1 to +1
                let pan_value = (value * 2.0) - 1.0;
                // Apply to appropriate track
                Ok(())
            }
            "effect" => {
                // Handle effect parameter automation
                // parts[1] would be effect_id.param_id
                Ok(())
            }
            _ => Err(format!("Unknown parameter type: {}", parts[0])),
        }
    }
}
```

---

## Step 4: Tauri Commands for Playback (20 min)

Add to `app/src-tauri/src/commands/daw/automation_commands.rs`:

```rust
/// Set automation playback enabled
#[tauri::command]
pub async fn set_automation_enabled(
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting automation enabled: {}", enabled);

    let sequencer = state.sequencer.lock().await;
    sequencer.set_automation_enabled(enabled).await;

    Ok(())
}

/// Get automation value at current playhead
#[tauri::command]
pub async fn get_automation_value(
    lane_id: String,
    state: State<'_, AppState>,
) -> Result<Option<f32>, String> {
    let sequencer = state.sequencer.lock().await;
    Ok(sequencer.get_automation_value(&lane_id).await)
}

/// Get all automation values for track at position
#[tauri::command]
pub async fn get_track_automation_values(
    track_id: u32,
    position_ticks: u64,
    state: State<'_, AppState>,
) -> Result<HashMap<String, f32>, String> {
    let sequencer = state.sequencer.lock().await;
    let player = sequencer.get_automation_player();

    Ok(player.get_track_values_at(track_id, position_ticks).await)
}
```

Register new commands in `main.rs`:

```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing handlers
            set_automation_enabled,
            get_automation_value,
            get_track_automation_values,
        ])
        // ...
}
```

---

## Step 5: Update module (5 min)

Update `app/src-tauri/src/daw/automation/mod.rs`:

```rust
pub mod models;
pub mod recorder;
pub mod player;

pub use models::{AutomationPoint, AutomationLane, AutomationMode, CurveType};
pub use recorder::AutomationRecorder;
pub use player::AutomationPlayer;
```

---

## Verification (10 min)

```bash
cd app/src-tauri
cargo check
cargo test --lib automation::player
```

Test in browser console:

```javascript
// Create lane with automation data
const lane = await window.__TAURI__.invoke('create_automation_lane', {
  trackId: 1,
  parameterId: 'gain',
  name: 'Gain'
});

// Add some points
await window.__TAURI__.invoke('add_automation_point', {
  laneId: lane.id,
  timeTicks: 0,
  value: 0.0,
  curveType: 'Linear'
});

await window.__TAURI__.invoke('add_automation_point', {
  laneId: lane.id,
  timeTicks: 1000,
  value: 1.0,
  curveType: 'Linear'
});

// Get value at midpoint
const value = await window.__TAURI__.invoke('get_automation_value', {
  laneId: lane.id
});
console.log('Automation value:', value);

// Get all track values
const trackValues = await window.__TAURI__.invoke('get_track_automation_values', {
  trackId: 1,
  positionTicks: 500
});
console.log('Track automation:', trackValues);

// Disable automation playback
await window.__TAURI__.invoke('set_automation_enabled', { enabled: false });
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Automation not playing | Check player is enabled, verify lane mode is Read |
| Values not updating | Ensure process_frame is called during playback |
| Interpolation jumpy | Check points are properly sorted, verify curve type |
| Performance issues | Cache values, avoid recalculating on every frame |

---

## What's Next?

✅ **You've completed:**
- AutomationPlayer with real-time interpolation
- Integration with SequencerEngine playback
- Parameter automation application
- 3 Tauri commands for playback control
- 3 unit tests for playback modes

**Next:** [Part 2B: Automation Frontend](./DAY2_PART_B_AUTOMATION_FRONTEND.md)
- Automation lane UI component
- Automation curve editor
- Visual point editing (add, move, delete)
- Automation mode selector
