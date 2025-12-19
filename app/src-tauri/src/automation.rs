/// Automation Lane System
///
/// Provides automation for track parameters (volume, pan, MIDI CC, custom parameters).
/// Implements point-based automation curves with multiple interpolation types.
///
/// Trusty Module: Pure data structures and algorithms for automation curves.
/// Grown-up Script: Tauri commands for automation lane management with side effects.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Automation point in time
///
/// Represents a single control point in an automation curve.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationPoint {
    /// Unique identifier for this point
    pub id: i32,
    /// Time position in MIDI ticks
    pub time: u64,
    /// Normalized value (0.0 to 1.0)
    pub value: f64,
}

impl AutomationPoint {
    /// Create a new automation point
    ///
    /// # Arguments
    /// * `id` - Unique identifier
    /// * `time` - Time position in ticks
    /// * `value` - Normalized value (will be clamped to 0.0-1.0)
    ///
    /// # Returns
    /// New automation point with clamped value
    pub fn new(id: i32, time: u64, value: f64) -> Self {
        Self { id, time, value: value.clamp(0.0, 1.0) }
    }

    /// Validate point data
    ///
    /// # Returns
    /// Ok if valid, Err with message if invalid
    pub fn validate(&self) -> Result<(), String> {
        if self.value < 0.0 || self.value > 1.0 {
            return Err(format!("Value {} out of range 0.0-1.0", self.value));
        }
        Ok(())
    }
}

/// Curve interpolation type
///
/// Defines how values are interpolated between automation points.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum CurveType {
    /// Linear interpolation (straight lines)
    #[default]
    Linear,
    /// Smooth bezier curves
    Bezier,
    /// Exponential curves (logarithmic feel)
    Exponential,
    /// Step (hold value until next point)
    Step,
    /// S-Curve (smooth ease-in-out)
    SCurve,
    /// Hold (alias for Step, keeps value constant)
    Hold,
}

/// Automation mode
///
/// Defines how automation behaves during playback and recording.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum AutomationMode {
    /// Automation is disabled, manual control only
    Off,
    /// Read automation data during playback
    #[default]
    Read,
    /// Write (overwrite) automation data during playback
    Write,
    /// Latch mode: write when parameter changes, hold last value
    Latch,
    /// Touch mode: write while touching, return to automation when released
    Touch,
}

/// Parameter type for automation
///
/// Defines which track parameter is being automated.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ParameterType {
    /// Track volume/gain (0-127 MIDI, or normalized 0.0-1.0)
    Volume,
    /// Alias for Volume (mixer gain control)
    Gain,
    /// Stereo pan (0-127, 64=center, or normalized -1.0 to 1.0)
    Pan,
    /// MIDI Control Change (0-127)
    CC(u8),
    /// Aux send level (send_id, level 0.0-1.0)
    Send(u32),
    /// Effect/plugin parameter (effect_id, param_id)
    EffectParam { effect_id: u32, param_id: u32 },
    /// Custom parameter (plugin/synth)
    Custom(u8),
}

impl ParameterType {
    /// Convert to display string
    pub fn as_string(&self) -> String {
        match self {
            Self::Volume => "Volume".to_string(),
            Self::Gain => "Gain".to_string(),
            Self::Pan => "Pan".to_string(),
            Self::CC(num) => format!("CC{}", num),
            Self::Send(id) => format!("Send {}", id),
            Self::EffectParam { effect_id, param_id } => {
                format!("Effect {} Param {}", effect_id, param_id)
            }
            Self::Custom(num) => format!("Custom{}", num),
        }
    }

    /// Get color for visualization
    pub fn color(&self) -> &'static str {
        match self {
            Self::Volume => "#4ade80",        // green
            Self::Gain => "#4ade80",          // green (same as Volume)
            Self::Pan => "#60a5fa",           // blue
            Self::CC(_) => "#a78bfa",         // purple
            Self::Send(_) => "#f97316",       // orange
            Self::EffectParam { .. } => "#ec4899", // pink
            Self::Custom(_) => "#fbbf24",     // yellow
        }
    }
}

/// Automation curve
///
/// Collection of points defining parameter automation over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationCurve {
    /// Automation points (sorted by time)
    pub points: Vec<AutomationPoint>,
    /// Curve interpolation type
    pub curve_type: CurveType,
    /// Next point ID
    next_id: i32,
}

impl AutomationCurve {
    /// Create new empty automation curve
    pub fn new() -> Self {
        Self { points: Vec::new(), curve_type: CurveType::Linear, next_id: 1 }
    }

    /// Add automation point
    ///
    /// # Arguments
    /// * `time` - Time position in ticks
    /// * `value` - Normalized value (0.0-1.0)
    ///
    /// # Returns
    /// ID of newly created point
    pub fn add_point(&mut self, time: u64, value: f64) -> Result<i32, String> {
        let point = AutomationPoint::new(self.next_id, time, value);
        point.validate()?;

        let id = self.next_id;
        self.next_id += 1;

        // Insert in sorted order by time
        let insert_idx =
            self.points.binary_search_by_key(&time, |p| p.time).unwrap_or_else(|idx| idx);

        self.points.insert(insert_idx, point);
        Ok(id)
    }

    /// Remove automation point by ID
    ///
    /// # Returns
    /// Ok if removed, Err if not found
    pub fn remove_point(&mut self, point_id: i32) -> Result<(), String> {
        let initial_len = self.points.len();
        self.points.retain(|p| p.id != point_id);

        if self.points.len() == initial_len {
            Err(format!("Point {} not found", point_id))
        } else {
            Ok(())
        }
    }

    /// Remove multiple automation points by ID (batch operation)
    ///
    /// # Arguments
    /// * `point_ids` - Vector of point IDs to remove
    ///
    /// # Returns
    /// Number of points removed
    pub fn remove_points_batch(&mut self, point_ids: &[i32]) -> usize {
        let initial_len = self.points.len();
        self.points.retain(|p| !point_ids.contains(&p.id));
        initial_len - self.points.len()
    }

    /// Move automation point
    ///
    /// # Arguments
    /// * `point_id` - ID of point to move
    /// * `new_time` - New time position
    /// * `new_value` - New normalized value
    ///
    /// # Returns
    /// Ok if moved, Err if not found or invalid value
    pub fn move_point(
        &mut self,
        point_id: i32,
        new_time: u64,
        new_value: f64,
    ) -> Result<(), String> {
        // Clamp value
        let clamped_value = new_value.clamp(0.0, 1.0);

        // Remove old point
        let old_point_idx = self
            .points
            .iter()
            .position(|p| p.id == point_id)
            .ok_or_else(|| format!("Point {} not found", point_id))?;

        self.points.remove(old_point_idx);

        // Create new point with same ID
        let point = AutomationPoint { id: point_id, time: new_time, value: clamped_value };

        // Insert in sorted order
        let insert_idx = self
            .points
            .binary_search_by_key(&new_time, |p| p.time)
            .unwrap_or_else(|idx| idx);

        self.points.insert(insert_idx, point);
        Ok(())
    }

    /// Get value at specific time using interpolation
    ///
    /// # Arguments
    /// * `time` - Time position in ticks
    ///
    /// # Returns
    /// Interpolated value at time, or None if no points exist
    pub fn get_value_at(&self, time: u64) -> Option<f64> {
        if self.points.is_empty() {
            return None;
        }

        // Find surrounding points
        let idx = self.points.binary_search_by_key(&time, |p| p.time);

        match idx {
            Ok(i) => {
                // Exact match
                Some(self.points[i].value)
            },
            Err(0) => {
                // Before first point
                Some(self.points[0].value)
            },
            Err(i) if i >= self.points.len() => {
                // After last point
                Some(self.points[self.points.len() - 1].value)
            },
            Err(i) => {
                // Between points i-1 and i
                let p1 = &self.points[i - 1];
                let p2 = &self.points[i];

                Some(self.interpolate(p1, p2, time))
            },
        }
    }

    /// Interpolate between two points
    ///
    /// # Arguments
    /// * `p1` - Start point
    /// * `p2` - End point
    /// * `time` - Time position between p1 and p2
    ///
    /// # Returns
    /// Interpolated value
    fn interpolate(&self, p1: &AutomationPoint, p2: &AutomationPoint, time: u64) -> f64 {
        if p1.time == p2.time {
            return p1.value;
        }

        // Normalized position (0.0 to 1.0)
        let t = (time.saturating_sub(p1.time)) as f64 / (p2.time - p1.time) as f64;
        let t = t.clamp(0.0, 1.0);

        match self.curve_type {
            CurveType::Linear => {
                // Linear interpolation
                p1.value + (p2.value - p1.value) * t
            },
            CurveType::Bezier => {
                // Smooth bezier curve (cubic ease in-out)
                let t_smooth = t * t * (3.0 - 2.0 * t);
                p1.value + (p2.value - p1.value) * t_smooth
            },
            CurveType::Exponential => {
                // Exponential curve
                let t_exp = if p2.value > p1.value {
                    // Rising: slow start, fast end
                    t * t
                } else {
                    // Falling: fast start, slow end
                    1.0 - (1.0 - t) * (1.0 - t)
                };
                p1.value + (p2.value - p1.value) * t_exp
            },
            CurveType::Step | CurveType::Hold => {
                // Hold value until next point
                p1.value
            },
            CurveType::SCurve => {
                // S-Curve (smooth ease-in-out with cubic bezier)
                // Uses smoothstep formula: 6t^5 - 15t^4 + 10t^3
                let t_smooth = t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
                p1.value + (p2.value - p1.value) * t_smooth
            },
        }
    }

    /// Get all points in time range
    ///
    /// # Arguments
    /// * `start_time` - Start of range (inclusive)
    /// * `end_time` - End of range (inclusive)
    ///
    /// # Returns
    /// Vector of points in range
    pub fn get_points_in_range(&self, start_time: u64, end_time: u64) -> Vec<AutomationPoint> {
        self.points
            .iter()
            .filter(|p| p.time >= start_time && p.time <= end_time)
            .cloned()
            .collect()
    }

    /// Clear all points
    pub fn clear(&mut self) {
        self.points.clear();
    }

    /// Get point count
    pub fn point_count(&self) -> usize {
        self.points.len()
    }
}

impl Default for AutomationCurve {
    fn default() -> Self {
        Self::new()
    }
}

/// Automation lane
///
/// Represents automation for a single parameter on a track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationLane {
    /// Unique lane identifier
    pub id: i32,
    /// Track this lane belongs to
    pub track_id: i32,
    /// Parameter being automated
    pub parameter_type: ParameterType,
    /// Automation curve data
    pub curve: AutomationCurve,
    /// Automation mode (Off, Read, Write, Latch, Touch)
    pub mode: AutomationMode,
    /// Lane enabled state
    pub enabled: bool,
    /// Display name (optional override)
    pub name: Option<String>,
}

impl AutomationLane {
    /// Create new automation lane
    ///
    /// # Arguments
    /// * `id` - Unique lane ID
    /// * `track_id` - Parent track ID
    /// * `parameter_type` - Parameter to automate
    pub fn new(id: i32, track_id: i32, parameter_type: ParameterType) -> Self {
        Self {
            id,
            track_id,
            parameter_type,
            curve: AutomationCurve::new(),
            mode: AutomationMode::Read,
            enabled: true,
            name: None,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| self.parameter_type.as_string())
    }

    /// Get color for visualization
    pub fn color(&self) -> &'static str {
        self.parameter_type.color()
    }
}

/// Automation track
///
/// Collection of all automation lanes for a single track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationTrack {
    /// Track this automation belongs to
    pub track_id: i32,
    /// Automation lanes by parameter type
    pub lanes: HashMap<ParameterType, AutomationLane>,
    /// Next lane ID
    next_lane_id: i32,
}

impl AutomationTrack {
    /// Create new automation track
    pub fn new(track_id: i32) -> Self {
        Self { track_id, lanes: HashMap::new(), next_lane_id: 1 }
    }

    /// Add automation lane
    ///
    /// # Arguments
    /// * `parameter_type` - Parameter to automate
    ///
    /// # Returns
    /// ID of created lane, or Err if lane already exists
    pub fn add_lane(&mut self, parameter_type: ParameterType) -> Result<i32, String> {
        if self.lanes.contains_key(&parameter_type) {
            return Err(format!("Lane for {:?} already exists", parameter_type));
        }

        let lane_id = self.next_lane_id;
        self.next_lane_id += 1;

        let lane = AutomationLane::new(lane_id, self.track_id, parameter_type);
        self.lanes.insert(parameter_type, lane);

        Ok(lane_id)
    }

    /// Remove automation lane
    ///
    /// # Returns
    /// Ok if removed, Err if not found
    pub fn remove_lane(&mut self, parameter_type: ParameterType) -> Result<(), String> {
        self.lanes
            .remove(&parameter_type)
            .ok_or_else(|| format!("Lane for {:?} not found", parameter_type))?;
        Ok(())
    }

    /// Get lane by parameter type
    pub fn get_lane(&self, parameter_type: ParameterType) -> Option<&AutomationLane> {
        self.lanes.get(&parameter_type)
    }

    /// Get mutable lane by parameter type
    pub fn get_lane_mut(&mut self, parameter_type: ParameterType) -> Option<&mut AutomationLane> {
        self.lanes.get_mut(&parameter_type)
    }

    /// Get all lanes as vector
    pub fn get_all_lanes(&self) -> Vec<&AutomationLane> {
        self.lanes.values().collect()
    }

    /// Get lane count
    pub fn lane_count(&self) -> usize {
        self.lanes.len()
    }
}

/// Automation manager
///
/// Manages automation for all tracks in the project.
/// Grown-up Script: Maintains global state with side effects.
pub struct AutomationManager {
    /// Automation tracks by track ID
    tracks: HashMap<i32, AutomationTrack>,
}

impl AutomationManager {
    /// Create new automation manager
    pub fn new() -> Self {
        Self { tracks: HashMap::new() }
    }

    /// Get or create automation track
    ///
    /// # Arguments
    /// * `track_id` - Track ID to get/create automation for
    ///
    /// # Returns
    /// Mutable reference to automation track
    fn get_or_create_track(&mut self, track_id: i32) -> &mut AutomationTrack {
        self.tracks.entry(track_id).or_insert_with(|| AutomationTrack::new(track_id))
    }

    /// Create automation lane
    ///
    /// # Arguments
    /// * `track_id` - Parent track ID
    /// * `parameter_type` - Parameter to automate
    ///
    /// # Returns
    /// Lane ID, or Err if lane already exists
    pub fn create_lane(
        &mut self,
        track_id: i32,
        parameter_type: ParameterType,
    ) -> Result<i32, String> {
        let track = self.get_or_create_track(track_id);
        track.add_lane(parameter_type)
    }

    /// Delete automation lane
    ///
    /// # Arguments
    /// * `track_id` - Parent track ID
    /// * `parameter_type` - Parameter type
    ///
    /// # Returns
    /// Ok if deleted, Err if not found
    pub fn delete_lane(
        &mut self,
        track_id: i32,
        parameter_type: ParameterType,
    ) -> Result<(), String> {
        let track = self
            .tracks
            .get_mut(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;
        track.remove_lane(parameter_type)
    }

    /// Add automation point
    ///
    /// # Arguments
    /// * `track_id` - Parent track ID
    /// * `parameter_type` - Parameter type
    /// * `time` - Time in ticks
    /// * `value` - Normalized value (0.0-1.0)
    ///
    /// # Returns
    /// Point ID, or Err if lane not found
    pub fn add_point(
        &mut self,
        track_id: i32,
        parameter_type: ParameterType,
        time: u64,
        value: f64,
    ) -> Result<i32, String> {
        let track = self
            .tracks
            .get_mut(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;

        let lane = track
            .get_lane_mut(parameter_type)
            .ok_or_else(|| format!("Lane for {:?} not found", parameter_type))?;

        lane.curve.add_point(time, value)
    }

    /// Remove automation point
    ///
    /// # Arguments
    /// * `track_id` - Parent track ID
    /// * `parameter_type` - Parameter type
    /// * `point_id` - Point ID to remove
    ///
    /// # Returns
    /// Ok if removed, Err if not found
    pub fn remove_point(
        &mut self,
        track_id: i32,
        parameter_type: ParameterType,
        point_id: i32,
    ) -> Result<(), String> {
        let track = self
            .tracks
            .get_mut(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;

        let lane = track
            .get_lane_mut(parameter_type)
            .ok_or_else(|| format!("Lane for {:?} not found", parameter_type))?;

        lane.curve.remove_point(point_id)
    }

    /// Remove multiple automation points (batch operation)
    ///
    /// # Arguments
    /// * `track_id` - Parent track ID
    /// * `parameter_type` - Parameter type
    /// * `point_ids` - Vector of point IDs to remove
    ///
    /// # Returns
    /// Number of points removed
    pub fn remove_points_batch(
        &mut self,
        track_id: i32,
        parameter_type: ParameterType,
        point_ids: &[i32],
    ) -> Result<usize, String> {
        let track = self
            .tracks
            .get_mut(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;

        let lane = track
            .get_lane_mut(parameter_type)
            .ok_or_else(|| format!("Lane for {:?} not found", parameter_type))?;

        Ok(lane.curve.remove_points_batch(point_ids))
    }

    /// Move automation point
    ///
    /// # Arguments
    /// * `track_id` - Parent track ID
    /// * `parameter_type` - Parameter type
    /// * `point_id` - Point ID to move
    /// * `new_time` - New time position
    /// * `new_value` - New normalized value
    ///
    /// # Returns
    /// Ok if moved, Err if not found or invalid
    pub fn move_point(
        &mut self,
        track_id: i32,
        parameter_type: ParameterType,
        point_id: i32,
        new_time: u64,
        new_value: f64,
    ) -> Result<(), String> {
        let track = self
            .tracks
            .get_mut(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;

        let lane = track
            .get_lane_mut(parameter_type)
            .ok_or_else(|| format!("Lane for {:?} not found", parameter_type))?;

        lane.curve.move_point(point_id, new_time, new_value)
    }

    /// Set curve type
    ///
    /// # Arguments
    /// * `track_id` - Parent track ID
    /// * `parameter_type` - Parameter type
    /// * `curve_type` - New curve type
    ///
    /// # Returns
    /// Ok if set, Err if lane not found
    pub fn set_curve_type(
        &mut self,
        track_id: i32,
        parameter_type: ParameterType,
        curve_type: CurveType,
    ) -> Result<(), String> {
        let track = self
            .tracks
            .get_mut(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;

        let lane = track
            .get_lane_mut(parameter_type)
            .ok_or_else(|| format!("Lane for {:?} not found", parameter_type))?;

        lane.curve.curve_type = curve_type;
        Ok(())
    }

    /// Get automation lane
    ///
    /// # Arguments
    /// * `track_id` - Parent track ID
    /// * `parameter_type` - Parameter type
    ///
    /// # Returns
    /// Lane, or Err if not found
    pub fn get_lane(
        &self,
        track_id: i32,
        parameter_type: ParameterType,
    ) -> Result<AutomationLane, String> {
        let track = self
            .tracks
            .get(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;

        track
            .get_lane(parameter_type)
            .cloned()
            .ok_or_else(|| format!("Lane for {:?} not found", parameter_type))
    }

    /// Get all lanes for track
    ///
    /// # Arguments
    /// * `track_id` - Track ID
    ///
    /// # Returns
    /// Vector of all lanes, or empty vector if track not found
    pub fn get_track_lanes(&self, track_id: i32) -> Vec<AutomationLane> {
        self.tracks
            .get(&track_id)
            .map(|track| track.get_all_lanes().into_iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get value at time
    ///
    /// # Arguments
    /// * `track_id` - Track ID
    /// * `parameter_type` - Parameter type
    /// * `time` - Time in ticks
    ///
    /// # Returns
    /// Interpolated value, or None if lane not found or no points
    pub fn get_value_at(
        &self,
        track_id: i32,
        parameter_type: ParameterType,
        time: u64,
    ) -> Option<f64> {
        let track = self.tracks.get(&track_id)?;
        let lane = track.get_lane(parameter_type)?;
        lane.curve.get_value_at(time)
    }

    /// Remove track automation
    ///
    /// # Arguments
    /// * `track_id` - Track ID to remove
    pub fn remove_track(&mut self, track_id: i32) {
        self.tracks.remove(&track_id);
    }

    /// Clear all automation
    pub fn clear_all(&mut self) {
        self.tracks.clear();
    }

    /// Get track by ID
    ///
    /// # Arguments
    /// * `track_id` - Track ID
    ///
    /// # Returns
    /// AutomationTrack reference, or Err if not found
    pub fn get_track(&self, track_id: i32) -> Result<&AutomationTrack, String> {
        self.tracks
            .get(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))
    }

    /// Get mutable lane for track/parameter
    ///
    /// # Arguments
    /// * `track_id` - Track ID
    /// * `parameter_type` - Parameter type
    ///
    /// # Returns
    /// Mutable lane reference, or Err if not found
    pub fn get_lane_mut(
        &mut self,
        track_id: i32,
        parameter_type: ParameterType,
    ) -> Result<&mut AutomationLane, String> {
        let track = self
            .tracks
            .get_mut(&track_id)
            .ok_or_else(|| format!("Track {} not found", track_id))?;

        track
            .get_lane_mut(parameter_type)
            .ok_or_else(|| format!("Lane for {:?} not found", parameter_type))
    }

    /// Set automation mode
    ///
    /// # Arguments
    /// * `track_id` - Track ID
    /// * `parameter_type` - Parameter type
    /// * `mode` - New automation mode
    ///
    /// # Returns
    /// Ok if set, Err if lane not found
    pub fn set_automation_mode(
        &mut self,
        track_id: i32,
        parameter_type: ParameterType,
        mode: AutomationMode,
    ) -> Result<(), String> {
        let lane = self.get_lane_mut(track_id, parameter_type)?;
        lane.mode = mode;
        Ok(())
    }

    /// Get automation mode
    ///
    /// # Arguments
    /// * `track_id` - Track ID
    /// * `parameter_type` - Parameter type
    ///
    /// # Returns
    /// Current automation mode, or Err if lane not found
    pub fn get_automation_mode(
        &self,
        track_id: i32,
        parameter_type: ParameterType,
    ) -> Result<AutomationMode, String> {
        let lane = self.get_lane(track_id, parameter_type)?;
        Ok(lane.mode)
    }
}

impl Default for AutomationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automation_point_new() {
        let point = AutomationPoint::new(1, 100, 0.5);
        assert_eq!(point.id, 1);
        assert_eq!(point.time, 100);
        assert_eq!(point.value, 0.5);
    }

    #[test]
    fn test_automation_point_clamps_value() {
        let point = AutomationPoint::new(1, 100, 1.5);
        assert_eq!(point.value, 1.0);

        let point = AutomationPoint::new(2, 100, -0.5);
        assert_eq!(point.value, 0.0);
    }

    #[test]
    fn test_automation_point_validate() {
        let point = AutomationPoint::new(1, 100, 0.5);
        assert!(point.validate().is_ok());

        let mut point = AutomationPoint::new(1, 100, 0.5);
        point.value = 1.5; // Bypass constructor clamping
        assert!(point.validate().is_err());
    }

    #[test]
    fn test_parameter_type_to_string() {
        assert_eq!(ParameterType::Volume.as_string(), "Volume");
        assert_eq!(ParameterType::Pan.as_string(), "Pan");
        assert_eq!(ParameterType::CC(1).as_string(), "CC1");
        assert_eq!(ParameterType::Custom(42).as_string(), "Custom42");
    }

    #[test]
    fn test_parameter_type_color() {
        assert_eq!(ParameterType::Volume.color(), "#4ade80");
        assert_eq!(ParameterType::Pan.color(), "#60a5fa");
        assert_eq!(ParameterType::CC(1).color(), "#a78bfa");
        assert_eq!(ParameterType::Custom(1).color(), "#fbbf24");
    }

    #[test]
    fn test_automation_curve_new() {
        let curve = AutomationCurve::new();
        assert_eq!(curve.points.len(), 0);
        assert_eq!(curve.curve_type, CurveType::Linear);
    }

    #[test]
    fn test_automation_curve_add_point() {
        let mut curve = AutomationCurve::new();

        let id1 = curve.add_point(100, 0.5).unwrap();
        assert_eq!(id1, 1);
        assert_eq!(curve.points.len(), 1);

        let id2 = curve.add_point(50, 0.3).unwrap();
        assert_eq!(id2, 2);
        assert_eq!(curve.points.len(), 2);

        // Should be sorted by time
        assert_eq!(curve.points[0].time, 50);
        assert_eq!(curve.points[1].time, 100);
    }

    #[test]
    fn test_automation_curve_add_point_clamps() {
        let mut curve = AutomationCurve::new();

        curve.add_point(100, 1.5).unwrap();
        assert_eq!(curve.points[0].value, 1.0);

        curve.add_point(200, -0.5).unwrap();
        assert_eq!(curve.points[1].value, 0.0);
    }

    #[test]
    fn test_automation_curve_remove_point() {
        let mut curve = AutomationCurve::new();
        let id = curve.add_point(100, 0.5).unwrap();

        assert!(curve.remove_point(id).is_ok());
        assert_eq!(curve.points.len(), 0);

        assert!(curve.remove_point(999).is_err());
    }

    #[test]
    fn test_automation_curve_move_point() {
        let mut curve = AutomationCurve::new();

        curve.add_point(50, 0.3).unwrap();
        let id = curve.add_point(100, 0.5).unwrap();
        curve.add_point(150, 0.7).unwrap();

        assert!(curve.move_point(id, 120, 0.6).is_ok());

        // Should still be 3 points
        assert_eq!(curve.points.len(), 3);

        // Find moved point
        let moved = curve.points.iter().find(|p| p.id == id).unwrap();
        assert_eq!(moved.time, 120);
        assert_eq!(moved.value, 0.6);
    }

    #[test]
    fn test_automation_curve_move_point_reorders() {
        let mut curve = AutomationCurve::new();

        let id1 = curve.add_point(50, 0.3).unwrap();
        let id2 = curve.add_point(100, 0.5).unwrap();

        // Move first point after second
        curve.move_point(id1, 150, 0.7).unwrap();

        // Should be reordered by time
        assert_eq!(curve.points[0].id, id2);
        assert_eq!(curve.points[1].id, id1);
    }

    #[test]
    fn test_automation_curve_get_value_at_exact() {
        let mut curve = AutomationCurve::new();
        curve.add_point(100, 0.5).unwrap();

        assert_eq!(curve.get_value_at(100), Some(0.5));
    }

    #[test]
    fn test_automation_curve_get_value_at_before_first() {
        let mut curve = AutomationCurve::new();
        curve.add_point(100, 0.5).unwrap();

        assert_eq!(curve.get_value_at(50), Some(0.5));
    }

    #[test]
    fn test_automation_curve_get_value_at_after_last() {
        let mut curve = AutomationCurve::new();
        curve.add_point(100, 0.5).unwrap();

        assert_eq!(curve.get_value_at(150), Some(0.5));
    }

    #[test]
    fn test_automation_curve_get_value_at_linear() {
        let mut curve = AutomationCurve::new();
        curve.curve_type = CurveType::Linear;
        curve.add_point(0, 0.0).unwrap();
        curve.add_point(100, 1.0).unwrap();

        // Midpoint should be 0.5
        let value = curve.get_value_at(50).unwrap();
        assert!((value - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_automation_curve_get_value_at_step() {
        let mut curve = AutomationCurve::new();
        curve.curve_type = CurveType::Step;
        curve.add_point(0, 0.3).unwrap();
        curve.add_point(100, 0.8).unwrap();

        // Should hold first value until next point
        assert_eq!(curve.get_value_at(50), Some(0.3));
        assert_eq!(curve.get_value_at(99), Some(0.3));
        assert_eq!(curve.get_value_at(100), Some(0.8));
    }

    #[test]
    fn test_automation_curve_get_value_at_empty() {
        let curve = AutomationCurve::new();
        assert_eq!(curve.get_value_at(100), None);
    }

    #[test]
    fn test_automation_curve_get_points_in_range() {
        let mut curve = AutomationCurve::new();
        curve.add_point(50, 0.3).unwrap();
        curve.add_point(100, 0.5).unwrap();
        curve.add_point(150, 0.7).unwrap();
        curve.add_point(200, 0.9).unwrap();

        let points = curve.get_points_in_range(75, 175);
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].time, 100);
        assert_eq!(points[1].time, 150);
    }

    #[test]
    fn test_automation_curve_clear() {
        let mut curve = AutomationCurve::new();
        curve.add_point(100, 0.5).unwrap();
        curve.add_point(200, 0.7).unwrap();

        curve.clear();
        assert_eq!(curve.points.len(), 0);
    }

    #[test]
    fn test_automation_curve_point_count() {
        let mut curve = AutomationCurve::new();
        assert_eq!(curve.point_count(), 0);

        curve.add_point(100, 0.5).unwrap();
        assert_eq!(curve.point_count(), 1);
    }

    #[test]
    fn test_automation_lane_new() {
        let lane = AutomationLane::new(1, 10, ParameterType::Volume);
        assert_eq!(lane.id, 1);
        assert_eq!(lane.track_id, 10);
        assert_eq!(lane.parameter_type, ParameterType::Volume);
        assert!(lane.enabled);
        assert!(lane.name.is_none());
    }

    #[test]
    fn test_automation_lane_display_name() {
        let mut lane = AutomationLane::new(1, 10, ParameterType::Volume);
        assert_eq!(lane.display_name(), "Volume");

        lane.name = Some("My Volume".to_string());
        assert_eq!(lane.display_name(), "My Volume");
    }

    #[test]
    fn test_automation_lane_color() {
        let lane = AutomationLane::new(1, 10, ParameterType::Volume);
        assert_eq!(lane.color(), "#4ade80");
    }

    #[test]
    fn test_automation_track_new() {
        let track = AutomationTrack::new(10);
        assert_eq!(track.track_id, 10);
        assert_eq!(track.lanes.len(), 0);
    }

    #[test]
    fn test_automation_track_add_lane() {
        let mut track = AutomationTrack::new(10);

        let id = track.add_lane(ParameterType::Volume).unwrap();
        assert_eq!(id, 1);
        assert_eq!(track.lanes.len(), 1);

        // Duplicate should fail
        assert!(track.add_lane(ParameterType::Volume).is_err());
    }

    #[test]
    fn test_automation_track_remove_lane() {
        let mut track = AutomationTrack::new(10);
        track.add_lane(ParameterType::Volume).unwrap();

        assert!(track.remove_lane(ParameterType::Volume).is_ok());
        assert_eq!(track.lanes.len(), 0);

        assert!(track.remove_lane(ParameterType::Pan).is_err());
    }

    #[test]
    fn test_automation_track_get_lane() {
        let mut track = AutomationTrack::new(10);
        track.add_lane(ParameterType::Volume).unwrap();

        let lane = track.get_lane(ParameterType::Volume);
        assert!(lane.is_some());
        assert_eq!(lane.unwrap().parameter_type, ParameterType::Volume);

        let lane = track.get_lane(ParameterType::Pan);
        assert!(lane.is_none());
    }

    #[test]
    fn test_automation_track_get_all_lanes() {
        let mut track = AutomationTrack::new(10);
        track.add_lane(ParameterType::Volume).unwrap();
        track.add_lane(ParameterType::Pan).unwrap();

        let lanes = track.get_all_lanes();
        assert_eq!(lanes.len(), 2);
    }

    #[test]
    fn test_automation_track_lane_count() {
        let mut track = AutomationTrack::new(10);
        assert_eq!(track.lane_count(), 0);

        track.add_lane(ParameterType::Volume).unwrap();
        assert_eq!(track.lane_count(), 1);
    }

    #[test]
    fn test_automation_manager_new() {
        let manager = AutomationManager::new();
        assert_eq!(manager.tracks.len(), 0);
    }

    #[test]
    fn test_automation_manager_create_lane() {
        let mut manager = AutomationManager::new();

        let id = manager.create_lane(10, ParameterType::Volume).unwrap();
        assert_eq!(id, 1);

        // Duplicate should fail
        assert!(manager.create_lane(10, ParameterType::Volume).is_err());
    }

    #[test]
    fn test_automation_manager_delete_lane() {
        let mut manager = AutomationManager::new();
        manager.create_lane(10, ParameterType::Volume).unwrap();

        assert!(manager.delete_lane(10, ParameterType::Volume).is_ok());
        assert!(manager.delete_lane(10, ParameterType::Pan).is_err());
    }

    #[test]
    fn test_automation_manager_add_point() {
        let mut manager = AutomationManager::new();
        manager.create_lane(10, ParameterType::Volume).unwrap();

        let point_id = manager.add_point(10, ParameterType::Volume, 100, 0.5).unwrap();
        assert_eq!(point_id, 1);

        // Non-existent lane should fail
        assert!(manager.add_point(10, ParameterType::Pan, 100, 0.5).is_err());
    }

    #[test]
    fn test_automation_manager_remove_point() {
        let mut manager = AutomationManager::new();
        manager.create_lane(10, ParameterType::Volume).unwrap();
        let point_id = manager.add_point(10, ParameterType::Volume, 100, 0.5).unwrap();

        assert!(manager.remove_point(10, ParameterType::Volume, point_id).is_ok());
        assert!(manager.remove_point(10, ParameterType::Volume, 999).is_err());
    }

    #[test]
    fn test_automation_manager_move_point() {
        let mut manager = AutomationManager::new();
        manager.create_lane(10, ParameterType::Volume).unwrap();
        let point_id = manager.add_point(10, ParameterType::Volume, 100, 0.5).unwrap();

        assert!(manager.move_point(10, ParameterType::Volume, point_id, 150, 0.7).is_ok());
    }

    #[test]
    fn test_automation_manager_set_curve_type() {
        let mut manager = AutomationManager::new();
        manager.create_lane(10, ParameterType::Volume).unwrap();

        assert!(manager.set_curve_type(10, ParameterType::Volume, CurveType::Bezier).is_ok());
    }

    #[test]
    fn test_automation_manager_get_lane() {
        let mut manager = AutomationManager::new();
        manager.create_lane(10, ParameterType::Volume).unwrap();

        let lane = manager.get_lane(10, ParameterType::Volume);
        assert!(lane.is_ok());
        assert_eq!(lane.unwrap().parameter_type, ParameterType::Volume);

        assert!(manager.get_lane(10, ParameterType::Pan).is_err());
    }

    #[test]
    fn test_automation_manager_get_track_lanes() {
        let mut manager = AutomationManager::new();
        manager.create_lane(10, ParameterType::Volume).unwrap();
        manager.create_lane(10, ParameterType::Pan).unwrap();

        let lanes = manager.get_track_lanes(10);
        assert_eq!(lanes.len(), 2);

        let lanes = manager.get_track_lanes(999);
        assert_eq!(lanes.len(), 0);
    }

    #[test]
    fn test_automation_manager_get_value_at() {
        let mut manager = AutomationManager::new();
        manager.create_lane(10, ParameterType::Volume).unwrap();
        manager.add_point(10, ParameterType::Volume, 100, 0.5).unwrap();

        let value = manager.get_value_at(10, ParameterType::Volume, 100);
        assert_eq!(value, Some(0.5));

        let value = manager.get_value_at(10, ParameterType::Pan, 100);
        assert_eq!(value, None);
    }

    #[test]
    fn test_automation_manager_remove_track() {
        let mut manager = AutomationManager::new();
        manager.create_lane(10, ParameterType::Volume).unwrap();

        manager.remove_track(10);
        assert!(manager.get_lane(10, ParameterType::Volume).is_err());
    }

    #[test]
    fn test_automation_manager_clear_all() {
        let mut manager = AutomationManager::new();
        manager.create_lane(10, ParameterType::Volume).unwrap();
        manager.create_lane(20, ParameterType::Pan).unwrap();

        manager.clear_all();
        assert_eq!(manager.tracks.len(), 0);
    }

    #[test]
    fn test_bezier_interpolation() {
        let mut curve = AutomationCurve::new();
        curve.curve_type = CurveType::Bezier;
        curve.add_point(0, 0.0).unwrap();
        curve.add_point(100, 1.0).unwrap();

        // Bezier should be smoother than linear (slower at edges, faster in middle)
        let value_25 = curve.get_value_at(25).unwrap();
        let value_50 = curve.get_value_at(50).unwrap();
        let value_75 = curve.get_value_at(75).unwrap();

        // At 25%, bezier should be less than linear (0.25)
        assert!(value_25 < 0.27);
        // At 50%, should be close to 0.5
        assert!((value_50 - 0.5).abs() < 0.05);
        // At 75%, bezier should be more than linear (0.75)
        assert!(value_75 > 0.73);
    }

    #[test]
    fn test_exponential_interpolation_rising() {
        let mut curve = AutomationCurve::new();
        curve.curve_type = CurveType::Exponential;
        curve.add_point(0, 0.0).unwrap();
        curve.add_point(100, 1.0).unwrap();

        // Exponential rising: slow start, fast end
        let value_25 = curve.get_value_at(25).unwrap();
        let value_75 = curve.get_value_at(75).unwrap();

        // At 25%, should be less than linear (0.25)
        assert!(value_25 < 0.1);
        // At 75%, should be more than linear (0.75)
        assert!(value_75 > 0.5);
    }

    #[test]
    fn test_exponential_interpolation_falling() {
        let mut curve = AutomationCurve::new();
        curve.curve_type = CurveType::Exponential;
        curve.add_point(0, 1.0).unwrap();
        curve.add_point(100, 0.0).unwrap();

        // Exponential falling: fast start, slow end
        // Using t_exp = 1.0 - (1.0 - t) * (1.0 - t)
        // At t=0.25: t_exp = 1.0 - 0.75*0.75 = 0.4375
        // value = 1.0 + (0.0 - 1.0) * 0.4375 = 0.5625
        let value_25 = curve.get_value_at(25).unwrap();
        let value_75 = curve.get_value_at(75).unwrap();

        // At 25%, value should be less than linear (0.75) due to fast decay
        assert!(value_25 < 0.75, "value_25={} should be < 0.75", value_25);
        // At 75%, should be less than linear (0.25) - approaching target slowly
        assert!(value_75 < 0.25, "value_75={} should be < 0.25", value_75);
    }

    #[test]
    fn test_multiple_cc_lanes() {
        let mut manager = AutomationManager::new();

        manager.create_lane(10, ParameterType::CC(1)).unwrap();
        manager.create_lane(10, ParameterType::CC(7)).unwrap();
        manager.create_lane(10, ParameterType::CC(64)).unwrap();

        let lanes = manager.get_track_lanes(10);
        assert_eq!(lanes.len(), 3);
    }

    #[test]
    fn test_curve_interpolation_edge_cases() {
        let mut curve = AutomationCurve::new();
        curve.add_point(100, 0.5).unwrap();

        // Same time as point
        assert_eq!(curve.get_value_at(100), Some(0.5));

        // Before first point
        assert_eq!(curve.get_value_at(0), Some(0.5));

        // After last point
        assert_eq!(curve.get_value_at(200), Some(0.5));
    }

    #[test]
    fn test_point_sorting_with_many_points() {
        let mut curve = AutomationCurve::new();

        // Add points in random order
        curve.add_point(500, 0.5).unwrap();
        curve.add_point(100, 0.1).unwrap();
        curve.add_point(300, 0.3).unwrap();
        curve.add_point(200, 0.2).unwrap();
        curve.add_point(400, 0.4).unwrap();

        // Verify sorted order
        for i in 0..curve.points.len() - 1 {
            assert!(curve.points[i].time <= curve.points[i + 1].time);
        }
    }

    #[test]
    fn test_value_clamping_throughout_pipeline() {
        let mut manager = AutomationManager::new();
        manager.create_lane(10, ParameterType::Volume).unwrap();

        // Try to add out-of-range point
        manager.add_point(10, ParameterType::Volume, 100, 2.0).unwrap();

        let value = manager.get_value_at(10, ParameterType::Volume, 100);
        assert_eq!(value, Some(1.0)); // Should be clamped

        // Try to move to out-of-range
        let point_id = 1;
        manager.move_point(10, ParameterType::Volume, point_id, 200, -1.0).unwrap();

        let value = manager.get_value_at(10, ParameterType::Volume, 200);
        assert_eq!(value, Some(0.0)); // Should be clamped
    }

    #[test]
    fn test_automation_mode() {
        let mut manager = AutomationManager::new();
        manager.create_lane(1, ParameterType::Volume).unwrap();

        // Default mode should be Read
        let mode = manager.get_automation_mode(1, ParameterType::Volume).unwrap();
        assert_eq!(mode, AutomationMode::Read);

        // Test setting different modes
        manager.set_automation_mode(1, ParameterType::Volume, AutomationMode::Write).unwrap();
        let mode = manager.get_automation_mode(1, ParameterType::Volume).unwrap();
        assert_eq!(mode, AutomationMode::Write);

        manager.set_automation_mode(1, ParameterType::Volume, AutomationMode::Off).unwrap();
        let mode = manager.get_automation_mode(1, ParameterType::Volume).unwrap();
        assert_eq!(mode, AutomationMode::Off);

        manager.set_automation_mode(1, ParameterType::Volume, AutomationMode::Latch).unwrap();
        let mode = manager.get_automation_mode(1, ParameterType::Volume).unwrap();
        assert_eq!(mode, AutomationMode::Latch);

        manager.set_automation_mode(1, ParameterType::Volume, AutomationMode::Touch).unwrap();
        let mode = manager.get_automation_mode(1, ParameterType::Volume).unwrap();
        assert_eq!(mode, AutomationMode::Touch);
    }

    #[test]
    fn test_send_parameter_type() {
        let mut manager = AutomationManager::new();

        // Test Send parameter types
        manager.create_lane(1, ParameterType::Send(0)).unwrap();
        manager.create_lane(1, ParameterType::Send(1)).unwrap();
        manager.create_lane(1, ParameterType::Send(2)).unwrap();

        // Verify they are distinct
        let lanes = manager.get_track_lanes(1);
        assert_eq!(lanes.len(), 3);

        // Test display names
        assert_eq!(ParameterType::Send(0).as_string(), "Send 0");
        assert_eq!(ParameterType::Send(1).as_string(), "Send 1");

        // Test colors
        assert_eq!(ParameterType::Send(0).color(), "#f97316");
    }

    #[test]
    fn test_effect_param_type() {
        let mut manager = AutomationManager::new();

        // Test EffectParam parameter types
        manager.create_lane(1, ParameterType::EffectParam { effect_id: 0, param_id: 0 }).unwrap();
        manager.create_lane(1, ParameterType::EffectParam { effect_id: 0, param_id: 1 }).unwrap();
        manager.create_lane(1, ParameterType::EffectParam { effect_id: 1, param_id: 0 }).unwrap();

        // Verify they are distinct
        let lanes = manager.get_track_lanes(1);
        assert_eq!(lanes.len(), 3);

        // Test display names
        assert_eq!(
            ParameterType::EffectParam { effect_id: 0, param_id: 5 }.as_string(),
            "Effect 0 Param 5"
        );

        // Test colors
        assert_eq!(ParameterType::EffectParam { effect_id: 0, param_id: 0 }.color(), "#ec4899");
    }

    #[test]
    fn test_gain_parameter_type() {
        let mut manager = AutomationManager::new();

        // Test both Volume and Gain (they're aliases)
        manager.create_lane(1, ParameterType::Volume).unwrap();
        manager.create_lane(1, ParameterType::Gain).unwrap();

        // Verify they are distinct parameters
        let lanes = manager.get_track_lanes(1);
        assert_eq!(lanes.len(), 2);

        // Test display names
        assert_eq!(ParameterType::Volume.as_string(), "Volume");
        assert_eq!(ParameterType::Gain.as_string(), "Gain");

        // Both should have same color (green)
        assert_eq!(ParameterType::Volume.color(), "#4ade80");
        assert_eq!(ParameterType::Gain.color(), "#4ade80");
    }

    #[test]
    fn test_automation_with_new_parameter_types() {
        let mut manager = AutomationManager::new();

        // Create lanes for all new parameter types
        manager.create_lane(1, ParameterType::Gain).unwrap();
        manager.create_lane(1, ParameterType::Send(0)).unwrap();
        manager.create_lane(1, ParameterType::EffectParam { effect_id: 0, param_id: 0 }).unwrap();

        // Add points
        manager.add_point(1, ParameterType::Gain, 0, 0.0).unwrap();
        manager.add_point(1, ParameterType::Gain, 100, 1.0).unwrap();

        manager.add_point(1, ParameterType::Send(0), 0, 0.5).unwrap();
        manager.add_point(1, ParameterType::Send(0), 100, 0.8).unwrap();

        manager.add_point(1, ParameterType::EffectParam { effect_id: 0, param_id: 0 }, 0, 0.3).unwrap();
        manager.add_point(1, ParameterType::EffectParam { effect_id: 0, param_id: 0 }, 100, 0.7).unwrap();

        // Verify interpolation works
        let gain_value = manager.get_value_at(1, ParameterType::Gain, 50).unwrap();
        assert!((gain_value - 0.5).abs() < 0.01);

        let send_value = manager.get_value_at(1, ParameterType::Send(0), 50).unwrap();
        assert!((send_value - 0.65).abs() < 0.01);

        let effect_value = manager.get_value_at(1, ParameterType::EffectParam { effect_id: 0, param_id: 0 }, 50).unwrap();
        assert!((effect_value - 0.5).abs() < 0.01);
    }
}
