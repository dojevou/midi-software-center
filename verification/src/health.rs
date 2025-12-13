//! Health Check Verification Module
//!
//! Verifies runtime application health and system resources.

use sqlx::PgPool;
use std::path::Path;

/// Health check error
#[derive(Debug, Clone)]
pub struct HealthError {
    pub category: String,
    pub component: String,
    pub message: String,
    pub severity: HealthErrorSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthErrorSeverity {
    Critical,
    Warning,
    Info,
}

/// Verifies application and system health
pub struct HealthVerifier;

impl HealthVerifier {
    /// Verify database health and connectivity
    pub async fn verify_database_health(pool: &PgPool) -> Result<(), Vec<HealthError>> {
        let mut errors = Vec::new();

        // Check database connection
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => {
                println!("Database connection: OK");
            },
            Err(e) => {
                errors.push(HealthError {
                    category: "database".to_string(),
                    component: "connection".to_string(),
                    message: format!("Database connection failed: {}", e),
                    severity: HealthErrorSeverity::Critical,
                });
                return Err(errors);
            },
        }

        // Check table sizes and data integrity
        let table_checks = vec![
            ("files", 1000, "Expect at least 1000 files"),
            ("musical_metadata", 500, "Expect at least 500 with metadata"),
            ("tags", 50, "Expect at least 50 tags"),
        ];

        for (table, min_rows, description) in table_checks {
            let count: Result<i64, _> =
                sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table))
                    .fetch_one(pool)
                    .await;

            match count {
                Ok(count) => {
                    if count < min_rows {
                        errors.push(HealthError {
                            category: "database".to_string(),
                            component: table.to_string(),
                            message: format!(
                                "Table {} has only {} rows ({})",
                                table, count, description
                            ),
                            severity: HealthErrorSeverity::Warning,
                        });
                    } else {
                        println!("Table {}: {} rows", table, count);
                    }
                },
                Err(e) => {
                    errors.push(HealthError {
                        category: "database".to_string(),
                        component: table.to_string(),
                        message: format!("Failed to count rows in {}: {}", table, e),
                        severity: HealthErrorSeverity::Warning,
                    });
                },
            }
        }

        // Check for orphaned records
        let orphan_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM musical_metadata m
             LEFT JOIN files f ON m.file_id = f.id
             WHERE f.id IS NULL",
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        if orphan_count > 0 {
            errors.push(HealthError {
                category: "database".to_string(),
                component: "integrity".to_string(),
                message: format!("Found {} orphaned metadata records", orphan_count),
                severity: HealthErrorSeverity::Warning,
            });
        }

        // Check database size
        let db_size: Result<String, _> =
            sqlx::query_scalar("SELECT pg_size_pretty(pg_database_size(current_database()))")
                .fetch_one(pool)
                .await;

        if let Ok(size) = db_size {
            println!("Database size: {}", size);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Verify file system health
    pub fn verify_file_system_health() -> Result<(), Vec<HealthError>> {
        let mut errors = Vec::new();

        // Check disk space
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;

            if let Ok(output) = Command::new("df").args(["-h", "/"]).output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                println!("Disk usage:\n{}", output_str);

                // Parse and check available space
                if let Some(available_gb) = Self::parse_available_gb(&output_str) {
                    if available_gb < 5.0 {
                        errors.push(HealthError {
                            category: "filesystem".to_string(),
                            component: "disk".to_string(),
                            message: format!("Low disk space: {:.1}GB available", available_gb),
                            severity: HealthErrorSeverity::Critical,
                        });
                    } else if available_gb < 20.0 {
                        errors.push(HealthError {
                            category: "filesystem".to_string(),
                            component: "disk".to_string(),
                            message: format!(
                                "Disk space getting low: {:.1}GB available",
                                available_gb
                            ),
                            severity: HealthErrorSeverity::Warning,
                        });
                    }
                }
            }
        }

        // Check if critical directories are accessible
        let critical_dirs =
            vec![("./app", "Application directory"), ("./target", "Build output directory")];

        for (dir, description) in critical_dirs {
            if let Err(e) = std::fs::metadata(dir) {
                errors.push(HealthError {
                    category: "filesystem".to_string(),
                    component: dir.to_string(),
                    message: format!("Cannot access {}: {}", description, e),
                    severity: HealthErrorSeverity::Warning,
                });
            }
        }

        // Check MIDI data directories
        let data_dirs = vec![
            "/home/dojevou/tmp/midi_splits_fast",
            "/home/dojevou/projects/midi-software-center/midi-library",
        ];

        for dir in data_dirs {
            let path = Path::new(dir);
            if path.exists() {
                // Count files in directory
                if let Ok(entries) = std::fs::read_dir(path) {
                    let count = entries.count();
                    println!("Directory {} contains {} entries", dir, count);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Parse available disk space from df output
    #[cfg(target_os = "linux")]
    fn parse_available_gb(df_output: &str) -> Option<f64> {
        // df -h output format: Filesystem Size Used Avail Use% Mounted
        for line in df_output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let avail = parts[3];
                // Parse size like "50G" or "500M"
                if avail.ends_with('G') {
                    return avail.trim_end_matches('G').parse().ok();
                } else if avail.ends_with('M') {
                    return avail.trim_end_matches('M').parse::<f64>().ok().map(|m| m / 1024.0);
                } else if avail.ends_with('T') {
                    return avail.trim_end_matches('T').parse::<f64>().ok().map(|t| t * 1024.0);
                }
            }
        }
        None
    }

    /// Verify system resources
    pub fn verify_system_resources() -> Result<(), Vec<HealthError>> {
        let mut errors = Vec::new();

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;

            // Check memory usage
            if let Ok(output) = Command::new("free").args(["-h"]).output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                println!("Memory usage:\n{}", output_str);

                // Parse and check available memory
                if let Some(available_gb) = Self::parse_available_memory(&output_str) {
                    if available_gb < 1.0 {
                        errors.push(HealthError {
                            category: "system".to_string(),
                            component: "memory".to_string(),
                            message: format!("Low memory: {:.1}GB available", available_gb),
                            severity: HealthErrorSeverity::Critical,
                        });
                    } else if available_gb < 4.0 {
                        errors.push(HealthError {
                            category: "system".to_string(),
                            component: "memory".to_string(),
                            message: format!("Memory getting low: {:.1}GB available", available_gb),
                            severity: HealthErrorSeverity::Warning,
                        });
                    }
                }
            }

            // Check CPU load
            if let Ok(output) = Command::new("uptime").output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                println!("System load: {}", output_str.trim());

                // Parse load average
                if let Some(load) = Self::parse_load_average(&output_str) {
                    let num_cpus = num_cpus::get() as f64;
                    if load > num_cpus * 2.0 {
                        errors.push(HealthError {
                            category: "system".to_string(),
                            component: "cpu".to_string(),
                            message: format!("High CPU load: {:.2} ({}x cores)", load, num_cpus),
                            severity: HealthErrorSeverity::Warning,
                        });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Parse available memory from free -h output
    #[cfg(target_os = "linux")]
    fn parse_available_memory(free_output: &str) -> Option<f64> {
        for line in free_output.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 7 {
                    let available = parts[6];
                    if available.ends_with("Gi") {
                        return available.trim_end_matches("Gi").parse().ok();
                    } else if available.ends_with("Mi") {
                        return available
                            .trim_end_matches("Mi")
                            .parse::<f64>()
                            .ok()
                            .map(|m| m / 1024.0);
                    } else if available.ends_with('G') {
                        return available.trim_end_matches('G').parse().ok();
                    } else if available.ends_with('M') {
                        return available
                            .trim_end_matches('M')
                            .parse::<f64>()
                            .ok()
                            .map(|m| m / 1024.0);
                    }
                }
            }
        }
        None
    }

    /// Parse load average from uptime output
    #[cfg(target_os = "linux")]
    fn parse_load_average(uptime_output: &str) -> Option<f64> {
        if let Some(pos) = uptime_output.find("load average:") {
            let rest = &uptime_output[pos + 13..];
            let load_str = rest.split(',').next()?.trim();
            return load_str.parse().ok();
        }
        None
    }

    /// Get health verification summary
    pub fn get_summary() -> HealthSummary {
        let filesystem_ok = Self::verify_file_system_health().is_ok();
        let resources_ok = Self::verify_system_resources().is_ok();

        HealthSummary {
            filesystem_healthy: filesystem_ok,
            resources_healthy: resources_ok,
            database_healthy: false, // Set separately with async check
        }
    }

    /// Get async health summary with database check
    pub async fn get_full_summary(pool: &PgPool) -> HealthSummary {
        let filesystem_ok = Self::verify_file_system_health().is_ok();
        let resources_ok = Self::verify_system_resources().is_ok();
        let database_ok = Self::verify_database_health(pool).await.is_ok();

        HealthSummary {
            filesystem_healthy: filesystem_ok,
            resources_healthy: resources_ok,
            database_healthy: database_ok,
        }
    }
}

/// Health verification summary
#[derive(Debug, Clone)]
pub struct HealthSummary {
    pub filesystem_healthy: bool,
    pub resources_healthy: bool,
    pub database_healthy: bool,
}

impl HealthSummary {
    pub fn all_healthy(&self) -> bool {
        self.filesystem_healthy && self.resources_healthy && self.database_healthy
    }
}

impl std::fmt::Display for HealthSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Health Summary:\n  Filesystem: {}\n  Resources: {}\n  Database: {}",
            if self.filesystem_healthy {
                "HEALTHY"
            } else {
                "UNHEALTHY"
            },
            if self.resources_healthy {
                "HEALTHY"
            } else {
                "UNHEALTHY"
            },
            if self.database_healthy {
                "HEALTHY"
            } else {
                "UNHEALTHY"
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_error_severity() {
        let error = HealthError {
            category: "test".to_string(),
            component: "test-component".to_string(),
            message: "Test message".to_string(),
            severity: HealthErrorSeverity::Critical,
        };
        assert_eq!(error.severity, HealthErrorSeverity::Critical);
    }

    #[test]
    fn test_health_summary_display() {
        let summary = HealthSummary {
            filesystem_healthy: true,
            resources_healthy: true,
            database_healthy: false,
        };
        let display = format!("{}", summary);
        assert!(display.contains("Filesystem: HEALTHY"));
        assert!(display.contains("Database: UNHEALTHY"));
    }

    #[test]
    fn test_health_summary_all_healthy() {
        let summary = HealthSummary {
            filesystem_healthy: true,
            resources_healthy: true,
            database_healthy: true,
        };
        assert!(summary.all_healthy());

        let unhealthy = HealthSummary {
            filesystem_healthy: true,
            resources_healthy: false,
            database_healthy: true,
        };
        assert!(!unhealthy.all_healthy());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_available_gb() {
        let df_output = "Filesystem      Size  Used Avail Use% Mounted on\n/dev/sda1       100G   50G   45G  53% /";
        let result = HealthVerifier::parse_available_gb(df_output);
        assert_eq!(result, Some(45.0));
    }
}
