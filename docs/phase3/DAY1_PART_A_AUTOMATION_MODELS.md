# Day 1, Part 1A: Automation Models

**Duration:** 2.5 hours
**Prerequisites:** Phase 2 complete (mixer system functional)
**Files to create:** 3

---

## Overview

Build automation system foundation:
1. AutomationLane model (per-parameter automation)
2. AutomationPoint model (individual points in automation curve)
3. Interpolation algorithms (linear, step, cubic)
4. Database schema for automation storage

---

## Step 1: Automation Point Model (30 min)

Create `app/src-tauri/src/daw/automation/models/automation_point.rs`:

```rust
use serde::{Deserialize, Serialize};

/// A single point in an automation curve
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationPoint {
    /// Time position in ticks (MIDI time)
    pub time_ticks: u64,

    /// Value at this point (normalized 0.0 to 1.0)
    pub value: f32,

    /// Curve type from this point to the next
    pub curve_type: CurveType,
}

/// Interpolation curve types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CurveType {
    /// Linear interpolation (straight line)
    Linear,

    /// Step (no interpolation, hold value)
    Step,

    /// Cubic spline (smooth curve)
    Cubic,
}

impl AutomationPoint {
    pub fn new(time_ticks: u64, value: f32) -> Self {
        Self {
            time_ticks,
            value: value.clamp(0.0, 1.0),
            curve_type: CurveType::Linear,
        }
    }

    pub fn with_curve(time_ticks: u64, value: f32, curve_type: CurveType) -> Self {
        Self {
            time_ticks,
            value: value.clamp(0.0, 1.0),
            curve_type,
        }
    }

    /// Interpolate value between this point and next point
    pub fn interpolate_to(&self, next: &AutomationPoint, time_ticks: u64) -> f32 {
        if time_ticks <= self.time_ticks {
            return self.value;
        }

        if time_ticks >= next.time_ticks {
            return next.value;
        }

        match self.curve_type {
            CurveType::Step => self.value,
            CurveType::Linear => {
                let t = (time_ticks - self.time_ticks) as f32
                    / (next.time_ticks - self.time_ticks) as f32;
                self.value + (next.value - self.value) * t
            }
            CurveType::Cubic => {
                let t = (time_ticks - self.time_ticks) as f32
                    / (next.time_ticks - self.time_ticks) as f32;
                // Hermite cubic interpolation
                let t2 = t * t;
                let t3 = t2 * t;
                let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
                let h10 = t3 - 2.0 * t2 + t;
                let h01 = -2.0 * t3 + 3.0 * t2;
                let h11 = t3 - t2;

                let m0 = 0.0; // Tangent at start
                let m1 = 0.0; // Tangent at end

                h00 * self.value + h10 * m0 + h01 * next.value + h11 * m1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_interpolation() {
        let p1 = AutomationPoint::new(0, 0.0);
        let p2 = AutomationPoint::new(100, 1.0);

        assert_eq!(p1.interpolate_to(&p2, 0), 0.0);
        assert_eq!(p1.interpolate_to(&p2, 50), 0.5);
        assert_eq!(p1.interpolate_to(&p2, 100), 1.0);
    }

    #[test]
    fn test_step_interpolation() {
        let p1 = AutomationPoint::with_curve(0, 0.0, CurveType::Step);
        let p2 = AutomationPoint::new(100, 1.0);

        assert_eq!(p1.interpolate_to(&p2, 0), 0.0);
        assert_eq!(p1.interpolate_to(&p2, 50), 0.0);
        assert_eq!(p1.interpolate_to(&p2, 99), 0.0);
        assert_eq!(p1.interpolate_to(&p2, 100), 1.0);
    }

    #[test]
    fn test_value_clamping() {
        let p1 = AutomationPoint::new(0, -0.5);
        assert_eq!(p1.value, 0.0);

        let p2 = AutomationPoint::new(0, 1.5);
        assert_eq!(p2.value, 1.0);
    }
}
```

---

## Step 2: Automation Lane Model (45 min)

Create `app/src-tauri/src/daw/automation/models/automation_lane.rs`:

```rust
use super::automation_point::{AutomationPoint, CurveType};
use serde::{Deserialize, Serialize};

/// Automation lane for a single parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationLane {
    /// Unique identifier
    pub id: String,

    /// Track this automation belongs to
    pub track_id: u32,

    /// Parameter being automated (e.g., "gain", "pan", "effect1.reverb_mix")
    pub parameter_id: String,

    /// Display name
    pub name: String,

    /// Automation points (sorted by time)
    pub points: Vec<AutomationPoint>,

    /// Whether automation is enabled
    pub enabled: bool,

    /// Automation mode
    pub mode: AutomationMode,
}

/// Automation recording/playback mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AutomationMode {
    /// Read automation (playback only)
    Read,

    /// Write automation (record, overwrite existing)
    Write,

    /// Latch (start recording on first change, continue until stop)
    Latch,

    /// Touch (record only while adjusting, revert to read when released)
    Touch,

    /// Off (ignore automation data)
    Off,
}

impl AutomationLane {
    pub fn new(id: String, track_id: u32, parameter_id: String, name: String) -> Self {
        Self {
            id,
            track_id,
            parameter_id,
            name,
            points: Vec::new(),
            enabled: true,
            mode: AutomationMode::Read,
        }
    }

    /// Add a point to the lane (maintains sorted order)
    pub fn add_point(&mut self, point: AutomationPoint) {
        // Find insertion position
        let pos = self.points
            .binary_search_by_key(&point.time_ticks, |p| p.time_ticks)
            .unwrap_or_else(|e| e);

        // If point exists at this time, replace it
        if pos < self.points.len() && self.points[pos].time_ticks == point.time_ticks {
            self.points[pos] = point;
        } else {
            self.points.insert(pos, point);
        }
    }

    /// Remove point at specific time
    pub fn remove_point(&mut self, time_ticks: u64) -> Option<AutomationPoint> {
        if let Some(pos) = self.points.iter().position(|p| p.time_ticks == time_ticks) {
            Some(self.points.remove(pos))
        } else {
            None
        }
    }

    /// Get value at specific time (with interpolation)
    pub fn get_value_at(&self, time_ticks: u64) -> Option<f32> {
        if !self.enabled || self.points.is_empty() {
            return None;
        }

        // Before first point
        if time_ticks < self.points[0].time_ticks {
            return Some(self.points[0].value);
        }

        // After last point
        if time_ticks >= self.points.last().unwrap().time_ticks {
            return Some(self.points.last().unwrap().value);
        }

        // Find surrounding points
        for i in 0..self.points.len() - 1 {
            if time_ticks >= self.points[i].time_ticks
                && time_ticks < self.points[i + 1].time_ticks
            {
                return Some(self.points[i].interpolate_to(&self.points[i + 1], time_ticks));
            }
        }

        None
    }

    /// Clear all points
    pub fn clear(&mut self) {
        self.points.clear();
    }

    /// Get points in time range
    pub fn get_points_in_range(&self, start_ticks: u64, end_ticks: u64) -> Vec<&AutomationPoint> {
        self.points
            .iter()
            .filter(|p| p.time_ticks >= start_ticks && p.time_ticks <= end_ticks)
            .collect()
    }

    /// Remove points in time range
    pub fn remove_points_in_range(&mut self, start_ticks: u64, end_ticks: u64) {
        self.points.retain(|p| p.time_ticks < start_ticks || p.time_ticks > end_ticks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_points() {
        let mut lane = AutomationLane::new(
            "auto1".to_string(),
            1,
            "gain".to_string(),
            "Gain".to_string(),
        );

        lane.add_point(AutomationPoint::new(100, 0.5));
        lane.add_point(AutomationPoint::new(0, 0.0));
        lane.add_point(AutomationPoint::new(50, 0.25));

        assert_eq!(lane.points.len(), 3);
        assert_eq!(lane.points[0].time_ticks, 0);
        assert_eq!(lane.points[1].time_ticks, 50);
        assert_eq!(lane.points[2].time_ticks, 100);
    }

    #[test]
    fn test_replace_point() {
        let mut lane = AutomationLane::new(
            "auto1".to_string(),
            1,
            "gain".to_string(),
            "Gain".to_string(),
        );

        lane.add_point(AutomationPoint::new(50, 0.5));
        lane.add_point(AutomationPoint::new(50, 0.8));

        assert_eq!(lane.points.len(), 1);
        assert_eq!(lane.points[0].value, 0.8);
    }

    #[test]
    fn test_get_value_at() {
        let mut lane = AutomationLane::new(
            "auto1".to_string(),
            1,
            "gain".to_string(),
            "Gain".to_string(),
        );

        lane.add_point(AutomationPoint::new(0, 0.0));
        lane.add_point(AutomationPoint::new(100, 1.0));

        assert_eq!(lane.get_value_at(0), Some(0.0));
        assert_eq!(lane.get_value_at(50), Some(0.5));
        assert_eq!(lane.get_value_at(100), Some(1.0));
        assert_eq!(lane.get_value_at(150), Some(1.0)); // After last point
    }

    #[test]
    fn test_remove_points_in_range() {
        let mut lane = AutomationLane::new(
            "auto1".to_string(),
            1,
            "gain".to_string(),
            "Gain".to_string(),
        );

        lane.add_point(AutomationPoint::new(0, 0.0));
        lane.add_point(AutomationPoint::new(50, 0.5));
        lane.add_point(AutomationPoint::new(100, 1.0));

        lane.remove_points_in_range(25, 75);

        assert_eq!(lane.points.len(), 2);
        assert_eq!(lane.points[0].time_ticks, 0);
        assert_eq!(lane.points[1].time_ticks, 100);
    }
}
```

---

## Step 3: Module Organization (15 min)

Create `app/src-tauri/src/daw/automation/models/mod.rs`:

```rust
pub mod automation_point;
pub mod automation_lane;

pub use automation_point::{AutomationPoint, CurveType};
pub use automation_lane::{AutomationLane, AutomationMode};
```

Create `app/src-tauri/src/daw/automation/mod.rs`:

```rust
pub mod models;

pub use models::{AutomationPoint, AutomationLane, AutomationMode, CurveType};
```

Update `app/src-tauri/src/daw/mod.rs`:

```rust
pub mod mixer;
pub mod sequencer;
pub mod automation;  // NEW

pub use automation::{AutomationPoint, AutomationLane, AutomationMode, CurveType};
```

---

## Step 4: Database Schema (45 min)

Create `database/migrations/022_automation_presets_projects.sql`:

```sql
-- Automation lanes
CREATE TABLE IF NOT EXISTS automation_lanes (
    id TEXT PRIMARY KEY,
    track_id INTEGER NOT NULL,
    parameter_id TEXT NOT NULL,
    name TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    mode TEXT NOT NULL DEFAULT 'Read',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(track_id, parameter_id)
);

CREATE INDEX idx_automation_lanes_track ON automation_lanes(track_id);
CREATE INDEX idx_automation_lanes_parameter ON automation_lanes(parameter_id);

-- Automation points
CREATE TABLE IF NOT EXISTS automation_points (
    id SERIAL PRIMARY KEY,
    lane_id TEXT NOT NULL REFERENCES automation_lanes(id) ON DELETE CASCADE,
    time_ticks BIGINT NOT NULL,
    value REAL NOT NULL CHECK (value >= 0.0 AND value <= 1.0),
    curve_type TEXT NOT NULL DEFAULT 'Linear',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(lane_id, time_ticks)
);

CREATE INDEX idx_automation_points_lane ON automation_points(lane_id);
CREATE INDEX idx_automation_points_time ON automation_points(lane_id, time_ticks);

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_automation_lane_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER automation_lane_update_timestamp
    BEFORE UPDATE ON automation_lanes
    FOR EACH ROW
    EXECUTE FUNCTION update_automation_lane_timestamp();

-- Helper function to get automation value at time
CREATE OR REPLACE FUNCTION get_automation_value(
    p_lane_id TEXT,
    p_time_ticks BIGINT
) RETURNS REAL AS $$
DECLARE
    v_value REAL;
    v_prev_point RECORD;
    v_next_point RECORD;
    v_t REAL;
BEGIN
    -- Get surrounding points
    SELECT * INTO v_prev_point
    FROM automation_points
    WHERE lane_id = p_lane_id AND time_ticks <= p_time_ticks
    ORDER BY time_ticks DESC
    LIMIT 1;

    SELECT * INTO v_next_point
    FROM automation_points
    WHERE lane_id = p_lane_id AND time_ticks > p_time_ticks
    ORDER BY time_ticks ASC
    LIMIT 1;

    -- If no previous point, return NULL
    IF v_prev_point IS NULL THEN
        RETURN NULL;
    END IF;

    -- If no next point, return previous point value
    IF v_next_point IS NULL THEN
        RETURN v_prev_point.value;
    END IF;

    -- If curve type is Step, return previous value
    IF v_prev_point.curve_type = 'Step' THEN
        RETURN v_prev_point.value;
    END IF;

    -- Linear interpolation
    v_t := (p_time_ticks - v_prev_point.time_ticks)::REAL
         / (v_next_point.time_ticks - v_prev_point.time_ticks)::REAL;

    v_value := v_prev_point.value + (v_next_point.value - v_prev_point.value) * v_t;

    RETURN v_value;
END;
$$ LANGUAGE plpgsql;
```

Apply migration:

```bash
psql "postgresql://midiuser:145278963@localhost:5433/midi_library" \
  -f database/migrations/022_automation_presets_projects.sql
```

---

## Step 5: Repository Layer (30 min)

Create `app/src-tauri/src/db/repositories/automation_repository.rs`:

```rust
use crate::daw::automation::{AutomationLane, AutomationPoint, AutomationMode, CurveType};
use sqlx::PgPool;

pub struct AutomationRepository {
    pool: PgPool,
}

impl AutomationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Save automation lane (without points)
    pub async fn save_lane(&self, lane: &AutomationLane) -> Result<(), sqlx::Error> {
        let mode_str = format!("{:?}", lane.mode);

        sqlx::query!(
            r#"
            INSERT INTO automation_lanes (id, track_id, parameter_id, name, enabled, mode)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (track_id, parameter_id)
            DO UPDATE SET
                name = EXCLUDED.name,
                enabled = EXCLUDED.enabled,
                mode = EXCLUDED.mode,
                updated_at = CURRENT_TIMESTAMP
            "#,
            lane.id,
            lane.track_id as i32,
            lane.parameter_id,
            lane.name,
            lane.enabled,
            mode_str,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Save automation points for a lane
    pub async fn save_points(&self, lane_id: &str, points: &[AutomationPoint]) -> Result<(), sqlx::Error> {
        // Delete existing points
        sqlx::query!("DELETE FROM automation_points WHERE lane_id = $1", lane_id)
            .execute(&self.pool)
            .await?;

        // Insert new points
        for point in points {
            let curve_type_str = format!("{:?}", point.curve_type);

            sqlx::query!(
                r#"
                INSERT INTO automation_points (lane_id, time_ticks, value, curve_type)
                VALUES ($1, $2, $3, $4)
                "#,
                lane_id,
                point.time_ticks as i64,
                point.value,
                curve_type_str,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Load automation lane by ID
    pub async fn load_lane(&self, lane_id: &str) -> Result<Option<AutomationLane>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, track_id, parameter_id, name, enabled, mode
            FROM automation_lanes
            WHERE id = $1
            "#,
            lane_id,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let mode = match row.mode.as_str() {
                "Read" => AutomationMode::Read,
                "Write" => AutomationMode::Write,
                "Latch" => AutomationMode::Latch,
                "Touch" => AutomationMode::Touch,
                "Off" => AutomationMode::Off,
                _ => AutomationMode::Read,
            };

            let points = self.load_points(lane_id).await?;

            Ok(Some(AutomationLane {
                id: row.id,
                track_id: row.track_id as u32,
                parameter_id: row.parameter_id,
                name: row.name,
                points,
                enabled: row.enabled,
                mode,
            }))
        } else {
            Ok(None)
        }
    }

    /// Load automation points for a lane
    pub async fn load_points(&self, lane_id: &str) -> Result<Vec<AutomationPoint>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT time_ticks, value, curve_type
            FROM automation_points
            WHERE lane_id = $1
            ORDER BY time_ticks ASC
            "#,
            lane_id,
        )
        .fetch_all(&self.pool)
        .await?;

        let points = rows
            .into_iter()
            .map(|row| {
                let curve_type = match row.curve_type.as_str() {
                    "Step" => CurveType::Step,
                    "Cubic" => CurveType::Cubic,
                    _ => CurveType::Linear,
                };

                AutomationPoint {
                    time_ticks: row.time_ticks as u64,
                    value: row.value,
                    curve_type,
                }
            })
            .collect();

        Ok(points)
    }

    /// Load all lanes for a track
    pub async fn load_track_lanes(&self, track_id: u32) -> Result<Vec<AutomationLane>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT id
            FROM automation_lanes
            WHERE track_id = $1
            "#,
            track_id as i32,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut lanes = Vec::new();
        for row in rows {
            if let Some(lane) = self.load_lane(&row.id).await? {
                lanes.push(lane);
            }
        }

        Ok(lanes)
    }

    /// Delete automation lane
    pub async fn delete_lane(&self, lane_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM automation_lanes WHERE id = $1", lane_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
```

Add to `app/src-tauri/src/db/repositories/mod.rs`:

```rust
pub mod automation_repository;
pub use automation_repository::AutomationRepository;
```

---

## Verification (15 min)

```bash
cd app/src-tauri
cargo check
cargo test --lib automation
```

Test database functions:

```sql
-- Insert test lane
INSERT INTO automation_lanes (id, track_id, parameter_id, name)
VALUES ('test1', 1, 'gain', 'Gain');

-- Insert test points
INSERT INTO automation_points (lane_id, time_ticks, value, curve_type)
VALUES
    ('test1', 0, 0.0, 'Linear'),
    ('test1', 100, 0.5, 'Linear'),
    ('test1', 200, 1.0, 'Linear');

-- Test interpolation function
SELECT get_automation_value('test1', 50);  -- Should return ~0.25
SELECT get_automation_value('test1', 150); -- Should return ~0.75
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Points not sorted | `add_point()` uses binary search to maintain order |
| Interpolation incorrect | Check curve type, verify time_ticks calculation |
| Database constraint error | Ensure unique (track_id, parameter_id) per lane |
| Value out of range | AutomationPoint clamps values to 0.0-1.0 |

---

## What's Next?

✅ **You've completed:**
- AutomationPoint model with 3 interpolation types
- AutomationLane model with sorted point management
- Database schema for automation storage
- Repository layer for persistence
- 8 unit tests covering core functionality

**Next:** [Part 1B: Automation Recording](./DAY1_PART_B_AUTOMATION_RECORDING.md)
- Automation recorder component
- Recording modes (write, latch, touch)
- Tauri commands for automation
- Parameter change capture
