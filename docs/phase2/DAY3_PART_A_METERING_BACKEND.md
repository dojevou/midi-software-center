# Day 3, Part 2A: Metering Backend

**Duration:** 2 hours
**Prerequisites:** Day 1 complete (mixer commands working)
**Files to create/modify:** 2

---

## Overview

Add real-time VU metering system:
1. Peak and RMS level detection in audio callback
2. Metering state management
3. Event streaming to frontend
4. Per-track and master metering

---

## Step 1: Metering Models (30 min)

Create `app/src-tauri/src/daw/mixer/metering.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Audio level metering for a single channel (track or master)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterLevel {
    /// Peak level in decibels
    pub peak_db: f32,

    /// RMS (average) level in decibels
    pub rms_db: f32,

    /// Peak hold value (for visual feedback)
    pub peak_hold_db: f32,

    /// Number of samples since last peak hold reset
    pub peak_hold_samples: u32,
}

impl Default for MeterLevel {
    fn default() -> Self {
        Self {
            peak_db: -100.0,
            rms_db: -100.0,
            peak_hold_db: -100.0,
            peak_hold_samples: 0,
        }
    }
}

impl MeterLevel {
    /// Create new meter level
    pub fn new() -> Self {
        Self::default()
    }

    /// Update meter with audio buffer
    /// `samples` - audio samples (mono or stereo interleaved)
    /// `sample_rate` - audio sample rate
    pub fn update(&mut self, samples: &[f32], sample_rate: u32) {
        if samples.is_empty() {
            return;
        }

        // Calculate peak (max absolute value)
        let peak_linear = samples
            .iter()
            .map(|s| s.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        // Calculate RMS (root mean square)
        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        let rms_linear = (sum_squares / samples.len() as f32).sqrt();

        // Convert to dB
        self.peak_db = Self::linear_to_db(peak_linear);
        self.rms_db = Self::linear_to_db(rms_linear);

        // Update peak hold (decays over time)
        if self.peak_db > self.peak_hold_db {
            self.peak_hold_db = self.peak_db;
            self.peak_hold_samples = 0;
        } else {
            self.peak_hold_samples += samples.len() as u32;

            // Decay after 2 seconds
            let hold_duration_samples = sample_rate * 2;
            if self.peak_hold_samples > hold_duration_samples {
                self.peak_hold_db = self.peak_db;
                self.peak_hold_samples = 0;
            }
        }
    }

    /// Convert linear amplitude to decibels
    fn linear_to_db(linear: f32) -> f32 {
        if linear <= 0.0001 {
            -100.0
        } else {
            20.0 * linear.log10()
        }
    }

    /// Check if signal is clipping (> 0 dB)
    pub fn is_clipping(&self) -> bool {
        self.peak_db > 0.0
    }

    /// Check if signal is near clipping (> -3 dB)
    pub fn is_near_clipping(&self) -> bool {
        self.peak_db > -3.0
    }

    /// Reset meter to silence
    pub fn reset(&mut self) {
        self.peak_db = -100.0;
        self.rms_db = -100.0;
        self.peak_hold_db = -100.0;
        self.peak_hold_samples = 0;
    }
}

/// Metering configuration for entire mixer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteringConfig {
    /// Per-track meters
    pub track_meters: HashMap<u32, MeterLevel>,

    /// Master output meter
    pub master_meter: MeterLevel,

    /// Sample rate (needed for time-based calculations)
    pub sample_rate: u32,

    /// Metering enabled flag
    pub enabled: bool,
}

impl Default for MeteringConfig {
    fn default() -> Self {
        Self {
            track_meters: HashMap::new(),
            master_meter: MeterLevel::new(),
            sample_rate: 48000,
            enabled: true,
        }
    }
}

impl MeteringConfig {
    /// Create new metering config
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            ..Default::default()
        }
    }

    /// Get or create meter for track
    pub fn get_or_create_track_meter(&mut self, track_id: u32) -> &mut MeterLevel {
        self.track_meters
            .entry(track_id)
            .or_insert_with(MeterLevel::new)
    }

    /// Get track meter (immutable)
    pub fn get_track_meter(&self, track_id: u32) -> Option<&MeterLevel> {
        self.track_meters.get(&track_id)
    }

    /// Update track meter with audio samples
    pub fn update_track(&mut self, track_id: u32, samples: &[f32]) {
        if !self.enabled {
            return;
        }

        let meter = self.get_or_create_track_meter(track_id);
        meter.update(samples, self.sample_rate);
    }

    /// Update master meter with audio samples
    pub fn update_master(&mut self, samples: &[f32]) {
        if !self.enabled {
            return;
        }

        self.master_meter.update(samples, self.sample_rate);
    }

    /// Reset all meters to silence
    pub fn reset_all(&mut self) {
        for meter in self.track_meters.values_mut() {
            meter.reset();
        }
        self.master_meter.reset();
    }

    /// Check if any track is clipping
    pub fn any_clipping(&self) -> bool {
        self.master_meter.is_clipping()
            || self
                .track_meters
                .values()
                .any(|m| m.is_clipping())
    }

    /// Get all clipping track IDs
    pub fn get_clipping_tracks(&self) -> Vec<u32> {
        self.track_meters
            .iter()
            .filter(|(_, meter)| meter.is_clipping())
            .map(|(id, _)| *id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meter_level_silence() {
        let mut meter = MeterLevel::new();
        meter.update(&vec![0.0; 1024], 48000);

        assert!(meter.peak_db < -90.0, "Silent signal should have very low peak");
        assert!(meter.rms_db < -90.0, "Silent signal should have very low RMS");
    }

    #[test]
    fn test_meter_level_full_scale() {
        let mut meter = MeterLevel::new();
        meter.update(&vec![1.0; 1024], 48000);

        assert!(
            (meter.peak_db - 0.0).abs() < 0.1,
            "Full scale signal should be 0 dB"
        );
        assert!(meter.is_clipping());
    }

    #[test]
    fn test_peak_hold() {
        let mut meter = MeterLevel::new();

        // Loud signal
        meter.update(&vec![1.0; 1024], 48000);
        let peak_hold = meter.peak_hold_db;

        // Silent signal
        meter.update(&vec![0.0; 1024], 48000);

        // Peak hold should still be near 0 dB
        assert!(
            (meter.peak_hold_db - peak_hold).abs() < 0.1,
            "Peak hold should persist"
        );
    }

    #[test]
    fn test_metering_config() {
        let mut config = MeteringConfig::new(48000);

        config.update_track(1, &vec![0.5; 512]);
        config.update_master(&vec![0.3; 512]);

        assert!(config.get_track_meter(1).is_some());
        assert!(config.master_meter.peak_db > -100.0);
    }
}
```

---

## Step 2: Integrate with SequencerEngine (45 min)

Update `app/src-tauri/src/daw/sequencer/engine.rs`:

```rust
use crate::daw::mixer::metering::{MeteringConfig, MeterLevel};

pub struct SequencerEngine {
    mixer: Arc<Mutex<MixerConfig>>,
    metering: Arc<Mutex<MeteringConfig>>, // NEW
    // ... other fields
}

impl SequencerEngine {
    pub fn new() -> Self {
        Self {
            mixer: Arc::new(Mutex::new(MixerConfig::default())),
            metering: Arc::new(Mutex::new(MeteringConfig::new(48000))), // NEW
            // ... other fields
        }
    }

    /// Get metering configuration
    pub async fn get_metering(&self) -> MeteringConfig {
        self.metering.lock().await.clone()
    }

    /// Enable/disable metering
    pub async fn set_metering_enabled(&self, enabled: bool) -> Result<(), String> {
        let mut metering = self.metering.lock().await;
        metering.enabled = enabled;
        Ok(())
    }

    /// Reset all meters
    pub async fn reset_meters(&self) -> Result<(), String> {
        let mut metering = self.metering.lock().await;
        metering.reset_all();
        Ok(())
    }

    // This will be called from audio callback
    fn process_audio_with_metering(&mut self, output_buffer: &mut [f32]) {
        let mixer = self.mixer.blocking_lock();
        let mut metering = self.metering.blocking_lock();

        // Temporary buffers for per-track audio
        let mut track_buffers: HashMap<u32, Vec<f32>> = HashMap::new();

        // Process each active track
        for track_id in self.get_active_track_ids() {
            if !mixer.is_track_audible(track_id) {
                continue;
            }

            // Get track audio samples
            let track_samples = self.get_track_audio(track_id);

            // Apply mixer gains
            let (left_gain, right_gain) = mixer.get_effective_gain(track_id);

            let mut processed_samples = Vec::new();

            // Mix into output buffer and collect for metering
            for (i, &sample) in track_samples.iter().enumerate() {
                let stereo_idx = i * 2;
                if stereo_idx + 1 < output_buffer.len() {
                    let left = sample * left_gain;
                    let right = sample * right_gain;

                    output_buffer[stereo_idx] += left;
                    output_buffer[stereo_idx + 1] += right;

                    // Store for metering (mono sum)
                    processed_samples.push((left + right) * 0.5);
                }
            }

            // Update track meter
            metering.update_track(track_id, &processed_samples);
            track_buffers.insert(track_id, processed_samples);
        }

        // Update master meter
        metering.update_master(output_buffer);
    }
}
```

---

## Step 3: Tauri Commands (30 min)

Create `app/src-tauri/src/commands/daw/metering_commands.rs`:

```rust
use crate::daw::mixer::metering::{MeteringConfig, MeterLevel};
use crate::AppState;
use tauri::State;

/// Get complete metering configuration
#[tauri::command]
pub async fn get_metering_config(state: State<'_, AppState>) -> Result<MeteringConfig, String> {
    let sequencer = state.sequencer.lock().await;
    Ok(sequencer.get_metering().await)
}

/// Get meter level for specific track
#[tauri::command]
pub async fn get_track_meter(
    track_id: u32,
    state: State<'_, AppState>,
) -> Result<Option<MeterLevel>, String> {
    let sequencer = state.sequencer.lock().await;
    let metering = sequencer.get_metering().await;
    Ok(metering.get_track_meter(track_id).cloned())
}

/// Get master meter level
#[tauri::command]
pub async fn get_master_meter(state: State<'_, AppState>) -> Result<MeterLevel, String> {
    let sequencer = state.sequencer.lock().await;
    let metering = sequencer.get_metering().await;
    Ok(metering.master_meter)
}

/// Enable/disable metering
#[tauri::command]
pub async fn set_metering_enabled(
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let sequencer = state.sequencer.lock().await;
    sequencer.set_metering_enabled(enabled).await
}

/// Reset all meters to silence
#[tauri::command]
pub async fn reset_meters(state: State<'_, AppState>) -> Result<(), String> {
    let sequencer = state.sequencer.lock().await;
    sequencer.reset_meters().await
}

/// Check if any track is clipping
#[tauri::command]
pub async fn check_clipping(state: State<'_, AppState>) -> Result<bool, String> {
    let sequencer = state.sequencer.lock().await;
    let metering = sequencer.get_metering().await;
    Ok(metering.any_clipping())
}

/// Get all clipping track IDs
#[tauri::command]
pub async fn get_clipping_tracks(state: State<'_, AppState>) -> Result<Vec<u32>, String> {
    let sequencer = state.sequencer.lock().await;
    let metering = sequencer.get_metering().await;
    Ok(metering.get_clipping_tracks())
}
```

Update `app/src-tauri/src/commands/daw/mod.rs`:

```rust
pub mod metering_commands;
pub use metering_commands::*;
```

Register in `main.rs`:

```rust
use midi_app::commands::daw::{
    // ... existing mixer commands ...
    get_metering_config,
    get_track_meter,
    get_master_meter,
    set_metering_enabled,
    reset_meters,
    check_clipping,
    get_clipping_tracks,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing commands ...
            get_metering_config,
            get_track_meter,
            get_master_meter,
            set_metering_enabled,
            reset_meters,
            check_clipping,
            get_clipping_tracks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## Step 4: Event Streaming (15 min)

Add event emission to `SequencerEngine` for real-time updates:

```rust
use tauri::Emitter;

impl SequencerEngine {
    /// Emit metering update events (call this ~60fps from audio thread)
    pub fn emit_metering_events(&self, app_handle: &tauri::AppHandle) {
        if let Ok(metering) = self.metering.try_lock() {
            // Emit to frontend
            let _ = app_handle.emit("metering-update", &*metering);
        }
    }
}
```

Set up timer in `main.rs` to emit events:

```rust
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {
    let app = tauri::Builder::default()
        // ... setup ...
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    let app_handle = app.handle().clone();
    let state = app.state::<AppState>();

    // Spawn metering update task (60 fps = ~16ms)
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(16));

        loop {
            interval.tick().await;

            let sequencer = state.sequencer.lock().await;
            sequencer.emit_metering_events(&app_handle);
        }
    });

    app.run(|_, _| {});
}
```

---

## Verification (10 min)

```bash
cd app/src-tauri
cargo check
cargo test --lib metering
```

Test commands in browser console:

```javascript
// Get metering config
const metering = await window.__TAURI__.invoke('get_metering_config');
console.log('Metering:', metering);

// Get track meter
const trackMeter = await window.__TAURI__.invoke('get_track_meter', { trackId: 1 });
console.log('Track meter:', trackMeter);

// Get master meter
const masterMeter = await window.__TAURI__.invoke('get_master_meter');
console.log('Master meter:', masterMeter);

// Listen for metering updates
await window.__TAURI__.event.listen('metering-update', (event) => {
  console.log('Metering update:', event.payload);
});
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| No metering events | Check tokio task is running, verify emit() calls |
| Meters always -100 dB | Ensure audio is playing, check audio callback integration |
| High CPU usage | Reduce event emission rate from 60fps to 30fps |
| Meters lag behind audio | Check buffer sizes, ensure non-blocking locks in audio thread |

---

## What's Next?

✅ **You've completed:**
- Peak and RMS level detection
- Per-track and master metering
- Event streaming to frontend (60fps)
- Clipping detection

**Next:** [Part 2B: Metering Frontend](./DAY3_PART_B_METERING_FRONTEND.md)
- Real-time VU meter components
- Event listeners and state management
- Visual feedback for clipping
- Peak hold indicators
