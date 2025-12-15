use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum MeteringMode {
    Peak,
    Rms,
    #[default]
    Both,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum FaderType {
    Linear,
    #[default]
    Exponential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerSettings {
    pub metering_mode: MeteringMode,
    pub fader_type: FaderType,
    pub master_level_db: f32,
    pub clip_threshold_db: f32,
}

impl Default for MixerSettings {
    fn default() -> Self {
        Self {
            metering_mode: MeteringMode::default(),
            fader_type: FaderType::default(),
            master_level_db: 0.0,
            clip_threshold_db: -0.1,
        }
    }
}

impl MixerSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_metering_mode(mut self, mode: MeteringMode) -> Self {
        self.metering_mode = mode;
        self
    }

    pub fn with_fader_type(mut self, fader_type: FaderType) -> Self {
        self.fader_type = fader_type;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.master_level_db < -60.0 || self.master_level_db > 12.0 {
            return Err("Master level must be between -60.0 and 12.0 dB".to_string());
        }

        if self.clip_threshold_db < -12.0 || self.clip_threshold_db > 0.0 {
            return Err("Clip threshold must be between -12.0 and 0.0 dB".to_string());
        }

        Ok(())
    }

    pub fn set_metering_mode(&mut self, mode: MeteringMode) {
        self.metering_mode = mode;
    }

    pub fn set_fader_type(&mut self, fader_type: FaderType) {
        self.fader_type = fader_type;
    }

    pub fn set_master_level(&mut self, level_db: f32) -> Result<(), String> {
        if !(-60.0..=12.0).contains(&level_db) {
            return Err("Master level must be between -60.0 and 12.0 dB".to_string());
        }
        self.master_level_db = level_db;
        Ok(())
    }

    pub fn set_clip_threshold(&mut self, threshold_db: f32) -> Result<(), String> {
        if !(-12.0..=0.0).contains(&threshold_db) {
            return Err("Clip threshold must be between -12.0 and 0.0 dB".to_string());
        }
        self.clip_threshold_db = threshold_db;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mixer_settings() {
        let settings = MixerSettings::default();

        assert_eq!(settings.metering_mode, MeteringMode::Both);
        assert_eq!(settings.fader_type, FaderType::Exponential);
        assert_eq!(settings.master_level_db, 0.0);
        assert_eq!(settings.clip_threshold_db, -0.1);
    }

    #[test]
    fn test_builder_pattern() {
        let settings = MixerSettings::new()
            .with_metering_mode(MeteringMode::Peak)
            .with_fader_type(FaderType::Linear);

        assert_eq!(settings.metering_mode, MeteringMode::Peak);
        assert_eq!(settings.fader_type, FaderType::Linear);
    }

    #[test]
    fn test_validate_success() {
        let settings = MixerSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_master_level_too_low() {
        let mut settings = MixerSettings::default();
        settings.master_level_db = -61.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_master_level_too_high() {
        let mut settings = MixerSettings::default();
        settings.master_level_db = 13.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_clip_threshold_too_low() {
        let mut settings = MixerSettings::default();
        settings.clip_threshold_db = -13.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_clip_threshold_too_high() {
        let mut settings = MixerSettings::default();
        settings.clip_threshold_db = 0.1;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_set_metering_mode() {
        let mut settings = MixerSettings::default();
        settings.set_metering_mode(MeteringMode::Rms);
        assert_eq!(settings.metering_mode, MeteringMode::Rms);
    }

    #[test]
    fn test_set_fader_type() {
        let mut settings = MixerSettings::default();
        settings.set_fader_type(FaderType::Linear);
        assert_eq!(settings.fader_type, FaderType::Linear);
    }

    #[test]
    fn test_set_master_level_valid() {
        let mut settings = MixerSettings::default();
        assert!(settings.set_master_level(-6.0).is_ok());
        assert_eq!(settings.master_level_db, -6.0);
    }

    #[test]
    fn test_set_master_level_invalid() {
        let mut settings = MixerSettings::default();
        assert!(settings.set_master_level(-70.0).is_err());
        assert!(settings.set_master_level(15.0).is_err());
    }

    #[test]
    fn test_set_clip_threshold_valid() {
        let mut settings = MixerSettings::default();
        assert!(settings.set_clip_threshold(-3.0).is_ok());
        assert_eq!(settings.clip_threshold_db, -3.0);
    }

    #[test]
    fn test_set_clip_threshold_invalid() {
        let mut settings = MixerSettings::default();
        assert!(settings.set_clip_threshold(-15.0).is_err());
        assert!(settings.set_clip_threshold(1.0).is_err());
    }

    #[test]
    fn test_metering_modes() {
        assert_ne!(MeteringMode::Peak, MeteringMode::Rms);
        assert_ne!(MeteringMode::Peak, MeteringMode::Both);
        assert_eq!(MeteringMode::Both, MeteringMode::default());
    }

    #[test]
    fn test_fader_types() {
        assert_ne!(FaderType::Linear, FaderType::Exponential);
        assert_eq!(FaderType::Exponential, FaderType::default());
    }

    #[test]
    fn test_serialization() {
        let settings = MixerSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: MixerSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.metering_mode, deserialized.metering_mode);
        assert_eq!(settings.fader_type, deserialized.fader_type);
    }
}
