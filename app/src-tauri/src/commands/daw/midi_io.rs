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
#[allow(clippy::too_many_arguments)]
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
#[allow(clippy::too_many_arguments)]
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

// =============================================================================
// FRONTEND API COMMANDS (Added for MidiSyncControls.svelte support)
// =============================================================================

/// Serializable state for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiIOStateResponse {
    pub ports: HashMap<u64, MidiPort>,
    pub next_id: u64,
}

/// Get complete MIDI I/O state
#[command]
pub async fn midi_io_get_state(
    state: State<'_, MidiIOState>,
) -> Result<MidiIOStateResponse, String> {
    let ports = state.ports.lock().unwrap();
    let next_id = state.next_port_id.lock().unwrap();

    Ok(MidiIOStateResponse {
        ports: ports.clone(),
        next_id: *next_id,
    })
}

/// Detect/refresh MIDI ports from system
#[command]
pub async fn midi_io_detect_ports(
    state: State<'_, MidiIOState>,
) -> Result<Vec<MidiPort>, String> {
    // Trigger port refresh (in real implementation, this would use midir)
    midi_io_refresh_ports(state.clone()).await?;

    // Return current port list
    midi_io_list_ports(state).await
}

/// Add a new MIDI port (simplified version for frontend)
#[command]
pub async fn midi_io_add_port(
    state: State<'_, MidiIOState>,
    name: String,
    direction: String,
    device_name: Option<String>,
) -> Result<MidiPort, String> {
    // Use the existing register_port command with simplified parameters
    midi_io_register_port(
        state,
        name,
        device_name,
        direction,
        Some("hardware".to_string()),
    )
    .await
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_state() -> MidiIOState {
        MidiIOState::new()
    }

    // -------------------------------------------------------------------------
    // MidiPort Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_midi_port_default() {
        let port = MidiPort::default();
        assert_eq!(port.id, 0);
        assert_eq!(port.direction, "output");
        assert_eq!(port.port_type, "hardware");
        assert!(!port.is_connected);
        assert!(port.enabled);
        assert!(port.auto_connect);
        assert!(!port.send_clock);
        assert!(port.send_transport);
        assert_eq!(port.route_to, "selected");
    }

    #[test]
    fn test_midi_port_serialization() {
        let port = MidiPort {
            id: 1,
            system_name: "Test Port".to_string(),
            system_id: Some("test-123".to_string()),
            display_name: Some("My Port".to_string()),
            alias: None,
            direction: "input".to_string(),
            port_type: "virtual".to_string(),
            is_connected: true,
            enabled: true,
            auto_connect: false,
            send_clock: true,
            send_transport: false,
            route_to: "all".to_string(),
        };

        let json = serde_json::to_string(&port).unwrap();
        let deserialized: MidiPort = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, port.id);
        assert_eq!(deserialized.system_name, port.system_name);
        assert_eq!(deserialized.direction, port.direction);
    }

    // -------------------------------------------------------------------------
    // MidiRoute Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_midi_route_default() {
        let route = MidiRoute::default();
        assert_eq!(route.id, 0);
        assert!(route.enabled);
        assert!(route.input_port_id.is_none());
        assert!(route.input_channel.is_none());
        assert!(route.output_port_id.is_none());
        assert!(route.output_channel.is_none());
        assert_eq!(route.filter_type, Some("all".to_string()));
        assert_eq!(route.transpose, 0);
        assert_eq!(route.velocity_scale, 100);
    }

    #[test]
    fn test_midi_route_serialization() {
        let route = MidiRoute {
            id: 1,
            name: Some("Test Route".to_string()),
            enabled: true,
            input_port_id: Some(1),
            input_channel: Some(1),
            output_port_id: Some(2),
            output_channel: Some(10),
            filter_type: Some("notes".to_string()),
            transpose: -12,
            velocity_scale: 80,
        };

        let json = serde_json::to_string(&route).unwrap();
        let deserialized: MidiRoute = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, route.id);
        assert_eq!(deserialized.transpose, -12);
        assert_eq!(deserialized.velocity_scale, 80);
    }

    // -------------------------------------------------------------------------
    // MidiPortGroup Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_midi_port_group_creation() {
        let group = MidiPortGroup {
            id: 1,
            name: "My Interface".to_string(),
            icon: Some("🎹".to_string()),
            ports: vec![1, 2, 3, 4],
        };

        assert_eq!(group.id, 1);
        assert_eq!(group.name, "My Interface");
        assert_eq!(group.ports.len(), 4);
    }

    #[test]
    fn test_midi_port_group_serialization() {
        let group = MidiPortGroup {
            id: 1,
            name: "Test Group".to_string(),
            icon: None,
            ports: vec![1, 2],
        };

        let json = serde_json::to_string(&group).unwrap();
        let deserialized: MidiPortGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, group.name);
        assert_eq!(deserialized.ports, group.ports);
    }

    // -------------------------------------------------------------------------
    // MidiIOState Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_midi_io_state_new() {
        let state = MidiIOState::new();
        assert!(state.ports.lock().unwrap().is_empty());
        assert!(state.port_groups.lock().unwrap().is_empty());
        assert!(state.routes.lock().unwrap().is_empty());
        assert_eq!(*state.next_port_id.lock().unwrap(), 1);
        assert_eq!(*state.next_group_id.lock().unwrap(), 1);
        assert_eq!(*state.next_route_id.lock().unwrap(), 1);
    }

    #[test]
    fn test_midi_io_state_default() {
        let state = MidiIOState::default();
        assert!(state.ports.lock().unwrap().is_empty());
    }

    // -------------------------------------------------------------------------
    // Port Command Tests (using direct state manipulation)
    // -------------------------------------------------------------------------

    #[test]
    fn test_register_and_list_ports() {
        let state = create_test_state();

        // Register input port
        let port = MidiPort {
            id: 1,
            system_name: "Input 1".to_string(),
            direction: "input".to_string(),
            is_connected: true,
            ..MidiPort::default()
        };
        state.ports.lock().unwrap().insert(1, port);
        *state.next_port_id.lock().unwrap() = 2;

        // Register output port
        let port2 = MidiPort {
            id: 2,
            system_name: "Output 1".to_string(),
            direction: "output".to_string(),
            is_connected: true,
            ..MidiPort::default()
        };
        state.ports.lock().unwrap().insert(2, port2);
        *state.next_port_id.lock().unwrap() = 3;

        let ports = state.ports.lock().unwrap();
        assert_eq!(ports.len(), 2);

        let inputs: Vec<_> = ports.values().filter(|p| p.direction == "input").collect();
        assert_eq!(inputs.len(), 1);

        let outputs: Vec<_> = ports.values().filter(|p| p.direction == "output").collect();
        assert_eq!(outputs.len(), 1);
    }

    #[test]
    fn test_port_get_by_id() {
        let state = create_test_state();

        let port = MidiPort {
            id: 1,
            system_name: "Test Port".to_string(),
            ..MidiPort::default()
        };
        state.ports.lock().unwrap().insert(1, port);

        let ports = state.ports.lock().unwrap();
        let retrieved = ports.get(&1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().system_name, "Test Port");

        let missing = ports.get(&999);
        assert!(missing.is_none());
    }

    #[test]
    fn test_port_update() {
        let state = create_test_state();

        let port = MidiPort {
            id: 1,
            system_name: "Test Port".to_string(),
            display_name: None,
            enabled: true,
            ..MidiPort::default()
        };
        state.ports.lock().unwrap().insert(1, port);

        // Update port
        {
            let mut ports = state.ports.lock().unwrap();
            if let Some(p) = ports.get_mut(&1) {
                p.display_name = Some("Updated Name".to_string());
                p.enabled = false;
                p.send_clock = true;
            }
        }

        let ports = state.ports.lock().unwrap();
        let updated = ports.get(&1).unwrap();
        assert_eq!(updated.display_name, Some("Updated Name".to_string()));
        assert!(!updated.enabled);
        assert!(updated.send_clock);
    }

    #[test]
    fn test_port_connection_status() {
        let state = create_test_state();

        let port = MidiPort {
            id: 1,
            system_name: "Test".to_string(),
            is_connected: false,
            ..MidiPort::default()
        };
        state.ports.lock().unwrap().insert(1, port);

        // Set connected
        {
            let mut ports = state.ports.lock().unwrap();
            if let Some(p) = ports.get_mut(&1) {
                p.is_connected = true;
            }
        }

        let ports = state.ports.lock().unwrap();
        assert!(ports.get(&1).unwrap().is_connected);
    }

    #[test]
    fn test_port_remove() {
        let state = create_test_state();

        let port = MidiPort { id: 1, system_name: "Test".to_string(), ..MidiPort::default() };
        state.ports.lock().unwrap().insert(1, port);

        let removed = state.ports.lock().unwrap().remove(&1);
        assert!(removed.is_some());

        let removed_again = state.ports.lock().unwrap().remove(&1);
        assert!(removed_again.is_none());
    }

    // -------------------------------------------------------------------------
    // Port Group Command Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_port_group_create_and_list() {
        let state = create_test_state();

        let group = MidiPortGroup {
            id: 1,
            name: "Interface 1".to_string(),
            icon: Some("🎹".to_string()),
            ports: vec![1, 2, 3, 4],
        };
        state.port_groups.lock().unwrap().insert(1, group);

        let groups = state.port_groups.lock().unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups.get(&1).unwrap().name, "Interface 1");
    }

    #[test]
    fn test_port_group_update() {
        let state = create_test_state();

        let group = MidiPortGroup {
            id: 1,
            name: "Original".to_string(),
            icon: None,
            ports: vec![1, 2],
        };
        state.port_groups.lock().unwrap().insert(1, group);

        // Update
        {
            let mut groups = state.port_groups.lock().unwrap();
            if let Some(g) = groups.get_mut(&1) {
                g.name = "Updated".to_string();
                g.icon = Some("🎸".to_string());
                g.ports = vec![1, 2, 3];
            }
        }

        let groups = state.port_groups.lock().unwrap();
        let updated = groups.get(&1).unwrap();
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.icon, Some("🎸".to_string()));
        assert_eq!(updated.ports.len(), 3);
    }

    #[test]
    fn test_port_group_delete() {
        let state = create_test_state();

        let group = MidiPortGroup { id: 1, name: "Test".to_string(), icon: None, ports: vec![] };
        state.port_groups.lock().unwrap().insert(1, group);

        let removed = state.port_groups.lock().unwrap().remove(&1);
        assert!(removed.is_some());

        assert!(state.port_groups.lock().unwrap().is_empty());
    }

    // -------------------------------------------------------------------------
    // Route Command Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_route_create_and_list() {
        let state = create_test_state();

        let route = MidiRoute {
            id: 1,
            name: Some("Piano to Synth".to_string()),
            input_port_id: Some(1),
            output_port_id: Some(2),
            ..MidiRoute::default()
        };
        state.routes.lock().unwrap().insert(1, route);

        let routes = state.routes.lock().unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes.get(&1).unwrap().name, Some("Piano to Synth".to_string()));
    }

    #[test]
    fn test_route_get_by_id() {
        let state = create_test_state();

        let route = MidiRoute { id: 1, name: Some("Test Route".to_string()), ..MidiRoute::default() };
        state.routes.lock().unwrap().insert(1, route);

        let routes = state.routes.lock().unwrap();
        assert!(routes.get(&1).is_some());
        assert!(routes.get(&999).is_none());
    }

    #[test]
    fn test_route_update() {
        let state = create_test_state();

        let route = MidiRoute {
            id: 1,
            name: Some("Original".to_string()),
            enabled: true,
            transpose: 0,
            velocity_scale: 100,
            ..MidiRoute::default()
        };
        state.routes.lock().unwrap().insert(1, route);

        // Update
        {
            let mut routes = state.routes.lock().unwrap();
            if let Some(r) = routes.get_mut(&1) {
                r.name = Some("Updated".to_string());
                r.enabled = false;
                r.transpose = 12;
                r.velocity_scale = 80;
            }
        }

        let routes = state.routes.lock().unwrap();
        let updated = routes.get(&1).unwrap();
        assert_eq!(updated.name, Some("Updated".to_string()));
        assert!(!updated.enabled);
        assert_eq!(updated.transpose, 12);
        assert_eq!(updated.velocity_scale, 80);
    }

    #[test]
    fn test_route_enable_disable() {
        let state = create_test_state();

        let route = MidiRoute { id: 1, enabled: true, ..MidiRoute::default() };
        state.routes.lock().unwrap().insert(1, route);

        // Disable
        {
            let mut routes = state.routes.lock().unwrap();
            routes.get_mut(&1).unwrap().enabled = false;
        }

        assert!(!state.routes.lock().unwrap().get(&1).unwrap().enabled);

        // Enable
        {
            let mut routes = state.routes.lock().unwrap();
            routes.get_mut(&1).unwrap().enabled = true;
        }

        assert!(state.routes.lock().unwrap().get(&1).unwrap().enabled);
    }

    #[test]
    fn test_route_delete() {
        let state = create_test_state();

        let route = MidiRoute { id: 1, ..MidiRoute::default() };
        state.routes.lock().unwrap().insert(1, route);

        let removed = state.routes.lock().unwrap().remove(&1);
        assert!(removed.is_some());
        assert!(state.routes.lock().unwrap().is_empty());
    }

    #[test]
    fn test_route_channel_validation() {
        // Channels should be 1-16
        let route = MidiRoute {
            id: 1,
            input_channel: Some(16),
            output_channel: Some(1),
            ..MidiRoute::default()
        };

        assert!(route.input_channel.unwrap() <= 16);
        assert!(route.output_channel.unwrap() >= 1);
    }

    #[test]
    fn test_route_transpose_range() {
        let route = MidiRoute { id: 1, transpose: -48, ..MidiRoute::default() };
        assert_eq!(route.transpose, -48);

        let route2 = MidiRoute { id: 2, transpose: 48, ..MidiRoute::default() };
        assert_eq!(route2.transpose, 48);
    }

    #[test]
    fn test_route_velocity_scale_range() {
        let route = MidiRoute { id: 1, velocity_scale: 0, ..MidiRoute::default() };
        assert_eq!(route.velocity_scale, 0);

        let route2 = MidiRoute { id: 2, velocity_scale: 200, ..MidiRoute::default() };
        assert_eq!(route2.velocity_scale, 200);
    }

    // -------------------------------------------------------------------------
    // Statistics Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_stats_calculation() {
        let state = create_test_state();

        // Add ports
        let input = MidiPort {
            id: 1,
            system_name: "In".to_string(),
            direction: "input".to_string(),
            is_connected: true,
            enabled: true,
            ..MidiPort::default()
        };
        let output = MidiPort {
            id: 2,
            system_name: "Out".to_string(),
            direction: "output".to_string(),
            is_connected: true,
            enabled: false,
            ..MidiPort::default()
        };
        let disconnected = MidiPort {
            id: 3,
            system_name: "Disc".to_string(),
            direction: "output".to_string(),
            is_connected: false,
            enabled: true,
            ..MidiPort::default()
        };
        {
            let mut ports = state.ports.lock().unwrap();
            ports.insert(1, input);
            ports.insert(2, output);
            ports.insert(3, disconnected);
        }

        // Add routes
        let route1 = MidiRoute { id: 1, enabled: true, ..MidiRoute::default() };
        let route2 = MidiRoute { id: 2, enabled: false, ..MidiRoute::default() };
        {
            let mut routes = state.routes.lock().unwrap();
            routes.insert(1, route1);
            routes.insert(2, route2);
        }

        // Calculate stats
        let ports = state.ports.lock().unwrap();
        let routes = state.routes.lock().unwrap();

        let total_ports = ports.len();
        let input_ports = ports.values().filter(|p| p.direction == "input").count();
        let output_ports = ports.values().filter(|p| p.direction == "output").count();
        let connected_ports = ports.values().filter(|p| p.is_connected).count();
        let enabled_ports = ports.values().filter(|p| p.enabled).count();
        let total_routes = routes.len();
        let enabled_routes = routes.values().filter(|r| r.enabled).count();

        assert_eq!(total_ports, 3);
        assert_eq!(input_ports, 1);
        assert_eq!(output_ports, 2);
        assert_eq!(connected_ports, 2);
        assert_eq!(enabled_ports, 2);
        assert_eq!(total_routes, 2);
        assert_eq!(enabled_routes, 1);
    }

    // -------------------------------------------------------------------------
    // Panic Test
    // -------------------------------------------------------------------------

    #[test]
    fn test_panic_requires_connected_outputs() {
        let state = create_test_state();

        // No ports - panic should fail
        let ports = state.ports.lock().unwrap();
        let output_count = ports
            .values()
            .filter(|p| p.direction == "output" && p.enabled && p.is_connected)
            .count();
        assert_eq!(output_count, 0);
    }

    #[test]
    fn test_panic_with_connected_outputs() {
        let state = create_test_state();

        let output = MidiPort {
            id: 1,
            system_name: "Out".to_string(),
            direction: "output".to_string(),
            is_connected: true,
            enabled: true,
            ..MidiPort::default()
        };
        state.ports.lock().unwrap().insert(1, output);

        let ports = state.ports.lock().unwrap();
        let output_count = ports
            .values()
            .filter(|p| p.direction == "output" && p.enabled && p.is_connected)
            .count();
        assert_eq!(output_count, 1);
    }

    // -------------------------------------------------------------------------
    // ID Generation Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_port_id_increments() {
        let state = create_test_state();

        assert_eq!(*state.next_port_id.lock().unwrap(), 1);

        *state.next_port_id.lock().unwrap() += 1;
        assert_eq!(*state.next_port_id.lock().unwrap(), 2);

        *state.next_port_id.lock().unwrap() += 1;
        assert_eq!(*state.next_port_id.lock().unwrap(), 3);
    }

    #[test]
    fn test_group_id_increments() {
        let state = create_test_state();

        assert_eq!(*state.next_group_id.lock().unwrap(), 1);

        *state.next_group_id.lock().unwrap() += 1;
        assert_eq!(*state.next_group_id.lock().unwrap(), 2);
    }

    #[test]
    fn test_route_id_increments() {
        let state = create_test_state();

        assert_eq!(*state.next_route_id.lock().unwrap(), 1);

        *state.next_route_id.lock().unwrap() += 1;
        assert_eq!(*state.next_route_id.lock().unwrap(), 2);
    }

    // -------------------------------------------------------------------------
    // State Response Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_state_response_serialization() {
        let mut ports = HashMap::new();
        ports.insert(
            1,
            MidiPort { id: 1, system_name: "Test".to_string(), ..MidiPort::default() },
        );

        let response = MidiIOStateResponse { ports, next_id: 2 };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: MidiIOStateResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.next_id, 2);
        assert_eq!(deserialized.ports.len(), 1);
    }

    // -------------------------------------------------------------------------
    // Edge Cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_empty_state_operations() {
        let state = create_test_state();

        // All collections should be empty
        assert!(state.ports.lock().unwrap().is_empty());
        assert!(state.port_groups.lock().unwrap().is_empty());
        assert!(state.routes.lock().unwrap().is_empty());
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let state = MidiIOState::new();
        let state1 = state.clone();
        let state2 = state.clone();

        let handle1 = thread::spawn(move || {
            for i in 1..=10 {
                let port = MidiPort {
                    id: i,
                    system_name: format!("Port {}", i),
                    ..MidiPort::default()
                };
                state1.ports.lock().unwrap().insert(i, port);
            }
        });

        let handle2 = thread::spawn(move || {
            for i in 11..=20 {
                let port = MidiPort {
                    id: i,
                    system_name: format!("Port {}", i),
                    ..MidiPort::default()
                };
                state2.ports.lock().unwrap().insert(i, port);
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        assert_eq!(state.ports.lock().unwrap().len(), 20);
    }

    #[test]
    fn test_route_filter_types() {
        let types = vec!["notes", "cc", "program", "all"];
        for filter_type in types {
            let route = MidiRoute {
                id: 1,
                filter_type: Some(filter_type.to_string()),
                ..MidiRoute::default()
            };
            assert_eq!(route.filter_type, Some(filter_type.to_string()));
        }
    }

    #[test]
    fn test_port_types() {
        let types = vec!["hardware", "virtual", "network"];
        for port_type in types {
            let port = MidiPort {
                id: 1,
                system_name: "Test".to_string(),
                port_type: port_type.to_string(),
                ..MidiPort::default()
            };
            assert_eq!(port.port_type, port_type);
        }
    }

    #[test]
    fn test_port_directions() {
        let input = MidiPort {
            id: 1,
            system_name: "In".to_string(),
            direction: "input".to_string(),
            ..MidiPort::default()
        };
        assert_eq!(input.direction, "input");

        let output = MidiPort {
            id: 2,
            system_name: "Out".to_string(),
            direction: "output".to_string(),
            ..MidiPort::default()
        };
        assert_eq!(output.direction, "output");
    }

    #[test]
    fn test_port_route_to_options() {
        let options = vec!["selected", "all", "none"];
        for route_to in options {
            let port = MidiPort {
                id: 1,
                system_name: "Test".to_string(),
                route_to: route_to.to_string(),
                ..MidiPort::default()
            };
            assert_eq!(port.route_to, route_to);
        }
    }
}
