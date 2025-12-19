# Day 1, Part 1A: Mixer Models & State

**Duration:** 3 hours
**Prerequisites:** Phase 1 complete
**Files to create:** 3

---

## Overview

Build the foundation for the DAW mixer system:
1. Create TrackState model for per-track mixer settings
2. Create MixerConfig for global mixer state
3. Implement MixerRepository for state persistence
4. Add mixer state to SequencerEngine

---

## Step 1: TrackState Model (45 min)

Create `app/src-tauri/src/daw/mixer/models/track_state.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Per-track mixer state
/// Represents all mixer settings for a single track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackState {
    pub track_id: u32,

    /// Gain in decibels (-∞ to +12 dB)
    /// 0 dB = unity gain (no change)
    /// -6 dB = half volume
    /// -∞ dB = silence
    pub gain_db: f32,

    /// Pan position (-1.0 to +1.0)
    /// -1.0 = hard left
    ///  0.0 = center
    /// +1.0 = hard right
    pub pan: f32,

    /// Mute state
    pub muted: bool,

    /// Solo state
    pub soloed: bool,

    /// Track enabled (for playback)
    pub enabled: bool,

    /// Track name
    pub name: String,

    /// Track color (hex RGB)
    pub color: Option<String>,
}

impl TrackState {
    /// Create default track state
    pub fn new(track_id: u32, name: String) -> Self {
        Self {
            track_id,
            gain_db: 0.0,      // Unity gain
            pan: 0.0,          // Center
            muted: false,
            soloed: false,
            enabled: true,
            name,
            color: None,
        }
    }

    /// Convert gain in dB to linear amplitude multiplier
    pub fn gain_linear(&self) -> f32 {
        if self.gain_db <= -100.0 {
            0.0 // -∞ dB = silence
        } else {
            10_f32.powf(self.gain_db / 20.0)
        }
    }

    /// Get left channel gain (considering pan)
    pub fn left_gain(&self) -> f32 {
        let base_gain = self.gain_linear();
        if self.muted {
            return 0.0;
        }

        // Equal power panning
        let pan_angle = (self.pan + 1.0) * 0.25 * std::f32::consts::PI;
        base_gain * pan_angle.cos()
    }

    /// Get right channel gain (considering pan)
    pub fn right_gain(&self) -> f32 {
        let base_gain = self.gain_linear();
        if self.muted {
            return 0.0;
        }

        // Equal power panning
        let pan_angle = (self.pan + 1.0) * 0.25 * std::f32::consts::PI;
        base_gain * pan_angle.sin()
    }

    /// Set gain from linear amplitude
    pub fn set_gain_linear(&mut self, linear: f32) {
        if linear <= 0.0 {
            self.gain_db = -100.0; // Silence
        } else {
            self.gain_db = 20.0 * linear.log10();
        }
    }

    /// Validate and clamp values to safe ranges
    pub fn validate(&mut self) {
        // Clamp gain to reasonable range
        self.gain_db = self.gain_db.clamp(-100.0, 12.0);

        // Clamp pan to -1.0 to +1.0
        self.pan = self.pan.clamp(-1.0, 1.0);
    }
}

impl Default for TrackState {
    fn default() -> Self {
        Self::new(0, "Track".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gain_conversions() {
        let mut track = TrackState::new(1, "Test".to_string());

        // 0 dB = unity gain
        track.gain_db = 0.0;
        assert!((track.gain_linear() - 1.0).abs() < 0.001);

        // -6 dB ≈ 0.5 amplitude
        track.gain_db = -6.0;
        assert!((track.gain_linear() - 0.501).abs() < 0.01);

        // -∞ dB = silence
        track.gain_db = -100.0;
        assert_eq!(track.gain_linear(), 0.0);
    }

    #[test]
    fn test_panning() {
        let mut track = TrackState::new(1, "Test".to_string());
        track.gain_db = 0.0; // Unity gain

        // Center pan
        track.pan = 0.0;
        let left = track.left_gain();
        let right = track.right_gain();
        assert!((left - right).abs() < 0.001); // Equal

        // Hard left
        track.pan = -1.0;
        assert!(track.left_gain() > track.right_gain());

        // Hard right
        track.pan = 1.0;
        assert!(track.right_gain() > track.left_gain());
    }

    #[test]
    fn test_mute() {
        let mut track = TrackState::new(1, "Test".to_string());
        track.gain_db = 0.0;

        track.muted = true;
        assert_eq!(track.left_gain(), 0.0);
        assert_eq!(track.right_gain(), 0.0);
    }
}
```

---

## Step 2: MixerConfig Model (30 min)

Create `app/src-tauri/src/daw/mixer/models/mixer_config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::TrackState;

/// Global mixer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerConfig {
    /// Master gain in dB
    pub master_gain_db: f32,

    /// Master mute
    pub master_muted: bool,

    /// Per-track mixer states
    pub track_states: HashMap<u32, TrackState>,

    /// Solo mode active (any track soloed)
    pub solo_active: bool,
}

impl MixerConfig {
    pub fn new() -> Self {
        Self {
            master_gain_db: 0.0,
            master_muted: false,
            track_states: HashMap::new(),
            solo_active: false,
        }
    }

    /// Get or create track state
    pub fn get_or_create_track(&mut self, track_id: u32, name: String) -> &mut TrackState {
        self.track_states
            .entry(track_id)
            .or_insert_with(|| TrackState::new(track_id, name))
    }

    /// Get track state (read-only)
    pub fn get_track(&self, track_id: u32) -> Option<&TrackState> {
        self.track_states.get(&track_id)
    }

    /// Update solo state across all tracks
    pub fn update_solo_state(&mut self) {
        self.solo_active = self.track_states.values().any(|t| t.soloed);
    }

    /// Check if a track should be audible
    /// Takes into account mute, solo, and enabled state
    pub fn is_track_audible(&self, track_id: u32) -> bool {
        if self.master_muted {
            return false;
        }

        if let Some(track) = self.get_track(track_id) {
            if track.muted || !track.enabled {
                return false;
            }

            // If any track is soloed, only soloed tracks are audible
            if self.solo_active {
                return track.soloed;
            }

            true
        } else {
            false
        }
    }

    /// Get effective gain for a track (master * track)
    pub fn get_effective_gain(&self, track_id: u32) -> (f32, f32) {
        if !self.is_track_audible(track_id) {
            return (0.0, 0.0);
        }

        let master_linear = 10_f32.powf(self.master_gain_db / 20.0);

        if let Some(track) = self.get_track(track_id) {
            let left = track.left_gain() * master_linear;
            let right = track.right_gain() * master_linear;
            (left, right)
        } else {
            (0.0, 0.0)
        }
    }

    /// Remove a track state
    pub fn remove_track(&mut self, track_id: u32) {
        self.track_states.remove(&track_id);
        self.update_solo_state();
    }

    /// Get all track IDs
    pub fn track_ids(&self) -> Vec<u32> {
        self.track_states.keys().copied().collect()
    }

    /// Validate all track states
    pub fn validate(&mut self) {
        for track in self.track_states.values_mut() {
            track.validate();
        }

        // Clamp master gain
        self.master_gain_db = self.master_gain_db.clamp(-100.0, 12.0);
    }
}

impl Default for MixerConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solo_mode() {
        let mut mixer = MixerConfig::new();

        mixer.get_or_create_track(1, "Track 1".to_string());
        mixer.get_or_create_track(2, "Track 2".to_string());

        // No solo - both audible
        assert!(mixer.is_track_audible(1));
        assert!(mixer.is_track_audible(2));

        // Solo track 1
        mixer.track_states.get_mut(&1).unwrap().soloed = true;
        mixer.update_solo_state();

        assert!(mixer.is_track_audible(1));
        assert!(!mixer.is_track_audible(2)); // Track 2 not soloed

        // Solo track 2 as well
        mixer.track_states.get_mut(&2).unwrap().soloed = true;
        mixer.update_solo_state();

        assert!(mixer.is_track_audible(1));
        assert!(mixer.is_track_audible(2)); // Both soloed
    }

    #[test]
    fn test_master_gain() {
        let mut mixer = MixerConfig::new();
        let track = mixer.get_or_create_track(1, "Track 1".to_string());
        track.gain_db = 0.0; // Unity

        mixer.master_gain_db = -6.0; // Half volume

        let (left, right) = mixer.get_effective_gain(1);
        assert!((left - 0.501).abs() < 0.01); // ~0.5
    }
}
```

Create `app/src-tauri/src/daw/mixer/models/mod.rs`:

```rust
pub mod mixer_config;
pub mod track_state;

pub use mixer_config::MixerConfig;
pub use track_state::TrackState;
```

---

## Step 3: Mixer Module Structure (15 min)

Create `app/src-tauri/src/daw/mixer/mod.rs`:

```rust
pub mod models;

pub use models::{MixerConfig, TrackState};
```

Update `app/src-tauri/src/daw/mod.rs`:

```rust
pub mod sequencer;
pub mod mixer;

pub use sequencer::SequencerEngine;
pub use mixer::{MixerConfig, TrackState};
```

---

## Step 4: Integrate Mixer into SequencerEngine (1 hour 30 min)

Update `app/src-tauri/src/daw/sequencer/engine.rs`:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::daw::mixer::MixerConfig;

pub struct SequencerEngine {
    // ... existing fields ...

    /// Mixer configuration
    mixer: Arc<Mutex<MixerConfig>>,
}

impl SequencerEngine {
    pub fn new() -> Self {
        Self {
            // ... existing fields ...
            mixer: Arc::new(Mutex::new(MixerConfig::new())),
        }
    }

    /// Get mixer configuration (for reading)
    pub async fn get_mixer(&self) -> MixerConfig {
        self.mixer.lock().await.clone()
    }

    /// Set track gain
    pub async fn set_track_gain(&self, track_id: u32, gain_db: f32) -> Result<(), String> {
        let mut mixer = self.mixer.lock().await;

        if let Some(track) = mixer.track_states.get_mut(&track_id) {
            track.gain_db = gain_db;
            track.validate();
            log::debug!("Set track {} gain to {} dB", track_id, gain_db);
            Ok(())
        } else {
            Err(format!("Track {} not found", track_id))
        }
    }

    /// Set track pan
    pub async fn set_track_pan(&self, track_id: u32, pan: f32) -> Result<(), String> {
        let mut mixer = self.mixer.lock().await;

        if let Some(track) = mixer.track_states.get_mut(&track_id) {
            track.pan = pan;
            track.validate();
            log::debug!("Set track {} pan to {}", track_id, pan);
            Ok(())
        } else {
            Err(format!("Track {} not found", track_id))
        }
    }

    /// Set track mute
    pub async fn set_track_mute(&self, track_id: u32, muted: bool) -> Result<(), String> {
        let mut mixer = self.mixer.lock().await;

        if let Some(track) = mixer.track_states.get_mut(&track_id) {
            track.muted = muted;
            log::debug!("Set track {} mute to {}", track_id, muted);
            Ok(())
        } else {
            Err(format!("Track {} not found", track_id))
        }
    }

    /// Set track solo
    pub async fn set_track_solo(&self, track_id: u32, soloed: bool) -> Result<(), String> {
        let mut mixer = self.mixer.lock().await;

        if let Some(track) = mixer.track_states.get_mut(&track_id) {
            track.soloed = soloed;
            mixer.update_solo_state();
            log::debug!("Set track {} solo to {}", track_id, soloed);
            Ok(())
        } else {
            Err(format!("Track {} not found", track_id))
        }
    }

    /// Set master gain
    pub async fn set_master_gain(&self, gain_db: f32) -> Result<(), String> {
        let mut mixer = self.mixer.lock().await;
        mixer.master_gain_db = gain_db.clamp(-100.0, 12.0);
        log::debug!("Set master gain to {} dB", mixer.master_gain_db);
        Ok(())
    }

    /// Set master mute
    pub async fn set_master_mute(&self, muted: bool) -> Result<(), String> {
        let mut mixer = self.mixer.lock().await;
        mixer.master_muted = muted;
        log::debug!("Set master mute to {}", muted);
        Ok(())
    }

    /// When adding a track, create mixer state
    pub async fn add_track(&mut self, midi_data: Vec<u8>, name: String) -> Result<u32, String> {
        // ... existing track creation code ...

        // Create mixer state for new track
        let track_id = /* get track id from creation */;
        let mut mixer = self.mixer.lock().await;
        mixer.get_or_create_track(track_id, name.clone());

        log::info!("Added track {} with mixer state", track_id);

        Ok(track_id)
    }

    /// When removing a track, clean up mixer state
    pub async fn remove_track(&mut self, track_id: u32) -> Result<(), String> {
        // ... existing track removal code ...

        // Remove mixer state
        let mut mixer = self.mixer.lock().await;
        mixer.remove_track(track_id);

        log::info!("Removed track {} and mixer state", track_id);

        Ok(())
    }

    // IMPORTANT: Update audio processing to apply mixer gains
    // This will be done in the audio callback
    // For now, just structure is in place
}
```

---

## Verification (30 min)

### 1. Compilation Check

```bash
cd app/src-tauri
cargo check
cargo test mixer
```

**Expected:** All tests pass, no compilation errors

### 2. Test Mixer Models

Run the unit tests:

```bash
cargo test daw::mixer --lib
```

**Expected output:**
```
running 3 tests
test daw::mixer::models::track_state::tests::test_gain_conversions ... ok
test daw::mixer::models::track_state::tests::test_panning ... ok
test daw::mixer::models::track_state::tests::test_mute ... ok
test daw::mixer::models::mixer_config::tests::test_solo_mode ... ok
test daw::mixer::models::mixer_config::tests::test_master_gain ... ok

test result: ok. 5 passed
```

### 3. Manual Test

Create a simple test in `main.rs` or integration test:

```rust
#[tokio::test]
async fn test_mixer_integration() {
    let mut engine = SequencerEngine::new();

    // Add a track (you'll need actual MIDI data)
    let track_id = 1; // Assume track exists

    // Test gain
    engine.set_track_gain(track_id, -6.0).await.unwrap();

    // Test pan
    engine.set_track_pan(track_id, 0.5).await.unwrap(); // Right

    // Test mute
    engine.set_track_mute(track_id, true).await.unwrap();

    // Get mixer state
    let mixer = engine.get_mixer().await;
    let track = mixer.get_track(track_id).unwrap();

    assert_eq!(track.gain_db, -6.0);
    assert_eq!(track.pan, 0.5);
    assert!(track.muted);
}
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "MixerConfig not found" | Check `mod.rs` exports in mixer module |
| Gain calculations wrong | Verify dB to linear conversion: `10^(dB/20)` |
| Pan doesn't sound equal power | Check pan angle calculation uses π/4 |
| Tests fail on floating point | Use `.abs() < 0.001` for float comparisons |
| SequencerEngine compilation errors | May need to update existing add_track/remove_track signatures |

---

## What's Next?

✅ **You've completed:**
- TrackState model with gain/pan/mute/solo
- MixerConfig for global state management
- Integration with SequencerEngine
- Unit tests for mixer logic

**Next:** [Part 1B: Core Mixer Commands](./DAY1_PART_B_MIXER_COMMANDS.md)
- Create Tauri commands for all mixer operations
- Register commands in main.rs
- Test from frontend

**Note:** Audio won't be affected yet - that requires audio callback integration in Part 1B.
