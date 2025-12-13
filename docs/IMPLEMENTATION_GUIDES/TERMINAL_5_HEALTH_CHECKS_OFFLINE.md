# Terminal 5: Health Checks & Offline Mode Implementation Guide

## Overview
This guide covers implementing service health monitoring for PostgreSQL/Meilisearch and an offline caching strategy when database services are unavailable.

---

## Part 1: Health Checks System

### 1.1 Rust Backend - Health Monitor Module

**File: `shared/rust/src/health/mod.rs`**
```rust
pub mod checker;
pub mod status;

pub use checker::HealthChecker;
pub use status::{HealthStatus, ServiceHealth, SystemHealth};
```

**File: `shared/rust/src/health/status.rs`**
```rust
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
```

**File: `shared/rust/src/health/checker.rs`**
```rust
use super::status::{HealthStatus, ServiceHealth, SystemHealth};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

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

        match sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(pool)
            .await
        {
            Ok(_) => {
                let latency = start.elapsed();
                let latency_ms = latency.as_millis() as u64;

                // Get additional details
                let details = self.get_postgres_details(pool).await;

                if latency_ms > POSTGRES_LATENCY_WARNING_MS {
                    warn!(
                        latency_ms = latency_ms,
                        "PostgreSQL responding slowly"
                    );
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
            }
            Err(e) => {
                error!(error = %e, "PostgreSQL health check failed");
                ServiceHealth::unhealthy("postgresql", e.to_string())
            }
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
                        warn!(
                            latency_ms = latency_ms,
                            "Meilisearch responding slowly"
                        );
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
                    ServiceHealth::unhealthy(
                        "meilisearch",
                        format!("HTTP {}", status),
                    )
                }
            }
            Err(e) => {
                error!(error = %e, "Meilisearch health check failed");
                ServiceHealth::unhealthy("meilisearch", e.to_string())
            }
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
            }
            _ => serde_json::json!({}),
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}
```

### 1.2 Tauri Commands for Health Checks

**File: `pipeline/src-tauri/src/commands/health.rs`**
```rust
use shared::health::{HealthChecker, HealthStatus, ServiceHealth, SystemHealth};
use tauri::State;
use std::sync::Arc;
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
```

### 1.3 Frontend Health Store

**File: `app/src/lib/stores/healthStore.ts`**
```typescript
import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Types
export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy' | 'unknown';

export interface ServiceHealth {
  name: string;
  status: HealthStatus;
  latency_ms: number | null;
  last_check: number;
  error: string | null;
  details: Record<string, unknown> | null;
}

export interface SystemHealth {
  overall_status: HealthStatus;
  services: ServiceHealth[];
  timestamp: number;
  uptime_seconds: number;
}

interface HealthState {
  health: SystemHealth | null;
  isChecking: boolean;
  lastError: string | null;
  checkInterval: number | null;
}

// Constants
const DEFAULT_CHECK_INTERVAL = 30000; // 30 seconds
const UNHEALTHY_CHECK_INTERVAL = 5000; // 5 seconds when unhealthy

// Store
function createHealthStore() {
  const { subscribe, set, update } = writable<HealthState>({
    health: null,
    isChecking: false,
    lastError: null,
    checkInterval: null,
  });

  let intervalId: ReturnType<typeof setInterval> | null = null;

  async function checkHealth(): Promise<SystemHealth | null> {
    update(s => ({ ...s, isChecking: true, lastError: null }));

    try {
      const health = await invoke<SystemHealth>('check_system_health');
      update(s => ({ ...s, health, isChecking: false }));

      // Emit event for other stores to react
      window.dispatchEvent(new CustomEvent('health-updated', { detail: health }));

      return health;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      update(s => ({ ...s, isChecking: false, lastError: message }));
      return null;
    }
  }

  async function getCachedHealth(): Promise<SystemHealth | null> {
    try {
      return await invoke<SystemHealth | null>('get_cached_health');
    } catch {
      return null;
    }
  }

  function startMonitoring(interval: number = DEFAULT_CHECK_INTERVAL): void {
    stopMonitoring();

    // Initial check
    checkHealth();

    // Setup interval
    intervalId = setInterval(async () => {
      const health = await checkHealth();

      // Increase check frequency when unhealthy
      if (health && health.overall_status === 'unhealthy') {
        if (get({ subscribe }).checkInterval !== UNHEALTHY_CHECK_INTERVAL) {
          startMonitoring(UNHEALTHY_CHECK_INTERVAL);
        }
      } else if (get({ subscribe }).checkInterval !== interval) {
        startMonitoring(interval);
      }
    }, interval);

    update(s => ({ ...s, checkInterval: interval }));
  }

  function stopMonitoring(): void {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
    update(s => ({ ...s, checkInterval: null }));
  }

  function getServiceStatus(serviceName: string): ServiceHealth | null {
    const state = get({ subscribe });
    return state.health?.services.find(s => s.name === serviceName) || null;
  }

  return {
    subscribe,
    checkHealth,
    getCachedHealth,
    startMonitoring,
    stopMonitoring,
    getServiceStatus,

    // Cleanup
    destroy: () => {
      stopMonitoring();
    },
  };
}

export const healthStore = createHealthStore();

// Derived stores for specific services
export const postgresHealth = derived(healthStore, ($health) =>
  $health.health?.services.find(s => s.name === 'postgresql') || null
);

export const meilisearchHealth = derived(healthStore, ($health) =>
  $health.health?.services.find(s => s.name === 'meilisearch') || null
);

export const overallHealth = derived(healthStore, ($health) =>
  $health.health?.overall_status || 'unknown'
);

export const isSystemHealthy = derived(healthStore, ($health) =>
  $health.health?.overall_status === 'healthy'
);
```

### 1.4 Health Status Components

**File: `app/src/lib/components/health/HealthIndicator.svelte`**
```svelte
<script lang="ts">
  import { healthStore, type HealthStatus, type ServiceHealth } from '$lib/stores/healthStore';
  import { onMount, onDestroy } from 'svelte';
  import { fade, scale } from 'svelte/transition';

  export let showDetails = false;
  export let compact = false;

  let expanded = false;

  const statusColors: Record<HealthStatus, string> = {
    healthy: 'var(--color-success, #22c55e)',
    degraded: 'var(--color-warning, #eab308)',
    unhealthy: 'var(--color-error, #ef4444)',
    unknown: 'var(--color-muted, #6b7280)',
  };

  const statusLabels: Record<HealthStatus, string> = {
    healthy: 'All Systems Operational',
    degraded: 'Degraded Performance',
    unhealthy: 'Service Unavailable',
    unknown: 'Checking...',
  };

  function getStatusIcon(status: HealthStatus): string {
    switch (status) {
      case 'healthy': return '●';
      case 'degraded': return '◐';
      case 'unhealthy': return '○';
      default: return '◌';
    }
  }

  function formatLatency(ms: number | null): string {
    if (ms === null) return 'N/A';
    if (ms < 1) return '<1ms';
    return `${ms}ms`;
  }

  function formatUptime(seconds: number): string {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);

    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  }

  onMount(() => {
    healthStore.startMonitoring();
  });

  onDestroy(() => {
    healthStore.stopMonitoring();
  });
</script>

<div
  class="health-indicator"
  class:compact
  role="status"
  aria-live="polite"
>
  <button
    class="health-button"
    on:click={() => expanded = !expanded}
    aria-expanded={expanded}
    aria-label="System health status"
  >
    <span
      class="status-dot"
      style="color: {statusColors[$healthStore.health?.overall_status || 'unknown']}"
    >
      {getStatusIcon($healthStore.health?.overall_status || 'unknown')}
    </span>

    {#if !compact}
      <span class="status-label">
        {statusLabels[$healthStore.health?.overall_status || 'unknown']}
      </span>
    {/if}

    {#if $healthStore.isChecking}
      <span class="checking-indicator" aria-label="Checking health">
        <svg class="spinner" viewBox="0 0 24 24" width="14" height="14">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2" fill="none" stroke-dasharray="31.4 31.4" />
        </svg>
      </span>
    {/if}
  </button>

  {#if expanded && showDetails && $healthStore.health}
    <div
      class="health-details"
      transition:scale={{ duration: 150, start: 0.95 }}
    >
      <div class="details-header">
        <h3>System Health</h3>
        <span class="uptime">
          Uptime: {formatUptime($healthStore.health.uptime_seconds)}
        </span>
      </div>

      <ul class="services-list">
        {#each $healthStore.health.services as service}
          <li class="service-item">
            <span
              class="service-status"
              style="color: {statusColors[service.status]}"
            >
              {getStatusIcon(service.status)}
            </span>
            <span class="service-name">{service.name}</span>
            <span class="service-latency">{formatLatency(service.latency_ms)}</span>
            {#if service.error}
              <span class="service-error" title={service.error}>!</span>
            {/if}
          </li>
        {/each}
      </ul>

      <button
        class="refresh-button"
        on:click={() => healthStore.checkHealth()}
        disabled={$healthStore.isChecking}
      >
        Refresh
      </button>
    </div>
  {/if}
</div>

<style>
  .health-indicator {
    position: relative;
    display: inline-block;
  }

  .health-button {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    background: transparent;
    border: 1px solid var(--border-color, #333);
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    color: inherit;
    transition: background-color 0.15s;
  }

  .health-button:hover {
    background: var(--hover-bg, rgba(255, 255, 255, 0.05));
  }

  .compact .health-button {
    padding: 2px 6px;
  }

  .status-dot {
    font-size: 10px;
    line-height: 1;
  }

  .status-label {
    white-space: nowrap;
  }

  .checking-indicator {
    display: flex;
    align-items: center;
  }

  .spinner {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .health-details {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    padding: 12px;
    background: var(--panel-bg, #1a1a1a);
    border: 1px solid var(--border-color, #333);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    min-width: 220px;
    z-index: 1000;
  }

  .details-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border-color, #333);
  }

  .details-header h3 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }

  .uptime {
    font-size: 11px;
    color: var(--text-muted, #888);
  }

  .services-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .service-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 0;
    font-size: 12px;
  }

  .service-status {
    font-size: 8px;
  }

  .service-name {
    flex: 1;
    text-transform: capitalize;
  }

  .service-latency {
    color: var(--text-muted, #888);
    font-family: monospace;
    font-size: 11px;
  }

  .service-error {
    color: var(--color-error, #ef4444);
    font-weight: bold;
    cursor: help;
  }

  .refresh-button {
    width: 100%;
    margin-top: 12px;
    padding: 6px;
    background: var(--button-bg, #333);
    border: none;
    border-radius: 4px;
    color: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .refresh-button:hover:not(:disabled) {
    background: var(--button-hover-bg, #444);
  }

  .refresh-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
```

**File: `app/src/lib/components/health/ServiceStatusBadge.svelte`**
```svelte
<script lang="ts">
  import type { HealthStatus } from '$lib/stores/healthStore';

  export let status: HealthStatus;
  export let serviceName: string;
  export let latency: number | null = null;
  export let size: 'sm' | 'md' | 'lg' = 'md';

  const statusStyles: Record<HealthStatus, { bg: string; text: string }> = {
    healthy: { bg: 'rgba(34, 197, 94, 0.15)', text: '#22c55e' },
    degraded: { bg: 'rgba(234, 179, 8, 0.15)', text: '#eab308' },
    unhealthy: { bg: 'rgba(239, 68, 68, 0.15)', text: '#ef4444' },
    unknown: { bg: 'rgba(107, 114, 128, 0.15)', text: '#6b7280' },
  };

  $: style = statusStyles[status];
</script>

<span
  class="badge size-{size}"
  style="background-color: {style.bg}; color: {style.text}"
  role="status"
  aria-label="{serviceName} status: {status}"
>
  <span class="dot" />
  <span class="name">{serviceName}</span>
  {#if latency !== null}
    <span class="latency">{latency}ms</span>
  {/if}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 9999px;
    font-weight: 500;
    text-transform: capitalize;
  }

  .size-sm {
    font-size: 10px;
    padding: 2px 8px;
    gap: 4px;
  }

  .size-md {
    font-size: 12px;
  }

  .size-lg {
    font-size: 14px;
    padding: 6px 14px;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: currentColor;
  }

  .size-sm .dot {
    width: 4px;
    height: 4px;
  }

  .size-lg .dot {
    width: 8px;
    height: 8px;
  }

  .name {
    white-space: nowrap;
  }

  .latency {
    opacity: 0.7;
    font-family: monospace;
    font-size: 0.9em;
  }
</style>
```

---

## Part 2: Offline Mode & Caching Strategy

### 2.1 Offline Store

**File: `app/src/lib/stores/offlineStore.ts`**
```typescript
import { writable, derived, get } from 'svelte/store';
import { healthStore, isSystemHealthy } from './healthStore';

// Types
export interface CachedItem<T> {
  data: T;
  cachedAt: number;
  expiresAt: number;
  key: string;
}

export interface OfflineState {
  isOffline: boolean;
  lastOnlineAt: number | null;
  pendingOperations: PendingOperation[];
  cacheStats: CacheStats;
}

export interface PendingOperation {
  id: string;
  type: 'create' | 'update' | 'delete';
  entity: string;
  data: unknown;
  createdAt: number;
  retryCount: number;
}

export interface CacheStats {
  itemCount: number;
  totalSize: number;
  oldestItem: number | null;
  newestItem: number | null;
}

// Constants
const CACHE_PREFIX = 'midi_cache_';
const CACHE_INDEX_KEY = 'midi_cache_index';
const DEFAULT_TTL = 1000 * 60 * 60; // 1 hour
const MAX_CACHE_SIZE = 50 * 1024 * 1024; // 50MB
const MAX_PENDING_OPERATIONS = 100;

// Cache Index for tracking all cached items
interface CacheIndex {
  keys: string[];
  lastUpdated: number;
}

function getCacheIndex(): CacheIndex {
  try {
    const stored = localStorage.getItem(CACHE_INDEX_KEY);
    return stored ? JSON.parse(stored) : { keys: [], lastUpdated: Date.now() };
  } catch {
    return { keys: [], lastUpdated: Date.now() };
  }
}

function saveCacheIndex(index: CacheIndex): void {
  try {
    localStorage.setItem(CACHE_INDEX_KEY, JSON.stringify(index));
  } catch (e) {
    console.error('Failed to save cache index:', e);
  }
}

// Store
function createOfflineStore() {
  const { subscribe, set, update } = writable<OfflineState>({
    isOffline: false,
    lastOnlineAt: null,
    pendingOperations: [],
    cacheStats: {
      itemCount: 0,
      totalSize: 0,
      oldestItem: null,
      newestItem: null,
    },
  });

  // Load pending operations from storage
  function loadPendingOperations(): PendingOperation[] {
    try {
      const stored = localStorage.getItem('midi_pending_ops');
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }

  // Save pending operations to storage
  function savePendingOperations(ops: PendingOperation[]): void {
    try {
      localStorage.setItem('midi_pending_ops', JSON.stringify(ops));
    } catch (e) {
      console.error('Failed to save pending operations:', e);
    }
  }

  // Cache operations
  function cacheItem<T>(key: string, data: T, ttl: number = DEFAULT_TTL): void {
    const cacheKey = CACHE_PREFIX + key;
    const item: CachedItem<T> = {
      data,
      cachedAt: Date.now(),
      expiresAt: Date.now() + ttl,
      key,
    };

    try {
      const serialized = JSON.stringify(item);

      // Check if we need to evict items
      const currentSize = getCacheSize();
      if (currentSize + serialized.length > MAX_CACHE_SIZE) {
        evictOldestItems(serialized.length);
      }

      localStorage.setItem(cacheKey, serialized);

      // Update index
      const index = getCacheIndex();
      if (!index.keys.includes(key)) {
        index.keys.push(key);
      }
      index.lastUpdated = Date.now();
      saveCacheIndex(index);

      updateCacheStats();
    } catch (e) {
      console.error('Failed to cache item:', e);
      // Try to evict and retry
      evictOldestItems(MAX_CACHE_SIZE / 4);
      try {
        localStorage.setItem(cacheKey, JSON.stringify(item));
      } catch {
        console.error('Cache storage full, unable to cache item');
      }
    }
  }

  function getCachedItem<T>(key: string): T | null {
    const cacheKey = CACHE_PREFIX + key;
    try {
      const stored = localStorage.getItem(cacheKey);
      if (!stored) return null;

      const item: CachedItem<T> = JSON.parse(stored);

      // Check expiration
      if (Date.now() > item.expiresAt) {
        removeCachedItem(key);
        return null;
      }

      return item.data;
    } catch {
      return null;
    }
  }

  function removeCachedItem(key: string): void {
    const cacheKey = CACHE_PREFIX + key;
    localStorage.removeItem(cacheKey);

    // Update index
    const index = getCacheIndex();
    index.keys = index.keys.filter(k => k !== key);
    index.lastUpdated = Date.now();
    saveCacheIndex(index);

    updateCacheStats();
  }

  function getCacheSize(): number {
    let size = 0;
    const index = getCacheIndex();

    for (const key of index.keys) {
      const item = localStorage.getItem(CACHE_PREFIX + key);
      if (item) {
        size += item.length * 2; // UTF-16 encoding
      }
    }

    return size;
  }

  function evictOldestItems(bytesNeeded: number): void {
    const index = getCacheIndex();
    const items: { key: string; cachedAt: number; size: number }[] = [];

    for (const key of index.keys) {
      const stored = localStorage.getItem(CACHE_PREFIX + key);
      if (stored) {
        try {
          const item = JSON.parse(stored);
          items.push({
            key,
            cachedAt: item.cachedAt,
            size: stored.length * 2,
          });
        } catch {
          // Invalid item, remove it
          localStorage.removeItem(CACHE_PREFIX + key);
        }
      }
    }

    // Sort by age (oldest first)
    items.sort((a, b) => a.cachedAt - b.cachedAt);

    let freedBytes = 0;
    const keysToRemove: string[] = [];

    for (const item of items) {
      if (freedBytes >= bytesNeeded) break;
      keysToRemove.push(item.key);
      freedBytes += item.size;
    }

    for (const key of keysToRemove) {
      localStorage.removeItem(CACHE_PREFIX + key);
    }

    // Update index
    index.keys = index.keys.filter(k => !keysToRemove.includes(k));
    index.lastUpdated = Date.now();
    saveCacheIndex(index);

    console.log(`Evicted ${keysToRemove.length} cache items, freed ${freedBytes} bytes`);
  }

  function updateCacheStats(): void {
    const index = getCacheIndex();
    let totalSize = 0;
    let oldestItem: number | null = null;
    let newestItem: number | null = null;

    for (const key of index.keys) {
      const stored = localStorage.getItem(CACHE_PREFIX + key);
      if (stored) {
        totalSize += stored.length * 2;
        try {
          const item = JSON.parse(stored);
          if (oldestItem === null || item.cachedAt < oldestItem) {
            oldestItem = item.cachedAt;
          }
          if (newestItem === null || item.cachedAt > newestItem) {
            newestItem = item.cachedAt;
          }
        } catch {
          // Skip invalid items
        }
      }
    }

    update(s => ({
      ...s,
      cacheStats: {
        itemCount: index.keys.length,
        totalSize,
        oldestItem,
        newestItem,
      },
    }));
  }

  // Pending operations
  function addPendingOperation(
    type: PendingOperation['type'],
    entity: string,
    data: unknown
  ): string {
    const id = crypto.randomUUID();
    const operation: PendingOperation = {
      id,
      type,
      entity,
      data,
      createdAt: Date.now(),
      retryCount: 0,
    };

    update(s => {
      let ops = [...s.pendingOperations, operation];

      // Limit pending operations
      if (ops.length > MAX_PENDING_OPERATIONS) {
        ops = ops.slice(-MAX_PENDING_OPERATIONS);
      }

      savePendingOperations(ops);
      return { ...s, pendingOperations: ops };
    });

    return id;
  }

  function removePendingOperation(id: string): void {
    update(s => {
      const ops = s.pendingOperations.filter(op => op.id !== id);
      savePendingOperations(ops);
      return { ...s, pendingOperations: ops };
    });
  }

  async function syncPendingOperations(): Promise<{ synced: number; failed: number }> {
    const state = get({ subscribe });
    let synced = 0;
    let failed = 0;

    for (const op of state.pendingOperations) {
      try {
        await executePendingOperation(op);
        removePendingOperation(op.id);
        synced++;
      } catch (error) {
        console.error(`Failed to sync operation ${op.id}:`, error);

        // Increment retry count
        update(s => ({
          ...s,
          pendingOperations: s.pendingOperations.map(o =>
            o.id === op.id ? { ...o, retryCount: o.retryCount + 1 } : o
          ),
        }));

        failed++;

        // Remove operations that have failed too many times
        if (op.retryCount >= 5) {
          removePendingOperation(op.id);
          console.warn(`Operation ${op.id} removed after 5 failed retries`);
        }
      }
    }

    return { synced, failed };
  }

  async function executePendingOperation(op: PendingOperation): Promise<void> {
    // This should be implemented based on your actual API
    const { invoke } = await import('@tauri-apps/api/core');

    switch (op.type) {
      case 'create':
        await invoke(`create_${op.entity}`, { data: op.data });
        break;
      case 'update':
        await invoke(`update_${op.entity}`, { data: op.data });
        break;
      case 'delete':
        await invoke(`delete_${op.entity}`, { data: op.data });
        break;
    }
  }

  function clearCache(): void {
    const index = getCacheIndex();
    for (const key of index.keys) {
      localStorage.removeItem(CACHE_PREFIX + key);
    }
    localStorage.removeItem(CACHE_INDEX_KEY);
    updateCacheStats();
  }

  // Initialize
  const pendingOps = loadPendingOperations();
  set({
    isOffline: false,
    lastOnlineAt: Date.now(),
    pendingOperations: pendingOps,
    cacheStats: {
      itemCount: 0,
      totalSize: 0,
      oldestItem: null,
      newestItem: null,
    },
  });
  updateCacheStats();

  // Listen for health changes
  healthStore.subscribe(health => {
    const wasOffline = get({ subscribe }).isOffline;
    const isNowOffline = health.health?.overall_status === 'unhealthy';

    if (wasOffline && !isNowOffline) {
      // Coming back online - sync pending operations
      syncPendingOperations().then(({ synced, failed }) => {
        if (synced > 0) {
          console.log(`Synced ${synced} pending operations`);
        }
        if (failed > 0) {
          console.warn(`Failed to sync ${failed} operations`);
        }
      });
    }

    update(s => ({
      ...s,
      isOffline: isNowOffline,
      lastOnlineAt: isNowOffline ? s.lastOnlineAt : Date.now(),
    }));
  });

  return {
    subscribe,
    cacheItem,
    getCachedItem,
    removeCachedItem,
    addPendingOperation,
    removePendingOperation,
    syncPendingOperations,
    clearCache,
    getCacheSize,

    // Convenience methods
    cacheSearchResults: (query: string, results: unknown) =>
      cacheItem(`search:${query}`, results, DEFAULT_TTL),
    getCachedSearch: (query: string) =>
      getCachedItem(`search:${query}`),
    cacheFileDetails: (fileId: number, details: unknown) =>
      cacheItem(`file:${fileId}`, details, DEFAULT_TTL * 24),
    getCachedFileDetails: (fileId: number) =>
      getCachedItem(`file:${fileId}`),
  };
}

export const offlineStore = createOfflineStore();

// Derived stores
export const isOffline = derived(offlineStore, $store => $store.isOffline);

export const pendingOperationsCount = derived(
  offlineStore,
  $store => $store.pendingOperations.length
);

export const hasPendingOperations = derived(
  offlineStore,
  $store => $store.pendingOperations.length > 0
);
```

### 2.2 Offline-Aware API Wrapper

**File: `app/src/lib/api/offlineApi.ts`**
```typescript
import { invoke } from '@tauri-apps/api/core';
import { get } from 'svelte/store';
import { offlineStore, isOffline } from '$lib/stores/offlineStore';
import { healthStore } from '$lib/stores/healthStore';

export interface OfflineApiOptions {
  cacheKey?: string;
  cacheTtl?: number;
  allowOffline?: boolean;
  queueIfOffline?: boolean;
}

export class OfflineApiError extends Error {
  constructor(
    message: string,
    public readonly isOfflineError: boolean = false,
    public readonly cachedData?: unknown
  ) {
    super(message);
    this.name = 'OfflineApiError';
  }
}

export async function offlineInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
  options: OfflineApiOptions = {}
): Promise<T> {
  const {
    cacheKey,
    cacheTtl,
    allowOffline = true,
    queueIfOffline = false,
  } = options;

  const offline = get(isOffline);

  // If online, make the request
  if (!offline) {
    try {
      const result = await invoke<T>(command, args);

      // Cache the result if cacheKey is provided
      if (cacheKey) {
        offlineStore.cacheItem(cacheKey, result, cacheTtl);
      }

      return result;
    } catch (error) {
      // Check if this is a connection error
      if (isConnectionError(error)) {
        // Re-check health
        await healthStore.checkHealth();

        // If we're now offline, try cache
        if (allowOffline && cacheKey) {
          const cached = offlineStore.getCachedItem<T>(cacheKey);
          if (cached !== null) {
            console.log(`Returning cached data for ${command}`);
            return cached;
          }
        }
      }
      throw error;
    }
  }

  // We're offline
  if (!allowOffline) {
    throw new OfflineApiError(
      `Cannot execute ${command}: system is offline`,
      true
    );
  }

  // Try to get cached data
  if (cacheKey) {
    const cached = offlineStore.getCachedItem<T>(cacheKey);
    if (cached !== null) {
      console.log(`Returning cached data for ${command} (offline)`);
      return cached;
    }
  }

  // Queue the operation if it's a mutation
  if (queueIfOffline && args) {
    const entity = command.replace(/^(create_|update_|delete_)/, '');
    const type = command.startsWith('create_')
      ? 'create'
      : command.startsWith('update_')
      ? 'update'
      : command.startsWith('delete_')
      ? 'delete'
      : null;

    if (type) {
      offlineStore.addPendingOperation(type, entity, args);
      throw new OfflineApiError(
        `Operation queued for sync when online`,
        true
      );
    }
  }

  throw new OfflineApiError(
    `Cannot execute ${command}: no cached data available`,
    true
  );
}

function isConnectionError(error: unknown): boolean {
  if (error instanceof Error) {
    const message = error.message.toLowerCase();
    return (
      message.includes('connection') ||
      message.includes('network') ||
      message.includes('offline') ||
      message.includes('timeout') ||
      message.includes('econnrefused')
    );
  }
  return false;
}

// Convenience wrappers for common operations
export const offlineApi = {
  async searchFiles(query: string, filters?: Record<string, unknown>) {
    return offlineInvoke('search_files', { query, filters }, {
      cacheKey: `search:${query}:${JSON.stringify(filters || {})}`,
      cacheTtl: 1000 * 60 * 30, // 30 minutes
      allowOffline: true,
    });
  },

  async getFileDetails(fileId: number) {
    return offlineInvoke('get_file_details', { fileId }, {
      cacheKey: `file:${fileId}`,
      cacheTtl: 1000 * 60 * 60 * 24, // 24 hours
      allowOffline: true,
    });
  },

  async getTags() {
    return offlineInvoke('get_all_tags', undefined, {
      cacheKey: 'tags:all',
      cacheTtl: 1000 * 60 * 60, // 1 hour
      allowOffline: true,
    });
  },

  async updateFileTags(fileId: number, tagIds: number[]) {
    return offlineInvoke('update_file_tags', { fileId, tagIds }, {
      queueIfOffline: true,
    });
  },

  async createTag(name: string, category: string) {
    return offlineInvoke('create_tag', { name, category }, {
      queueIfOffline: true,
    });
  },
};
```

### 2.3 Offline Status Banner Component

**File: `app/src/lib/components/health/OfflineBanner.svelte`**
```svelte
<script lang="ts">
  import { offlineStore, isOffline, pendingOperationsCount } from '$lib/stores/offlineStore';
  import { slide } from 'svelte/transition';

  export let position: 'top' | 'bottom' = 'top';

  function formatLastOnline(timestamp: number | null): string {
    if (!timestamp) return 'Unknown';

    const diff = Date.now() - timestamp;
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);

    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    return new Date(timestamp).toLocaleDateString();
  }

  async function retrySyncNow() {
    const { synced, failed } = await offlineStore.syncPendingOperations();
    console.log(`Manual sync: ${synced} synced, ${failed} failed`);
  }
</script>

{#if $isOffline}
  <div
    class="offline-banner position-{position}"
    role="alert"
    aria-live="assertive"
    transition:slide={{ duration: 200 }}
  >
    <div class="banner-content">
      <span class="offline-icon">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="1" y1="1" x2="23" y2="23" />
          <path d="M16.72 11.06A10.94 10.94 0 0 1 19 12.55" />
          <path d="M5 12.55a10.94 10.94 0 0 1 5.17-2.39" />
          <path d="M10.71 5.05A16 16 0 0 1 22.58 9" />
          <path d="M1.42 9a15.91 15.91 0 0 1 4.7-2.88" />
          <path d="M8.53 16.11a6 6 0 0 1 6.95 0" />
          <line x1="12" y1="20" x2="12.01" y2="20" />
        </svg>
      </span>
      <span class="offline-text">
        You're offline
        {#if $offlineStore.lastOnlineAt}
          <span class="last-online">
            (Last online: {formatLastOnline($offlineStore.lastOnlineAt)})
          </span>
        {/if}
      </span>
    </div>

    {#if $pendingOperationsCount > 0}
      <div class="pending-ops">
        <span>{$pendingOperationsCount} pending change{$pendingOperationsCount !== 1 ? 's' : ''}</span>
        <button class="sync-button" on:click={retrySyncNow}>
          Retry now
        </button>
      </div>
    {/if}

    <div class="banner-info">
      <p>Some features may be limited. Cached data will be shown where available.</p>
    </div>
  </div>
{/if}

<style>
  .offline-banner {
    position: fixed;
    left: 0;
    right: 0;
    padding: 12px 20px;
    background: linear-gradient(135deg, #92400e 0%, #78350f 100%);
    color: white;
    z-index: 9999;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .position-top {
    top: 0;
  }

  .position-bottom {
    bottom: 0;
  }

  .banner-content {
    display: flex;
    align-items: center;
    gap: 10px;
    font-weight: 600;
  }

  .offline-icon {
    display: flex;
    align-items: center;
  }

  .offline-text {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .last-online {
    font-size: 12px;
    font-weight: normal;
    opacity: 0.8;
  }

  .pending-ops {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid rgba(255, 255, 255, 0.2);
    font-size: 13px;
  }

  .sync-button {
    padding: 4px 12px;
    background: rgba(255, 255, 255, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 4px;
    color: white;
    font-size: 12px;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .sync-button:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  .banner-info {
    margin-top: 8px;
    font-size: 12px;
    opacity: 0.9;
  }

  .banner-info p {
    margin: 0;
  }
</style>
```

---

## Part 3: Integration & Startup

### 3.1 App Initialization

**File: `app/src/lib/init/healthInit.ts`**
```typescript
import { healthStore } from '$lib/stores/healthStore';
import { offlineStore } from '$lib/stores/offlineStore';

export async function initializeHealthSystem(): Promise<void> {
  // Start health monitoring
  healthStore.startMonitoring(30000); // Check every 30 seconds

  // Do initial health check
  const health = await healthStore.checkHealth();

  if (health && health.overall_status !== 'healthy') {
    console.warn('System starting with degraded/unhealthy services:', health);
  }

  // If coming online after being offline, sync pending operations
  if (health && health.overall_status === 'healthy') {
    const { synced, failed } = await offlineStore.syncPendingOperations();
    if (synced > 0 || failed > 0) {
      console.log(`Startup sync: ${synced} synced, ${failed} failed`);
    }
  }
}

export function cleanupHealthSystem(): void {
  healthStore.stopMonitoring();
}
```

### 3.2 Update lib.rs exports

**Add to `shared/rust/src/lib.rs`:**
```rust
pub mod health;
```

### 3.3 Register Commands in main.rs

**Add to your Tauri command registration:**
```rust
use pipeline::commands::health::{
    check_system_health,
    get_cached_health,
    check_postgres_health,
    check_meilisearch_health,
    HealthState,
};

// In your main/setup:
let health_checker = HealthChecker::new()
    .with_postgres(pool.clone())
    .with_meilisearch(
        std::env::var("MEILISEARCH_URL").unwrap_or_else(|_| "http://localhost:7700".to_string()),
        std::env::var("MEILISEARCH_KEY").ok(),
    );

app.manage(HealthState::new(health_checker));

// Add to invoke_handler:
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    check_system_health,
    get_cached_health,
    check_postgres_health,
    check_meilisearch_health,
])
```

---

## Summary Checklist

### Health Checks
- [ ] Create `shared/rust/src/health/mod.rs`
- [ ] Create `shared/rust/src/health/status.rs`
- [ ] Create `shared/rust/src/health/checker.rs`
- [ ] Create `pipeline/src-tauri/src/commands/health.rs`
- [ ] Create `app/src/lib/stores/healthStore.ts`
- [ ] Create `app/src/lib/components/health/HealthIndicator.svelte`
- [ ] Create `app/src/lib/components/health/ServiceStatusBadge.svelte`

### Offline Mode
- [ ] Create `app/src/lib/stores/offlineStore.ts`
- [ ] Create `app/src/lib/api/offlineApi.ts`
- [ ] Create `app/src/lib/components/health/OfflineBanner.svelte`

### Integration
- [ ] Create `app/src/lib/init/healthInit.ts`
- [ ] Export health module from `shared/rust/src/lib.rs`
- [ ] Register health commands in Tauri
- [ ] Add HealthIndicator to StatusBar
- [ ] Add OfflineBanner to App layout

### Testing
- [ ] Test PostgreSQL health check with database down
- [ ] Test Meilisearch health check with service down
- [ ] Test cache eviction when storage is full
- [ ] Test pending operation sync on reconnection
- [ ] Test offline data retrieval from cache
