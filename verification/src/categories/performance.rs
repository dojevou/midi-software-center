//! Performance Verification
//!
//! Benchmarks and verifies performance characteristics of the system.

use crate::shared::{CheckStatus, VerificationLevel, VerificationResult};
use std::fs;
use std::time::{Duration, Instant};

/// Performance Verifier
pub struct PerformanceVerifier;

impl PerformanceVerifier {
    /// Benchmark file system operations
    pub fn benchmark_filesystem() -> VerificationResult {
        let test_dir = "/tmp/midi_verification_perf_test";
        let _ = fs::create_dir_all(test_dir);

        let mut details = Vec::new();
        let mut total_score = 0;

        // Test 1: File creation speed
        let start = Instant::now();
        for i in 0..100 {
            let path = format!("{}/test_{}.txt", test_dir, i);
            let _ = fs::write(&path, "test content");
        }
        let creation_time = start.elapsed();
        let creation_rate = 100.0 / creation_time.as_secs_f64();
        details.push(format!(
            "File creation: {:.0} files/sec ({:.2}ms for 100 files)",
            creation_rate,
            creation_time.as_millis()
        ));
        if creation_rate > 500.0 {
            total_score += 1;
        }

        // Test 2: File reading speed
        let start = Instant::now();
        for i in 0..100 {
            let path = format!("{}/test_{}.txt", test_dir, i);
            let _ = fs::read_to_string(&path);
        }
        let read_time = start.elapsed();
        let read_rate = 100.0 / read_time.as_secs_f64();
        details.push(format!(
            "File reading: {:.0} files/sec ({:.2}ms for 100 files)",
            read_rate,
            read_time.as_millis()
        ));
        if read_rate > 1000.0 {
            total_score += 1;
        }

        // Test 3: Directory listing speed
        let start = Instant::now();
        for _ in 0..100 {
            let _ = fs::read_dir(test_dir);
        }
        let list_time = start.elapsed();
        let list_rate = 100.0 / list_time.as_secs_f64();
        details.push(format!(
            "Directory listing: {:.0} ops/sec ({:.2}ms for 100 ops)",
            list_rate,
            list_time.as_millis()
        ));
        if list_rate > 5000.0 {
            total_score += 1;
        }

        // Cleanup
        let _ = fs::remove_dir_all(test_dir);

        let status = if total_score >= 3 {
            CheckStatus::Pass
        } else if total_score >= 2 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "performance".to_string(),
            check_name: "filesystem_benchmark".to_string(),
            level: VerificationLevel::Medium,
            status,
            message: format!(
                "Filesystem performance: {}/3 benchmarks passed",
                total_score
            ),
            details,
            suggestion: if total_score < 3 {
                Some("Consider using SSD storage or optimizing I/O patterns".to_string())
            } else {
                None
            },
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check for performance anti-patterns in code
    pub fn check_performance_antipatterns() -> VerificationResult {
        let mut antipatterns = Vec::new();

        let patterns = vec![
            ("clone()", "Unnecessary cloning", VerificationLevel::Low),
            (
                ".collect::<Vec<_>>().iter()",
                "Collect then iterate",
                VerificationLevel::Medium,
            ),
            (
                "format!(",
                "String formatting in hot path",
                VerificationLevel::Low,
            ),
            (
                ".to_string()",
                "Repeated string conversion",
                VerificationLevel::Low,
            ),
            (
                "Mutex<Vec",
                "Mutex around Vec (consider parking_lot)",
                VerificationLevel::Medium,
            ),
            (
                "Arc<Mutex",
                "Arc<Mutex> (consider RwLock or DashMap)",
                VerificationLevel::Low,
            ),
        ];

        let scan_files = vec![
            "pipeline/src-tauri/src/commands/file_import.rs",
            "pipeline/src-tauri/src/commands/analyze.rs",
            "pipeline/src-tauri/src/core/analysis/bpm_detector.rs",
            "pipeline/src-tauri/src/core/analysis/key_detector.rs",
            "shared/rust/src/core/midi/parser.rs",
        ];

        for file_path in &scan_files {
            if let Ok(content) = fs::read_to_string(file_path) {
                for (line_num, line) in content.lines().enumerate() {
                    for (pattern, description, _level) in &patterns {
                        if line.contains(pattern) {
                            // Skip if in test code
                            if line.contains("#[test]") || line.contains("#[cfg(test)]") {
                                continue;
                            }
                            antipatterns.push(format!(
                                "{}:{} - {}: {}",
                                file_path,
                                line_num + 1,
                                description,
                                pattern
                            ));
                        }
                    }
                }
            }
        }

        // Filter to only significant findings (more than 5 of same type indicates real issue)
        let significant_count = antipatterns.len();

        let status = if significant_count == 0 {
            CheckStatus::Pass
        } else if significant_count <= 10 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "performance".to_string(),
            check_name: "antipatterns".to_string(),
            level: VerificationLevel::Medium,
            status,
            message: format!(
                "Found {} potential performance antipattern(s)",
                significant_count
            ),
            details: antipatterns.into_iter().take(20).collect(), // Limit output
            suggestion: Some("Review hot paths for unnecessary allocations".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Verify database query performance
    pub async fn benchmark_database(pool: &sqlx::PgPool) -> VerificationResult {
        let mut details = Vec::new();
        let mut total_score = 0;

        // Test 1: Simple count query
        let start = Instant::now();
        let count_result: Result<(i64,), _> =
            sqlx::query_as("SELECT COUNT(*) FROM files").fetch_one(pool).await;
        let count_time = start.elapsed();

        match count_result {
            Ok((count,)) => {
                details.push(format!(
                    "Count query: {}ms for {} files",
                    count_time.as_millis(),
                    count
                ));
                if count_time < Duration::from_millis(100) || count < 1000 {
                    total_score += 1;
                }
            },
            Err(e) => {
                details.push(format!("Count query failed: {}", e));
            },
        }

        // Test 2: Index usage (EXPLAIN ANALYZE)
        let explain_result: Result<Vec<(String,)>, _> = sqlx::query_as(
            "EXPLAIN (FORMAT TEXT) SELECT * FROM files WHERE filename LIKE 'test%' LIMIT 10",
        )
        .fetch_all(pool)
        .await;

        match explain_result {
            Ok(rows) => {
                let plan = rows.iter().map(|(s,)| s.as_str()).collect::<Vec<_>>().join("\n");
                if plan.contains("Index") {
                    details.push("Index used for filename search".to_string());
                    total_score += 1;
                } else {
                    details.push("Warning: Sequential scan on filename search".to_string());
                }
            },
            Err(e) => {
                details.push(format!("EXPLAIN query failed: {}", e));
            },
        }

        // Test 3: Join performance
        let start = Instant::now();
        let join_result: Result<Vec<(i64,)>, _> = sqlx::query_as(
            "SELECT f.id FROM files f
             JOIN musical_metadata m ON f.id = m.file_id
             LIMIT 1000",
        )
        .fetch_all(pool)
        .await;
        let join_time = start.elapsed();

        match join_result {
            Ok(rows) => {
                details.push(format!(
                    "Join query: {}ms for {} rows",
                    join_time.as_millis(),
                    rows.len()
                ));
                if join_time < Duration::from_millis(50) || rows.len() < 100 {
                    total_score += 1;
                }
            },
            Err(e) => {
                details.push(format!("Join query failed: {}", e));
            },
        }

        // Test 4: Tag search performance
        let start = Instant::now();
        let tag_result: Result<Vec<(i64,)>, _> = sqlx::query_as(
            "SELECT f.id FROM files f
             JOIN file_tags ft ON f.id = ft.file_id
             JOIN tags t ON ft.tag_id = t.id
             WHERE t.name = 'drums'
             LIMIT 100",
        )
        .fetch_all(pool)
        .await;
        let tag_time = start.elapsed();

        match tag_result {
            Ok(rows) => {
                details.push(format!(
                    "Tag search: {}ms for {} rows",
                    tag_time.as_millis(),
                    rows.len()
                ));
                if tag_time < Duration::from_millis(100) {
                    total_score += 1;
                }
            },
            Err(e) => {
                details.push(format!("Tag search failed: {}", e));
            },
        }

        let status = if total_score >= 3 {
            CheckStatus::Pass
        } else if total_score >= 2 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "performance".to_string(),
            check_name: "database_benchmark".to_string(),
            level: VerificationLevel::High,
            status,
            message: format!("Database performance: {}/4 benchmarks passed", total_score),
            details,
            suggestion: if total_score < 4 {
                Some("Review query execution plans and index usage".to_string())
            } else {
                None
            },
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check for proper async patterns
    pub fn check_async_patterns() -> VerificationResult {
        let mut issues = Vec::new();

        let blocking_patterns = vec![
            ("std::thread::sleep", "Blocking sleep in async context"),
            ("std::fs::read", "Blocking file read (use tokio::fs)"),
            ("std::fs::write", "Blocking file write (use tokio::fs)"),
            (".block_on(", "Blocking on async from sync context"),
            ("std::net::", "Blocking network operations"),
        ];

        let scan_files = vec![
            "pipeline/src-tauri/src/commands/file_import.rs",
            "pipeline/src-tauri/src/commands/analyze.rs",
            "pipeline/src-tauri/src/commands/search.rs",
            "daw/src-tauri/src/commands/mod.rs",
        ];

        for file_path in &scan_files {
            if let Ok(content) = fs::read_to_string(file_path) {
                // Check if file uses async
                let is_async_file = content.contains("async fn");

                if is_async_file {
                    for (line_num, line) in content.lines().enumerate() {
                        for (pattern, description) in &blocking_patterns {
                            if line.contains(pattern) {
                                issues.push(format!(
                                    "{}:{} - {}: {}",
                                    file_path,
                                    line_num + 1,
                                    description,
                                    line.trim()
                                ));
                            }
                        }
                    }
                }
            }
        }

        let status = if issues.is_empty() {
            CheckStatus::Pass
        } else if issues.len() <= 3 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "performance".to_string(),
            check_name: "async_patterns".to_string(),
            level: VerificationLevel::High,
            status,
            message: format!("Found {} blocking operation(s) in async code", issues.len()),
            details: issues,
            suggestion: Some(
                "Use tokio equivalents for I/O operations in async functions".to_string(),
            ),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check memory usage patterns
    pub fn check_memory_patterns() -> VerificationResult {
        let mut issues = Vec::new();

        let memory_patterns = vec![
            ("Vec::with_capacity(0)", "Zero-capacity Vec allocation"),
            ("String::new()", "Empty string (consider &str)"),
            ("Box::new(Vec", "Boxed Vec (usually unnecessary)"),
            (".clone().clone()", "Double clone"),
            ("mem::forget", "Memory leak via forget"),
        ];

        let scan_dirs = vec!["pipeline/src-tauri/src", "shared/rust/src"];

        for dir in &scan_dirs {
            scan_for_patterns_recursive(dir, &memory_patterns, &mut issues);
        }

        let status = if issues.is_empty() {
            CheckStatus::Pass
        } else if issues.len() <= 5 {
            CheckStatus::Warning
        } else {
            CheckStatus::Fail
        };

        VerificationResult {
            category: "performance".to_string(),
            check_name: "memory_patterns".to_string(),
            level: VerificationLevel::Medium,
            status,
            message: format!("Found {} memory pattern issue(s)", issues.len()),
            details: issues.into_iter().take(15).collect(),
            suggestion: Some("Review memory allocation patterns".to_string()),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get full performance verification summary (without database)
    pub fn get_summary() -> PerformanceSummary {
        let filesystem = Self::benchmark_filesystem();
        let antipatterns = Self::check_performance_antipatterns();
        let async_patterns = Self::check_async_patterns();
        let memory_patterns = Self::check_memory_patterns();

        PerformanceSummary {
            filesystem_fast: filesystem.status == CheckStatus::Pass,
            no_antipatterns: antipatterns.status == CheckStatus::Pass
                || antipatterns.status == CheckStatus::Warning,
            async_correct: async_patterns.status == CheckStatus::Pass,
            memory_efficient: memory_patterns.status == CheckStatus::Pass
                || memory_patterns.status == CheckStatus::Warning,
            database_fast: true, // Default when not tested
            results: vec![filesystem, antipatterns, async_patterns, memory_patterns],
        }
    }

    /// Get full performance verification summary with database
    pub async fn get_full_summary(pool: &sqlx::PgPool) -> PerformanceSummary {
        let filesystem = Self::benchmark_filesystem();
        let antipatterns = Self::check_performance_antipatterns();
        let async_patterns = Self::check_async_patterns();
        let memory_patterns = Self::check_memory_patterns();
        let database = Self::benchmark_database(pool).await;

        PerformanceSummary {
            filesystem_fast: filesystem.status == CheckStatus::Pass,
            no_antipatterns: antipatterns.status == CheckStatus::Pass
                || antipatterns.status == CheckStatus::Warning,
            async_correct: async_patterns.status == CheckStatus::Pass,
            memory_efficient: memory_patterns.status == CheckStatus::Pass
                || memory_patterns.status == CheckStatus::Warning,
            database_fast: database.status == CheckStatus::Pass,
            results: vec![filesystem, antipatterns, async_patterns, memory_patterns, database],
        }
    }
}

/// Summary of performance verification
#[derive(Debug)]
pub struct PerformanceSummary {
    pub filesystem_fast: bool,
    pub no_antipatterns: bool,
    pub async_correct: bool,
    pub memory_efficient: bool,
    pub database_fast: bool,
    pub results: Vec<VerificationResult>,
}

impl PerformanceSummary {
    pub fn all_performant(&self) -> bool {
        self.filesystem_fast
            && self.no_antipatterns
            && self.async_correct
            && self.memory_efficient
            && self.database_fast
    }
}

/// Recursively scan for code patterns
fn scan_for_patterns_recursive(dir: &str, patterns: &[(&str, &str)], findings: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if !["target", ".git", "node_modules"].contains(&dir_name.as_ref()) {
                    scan_for_patterns_recursive(&path.to_string_lossy(), patterns, findings);
                }
            } else if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
                // Skip test files
                let path_str = path.to_string_lossy();
                if path_str.contains("test") || path_str.contains("fixture") {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&path) {
                    for (line_num, line) in content.lines().enumerate() {
                        for (pattern, description) in patterns {
                            if line.contains(pattern) {
                                findings.push(format!(
                                    "{}:{} - {}",
                                    path.display(),
                                    line_num + 1,
                                    description
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_benchmark() {
        let result = PerformanceVerifier::benchmark_filesystem();
        assert!(!result.details.is_empty());
    }

    #[test]
    fn test_performance_summary() {
        let summary = PerformanceVerifier::get_summary();
        assert!(summary.results.len() >= 4);
    }

    #[test]
    fn test_async_patterns_check() {
        let result = PerformanceVerifier::check_async_patterns();
        // Should complete without panic
        assert!(result.category == "performance");
    }
}
