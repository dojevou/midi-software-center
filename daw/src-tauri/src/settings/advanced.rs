use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedSettings {
    pub debug_logging_enabled: bool,
    pub log_level: LogLevel,
    pub log_file_location: Option<PathBuf>,
    pub virtual_memory_pool_mb: u32,
    pub network_timeout_seconds: u32,
    pub plugin_search_paths: Vec<PathBuf>,
}

impl Default for AdvancedSettings {
    fn default() -> Self {
        Self {
            debug_logging_enabled: false,
            log_level: LogLevel::default(),
            log_file_location: None,
            virtual_memory_pool_mb: 512,
            network_timeout_seconds: 30,
            plugin_search_paths: Vec::new(),
        }
    }
}

impl AdvancedSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_debug_logging(mut self, enabled: bool) -> Self {
        self.debug_logging_enabled = enabled;
        self
    }

    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.virtual_memory_pool_mb < 128 || self.virtual_memory_pool_mb > 4096 {
            return Err("Virtual memory pool must be between 128 MB and 4096 MB".to_string());
        }

        if self.network_timeout_seconds < 5 || self.network_timeout_seconds > 300 {
            return Err("Network timeout must be between 5 and 300 seconds".to_string());
        }

        // Check if log file location is absolute (if provided)
        if let Some(path) = &self.log_file_location {
            if !path.is_absolute() {
                return Err("Log file location must be an absolute path".to_string());
            }
        }

        // Check if plugin search paths are absolute
        for path in &self.plugin_search_paths {
            if !path.is_absolute() {
                return Err(format!("Plugin search path must be absolute: {:?}", path));
            }
        }

        Ok(())
    }

    pub fn enable_debug_logging(&mut self) {
        self.debug_logging_enabled = true;
    }

    pub fn disable_debug_logging(&mut self) {
        self.debug_logging_enabled = false;
    }

    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
    }

    pub fn set_log_file_location(&mut self, path: Option<PathBuf>) -> Result<(), String> {
        if let Some(p) = &path {
            if !p.is_absolute() {
                return Err("Log file location must be an absolute path".to_string());
            }
        }
        self.log_file_location = path;
        Ok(())
    }

    pub fn set_virtual_memory_pool(&mut self, size_mb: u32) -> Result<(), String> {
        if !(128..=4096).contains(&size_mb) {
            return Err("Virtual memory pool must be between 128 MB and 4096 MB".to_string());
        }
        self.virtual_memory_pool_mb = size_mb;
        Ok(())
    }

    pub fn set_network_timeout(&mut self, seconds: u32) -> Result<(), String> {
        if !(5..=300).contains(&seconds) {
            return Err("Network timeout must be between 5 and 300 seconds".to_string());
        }
        self.network_timeout_seconds = seconds;
        Ok(())
    }

    pub fn add_plugin_search_path(&mut self, path: PathBuf) -> Result<(), String> {
        if !path.is_absolute() {
            return Err("Plugin search path must be absolute".to_string());
        }

        if self.plugin_search_paths.contains(&path) {
            return Err("Plugin search path already exists".to_string());
        }

        self.plugin_search_paths.push(path);
        Ok(())
    }

    pub fn remove_plugin_search_path(&mut self, path: &PathBuf) -> bool {
        if let Some(index) = self.plugin_search_paths.iter().position(|p| p == path) {
            self.plugin_search_paths.remove(index);
            true
        } else {
            false
        }
    }

    pub fn clear_plugin_search_paths(&mut self) {
        self.plugin_search_paths.clear();
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_default_advanced_settings() {
        let settings = AdvancedSettings::default();

        assert!(!settings.debug_logging_enabled);
        assert_eq!(settings.log_level, LogLevel::Info);
        assert!(settings.log_file_location.is_none());
        assert_eq!(settings.virtual_memory_pool_mb, 512);
        assert_eq!(settings.network_timeout_seconds, 30);
        assert!(settings.plugin_search_paths.is_empty());
    }

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Error.as_str(), "error");
        assert_eq!(LogLevel::Warn.as_str(), "warn");
        assert_eq!(LogLevel::Info.as_str(), "info");
        assert_eq!(LogLevel::Debug.as_str(), "debug");
        assert_eq!(LogLevel::Trace.as_str(), "trace");
    }

    #[test]
    fn test_builder_pattern() {
        let settings =
            AdvancedSettings::new().with_debug_logging(true).with_log_level(LogLevel::Debug);

        assert!(settings.debug_logging_enabled);
        assert_eq!(settings.log_level, LogLevel::Debug);
    }

    #[test]
    fn test_validate_success() {
        let settings = AdvancedSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_memory_pool_too_small() {
        let mut settings = AdvancedSettings::default();
        settings.virtual_memory_pool_mb = 100;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_memory_pool_too_large() {
        let mut settings = AdvancedSettings::default();
        settings.virtual_memory_pool_mb = 5000;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_timeout_too_small() {
        let mut settings = AdvancedSettings::default();
        settings.network_timeout_seconds = 2;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_timeout_too_large() {
        let mut settings = AdvancedSettings::default();
        settings.network_timeout_seconds = 400;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_relative_log_path() {
        let mut settings = AdvancedSettings::default();
        settings.log_file_location = Some(PathBuf::from("relative/path.log"));
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_validate_relative_plugin_path() {
        let mut settings = AdvancedSettings::default();
        settings.plugin_search_paths.push(PathBuf::from("relative/plugins"));
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_enable_debug_logging() {
        let mut settings = AdvancedSettings::default();
        settings.enable_debug_logging();
        assert!(settings.debug_logging_enabled);
    }

    #[test]
    fn test_disable_debug_logging() {
        let mut settings = AdvancedSettings::default();
        settings.enable_debug_logging();
        settings.disable_debug_logging();
        assert!(!settings.debug_logging_enabled);
    }

    #[test]
    fn test_set_log_level() {
        let mut settings = AdvancedSettings::default();
        settings.set_log_level(LogLevel::Trace);
        assert_eq!(settings.log_level, LogLevel::Trace);
    }

    #[test]
    fn test_set_log_file_location_valid() {
        let mut settings = AdvancedSettings::default();
        let result = settings.set_log_file_location(Some(PathBuf::from("/var/log/app.log")));
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_log_file_location_relative() {
        let mut settings = AdvancedSettings::default();
        let result = settings.set_log_file_location(Some(PathBuf::from("app.log")));
        assert!(result.is_err());
    }

    #[test]
    fn test_set_virtual_memory_pool_valid() {
        let mut settings = AdvancedSettings::default();
        assert!(settings.set_virtual_memory_pool(1024).is_ok());
        assert_eq!(settings.virtual_memory_pool_mb, 1024);
    }

    #[test]
    fn test_set_virtual_memory_pool_invalid() {
        let mut settings = AdvancedSettings::default();
        assert!(settings.set_virtual_memory_pool(50).is_err());
        assert!(settings.set_virtual_memory_pool(5000).is_err());
    }

    #[test]
    fn test_set_network_timeout_valid() {
        let mut settings = AdvancedSettings::default();
        assert!(settings.set_network_timeout(60).is_ok());
        assert_eq!(settings.network_timeout_seconds, 60);
    }

    #[test]
    fn test_set_network_timeout_invalid() {
        let mut settings = AdvancedSettings::default();
        assert!(settings.set_network_timeout(2).is_err());
        assert!(settings.set_network_timeout(400).is_err());
    }

    #[test]
    fn test_add_plugin_search_path_valid() {
        let mut settings = AdvancedSettings::default();
        let result = settings.add_plugin_search_path(PathBuf::from("/usr/lib/plugins"));
        assert!(result.is_ok());
        assert_eq!(settings.plugin_search_paths.len(), 1);
    }

    #[test]
    fn test_add_plugin_search_path_relative() {
        let mut settings = AdvancedSettings::default();
        let result = settings.add_plugin_search_path(PathBuf::from("plugins"));
        assert!(result.is_err());
    }

    #[test]
    fn test_add_plugin_search_path_duplicate() {
        let mut settings = AdvancedSettings::default();
        let path = PathBuf::from("/usr/lib/plugins");

        settings.add_plugin_search_path(path.clone()).unwrap();
        let result = settings.add_plugin_search_path(path);

        assert!(result.is_err());
        assert_eq!(settings.plugin_search_paths.len(), 1);
    }

    #[test]
    fn test_remove_plugin_search_path() {
        let mut settings = AdvancedSettings::default();
        let path = PathBuf::from("/usr/lib/plugins");

        settings.add_plugin_search_path(path.clone()).unwrap();
        assert!(settings.remove_plugin_search_path(&path));
        assert!(settings.plugin_search_paths.is_empty());
    }

    #[test]
    fn test_remove_plugin_search_path_nonexistent() {
        let mut settings = AdvancedSettings::default();
        let path = PathBuf::from("/usr/lib/plugins");
        assert!(!settings.remove_plugin_search_path(&path));
    }

    #[test]
    fn test_clear_plugin_search_paths() {
        let mut settings = AdvancedSettings::default();
        settings.add_plugin_search_path(PathBuf::from("/usr/lib/plugins1")).unwrap();
        settings.add_plugin_search_path(PathBuf::from("/usr/lib/plugins2")).unwrap();

        settings.clear_plugin_search_paths();
        assert!(settings.plugin_search_paths.is_empty());
    }
}
