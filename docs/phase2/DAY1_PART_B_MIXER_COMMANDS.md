# Day 1, Part 1B: Core Mixer Commands

**Duration:** 2 hours
**Prerequisites:** Part 1A complete
**Files to create/modify:** 2

---

## Overview

Create Tauri commands for all mixer operations:
1. Gain, pan, mute, solo commands for tracks
2. Master gain and mute commands
3. Get mixer state command
4. Register all commands

---

## Step 1: Mixer Commands (1 hour 15 min)

Create `app/src-tauri/src/commands/daw/mixer_commands.rs`:

```rust
use crate::daw::{MixerConfig, TrackState};
use crate::AppState;
use tauri::State;

// ============================================================================
// TRACK MIXER COMMANDS
// ============================================================================

/// Set track gain in decibels
#[tauri::command]
pub async fn set_track_gain(
    track_id: u32,
    gain_db: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting track {} gain to {} dB", track_id, gain_db);

    let sequencer = state.sequencer.lock().await;
    sequencer.set_track_gain(track_id, gain_db).await
}

/// Set track pan (-1.0 to +1.0)
#[tauri::command]
pub async fn set_track_pan(
    track_id: u32,
    pan: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting track {} pan to {}", track_id, pan);

    let sequencer = state.sequencer.lock().await;
    sequencer.set_track_pan(track_id, pan).await
}

/// Set track mute state
#[tauri::command]
pub async fn set_track_mute(
    track_id: u32,
    muted: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting track {} mute to {}", track_id, muted);

    let sequencer = state.sequencer.lock().await;
    sequencer.set_track_mute(track_id, muted).await
}

/// Toggle track mute
#[tauri::command]
pub async fn toggle_track_mute(
    track_id: u32,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let sequencer = state.sequencer.lock().await;
    let mixer = sequencer.get_mixer().await;

    let new_state = if let Some(track) = mixer.get_track(track_id) {
        !track.muted
    } else {
        return Err(format!("Track {} not found", track_id));
    };

    sequencer.set_track_mute(track_id, new_state).await?;

    log::info!("Toggled track {} mute to {}", track_id, new_state);

    Ok(new_state)
}

/// Set track solo state
#[tauri::command]
pub async fn set_track_solo(
    track_id: u32,
    soloed: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting track {} solo to {}", track_id, soloed);

    let sequencer = state.sequencer.lock().await;
    sequencer.set_track_solo(track_id, soloed).await
}

/// Toggle track solo
#[tauri::command]
pub async fn toggle_track_solo(
    track_id: u32,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let sequencer = state.sequencer.lock().await;
    let mixer = sequencer.get_mixer().await;

    let new_state = if let Some(track) = mixer.get_track(track_id) {
        !track.soloed
    } else {
        return Err(format!("Track {} not found", track_id));
    };

    sequencer.set_track_solo(track_id, new_state).await?;

    log::info!("Toggled track {} solo to {}", track_id, new_state);

    Ok(new_state)
}

/// Set track name
#[tauri::command]
pub async fn set_track_name(
    track_id: u32,
    name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting track {} name to '{}'", track_id, name);

    let sequencer = state.sequencer.lock().await;
    let mut mixer = sequencer.get_mixer().await;

    if let Some(track) = mixer.track_states.get_mut(&track_id) {
        track.name = name;
        Ok(())
    } else {
        Err(format!("Track {} not found", track_id))
    }
}

/// Set track color
#[tauri::command]
pub async fn set_track_color(
    track_id: u32,
    color: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting track {} color to '{}'", track_id, color);

    let sequencer = state.sequencer.lock().await;
    let mut mixer = sequencer.get_mixer().await;

    if let Some(track) = mixer.track_states.get_mut(&track_id) {
        track.color = Some(color);
        Ok(())
    } else {
        Err(format!("Track {} not found", track_id))
    }
}

/// Get track state
#[tauri::command]
pub async fn get_track_state(
    track_id: u32,
    state: State<'_, AppState>,
) -> Result<TrackState, String> {
    let sequencer = state.sequencer.lock().await;
    let mixer = sequencer.get_mixer().await;

    mixer
        .get_track(track_id)
        .cloned()
        .ok_or_else(|| format!("Track {} not found", track_id))
}

// ============================================================================
// MASTER MIXER COMMANDS
// ============================================================================

/// Set master gain in decibels
#[tauri::command]
pub async fn set_master_gain(
    gain_db: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting master gain to {} dB", gain_db);

    let sequencer = state.sequencer.lock().await;
    sequencer.set_master_gain(gain_db).await
}

/// Set master mute state
#[tauri::command]
pub async fn set_master_mute(
    muted: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting master mute to {}", muted);

    let sequencer = state.sequencer.lock().await;
    sequencer.set_master_mute(muted).await
}

/// Toggle master mute
#[tauri::command]
pub async fn toggle_master_mute(state: State<'_, AppState>) -> Result<bool, String> {
    let sequencer = state.sequencer.lock().await;
    let mixer = sequencer.get_mixer().await;

    let new_state = !mixer.master_muted;

    sequencer.set_master_mute(new_state).await?;

    log::info!("Toggled master mute to {}", new_state);

    Ok(new_state)
}

// ============================================================================
// MIXER STATE COMMANDS
// ============================================================================

/// Get complete mixer configuration
#[tauri::command]
pub async fn get_mixer_config(state: State<'_, AppState>) -> Result<MixerConfig, String> {
    let sequencer = state.sequencer.lock().await;
    Ok(sequencer.get_mixer().await)
}

/// Get all track states
#[tauri::command]
pub async fn get_all_track_states(
    state: State<'_, AppState>,
) -> Result<Vec<TrackState>, String> {
    let sequencer = state.sequencer.lock().await;
    let mixer = sequencer.get_mixer().await;

    let mut tracks: Vec<TrackState> = mixer.track_states.values().cloned().collect();
    tracks.sort_by_key(|t| t.track_id);

    Ok(tracks)
}

/// Reset all mixer settings to defaults
#[tauri::command]
pub async fn reset_mixer(state: State<'_, AppState>) -> Result<(), String> {
    log::warn!("Resetting entire mixer to defaults");

    let sequencer = state.sequencer.lock().await;

    // Reset master
    sequencer.set_master_gain(0.0).await?;
    sequencer.set_master_mute(false).await?;

    // Reset all tracks
    let mixer = sequencer.get_mixer().await;
    for track_id in mixer.track_ids() {
        sequencer.set_track_gain(track_id, 0.0).await?;
        sequencer.set_track_pan(track_id, 0.0).await?;
        sequencer.set_track_mute(track_id, false).await?;
        sequencer.set_track_solo(track_id, false).await?;
    }

    log::info!("Mixer reset complete");

    Ok(())
}

// ============================================================================
// BATCH OPERATIONS
// ============================================================================

/// Set gain for multiple tracks at once
#[tauri::command]
pub async fn set_tracks_gain(
    track_ids: Vec<u32>,
    gain_db: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting {} tracks to {} dB", track_ids.len(), gain_db);

    let sequencer = state.sequencer.lock().await;

    for track_id in track_ids {
        sequencer.set_track_gain(track_id, gain_db).await?;
    }

    Ok(())
}

/// Mute all tracks
#[tauri::command]
pub async fn mute_all_tracks(state: State<'_, AppState>) -> Result<(), String> {
    log::info!("Muting all tracks");

    let sequencer = state.sequencer.lock().await;
    let mixer = sequencer.get_mixer().await;

    for track_id in mixer.track_ids() {
        sequencer.set_track_mute(track_id, true).await?;
    }

    Ok(())
}

/// Unmute all tracks
#[tauri::command]
pub async fn unmute_all_tracks(state: State<'_, AppState>) -> Result<(), String> {
    log::info!("Unmuting all tracks");

    let sequencer = state.sequencer.lock().await;
    let mixer = sequencer.get_mixer().await;

    for track_id in mixer.track_ids() {
        sequencer.set_track_mute(track_id, false).await?;
    }

    Ok(())
}

/// Clear all solos
#[tauri::command]
pub async fn clear_all_solos(state: State<'_, AppState>) -> Result<(), String> {
    log::info!("Clearing all solos");

    let sequencer = state.sequencer.lock().await;
    let mixer = sequencer.get_mixer().await;

    for track_id in mixer.track_ids() {
        sequencer.set_track_solo(track_id, false).await?;
    }

    Ok(())
}
```

---

## Step 2: Register Commands (15 min)

Update `app/src-tauri/src/commands/daw/mod.rs`:

```rust
pub mod mixer_commands;

pub use mixer_commands::*;
```

Update `app/src-tauri/src/main.rs`:

```rust
use midi_app::commands::daw::{
    // Track mixer
    set_track_gain,
    set_track_pan,
    set_track_mute,
    toggle_track_mute,
    set_track_solo,
    toggle_track_solo,
    set_track_name,
    set_track_color,
    get_track_state,

    // Master mixer
    set_master_gain,
    set_master_mute,
    toggle_master_mute,

    // Mixer state
    get_mixer_config,
    get_all_track_states,
    reset_mixer,

    // Batch operations
    set_tracks_gain,
    mute_all_tracks,
    unmute_all_tracks,
    clear_all_solos,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...

            // Mixer commands
            set_track_gain,
            set_track_pan,
            set_track_mute,
            toggle_track_mute,
            set_track_solo,
            toggle_track_solo,
            set_track_name,
            set_track_color,
            get_track_state,
            set_master_gain,
            set_master_mute,
            toggle_master_mute,
            get_mixer_config,
            get_all_track_states,
            reset_mixer,
            set_tracks_gain,
            mute_all_tracks,
            unmute_all_tracks,
            clear_all_solos,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Step 3: Audio Integration (30 min)

**IMPORTANT:** The commands are now registered, but audio won't be affected until you update the audio callback in `SequencerEngine` to apply mixer gains.

This pseudocode shows the approach (actual implementation depends on your audio backend):

```rust
// In SequencerEngine audio callback
fn audio_callback(&mut self, output_buffer: &mut [f32]) {
    let mixer = self.mixer.lock().unwrap(); // Careful with locks in audio thread!

    for track_id in self.active_tracks() {
        if !mixer.is_track_audible(track_id) {
            continue; // Skip muted/not-soloed tracks
        }

        let (left_gain, right_gain) = mixer.get_effective_gain(track_id);

        // Get track audio samples
        let track_samples = self.get_track_samples(track_id);

        // Apply gains and mix into output
        for (i, sample) in track_samples.iter().enumerate() {
            output_buffer[i * 2] += sample * left_gain;     // Left
            output_buffer[i * 2 + 1] += sample * right_gain; // Right
        }
    }
}
```

**Note:** You'll implement full audio integration in Part 2A (Metering) where we add proper audio processing pipeline.

---

## Verification (15 min)

### 1. Compilation Check

```bash
cd app/src-tauri
cargo check
```

**Expected:** No errors, 17 new commands registered

### 2. Test Commands

```bash
make dev
```

In browser console:

```javascript
// Set gain
await window.__TAURI__.invoke('set_track_gain', {
  trackId: 1,
  gainDb: -6.0
});

// Set pan
await window.__TAURI__.invoke('set_track_pan', {
  trackId: 1,
  pan: 0.5 // Right
});

// Toggle mute
const muted = await window.__TAURI__.invoke('toggle_track_mute', {
  trackId: 1
});
console.log('Muted:', muted);

// Get mixer state
const mixer = await window.__TAURI__.invoke('get_mixer_config');
console.log('Mixer:', mixer);

// Get all tracks
const tracks = await window.__TAURI__.invoke('get_all_track_states');
console.log('Tracks:', tracks);
```

### 3. Verify State Changes

```javascript
// Before
const before = await window.__TAURI__.invoke('get_track_state', { trackId: 1 });
console.log('Before:', before);

// Change
await window.__TAURI__.invoke('set_track_gain', { trackId: 1, gainDb: -12.0 });

// After
const after = await window.__TAURI__.invoke('get_track_state', { trackId: 1 });
console.log('After:', after);

// Verify
console.log('Gain changed:', before.gain_db, '->', after.gain_db);
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Command not found" | Verify registration in `main.rs` invoke_handler |
| State not persisting | MixerConfig is in-memory only, add persistence later |
| Audio not affected | Expected - audio integration in Part 2A |
| Track not found errors | Ensure track exists in SequencerEngine first |

---

## What's Next?

✅ **You've completed:**
- 17 mixer commands (gain, pan, mute, solo, master, batch)
- Command registration
- State management through SequencerEngine

**Next:** [Part 1C: Frontend Mixer API](./DAY1_PART_C_MIXER_FRONTEND.md)
- TypeScript types for mixer
- Mixer API client
- Reactive Svelte stores
- Real-time state updates

**Note:** Audio will be audibly affected after Part 2A (Metering) when we add audio callback integration.
