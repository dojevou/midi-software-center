use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrackColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for TrackColor {
    fn default() -> Self {
        Self { r: 100, g: 150, b: 200 } // Light blue
    }
}

impl TrackColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err("Hex color must be 6 characters".to_string());
        }

        let r =
            u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid red component".to_string())?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| "Invalid green component".to_string())?;
        let b =
            u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid blue component".to_string())?;

        Ok(Self { r, g, b })
    }

    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackSettings {
    pub default_track_color: TrackColor,
    pub default_track_volume: f32,
    pub default_track_pan: f32,
    pub auto_arm_on_selection: bool,
}

impl Default for TrackSettings {
    fn default() -> Self {
        Self {
            default_track_color: TrackColor::default(),
            default_track_volume: 0.8,
            default_track_pan: 0.0,
            auto_arm_on_selection: false,
        }
    }
}

impl TrackSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_color(mut self, color: TrackColor) -> Self {
        self.default_track_color = color;
        self
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.default_track_volume = volume;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.default_track_volume < 0.0 || self.default_track_volume > 1.0 {
            return Err("Default track volume must be between 0.0 and 1.0".to_string());
        }

        if self.default_track_pan < -1.0 || self.default_track_pan > 1.0 {
            return Err("Default track pan must be between -1.0 and 1.0".to_string());
        }

        Ok(())
    }

    pub fn set_default_track_color(&mut self, color: TrackColor) {
        self.default_track_color = color;
    }

    pub fn set_default_track_volume(&mut self, volume: f32) -> Result<(), String> {
        if !(0.0..=1.0).contains(&volume) {
            return Err("Volume must be between 0.0 and 1.0".to_string());
        }
        self.default_track_volume = volume;
        Ok(())
    }

    pub fn set_default_track_pan(&mut self, pan: f32) -> Result<(), String> {
        if !(-1.0..=1.0).contains(&pan) {
            return Err("Pan must be between -1.0 and 1.0".to_string());
        }
        self.default_track_pan = pan;
        Ok(())
    }

    pub fn set_auto_arm_on_selection(&mut self, enabled: bool) {
        self.auto_arm_on_selection = enabled;
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_default_track_settings() {
        let settings = TrackSettings::default();

        assert_eq!(settings.default_track_color.r, 100);
        assert_eq!(settings.default_track_color.g, 150);
        assert_eq!(settings.default_track_color.b, 200);
        assert_eq!(settings.default_track_volume, 0.8);
        assert_eq!(settings.default_track_pan, 0.0);
        assert!(!settings.auto_arm_on_selection);
    }

    #[test]
    fn test_track_color_new() {
        let color = TrackColor::new(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_track_color_from_hex() {
        let color = TrackColor::from_hex("#FF8040").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_track_color_from_hex_without_hash() {
        let color = TrackColor::from_hex("FF8040").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_track_color_from_hex_invalid() {
        assert!(TrackColor::from_hex("GGGGGG").is_err());
        assert!(TrackColor::from_hex("FF80").is_err());
        assert!(TrackColor::from_hex("FF8040FF").is_err());
    }

    #[test]
    fn test_track_color_to_hex() {
        let color = TrackColor::new(255, 128, 64);
        assert_eq!(color.to_hex(), "#FF8040");
    }

    #[test]
    fn test_builder_pattern() {
        let settings = TrackSettings::new().with_color(TrackColor::new(255, 0, 0)).with_volume(0.5);

        assert_eq!(settings.default_track_color.r, 255);
        assert_eq!(settings.default_track_volume, 0.5);
    }

    #[test]
    fn test_validate_success() {
        let settings = TrackSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_volume_too_low() {
        let mut settings = TrackSettings::default();
        settings.default_track_volume = -0.1;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_volume_too_high() {
        let mut settings = TrackSettings::default();
        settings.default_track_volume = 1.1;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_pan_too_low() {
        let mut settings = TrackSettings::default();
        settings.default_track_pan = -1.1;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_pan_too_high() {
        let mut settings = TrackSettings::default();
        settings.default_track_pan = 1.1;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_set_default_track_color() {
        let mut settings = TrackSettings::default();
        let new_color = TrackColor::new(255, 0, 0);
        settings.set_default_track_color(new_color);
        assert_eq!(settings.default_track_color.r, 255);
    }

    #[test]
    fn test_set_default_track_volume_valid() {
        let mut settings = TrackSettings::default();
        assert!(settings.set_default_track_volume(0.5).is_ok());
        assert_eq!(settings.default_track_volume, 0.5);
    }

    #[test]
    fn test_set_default_track_volume_invalid() {
        let mut settings = TrackSettings::default();
        assert!(settings.set_default_track_volume(1.5).is_err());
    }

    #[test]
    fn test_set_default_track_pan_valid() {
        let mut settings = TrackSettings::default();
        assert!(settings.set_default_track_pan(0.5).is_ok());
        assert_eq!(settings.default_track_pan, 0.5);
    }

    #[test]
    fn test_set_default_track_pan_invalid() {
        let mut settings = TrackSettings::default();
        assert!(settings.set_default_track_pan(2.0).is_err());
    }

    #[test]
    fn test_set_auto_arm_on_selection() {
        let mut settings = TrackSettings::default();
        settings.set_auto_arm_on_selection(true);
        assert!(settings.auto_arm_on_selection);
    }
}
