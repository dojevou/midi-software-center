   /// Tauri mocking framework for testing IPC commands
   /// Captures event emissions and provides assertions

use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Serialize;

/// Events emitted during command execution
#[derive(Debug, Clone, PartialEq)]
pub struct EmittedEvent {
    pub event_name: String,
    pub payload: String, // JSON serialized
}

/// Mock Tauri Window for testing event emission
#[derive(Clone)]
pub struct MockWindow {
    emitted_events: Arc<Mutex<Vec<EmittedEvent>>>,
}

impl MockWindow {
    pub fn new() -> Self {
        Self {
            emitted_events: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Mock emit method (matches Tauri signature)
    pub async fn emit<S: Serialize>(&self, event: &str, payload: S) -> Result<(), String> {
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| format!("Failed to serialize payload: {}", e))?;
        
        self.emitted_events.lock().await.push(EmittedEvent {
            event_name: event.to_string(),
            payload: payload_json,
        });
        
        Ok(())
    }
    
    /// Get all emitted events (for assertions)
    pub async fn get_emitted_events(&self) -> Vec<EmittedEvent> {
        self.emitted_events.lock().await.clone()
    }
    
    /// Assert event was emitted
    pub async fn assert_event_emitted(&self, event_name: &str) {
        let events = self.get_emitted_events().await;
        assert!(
            events.iter().any(|e| e.event_name == event_name),
            "Expected event '{}' not emitted. Emitted events: {:?}",
            event_name,
            events.iter().map(|e| &e.event_name).collect::<Vec<_>>()
        );
    }
    
    /// Assert event count
    pub async fn assert_event_count(&self, event_name: &str, expected_count: usize) {
        let events = self.get_emitted_events().await;
        let actual_count = events.iter().filter(|e| e.event_name == event_name).count();
        assert_eq!(
            actual_count, expected_count,
            "Expected {} '{}' events, got {}",
            expected_count, event_name, actual_count
        );
    }
    
    /// Clear all events (for reset between tests)
    pub async fn clear_events(&self) {
        self.emitted_events.lock().await.clear();
    }
}

impl Default for MockWindow {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock AppHandle (minimal, can be extended)
pub struct MockAppHandle {
    pub window: MockWindow,
}

impl MockAppHandle {
    pub fn new() -> Self {
        Self {
            window: MockWindow::new(),
        }
    }
}

impl Default for MockAppHandle {
    fn default() -> Self {
        Self::new()
    }
}
