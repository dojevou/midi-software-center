/// DAW window state structures
///
/// Defines state for DAW windows including transport controls, playback position,
/// track information, and mixer state.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Playback state for the transport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum PlaybackState {
    /// Transport is stopped, position at start or last stop point
    #[default]
    Stopped,
    /// Transport is actively playing
    Playing,
    /// Transport is paused, position held
    Paused,
    /// Transport is recording MIDI input
    Recording,
}

impl PlaybackState {
    /// Check if transport is currently playing or recording
    pub fn is_active(&self) -> bool {
        matches!(self, PlaybackState::Playing | PlaybackState::Recording)
    }

    /// Check if can be paused
    pub fn can_pause(&self) -> bool {
        matches!(self, PlaybackState::Playing | PlaybackState::Recording)
    }

    /// Check if can be resumed
    pub fn can_resume(&self) -> bool {
        *self == PlaybackState::Paused
    }
}

/// Playback position in musical time
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackPosition {
    /// Current bar number (1-based)
    pub bar: i32,
    /// Current beat within bar (1-based)
    pub beat: i32,
    /// Current tick within beat (0-based)
    pub tick: i32,
    /// Total ticks from start
    pub total_ticks: u64,
}

impl Default for PlaybackPosition {
    fn default() -> Self {
        PlaybackPosition { bar: 1, beat: 1, tick: 0, total_ticks: 0 }
    }
}

impl PlaybackPosition {
    /// Create new position at bar/beat/tick
    pub fn new(bar: i32, beat: i32, tick: i32) -> Self {
        PlaybackPosition {
            bar,
            beat,
            tick,
            total_ticks: 0, // Calculated by sequencer
        }
    }

    /// Create position from total ticks
    pub fn from_ticks(total_ticks: u64, ticks_per_quarter: u16, beats_per_bar: u8) -> Self {
        let tpq = ticks_per_quarter as u64;
        let bpb = beats_per_bar as u64;

        let ticks_per_bar = tpq * bpb;
        let bar = (total_ticks / ticks_per_bar) as i32 + 1;

        let remaining_ticks = total_ticks % ticks_per_bar;
        let beat = (remaining_ticks / tpq) as i32 + 1;

        let tick = (remaining_ticks % tpq) as i32;

        PlaybackPosition { bar, beat, tick, total_ticks }
    }

    /// Check if position is valid
    pub fn is_valid(&self) -> bool {
        self.bar >= 1 && self.beat >= 1 && self.tick >= 0
    }
}

/// Transport control information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportInfo {
    /// Current tempo in BPM
    pub bpm: f32,
    /// Time signature numerator (beats per bar)
    pub time_signature_numerator: u8,
    /// Time signature denominator (note value)
    pub time_signature_denominator: u8,
    /// Key signature (e.g., "C", "Dm", "F#")
    pub key_signature: String,
    /// Current playback position
    pub position: PlaybackPosition,
    /// Ticks per quarter note (MIDI resolution)
    pub ticks_per_quarter: u16,
    /// Loop enabled
    pub loop_enabled: bool,
    /// Loop start position (in ticks)
    pub loop_start: u64,
    /// Loop end position (in ticks)
    pub loop_end: u64,
}

impl Default for TransportInfo {
    fn default() -> Self {
        TransportInfo {
            bpm: 120.0,
            time_signature_numerator: 4,
            time_signature_denominator: 4,
            key_signature: "C".to_string(),
            position: PlaybackPosition::default(),
            ticks_per_quarter: 480,
            loop_enabled: false,
            loop_start: 0,
            loop_end: 0,
        }
    }
}

impl TransportInfo {
    /// Create new transport with custom BPM
    pub fn with_bpm(mut self, bpm: f32) -> Self {
        self.bpm = bpm;
        self
    }

    /// Set time signature
    pub fn with_time_signature(mut self, numerator: u8, denominator: u8) -> Self {
        self.time_signature_numerator = numerator;
        self.time_signature_denominator = denominator;
        self
    }

    /// Set key signature
    pub fn with_key_signature(mut self, key: String) -> Self {
        self.key_signature = key;
        self
    }

    /// Validate BPM is in reasonable range
    pub fn is_bpm_valid(&self) -> bool {
        self.bpm >= 20.0 && self.bpm <= 999.0
    }

    /// Validate time signature
    pub fn is_time_signature_valid(&self) -> bool {
        self.time_signature_numerator >= 1
            && self.time_signature_numerator <= 32
            && matches!(self.time_signature_denominator, 1 | 2 | 4 | 8 | 16 | 32)
    }
}

/// Track information for DAW tracks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    /// Unique track ID
    pub id: i32,
    /// Track display label
    pub label: String,
    /// Track is visible in UI
    pub visible: bool,
    /// Track is muted (no audio output)
    pub muted: bool,
    /// Track is soloed (only solo tracks play)
    pub soloed: bool,
    /// Track color (hex string)
    pub color: String,
    /// Track height in pixels
    pub height: u32,
    /// MIDI channel (1-16)
    pub midi_channel: u8,
    /// Number of MIDI events on track
    pub event_count: usize,
}

impl TrackInfo {
    /// Create new track with ID and label
    pub fn new(id: i32, label: &str) -> Self {
        TrackInfo {
            id,
            label: label.to_string(),
            visible: true,
            muted: false,
            soloed: false,
            color: "#3B82F6".to_string(), // Default blue
            height: 120,
            midi_channel: 1,
            event_count: 0,
        }
    }

    /// Create with custom MIDI channel
    pub fn with_midi_channel(mut self, channel: u8) -> Self {
        self.midi_channel = channel.clamp(1, 16);
        self
    }

    /// Create with custom color
    pub fn with_color(mut self, color: String) -> Self {
        self.color = color;
        self
    }

    /// Check if track should play (not muted and either not soloed or is soloed)
    pub fn should_play(&self, any_solo: bool) -> bool {
        !self.muted && (!any_solo || self.soloed)
    }

    /// Check if MIDI channel is valid
    pub fn is_midi_channel_valid(&self) -> bool {
        self.midi_channel >= 1 && self.midi_channel <= 16
    }
}

/// Main DAW window state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAWWindowState {
    /// Current playback state
    pub playback_state: PlaybackState,
    /// Transport controls
    pub transport: TransportInfo,
    /// All tracks in the project
    pub tracks: HashMap<i32, TrackInfo>,
    /// Next available track ID
    pub next_track_id: i32,
    /// Selected track IDs
    pub selected_tracks: Vec<i32>,
    /// Zoom level (pixels per quarter note)
    pub zoom_level: f32,
    /// Scroll position (in ticks)
    pub scroll_position: u64,
}

impl Default for DAWWindowState {
    fn default() -> Self {
        DAWWindowState {
            playback_state: PlaybackState::Stopped,
            transport: TransportInfo::default(),
            tracks: HashMap::new(),
            next_track_id: 1,
            selected_tracks: Vec::new(),
            zoom_level: 4.0,
            scroll_position: 0,
        }
    }
}

impl DAWWindowState {
    /// Create new DAW state
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new track
    pub fn add_track(&mut self, label: String) -> i32 {
        let track_id = self.next_track_id;
        let track = TrackInfo::new(track_id, &label);
        self.tracks.insert(track_id, track);
        self.next_track_id += 1;
        track_id
    }

    /// Remove a track
    pub fn remove_track(&mut self, track_id: i32) -> Option<TrackInfo> {
        self.selected_tracks.retain(|&id| id != track_id);
        self.tracks.remove(&track_id)
    }

    /// Get track by ID
    pub fn get_track(&self, track_id: i32) -> Option<&TrackInfo> {
        self.tracks.get(&track_id)
    }

    /// Get mutable track by ID
    pub fn get_track_mut(&mut self, track_id: i32) -> Option<&mut TrackInfo> {
        self.tracks.get_mut(&track_id)
    }

    /// Get all tracks as sorted vector
    pub fn get_all_tracks(&self) -> Vec<TrackInfo> {
        let mut tracks: Vec<_> = self.tracks.values().cloned().collect();
        tracks.sort_by_key(|t| t.id);
        tracks
    }

    /// Check if any tracks are soloed
    pub fn has_soloed_tracks(&self) -> bool {
        self.tracks.values().any(|t| t.soloed)
    }

    /// Get track count
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }
}

/// Channel type for mixer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelType {
    /// Regular MIDI track channel
    Track,
    /// Master output channel
    Master,
    /// Auxiliary send/return channel
    Aux,
}

/// Mixer channel information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerChannel {
    /// Channel ID (matches track ID for track channels)
    pub id: i32,
    /// Channel type
    pub channel_type: ChannelType,
    /// Channel label
    pub label: String,
    /// Volume level (0.0 to 1.0)
    pub volume: f32,
    /// Pan position (-1.0 left to 1.0 right)
    pub pan: f32,
    /// Channel is muted
    pub muted: bool,
    /// Channel is soloed
    pub soloed: bool,
    /// Meter level (current audio level 0.0 to 1.0)
    pub meter_level: f32,
}

impl MixerChannel {
    /// Create new mixer channel
    pub fn new(id: i32, channel_type: ChannelType, label: &str) -> Self {
        MixerChannel {
            id,
            channel_type,
            label: label.to_string(),
            volume: 0.8,
            pan: 0.0,
            muted: false,
            soloed: false,
            meter_level: 0.0,
        }
    }

    /// Create track channel
    pub fn track(id: i32, label: &str) -> Self {
        Self::new(id, ChannelType::Track, label)
    }

    /// Create master channel
    pub fn master() -> Self {
        Self::new(-1, ChannelType::Master, "Master")
    }

    /// Set volume (clamped to 0.0-1.0)
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Set pan (clamped to -1.0 to 1.0)
    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    /// Check if channel is valid
    pub fn is_valid(&self) -> bool {
        self.volume >= 0.0 && self.volume <= 1.0 && self.pan >= -1.0 && self.pan <= 1.0
    }
}

/// Mixer window state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerWindowState {
    /// All mixer channels
    pub channels: HashMap<i32, MixerChannel>,
    /// Master channel
    pub master: MixerChannel,
    /// Show meter
    pub show_meters: bool,
    /// Show effects
    pub show_effects: bool,
}

impl Default for MixerWindowState {
    fn default() -> Self {
        MixerWindowState {
            channels: HashMap::new(),
            master: MixerChannel::master(),
            show_meters: true,
            show_effects: true,
        }
    }
}

impl MixerWindowState {
    /// Create new mixer state
    pub fn new() -> Self {
        Self::default()
    }

    /// Add channel from track
    pub fn add_channel_from_track(&mut self, track: &TrackInfo) {
        let channel = MixerChannel::track(track.id, &track.label);
        self.channels.insert(track.id, channel);
    }

    /// Remove channel
    pub fn remove_channel(&mut self, channel_id: i32) -> Option<MixerChannel> {
        self.channels.remove(&channel_id)
    }

    /// Get channel by ID
    pub fn get_channel(&self, channel_id: i32) -> Option<&MixerChannel> {
        self.channels.get(&channel_id)
    }

    /// Get mutable channel by ID
    pub fn get_channel_mut(&mut self, channel_id: i32) -> Option<&mut MixerChannel> {
        self.channels.get_mut(&channel_id)
    }

    /// Get all channels as sorted vector
    pub fn get_all_channels(&self) -> Vec<MixerChannel> {
        let mut channels: Vec<_> = self.channels.values().cloned().collect();
        channels.sort_by_key(|c| c.id);
        channels
    }

    /// Sync channels with tracks
    pub fn sync_with_tracks(&mut self, tracks: &HashMap<i32, TrackInfo>) {
        // Remove channels for deleted tracks
        self.channels.retain(|id, _| tracks.contains_key(id));

        // Add channels for new tracks
        for track in tracks.values() {
            if !self.channels.contains_key(&track.id) {
                self.add_channel_from_track(track);
            } else if let Some(channel) = self.channels.get_mut(&track.id) {
                // Update label if changed
                channel.label = track.label.clone();
                // Sync mute/solo state
                channel.muted = track.muted;
                channel.soloed = track.soloed;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_state_active() {
        assert!(PlaybackState::Playing.is_active());
        assert!(PlaybackState::Recording.is_active());
        assert!(!PlaybackState::Stopped.is_active());
        assert!(!PlaybackState::Paused.is_active());
    }

    #[test]
    fn test_playback_position_from_ticks() {
        // 480 ticks per quarter, 4 beats per bar
        let pos = PlaybackPosition::from_ticks(1920, 480, 4);
        assert_eq!(pos.bar, 2); // 1920 ticks = 1 full bar + start of bar 2
        assert_eq!(pos.beat, 1);
        assert_eq!(pos.tick, 0);
    }

    #[test]
    fn test_transport_validation() {
        let mut transport = TransportInfo::default();
        assert!(transport.is_bpm_valid());
        assert!(transport.is_time_signature_valid());

        transport.bpm = 10.0;
        assert!(!transport.is_bpm_valid());

        transport.bpm = 120.0;
        transport.time_signature_denominator = 3;
        assert!(!transport.is_time_signature_valid());
    }

    #[test]
    fn test_track_info_should_play() {
        let mut track = TrackInfo::new(1, "Test");

        // Normal playback
        assert!(track.should_play(false));

        // Muted
        track.muted = true;
        assert!(!track.should_play(false));

        // Soloed (not this track)
        track.muted = false;
        assert!(!track.should_play(true));

        // Soloed (this track)
        track.soloed = true;
        assert!(track.should_play(true));
    }

    #[test]
    fn test_daw_state_track_operations() {
        let mut state = DAWWindowState::new();

        let id = state.add_track("Piano".to_string());
        assert_eq!(id, 1);
        assert_eq!(state.track_count(), 1);

        let track = state.get_track(id).unwrap();
        assert_eq!(track.label, "Piano");

        state.remove_track(id);
        assert_eq!(state.track_count(), 0);
    }

    #[test]
    fn test_mixer_channel_validation() {
        let mut channel = MixerChannel::track(1, "Test");
        assert!(channel.is_valid());

        channel.set_volume(1.5);
        assert_eq!(channel.volume, 1.0);

        channel.set_pan(-2.0);
        assert_eq!(channel.pan, -1.0);
    }

    #[test]
    fn test_mixer_sync_with_tracks() {
        let mut mixer = MixerWindowState::new();
        let mut tracks = HashMap::new();

        let track = TrackInfo::new(1, "Piano");
        tracks.insert(1, track);

        mixer.sync_with_tracks(&tracks);
        assert_eq!(mixer.channels.len(), 1);

        tracks.remove(&1);
        mixer.sync_with_tracks(&tracks);
        assert_eq!(mixer.channels.len(), 0);
    }
}
