use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum KeybindingProfile {
    #[default]
    Default,
    Ableton,
    ProTools,
    StudioOne,
    FLStudio,
    Logic,
    Reaper,
}

impl KeybindingProfile {
    pub fn all_profiles() -> Vec<KeybindingProfile> {
        vec![
            KeybindingProfile::Default,
            KeybindingProfile::Ableton,
            KeybindingProfile::ProTools,
            KeybindingProfile::StudioOne,
            KeybindingProfile::FLStudio,
            KeybindingProfile::Logic,
            KeybindingProfile::Reaper,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            KeybindingProfile::Default => "Default",
            KeybindingProfile::Ableton => "Ableton Live",
            KeybindingProfile::ProTools => "Pro Tools",
            KeybindingProfile::StudioOne => "Studio One",
            KeybindingProfile::FLStudio => "FL Studio",
            KeybindingProfile::Logic => "Logic Pro",
            KeybindingProfile::Reaper => "Reaper",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyboardSettings {
    pub profile: KeybindingProfile,
    pub custom_keybindings: HashMap<String, String>,
}

impl KeyboardSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_profile(mut self, profile: KeybindingProfile) -> Self {
        self.profile = profile;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        // Check for duplicate keybindings
        let mut seen_keys: HashMap<String, String> = HashMap::new();

        for (command_id, keybinding) in &self.custom_keybindings {
            if let Some(existing_command) = seen_keys.get(keybinding) {
                return Err(format!(
                    "Duplicate keybinding '{}' for commands '{}' and '{}'",
                    keybinding, existing_command, command_id
                ));
            }
            seen_keys.insert(keybinding.clone(), command_id.clone());
        }

        Ok(())
    }

    pub fn set_profile(&mut self, profile: KeybindingProfile) {
        self.profile = profile;
    }

    pub fn set_keybinding(&mut self, command_id: String, keybinding: String) -> Result<(), String> {
        // Check if keybinding is already used by another command
        for (existing_id, existing_key) in &self.custom_keybindings {
            if existing_key == &keybinding && existing_id != &command_id {
                return Err(format!(
                    "Keybinding '{}' is already assigned to command '{}'",
                    keybinding, existing_id
                ));
            }
        }

        self.custom_keybindings.insert(command_id, keybinding);
        Ok(())
    }

    pub fn remove_keybinding(&mut self, command_id: &str) -> bool {
        self.custom_keybindings.remove(command_id).is_some()
    }

    pub fn get_keybinding(&self, command_id: &str) -> Option<&String> {
        self.custom_keybindings.get(command_id)
    }

    pub fn reset_to_defaults(&mut self) {
        self.custom_keybindings.clear();
        self.profile = KeybindingProfile::default();
    }

    pub fn import_profile(&mut self, profile: KeybindingProfile) -> Result<(), String> {
        self.custom_keybindings.clear();
        self.profile = profile;

        // Load profile-specific keybindings
        match profile {
            KeybindingProfile::Default => {
                // Use default keybindings (already defined in command_palette)
            },
            KeybindingProfile::Ableton => {
                self.custom_keybindings
                    .insert("transport.play".to_string(), "Space".to_string());
                self.custom_keybindings.insert("transport.record".to_string(), "F9".to_string());
                self.custom_keybindings
                    .insert("track.new".to_string(), "Ctrl+Shift+T".to_string());
            },
            KeybindingProfile::ProTools => {
                self.custom_keybindings
                    .insert("transport.play".to_string(), "Space".to_string());
                self.custom_keybindings
                    .insert("transport.record".to_string(), "F12".to_string());
                self.custom_keybindings
                    .insert("track.new".to_string(), "Ctrl+Shift+N".to_string());
            },
            KeybindingProfile::StudioOne => {
                self.custom_keybindings
                    .insert("transport.play".to_string(), "Space".to_string());
                self.custom_keybindings
                    .insert("transport.record".to_string(), "Num*".to_string());
                self.custom_keybindings.insert("track.new".to_string(), "T".to_string());
            },
            KeybindingProfile::FLStudio => {
                self.custom_keybindings
                    .insert("transport.play".to_string(), "Space".to_string());
                self.custom_keybindings
                    .insert("transport.record".to_string(), "Ctrl+R".to_string());
                self.custom_keybindings
                    .insert("track.new".to_string(), "Ctrl+Shift+A".to_string());
            },
            KeybindingProfile::Logic => {
                self.custom_keybindings
                    .insert("transport.play".to_string(), "Space".to_string());
                self.custom_keybindings.insert("transport.record".to_string(), "R".to_string());
                self.custom_keybindings.insert("track.new".to_string(), "Ctrl+N".to_string());
            },
            KeybindingProfile::Reaper => {
                self.custom_keybindings
                    .insert("transport.play".to_string(), "Space".to_string());
                self.custom_keybindings
                    .insert("transport.record".to_string(), "Ctrl+R".to_string());
                self.custom_keybindings.insert("track.new".to_string(), "Ctrl+T".to_string());
            },
        }

        Ok(())
    }

    pub fn has_conflicts(&self) -> bool {
        let mut seen_keys: HashMap<String, String> = HashMap::new();

        for (command_id, keybinding) in &self.custom_keybindings {
            if seen_keys.contains_key(keybinding) {
                return true;
            }
            seen_keys.insert(keybinding.clone(), command_id.clone());
        }

        false
    }

    pub fn find_conflicts(&self) -> Vec<(String, String, String)> {
        let mut conflicts = Vec::new();
        let mut seen_keys: HashMap<String, String> = HashMap::new();

        for (command_id, keybinding) in &self.custom_keybindings {
            if let Some(existing_command) = seen_keys.get(keybinding) {
                conflicts.push((
                    keybinding.clone(),
                    existing_command.clone(),
                    command_id.clone(),
                ));
            } else {
                seen_keys.insert(keybinding.clone(), command_id.clone());
            }
        }

        conflicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_keyboard_settings() {
        let settings = KeyboardSettings::default();
        assert_eq!(settings.profile, KeybindingProfile::Default);
        assert!(settings.custom_keybindings.is_empty());
    }

    #[test]
    fn test_profile_names() {
        assert_eq!(KeybindingProfile::Default.name(), "Default");
        assert_eq!(KeybindingProfile::Ableton.name(), "Ableton Live");
        assert_eq!(KeybindingProfile::ProTools.name(), "Pro Tools");
        assert_eq!(KeybindingProfile::StudioOne.name(), "Studio One");
        assert_eq!(KeybindingProfile::FLStudio.name(), "FL Studio");
        assert_eq!(KeybindingProfile::Logic.name(), "Logic Pro");
        assert_eq!(KeybindingProfile::Reaper.name(), "Reaper");
    }

    #[test]
    fn test_all_profiles() {
        let profiles = KeybindingProfile::all_profiles();
        assert_eq!(profiles.len(), 7);
    }

    #[test]
    fn test_builder_pattern() {
        let settings = KeyboardSettings::new().with_profile(KeybindingProfile::Ableton);
        assert_eq!(settings.profile, KeybindingProfile::Ableton);
    }

    #[test]
    fn test_set_profile() {
        let mut settings = KeyboardSettings::default();
        settings.set_profile(KeybindingProfile::Logic);
        assert_eq!(settings.profile, KeybindingProfile::Logic);
    }

    #[test]
    fn test_set_keybinding() {
        let mut settings = KeyboardSettings::default();
        let result = settings.set_keybinding("transport.play".to_string(), "Space".to_string());
        assert!(result.is_ok());
        assert_eq!(
            settings.get_keybinding("transport.play"),
            Some(&"Space".to_string())
        );
    }

    #[test]
    fn test_set_keybinding_duplicate() {
        let mut settings = KeyboardSettings::default();
        settings
            .set_keybinding("transport.play".to_string(), "Space".to_string())
            .unwrap();

        let result = settings.set_keybinding("transport.stop".to_string(), "Space".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_keybinding() {
        let mut settings = KeyboardSettings::default();
        settings
            .set_keybinding("transport.play".to_string(), "Space".to_string())
            .unwrap();

        assert!(settings.remove_keybinding("transport.play"));
        assert!(settings.get_keybinding("transport.play").is_none());
    }

    #[test]
    fn test_remove_keybinding_nonexistent() {
        let mut settings = KeyboardSettings::default();
        assert!(!settings.remove_keybinding("nonexistent.command"));
    }

    #[test]
    fn test_get_keybinding() {
        let mut settings = KeyboardSettings::default();
        settings
            .set_keybinding("transport.play".to_string(), "Space".to_string())
            .unwrap();

        assert_eq!(
            settings.get_keybinding("transport.play"),
            Some(&"Space".to_string())
        );
        assert_eq!(settings.get_keybinding("nonexistent"), None);
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut settings = KeyboardSettings::default();
        settings
            .set_keybinding("transport.play".to_string(), "Space".to_string())
            .unwrap();
        settings.set_profile(KeybindingProfile::Ableton);

        settings.reset_to_defaults();

        assert_eq!(settings.profile, KeybindingProfile::Default);
        assert!(settings.custom_keybindings.is_empty());
    }

    #[test]
    fn test_import_profile_default() {
        let mut settings = KeyboardSettings::default();
        assert!(settings.import_profile(KeybindingProfile::Default).is_ok());
        assert_eq!(settings.profile, KeybindingProfile::Default);
    }

    #[test]
    fn test_import_profile_ableton() {
        let mut settings = KeyboardSettings::default();
        assert!(settings.import_profile(KeybindingProfile::Ableton).is_ok());
        assert_eq!(settings.profile, KeybindingProfile::Ableton);
        assert!(!settings.custom_keybindings.is_empty());
    }

    #[test]
    fn test_import_profile_clears_existing() {
        let mut settings = KeyboardSettings::default();
        settings
            .set_keybinding("custom.command".to_string(), "Ctrl+X".to_string())
            .unwrap();

        settings.import_profile(KeybindingProfile::Logic).unwrap();

        assert!(settings.get_keybinding("custom.command").is_none());
    }

    #[test]
    fn test_validate_no_conflicts() {
        let mut settings = KeyboardSettings::default();
        settings
            .set_keybinding("transport.play".to_string(), "Space".to_string())
            .unwrap();
        settings
            .set_keybinding("transport.stop".to_string(), "Ctrl+S".to_string())
            .unwrap();

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_has_conflicts_false() {
        let mut settings = KeyboardSettings::default();
        settings
            .set_keybinding("transport.play".to_string(), "Space".to_string())
            .unwrap();
        settings
            .set_keybinding("transport.stop".to_string(), "Ctrl+S".to_string())
            .unwrap();

        assert!(!settings.has_conflicts());
    }

    #[test]
    fn test_find_conflicts_empty() {
        let mut settings = KeyboardSettings::default();
        settings
            .set_keybinding("transport.play".to_string(), "Space".to_string())
            .unwrap();

        let conflicts = settings.find_conflicts();
        assert!(conflicts.is_empty());
    }
}
