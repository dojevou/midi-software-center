use midi_library_shared::health::{HealthChecker, ServiceHealth, SystemHealth};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

pub struct HealthState {
    checker: Arc<RwLock<HealthChecker>>,
}

impl HealthState {
    pub fn new(checker: HealthChecker) -> Self {
        Self {
            checker: Arc::new(RwLock::new(checker)),
        }
    }
}

#[tauri::command]
pub async fn check_system_health(
    health_state: State<'_, HealthState>,
) -> Result<SystemHealth, String> {
    let checker = health_state.checker.read().await;
    Ok(checker.check_all().await)
}

#[tauri::command]
pub async fn get_cached_health(
    health_state: State<'_, HealthState>,
) -> Result<Option<SystemHealth>, String> {
    let checker = health_state.checker.read().await;
    Ok(checker.get_cached_health().await)
}

#[tauri::command]
pub async fn check_postgres_health(
    health_state: State<'_, HealthState>,
) -> Result<ServiceHealth, String> {
    let health = check_system_health(health_state).await?;
    health
        .get_service("postgresql")
        .cloned()
        .ok_or_else(|| "PostgreSQL not configured".to_string())
}

#[tauri::command]
pub async fn check_meilisearch_health(
    health_state: State<'_, HealthState>,
) -> Result<ServiceHealth, String> {
    let health = check_system_health(health_state).await?;
    health
        .get_service("meilisearch")
        .cloned()
        .ok_or_else(|| "Meilisearch not configured".to_string())
}
