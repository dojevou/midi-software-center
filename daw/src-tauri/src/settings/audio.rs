use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BufferSize {
    Samples32 = 32,
    Samples64 = 64,
    Samples128 = 128,
    Samples256 = 256,
    Samples512 = 512,
    Samples1024 = 1024,
    Samples2048 = 2048,
    Samples4096 = 4096,
}

impl Default for BufferSize {
    fn default() -> Self {
        BufferSize::Samples512
    }
}

impl BufferSize {
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn from_u32(value: u32) -> Result<Self, String> {
        match value {
            32 => Ok(BufferSize::Samples32),
            64 => Ok(BufferSize::Samples64),
            128 => Ok(BufferSize::Samples128),
            256 => Ok(BufferSize::Samples256),
            512 => Ok(BufferSize::Samples512),
            1024 => Ok(BufferSize::Samples1024),
            2048 => Ok(BufferSize::Samples2048),
            4096 => Ok(BufferSize::Samples4096),
            _ => Err(format!("Invalid buffer size: {}", value)),
        }
    }

    pub fn latency_ms(self, sample_rate: SampleRate) -> f64 {
        let samples = self.as_u32() as f64;
        let rate = sample_rate.as_u32() as f64;
        (samples / rate) * 1000.0
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SampleRate {
    Hz44100 = 44100,
    Hz48000 = 48000,
    Hz88200 = 88200,
    Hz96000 = 96000,
    Hz176400 = 176400,
    Hz192000 = 192000,
}

impl Default for SampleRate {
    fn default() -> Self {
        SampleRate::Hz48000
    }
}

impl SampleRate {
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn from_u32(value: u32) -> Result<Self, String> {
        match value {
            44100 => Ok(SampleRate::Hz44100),
            48000 => Ok(SampleRate::Hz48000),
            88200 => Ok(SampleRate::Hz88200),
            96000 => Ok(SampleRate::Hz96000),
            176400 => Ok(SampleRate::Hz176400),
            192000 => Ok(SampleRate::Hz192000),
            _ => Err(format!("Invalid sample rate: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub buffer_size: BufferSize,
    pub sample_rate: SampleRate,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    pub latency_monitoring_enabled: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            buffer_size: BufferSize::default(),
            sample_rate: SampleRate::default(),
            input_device: None,
            output_device: None,
            latency_monitoring_enabled: true,
        }
    }
}

impl AudioSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_buffer_size(mut self, buffer_size: BufferSize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    pub fn with_sample_rate(mut self, sample_rate: SampleRate) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    pub fn with_devices(mut self, input: Option<String>, output: Option<String>) -> Self {
        self.input_device = input;
        self.output_device = output;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        // No additional validation needed beyond type safety
        Ok(())
    }

    pub fn set_buffer_size(&mut self, buffer_size: BufferSize) {
        self.buffer_size = buffer_size;
    }

    pub fn set_sample_rate(&mut self, sample_rate: SampleRate) {
        self.sample_rate = sample_rate;
    }

    pub fn set_input_device(&mut self, device: Option<String>) {
        self.input_device = device;
    }

    pub fn set_output_device(&mut self, device: Option<String>) {
        self.output_device = device;
    }

    pub fn set_latency_monitoring_enabled(&mut self, enabled: bool) {
        self.latency_monitoring_enabled = enabled;
    }

    pub fn expected_latency_ms(&self) -> f64 {
        self.buffer_size.latency_ms(self.sample_rate)
    }

    pub fn has_input_device(&self) -> bool {
        self.input_device.is_some()
    }

    pub fn has_output_device(&self) -> bool {
        self.output_device.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_audio_settings() {
        let settings = AudioSettings::default();

        assert_eq!(settings.buffer_size, BufferSize::Samples512);
        assert_eq!(settings.sample_rate, SampleRate::Hz48000);
        assert!(settings.input_device.is_none());
        assert!(settings.output_device.is_none());
        assert!(settings.latency_monitoring_enabled);
    }

    #[test]
    fn test_buffer_size_as_u32() {
        assert_eq!(BufferSize::Samples32.as_u32(), 32);
        assert_eq!(BufferSize::Samples512.as_u32(), 512);
        assert_eq!(BufferSize::Samples4096.as_u32(), 4096);
    }

    #[test]
    fn test_buffer_size_from_u32() {
        assert_eq!(BufferSize::from_u32(128).unwrap(), BufferSize::Samples128);
        assert_eq!(BufferSize::from_u32(1024).unwrap(), BufferSize::Samples1024);
        assert!(BufferSize::from_u32(333).is_err());
    }

    #[test]
    fn test_sample_rate_as_u32() {
        assert_eq!(SampleRate::Hz44100.as_u32(), 44100);
        assert_eq!(SampleRate::Hz48000.as_u32(), 48000);
        assert_eq!(SampleRate::Hz192000.as_u32(), 192000);
    }

    #[test]
    fn test_sample_rate_from_u32() {
        assert_eq!(SampleRate::from_u32(44100).unwrap(), SampleRate::Hz44100);
        assert_eq!(SampleRate::from_u32(96000).unwrap(), SampleRate::Hz96000);
        assert!(SampleRate::from_u32(12345).is_err());
    }

    #[test]
    fn test_latency_calculation() {
        let latency = BufferSize::Samples512.latency_ms(SampleRate::Hz48000);
        assert!((latency - 10.67).abs() < 0.1); // ~10.67ms
    }

    #[test]
    fn test_builder_pattern() {
        let settings = AudioSettings::new()
            .with_buffer_size(BufferSize::Samples256)
            .with_sample_rate(SampleRate::Hz44100)
            .with_devices(Some("Input Device".to_string()), Some("Output Device".to_string()));

        assert_eq!(settings.buffer_size, BufferSize::Samples256);
        assert_eq!(settings.sample_rate, SampleRate::Hz44100);
        assert_eq!(settings.input_device, Some("Input Device".to_string()));
        assert_eq!(settings.output_device, Some("Output Device".to_string()));
    }

    #[test]
    fn test_validate() {
        let settings = AudioSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_set_buffer_size() {
        let mut settings = AudioSettings::default();
        settings.set_buffer_size(BufferSize::Samples1024);
        assert_eq!(settings.buffer_size, BufferSize::Samples1024);
    }

    #[test]
    fn test_set_sample_rate() {
        let mut settings = AudioSettings::default();
        settings.set_sample_rate(SampleRate::Hz96000);
        assert_eq!(settings.sample_rate, SampleRate::Hz96000);
    }

    #[test]
    fn test_set_input_device() {
        let mut settings = AudioSettings::default();
        settings.set_input_device(Some("Test Input".to_string()));
        assert_eq!(settings.input_device, Some("Test Input".to_string()));
    }

    #[test]
    fn test_set_output_device() {
        let mut settings = AudioSettings::default();
        settings.set_output_device(Some("Test Output".to_string()));
        assert_eq!(settings.output_device, Some("Test Output".to_string()));
    }

    #[test]
    fn test_set_latency_monitoring() {
        let mut settings = AudioSettings::default();
        settings.set_latency_monitoring_enabled(false);
        assert!(!settings.latency_monitoring_enabled);
    }

    #[test]
    fn test_expected_latency_ms() {
        let settings = AudioSettings::default();
        let latency = settings.expected_latency_ms();
        assert!(latency > 0.0);
    }

    #[test]
    fn test_has_input_device() {
        let mut settings = AudioSettings::default();
        assert!(!settings.has_input_device());

        settings.set_input_device(Some("Device".to_string()));
        assert!(settings.has_input_device());
    }

    #[test]
    fn test_has_output_device() {
        let mut settings = AudioSettings::default();
        assert!(!settings.has_output_device());

        settings.set_output_device(Some("Device".to_string()));
        assert!(settings.has_output_device());
    }

    #[test]
    fn test_buffer_size_latency_various() {
        let rates = [
            SampleRate::Hz44100,
            SampleRate::Hz48000,
            SampleRate::Hz96000,
        ];

        for rate in rates {
            let latency_256 = BufferSize::Samples256.latency_ms(rate);
            let latency_512 = BufferSize::Samples512.latency_ms(rate);

            // Larger buffer size should have higher latency
            assert!(latency_512 > latency_256);
        }
    }

    #[test]
    fn test_serialization() {
        let settings = AudioSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AudioSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.buffer_size, deserialized.buffer_size);
        assert_eq!(settings.sample_rate, deserialized.sample_rate);
    }
}
