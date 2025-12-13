use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DataRetentionPolicy {
    Days7,
    #[default]
    Days30,
    Days90,
}

impl DataRetentionPolicy {
    pub fn as_days(&self) -> u32 {
        match self {
            DataRetentionPolicy::Days7 => 7,
            DataRetentionPolicy::Days30 => 30,
            DataRetentionPolicy::Days90 => 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub analytics_enabled: bool,
    pub crash_reporting_enabled: bool,
    pub usage_tracking_enabled: bool,
    pub data_retention_policy: DataRetentionPolicy,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            analytics_enabled: true,
            crash_reporting_enabled: true,
            usage_tracking_enabled: true,
            data_retention_policy: DataRetentionPolicy::default(),
        }
    }
}

impl PrivacySettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_all_disabled(mut self) -> Self {
        self.analytics_enabled = false;
        self.crash_reporting_enabled = false;
        self.usage_tracking_enabled = false;
        self
    }

    pub fn with_retention_policy(mut self, policy: DataRetentionPolicy) -> Self {
        self.data_retention_policy = policy;
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        // No additional validation needed
        Ok(())
    }

    pub fn enable_analytics(&mut self) {
        self.analytics_enabled = true;
    }

    pub fn disable_analytics(&mut self) {
        self.analytics_enabled = false;
    }

    pub fn enable_crash_reporting(&mut self) {
        self.crash_reporting_enabled = true;
    }

    pub fn disable_crash_reporting(&mut self) {
        self.crash_reporting_enabled = false;
    }

    pub fn enable_usage_tracking(&mut self) {
        self.usage_tracking_enabled = true;
    }

    pub fn disable_usage_tracking(&mut self) {
        self.usage_tracking_enabled = false;
    }

    pub fn set_data_retention_policy(&mut self, policy: DataRetentionPolicy) {
        self.data_retention_policy = policy;
    }

    pub fn disable_all_tracking(&mut self) {
        self.analytics_enabled = false;
        self.crash_reporting_enabled = false;
        self.usage_tracking_enabled = false;
    }

    pub fn enable_all_tracking(&mut self) {
        self.analytics_enabled = true;
        self.crash_reporting_enabled = true;
        self.usage_tracking_enabled = true;
    }

    pub fn is_any_tracking_enabled(&self) -> bool {
        self.analytics_enabled || self.crash_reporting_enabled || self.usage_tracking_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_privacy_settings() {
        let settings = PrivacySettings::default();

        assert!(settings.analytics_enabled);
        assert!(settings.crash_reporting_enabled);
        assert!(settings.usage_tracking_enabled);
        assert_eq!(settings.data_retention_policy, DataRetentionPolicy::Days30);
    }

    #[test]
    fn test_retention_policy_days() {
        assert_eq!(DataRetentionPolicy::Days7.as_days(), 7);
        assert_eq!(DataRetentionPolicy::Days30.as_days(), 30);
        assert_eq!(DataRetentionPolicy::Days90.as_days(), 90);
    }

    #[test]
    fn test_builder_pattern() {
        let settings = PrivacySettings::new()
            .with_all_disabled()
            .with_retention_policy(DataRetentionPolicy::Days7);

        assert!(!settings.analytics_enabled);
        assert!(!settings.crash_reporting_enabled);
        assert!(!settings.usage_tracking_enabled);
        assert_eq!(settings.data_retention_policy, DataRetentionPolicy::Days7);
    }

    #[test]
    fn test_validate() {
        let settings = PrivacySettings::default();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_enable_analytics() {
        let mut settings = PrivacySettings::default();
        settings.disable_analytics();
        settings.enable_analytics();
        assert!(settings.analytics_enabled);
    }

    #[test]
    fn test_disable_analytics() {
        let mut settings = PrivacySettings::default();
        settings.disable_analytics();
        assert!(!settings.analytics_enabled);
    }

    #[test]
    fn test_enable_crash_reporting() {
        let mut settings = PrivacySettings::default();
        settings.disable_crash_reporting();
        settings.enable_crash_reporting();
        assert!(settings.crash_reporting_enabled);
    }

    #[test]
    fn test_disable_crash_reporting() {
        let mut settings = PrivacySettings::default();
        settings.disable_crash_reporting();
        assert!(!settings.crash_reporting_enabled);
    }

    #[test]
    fn test_enable_usage_tracking() {
        let mut settings = PrivacySettings::default();
        settings.disable_usage_tracking();
        settings.enable_usage_tracking();
        assert!(settings.usage_tracking_enabled);
    }

    #[test]
    fn test_disable_usage_tracking() {
        let mut settings = PrivacySettings::default();
        settings.disable_usage_tracking();
        assert!(!settings.usage_tracking_enabled);
    }

    #[test]
    fn test_set_data_retention_policy() {
        let mut settings = PrivacySettings::default();
        settings.set_data_retention_policy(DataRetentionPolicy::Days90);
        assert_eq!(settings.data_retention_policy, DataRetentionPolicy::Days90);
    }

    #[test]
    fn test_disable_all_tracking() {
        let mut settings = PrivacySettings::default();
        settings.disable_all_tracking();

        assert!(!settings.analytics_enabled);
        assert!(!settings.crash_reporting_enabled);
        assert!(!settings.usage_tracking_enabled);
    }

    #[test]
    fn test_enable_all_tracking() {
        let mut settings = PrivacySettings::new().with_all_disabled();
        settings.enable_all_tracking();

        assert!(settings.analytics_enabled);
        assert!(settings.crash_reporting_enabled);
        assert!(settings.usage_tracking_enabled);
    }

    #[test]
    fn test_is_any_tracking_enabled() {
        let mut settings = PrivacySettings::default();
        assert!(settings.is_any_tracking_enabled());

        settings.disable_all_tracking();
        assert!(!settings.is_any_tracking_enabled());

        settings.enable_analytics();
        assert!(settings.is_any_tracking_enabled());
    }

    #[test]
    fn test_serialization() {
        let settings = PrivacySettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: PrivacySettings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings.analytics_enabled, deserialized.analytics_enabled);
        assert_eq!(
            settings.data_retention_policy,
            deserialized.data_retention_policy
        );
    }
}
