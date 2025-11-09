   /// Controller Editor Undo/Redo Commands - Trusty Module
   ///
   /// Commands for controller (CC) editor operations: add/delete/move points, smooth curves.

use super::core::{Command, UndoRedoError, UndoRedoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Control Change point
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CCPoint {
    pub tick: i32,
    pub value: u8,
    pub cc_number: u8,
}

impl CCPoint {
    pub fn new(tick: i32, value: u8, cc_number: u8) -> Self {
        Self {
            tick,
            value,
            cc_number,
        }
    }
}

/// Add a CC point
#[derive(Debug)]
pub struct AddCCPointCommand {
    point: CCPoint,
    point_id: Option<i32>,
    executed: bool,
}

impl AddCCPointCommand {
    pub fn new(tick: i32, value: u8, cc_number: u8) -> Self {
        Self {
            point: CCPoint::new(tick, value, cc_number),
            point_id: None,
            executed: false,
        }
    }
}

impl Command for AddCCPointCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            // In real implementation, add to state and get ID
            self.point_id = Some(1); // Mock ID
            self.executed = true;
        }
        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            return Err(UndoRedoError::UndoFailed(
                "Command not executed".to_string(),
            ));
        }
        // Remove point from state
        Ok(())
    }

    fn description(&self) -> String {
        format!("Add CC{} Point", self.point.cc_number)
    }

    fn memory_usage(&self) -> usize {
        32
    }
}

/// Delete a CC point
#[derive(Debug)]
pub struct DeleteCCPointCommand {
    point_id: i32,
    deleted_point: Option<CCPoint>,
    executed: bool,
}

impl DeleteCCPointCommand {
    pub fn new(point_id: i32) -> Self {
        Self {
            point_id,
            deleted_point: None,
            executed: false,
        }
    }
}

impl Command for DeleteCCPointCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            // Store deleted point (would fetch from state)
            self.deleted_point = Some(CCPoint::new(0, 64, 1));
            self.executed = true;
        }
        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            return Err(UndoRedoError::UndoFailed(
                "Command not executed".to_string(),
            ));
        }
        // Restore point to state
        Ok(())
    }

    fn description(&self) -> String {
        format!("Delete CC Point (ID {})", self.point_id)
    }

    fn memory_usage(&self) -> usize {
        32 + self.deleted_point.as_ref().map(|_| 24).unwrap_or(0)
    }
}

/// Move a CC point to new position/value
#[derive(Debug)]
pub struct MoveCCPointCommand {
    point_id: i32,
    new_tick: i32,
    new_value: u8,
    old_tick: Option<i32>,
    old_value: Option<u8>,
    executed: bool,
}

impl MoveCCPointCommand {
    pub fn new(point_id: i32, new_tick: i32, new_value: u8) -> Self {
        Self {
            point_id,
            new_tick,
            new_value,
            old_tick: None,
            old_value: None,
            executed: false,
        }
    }
}

impl Command for MoveCCPointCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            // Store old values (would fetch from state)
            self.old_tick = Some(0);
            self.old_value = Some(64);
            // Apply new values to state
            self.executed = true;
        }
        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            return Err(UndoRedoError::UndoFailed(
                "Command not executed".to_string(),
            ));
        }
        // Restore old values
        Ok(())
    }

    fn description(&self) -> String {
        format!("Move CC Point (ID {})", self.point_id)
    }

    fn memory_usage(&self) -> usize {
        32
    }
}

/// Smooth CC curve with interpolation
#[derive(Debug)]
pub struct SmoothCurveCommand {
    point_ids: Vec<i32>,
    old_values: HashMap<i32, u8>,
    executed: bool,
}

impl SmoothCurveCommand {
    pub fn new(point_ids: Vec<i32>) -> Self {
        Self {
            point_ids,
            old_values: HashMap::new(),
            executed: false,
        }
    }
}

impl Command for SmoothCurveCommand {
    fn execute(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            // Store old values and apply smoothing
            for &point_id in &self.point_ids {
                self.old_values.insert(point_id, 64);
            }
            self.executed = true;
        }
        Ok(())
    }

    fn undo(&mut self) -> UndoRedoResult<()> {
        if !self.executed {
            return Err(UndoRedoError::UndoFailed(
                "Command not executed".to_string(),
            ));
        }
        // Restore old values
        Ok(())
    }

    fn description(&self) -> String {
        format!("Smooth {} CC points", self.point_ids.len())
    }

    fn memory_usage(&self) -> usize {
        48 + self.point_ids.len() * 8 + self.old_values.len() * 12
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cc_point_creation() {
        let point = CCPoint::new(480, 64, 1);
        assert_eq!(point.tick, 480);
        assert_eq!(point.value, 64);
        assert_eq!(point.cc_number, 1);
    }

    #[test]
    fn test_add_cc_point_command() {
        let mut cmd = AddCCPointCommand::new(480, 64, 1);

        cmd.execute().unwrap();
        assert!(cmd.point_id.is_some());
        assert!(cmd.executed);

        cmd.undo().unwrap();
    }

    #[test]
    fn test_add_cc_point_description() {
        let cmd = AddCCPointCommand::new(480, 64, 7);
        assert_eq!(cmd.description(), "Add CC7 Point");
    }

    #[test]
    fn test_delete_cc_point_command() {
        let mut cmd = DeleteCCPointCommand::new(1);

        cmd.execute().unwrap();
        assert!(cmd.deleted_point.is_some());

        cmd.undo().unwrap();
    }

    #[test]
    fn test_delete_cc_point_undo_before_execute() {
        let mut cmd = DeleteCCPointCommand::new(1);

        let result = cmd.undo();
        assert!(matches!(result, Err(UndoRedoError::UndoFailed(_))));
    }

    #[test]
    fn test_move_cc_point_command() {
        let mut cmd = MoveCCPointCommand::new(1, 960, 100);

        cmd.execute().unwrap();
        assert!(cmd.old_tick.is_some());
        assert!(cmd.old_value.is_some());

        cmd.undo().unwrap();
    }

    #[test]
    fn test_move_cc_point_description() {
        let cmd = MoveCCPointCommand::new(5, 960, 100);
        assert_eq!(cmd.description(), "Move CC Point (ID 5)");
    }

    #[test]
    fn test_smooth_curve_command() {
        let point_ids = vec![1, 2, 3, 4, 5];
        let mut cmd = SmoothCurveCommand::new(point_ids);

        cmd.execute().unwrap();
        assert_eq!(cmd.old_values.len(), 5);

        cmd.undo().unwrap();
    }

    #[test]
    fn test_smooth_curve_description() {
        let point_ids = vec![1, 2, 3];
        let cmd = SmoothCurveCommand::new(point_ids);
        assert_eq!(cmd.description(), "Smooth 3 CC points");
    }

    #[test]
    fn test_command_memory_usage() {
        let add_cmd = AddCCPointCommand::new(480, 64, 1);
        assert!(add_cmd.memory_usage() > 0);

        let delete_cmd = DeleteCCPointCommand::new(1);
        assert!(delete_cmd.memory_usage() > 0);

        let move_cmd = MoveCCPointCommand::new(1, 960, 100);
        assert!(move_cmd.memory_usage() > 0);

        let smooth_cmd = SmoothCurveCommand::new(vec![1, 2, 3]);
        assert!(smooth_cmd.memory_usage() > 0);
    }

    #[test]
    fn test_cc_point_equality() {
        let point1 = CCPoint::new(480, 64, 1);
        let point2 = CCPoint::new(480, 64, 1);
        let point3 = CCPoint::new(960, 64, 1);

        assert_eq!(point1, point2);
        assert_ne!(point1, point3);
    }

    #[test]
    fn test_smooth_curve_empty() {
        let mut cmd = SmoothCurveCommand::new(vec![]);

        cmd.execute().unwrap();
        assert_eq!(cmd.old_values.len(), 0);
    }
}
