// MIDI Router - Route MIDI messages between devices and destinations
// GROWN-UP SCRIPT - Routing logic with state management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// MIDI message filter for routing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MidiFilter {
    pub channel_filter: Option<Vec<u8>>,
    pub message_type_filter: Option<Vec<String>>,
    pub note_range: Option<(u8, u8)>,
    pub velocity_range: Option<(u8, u8)>,
    pub transpose: i8,
}

impl MidiFilter {
    /// Create a new empty filter (passes all messages)
    pub fn new() -> Self {
        Self {
            channel_filter: None,
            message_type_filter: None,
            note_range: None,
            velocity_range: None,
            transpose: 0,
        }
    }

    /// Check if message passes filter
    pub fn passes(&self, channel: u8, message_type: &str, note: u8, velocity: u8) -> bool {
        // Channel filter
        if let Some(ref channels) = self.channel_filter {
            if !channels.contains(&channel) {
                return false;
            }
        }

        // Message type filter
        if let Some(ref types) = self.message_type_filter {
            if !types.contains(&message_type.to_string()) {
                return false;
            }
        }

        // Note range filter
        if let Some((min, max)) = self.note_range {
            if note < min || note > max {
                return false;
            }
        }

        // Velocity range filter
        if let Some((min, max)) = self.velocity_range {
            if velocity < min || velocity > max {
                return false;
            }
        }

        true
    }

    /// Apply transpose to note
    pub fn apply_transpose(&self, note: u8) -> u8 {
        let transposed = note as i16 + self.transpose as i16;
        transposed.clamp(0, 127) as u8
    }
}

impl Default for MidiFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// MIDI routing destination
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RouteDestination {
    Device(String),
    Track(i32),
    Virtual(String),
}

impl RouteDestination {
    /// Get destination identifier
    pub fn id(&self) -> String {
        match self {
            RouteDestination::Device(id) => format!("device:{}", id),
            RouteDestination::Track(id) => format!("track:{}", id),
            RouteDestination::Virtual(id) => format!("virtual:{}", id),
        }
    }
}

/// MIDI route configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiRoute {
    pub id: i32,
    pub from_device: String,
    pub to_destination: RouteDestination,
    pub filter: MidiFilter,
    pub enabled: bool,
    pub name: String,
}

impl MidiRoute {
    /// Create a new route
    pub fn new(
        id: i32,
        from_device: String,
        to_destination: RouteDestination,
        name: String,
    ) -> Self {
        Self { id, from_device, to_destination, filter: MidiFilter::new(), enabled: true, name }
    }

    /// Enable route
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable route
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Set filter
    pub fn set_filter(&mut self, filter: MidiFilter) {
        self.filter = filter;
    }
}

/// State for MIDI routing
#[derive(Debug, Clone)]
pub struct MidiRouterState {
    pub routes: HashMap<i32, MidiRoute>,
    pub next_route_id: i32,
}

impl MidiRouterState {
    /// Create new router state
    pub fn new() -> Self {
        Self { routes: HashMap::new(), next_route_id: 1 }
    }

    /// Create a new route
    pub fn create_route(
        &mut self,
        from_device: String,
        to_destination: RouteDestination,
        name: String,
    ) -> i32 {
        let route_id = self.next_route_id;
        self.next_route_id += 1;

        let route = MidiRoute::new(route_id, from_device, to_destination, name);
        self.routes.insert(route_id, route);

        route_id
    }

    /// Delete a route
    pub fn delete_route(&mut self, route_id: i32) -> Result<()> {
        if self.routes.remove(&route_id).is_none() {
            return Err(anyhow::anyhow!("Route not found: {}", route_id));
        }
        Ok(())
    }

    /// Enable a route
    pub fn enable_route(&mut self, route_id: i32) -> Result<()> {
        let route = self
            .routes
            .get_mut(&route_id)
            .ok_or_else(|| anyhow::anyhow!("Route not found: {}", route_id))?;

        route.enable();
        Ok(())
    }

    /// Disable a route
    pub fn disable_route(&mut self, route_id: i32) -> Result<()> {
        let route = self
            .routes
            .get_mut(&route_id)
            .ok_or_else(|| anyhow::anyhow!("Route not found: {}", route_id))?;

        route.disable();
        Ok(())
    }

    /// Get all routes
    pub fn get_all_routes(&self) -> Vec<MidiRoute> {
        let mut routes: Vec<MidiRoute> = self.routes.values().cloned().collect();
        routes.sort_by_key(|r| r.id);
        routes
    }

    /// Get route by id
    pub fn get_route(&self, route_id: i32) -> Option<&MidiRoute> {
        self.routes.get(&route_id)
    }

    /// Update route filter
    pub fn update_filter(&mut self, route_id: i32, filter: MidiFilter) -> Result<()> {
        let route = self
            .routes
            .get_mut(&route_id)
            .ok_or_else(|| anyhow::anyhow!("Route not found: {}", route_id))?;

        route.set_filter(filter);
        Ok(())
    }

    /// Get active routes for a device
    pub fn get_active_routes_for_device(&self, device_id: &str) -> Vec<&MidiRoute> {
        self.routes
            .values()
            .filter(|r| r.enabled && r.from_device == device_id)
            .collect()
    }

    /// Test if route configuration is valid
    pub fn test_route(&self, route_id: i32) -> Result<bool> {
        let route = self
            .routes
            .get(&route_id)
            .ok_or_else(|| anyhow::anyhow!("Route not found: {}", route_id))?;

        // Basic validation
        if route.from_device.is_empty() {
            return Ok(false);
        }

        // In production, would verify device exists and destination is valid
        Ok(true)
    }
}

impl Default for MidiRouterState {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri Commands

/// Create a new MIDI route
#[tauri::command]
pub async fn create_route(
    from: String,
    to: String,
    state: State<'_, Arc<RwLock<MidiRouterState>>>,
) -> Result<i32, String> {
    create_route_impl(&from, &to, &*state).await.map_err(|e| e.to_string())
}

pub async fn create_route_impl(
    from: &str,
    to: &str,
    state: &Arc<RwLock<MidiRouterState>>,
) -> Result<i32> {
    let mut router_state = state.write().await;

    // Parse destination
    let destination = parse_destination(to)?;

    let route_id =
        router_state.create_route(from.to_string(), destination, format!("{} → {}", from, to));

    Ok(route_id)
}

/// Parse destination string into RouteDestination
fn parse_destination(dest: &str) -> Result<RouteDestination> {
    if let Some(device_id) = dest.strip_prefix("device:") {
        Ok(RouteDestination::Device(device_id.to_string()))
    } else if let Some(track_id) = dest.strip_prefix("track:") {
        let track_num = track_id.parse::<i32>().context("Invalid track number")?;
        Ok(RouteDestination::Track(track_num))
    } else if let Some(virtual_id) = dest.strip_prefix("virtual:") {
        Ok(RouteDestination::Virtual(virtual_id.to_string()))
    } else {
        // Default to device
        Ok(RouteDestination::Device(dest.to_string()))
    }
}

/// Delete a route
#[tauri::command]
pub async fn delete_route(
    route_id: i32,
    state: State<'_, Arc<RwLock<MidiRouterState>>>,
) -> Result<(), String> {
    delete_route_impl(route_id, &*state).await.map_err(|e| e.to_string())
}

pub async fn delete_route_impl(
    route_id: i32,
    state: &Arc<RwLock<MidiRouterState>>,
) -> Result<()> {
    let mut router_state = state.write().await;
    router_state.delete_route(route_id).context("Failed to delete route")?;
    Ok(())
}

/// Enable a route
#[tauri::command]
pub async fn enable_route(
    route_id: i32,
    state: State<'_, Arc<RwLock<MidiRouterState>>>,
) -> Result<(), String> {
    enable_route_impl(route_id, &*state).await.map_err(|e| e.to_string())
}

pub async fn enable_route_impl(
    route_id: i32,
    state: &Arc<RwLock<MidiRouterState>>,
) -> Result<()> {
    let mut router_state = state.write().await;
    router_state.enable_route(route_id).context("Failed to enable route")?;
    Ok(())
}

/// Disable a route
#[tauri::command]
pub async fn disable_route(
    route_id: i32,
    state: State<'_, Arc<RwLock<MidiRouterState>>>,
) -> Result<(), String> {
    disable_route_impl(route_id, &*state).await.map_err(|e| e.to_string())
}

pub async fn disable_route_impl(
    route_id: i32,
    state: &Arc<RwLock<MidiRouterState>>,
) -> Result<()> {
    let mut router_state = state.write().await;
    router_state.disable_route(route_id).context("Failed to disable route")?;
    Ok(())
}

/// Get all routes
#[tauri::command]
pub async fn get_all_routes(
    state: State<'_, Arc<RwLock<MidiRouterState>>>,
) -> Result<Vec<MidiRoute>, String> {
    get_all_routes_impl(&*state).await.map_err(|e| e.to_string())
}

pub async fn get_all_routes_impl(
    state: &Arc<RwLock<MidiRouterState>>,
) -> Result<Vec<MidiRoute>> {
    let router_state = state.read().await;
    Ok(router_state.get_all_routes())
}

/// Test if route is valid
#[tauri::command]
pub async fn test_route(
    route_id: i32,
    state: State<'_, Arc<RwLock<MidiRouterState>>>,
) -> Result<bool, String> {
    test_route_impl(route_id, &*state).await.map_err(|e| e.to_string())
}

pub async fn test_route_impl(
    route_id: i32,
    state: &Arc<RwLock<MidiRouterState>>,
) -> Result<bool> {
    let router_state = state.read().await;
    router_state.test_route(route_id).context("Failed to test route")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_filter_creation() {
        let filter = MidiFilter::new();

        assert!(filter.channel_filter.is_none());
        assert!(filter.message_type_filter.is_none());
        assert!(filter.note_range.is_none());
        assert!(filter.velocity_range.is_none());
        assert_eq!(filter.transpose, 0);
    }

    #[test]
    fn test_midi_filter_passes() {
        let mut filter = MidiFilter::new();

        // All messages should pass by default
        assert!(filter.passes(0, "note_on", 60, 100));

        // Add channel filter
        filter.channel_filter = Some(vec![0, 1]);
        assert!(filter.passes(0, "note_on", 60, 100));
        assert!(!filter.passes(2, "note_on", 60, 100));

        // Add message type filter
        filter.message_type_filter = Some(vec!["note_on".to_string()]);
        assert!(filter.passes(0, "note_on", 60, 100));
        assert!(!filter.passes(0, "control_change", 60, 100));

        // Add note range filter
        filter.note_range = Some((60, 72));
        assert!(filter.passes(0, "note_on", 60, 100));
        assert!(!filter.passes(0, "note_on", 50, 100));

        // Add velocity range filter
        filter.velocity_range = Some((64, 127));
        assert!(filter.passes(0, "note_on", 60, 100));
        assert!(!filter.passes(0, "note_on", 60, 50));
    }

    #[test]
    fn test_midi_filter_transpose() {
        let mut filter = MidiFilter::new();

        assert_eq!(filter.apply_transpose(60), 60);

        filter.transpose = 12;
        assert_eq!(filter.apply_transpose(60), 72);

        filter.transpose = -12;
        assert_eq!(filter.apply_transpose(60), 48);

        // Test clamping
        filter.transpose = 100;
        assert_eq!(filter.apply_transpose(60), 127);

        filter.transpose = -100;
        assert_eq!(filter.apply_transpose(60), 0);
    }

    #[test]
    fn test_route_destination() {
        let device = RouteDestination::Device("midi-1".to_string());
        assert_eq!(device.id(), "device:midi-1");

        let track = RouteDestination::Track(1);
        assert_eq!(track.id(), "track:1");

        let virtual_dest = RouteDestination::Virtual("virt-1".to_string());
        assert_eq!(virtual_dest.id(), "virtual:virt-1");
    }

    #[test]
    fn test_midi_route_creation() {
        let route = MidiRoute::new(
            1,
            "midi-1".to_string(),
            RouteDestination::Track(1),
            "Test Route".to_string(),
        );

        assert_eq!(route.id, 1);
        assert_eq!(route.from_device, "midi-1");
        assert!(route.enabled);
        assert_eq!(route.name, "Test Route");
    }

    #[test]
    fn test_midi_route_enable_disable() {
        let mut route = MidiRoute::new(
            1,
            "midi-1".to_string(),
            RouteDestination::Track(1),
            "Test".to_string(),
        );

        route.disable();
        assert!(!route.enabled);

        route.enable();
        assert!(route.enabled);
    }

    #[test]
    fn test_router_state_creation() {
        let state = MidiRouterState::new();

        assert!(state.routes.is_empty());
        assert_eq!(state.next_route_id, 1);
    }

    #[test]
    fn test_create_route() {
        let mut state = MidiRouterState::new();

        let route_id = state.create_route(
            "midi-1".to_string(),
            RouteDestination::Track(1),
            "Test Route".to_string(),
        );

        assert_eq!(route_id, 1);
        assert_eq!(state.routes.len(), 1);
        assert_eq!(state.next_route_id, 2);
    }

    #[test]
    fn test_delete_route() {
        let mut state = MidiRouterState::new();

        let route_id = state.create_route(
            "midi-1".to_string(),
            RouteDestination::Track(1),
            "Test".to_string(),
        );

        let result = state.delete_route(route_id);
        assert!(result.is_ok());
        assert!(state.routes.is_empty());
    }

    #[test]
    fn test_delete_nonexistent_route() {
        let mut state = MidiRouterState::new();

        let result = state.delete_route(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_enable_route() {
        let mut state = MidiRouterState::new();

        let route_id = state.create_route(
            "midi-1".to_string(),
            RouteDestination::Track(1),
            "Test".to_string(),
        );

        state.routes.get_mut(&route_id).unwrap().disable();

        let result = state.enable_route(route_id);
        assert!(result.is_ok());
        assert!(state.routes.get(&route_id).unwrap().enabled);
    }

    #[test]
    fn test_disable_route() {
        let mut state = MidiRouterState::new();

        let route_id = state.create_route(
            "midi-1".to_string(),
            RouteDestination::Track(1),
            "Test".to_string(),
        );

        let result = state.disable_route(route_id);
        assert!(result.is_ok());
        assert!(!state.routes.get(&route_id).unwrap().enabled);
    }

    #[test]
    fn test_get_all_routes() {
        let mut state = MidiRouterState::new();

        state.create_route(
            "midi-1".to_string(),
            RouteDestination::Track(1),
            "Route 1".to_string(),
        );
        state.create_route(
            "midi-2".to_string(),
            RouteDestination::Track(2),
            "Route 2".to_string(),
        );

        let routes = state.get_all_routes();
        assert_eq!(routes.len(), 2);
    }

    #[test]
    fn test_get_route() {
        let mut state = MidiRouterState::new();

        let route_id = state.create_route(
            "midi-1".to_string(),
            RouteDestination::Track(1),
            "Test".to_string(),
        );

        let route = state.get_route(route_id);
        assert!(route.is_some());
        assert_eq!(route.unwrap().id, route_id);
    }

    #[test]
    fn test_update_filter() {
        let mut state = MidiRouterState::new();

        let route_id = state.create_route(
            "midi-1".to_string(),
            RouteDestination::Track(1),
            "Test".to_string(),
        );

        let mut filter = MidiFilter::new();
        filter.transpose = 12;

        let result = state.update_filter(route_id, filter);
        assert!(result.is_ok());

        let route = state.get_route(route_id).unwrap();
        assert_eq!(route.filter.transpose, 12);
    }

    #[test]
    fn test_get_active_routes_for_device() {
        let mut state = MidiRouterState::new();

        let route1 = state.create_route(
            "midi-1".to_string(),
            RouteDestination::Track(1),
            "Route 1".to_string(),
        );
        let route2 = state.create_route(
            "midi-1".to_string(),
            RouteDestination::Track(2),
            "Route 2".to_string(),
        );
        let _route3 = state.create_route(
            "midi-2".to_string(),
            RouteDestination::Track(3),
            "Route 3".to_string(),
        );

        state.disable_route(route2).unwrap();

        let active = state.get_active_routes_for_device("midi-1");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, route1);
    }

    #[test]
    fn test_test_route() {
        let mut state = MidiRouterState::new();

        let route_id = state.create_route(
            "midi-1".to_string(),
            RouteDestination::Track(1),
            "Test".to_string(),
        );

        let result = state.test_route(route_id);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_parse_destination() {
        let device = parse_destination("device:midi-1").unwrap();
        assert_eq!(device, RouteDestination::Device("midi-1".to_string()));

        let track = parse_destination("track:1").unwrap();
        assert_eq!(track, RouteDestination::Track(1));

        let virtual_dest = parse_destination("virtual:virt-1").unwrap();
        assert_eq!(
            virtual_dest,
            RouteDestination::Virtual("virt-1".to_string())
        );

        // Default to device
        let default = parse_destination("midi-1").unwrap();
        assert_eq!(default, RouteDestination::Device("midi-1".to_string()));
    }

    #[tokio::test]
    async fn test_create_route_impl() {
        let state = Arc::new(RwLock::new(MidiRouterState::new()));

        let route_id = create_route_impl("midi-1", "track:1", &state).await;
        assert!(route_id.is_ok());

        let s = state.read().await;
        assert_eq!(s.routes.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_route_impl() {
        let state = Arc::new(RwLock::new(MidiRouterState::new()));
        let route_id = {
            let mut s = state.write().await;
            s.create_route(
                "midi-1".to_string(),
                RouteDestination::Track(1),
                "Test".to_string(),
            )
        };

        let result = delete_route_impl(route_id, &state).await;
        assert!(result.is_ok());

        let s = state.read().await;
        assert!(s.routes.is_empty());
    }

    #[tokio::test]
    async fn test_enable_route_impl() {
        let state = Arc::new(RwLock::new(MidiRouterState::new()));
        let route_id = {
            let mut s = state.write().await;
            let id = s.create_route(
                "midi-1".to_string(),
                RouteDestination::Track(1),
                "Test".to_string(),
            );
            s.disable_route(id).unwrap();
            id
        };

        let result = enable_route_impl(route_id, &state).await;
        assert!(result.is_ok());

        let s = state.read().await;
        assert!(s.routes.get(&route_id).unwrap().enabled);
    }

    #[tokio::test]
    async fn test_disable_route_impl() {
        let state = Arc::new(RwLock::new(MidiRouterState::new()));
        let route_id = {
            let mut s = state.write().await;
            s.create_route(
                "midi-1".to_string(),
                RouteDestination::Track(1),
                "Test".to_string(),
            )
        };

        let result = disable_route_impl(route_id, &state).await;
        assert!(result.is_ok());

        let s = state.read().await;
        assert!(!s.routes.get(&route_id).unwrap().enabled);
    }

    #[tokio::test]
    async fn test_get_all_routes_impl() {
        let state = Arc::new(RwLock::new(MidiRouterState::new()));
        {
            let mut s = state.write().await;
            s.create_route(
                "midi-1".to_string(),
                RouteDestination::Track(1),
                "Route 1".to_string(),
            );
            s.create_route(
                "midi-2".to_string(),
                RouteDestination::Track(2),
                "Route 2".to_string(),
            );
        }

        let routes = get_all_routes_impl(&state).await;
        assert!(routes.is_ok());
        assert_eq!(routes.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_test_route_impl() {
        let state = Arc::new(RwLock::new(MidiRouterState::new()));
        let route_id = {
            let mut s = state.write().await;
            s.create_route(
                "midi-1".to_string(),
                RouteDestination::Track(1),
                "Test".to_string(),
            )
        };

        let result = test_route_impl(route_id, &state).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
