#![allow(dead_code)]
//! MIDI I/O Commands - Port management, routing, and configuration
//!
//! This module provides Tauri commands for managing MIDI ports, port groups,
//! and routing rules. It corresponds to the midi_ports, midi_port_groups,
//! midi_port_group_members, and midi_routes tables from migration 018.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{command, State};

// =============================================================================
// DATA STRUCTURES
// =============================================================================

/// MIDI Port (physical or virtual)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiPort {
    pub id: u64,
    pub system_name: String,
    pub system_id: Option<String>,
    pub display_name: Option<String>,
    pub alias: Option<String>,
    pub direction: String, // "input" or "output"
    pub port_type: String, // "hardware", "virtual", "network"
    pub is_connected: bool,
    pub enabled: bool,
    pub auto_connect: bool,
    pub send_clock: bool,
    pub send_transport: bool,
    pub route_to: String, // "selected", "all", "none"
}

impl Default for MidiPort {
    fn default() -> Self {
        Self {
            id: 0,
            system_name: String::new(),
            system_id: None,
            display_name: None,
            alias: None,
            direction: "output".to_string(),
            port_type: "hardware".to_string(),
            is_connected: false,
            enabled: true,
            auto_connect: true,
            send_clock: false,
            send_transport: true,
            route_to: "selected".to_string(),
        }
    }
}

/// MIDI Port Group (for multi-port interfaces)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiPortGroup {
    pub id: u64,
    pub name: String,
    pub icon: Option<String>,
    pub ports: Vec<u64>, // Port IDs in order
}

/// MIDI Route (input to output mapping)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiRoute {
    pub id: u64,
    pub name: Option<String>,
    pub enabled: bool,
    pub input_port_id: Option<u64>,
    pub input_channel: Option<u8>, // None = all channels
    pub output_port_id: Option<u64>,
    pub output_channel: Option<u8>,  // None = pass through
    pub filter_type: Option<String>, // "notes", "cc", "program", "all"
    pub transpose: i8,
    pub velocity_scale: u8,
}

impl Default for MidiRoute {
    fn default() -> Self {
        Self {
            id: 0,
            name: None,
            enabled: true,
            input_port_id: None,
            input_channel: None,
            output_port_id: None,
            output_channel: None,
            filter_type: Some("all".to_string()),
            transpose: 0,
            velocity_scale: 100,
        }
    }
}

// =============================================================================
// STATE
// =============================================================================

#[derive(Debug, Clone, Default)]
pub struct MidiIOState {
    pub ports: Arc<Mutex<HashMap<u64, MidiPort>>>,
    pub port_groups: Arc<Mutex<HashMap<u64, MidiPortGroup>>>,
    pub routes: Arc<Mutex<HashMap<u64, MidiRoute>>>,
    next_port_id: Arc<Mutex<u64>>,
    next_group_id: Arc<Mutex<u64>>,
    next_route_id: Arc<Mutex<u64>>,
}

impl MidiIOState {
    pub fn new() -> Self {
        Self {
            ports: Arc::new(Mutex::new(HashMap::new())),
            port_groups: Arc::new(Mutex::new(HashMap::new())),
            routes: Arc::new(Mutex::new(HashMap::new())),
            next_port_id: Arc::new(Mutex::new(1)),
            next_group_id: Arc::new(Mutex::new(1)),
            next_route_id: Arc::new(Mutex::new(1)),
        }
    }
}

// =============================================================================
// PORT COMMANDS
// =============================================================================

/// List all MIDI ports
#[command]
pub async fn midi_io_list_ports(state: State<'_, MidiIOState>) -> Result<Vec<MidiPort>, String> {
    let ports = state.ports.lock().unwrap();
    Ok(ports.values().cloned().collect())
}

/// List input ports only
#[command]
pub async fn midi_io_list_inputs(state: State<'_, MidiIOState>) -> Result<Vec<MidiPort>, String> {
    let ports = state.ports.lock().unwrap();
    Ok(ports.values().filter(|p| p.direction == "input").cloned().collect())
}

/// List output ports only
#[command]
pub async fn midi_io_list_outputs(state: State<'_, MidiIOState>) -> Result<Vec<MidiPort>, String> {
    let ports = state.ports.lock().unwrap();
    Ok(ports.values().filter(|p| p.direction == "output").cloned().collect())
}

/// Get a specific port by ID
#[command]
pub async fn midi_io_get_port(
    state: State<'_, MidiIOState>,
    port_id: u64,
) -> Result<MidiPort, String> {
    let ports = state.ports.lock().unwrap();
    ports.get(&port_id).cloned().ok_or_else(|| "Port not found".to_string())
}

/// Register a new MIDI port (called when system discovers ports)
#[command]
pub async fn midi_io_register_port(
    state: State<'_, MidiIOState>,
    system_name: String,
    system_id: Option<String>,
    direction: String,
    port_type: Option<String>,
) -> Result<MidiPort, String> {
    if direction != "input" && direction != "output" {
        return Err("Direction must be 'input' or 'output'".to_string());
    }

    let mut ports = state.ports.lock().unwrap();
    let mut next_id = state.next_port_id.lock().unwrap();

    // Check if port already exists
    for port in ports.values() {
        if port.system_name == system_name && port.direction == direction {
            return Err("Port already registered".to_string());
        }
    }

    let port = MidiPort {
        id: *next_id,
        system_name,
        system_id,
        direction,
        port_type: port_type.unwrap_or_else(|| "hardware".to_string()),
        is_connected: true,
        ..MidiPort::default()
    };

    ports.insert(port.id, port.clone());
    *next_id += 1;

    Ok(port)
}

/// Update port settings
#[command]
pub async fn midi_io_update_port(
    state: State<'_, MidiIOState>,
    port_id: u64,
    display_name: Option<String>,
    alias: Option<String>,
    enabled: Option<bool>,
    auto_connect: Option<bool>,
    send_clock: Option<bool>,
    send_transport: Option<bool>,
    route_to: Option<String>,
) -> Result<MidiPort, String> {
    let mut ports = state.ports.lock().unwrap();

    if let Some(port) = ports.get_mut(&port_id) {
        if let Some(v) = display_name {
            port.display_name = Some(v);
        }
        if let Some(v) = alias {
            port.alias = Some(v);
        }
        if let Some(v) = enabled {
            port.enabled = v;
        }
        if let Some(v) = auto_connect {
            port.auto_connect = v;
        }
        if let Some(v) = send_clock {
            port.send_clock = v;
        }
        if let Some(v) = send_transport {
            port.send_transport = v;
        }
        if let Some(v) = route_to {
            if v == "selected" || v == "all" || v == "none" {
                port.route_to = v;
            }
        }
        Ok(port.clone())
    } else {
        Err("Port not found".to_string())
    }
}

/// Update port connection status
#[command]
pub async fn midi_io_set_port_connected(
    state: State<'_, MidiIOState>,
    port_id: u64,
    is_connected: bool,
) -> Result<(), String> {
    let mut ports = state.ports.lock().unwrap();

    if let Some(port) = ports.get_mut(&port_id) {
        port.is_connected = is_connected;
        Ok(())
    } else {
        Err("Port not found".to_string())
    }
}

/// Remove a port
#[command]
pub async fn midi_io_remove_port(
    state: State<'_, MidiIOState>,
    port_id: u64,
) -> Result<(), String> {
    let mut ports = state.ports.lock().unwrap();
    ports.remove(&port_id).map(|_| ()).ok_or_else(|| "Port not found".to_string())
}

// =============================================================================
// PORT GROUP COMMANDS
// =============================================================================

/// List all port groups
#[command]
pub async fn midi_io_list_port_groups(
    state: State<'_, MidiIOState>,
) -> Result<Vec<MidiPortGroup>, String> {
    let groups = state.port_groups.lock().unwrap();
    Ok(groups.values().cloned().collect())
}

/// Create a new port group
#[command]
pub async fn midi_io_create_port_group(
    state: State<'_, MidiIOState>,
    name: String,
    icon: Option<String>,
    port_ids: Vec<u64>,
) -> Result<MidiPortGroup, String> {
    let mut groups = state.port_groups.lock().unwrap();
    let mut next_id = state.next_group_id.lock().unwrap();

    let group = MidiPortGroup { id: *next_id, name, icon, ports: port_ids };

    groups.insert(group.id, group.clone());
    *next_id += 1;

    Ok(group)
}

/// Update a port group
#[command]
pub async fn midi_io_update_port_group(
    state: State<'_, MidiIOState>,
    group_id: u64,
    name: Option<String>,
    icon: Option<String>,
    port_ids: Option<Vec<u64>>,
) -> Result<MidiPortGroup, String> {
    let mut groups = state.port_groups.lock().unwrap();

    if let Some(group) = groups.get_mut(&group_id) {
        if let Some(v) = name {
            group.name = v;
        }
        if let Some(v) = icon {
            group.icon = Some(v);
        }
        if let Some(v) = port_ids {
            group.ports = v;
        }
        Ok(group.clone())
    } else {
        Err("Port group not found".to_string())
    }
}

/// Delete a port group
#[command]
pub async fn midi_io_delete_port_group(
    state: State<'_, MidiIOState>,
    group_id: u64,
) -> Result<(), String> {
    let mut groups = state.port_groups.lock().unwrap();
    groups
        .remove(&group_id)
        .map(|_| ())
        .ok_or_else(|| "Port group not found".to_string())
}

// =============================================================================
// ROUTING COMMANDS
// =============================================================================

/// List all MIDI routes
#[command]
pub async fn midi_io_list_routes(state: State<'_, MidiIOState>) -> Result<Vec<MidiRoute>, String> {
    let routes = state.routes.lock().unwrap();
    Ok(routes.values().cloned().collect())
}

/// Get a specific route
#[command]
pub async fn midi_io_get_route(
    state: State<'_, MidiIOState>,
    route_id: u64,
) -> Result<MidiRoute, String> {
    let routes = state.routes.lock().unwrap();
    routes.get(&route_id).cloned().ok_or_else(|| "Route not found".to_string())
}

/// Create a new MIDI route
#[command]
pub async fn midi_io_create_route(
    state: State<'_, MidiIOState>,
    name: Option<String>,
    input_port_id: Option<u64>,
    input_channel: Option<u8>,
    output_port_id: Option<u64>,
    output_channel: Option<u8>,
    filter_type: Option<String>,
) -> Result<MidiRoute, String> {
    // Validate channel ranges
    if let Some(ch) = input_channel {
        if ch > 16 {
            return Err("Input channel must be 1-16".to_string());
        }
    }
    if let Some(ch) = output_channel {
        if ch > 16 {
            return Err("Output channel must be 1-16".to_string());
        }
    }

    let mut routes = state.routes.lock().unwrap();
    let mut next_id = state.next_route_id.lock().unwrap();

    let route = MidiRoute {
        id: *next_id,
        name,
        enabled: true,
        input_port_id,
        input_channel,
        output_port_id,
        output_channel,
        filter_type,
        transpose: 0,
        velocity_scale: 100,
    };

    routes.insert(route.id, route.clone());
    *next_id += 1;

    Ok(route)
}

/// Update a MIDI route
#[command]
pub async fn midi_io_update_route(
    state: State<'_, MidiIOState>,
    route_id: u64,
    name: Option<String>,
    enabled: Option<bool>,
    input_port_id: Option<u64>,
    input_channel: Option<u8>,
    output_port_id: Option<u64>,
    output_channel: Option<u8>,
    filter_type: Option<String>,
    transpose: Option<i8>,
    velocity_scale: Option<u8>,
) -> Result<MidiRoute, String> {
    let mut routes = state.routes.lock().unwrap();

    if let Some(route) = routes.get_mut(&route_id) {
        if let Some(v) = name {
            route.name = Some(v);
        }
        if let Some(v) = enabled {
            route.enabled = v;
        }
        if let Some(v) = input_port_id {
            route.input_port_id = Some(v);
        }
        if let Some(v) = input_channel {
            route.input_channel = Some(v.clamp(1, 16));
        }
        if let Some(v) = output_port_id {
            route.output_port_id = Some(v);
        }
        if let Some(v) = output_channel {
            route.output_channel = Some(v.clamp(1, 16));
        }
        if let Some(v) = filter_type {
            route.filter_type = Some(v);
        }
        if let Some(v) = transpose {
            route.transpose = v.clamp(-48, 48);
        }
        if let Some(v) = velocity_scale {
            route.velocity_scale = v.min(200);
        }
        Ok(route.clone())
    } else {
        Err("Route not found".to_string())
    }
}

/// Enable/disable a route
#[command]
pub async fn midi_io_set_route_enabled(
    state: State<'_, MidiIOState>,
    route_id: u64,
    enabled: bool,
) -> Result<(), String> {
    let mut routes = state.routes.lock().unwrap();

    if let Some(route) = routes.get_mut(&route_id) {
        route.enabled = enabled;
        Ok(())
    } else {
        Err("Route not found".to_string())
    }
}

/// Delete a MIDI route
#[command]
pub async fn midi_io_delete_route(
    state: State<'_, MidiIOState>,
    route_id: u64,
) -> Result<(), String> {
    let mut routes = state.routes.lock().unwrap();
    routes
        .remove(&route_id)
        .map(|_| ())
        .ok_or_else(|| "Route not found".to_string())
}

// =============================================================================
// UTILITY COMMANDS
// =============================================================================

/// Refresh ports (trigger system rescan)
#[command]
pub async fn midi_io_refresh_ports(_state: State<'_, MidiIOState>) -> Result<(), String> {
    // In a real implementation, this would trigger midir port enumeration
    // For now, just return success
    Ok(())
}

/// Send MIDI panic (all notes off on all outputs)
#[command]
pub async fn midi_io_panic(state: State<'_, MidiIOState>) -> Result<(), String> {
    let ports = state.ports.lock().unwrap();

    // Count output ports that would receive panic
    let output_count = ports
        .values()
        .filter(|p| p.direction == "output" && p.enabled && p.is_connected)
        .count();

    // In a real implementation, this would send CC 123 (All Notes Off) to all channels
    // on all connected output ports

    if output_count > 0 {
        Ok(())
    } else {
        Err("No connected output ports".to_string())
    }
}

/// Get summary statistics
#[command]
pub async fn midi_io_get_stats(
    state: State<'_, MidiIOState>,
) -> Result<HashMap<String, usize>, String> {
    let ports = state.ports.lock().unwrap();
    let routes = state.routes.lock().unwrap();

    let mut stats = HashMap::new();
    stats.insert("total_ports".to_string(), ports.len());
    stats.insert(
        "input_ports".to_string(),
        ports.values().filter(|p| p.direction == "input").count(),
    );
    stats.insert(
        "output_ports".to_string(),
        ports.values().filter(|p| p.direction == "output").count(),
    );
    stats.insert(
        "connected_ports".to_string(),
        ports.values().filter(|p| p.is_connected).count(),
    );
    stats.insert(
        "enabled_ports".to_string(),
        ports.values().filter(|p| p.enabled).count(),
    );
    stats.insert("total_routes".to_string(), routes.len());
    stats.insert(
        "enabled_routes".to_string(),
        routes.values().filter(|r| r.enabled).count(),
    );

    Ok(stats)
}
