use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SyncMode {
    #[default]
    Internal,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiSettings {
    pub default_input_device: Option<String>,
    pub default_output_device: Option<String>,
    pub sync_mode: SyncMode,
    pub tempo_sync_enabled: bool,
    pub flush_notes_on_stop: bool,
}

impl Default for MidiSettings {
    fn default() -> Self {
        Self {
            default_input_device: None,
            default_output_device: None,
            sync_mode: SyncMode::default(),
            tempo_sync_enabled: false,
            flush_notes_on_stop: true,
        }
    }
}

impl MidiSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_devices(mut self, input: Option<String>, output: Option<String>) -> Self {
        self.default_input_device = input;
        self.default_output_device = output;
        self
    }

    pub fn with_sync_mode(mut self, mode: SyncMode) -> Self {
        self.sync_mode = mode;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        // No additional validation needed
        Ok(())
    }

    pub fn set_default_input(&mut self, device: Option<String>) {
        self.default_input_device = device;
    }

    pub fn set_default_output(&mut self, device: Option<String>) {
        self.default_output_device = device;
    }

    pub fn set_sync_mode(&mut self, mode: SyncMode) {
        self.sync_mode = mode;
    }

    pub fn set_tempo_sync_enabled(&mut self, enabled: bool) {
        self.tempo_sync_enabled = enabled;
    }

    pub fn set_flush_notes_on_stop(&mut self, enabled: bool) {
        self.flush_notes_on_stop = enabled;
    }

    pub fn has_input_device(&self) -> bool {
        self.default_input_device.is_some()
    }

    pub fn has_output_device(&self) -> bool {
        self.default_output_device.is_some()
    }

    pub fn is_external_sync(&self) -> bool {
        matches!(self.sync_mode, SyncMode::External)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_midi_settings() {
        let settings = MidiSettings::default();

        assert!(settings.default_input_device.is_none());
        assert!(settings.default_output_device.is_none());
        assert_eq!(settings.sync_mode, SyncMode::Internal);
        assert!(!settings.tempo_sync_enabled);
        assert!(settings.flush_notes_on_stop);
    }

    #[test]
    fn test_builder_pattern() {
        let settings = MidiSettings::new()
            .with_devices(Some("Input".to_string()), Some("Output".to_string()))
            .with_sync_mode(SyncMode::External);

        assert_eq!(settings.default_input_device, Some("Input".to_string()));
        assert_eq!(settings.default_output_device, Some("Output".to_string()));
        assert_eq!(settings.sync_mode, SyncMode::External);
    }

    #[test]
    fn test_validate() {
        let settings = MidiSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_set_default_input() {
        let mut settings = MidiSettings::default();
        settings.set_default_input(Some("Test Input".to_string()));
        assert_eq!(
            settings.default_input_device,
            Some("Test Input".to_string())
        );
    }

    #[test]
    fn test_set_default_output() {
        let mut settings = MidiSettings::default();
        settings.set_default_output(Some("Test Output".to_string()));
        assert_eq!(
            settings.default_output_device,
            Some("Test Output".to_string())
        );
    }

    #[test]
    fn test_set_sync_mode() {
        let mut settings = MidiSettings::default();
        settings.set_sync_mode(SyncMode::External);
        assert_eq!(settings.sync_mode, SyncMode::External);
    }

    #[test]
    fn test_set_tempo_sync_enabled() {
        let mut settings = MidiSettings::default();
        settings.set_tempo_sync_enabled(true);
        assert!(settings.tempo_sync_enabled);
    }

    #[test]
    fn test_set_flush_notes_on_stop() {
        let mut settings = MidiSettings::default();
        settings.set_flush_notes_on_stop(false);
        assert!(!settings.flush_notes_on_stop);
    }

    #[test]
    fn test_has_input_device() {
        let mut settings = MidiSettings::default();
        assert!(!settings.has_input_device());

        settings.set_default_input(Some("Device".to_string()));
        assert!(settings.has_input_device());
    }

    #[test]
    fn test_has_output_device() {
        let mut settings = MidiSettings::default();
        assert!(!settings.has_output_device());

        settings.set_default_output(Some("Device".to_string()));
        assert!(settings.has_output_device());
    }

    #[test]
    fn test_is_external_sync() {
        let mut settings = MidiSettings::default();
        assert!(!settings.is_external_sync());

        settings.set_sync_mode(SyncMode::External);
        assert!(settings.is_external_sync());
    }

    #[test]
    fn test_serialization() {
        let settings = MidiSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: MidiSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.sync_mode, deserialized.sync_mode);
        assert_eq!(settings.tempo_sync_enabled, deserialized.tempo_sync_enabled);
    }

    #[test]
    fn test_clear_devices() {
        let mut settings =
            MidiSettings::new().with_devices(Some("Input".to_string()), Some("Output".to_string()));

        settings.set_default_input(None);
        settings.set_default_output(None);

        assert!(settings.default_input_device.is_none());
        assert!(settings.default_output_device.is_none());
    }

    #[test]
    fn test_sync_mode_variants() {
        let internal = SyncMode::Internal;
        let external = SyncMode::External;

        assert_ne!(internal, external);
        assert_eq!(internal, SyncMode::default());
    }

    #[test]
    fn test_tempo_sync_toggle() {
        let mut settings = MidiSettings::default();

        settings.set_tempo_sync_enabled(true);
        assert!(settings.tempo_sync_enabled);

        settings.set_tempo_sync_enabled(false);
        assert!(!settings.tempo_sync_enabled);
    }

    #[test]
    fn test_flush_notes_toggle() {
        let mut settings = MidiSettings::default();

        settings.set_flush_notes_on_stop(false);
        assert!(!settings.flush_notes_on_stop);

        settings.set_flush_notes_on_stop(true);
        assert!(settings.flush_notes_on_stop);
    }

    #[test]
    fn test_external_sync_behavior() {
        let mut settings = MidiSettings::default();
        settings.set_sync_mode(SyncMode::External);
        settings.set_tempo_sync_enabled(true);

        assert!(settings.is_external_sync());
        assert!(settings.tempo_sync_enabled);
    }
}
