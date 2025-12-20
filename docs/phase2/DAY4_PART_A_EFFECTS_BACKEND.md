# Day 4, Part 3A: Effects Backend

**Duration:** 2.5 hours
**Prerequisites:** Days 1-3 complete
**Files to create:** 3

---

## Overview

Build foundation for effect chain system:
1. Effect models (gain, EQ, reverb placeholders)
2. Effect chain management
3. Effect parameter system
4. Integration with mixer

**Note:** This creates the foundation. Full VST/LV2 integration comes later.

---

## Step 1: Effect Models (45 min)

Create `app/src-tauri/src/daw/mixer/effects/models.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Effect parameter value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ParameterValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
}

/// Effect parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectParameter {
    pub id: String,
    pub name: String,
    pub value: ParameterValue,
    pub default: ParameterValue,
    pub min: Option<ParameterValue>,
    pub max: Option<ParameterValue>,
    pub unit: Option<String>,
}

impl EffectParameter {
    pub fn float(id: &str, name: &str, value: f32, min: f32, max: f32, unit: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            value: ParameterValue::Float(value),
            default: ParameterValue::Float(value),
            min: Some(ParameterValue::Float(min)),
            max: Some(ParameterValue::Float(max)),
            unit: Some(unit.to_string()),
        }
    }

    pub fn bool(id: &str, name: &str, value: bool) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            value: ParameterValue::Bool(value),
            default: ParameterValue::Bool(value),
            min: None,
            max: None,
            unit: None,
        }
    }
}

/// Effect types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffectType {
    Gain,
    EQ,
    Reverb,
    Delay,
    Compressor,
    Limiter,
    Custom(String),
}

/// Effect instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub id: String,
    pub effect_type: EffectType,
    pub name: String,
    pub enabled: bool,
    pub parameters: HashMap<String, EffectParameter>,
}

impl Effect {
    /// Create new effect
    pub fn new(id: String, effect_type: EffectType, name: String) -> Self {
        let parameters = Self::default_parameters(&effect_type);

        Self {
            id,
            effect_type,
            name,
            enabled: true,
            parameters,
        }
    }

    /// Get default parameters for effect type
    fn default_parameters(effect_type: &EffectType) -> HashMap<String, EffectParameter> {
        let mut params = HashMap::new();

        match effect_type {
            EffectType::Gain => {
                params.insert(
                    "gain".to_string(),
                    EffectParameter::float("gain", "Gain", 0.0, -60.0, 12.0, "dB"),
                );
            }
            EffectType::EQ => {
                params.insert(
                    "low_freq".to_string(),
                    EffectParameter::float("low_freq", "Low Freq", 100.0, 20.0, 500.0, "Hz"),
                );
                params.insert(
                    "low_gain".to_string(),
                    EffectParameter::float("low_gain", "Low Gain", 0.0, -12.0, 12.0, "dB"),
                );
                params.insert(
                    "mid_freq".to_string(),
                    EffectParameter::float("mid_freq", "Mid Freq", 1000.0, 200.0, 5000.0, "Hz"),
                );
                params.insert(
                    "mid_gain".to_string(),
                    EffectParameter::float("mid_gain", "Mid Gain", 0.0, -12.0, 12.0, "dB"),
                );
                params.insert(
                    "high_freq".to_string(),
                    EffectParameter::float("high_freq", "High Freq", 8000.0, 2000.0, 20000.0, "Hz"),
                );
                params.insert(
                    "high_gain".to_string(),
                    EffectParameter::float("high_gain", "High Gain", 0.0, -12.0, 12.0, "dB"),
                );
            }
            EffectType::Reverb => {
                params.insert(
                    "room_size".to_string(),
                    EffectParameter::float("room_size", "Room Size", 0.5, 0.0, 1.0, ""),
                );
                params.insert(
                    "damping".to_string(),
                    EffectParameter::float("damping", "Damping", 0.5, 0.0, 1.0, ""),
                );
                params.insert(
                    "wet".to_string(),
                    EffectParameter::float("wet", "Wet", 0.3, 0.0, 1.0, ""),
                );
                params.insert(
                    "dry".to_string(),
                    EffectParameter::float("dry", "Dry", 0.7, 0.0, 1.0, ""),
                );
            }
            EffectType::Delay => {
                params.insert(
                    "time".to_string(),
                    EffectParameter::float("time", "Delay Time", 0.5, 0.001, 2.0, "s"),
                );
                params.insert(
                    "feedback".to_string(),
                    EffectParameter::float("feedback", "Feedback", 0.3, 0.0, 0.95, ""),
                );
                params.insert(
                    "mix".to_string(),
                    EffectParameter::float("mix", "Mix", 0.3, 0.0, 1.0, ""),
                );
            }
            EffectType::Compressor => {
                params.insert(
                    "threshold".to_string(),
                    EffectParameter::float("threshold", "Threshold", -20.0, -60.0, 0.0, "dB"),
                );
                params.insert(
                    "ratio".to_string(),
                    EffectParameter::float("ratio", "Ratio", 4.0, 1.0, 20.0, ":1"),
                );
                params.insert(
                    "attack".to_string(),
                    EffectParameter::float("attack", "Attack", 5.0, 0.1, 100.0, "ms"),
                );
                params.insert(
                    "release".to_string(),
                    EffectParameter::float("release", "Release", 50.0, 10.0, 1000.0, "ms"),
                );
            }
            EffectType::Limiter => {
                params.insert(
                    "threshold".to_string(),
                    EffectParameter::float("threshold", "Threshold", -3.0, -20.0, 0.0, "dB"),
                );
                params.insert(
                    "release".to_string(),
                    EffectParameter::float("release", "Release", 50.0, 10.0, 500.0, "ms"),
                );
            }
            EffectType::Custom(_) => {
                // Custom effects define their own parameters
            }
        }

        params
    }

    /// Get parameter value
    pub fn get_parameter(&self, param_id: &str) -> Option<&EffectParameter> {
        self.parameters.get(param_id)
    }

    /// Set parameter value
    pub fn set_parameter(&mut self, param_id: &str, value: ParameterValue) -> Result<(), String> {
        if let Some(param) = self.parameters.get_mut(param_id) {
            // TODO: Validate against min/max
            param.value = value;
            Ok(())
        } else {
            Err(format!("Parameter {} not found", param_id))
        }
    }

    /// Process audio (placeholder - real DSP comes later)
    pub fn process(&self, _input: &[f32], _output: &mut [f32]) {
        // TODO: Implement actual DSP processing
        // For now, just pass through
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_creation() {
        let effect = Effect::new("gain1".to_string(), EffectType::Gain, "Gain".to_string());

        assert_eq!(effect.id, "gain1");
        assert_eq!(effect.effect_type, EffectType::Gain);
        assert!(effect.enabled);
        assert!(effect.parameters.contains_key("gain"));
    }

    #[test]
    fn test_parameter_set() {
        let mut effect = Effect::new("eq1".to_string(), EffectType::EQ, "EQ".to_string());

        effect
            .set_parameter("low_gain", ParameterValue::Float(6.0))
            .unwrap();

        if let Some(ParameterValue::Float(gain)) = effect
            .get_parameter("low_gain")
            .map(|p| &p.value)
        {
            assert_eq!(*gain, 6.0);
        } else {
            panic!("Expected float parameter");
        }
    }
}
```

---

## Step 2: Effect Chain (45 min)

Create `app/src-tauri/src/daw/mixer/effects/chain.rs`:

```rust
use super::models::{Effect, EffectType, ParameterValue};
use serde::{Deserialize, Serialize};

/// Effect chain for a single track or bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectChain {
    pub track_id: u32,
    pub effects: Vec<Effect>,
}

impl EffectChain {
    /// Create new effect chain
    pub fn new(track_id: u32) -> Self {
        Self {
            track_id,
            effects: Vec::new(),
        }
    }

    /// Add effect to end of chain
    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }

    /// Insert effect at specific position
    pub fn insert_effect(&mut self, index: usize, effect: Effect) -> Result<(), String> {
        if index > self.effects.len() {
            return Err(format!(
                "Index {} out of bounds (len: {})",
                index,
                self.effects.len()
            ));
        }

        self.effects.insert(index, effect);
        Ok(())
    }

    /// Remove effect by ID
    pub fn remove_effect(&mut self, effect_id: &str) -> Result<Effect, String> {
        if let Some(pos) = self.effects.iter().position(|e| e.id == effect_id) {
            Ok(self.effects.remove(pos))
        } else {
            Err(format!("Effect {} not found", effect_id))
        }
    }

    /// Move effect to new position
    pub fn move_effect(&mut self, effect_id: &str, new_index: usize) -> Result<(), String> {
        let old_pos = self
            .effects
            .iter()
            .position(|e| e.id == effect_id)
            .ok_or_else(|| format!("Effect {} not found", effect_id))?;

        if new_index >= self.effects.len() {
            return Err(format!(
                "Index {} out of bounds (len: {})",
                new_index,
                self.effects.len()
            ));
        }

        let effect = self.effects.remove(old_pos);
        self.effects.insert(new_index, effect);

        Ok(())
    }

    /// Get effect by ID
    pub fn get_effect(&self, effect_id: &str) -> Option<&Effect> {
        self.effects.iter().find(|e| e.id == effect_id)
    }

    /// Get mutable effect by ID
    pub fn get_effect_mut(&mut self, effect_id: &str) -> Option<&mut Effect> {
        self.effects.iter_mut().find(|e| e.id == effect_id)
    }

    /// Enable/disable effect
    pub fn set_effect_enabled(&mut self, effect_id: &str, enabled: bool) -> Result<(), String> {
        if let Some(effect) = self.get_effect_mut(effect_id) {
            effect.enabled = enabled;
            Ok(())
        } else {
            Err(format!("Effect {} not found", effect_id))
        }
    }

    /// Set effect parameter
    pub fn set_effect_parameter(
        &mut self,
        effect_id: &str,
        param_id: &str,
        value: ParameterValue,
    ) -> Result<(), String> {
        if let Some(effect) = self.get_effect_mut(effect_id) {
            effect.set_parameter(param_id, value)
        } else {
            Err(format!("Effect {} not found", effect_id))
        }
    }

    /// Process audio through effect chain
    pub fn process(&self, input: &[f32], output: &mut [f32]) {
        // Copy input to output
        output.copy_from_slice(input);

        // Process through each enabled effect in order
        for effect in &self.effects {
            if effect.enabled {
                effect.process(output, output);
            }
        }
    }

    /// Get effect count
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_chain() {
        let mut chain = EffectChain::new(1);

        let effect1 = Effect::new("gain1".to_string(), EffectType::Gain, "Gain 1".to_string());
        let effect2 = Effect::new("eq1".to_string(), EffectType::EQ, "EQ 1".to_string());

        chain.add_effect(effect1);
        chain.add_effect(effect2);

        assert_eq!(chain.len(), 2);
        assert!(chain.get_effect("gain1").is_some());
        assert!(chain.get_effect("eq1").is_some());
    }

    #[test]
    fn test_move_effect() {
        let mut chain = EffectChain::new(1);

        chain.add_effect(Effect::new(
            "e1".to_string(),
            EffectType::Gain,
            "E1".to_string(),
        ));
        chain.add_effect(Effect::new(
            "e2".to_string(),
            EffectType::EQ,
            "E2".to_string(),
        ));
        chain.add_effect(Effect::new(
            "e3".to_string(),
            EffectType::Reverb,
            "E3".to_string(),
        ));

        chain.move_effect("e3", 0).unwrap();

        assert_eq!(chain.effects[0].id, "e3");
        assert_eq!(chain.effects[1].id, "e1");
        assert_eq!(chain.effects[2].id, "e2");
    }
}
```

---

## Step 3: Tauri Commands (45 min)

Create `app/src-tauri/src/commands/daw/effects_commands.rs`:

```rust
use crate::daw::mixer::effects::{Effect, EffectChain, EffectType, ParameterValue};
use crate::AppState;
use tauri::State;

/// Add effect to track
#[tauri::command]
pub async fn add_effect(
    track_id: u32,
    effect_id: String,
    effect_type: EffectType,
    name: String,
    state: State<'_, AppState>,
) -> Result<Effect, String> {
    log::info!(
        "Adding effect {} ({:?}) to track {}",
        effect_id,
        effect_type,
        track_id
    );

    let sequencer = state.sequencer.lock().await;
    let effect = Effect::new(effect_id, effect_type, name);

    sequencer.add_effect(track_id, effect.clone()).await?;

    Ok(effect)
}

/// Remove effect from track
#[tauri::command]
pub async fn remove_effect(
    track_id: u32,
    effect_id: String,
    state: State<'_, AppState>,
) -> Result<Effect, String> {
    log::info!("Removing effect {} from track {}", effect_id, track_id);

    let sequencer = state.sequencer.lock().await;
    sequencer.remove_effect(track_id, &effect_id).await
}

/// Move effect to new position in chain
#[tauri::command]
pub async fn move_effect(
    track_id: u32,
    effect_id: String,
    new_index: usize,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Moving effect {} to position {} on track {}",
        effect_id,
        new_index,
        track_id
    );

    let sequencer = state.sequencer.lock().await;
    sequencer.move_effect(track_id, &effect_id, new_index).await
}

/// Enable/disable effect
#[tauri::command]
pub async fn set_effect_enabled(
    track_id: u32,
    effect_id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Setting effect {} enabled={} on track {}",
        effect_id,
        enabled,
        track_id
    );

    let sequencer = state.sequencer.lock().await;
    sequencer
        .set_effect_enabled(track_id, &effect_id, enabled)
        .await
}

/// Set effect parameter
#[tauri::command]
pub async fn set_effect_parameter(
    track_id: u32,
    effect_id: String,
    param_id: String,
    value: ParameterValue,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Setting effect {} parameter {} on track {}",
        effect_id,
        param_id,
        track_id
    );

    let sequencer = state.sequencer.lock().await;
    sequencer
        .set_effect_parameter(track_id, &effect_id, &param_id, value)
        .await
}

/// Get effect chain for track
#[tauri::command]
pub async fn get_effect_chain(
    track_id: u32,
    state: State<'_, AppState>,
) -> Result<EffectChain, String> {
    let sequencer = state.sequencer.lock().await;
    sequencer.get_effect_chain(track_id).await
}

/// Get single effect
#[tauri::command]
pub async fn get_effect(
    track_id: u32,
    effect_id: String,
    state: State<'_, AppState>,
) -> Result<Effect, String> {
    let sequencer = state.sequencer.lock().await;
    sequencer.get_effect(track_id, &effect_id).await
}
```

Register commands in `mod.rs` and `main.rs`:

```rust
// In commands/daw/mod.rs
pub mod effects_commands;
pub use effects_commands::*;

// In main.rs
use midi_app::commands::daw::{
    // ... existing ...
    add_effect,
    remove_effect,
    move_effect,
    set_effect_enabled,
    set_effect_parameter,
    get_effect_chain,
    get_effect,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing ...
            add_effect,
            remove_effect,
            move_effect,
            set_effect_enabled,
            set_effect_parameter,
            get_effect_chain,
            get_effect,
        ])
        // ...
}
```

---

## Verification (15 min)

```bash
cd app/src-tauri
cargo check
cargo test --lib effects
```

Test in browser console:

```javascript
// Add gain effect
const effect = await window.__TAURI__.invoke('add_effect', {
  trackId: 1,
  effectId: 'gain1',
  effectType: 'Gain',
  name: 'Gain 1'
});
console.log('Added effect:', effect);

// Get effect chain
const chain = await window.__TAURI__.invoke('get_effect_chain', { trackId: 1 });
console.log('Effect chain:', chain);

// Set parameter
await window.__TAURI__.invoke('set_effect_parameter', {
  trackId: 1,
  effectId: 'gain1',
  paramId: 'gain',
  value: { type: 'Float', value: 6.0 }
});

// Disable effect
await window.__TAURI__.invoke('set_effect_enabled', {
  trackId: 1,
  effectId: 'gain1',
  enabled: false
});
```

---

## What's Next?

✅ **You've completed:**
- Effect models with parameters
- Effect chain management
- 7 Tauri commands for effects
- Foundation for VST/LV2 integration

**Next:** [Part 3B: Effects Frontend](./DAY4_PART_B_EFFECTS_FRONTEND.md)
- Effect rack UI component
- Parameter controls
- Effect selector
- Drag-and-drop reordering
