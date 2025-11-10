use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum VirtualScrollingThreshold {
    Items100 = 100,
    Items500 = 500,
    #[default]
    Items1000 = 1000,
    Items5000 = 5000,
}


impl VirtualScrollingThreshold {
    pub fn as_usize(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    pub cache_size_mb: u32,
    pub virtual_scrolling_threshold: VirtualScrollingThreshold,
    pub batch_operation_thread_count: u32,
    pub memory_limit_alert_enabled: bool,
    pub memory_limit_mb: u32,
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            cache_size_mb: 500,
            virtual_scrolling_threshold: VirtualScrollingThreshold::default(),
            batch_operation_thread_count: 4,
            memory_limit_alert_enabled: true,
            memory_limit_mb: 2048,
        }
    }
}

impl PerformanceSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cache_size(mut self, size_mb: u32) -> Self {
        self.cache_size_mb = size_mb;
        self
    }

    pub fn with_thread_count(mut self, count: u32) -> Self {
        self.batch_operation_thread_count = count;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.cache_size_mb < 100 || self.cache_size_mb > 2048 {
            return Err("Cache size must be between 100 MB and 2048 MB".to_string());
        }

        if self.batch_operation_thread_count < 1 || self.batch_operation_thread_count > 16 {
            return Err("Thread count must be between 1 and 16".to_string());
        }

        if self.memory_limit_mb < 512 || self.memory_limit_mb > 16384 {
            return Err("Memory limit must be between 512 MB and 16384 MB".to_string());
        }

        Ok(())
    }

    pub fn set_cache_size(&mut self, size_mb: u32) -> Result<(), String> {
        if !(100..=2048).contains(&size_mb) {
            return Err("Cache size must be between 100 MB and 2048 MB".to_string());
        }
        self.cache_size_mb = size_mb;
        Ok(())
    }

    pub fn set_virtual_scrolling_threshold(&mut self, threshold: VirtualScrollingThreshold) {
        self.virtual_scrolling_threshold = threshold;
    }

    pub fn set_thread_count(&mut self, count: u32) -> Result<(), String> {
        if !(1..=16).contains(&count) {
            return Err("Thread count must be between 1 and 16".to_string());
        }
        self.batch_operation_thread_count = count;
        Ok(())
    }

    pub fn set_memory_limit_alert_enabled(&mut self, enabled: bool) {
        self.memory_limit_alert_enabled = enabled;
    }

    pub fn set_memory_limit(&mut self, limit_mb: u32) -> Result<(), String> {
        if !(512..=16384).contains(&limit_mb) {
            return Err("Memory limit must be between 512 MB and 16384 MB".to_string());
        }
        self.memory_limit_mb = limit_mb;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_performance_settings() {
        let settings = PerformanceSettings::default();

        assert_eq!(settings.cache_size_mb, 500);
        assert_eq!(
            settings.virtual_scrolling_threshold,
            VirtualScrollingThreshold::Items1000
        );
        assert_eq!(settings.batch_operation_thread_count, 4);
        assert!(settings.memory_limit_alert_enabled);
        assert_eq!(settings.memory_limit_mb, 2048);
    }

    #[test]
    fn test_virtual_scrolling_threshold_values() {
        assert_eq!(VirtualScrollingThreshold::Items100.as_usize(), 100);
        assert_eq!(VirtualScrollingThreshold::Items500.as_usize(), 500);
        assert_eq!(VirtualScrollingThreshold::Items1000.as_usize(), 1000);
        assert_eq!(VirtualScrollingThreshold::Items5000.as_usize(), 5000);
    }

    #[test]
    fn test_builder_pattern() {
        let settings = PerformanceSettings::new().with_cache_size(1000).with_thread_count(8);

        assert_eq!(settings.cache_size_mb, 1000);
        assert_eq!(settings.batch_operation_thread_count, 8);
    }

    #[test]
    fn test_validate_success() {
        let settings = PerformanceSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_cache_size_too_small() {
        let mut settings = PerformanceSettings::default();
        settings.cache_size_mb = 50;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_cache_size_too_large() {
        let mut settings = PerformanceSettings::default();
        settings.cache_size_mb = 3000;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_thread_count_too_small() {
        let mut settings = PerformanceSettings::default();
        settings.batch_operation_thread_count = 0;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_thread_count_too_large() {
        let mut settings = PerformanceSettings::default();
        settings.batch_operation_thread_count = 17;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_memory_limit_too_small() {
        let mut settings = PerformanceSettings::default();
        settings.memory_limit_mb = 256;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_memory_limit_too_large() {
        let mut settings = PerformanceSettings::default();
        settings.memory_limit_mb = 20000;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_set_cache_size_valid() {
        let mut settings = PerformanceSettings::default();
        assert!(settings.set_cache_size(1024).is_ok());
        assert_eq!(settings.cache_size_mb, 1024);
    }

    #[test]
    fn test_set_cache_size_invalid() {
        let mut settings = PerformanceSettings::default();
        assert!(settings.set_cache_size(50).is_err());
        assert!(settings.set_cache_size(3000).is_err());
    }

    #[test]
    fn test_set_virtual_scrolling_threshold() {
        let mut settings = PerformanceSettings::default();
        settings.set_virtual_scrolling_threshold(VirtualScrollingThreshold::Items5000);
        assert_eq!(
            settings.virtual_scrolling_threshold,
            VirtualScrollingThreshold::Items5000
        );
    }

    #[test]
    fn test_set_thread_count_valid() {
        let mut settings = PerformanceSettings::default();
        assert!(settings.set_thread_count(8).is_ok());
        assert_eq!(settings.batch_operation_thread_count, 8);
    }

    #[test]
    fn test_set_thread_count_invalid() {
        let mut settings = PerformanceSettings::default();
        assert!(settings.set_thread_count(0).is_err());
        assert!(settings.set_thread_count(20).is_err());
    }

    #[test]
    fn test_set_memory_limit_alert_enabled() {
        let mut settings = PerformanceSettings::default();
        settings.set_memory_limit_alert_enabled(false);
        assert!(!settings.memory_limit_alert_enabled);
    }

    #[test]
    fn test_set_memory_limit_valid() {
        let mut settings = PerformanceSettings::default();
        assert!(settings.set_memory_limit(4096).is_ok());
        assert_eq!(settings.memory_limit_mb, 4096);
    }

    #[test]
    fn test_set_memory_limit_invalid() {
        let mut settings = PerformanceSettings::default();
        assert!(settings.set_memory_limit(256).is_err());
        assert!(settings.set_memory_limit(20000).is_err());
    }
}
