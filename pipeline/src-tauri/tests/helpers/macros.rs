#![allow(dead_code, unused_imports, unused_variables)]
//! Test macros for common testing patterns
//!
//! Provides convenience macros for database testing, error handling,
//! and performance benchmarking in tests.

/// Execute a test within a database transaction that automatically rolls back
///
/// This macro wraps test code in a transaction that is never committed,
/// ensuring database state is always clean after tests.
///
/// # Example
///
/// ```no_run
/// use crate::helpers::db::setup_test_pool;
/// use crate::test_transaction;
///
/// #[tokio::test]
/// async fn test_insert_file() {
///     test_transaction!(pool, tx, {
///         // Your test code here
///         // tx will be rolled back automatically
///     });
/// }
/// ```
#[macro_export]
macro_rules! test_transaction {
    ($pool:ident, $tx:ident, $body:block) => {{
        let $pool = $crate::helpers::db::setup_test_pool().await;
        let mut $tx = $crate::helpers::db::create_transaction(&$pool).await;

        $body

        // Transaction is dropped here and automatically rolled back
    }};
}

/// Assert that a database error matches a specific error type
///
/// Useful for testing error handling in repository code.
///
/// # Example
///
/// ```no_run
/// use sqlx::Error;
///
/// let result = repository.insert(&pool, invalid_data).await;
/// assert_db_error!(result, Error::Database);
/// ```
#[macro_export]
macro_rules! assert_db_error {
    ($result:expr, $error_type:pat) => {{
        match $result {
            Err($error_type) => {
                // Expected error type
            },
            Err(e) => panic!(
                "Expected error type {}, but got: {:?}",
                stringify!($error_type),
                e
            ),
            Ok(_) => panic!(
                "Expected error type {}, but got Ok",
                stringify!($error_type)
            ),
        }
    }};
}

/// Assert that a database error contains a specific message substring
///
/// # Example
///
/// ```no_run
/// let result = repository.insert(&pool, duplicate_data).await;
/// assert_db_error_contains!(result, "duplicate key");
/// ```
#[macro_export]
macro_rules! assert_db_error_contains {
    ($result:expr, $substring:expr) => {{
        match $result {
            Err(e) => {
                let error_msg = format!("{:?}", e);
                assert!(
                    error_msg.contains($substring),
                    "Expected error to contain '{}', but got: {}",
                    $substring,
                    error_msg
                );
            },
            Ok(_) => panic!("Expected error containing '{}', but got Ok", $substring),
        }
    }};
}

/// Benchmark a query and assert it completes within a time threshold
///
/// Measures query execution time and fails if it exceeds the threshold.
///
/// # Example
///
/// ```no_run
/// benchmark_query!(
///     {
///         repository.search(&pool, complex_query).await.unwrap()
///     },
///     100, // milliseconds
///     "Complex search query"
/// );
/// ```
#[macro_export]
macro_rules! benchmark_query {
    ($query:block, $threshold_ms:expr, $description:expr) => {{
        let start = std::time::Instant::now();
        let result = $query;
        let elapsed = start.elapsed().as_millis();

        assert!(
            elapsed <= $threshold_ms,
            "{} took {}ms, expected <= {}ms (PERFORMANCE REGRESSION)",
            $description,
            elapsed,
            $threshold_ms
        );

        println!(
            "✓ {} completed in {}ms (threshold: {}ms)",
            $description, elapsed, $threshold_ms
        );

        result
    }};
}

/// Assert that a query returns a specific number of rows
///
/// # Example
///
/// ```no_run
/// let files = repository.get_all(&pool).await.unwrap();
/// assert_row_count!(files, 5);
/// ```
#[macro_export]
macro_rules! assert_row_count {
    ($result:expr, $expected:expr) => {{
        let count = $result.len();
        assert_eq!(
            count, $expected,
            "Expected {} rows, but got {}",
            $expected, count
        );
    }};
}

/// Create a test file with minimal required fields
///
/// Quick shorthand for creating simple test files without using the builder.
///
/// # Example
///
/// ```no_run
/// let file = test_file!("test.mid", "/tmp/test.mid");
/// ```
#[macro_export]
macro_rules! test_file {
    ($filename:expr, $filepath:expr) => {{
        $crate::fixtures::NewFileBuilder::new()
            .filename($filename)
            .filepath($filepath)
            .build()
    }};

    ($filename:expr, $filepath:expr, $hash:expr) => {{
        $crate::fixtures::NewFileBuilder::new()
            .filename($filename)
            .filepath($filepath)
            .content_hash($hash)
            .build()
    }};
}

/// Create a test tag with minimal required fields
///
/// # Example
///
/// ```no_run
/// let tag = test_tag!("drums");
/// let categorized_tag = test_tag!("house", "genre");
/// ```
#[macro_export]
macro_rules! test_tag {
    ($name:expr) => {{
        $crate::fixtures::NewTagBuilder::new($name).build()
    }};

    ($name:expr, $category:expr) => {{
        $crate::fixtures::NewTagBuilder::new($name).category($category).build()
    }};
}

/// Assert that two BigDecimal values are approximately equal
///
/// Useful for comparing floating-point database values (BPM, confidence, etc.).
///
/// # Example
///
/// ```no_run
/// use sqlx::types::BigDecimal;
/// use std::str::FromStr;
///
/// let expected = BigDecimal::from_str("120.0").unwrap();
/// let actual = BigDecimal::from_str("120.001").unwrap();
/// assert_bigdecimal_approx!(actual, expected, "0.01");
/// ```
#[macro_export]
macro_rules! assert_bigdecimal_approx {
    ($actual:expr, $expected:expr, $tolerance:expr) => {{
        use sqlx::types::BigDecimal;
        use std::str::FromStr;

        let tolerance = BigDecimal::from_str($tolerance).expect("Invalid tolerance");
        let diff = if $actual > $expected {
            &$actual - &$expected
        } else {
            &$expected - &$actual
        };

        assert!(
            diff <= tolerance,
            "BigDecimal values differ by more than {}: expected {}, got {}",
            $tolerance,
            $expected,
            $actual
        );
    }};
}

/// Assert that an optional value is Some and matches a condition
///
/// # Example
///
/// ```no_run
/// assert_some_eq!(file.manufacturer, "Ableton");
/// ```
#[macro_export]
macro_rules! assert_some_eq {
    ($option:expr, $expected:expr) => {{
        match $option {
            Some(ref value) => assert_eq!(value, &$expected),
            None => panic!("Expected Some({}), but got None", $expected),
        }
    }};
}

/// Assert that an optional value is None
///
/// # Example
///
/// ```no_run
/// assert_none!(file.parent_file_id);
/// ```
#[macro_export]
macro_rules! assert_none {
    ($option:expr) => {{
        assert!(
            $option.is_none(),
            "Expected None, but got Some({:?})",
            $option
        );
    }};
}

/// Retry a flaky database operation with exponential backoff
///
/// Useful for handling temporary database locks or connection issues in tests.
///
/// # Example
///
/// ```no_run
/// retry_db_operation!(
///     3, // max attempts
///     100, // initial delay ms
///     {
///         repository.insert(&pool, file).await
///     }
/// );
/// ```
#[macro_export]
macro_rules! retry_db_operation {
    ($max_attempts:expr, $initial_delay_ms:expr, $operation:block) => {{
        let mut attempts = 0;
        let mut delay_ms = $initial_delay_ms;

        loop {
            attempts += 1;

            match $operation {
                Ok(result) => break Ok(result),
                Err(e) if attempts >= $max_attempts => {
                    break Err(e);
                },
                Err(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    delay_ms *= 2; // Exponential backoff
                },
            }
        }
    }};
}

/// Log test progress with timing information
///
/// Useful for debugging slow tests or understanding test flow.
///
/// # Example
///
/// ```no_run
/// test_log!("Inserting 1000 files...");
/// // ... insert files ...
/// test_log!("Files inserted successfully");
/// ```
#[macro_export]
macro_rules! test_log {
    ($($arg:tt)*) => {{
        if cfg!(test) {
            println!("[TEST] {}: {}", chrono::Utc::now().format("%H:%M:%S%.3f"), format!($($arg)*));
        }
    }};
}

/// Create a test with automatic database cleanup before and after
///
/// Ensures a clean database state for each test.
///
/// # Example
///
/// ```no_run
/// test_with_cleanup!(test_name, {
///     // Test code here
///     // Database is cleaned before and after
/// });
/// ```
#[macro_export]
macro_rules! test_with_cleanup {
    ($test_name:ident, $body:block) => {
        #[tokio::test]
        async fn $test_name() {
            let pool = $crate::helpers::db::setup_test_pool().await;

            // Cleanup before test
            $crate::helpers::db::cleanup_database(&pool)
                .await
                .expect("Pre-test cleanup failed");

            // Run test
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| async { $body }));

            // Cleanup after test (even if test panicked)
            $crate::helpers::db::cleanup_database(&pool)
                .await
                .expect("Post-test cleanup failed");

            // Re-throw panic if test failed
            if let Err(e) = result {
                std::panic::resume_unwind(e);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use sqlx::types::BigDecimal;
    use std::str::FromStr;

    #[test]
    fn test_assert_some_eq() {
        let value = Some("test".to_string());
        assert_some_eq!(value, "test".to_string());
    }

    #[test]
    fn test_assert_none() {
        let value: Option<String> = None;
        assert_none!(value);
    }

    #[test]
    fn test_assert_bigdecimal_approx() {
        let a = BigDecimal::from_str("120.0").unwrap();
        let b = BigDecimal::from_str("120.001").unwrap();
        assert_bigdecimal_approx!(a, b, "0.01");
    }

    #[test]
    #[should_panic(expected = "BigDecimal values differ")]
    fn test_assert_bigdecimal_approx_fails() {
        let a = BigDecimal::from_str("120.0").unwrap();
        let b = BigDecimal::from_str("125.0").unwrap();
        assert_bigdecimal_approx!(a, b, "0.01");
    }

    #[tokio::test]
    async fn test_retry_db_operation_success() {
        let result: Result<i32, &str> = retry_db_operation!(3, 10, { Ok(42) });
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_db_operation_eventual_success() {
        let mut counter = 0;
        let result: Result<i32, &str> = retry_db_operation!(3, 10, {
            counter += 1;
            if counter < 2 {
                Err("temporary error")
            } else {
                Ok(42)
            }
        });
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter, 2);
    }

    #[test]
    fn test_assert_row_count() {
        let rows = [1, 2, 3, 4, 5];
        assert_row_count!(rows, 5);
    }
}
