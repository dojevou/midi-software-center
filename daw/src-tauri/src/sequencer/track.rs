   /// Track management for sequencer
   ///
   /// Grown-up Script: Manages collection of tracks with their properties and MIDI events.

use crate::models::sequencer::{Track, TrackProperties};
use crate::models::midi::MidiEvent;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages all tracks in the sequencer
///
/// Tracks contain MIDI events and playback properties (mute, solo, volume).
/// Thread-safe using RwLock for concurrent read access.
pub struct TrackManager {
    tracks: Arc<RwLock<HashMap<i32, Track>>>,
    next_id: Arc<RwLock<i32>>,
}

impl TrackManager {
    /// Create a new empty track manager
    pub fn new() -> Self {
        Self {
            tracks: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Add a new track with specified properties
    ///
    /// # Arguments
    /// * `file_id` - Database ID of the MIDI file for this track
    /// * `channel` - MIDI channel (0-15)
    /// * `events` - MIDI events for this track
    ///
    /// # Returns
    /// The newly created Track with assigned ID
    pub async fn add_track(
        &self,
        file_id: i32,
        channel: u8,
        events: Vec<MidiEvent>,
    ) -> Result<Track, String> {
        if channel > 15 {
            return Err(format!("Invalid MIDI channel: {}. Must be 0-15", channel));
        }

        let mut next_id = self.next_id.write().await;
        let track_id = *next_id;
        *next_id += 1;

        let track = Track {
            id: track_id,
            name: format!("Track {}", track_id),
            file_id,
            channel,
            muted: false,
            solo: false,
            volume: 100,
            pan: 64,
            color: "#888888".to_string(),
            events,
        };

        let mut tracks = self.tracks.write().await;
        tracks.insert(track_id, track.clone());

        Ok(track)
    }

    /// Remove a track by ID
    ///
    /// # Returns
    /// Ok(()) if track was removed, Err if track not found
    pub async fn remove_track(&self, track_id: i32) -> Result<(), String> {
        let mut tracks = self.tracks.write().await;
        tracks
            .remove(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;
        Ok(())
    }

    /// Update track properties (mute, solo, volume, pan)
    pub async fn update_track(
        &self,
        track_id: i32,
        properties: TrackProperties,
    ) -> Result<(), String> {
        let mut tracks = self.tracks.write().await;
        let track = tracks
            .get_mut(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;

        if let Some(muted) = properties.muted {
            track.muted = muted;
        }
        if let Some(solo) = properties.solo {
            track.solo = solo;
        }
        if let Some(volume) = properties.volume {
            if volume > 127 {
                return Err("Volume must be 0-127".to_string());
            }
            track.volume = volume;
        }
        if let Some(pan) = properties.pan {
            if pan > 127 {
                return Err("Pan must be 0-127".to_string());
            }
            track.pan = pan;
        }

        Ok(())
    }

    /// Get all tracks
    pub async fn get_tracks(&self) -> Vec<Track> {
        let tracks = self.tracks.read().await;
        tracks.values().cloned().collect()
    }

    /// Get a specific track by ID
    pub async fn get_track(&self, track_id: i32) -> Option<Track> {
        let tracks = self.tracks.read().await;
        tracks.get(&track_id).cloned()
    }

    /// Check if any track has solo enabled
    pub async fn has_solo(&self) -> bool {
        let tracks = self.tracks.read().await;
        tracks.values().any(|t| t.solo)
    }

    /// Get tracks that should play (considering mute/solo)
    ///
    /// Logic:
    /// - If any track is solo, only solo tracks play
    /// - Otherwise, all non-muted tracks play
    pub async fn get_active_tracks(&self) -> Vec<Track> {
        let tracks = self.tracks.read().await;
        let has_solo = tracks.values().any(|t| t.solo);

        tracks
            .values()
            .filter(|t| {
                if has_solo {
                    t.solo
                } else {
                    !t.muted
                }
            })
            .cloned()
            .collect()
    }

    /// Clear all tracks
    pub async fn clear(&self) {
        let mut tracks = self.tracks.write().await;
        tracks.clear();
        let mut next_id = self.next_id.write().await;
        *next_id = 1;
    }
}

impl Default for TrackManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(channel: u8) -> MidiEvent {
        use crate::models::midi::MidiEventType;

        MidiEvent {
            event_type: MidiEventType::NoteOn,
            tick: 0,
            channel,
            note: Some(60),
            velocity: Some(100),
            controller: None,
            value: None,
            program: None,
        }
    }

    #[tokio::test]
    async fn test_add_track() {
        let manager = TrackManager::new();
        let events = vec![create_test_event(0)];

        let track = manager.add_track(1, 0, events).await.unwrap();
        assert_eq!(track.id, 1);
        assert_eq!(track.file_id, 1);
        assert_eq!(track.channel, 0);
        assert!(!track.muted);
        assert!(!track.solo);
    }

    #[tokio::test]
    async fn test_add_track_invalid_channel() {
        let manager = TrackManager::new();
        let events = vec![create_test_event(0)];

        let result = manager.add_track(1, 16, events).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid MIDI channel"));
    }

    #[tokio::test]
    async fn test_remove_track() {
        let manager = TrackManager::new();
        let events = vec![create_test_event(0)];

        let track = manager.add_track(1, 0, events).await.unwrap();
        assert!(manager.remove_track(track.id).await.is_ok());
        assert!(manager.get_track(track.id).await.is_none());
    }

    #[tokio::test]
    async fn test_update_track() {
        let manager = TrackManager::new();
        let events = vec![create_test_event(0)];

        let track = manager.add_track(1, 0, events).await.unwrap();

        let props = TrackProperties {
            muted: Some(true),
            solo: Some(true),
            volume: Some(80),
            pan: Some(32),
        };

        manager.update_track(track.id, props).await.unwrap();

        let updated = manager.get_track(track.id).await.unwrap();
        assert!(updated.muted);
        assert!(updated.solo);
        assert_eq!(updated.volume, 80);
        assert_eq!(updated.pan, 32);
    }

    #[tokio::test]
    async fn test_update_track_invalid_volume() {
        let manager = TrackManager::new();
        let events = vec![create_test_event(0)];

        let track = manager.add_track(1, 0, events).await.unwrap();

        let props = TrackProperties {
            muted: None,
            solo: None,
            volume: Some(128),
            pan: None,
        };

        let result = manager.update_track(track.id, props).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Volume must be 0-127"));
    }

    #[tokio::test]
    async fn test_get_active_tracks_no_solo() {
        let manager = TrackManager::new();

        // Add 3 tracks: 1 muted, 2 unmuted
        manager.add_track(1, 0, vec![create_test_event(0)]).await.unwrap();
        let track2 = manager.add_track(2, 1, vec![create_test_event(1)]).await.unwrap();
        manager.add_track(3, 2, vec![create_test_event(2)]).await.unwrap();

        // Mute track 2
        manager.update_track(track2.id, TrackProperties {
            muted: Some(true),
            solo: None,
            volume: None,
            pan: None,
        }).await.unwrap();

        let active = manager.get_active_tracks().await;
        assert_eq!(active.len(), 2); // Only unmuted tracks
    }

    #[tokio::test]
    async fn test_get_active_tracks_with_solo() {
        let manager = TrackManager::new();

        // Add 3 tracks
        manager.add_track(1, 0, vec![create_test_event(0)]).await.unwrap();
        let track2 = manager.add_track(2, 1, vec![create_test_event(1)]).await.unwrap();
        manager.add_track(3, 2, vec![create_test_event(2)]).await.unwrap();

        // Solo track 2
        manager.update_track(track2.id, TrackProperties {
            muted: None,
            solo: Some(true),
            volume: None,
            pan: None,
        }).await.unwrap();

        let active = manager.get_active_tracks().await;
        assert_eq!(active.len(), 1); // Only solo track
        assert_eq!(active[0].id, track2.id);
    }

    #[tokio::test]
    async fn test_clear() {
        let manager = TrackManager::new();

        manager.add_track(1, 0, vec![create_test_event(0)]).await.unwrap();
        manager.add_track(2, 1, vec![create_test_event(1)]).await.unwrap();

        manager.clear().await;

        let tracks = manager.get_tracks().await;
        assert_eq!(tracks.len(), 0);

        // Next track should start at ID 1 again
        let track = manager.add_track(3, 0, vec![create_test_event(0)]).await.unwrap();
        assert_eq!(track.id, 1);
    }
}
