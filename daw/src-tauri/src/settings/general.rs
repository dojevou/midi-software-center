use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Japanese,
    Chinese,
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StartupBehavior {
    ShowSplashScreen,
    OpenLastProject,
    ShowStartPage,
}

impl Default for StartupBehavior {
    fn default() -> Self {
        StartupBehavior::ShowStartPage
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub theme: Theme,
    pub language: Language,
    pub auto_save_enabled: bool,
    pub auto_save_interval_minutes: u32,
    pub check_for_updates: bool,
    pub startup_behavior: StartupBehavior,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            language: Language::default(),
            auto_save_enabled: true,
            auto_save_interval_minutes: 5,
            check_for_updates: true,
            startup_behavior: StartupBehavior::default(),
        }
    }
}

impl GeneralSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    pub fn with_auto_save(mut self, enabled: bool, interval_minutes: u32) -> Self {
        self.auto_save_enabled = enabled;
        self.auto_save_interval_minutes = interval_minutes;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.auto_save_interval_minutes == 0 {
            return Err("Auto-save interval must be greater than 0".to_string());
        }

        if self.auto_save_interval_minutes > 60 {
            return Err("Auto-save interval must be 60 minutes or less".to_string());
        }

        Ok(())
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    pub fn set_auto_save_enabled(&mut self, enabled: bool) {
        self.auto_save_enabled = enabled;
    }

    pub fn set_auto_save_interval(&mut self, minutes: u32) -> Result<(), String> {
        if minutes == 0 {
            return Err("Auto-save interval must be greater than 0".to_string());
        }
        if minutes > 60 {
            return Err("Auto-save interval must be 60 minutes or less".to_string());
        }
        self.auto_save_interval_minutes = minutes;
        Ok(())
    }

    pub fn set_check_for_updates(&mut self, enabled: bool) {
        self.check_for_updates = enabled;
    }

    pub fn set_startup_behavior(&mut self, behavior: StartupBehavior) {
        self.startup_behavior = behavior;
    }

    pub fn is_dark_mode(&self) -> bool {
        matches!(self.theme, Theme::Dark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_general_settings() {
        let settings = GeneralSettings::default();

        assert_eq!(settings.theme, Theme::Dark);
        assert_eq!(settings.language, Language::English);
        assert!(settings.auto_save_enabled);
        assert_eq!(settings.auto_save_interval_minutes, 5);
        assert!(settings.check_for_updates);
        assert_eq!(settings.startup_behavior, StartupBehavior::ShowStartPage);
    }

    #[test]
    fn test_builder_pattern() {
        let settings = GeneralSettings::new()
            .with_theme(Theme::Light)
            .with_language(Language::Spanish)
            .with_auto_save(false, 10);

        assert_eq!(settings.theme, Theme::Light);
        assert_eq!(settings.language, Language::Spanish);
        assert!(!settings.auto_save_enabled);
        assert_eq!(settings.auto_save_interval_minutes, 10);
    }

    #[test]
    fn test_validate_success() {
        let settings = GeneralSettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_zero_interval() {
        let mut settings = GeneralSettings::default();
        settings.auto_save_interval_minutes = 0;

        let result = settings.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("greater than 0"));
    }

    #[test]
    fn test_validate_interval_too_large() {
        let mut settings = GeneralSettings::default();
        settings.auto_save_interval_minutes = 61;

        let result = settings.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("60 minutes or less"));
    }

    #[test]
    fn test_set_theme() {
        let mut settings = GeneralSettings::default();
        settings.set_theme(Theme::Light);
        assert_eq!(settings.theme, Theme::Light);
    }

    #[test]
    fn test_set_language() {
        let mut settings = GeneralSettings::default();
        settings.set_language(Language::French);
        assert_eq!(settings.language, Language::French);
    }

    #[test]
    fn test_set_auto_save_enabled() {
        let mut settings = GeneralSettings::default();
        settings.set_auto_save_enabled(false);
        assert!(!settings.auto_save_enabled);
    }

    #[test]
    fn test_set_auto_save_interval_valid() {
        let mut settings = GeneralSettings::default();
        let result = settings.set_auto_save_interval(10);
        assert!(result.is_ok());
        assert_eq!(settings.auto_save_interval_minutes, 10);
    }

    #[test]
    fn test_set_auto_save_interval_zero() {
        let mut settings = GeneralSettings::default();
        let result = settings.set_auto_save_interval(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_auto_save_interval_too_large() {
        let mut settings = GeneralSettings::default();
        let result = settings.set_auto_save_interval(61);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_check_for_updates() {
        let mut settings = GeneralSettings::default();
        settings.set_check_for_updates(false);
        assert!(!settings.check_for_updates);
    }

    #[test]
    fn test_set_startup_behavior() {
        let mut settings = GeneralSettings::default();
        settings.set_startup_behavior(StartupBehavior::OpenLastProject);
        assert_eq!(settings.startup_behavior, StartupBehavior::OpenLastProject);
    }

    #[test]
    fn test_is_dark_mode() {
        let mut settings = GeneralSettings::default();
        assert!(settings.is_dark_mode());

        settings.set_theme(Theme::Light);
        assert!(!settings.is_dark_mode());
    }
}
