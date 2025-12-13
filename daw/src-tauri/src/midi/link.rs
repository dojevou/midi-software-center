// =============================================================================
// ABLETON LINK INTEGRATION
// =============================================================================
// Sync with Ableton Live, iOS apps, and other Link-enabled applications.
//
// CLAUDE CODE INSTRUCTIONS:
// 1. Add to Cargo.toml: rusty_link = "0.3"
// 2. Location: daw/src-tauri/src/midi/link.rs
// 3. Link runs on its own network thread for precise timing
//
// FEATURES:
// - Join Link session with other apps
// - Sync tempo and phase
// - Start/stop sync
// - Quantum (beats per phase) configuration
// - Peer count monitoring
// =============================================================================

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::broadcast;

#[cfg(feature = "link")]
use parking_lot::RwLock;

// Note: In production, use the actual rusty_link crate
// This is a compatible interface implementation

/// Link session state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinkState {
    pub enabled: bool,
    pub num_peers: u32,
    pub tempo: f64,
    pub beat: f64,
    pub phase: f64,
    pub is_playing: bool,
    pub quantum: f64,
}

/// Link event for subscribers
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LinkEvent {
    TempoChanged(f64),
    NumPeersChanged(u32),
    StartStopChanged(bool),
    BeatPhase { beat: f64, phase: f64 },
}

/// Ableton Link manager
#[allow(dead_code)]
pub struct AbletonLink {
    enabled: AtomicBool,
    tempo: AtomicU64,      // Stored as tempo * 1000
    quantum: AtomicU64,    // Stored as quantum * 1000
    num_peers: AtomicU64,
    is_playing: AtomicBool,
    start_stop_sync: AtomicBool,

    // Beat tracking
    beat: AtomicU64,       // Stored as beat * 1000000 for precision
    session_start_us: AtomicU64,
    
    // Event broadcasting
    event_tx: broadcast::Sender<LinkEvent>,
    
    // Internal link state (would be rusty_link::AbletonLink in production)
    #[cfg(feature = "link")]
    link: RwLock<Option<rusty_link::AbletonLink>>,
}

impl AbletonLink {
    /// Create new Link instance
    pub fn new(initial_tempo: f64) -> Arc<Self> {
        let (event_tx, _) = broadcast::channel(256);
        
        Arc::new(Self {
            enabled: AtomicBool::new(false),
            tempo: AtomicU64::new((initial_tempo * 1000.0) as u64),
            quantum: AtomicU64::new(4000),  // 4 beats default
            num_peers: AtomicU64::new(0),
            is_playing: AtomicBool::new(false),
            start_stop_sync: AtomicBool::new(true),
            beat: AtomicU64::new(0),
            session_start_us: AtomicU64::new(0),
            event_tx,
            #[cfg(feature = "link")]
            link: RwLock::new(None),
        })
    }

    /// Subscribe to Link events
    pub fn subscribe(&self) -> broadcast::Receiver<LinkEvent> {
        self.event_tx.subscribe()
    }

    /// Enable/disable Link
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
        
        #[cfg(feature = "link")]
        {
            let mut link = self.link.write();
            if enabled {
                if link.is_none() {
                    *link = Some(rusty_link::AbletonLink::new(self.tempo()));
                }
                if let Some(ref l) = *link {
                    l.enable(true);
                }
            } else if let Some(ref l) = *link {
                l.enable(false);
            }
        }
        
        tracing::info!("Ableton Link {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Check if Link is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Get current tempo
    pub fn tempo(&self) -> f64 {
        self.tempo.load(Ordering::Relaxed) as f64 / 1000.0
    }

    /// Set tempo (will be synced to Link session)
    pub fn set_tempo(&self, bpm: f64) {
        let bpm = bpm.clamp(20.0, 999.0);
        self.tempo.store((bpm * 1000.0) as u64, Ordering::Relaxed);
        
        #[cfg(feature = "link")]
        if let Some(ref link) = *self.link.read() {
            let session = link.capture_audio_session_state();
            session.set_tempo(bpm, link.clock_micros());
            link.commit_audio_session_state(session);
        }
        
        let _ = self.event_tx.send(LinkEvent::TempoChanged(bpm));
    }

    /// Get quantum (beats per phase)
    pub fn quantum(&self) -> f64 {
        self.quantum.load(Ordering::Relaxed) as f64 / 1000.0
    }

    /// Set quantum
    pub fn set_quantum(&self, quantum: f64) {
        let quantum = quantum.clamp(1.0, 16.0);
        self.quantum.store((quantum * 1000.0) as u64, Ordering::Relaxed);
    }

    /// Get number of peers in session
    pub fn num_peers(&self) -> u32 {
        self.num_peers.load(Ordering::Relaxed) as u32
    }

    /// Check if transport is playing
    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::Relaxed)
    }

    /// Start transport (with start/stop sync)
    pub fn start(&self) {
        if !self.start_stop_sync.load(Ordering::Relaxed) {
            return;
        }
        
        self.is_playing.store(true, Ordering::Relaxed);
        
        #[cfg(feature = "link")]
        if let Some(ref link) = *self.link.read() {
            let session = link.capture_audio_session_state();
            session.set_is_playing(true, link.clock_micros());
            link.commit_audio_session_state(session);
        }
        
        let _ = self.event_tx.send(LinkEvent::StartStopChanged(true));
    }

    /// Stop transport
    pub fn stop(&self) {
        if !self.start_stop_sync.load(Ordering::Relaxed) {
            return;
        }
        
        self.is_playing.store(false, Ordering::Relaxed);
        
        #[cfg(feature = "link")]
        if let Some(ref link) = *self.link.read() {
            let session = link.capture_audio_session_state();
            session.set_is_playing(false, link.clock_micros());
            link.commit_audio_session_state(session);
        }
        
        let _ = self.event_tx.send(LinkEvent::StartStopChanged(false));
    }

    /// Enable/disable start/stop sync
    pub fn set_start_stop_sync(&self, enabled: bool) {
        self.start_stop_sync.store(enabled, Ordering::Relaxed);
        
        #[cfg(feature = "link")]
        if let Some(ref link) = *self.link.read() {
            link.enable_start_stop_sync(enabled);
        }
    }

    /// Get current beat position
    pub fn beat(&self) -> f64 {
        self.beat.load(Ordering::Relaxed) as f64 / 1000000.0
    }

    /// Get phase within quantum
    pub fn phase(&self) -> f64 {
        let beat = self.beat();
        let quantum = self.quantum();
        beat % quantum
    }

    /// Get complete Link state
    pub fn state(&self) -> LinkState {
        LinkState {
            enabled: self.is_enabled(),
            num_peers: self.num_peers(),
            tempo: self.tempo(),
            beat: self.beat(),
            phase: self.phase(),
            is_playing: self.is_playing(),
            quantum: self.quantum(),
        }
    }

    /// Request beat at specific time (for scheduling)
    #[allow(unused_variables)]
    pub fn request_beat_at_time(&self, beat: f64, time_us: u64, quantum: f64) {
        #[cfg(feature = "link")]
        if let Some(ref link) = *self.link.read() {
            let session = link.capture_audio_session_state();
            session.request_beat_at_time(beat, time_us as i64, quantum);
            link.commit_audio_session_state(session);
        }

        // Update local state
        self.beat.store((beat * 1000000.0) as u64, Ordering::Relaxed);
    }

    /// Force beat at specific time
    #[allow(unused_variables)]
    pub fn force_beat_at_time(&self, beat: f64, time_us: u64, quantum: f64) {
        #[cfg(feature = "link")]
        if let Some(ref link) = *self.link.read() {
            let session = link.capture_audio_session_state();
            session.force_beat_at_time(beat, time_us as i64, quantum);
            link.commit_audio_session_state(session);
        }

        self.beat.store((beat * 1000000.0) as u64, Ordering::Relaxed);
    }

    /// Update from audio thread (call this regularly)
    #[allow(unused_variables)]
    pub fn update(&self, output_latency_us: u64) -> LinkState {
        #[cfg(feature = "link")]
        if let Some(ref link) = *self.link.read() {
            let now = link.clock_micros();
            let session = link.capture_audio_session_state();
            
            // Update stored values
            let tempo = session.tempo();
            let beat = session.beat_at_time(now + output_latency_us as i64, self.quantum());
            let is_playing = session.is_playing();
            let num_peers = link.num_peers();
            
            self.tempo.store((tempo * 1000.0) as u64, Ordering::Relaxed);
            self.beat.store((beat * 1000000.0) as u64, Ordering::Relaxed);
            self.is_playing.store(is_playing, Ordering::Relaxed);
            self.num_peers.store(num_peers as u64, Ordering::Relaxed);
            
            // Send events if changed
            let _ = self.event_tx.send(LinkEvent::BeatPhase { 
                beat, 
                phase: beat % self.quantum() 
            });
        }
        
        // For mock implementation, simulate beat progression
        #[cfg(not(feature = "link"))]
        if self.is_playing.load(Ordering::Relaxed) {
            let tempo = self.tempo();
            let quantum = self.quantum();
            
            // Advance beat based on elapsed time (simplified)
            let current_beat = self.beat.load(Ordering::Relaxed) as f64 / 1000000.0;
            let new_beat = current_beat + tempo / 60.0 / 1000.0;  // ~1ms update rate
            self.beat.store((new_beat * 1000000.0) as u64, Ordering::Relaxed);
            
            let _ = self.event_tx.send(LinkEvent::BeatPhase {
                beat: new_beat,
                phase: new_beat % quantum,
            });
        }
        
        self.state()
    }
}

/// Run Link update loop (call from audio thread or dedicated timer)
pub async fn run_link_engine(link: Arc<AbletonLink>, output_latency_us: u64) {
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(1));
    
    loop {
        interval.tick().await;
        
        if link.is_enabled() {
            link.update(output_latency_us);
        }
    }
}

// =============================================================================
// TAURI COMMANDS
// =============================================================================

use tauri::State;

pub struct LinkStateWrapper(pub Arc<AbletonLink>);

#[tauri::command]
pub fn link_enable(state: State<LinkStateWrapper>, enabled: bool) {
    state.0.set_enabled(enabled);
}

#[tauri::command]
pub fn link_is_enabled(state: State<LinkStateWrapper>) -> bool {
    state.0.is_enabled()
}

#[tauri::command]
pub fn link_set_tempo(state: State<LinkStateWrapper>, bpm: f64) {
    state.0.set_tempo(bpm);
}

#[tauri::command]
pub fn link_get_tempo(state: State<LinkStateWrapper>) -> f64 {
    state.0.tempo()
}

#[tauri::command]
pub fn link_set_quantum(state: State<LinkStateWrapper>, quantum: f64) {
    state.0.set_quantum(quantum);
}

#[tauri::command]
pub fn link_start(state: State<LinkStateWrapper>) {
    state.0.start();
}

#[tauri::command]
pub fn link_stop(state: State<LinkStateWrapper>) {
    state.0.stop();
}

#[tauri::command]
pub fn link_get_state(state: State<LinkStateWrapper>) -> LinkState {
    state.0.state()
}

#[tauri::command]
pub fn link_num_peers(state: State<LinkStateWrapper>) -> u32 {
    state.0.num_peers()
}

#[tauri::command]
pub fn link_set_start_stop_sync(state: State<LinkStateWrapper>, enabled: bool) {
    state.0.set_start_stop_sync(enabled);
}

#[tauri::command]
pub fn link_get_beat(state: State<LinkStateWrapper>) -> f64 {
    state.0.beat()
}

#[tauri::command]
pub fn link_get_phase(state: State<LinkStateWrapper>) -> f64 {
    state.0.phase()
}

#[tauri::command]
pub fn link_request_beat_at_time(
    state: State<LinkStateWrapper>,
    beat: f64,
    time_us: u64,
    quantum: f64,
) {
    state.0.request_beat_at_time(beat, time_us, quantum);
}

#[tauri::command]
pub fn link_force_beat_at_time(
    state: State<LinkStateWrapper>,
    beat: f64,
    time_us: u64,
    quantum: f64,
) {
    state.0.force_beat_at_time(beat, time_us, quantum);
}

#[tauri::command]
pub fn link_update(state: State<LinkStateWrapper>, output_latency_us: u64) -> LinkState {
    state.0.update(output_latency_us)
}

/// Subscribe to Link events via Tauri events
#[tauri::command]
pub async fn link_subscribe(
    app: tauri::AppHandle,
    state: State<'_, LinkStateWrapper>,
) -> Result<(), String> {
    use tauri::Emitter;

    let mut rx = state.0.subscribe();

    // Spawn a task to forward Link events to the frontend
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let event_data = match event {
                LinkEvent::TempoChanged(tempo) => {
                    serde_json::json!({"type": "tempo", "value": tempo})
                }
                LinkEvent::NumPeersChanged(peers) => {
                    serde_json::json!({"type": "peers", "value": peers})
                }
                LinkEvent::StartStopChanged(playing) => {
                    serde_json::json!({"type": "playing", "value": playing})
                }
                LinkEvent::BeatPhase { beat, phase } => {
                    serde_json::json!({"type": "beat", "beat": beat, "phase": phase})
                }
            };

            if app.emit("link-event", event_data).is_err() {
                break;
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_creation() {
        let link = AbletonLink::new(120.0);
        assert_eq!(link.tempo(), 120.0);
    }

    #[test]
    fn test_tempo_change() {
        let link = AbletonLink::new(120.0);
        link.set_tempo(140.0);
        assert_eq!(link.tempo(), 140.0);
    }

    #[test]
    fn test_quantum() {
        let link = AbletonLink::new(120.0);
        assert_eq!(link.quantum(), 4.0);
        link.set_quantum(8.0);
        assert_eq!(link.quantum(), 8.0);
    }
}
