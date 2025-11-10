
use sqlx::types::BigDecimal;
/// Custom assertions for DAW command tests
use sqlx::PgPool;
use std::str::FromStr;

/// Assert file exists in database
pub async fn assert_file_exists(pool: &PgPool, file_id: i64) {
    let result = sqlx::query!("SELECT id FROM files WHERE id = $1", file_id)
        .fetch_optional(pool)
        .await
        .expect("Database query failed");

    assert!(
        result.is_some(),
        "Expected file {} to exist in database",
        file_id
    );
}

/// Assert file has specific metadata
pub async fn assert_file_has_metadata(pool: &PgPool, file_id: i64, expected_bpm: Option<f64>) {
    let result = sqlx::query!(
        "SELECT bpm FROM musical_metadata WHERE file_id = $1",
        file_id
    )
    .fetch_optional(pool)
    .await
    .expect("Database query failed");

    if let Some(expected) = expected_bpm {
        assert!(result.is_some(), "Expected metadata for file {}", file_id);
        let actual = result.unwrap().bpm;
        assert!(actual.is_some(), "Expected BPM for file {}", file_id);

        // Convert BigDecimal to f64 for comparison
        let actual_val = actual.unwrap();
        let expected_bd = BigDecimal::from_str(&expected.to_string())
            .unwrap_or_else(|_| BigDecimal::from_str("0").unwrap());
        let diff = if actual_val > expected_bd {
            (&actual_val - &expected_bd).to_string().parse::<f64>().unwrap_or(0.0)
        } else {
            (&expected_bd - &actual_val).to_string().parse::<f64>().unwrap_or(0.0)
        };

        assert!(diff < 0.01, "Expected BPM {}, got {}", expected, actual_val);
    }
}

/// Assert favorite exists
pub async fn assert_favorite_exists(pool: &PgPool, file_id: i64, should_exist: bool) {
    let result = sqlx::query!("SELECT file_id FROM favorites WHERE file_id = $1", file_id)
        .fetch_optional(pool)
        .await
        .expect("Database query failed");

    if should_exist {
        assert!(
            result.is_some(),
            "Expected favorite for file {} to exist",
            file_id
        );
    } else {
        assert!(
            result.is_none(),
            "Expected favorite for file {} to not exist",
            file_id
        );
    }
}

/// Assert BPM within range
pub fn assert_bpm_in_range(actual: f32, expected: f32, tolerance: f32) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "BPM {} not within {} of expected {}",
        actual,
        tolerance,
        expected
    );
}

/// Assert tick position valid
pub fn assert_tick_valid(tick: u64, max_tick: u64) {
    assert!(
        tick <= max_tick,
        "Tick {} exceeds maximum {}",
        tick,
        max_tick
    );
}

/// Assert MIDI channel valid (0-15)
pub fn assert_channel_valid(channel: u8) {
    assert!(
        channel < 16,
        "MIDI channel {} invalid (must be 0-15)",
        channel
    );
}

/// Assert MIDI note valid (0-127)
pub fn assert_note_valid(note: u8) {
    assert!(note <= 127, "MIDI note {} invalid (must be 0-127)", note);
}

/// Assert MIDI velocity valid (0-127)
pub fn assert_velocity_valid(velocity: u8) {
    assert!(
        velocity <= 127,
        "MIDI velocity {} invalid (must be 0-127)",
        velocity
    );
}

/// Assert result is error with specific message
pub fn assert_error_contains(result: Result<(), String>, expected_msg: &str) {
    assert!(result.is_err(), "Expected error, got Ok");
    let error = result.unwrap_err();
    assert!(
        error.contains(expected_msg),
        "Error '{}' does not contain '{}'",
        error,
        expected_msg
    );
}

/// Assert vec length
pub fn assert_length<T>(vec: &[T], expected: usize) {
    assert_eq!(
        vec.len(),
        expected,
        "Expected length {}, got {}",
        expected,
        vec.len()
    );
}

/// Assert vec not empty
pub fn assert_not_empty<T>(vec: &[T]) {
    assert!(!vec.is_empty(), "Expected non-empty vector");
}

/// Assert value in range
pub fn assert_in_range<T: PartialOrd + std::fmt::Debug>(value: T, min: T, max: T) {
    assert!(
        value >= min && value <= max,
        "Value {:?} not in range {:?}..{:?}",
        value,
        min,
        max
    );
}
