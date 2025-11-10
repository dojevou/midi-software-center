use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum WatchMode {
    Disabled,
    #[default]
    ActiveOnly,
    Continuous,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct LibrarySettings {
    pub library_paths: Vec<PathBuf>,
    pub watch_mode: WatchMode,
}


impl LibrarySettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.library_paths.push(path);
        self
    }

    pub fn with_watch_mode(mut self, mode: WatchMode) -> Self {
        self.watch_mode = mode;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        // Check if all paths are absolute
        for path in &self.library_paths {
            if !path.is_absolute() {
                return Err(format!("Library path must be absolute: {:?}", path));
            }
        }

        Ok(())
    }

    pub fn add_library_path(&mut self, path: PathBuf) -> Result<(), String> {
        if !path.is_absolute() {
            return Err("Library path must be absolute".to_string());
        }

        if self.library_paths.contains(&path) {
            return Err("Library path already exists".to_string());
        }

        self.library_paths.push(path);
        Ok(())
    }

    pub fn remove_library_path(&mut self, path: &PathBuf) -> bool {
        if let Some(index) = self.library_paths.iter().position(|p| p == path) {
            self.library_paths.remove(index);
            true
        } else {
            false
        }
    }

    pub fn clear_library_paths(&mut self) {
        self.library_paths.clear();
    }

    pub fn set_watch_mode(&mut self, mode: WatchMode) {
        self.watch_mode = mode;
    }

    pub fn has_library_paths(&self) -> bool {
        !self.library_paths.is_empty()
    }

    pub fn is_watch_enabled(&self) -> bool {
        !matches!(self.watch_mode, WatchMode::Disabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_library_settings() {
        let settings = LibrarySettings::default();

        assert!(settings.library_paths.is_empty());
        assert_eq!(settings.watch_mode, WatchMode::ActiveOnly);
    }

    #[test]
    fn test_builder_pattern() {
        let settings = LibrarySettings::new()
            .with_path(PathBuf::from("/music"))
            .with_watch_mode(WatchMode::Continuous);

        assert_eq!(settings.library_paths.len(), 1);
        assert_eq!(settings.watch_mode, WatchMode::Continuous);
    }

    #[test]
    fn test_validate_success() {
        let mut settings = LibrarySettings::default();
        settings.library_paths.push(PathBuf::from("/absolute/path"));
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_relative_path() {
        let mut settings = LibrarySettings::default();
        settings.library_paths.push(PathBuf::from("relative/path"));
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_add_library_path_valid() {
        let mut settings = LibrarySettings::default();
        let result = settings.add_library_path(PathBuf::from("/music"));
        assert!(result.is_ok());
        assert_eq!(settings.library_paths.len(), 1);
    }

    #[test]
    fn test_add_library_path_relative() {
        let mut settings = LibrarySettings::default();
        let result = settings.add_library_path(PathBuf::from("music"));
        assert!(result.is_err());
    }

    #[test]
    fn test_add_library_path_duplicate() {
        let mut settings = LibrarySettings::default();
        let path = PathBuf::from("/music");

        settings.add_library_path(path.clone()).unwrap();
        let result = settings.add_library_path(path);

        assert!(result.is_err());
        assert_eq!(settings.library_paths.len(), 1);
    }

    #[test]
    fn test_remove_library_path() {
        let mut settings = LibrarySettings::default();
        let path = PathBuf::from("/music");

        settings.add_library_path(path.clone()).unwrap();
        assert!(settings.remove_library_path(&path));
        assert!(settings.library_paths.is_empty());
    }

    #[test]
    fn test_remove_library_path_nonexistent() {
        let mut settings = LibrarySettings::default();
        let path = PathBuf::from("/music");
        assert!(!settings.remove_library_path(&path));
    }

    #[test]
    fn test_clear_library_paths() {
        let mut settings = LibrarySettings::default();
        settings.add_library_path(PathBuf::from("/music1")).unwrap();
        settings.add_library_path(PathBuf::from("/music2")).unwrap();

        settings.clear_library_paths();
        assert!(settings.library_paths.is_empty());
    }

    #[test]
    fn test_set_watch_mode() {
        let mut settings = LibrarySettings::default();
        settings.set_watch_mode(WatchMode::Disabled);
        assert_eq!(settings.watch_mode, WatchMode::Disabled);
    }

    #[test]
    fn test_has_library_paths() {
        let mut settings = LibrarySettings::default();
        assert!(!settings.has_library_paths());

        settings.add_library_path(PathBuf::from("/music")).unwrap();
        assert!(settings.has_library_paths());
    }

    #[test]
    fn test_is_watch_enabled() {
        let mut settings = LibrarySettings::default();
        assert!(settings.is_watch_enabled()); // Default is ActiveOnly

        settings.set_watch_mode(WatchMode::Disabled);
        assert!(!settings.is_watch_enabled());

        settings.set_watch_mode(WatchMode::Continuous);
        assert!(settings.is_watch_enabled());
    }

    #[test]
    fn test_watch_modes() {
        assert_ne!(WatchMode::Disabled, WatchMode::ActiveOnly);
        assert_ne!(WatchMode::Disabled, WatchMode::Continuous);
        assert_eq!(WatchMode::ActiveOnly, WatchMode::default());
    }

    #[test]
    fn test_multiple_paths() {
        let mut settings = LibrarySettings::default();
        settings.add_library_path(PathBuf::from("/music1")).unwrap();
        settings.add_library_path(PathBuf::from("/music2")).unwrap();
        settings.add_library_path(PathBuf::from("/music3")).unwrap();

        assert_eq!(settings.library_paths.len(), 3);
    }
}
