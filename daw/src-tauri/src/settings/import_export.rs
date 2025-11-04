use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DuplicateHandling {
    KeepFirst,
    KeepLast,
    Skip,
}

impl Default for DuplicateHandling {
    fn default() -> Self {
        DuplicateHandling::Skip
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportExportSettings {
    pub auto_tag_on_import: bool,
    pub analyze_bpm_on_import: bool,
    pub analyze_key_on_import: bool,
    pub nested_archive_depth_limit: u32,
    pub skip_patterns: Vec<String>,
    pub duplicate_handling: DuplicateHandling,
}

impl Default for ImportExportSettings {
    fn default() -> Self {
        Self {
            auto_tag_on_import: true,
            analyze_bpm_on_import: true,
            analyze_key_on_import: true,
            nested_archive_depth_limit: 3,
            skip_patterns: vec![
                "*.tmp".to_string(),
                "*.bak".to_string(),
                "*.log".to_string(),
            ],
            duplicate_handling: DuplicateHandling::default(),
        }
    }
}

impl ImportExportSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_auto_analysis(mut self, enabled: bool) -> Self {
        self.auto_tag_on_import = enabled;
        self.analyze_bpm_on_import = enabled;
        self.analyze_key_on_import = enabled;
        self
    }

    pub fn with_nested_depth_limit(mut self, limit: u32) -> Self {
        self.nested_archive_depth_limit = limit;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.nested_archive_depth_limit > 5 {
            return Err("Nested archive depth limit must be 5 or less".to_string());
        }

        Ok(())
    }

    pub fn set_auto_tag(&mut self, enabled: bool) {
        self.auto_tag_on_import = enabled;
    }

    pub fn set_analyze_bpm(&mut self, enabled: bool) {
        self.analyze_bpm_on_import = enabled;
    }

    pub fn set_analyze_key(&mut self, enabled: bool) {
        self.analyze_key_on_import = enabled;
    }

    pub fn set_nested_depth_limit(&mut self, limit: u32) -> Result<(), String> {
        if limit > 5 {
            return Err("Nested archive depth limit must be 5 or less".to_string());
        }
        self.nested_archive_depth_limit = limit;
        Ok(())
    }

    pub fn add_skip_pattern(&mut self, pattern: String) {
        if !self.skip_patterns.contains(&pattern) {
            self.skip_patterns.push(pattern);
        }
    }

    pub fn remove_skip_pattern(&mut self, pattern: &str) -> bool {
        if let Some(index) = self.skip_patterns.iter().position(|p| p == pattern) {
            self.skip_patterns.remove(index);
            true
        } else {
            false
        }
    }

    pub fn clear_skip_patterns(&mut self) {
        self.skip_patterns.clear();
    }

    pub fn set_duplicate_handling(&mut self, handling: DuplicateHandling) {
        self.duplicate_handling = handling;
    }

    pub fn should_analyze_on_import(&self) -> bool {
        self.auto_tag_on_import || self.analyze_bpm_on_import || self.analyze_key_on_import
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_import_export_settings() {
        let settings = ImportExportSettings::default();

        assert!(settings.auto_tag_on_import);
        assert!(settings.analyze_bpm_on_import);
        assert!(settings.analyze_key_on_import);
        assert_eq!(settings.nested_archive_depth_limit, 3);
        assert_eq!(settings.skip_patterns.len(), 3);
        assert_eq!(settings.duplicate_handling, DuplicateHandling::Skip);
    }

    #[test]
    fn test_builder_pattern() {
        let settings = ImportExportSettings::new()
            .with_auto_analysis(false)
            .with_nested_depth_limit(5);

        assert!(!settings.auto_tag_on_import);
        assert!(!settings.analyze_bpm_on_import);
        assert!(!settings.analyze_key_on_import);
        assert_eq!(settings.nested_archive_depth_limit, 5);
    }

    #[test]
    fn test_validate_success() {
        let settings = ImportExportSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_depth_limit_too_high() {
        let mut settings = ImportExportSettings::default();
        settings.nested_archive_depth_limit = 6;
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_set_auto_tag() {
        let mut settings = ImportExportSettings::default();
        settings.set_auto_tag(false);
        assert!(!settings.auto_tag_on_import);
    }

    #[test]
    fn test_set_analyze_bpm() {
        let mut settings = ImportExportSettings::default();
        settings.set_analyze_bpm(false);
        assert!(!settings.analyze_bpm_on_import);
    }

    #[test]
    fn test_set_analyze_key() {
        let mut settings = ImportExportSettings::default();
        settings.set_analyze_key(false);
        assert!(!settings.analyze_key_on_import);
    }

    #[test]
    fn test_set_nested_depth_limit_valid() {
        let mut settings = ImportExportSettings::default();
        assert!(settings.set_nested_depth_limit(5).is_ok());
        assert_eq!(settings.nested_archive_depth_limit, 5);
    }

    #[test]
    fn test_set_nested_depth_limit_invalid() {
        let mut settings = ImportExportSettings::default();
        assert!(settings.set_nested_depth_limit(6).is_err());
    }

    #[test]
    fn test_add_skip_pattern() {
        let mut settings = ImportExportSettings::default();
        let initial_count = settings.skip_patterns.len();

        settings.add_skip_pattern("*.txt".to_string());
        assert_eq!(settings.skip_patterns.len(), initial_count + 1);
        assert!(settings.skip_patterns.contains(&"*.txt".to_string()));
    }

    #[test]
    fn test_add_skip_pattern_duplicate() {
        let mut settings = ImportExportSettings::default();
        settings.add_skip_pattern("*.txt".to_string());
        let count_after_first = settings.skip_patterns.len();

        settings.add_skip_pattern("*.txt".to_string());
        assert_eq!(settings.skip_patterns.len(), count_after_first);
    }

    #[test]
    fn test_remove_skip_pattern() {
        let mut settings = ImportExportSettings::default();
        assert!(settings.remove_skip_pattern("*.tmp"));
        assert!(!settings.skip_patterns.contains(&"*.tmp".to_string()));
    }

    #[test]
    fn test_remove_skip_pattern_nonexistent() {
        let mut settings = ImportExportSettings::default();
        assert!(!settings.remove_skip_pattern("*.xyz"));
    }

    #[test]
    fn test_clear_skip_patterns() {
        let mut settings = ImportExportSettings::default();
        settings.clear_skip_patterns();
        assert!(settings.skip_patterns.is_empty());
    }

    #[test]
    fn test_set_duplicate_handling() {
        let mut settings = ImportExportSettings::default();
        settings.set_duplicate_handling(DuplicateHandling::KeepFirst);
        assert_eq!(settings.duplicate_handling, DuplicateHandling::KeepFirst);
    }

    #[test]
    fn test_should_analyze_on_import() {
        let mut settings = ImportExportSettings::default();
        assert!(settings.should_analyze_on_import());

        settings.set_auto_tag(false);
        settings.set_analyze_bpm(false);
        settings.set_analyze_key(false);
        assert!(!settings.should_analyze_on_import());
    }

    #[test]
    fn test_duplicate_handling_variants() {
        assert_ne!(DuplicateHandling::KeepFirst, DuplicateHandling::KeepLast);
        assert_ne!(DuplicateHandling::KeepFirst, DuplicateHandling::Skip);
        assert_eq!(DuplicateHandling::Skip, DuplicateHandling::default());
    }
}
