use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClickSound {
    Digital,
    WoodBlock,
    Cowbell,
    Beep,
}

impl Default for ClickSound {
    fn default() -> Self {
        ClickSound::Digital
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackSettings {
    pub metronome_enabled: bool,
    pub metronome_volume: f32,
    pub click_sound: ClickSound,
    pub click_on_beat: bool,
    pub click_on_offbeat: bool,
    pub backing_track_volume: f32,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            metronome_enabled: false,
            metronome_volume: 0.7,
            click_sound: ClickSound::default(),
            click_on_beat: true,
            click_on_offbeat: false,
            backing_track_volume: 0.8,
        }
    }
}

impl PlaybackSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metronome(mut self, enabled: bool, volume: f32) -> Self {
        self.metronome_enabled = enabled;
        self.metronome_volume = volume;
        self
    }

    pub fn with_click_sound(mut self, sound: ClickSound) -> Self {
        self.click_sound = sound;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.metronome_volume < 0.0 || self.metronome_volume > 1.0 {
            return Err("Metronome volume must be between 0.0 and 1.0".to_string());
        }

        if self.backing_track_volume < 0.0 || self.backing_track_volume > 1.0 {
            return Err("Backing track volume must be between 0.0 and 1.0".to_string());
        }

        Ok(())
    }

    pub fn set_metronome_enabled(&mut self, enabled: bool) {
        self.metronome_enabled = enabled;
    }

    pub fn set_metronome_volume(&mut self, volume: f32) -> Result<(), String> {
        if volume < 0.0 || volume > 1.0 {
            return Err("Volume must be between 0.0 and 1.0".to_string());
        }
        self.metronome_volume = volume;
        Ok(())
    }

    pub fn set_click_sound(&mut self, sound: ClickSound) {
        self.click_sound = sound;
    }

    pub fn set_click_on_beat(&mut self, enabled: bool) {
        self.click_on_beat = enabled;
    }

    pub fn set_click_on_offbeat(&mut self, enabled: bool) {
        self.click_on_offbeat = enabled;
    }

    pub fn set_backing_track_volume(&mut self, volume: f32) -> Result<(), String> {
        if volume < 0.0 || volume > 1.0 {
            return Err("Volume must be between 0.0 and 1.0".to_string());
        }
        self.backing_track_volume = volume;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_playback_settings() {
        let settings = PlaybackSettings::default();

        assert!(!settings.metronome_enabled);
        assert_eq!(settings.metronome_volume, 0.7);
        assert_eq!(settings.click_sound, ClickSound::Digital);
        assert!(settings.click_on_beat);
        assert!(!settings.click_on_offbeat);
        assert_eq!(settings.backing_track_volume, 0.8);
    }

    #[test]
    fn test_builder_pattern() {
        let settings = PlaybackSettings::new()
            .with_metronome(true, 0.5)
            .with_click_sound(ClickSound::WoodBlock);

        assert!(settings.metronome_enabled);
        assert_eq!(settings.metronome_volume, 0.5);
        assert_eq!(settings.click_sound, ClickSound::WoodBlock);
    }

    #[test]
    fn test_validate_success() {
        let settings = PlaybackSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_metronome_volume_too_low() {
        let mut settings = PlaybackSettings::default();
        settings.metronome_volume = -0.1;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_metronome_volume_too_high() {
        let mut settings = PlaybackSettings::default();
        settings.metronome_volume = 1.1;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_backing_track_volume_too_low() {
        let mut settings = PlaybackSettings::default();
        settings.backing_track_volume = -0.1;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_backing_track_volume_too_high() {
        let mut settings = PlaybackSettings::default();
        settings.backing_track_volume = 1.1;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_set_metronome_enabled() {
        let mut settings = PlaybackSettings::default();
        settings.set_metronome_enabled(true);
        assert!(settings.metronome_enabled);
    }

    #[test]
    fn test_set_metronome_volume_valid() {
        let mut settings = PlaybackSettings::default();
        assert!(settings.set_metronome_volume(0.5).is_ok());
        assert_eq!(settings.metronome_volume, 0.5);
    }

    #[test]
    fn test_set_metronome_volume_invalid() {
        let mut settings = PlaybackSettings::default();
        assert!(settings.set_metronome_volume(1.5).is_err());
        assert!(settings.set_metronome_volume(-0.5).is_err());
    }

    #[test]
    fn test_set_click_sound() {
        let mut settings = PlaybackSettings::default();
        settings.set_click_sound(ClickSound::Cowbell);
        assert_eq!(settings.click_sound, ClickSound::Cowbell);
    }

    #[test]
    fn test_set_click_on_beat() {
        let mut settings = PlaybackSettings::default();
        settings.set_click_on_beat(false);
        assert!(!settings.click_on_beat);
    }

    #[test]
    fn test_set_click_on_offbeat() {
        let mut settings = PlaybackSettings::default();
        settings.set_click_on_offbeat(true);
        assert!(settings.click_on_offbeat);
    }

    #[test]
    fn test_set_backing_track_volume_valid() {
        let mut settings = PlaybackSettings::default();
        assert!(settings.set_backing_track_volume(0.6).is_ok());
        assert_eq!(settings.backing_track_volume, 0.6);
    }

    #[test]
    fn test_set_backing_track_volume_invalid() {
        let mut settings = PlaybackSettings::default();
        assert!(settings.set_backing_track_volume(2.0).is_err());
    }

    #[test]
    fn test_click_sounds() {
        assert_ne!(ClickSound::Digital, ClickSound::WoodBlock);
        assert_ne!(ClickSound::Digital, ClickSound::Cowbell);
        assert_ne!(ClickSound::Digital, ClickSound::Beep);
        assert_eq!(ClickSound::Digital, ClickSound::default());
    }

    #[test]
    fn test_serialization() {
        let settings = PlaybackSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: PlaybackSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.metronome_enabled, deserialized.metronome_enabled);
        assert_eq!(settings.click_sound, deserialized.click_sound);
    }
}
