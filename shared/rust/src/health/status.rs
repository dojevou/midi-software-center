use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    pub last_check: i64,
    pub error: Option<String>,
    pub details: Option<serde_json::Value>,
}

impl ServiceHealth {
    pub fn healthy(name: impl Into<String>, latency: Duration) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            latency_ms: Some(latency.as_millis() as u64),
            last_check: chrono::Utc::now().timestamp_millis(),
            error: None,
            details: None,
        }
    }

    pub fn unhealthy(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            latency_ms: None,
            last_check: chrono::Utc::now().timestamp_millis(),
            error: Some(error.into()),
            details: None,
        }
    }

    pub fn degraded(name: impl Into<String>, latency: Duration, warning: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            latency_ms: Some(latency.as_millis() as u64),
            last_check: chrono::Utc::now().timestamp_millis(),
            error: Some(warning.into()),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: HealthStatus,
    pub services: Vec<ServiceHealth>,
    pub timestamp: i64,
    pub uptime_seconds: u64,
}

impl SystemHealth {
    pub fn new(services: Vec<ServiceHealth>, uptime_seconds: u64) -> Self {
        let overall_status = Self::calculate_overall_status(&services);
        Self {
            overall_status,
            services,
            timestamp: chrono::Utc::now().timestamp_millis(),
            uptime_seconds,
        }
    }

    fn calculate_overall_status(services: &[ServiceHealth]) -> HealthStatus {
        if services.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut has_degraded = false;
        for service in services {
            match service.status {
                HealthStatus::Unhealthy => return HealthStatus::Unhealthy,
                HealthStatus::Degraded => has_degraded = true,
                _ => {}
            }
        }

        if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.overall_status, HealthStatus::Healthy)
    }

    pub fn get_service(&self, name: &str) -> Option<&ServiceHealth> {
        self.services.iter().find(|s| s.name == name)
    }
}
