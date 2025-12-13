/// Temporary File Management
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::io::Result;

/// Manages temporary extraction directories
pub struct TempManager {
    base_dir: PathBuf,
    active_dirs: Vec<PathBuf>,
}

impl TempManager {
    /// Creates new temp manager
    ///
    /// # Returns
    /// * `Result<TempManager>` - New temp manager or I/O error
    pub fn new() -> Result<Self> {
        let base_dir = std::env::temp_dir().join("midi_extraction");
        fs::create_dir_all(&base_dir)?;

        Ok(Self { base_dir, active_dirs: Vec::new() })
    }

    /// Creates a new temporary directory
    ///
    /// # Returns
    /// * `Result<PathBuf>` - Path to created temp directory or I/O error
    pub fn create_temp_dir(&mut self) -> Result<PathBuf> {
        let dir_name = Uuid::new_v4().to_string();
        let temp_dir = self.base_dir.join(dir_name);

        fs::create_dir_all(&temp_dir)?;
        self.active_dirs.push(temp_dir.clone());

        Ok(temp_dir)
    }

    /// Cleans up all temporary directories
    ///
    /// # Returns
    /// * `Result<()>` - Success or I/O error
    pub fn cleanup(&mut self) -> Result<()> {
        for dir in &self.active_dirs {
            if dir.exists() {
                fs::remove_dir_all(dir)?;
            }
        }
        self.active_dirs.clear();
        Ok(())
    }

    /// Returns the base directory for temp files
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Returns count of active temporary directories
    pub fn active_count(&self) -> usize {
        self.active_dirs.len()
    }
}

impl Drop for TempManager {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_temp_dir() {
        let mut manager = TempManager::new().unwrap();
        let temp_dir = manager.create_temp_dir().unwrap();

        assert!(temp_dir.exists());
        assert_eq!(manager.active_count(), 1);

        manager.cleanup().unwrap();
        assert!(!temp_dir.exists());
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_multiple_temp_dirs() {
        let mut manager = TempManager::new().unwrap();

        let dir1 = manager.create_temp_dir().unwrap();
        let dir2 = manager.create_temp_dir().unwrap();

        assert!(dir1.exists());
        assert!(dir2.exists());
        assert_ne!(dir1, dir2);
        assert_eq!(manager.active_count(), 2);

        manager.cleanup().unwrap();
        assert!(!dir1.exists());
        assert!(!dir2.exists());
    }

    #[test]
    fn test_base_dir() {
        let manager = TempManager::new().unwrap();
        let base = manager.base_dir();

        assert!(base.exists());
        // Use to_string_lossy() to avoid unwrap - it's safe for testing
        assert!(base.to_string_lossy().contains("midi_extraction"));
    }

    #[test]
    fn test_cleanup_nonexistent_dir() {
        let mut manager = TempManager::new().unwrap();
        let temp_dir = manager.create_temp_dir().unwrap();

        // Manually remove the directory
        fs::remove_dir_all(&temp_dir).unwrap();

        // Cleanup should not fail even if dir doesn't exist
        assert!(manager.cleanup().is_ok());
    }
}
