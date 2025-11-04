use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncInterval {
    Manual,
    Minutes5,
    Minutes15,
    Hour1,
}

impl Default for SyncInterval {
    fn default() -> Self {
        SyncInterval::Manual
    }
}

impl SyncInterval {
    pub fn as_minutes(&self) -> Option<u32> {
        match self {
            SyncInterval::Manual => None,
            SyncInterval::Minutes5 => Some(5),
            SyncInterval::Minutes15 => Some(15),
            SyncInterval::Hour1 => Some(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub cloud_sync_enabled: bool,
    pub sync_interval: SyncInterval,
    pub sync_folders: Vec<String>,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            cloud_sync_enabled: false,
            sync_interval: SyncInterval::default(),
            sync_folders: Vec::new(),
        }
    }
}

impl SyncSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cloud_sync(mut self, enabled: bool) -> Self {
        self.cloud_sync_enabled = enabled;
        self
    }

    pub fn with_interval(mut self, interval: SyncInterval) -> Self {
        self.sync_interval = interval;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        // No additional validation needed
        Ok(())
    }

    pub fn enable_sync(&mut self) {
        self.cloud_sync_enabled = true;
    }

    pub fn disable_sync(&mut self) {
        self.cloud_sync_enabled = false;
    }

    pub fn set_sync_interval(&mut self, interval: SyncInterval) {
        self.sync_interval = interval;
    }

    pub fn add_sync_folder(&mut self, folder: String) {
        if !self.sync_folders.contains(&folder) {
            self.sync_folders.push(folder);
        }
    }

    pub fn remove_sync_folder(&mut self, folder: &str) -> bool {
        if let Some(index) = self.sync_folders.iter().position(|f| f == folder) {
            self.sync_folders.remove(index);
            true
        } else {
            false
        }
    }

    pub fn clear_sync_folders(&mut self) {
        self.sync_folders.clear();
    }

    pub fn is_auto_sync_enabled(&self) -> bool {
        self.cloud_sync_enabled && !matches!(self.sync_interval, SyncInterval::Manual)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_sync_settings() {
        let settings = SyncSettings::default();

        assert!(!settings.cloud_sync_enabled);
        assert_eq!(settings.sync_interval, SyncInterval::Manual);
        assert!(settings.sync_folders.is_empty());
    }

    #[test]
    fn test_sync_interval_as_minutes() {
        assert_eq!(SyncInterval::Manual.as_minutes(), None);
        assert_eq!(SyncInterval::Minutes5.as_minutes(), Some(5));
        assert_eq!(SyncInterval::Minutes15.as_minutes(), Some(15));
        assert_eq!(SyncInterval::Hour1.as_minutes(), Some(60));
    }

    #[test]
    fn test_builder_pattern() {
        let settings = SyncSettings::new()
            .with_cloud_sync(true)
            .with_interval(SyncInterval::Minutes15);

        assert!(settings.cloud_sync_enabled);
        assert_eq!(settings.sync_interval, SyncInterval::Minutes15);
    }

    #[test]
    fn test_validate() {
        let settings = SyncSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_enable_sync() {
        let mut settings = SyncSettings::default();
        settings.enable_sync();
        assert!(settings.cloud_sync_enabled);
    }

    #[test]
    fn test_disable_sync() {
        let mut settings = SyncSettings::default();
        settings.enable_sync();
        settings.disable_sync();
        assert!(!settings.cloud_sync_enabled);
    }

    #[test]
    fn test_set_sync_interval() {
        let mut settings = SyncSettings::default();
        settings.set_sync_interval(SyncInterval::Hour1);
        assert_eq!(settings.sync_interval, SyncInterval::Hour1);
    }

    #[test]
    fn test_add_sync_folder() {
        let mut settings = SyncSettings::default();
        settings.add_sync_folder("projects".to_string());
        assert_eq!(settings.sync_folders.len(), 1);
        assert!(settings.sync_folders.contains(&"projects".to_string()));
    }

    #[test]
    fn test_add_sync_folder_duplicate() {
        let mut settings = SyncSettings::default();
        settings.add_sync_folder("projects".to_string());
        settings.add_sync_folder("projects".to_string());
        assert_eq!(settings.sync_folders.len(), 1);
    }

    #[test]
    fn test_remove_sync_folder() {
        let mut settings = SyncSettings::default();
        settings.add_sync_folder("projects".to_string());
        assert!(settings.remove_sync_folder("projects"));
        assert!(settings.sync_folders.is_empty());
    }

    #[test]
    fn test_remove_sync_folder_nonexistent() {
        let mut settings = SyncSettings::default();
        assert!(!settings.remove_sync_folder("nonexistent"));
    }

    #[test]
    fn test_clear_sync_folders() {
        let mut settings = SyncSettings::default();
        settings.add_sync_folder("folder1".to_string());
        settings.add_sync_folder("folder2".to_string());
        settings.clear_sync_folders();
        assert!(settings.sync_folders.is_empty());
    }

    #[test]
    fn test_is_auto_sync_enabled() {
        let mut settings = SyncSettings::default();
        assert!(!settings.is_auto_sync_enabled()); // Disabled by default

        settings.enable_sync();
        assert!(!settings.is_auto_sync_enabled()); // Manual interval

        settings.set_sync_interval(SyncInterval::Minutes15);
        assert!(settings.is_auto_sync_enabled()); // Enabled with auto interval
    }

    #[test]
    fn test_serialization() {
        let settings = SyncSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: SyncSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.cloud_sync_enabled, deserialized.cloud_sync_enabled);
        assert_eq!(settings.sync_interval, deserialized.sync_interval);
    }
}
