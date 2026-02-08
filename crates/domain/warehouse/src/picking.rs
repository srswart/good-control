//! Picking workflow domain models and logic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{WarehouseError, Result};

/// Pick task type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickTaskType {
    /// Single order pick - pick all items for one order.
    SingleOrder,
    /// Wave pick - pick multiple orders in a single pass.
    Wave,
    /// Batch pick - pick multiple orders for same item simultaneously.
    Batch,
}

/// Pick task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickTaskStatus {
    /// Pick task is pending (not yet started).
    Pending,
    /// Pick task is in progress.
    InProgress,
    /// Pick task is completed.
    Completed,
    /// Pick task is cancelled.
    Cancelled,
}

/// Pick exception type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickExceptionType {
    /// Short pick - insufficient quantity available.
    ShortPick,
    /// Item damaged during pick.
    Damaged,
    /// Item not found at expected location.
    NotFound,
    /// Wrong item scanned.
    WrongItem,
}

/// Pick exception record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickException {
    /// Exception ID.
    pub id: Uuid,
    /// Pick line ID this exception applies to.
    pub pick_line_id: Uuid,
    /// Type of exception.
    pub exception_type: PickExceptionType,
    /// Expected quantity.
    pub expected_quantity: u32,
    /// Actually picked quantity.
    pub picked_quantity: u32,
    /// Operator notes about the exception.
    pub notes: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
}

impl PickException {
    /// Create a new pick exception.
    pub fn new(
        pick_line_id: Uuid,
        exception_type: PickExceptionType,
        expected_quantity: u32,
        picked_quantity: u32,
        notes: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            pick_line_id,
            exception_type,
            expected_quantity,
            picked_quantity,
            notes,
            created_at: Utc::now(),
        }
    }
}

/// Pick line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickLine {
    /// Line item ID.
    pub id: Uuid,
    /// Order line ID this pick is for.
    pub order_line_id: Uuid,
    /// SKU to pick.
    pub sku: String,
    /// Location to pick from.
    pub location_id: String,
    /// Quantity to pick.
    pub quantity_to_pick: u32,
    /// Quantity actually picked.
    pub quantity_picked: u32,
    /// Pick exceptions for this line.
    pub exceptions: Vec<PickException>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl PickLine {
    /// Create a new pick line.
    pub fn new(
        order_line_id: Uuid,
        sku: String,
        location_id: String,
        quantity_to_pick: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            order_line_id,
            sku,
            location_id,
            quantity_to_pick,
            quantity_picked: 0,
            exceptions: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Record a successful pick.
    pub fn record_pick(&mut self, quantity: u32) -> Result<()> {
        if quantity == 0 {
            return Err(WarehouseError::InvalidQuantity(
                "Pick quantity must be greater than zero".to_string(),
            ));
        }

        if self.quantity_picked + quantity > self.quantity_to_pick {
            return Err(WarehouseError::InvalidQuantity(format!(
                "Cannot pick more than required. Required: {}, Already picked: {}, Attempting: {}",
                self.quantity_to_pick, self.quantity_picked, quantity
            )));
        }

        self.quantity_picked += quantity;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Record a pick exception.
    pub fn record_exception(
        &mut self,
        exception_type: PickExceptionType,
        picked_quantity: u32,
        notes: Option<String>,
    ) -> Result<()> {
        let exception = PickException::new(
            self.id,
            exception_type,
            self.quantity_to_pick,
            picked_quantity,
            notes,
        );

        // Update picked quantity for partial picks
        if picked_quantity > 0 {
            self.quantity_picked += picked_quantity;
        }

        self.exceptions.push(exception);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if line is fully picked.
    pub fn is_fully_picked(&self) -> bool {
        self.quantity_picked >= self.quantity_to_pick
    }

    /// Check if line has exceptions.
    pub fn has_exceptions(&self) -> bool {
        !self.exceptions.is_empty()
    }
}

/// Pick task entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickTask {
    /// Unique pick task ID.
    pub id: Uuid,
    /// Task number (human-readable).
    pub task_number: String,
    /// Task type (single/wave/batch).
    pub task_type: PickTaskType,
    /// Order IDs included in this pick task.
    pub order_ids: Vec<Uuid>,
    /// Current status.
    pub status: PickTaskStatus,
    /// Pick lines.
    pub lines: Vec<PickLine>,
    /// Assigned operator.
    pub assigned_to: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl PickTask {
    /// Create a new pick task.
    pub fn new(
        task_number: String,
        task_type: PickTaskType,
        order_ids: Vec<Uuid>,
    ) -> Result<Self> {
        if order_ids.is_empty() {
            return Err(WarehouseError::InvalidStateTransition(
                "Pick task must have at least one order".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            task_number,
            task_type,
            order_ids,
            status: PickTaskStatus::Pending,
            lines: Vec::new(),
            assigned_to: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Add a pick line to the task.
    pub fn add_line(
        &mut self,
        order_line_id: Uuid,
        sku: String,
        location_id: String,
        quantity: u32,
    ) -> Result<()> {
        if self.status != PickTaskStatus::Pending {
            return Err(WarehouseError::InvalidStateTransition(
                "Can only add lines to pending pick tasks".to_string(),
            ));
        }

        let line = PickLine::new(order_line_id, sku, location_id, quantity);
        self.lines.push(line);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Start the pick task.
    pub fn start(&mut self, operator: String) -> Result<()> {
        if self.status != PickTaskStatus::Pending {
            return Err(WarehouseError::InvalidStateTransition(format!(
                "Cannot start pick task from status: {:?}",
                self.status
            )));
        }

        if self.lines.is_empty() {
            return Err(WarehouseError::InvalidStateTransition(
                "Cannot start pick task with no lines".to_string(),
            ));
        }

        self.status = PickTaskStatus::InProgress;
        self.assigned_to = Some(operator);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Record a pick for a specific line.
    pub fn record_pick(&mut self, line_id: Uuid, quantity: u32) -> Result<()> {
        if self.status != PickTaskStatus::InProgress {
            return Err(WarehouseError::InvalidStateTransition(
                "Can only record picks when task is in progress".to_string(),
            ));
        }

        let line = self
            .lines
            .iter_mut()
            .find(|l| l.id == line_id)
            .ok_or_else(|| {
                WarehouseError::InvalidStateTransition(format!("Line {} not found", line_id))
            })?;

        line.record_pick(quantity)?;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Record a pick exception for a specific line.
    pub fn record_exception(
        &mut self,
        line_id: Uuid,
        exception_type: PickExceptionType,
        picked_quantity: u32,
        notes: Option<String>,
    ) -> Result<()> {
        if self.status != PickTaskStatus::InProgress {
            return Err(WarehouseError::InvalidStateTransition(
                "Can only record exceptions when task is in progress".to_string(),
            ));
        }

        let line = self
            .lines
            .iter_mut()
            .find(|l| l.id == line_id)
            .ok_or_else(|| {
                WarehouseError::InvalidStateTransition(format!("Line {} not found", line_id))
            })?;

        line.record_exception(exception_type, picked_quantity, notes)?;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Complete the pick task.
    pub fn complete(&mut self) -> Result<()> {
        if self.status != PickTaskStatus::InProgress {
            return Err(WarehouseError::InvalidStateTransition(format!(
                "Cannot complete pick task from status: {:?}",
                self.status
            )));
        }

        // Allow completion even with exceptions (partial picks are valid)
        self.status = PickTaskStatus::Completed;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Cancel the pick task.
    pub fn cancel(&mut self) -> Result<()> {
        if self.status == PickTaskStatus::Completed {
            return Err(WarehouseError::InvalidStateTransition(
                "Cannot cancel completed pick task".to_string(),
            ));
        }

        self.status = PickTaskStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if task is ready for completion.
    pub fn is_ready_for_completion(&self) -> bool {
        self.status == PickTaskStatus::InProgress
            && !self.lines.is_empty()
            && self.lines.iter().any(|l| l.quantity_picked > 0)
    }

    /// Get total lines count.
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// Get completed lines count (fully picked).
    pub fn completed_lines(&self) -> usize {
        self.lines.iter().filter(|l| l.is_fully_picked()).count()
    }

    /// Get lines with exceptions count.
    pub fn exception_lines(&self) -> usize {
        self.lines.iter().filter(|l| l.has_exceptions()).count()
    }

    /// Get total quantity to pick.
    pub fn total_quantity_to_pick(&self) -> u32 {
        self.lines.iter().map(|l| l.quantity_to_pick).sum()
    }

    /// Get total quantity picked.
    pub fn total_quantity_picked(&self) -> u32 {
        self.lines.iter().map(|l| l.quantity_picked).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_task_creation() {
        let order_id = Uuid::new_v4();
        let task = PickTask::new(
            "PICK-001".to_string(),
            PickTaskType::SingleOrder,
            vec![order_id],
        )
        .unwrap();

        assert_eq!(task.status, PickTaskStatus::Pending);
        assert_eq!(task.task_type, PickTaskType::SingleOrder);
        assert_eq!(task.order_ids.len(), 1);
        assert!(task.lines.is_empty());
    }

    #[test]
    fn test_pick_task_requires_orders() {
        let result = PickTask::new(
            "PICK-001".to_string(),
            PickTaskType::SingleOrder,
            vec![],
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_add_pick_line() {
        let order_id = Uuid::new_v4();
        let order_line_id = Uuid::new_v4();
        let mut task = PickTask::new(
            "PICK-001".to_string(),
            PickTaskType::SingleOrder,
            vec![order_id],
        )
        .unwrap();

        task.add_line(
            order_line_id,
            "SKU-001".to_string(),
            "LOC-A1".to_string(),
            10,
        )
        .unwrap();

        assert_eq!(task.lines.len(), 1);
        assert_eq!(task.lines[0].sku, "SKU-001");
        assert_eq!(task.lines[0].quantity_to_pick, 10);
    }

    #[test]
    fn test_pick_workflow() {
        let order_id = Uuid::new_v4();
        let order_line_id = Uuid::new_v4();
        let mut task = PickTask::new(
            "PICK-001".to_string(),
            PickTaskType::SingleOrder,
            vec![order_id],
        )
        .unwrap();

        task.add_line(
            order_line_id,
            "SKU-001".to_string(),
            "LOC-A1".to_string(),
            10,
        )
        .unwrap();

        // Start task
        task.start("operator1".to_string()).unwrap();
        assert_eq!(task.status, PickTaskStatus::InProgress);
        assert_eq!(task.assigned_to, Some("operator1".to_string()));

        // Record pick
        let line_id = task.lines[0].id;
        task.record_pick(line_id, 10).unwrap();
        assert_eq!(task.lines[0].quantity_picked, 10);
        assert!(task.lines[0].is_fully_picked());

        // Complete task
        task.complete().unwrap();
        assert_eq!(task.status, PickTaskStatus::Completed);
    }

    #[test]
    fn test_partial_pick_with_exception() {
        let order_id = Uuid::new_v4();
        let order_line_id = Uuid::new_v4();
        let mut task = PickTask::new(
            "PICK-001".to_string(),
            PickTaskType::SingleOrder,
            vec![order_id],
        )
        .unwrap();

        task.add_line(
            order_line_id,
            "SKU-001".to_string(),
            "LOC-A1".to_string(),
            10,
        )
        .unwrap();

        task.start("operator1".to_string()).unwrap();

        // Record partial pick with short pick exception
        let line_id = task.lines[0].id;
        task.record_exception(
            line_id,
            PickExceptionType::ShortPick,
            7,
            Some("Only 7 units available".to_string()),
        )
        .unwrap();

        assert_eq!(task.lines[0].quantity_picked, 7);
        assert!(!task.lines[0].is_fully_picked());
        assert!(task.lines[0].has_exceptions());
        assert_eq!(task.lines[0].exceptions.len(), 1);
        assert_eq!(task.exception_lines(), 1);

        // Can still complete with partial pick
        assert!(task.is_ready_for_completion());
        task.complete().unwrap();
        assert_eq!(task.status, PickTaskStatus::Completed);
    }

    #[test]
    fn test_wave_pick_multiple_orders() {
        let order_id_1 = Uuid::new_v4();
        let order_id_2 = Uuid::new_v4();
        let mut task = PickTask::new(
            "PICK-WAVE-001".to_string(),
            PickTaskType::Wave,
            vec![order_id_1, order_id_2],
        )
        .unwrap();

        assert_eq!(task.task_type, PickTaskType::Wave);
        assert_eq!(task.order_ids.len(), 2);
    }

    #[test]
    fn test_pick_statistics() {
        let order_id = Uuid::new_v4();
        let mut task = PickTask::new(
            "PICK-001".to_string(),
            PickTaskType::SingleOrder,
            vec![order_id],
        )
        .unwrap();

        task.add_line(Uuid::new_v4(), "SKU-001".to_string(), "LOC-A1".to_string(), 10)
            .unwrap();
        task.add_line(Uuid::new_v4(), "SKU-002".to_string(), "LOC-A2".to_string(), 5)
            .unwrap();

        task.start("operator1".to_string()).unwrap();

        // Pick first line fully
        let line_id_1 = task.lines[0].id;
        task.record_pick(line_id_1, 10).unwrap();

        // Pick second line partially with exception
        let line_id_2 = task.lines[1].id;
        task.record_exception(line_id_2, PickExceptionType::ShortPick, 3, None)
            .unwrap();

        assert_eq!(task.total_lines(), 2);
        assert_eq!(task.completed_lines(), 1);
        assert_eq!(task.exception_lines(), 1);
        assert_eq!(task.total_quantity_to_pick(), 15);
        assert_eq!(task.total_quantity_picked(), 13);
    }
}


