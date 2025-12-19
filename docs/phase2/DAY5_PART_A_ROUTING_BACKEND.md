# Day 5, Part 4A: Routing Backend

**Duration:** 2 hours
**Prerequisites:** Days 1-4 complete
**Files to create:** 2

---

## Overview

Build audio routing system:
1. Bus/aux channel models
2. Send/return routing
3. Routing matrix
4. Integration with mixer

---

## Step 1: Routing Models (45 min)

Create `app/src-tauri/src/daw/mixer/routing.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bus type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BusType {
    /// Main output bus
    Master,
    /// Auxiliary bus (for effects sends, etc.)
    Aux,
    /// Group bus (for submixing)
    Group,
}

/// Audio bus (master, aux, group)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bus {
    pub id: u32,
    pub name: String,
    pub bus_type: BusType,
    pub gain_db: f32,
    pub muted: bool,
    pub enabled: bool,
}

impl Bus {
    pub fn new(id: u32, name: String, bus_type: BusType) -> Self {
        Self {
            id,
            name,
            bus_type,
            gain_db: 0.0,
            muted: false,
            enabled: true,
        }
    }

    pub fn master() -> Self {
        Self::new(0, "Master".to_string(), BusType::Master)
    }
}

/// Send from a track to a bus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Send {
    /// Source track ID
    pub from_track: u32,

    /// Destination bus ID
    pub to_bus: u32,

    /// Send level (0.0 to 1.0)
    pub level: f32,

    /// Pre or post fader
    pub pre_fader: bool,

    /// Enabled flag
    pub enabled: bool,
}

impl Send {
    pub fn new(from_track: u32, to_bus: u32) -> Self {
        Self {
            from_track,
            to_bus,
            level: 0.5,
            pre_fader: false,
            enabled: true,
        }
    }
}

/// Routing configuration for entire mixer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    /// All buses (master, aux, groups)
    pub buses: HashMap<u32, Bus>,

    /// Track-to-bus sends
    pub sends: Vec<Send>,

    /// Track routing (which bus each track outputs to)
    /// Default: all tracks route to master (bus 0)
    pub track_outputs: HashMap<u32, u32>,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        let mut buses = HashMap::new();
        buses.insert(0, Bus::master());

        Self {
            buses,
            sends: Vec::new(),
            track_outputs: HashMap::new(),
        }
    }
}

impl RoutingConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new bus
    pub fn add_bus(&mut self, bus: Bus) {
        self.buses.insert(bus.id, bus);
    }

    /// Remove a bus
    pub fn remove_bus(&mut self, bus_id: u32) -> Result<Bus, String> {
        if bus_id == 0 {
            return Err("Cannot remove master bus".to_string());
        }

        // Remove all sends to this bus
        self.sends.retain(|s| s.to_bus != bus_id);

        // Re-route tracks that output to this bus to master
        for output in self.track_outputs.values_mut() {
            if *output == bus_id {
                *output = 0; // Master
            }
        }

        self.buses
            .remove(&bus_id)
            .ok_or_else(|| format!("Bus {} not found", bus_id))
    }

    /// Get bus
    pub fn get_bus(&self, bus_id: u32) -> Option<&Bus> {
        self.buses.get(&bus_id)
    }

    /// Get mutable bus
    pub fn get_bus_mut(&mut self, bus_id: u32) -> Option<&mut Bus> {
        self.buses.get_mut(&bus_id)
    }

    /// Add a send
    pub fn add_send(&mut self, send: Send) {
        // Check if send already exists
        if let Some(existing) = self
            .sends
            .iter_mut()
            .find(|s| s.from_track == send.from_track && s.to_bus == send.to_bus)
        {
            *existing = send;
        } else {
            self.sends.push(send);
        }
    }

    /// Remove a send
    pub fn remove_send(&mut self, from_track: u32, to_bus: u32) -> Result<(), String> {
        if let Some(pos) = self
            .sends
            .iter()
            .position(|s| s.from_track == from_track && s.to_bus == to_bus)
        {
            self.sends.remove(pos);
            Ok(())
        } else {
            Err(format!(
                "Send from track {} to bus {} not found",
                from_track, to_bus
            ))
        }
    }

    /// Get send
    pub fn get_send(&self, from_track: u32, to_bus: u32) -> Option<&Send> {
        self.sends
            .iter()
            .find(|s| s.from_track == from_track && s.to_bus == to_bus)
    }

    /// Get mutable send
    pub fn get_send_mut(&mut self, from_track: u32, to_bus: u32) -> Option<&mut Send> {
        self.sends
            .iter_mut()
            .find(|s| s.from_track == from_track && s.to_bus == to_bus)
    }

    /// Get all sends from a track
    pub fn get_track_sends(&self, track_id: u32) -> Vec<&Send> {
        self.sends
            .iter()
            .filter(|s| s.from_track == track_id)
            .collect()
    }

    /// Get all sends to a bus
    pub fn get_bus_sends(&self, bus_id: u32) -> Vec<&Send> {
        self.sends.iter().filter(|s| s.to_bus == bus_id).collect()
    }

    /// Set track output bus
    pub fn set_track_output(&mut self, track_id: u32, bus_id: u32) -> Result<(), String> {
        if !self.buses.contains_key(&bus_id) {
            return Err(format!("Bus {} not found", bus_id));
        }

        self.track_outputs.insert(track_id, bus_id);
        Ok(())
    }

    /// Get track output bus (defaults to master)
    pub fn get_track_output(&self, track_id: u32) -> u32 {
        *self.track_outputs.get(&track_id).unwrap_or(&0)
    }

    /// Create default aux buses
    pub fn create_default_auxes(&mut self, count: u32) {
        for i in 1..=count {
            let bus = Bus::new(i, format!("Aux {}", i), BusType::Aux);
            self.add_bus(bus);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_config() {
        let mut routing = RoutingConfig::new();

        // Add aux bus
        routing.add_bus(Bus::new(1, "Aux 1".to_string(), BusType::Aux));

        // Add send
        let send = Send::new(1, 1);
        routing.add_send(send);

        assert_eq!(routing.buses.len(), 2); // Master + Aux 1
        assert_eq!(routing.sends.len(), 1);
    }

    #[test]
    fn test_remove_bus() {
        let mut routing = RoutingConfig::new();
        routing.add_bus(Bus::new(1, "Aux 1".to_string(), BusType::Aux));
        routing.add_send(Send::new(1, 1));
        routing.set_track_output(1, 1).unwrap();

        routing.remove_bus(1).unwrap();

        assert_eq!(routing.buses.len(), 1); // Only master
        assert_eq!(routing.sends.len(), 0); // Send removed
        assert_eq!(routing.get_track_output(1), 0); // Re-routed to master
    }

    #[test]
    fn test_cannot_remove_master() {
        let mut routing = RoutingConfig::new();

        assert!(routing.remove_bus(0).is_err());
    }
}
```

---

## Step 2: SequencerEngine Integration (30 min)

Update `app/src-tauri/src/daw/sequencer/engine.rs`:

```rust
use crate::daw::mixer::routing::{RoutingConfig, Bus, Send, BusType};

pub struct SequencerEngine {
    mixer: Arc<Mutex<MixerConfig>>,
    metering: Arc<Mutex<MeteringConfig>>,
    routing: Arc<Mutex<RoutingConfig>>, // NEW
    // ... other fields
}

impl SequencerEngine {
    pub fn new() -> Self {
        Self {
            mixer: Arc::new(Mutex::new(MixerConfig::default())),
            metering: Arc::new(Mutex::new(MeteringConfig::new(48000))),
            routing: Arc::new(Mutex::new(RoutingConfig::new())), // NEW
            // ... other fields
        }
    }

    // Routing methods
    pub async fn get_routing(&self) -> RoutingConfig {
        self.routing.lock().await.clone()
    }

    pub async fn add_bus(&self, bus: Bus) -> Result<(), String> {
        let mut routing = self.routing.lock().await;
        routing.add_bus(bus);
        Ok(())
    }

    pub async fn remove_bus(&self, bus_id: u32) -> Result<Bus, String> {
        let mut routing = self.routing.lock().await;
        routing.remove_bus(bus_id)
    }

    pub async fn add_send(&self, send: Send) -> Result<(), String> {
        let mut routing = self.routing.lock().await;
        routing.add_send(send);
        Ok(())
    }

    pub async fn remove_send(&self, from_track: u32, to_bus: u32) -> Result<(), String> {
        let mut routing = self.routing.lock().await;
        routing.remove_send(from_track, to_bus)
    }

    pub async fn set_send_level(
        &self,
        from_track: u32,
        to_bus: u32,
        level: f32,
    ) -> Result<(), String> {
        let mut routing = self.routing.lock().await;

        if let Some(send) = routing.get_send_mut(from_track, to_bus) {
            send.level = level.clamp(0.0, 1.0);
            Ok(())
        } else {
            Err(format!(
                "Send from track {} to bus {} not found",
                from_track, to_bus
            ))
        }
    }

    pub async fn set_track_output(&self, track_id: u32, bus_id: u32) -> Result<(), String> {
        let mut routing = self.routing.lock().await;
        routing.set_track_output(track_id, bus_id)
    }

    pub async fn create_default_auxes(&self, count: u32) -> Result<(), String> {
        let mut routing = self.routing.lock().await;
        routing.create_default_auxes(count);
        Ok(())
    }
}
```

---

## Step 3: Tauri Commands (30 min)

Create `app/src-tauri/src/commands/daw/routing_commands.rs`:

```rust
use crate::daw::mixer::routing::{Bus, BusType, RoutingConfig, Send};
use crate::AppState;
use tauri::State;

/// Get routing configuration
#[tauri::command]
pub async fn get_routing_config(state: State<'_, AppState>) -> Result<RoutingConfig, String> {
    let sequencer = state.sequencer.lock().await;
    Ok(sequencer.get_routing().await)
}

/// Add a new bus
#[tauri::command]
pub async fn add_bus(
    bus_id: u32,
    name: String,
    bus_type: BusType,
    state: State<'_, AppState>,
) -> Result<Bus, String> {
    log::info!("Adding bus {} ({:?}): {}", bus_id, bus_type, name);

    let bus = Bus::new(bus_id, name, bus_type);

    let sequencer = state.sequencer.lock().await;
    sequencer.add_bus(bus.clone()).await?;

    Ok(bus)
}

/// Remove a bus
#[tauri::command]
pub async fn remove_bus(bus_id: u32, state: State<'_, AppState>) -> Result<Bus, String> {
    log::info!("Removing bus {}", bus_id);

    let sequencer = state.sequencer.lock().await;
    sequencer.remove_bus(bus_id).await
}

/// Add a send from track to bus
#[tauri::command]
pub async fn add_send(
    from_track: u32,
    to_bus: u32,
    level: f32,
    pre_fader: bool,
    state: State<'_, AppState>,
) -> Result<Send, String> {
    log::info!("Adding send: track {} -> bus {} (level: {})", from_track, to_bus, level);

    let mut send = Send::new(from_track, to_bus);
    send.level = level.clamp(0.0, 1.0);
    send.pre_fader = pre_fader;

    let sequencer = state.sequencer.lock().await;
    sequencer.add_send(send.clone()).await?;

    Ok(send)
}

/// Remove a send
#[tauri::command]
pub async fn remove_send(
    from_track: u32,
    to_bus: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Removing send: track {} -> bus {}", from_track, to_bus);

    let sequencer = state.sequencer.lock().await;
    sequencer.remove_send(from_track, to_bus).await
}

/// Set send level
#[tauri::command]
pub async fn set_send_level(
    from_track: u32,
    to_bus: u32,
    level: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting send level: track {} -> bus {} = {}", from_track, to_bus, level);

    let sequencer = state.sequencer.lock().await;
    sequencer.set_send_level(from_track, to_bus, level).await
}

/// Set track output bus
#[tauri::command]
pub async fn set_track_output(
    track_id: u32,
    bus_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Setting track {} output to bus {}", track_id, bus_id);

    let sequencer = state.sequencer.lock().await;
    sequencer.set_track_output(track_id, bus_id).await
}

/// Create default aux buses
#[tauri::command]
pub async fn create_default_auxes(
    count: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!("Creating {} default aux buses", count);

    let sequencer = state.sequencer.lock().await;
    sequencer.create_default_auxes(count).await
}
```

Register in `mod.rs` and `main.rs`:

```rust
// In commands/daw/mod.rs
pub mod routing_commands;
pub use routing_commands::*;

// In main.rs
use midi_app::commands::daw::{
    // ... existing ...
    get_routing_config,
    add_bus,
    remove_bus,
    add_send,
    remove_send,
    set_send_level,
    set_track_output,
    create_default_auxes,
};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing ...
            get_routing_config,
            add_bus,
            remove_bus,
            add_send,
            remove_send,
            set_send_level,
            set_track_output,
            create_default_auxes,
        ])
        // ...
}
```

---

## Verification (15 min)

```bash
cd app/src-tauri
cargo check
cargo test --lib routing
```

Test in browser console:

```javascript
// Create aux buses
await window.__TAURI__.invoke('create_default_auxes', { count: 4 });

// Get routing config
const routing = await window.__TAURI__.invoke('get_routing_config');
console.log('Routing:', routing);

// Add send (track 1 -> aux 1)
await window.__TAURI__.invoke('add_send', {
  fromTrack: 1,
  toBus: 1,
  level: 0.5,
  preFader: false
});

// Set send level
await window.__TAURI__.invoke('set_send_level', {
  fromTrack: 1,
  toBus: 1,
  level: 0.8
});

// Set track output
await window.__TAURI__.invoke('set_track_output', {
  trackId: 2,
  busId: 1  // Route to Aux 1 instead of master
});
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Cannot remove master | Master bus (ID 0) is protected from removal |
| Send not found | Ensure bus exists before creating send |
| Track output error | Verify bus ID is valid |
| Circular routing | Add validation to prevent feedback loops |

---

## What's Next?

✅ **You've completed:**
- Bus/aux system with master, aux, group buses
- Send/return routing
- Track output routing
- 8 Tauri commands for routing

**Next:** [Part 4B: Routing Frontend](./DAY5_PART_B_ROUTING_FRONTEND.md)
- Routing matrix UI
- Send/return controls
- Bus management
- Visual routing display
