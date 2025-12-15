use super::status::{ServiceHealth, SystemHealth};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

const POSTGRES_LATENCY_WARNING_MS: u64 = 100;
const MEILISEARCH_LATENCY_WARNING_MS: u64 = 200;

pub struct HealthChecker {
    pg_pool: Option<PgPool>,
    meilisearch_url: Option<String>,
    meilisearch_key: Option<String>,
    start_time: Instant,
    last_health: Arc<RwLock<Option<SystemHealth>>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            pg_pool: None,
            meilisearch_url: None,
            meilisearch_key: None,
            start_time: Instant::now(),
            last_health: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_postgres(mut self, pool: PgPool) -> Self {
        self.pg_pool = Some(pool);
        self
    }

    pub fn with_meilisearch(mut self, url: String, key: Option<String>) -> Self {
        self.meilisearch_url = Some(url);
        self.meilisearch_key = key;
        self
    }

    pub async fn check_all(&self) -> SystemHealth {
        let mut services = Vec::new();

        // Check PostgreSQL
        if let Some(pool) = &self.pg_pool {
            services.push(self.check_postgres(pool).await);
        }

        // Check Meilisearch
        if let Some(url) = &self.meilisearch_url {
            services.push(self.check_meilisearch(url).await);
        }

        let uptime = self.start_time.elapsed().as_secs();
        let health = SystemHealth::new(services, uptime);

        // Cache the result
        *self.last_health.write().await = Some(health.clone());

        health
    }

    pub async fn get_cached_health(&self) -> Option<SystemHealth> {
        self.last_health.read().await.clone()
    }

    async fn check_postgres(&self, pool: &PgPool) -> ServiceHealth {
        let start = Instant::now();

        match sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(pool).await {
            Ok(_) => {
                let latency = start.elapsed();
                let latency_ms = latency.as_millis() as u64;

                // Get additional details
                let details = self.get_postgres_details(pool).await;

                if latency_ms > POSTGRES_LATENCY_WARNING_MS {
                    warn!(latency_ms = latency_ms, "PostgreSQL responding slowly");
                    ServiceHealth::degraded(
                        "postgresql",
                        latency,
                        format!("High latency: {}ms", latency_ms),
                    )
                    .with_details(details)
                } else {
                    debug!(latency_ms = latency_ms, "PostgreSQL health check passed");
                    ServiceHealth::healthy("postgresql", latency).with_details(details)
                }
            },
            Err(e) => {
                error!(error = %e, "PostgreSQL health check failed");
                ServiceHealth::unhealthy("postgresql", e.to_string())
            },
        }
    }

    async fn get_postgres_details(&self, pool: &PgPool) -> serde_json::Value {
        #[derive(sqlx::FromRow)]
        struct PgStats {
            active_connections: Option<i64>,
            database_size: Option<String>,
        }

        let stats = sqlx::query_as::<_, PgStats>(
            r#"
            SELECT
                (SELECT count(*) FROM pg_stat_activity WHERE state = 'active') as active_connections,
                pg_size_pretty(pg_database_size(current_database())) as database_size
            "#,
        )
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        match stats {
            Some(s) => serde_json::json!({
                "active_connections": s.active_connections,
                "database_size": s.database_size,
            }),
            None => serde_json::json!({}),
        }
    }

    async fn check_meilisearch(&self, url: &str) -> ServiceHealth {
        let start = Instant::now();
        let health_url = format!("{}/health", url.trim_end_matches('/'));

        let client = reqwest::Client::new();
        let mut request = client.get(&health_url).timeout(Duration::from_secs(5));

        if let Some(key) = &self.meilisearch_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        match request.send().await {
            Ok(response) => {
                let latency = start.elapsed();
                let latency_ms = latency.as_millis() as u64;

                if response.status().is_success() {
                    // Get index stats
                    let details = self.get_meilisearch_details(url).await;

                    if latency_ms > MEILISEARCH_LATENCY_WARNING_MS {
                        warn!(latency_ms = latency_ms, "Meilisearch responding slowly");
                        ServiceHealth::degraded(
                            "meilisearch",
                            latency,
                            format!("High latency: {}ms", latency_ms),
                        )
                        .with_details(details)
                    } else {
                        debug!(latency_ms = latency_ms, "Meilisearch health check passed");
                        ServiceHealth::healthy("meilisearch", latency).with_details(details)
                    }
                } else {
                    let status = response.status();
                    error!(status = %status, "Meilisearch returned error status");
                    ServiceHealth::unhealthy("meilisearch", format!("HTTP {}", status))
                }
            },
            Err(e) => {
                error!(error = %e, "Meilisearch health check failed");
                ServiceHealth::unhealthy("meilisearch", e.to_string())
            },
        }
    }

    async fn get_meilisearch_details(&self, url: &str) -> serde_json::Value {
        let stats_url = format!("{}/stats", url.trim_end_matches('/'));
        let client = reqwest::Client::new();
        let mut request = client.get(&stats_url).timeout(Duration::from_secs(5));

        if let Some(key) = &self.meilisearch_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        match request.send().await {
            Ok(response) if response.status().is_success() => {
                response.json().await.unwrap_or(serde_json::json!({}))
            },
            _ => serde_json::json!({}),
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}
