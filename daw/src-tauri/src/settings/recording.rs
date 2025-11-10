use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum RecordingFormat {
    #[default]
    Wav,
    Mp3,
    Flac,
}


impl RecordingFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            RecordingFormat::Wav => "wav",
            RecordingFormat::Mp3 => "mp3",
            RecordingFormat::Flac => "flac",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSettings {
    pub recording_format: RecordingFormat,
    pub input_monitoring_enabled: bool,
    pub latency_compensation_ms: f32,
    pub auto_punch_in_enabled: bool,
    pub auto_punch_out_enabled: bool,
    pub punch_in_bar: u32,
    pub punch_out_bar: u32,
}

impl Default for RecordingSettings {
    fn default() -> Self {
        Self {
            recording_format: RecordingFormat::default(),
            input_monitoring_enabled: true,
            latency_compensation_ms: 0.0,
            auto_punch_in_enabled: false,
            auto_punch_out_enabled: false,
            punch_in_bar: 1,
            punch_out_bar: 5,
        }
    }
}

impl RecordingSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_format(mut self, format: RecordingFormat) -> Self {
        self.recording_format = format;
        self
    }

    pub fn with_monitoring(mut self, enabled: bool) -> Self {
        self.input_monitoring_enabled = enabled;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.latency_compensation_ms < 0.0 || self.latency_compensation_ms > 1000.0 {
            return Err("Latency compensation must be between 0.0 and 1000.0 ms".to_string());
        }

        if self.punch_in_bar == 0 {
            return Err("Punch in bar must be greater than 0".to_string());
        }

        if self.punch_out_bar == 0 {
            return Err("Punch out bar must be greater than 0".to_string());
        }

        if self.auto_punch_in_enabled && self.auto_punch_out_enabled
            && self.punch_out_bar <= self.punch_in_bar {
                return Err("Punch out bar must be after punch in bar".to_string());
            }

        Ok(())
    }

    pub fn set_recording_format(&mut self, format: RecordingFormat) {
        self.recording_format = format;
    }

    pub fn set_input_monitoring_enabled(&mut self, enabled: bool) {
        self.input_monitoring_enabled = enabled;
    }

    pub fn set_latency_compensation(&mut self, ms: f32) -> Result<(), String> {
        if !(0.0..=1000.0).contains(&ms) {
            return Err("Latency compensation must be between 0.0 and 1000.0 ms".to_string());
        }
        self.latency_compensation_ms = ms;
        Ok(())
    }

    pub fn set_auto_punch_in_enabled(&mut self, enabled: bool) {
        self.auto_punch_in_enabled = enabled;
    }

    pub fn set_auto_punch_out_enabled(&mut self, enabled: bool) {
        self.auto_punch_out_enabled = enabled;
    }

    pub fn set_punch_in_bar(&mut self, bar: u32) -> Result<(), String> {
        if bar == 0 {
            return Err("Punch in bar must be greater than 0".to_string());
        }

        if self.auto_punch_out_enabled && bar >= self.punch_out_bar {
            return Err("Punch in bar must be before punch out bar".to_string());
        }

        self.punch_in_bar = bar;
        Ok(())
    }

    pub fn set_punch_out_bar(&mut self, bar: u32) -> Result<(), String> {
        if bar == 0 {
            return Err("Punch out bar must be greater than 0".to_string());
        }

        if self.auto_punch_in_enabled && bar <= self.punch_in_bar {
            return Err("Punch out bar must be after punch in bar".to_string());
        }

        self.punch_out_bar = bar;
        Ok(())
    }

    pub fn is_punch_enabled(&self) -> bool {
        self.auto_punch_in_enabled || self.auto_punch_out_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_recording_settings() {
        let settings = RecordingSettings::default();

        assert_eq!(settings.recording_format, RecordingFormat::Wav);
        assert!(settings.input_monitoring_enabled);
        assert_eq!(settings.latency_compensation_ms, 0.0);
        assert!(!settings.auto_punch_in_enabled);
        assert!(!settings.auto_punch_out_enabled);
        assert_eq!(settings.punch_in_bar, 1);
        assert_eq!(settings.punch_out_bar, 5);
    }

    #[test]
    fn test_recording_format_extension() {
        assert_eq!(RecordingFormat::Wav.extension(), "wav");
        assert_eq!(RecordingFormat::Mp3.extension(), "mp3");
        assert_eq!(RecordingFormat::Flac.extension(), "flac");
    }

    #[test]
    fn test_builder_pattern() {
        let settings = RecordingSettings::new()
            .with_format(RecordingFormat::Flac)
            .with_monitoring(false);

        assert_eq!(settings.recording_format, RecordingFormat::Flac);
        assert!(!settings.input_monitoring_enabled);
    }

    #[test]
    fn test_validate_success() {
        let settings = RecordingSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_latency_compensation_too_low() {
        let mut settings = RecordingSettings::default();
        settings.latency_compensation_ms = -1.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_latency_compensation_too_high() {
        let mut settings = RecordingSettings::default();
        settings.latency_compensation_ms = 1001.0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_punch_in_zero() {
        let mut settings = RecordingSettings::default();
        settings.punch_in_bar = 0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_punch_out_zero() {
        let mut settings = RecordingSettings::default();
        settings.punch_out_bar = 0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_punch_out_before_punch_in() {
        let mut settings = RecordingSettings::default();
        settings.auto_punch_in_enabled = true;
        settings.auto_punch_out_enabled = true;
        settings.punch_in_bar = 5;
        settings.punch_out_bar = 3;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_set_recording_format() {
        let mut settings = RecordingSettings::default();
        settings.set_recording_format(RecordingFormat::Mp3);
        assert_eq!(settings.recording_format, RecordingFormat::Mp3);
    }

    #[test]
    fn test_set_input_monitoring_enabled() {
        let mut settings = RecordingSettings::default();
        settings.set_input_monitoring_enabled(false);
        assert!(!settings.input_monitoring_enabled);
    }

    #[test]
    fn test_set_latency_compensation_valid() {
        let mut settings = RecordingSettings::default();
        assert!(settings.set_latency_compensation(10.0).is_ok());
        assert_eq!(settings.latency_compensation_ms, 10.0);
    }

    #[test]
    fn test_set_latency_compensation_invalid() {
        let mut settings = RecordingSettings::default();
        assert!(settings.set_latency_compensation(1500.0).is_err());
    }

    #[test]
    fn test_set_auto_punch_in_enabled() {
        let mut settings = RecordingSettings::default();
        settings.set_auto_punch_in_enabled(true);
        assert!(settings.auto_punch_in_enabled);
    }

    #[test]
    fn test_set_auto_punch_out_enabled() {
        let mut settings = RecordingSettings::default();
        settings.set_auto_punch_out_enabled(true);
        assert!(settings.auto_punch_out_enabled);
    }

    #[test]
    fn test_set_punch_in_bar_valid() {
        let mut settings = RecordingSettings::default();
        assert!(settings.set_punch_in_bar(3).is_ok());
        assert_eq!(settings.punch_in_bar, 3);
    }

    #[test]
    fn test_set_punch_in_bar_zero() {
        let mut settings = RecordingSettings::default();
        assert!(settings.set_punch_in_bar(0).is_err());
    }

    #[test]
    fn test_set_punch_out_bar_valid() {
        let mut settings = RecordingSettings::default();
        assert!(settings.set_punch_out_bar(10).is_ok());
        assert_eq!(settings.punch_out_bar, 10);
    }

    #[test]
    fn test_set_punch_out_bar_zero() {
        let mut settings = RecordingSettings::default();
        assert!(settings.set_punch_out_bar(0).is_err());
    }

    #[test]
    fn test_is_punch_enabled() {
        let mut settings = RecordingSettings::default();
        assert!(!settings.is_punch_enabled());

        settings.set_auto_punch_in_enabled(true);
        assert!(settings.is_punch_enabled());
    }
}
