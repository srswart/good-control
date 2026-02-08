//! Receiving workflow domain models and logic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{WarehouseError, Result};
use procurement::{PurchaseOrder, PurchaseOrderLine};

/// Receiving task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceivingTaskStatus {
    /// Receiving task is pending (not yet started).
    Pending,
    /// Receiving task is in progress.
    InProgress,
    /// Receiving task is completed.
    Completed,
    /// Receiving task is cancelled.
    Cancelled,
}

/// Discrepancy type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscrepancyType {
    /// Over-receipt: received more than expected.
    OverReceipt,
    /// Under-receipt: received less than expected.
    UnderReceipt,
    /// Short-receipt: received nothing when something was expected.
    ShortReceipt,
}

/// Receiving discrepancy record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivingDiscrepancy {
    /// Discrepancy ID.
    pub id: Uuid,
    /// Receiving line ID this discrepancy applies to.
    pub receiving_line_id: Uuid,
    /// Type of discrepancy.
    pub discrepancy_type: DiscrepancyType,
    /// Expected quantity.
    pub expected_quantity: u32,
    /// Received quantity.
    pub received_quantity: u32,
    /// Discrepancy amount (absolute difference).
    pub discrepancy_amount: u32,
    /// Notes about the discrepancy.
    pub notes: Option<String>,
    /// Whether discrepancy has been acknowledged/approved.
    pub acknowledged: bool,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
}

impl ReceivingDiscrepancy {
    /// Create a new discrepancy.
    pub fn new(
        receiving_line_id: Uuid,
        discrepancy_type: DiscrepancyType,
        expected_quantity: u32,
        received_quantity: u32,
        notes: Option<String>,
    ) -> Self {
        let discrepancy_amount = match discrepancy_type {
            DiscrepancyType::OverReceipt => received_quantity.saturating_sub(expected_quantity),
            DiscrepancyType::UnderReceipt | DiscrepancyType::ShortReceipt => {
                expected_quantity.saturating_sub(received_quantity)
            }
        };

        Self {
            id: Uuid::new_v4(),
            receiving_line_id,
            discrepancy_type,
            expected_quantity,
            received_quantity,
            discrepancy_amount,
            notes,
            acknowledged: false,
            created_at: Utc::now(),
        }
    }

    /// Acknowledge the discrepancy.
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }
}

/// QC outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QCOutcome {
    /// QC passed.
    Pass,
    /// QC failed.
    Fail,
    /// Item requires rework.
    Rework,
}

/// QC result record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QCResult {
    /// QC result ID.
    pub id: Uuid,
    /// Receiving line ID this QC result applies to.
    pub receiving_line_id: Uuid,
    /// QC outcome.
    pub outcome: QCOutcome,
    /// QC inspector name.
    pub inspector: String,
    /// QC notes.
    pub notes: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
}

impl QCResult {
    /// Create a new QC result.
    pub fn new(
        receiving_line_id: Uuid,
        outcome: QCOutcome,
        inspector: String,
        notes: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            receiving_line_id,
            outcome,
            inspector,
            notes,
            created_at: Utc::now(),
        }
    }
}

/// Quarantine status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineStatus {
    /// Not in quarantine.
    None,
    /// In quarantine.
    Quarantined,
    /// Released from quarantine to inventory.
    Released,
    /// Rejected (removed from receiving, no inventory update).
    Rejected,
}

/// Receiving line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivingLine {
    /// Receiving line ID.
    pub id: Uuid,
    /// Purchase order line ID (reference).
    pub po_line_id: Uuid,
    /// SKU.
    pub sku: String,
    /// Expected quantity.
    pub expected_quantity: u32,
    /// Received quantity.
    pub received_quantity: u32,
    /// Discrepancies for this line.
    pub discrepancies: Vec<ReceivingDiscrepancy>,
    /// QC result for this line.
    pub qc_result: Option<QCResult>,
    /// Quarantine status.
    pub quarantine_status: QuarantineStatus,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl ReceivingLine {
    /// Create a new receiving line from a purchase order line.
    pub fn from_po_line(po_line: &PurchaseOrderLine) -> Self {
        Self {
            id: Uuid::new_v4(),
            po_line_id: po_line.id,
            sku: po_line.sku.clone(),
            expected_quantity: po_line.quantity,
            received_quantity: 0,
            discrepancies: Vec::new(),
            qc_result: None,
            quarantine_status: QuarantineStatus::None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Record receipt of items.
    /// 
    /// Note: Quantity can be 0 to record short-receipt (nothing received when something was expected).
    pub fn record_receipt(&mut self, quantity: u32) -> Result<()> {
        // Allow 0 quantity for short-receipt scenarios
        self.received_quantity += quantity;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if line has discrepancies.
    pub fn has_discrepancies(&self) -> bool {
        !self.discrepancies.is_empty()
    }

    /// Check if all discrepancies are acknowledged.
    pub fn all_discrepancies_acknowledged(&self) -> bool {
        self.discrepancies.iter().all(|d| d.acknowledged)
    }

    /// Check if line has QC result.
    pub fn has_qc_result(&self) -> bool {
        self.qc_result.is_some()
    }

    /// Check if line is ready for completion (has QC result and all discrepancies acknowledged).
    pub fn is_ready_for_completion(&self) -> bool {
        self.has_qc_result() && (!self.has_discrepancies() || self.all_discrepancies_acknowledged())
    }
}

/// Receiving task entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivingTask {
    /// Receiving task ID.
    pub id: Uuid,
    /// Receiving task number (human-readable).
    pub task_number: String,
    /// Purchase order ID (reference).
    pub po_id: Uuid,
    /// Purchase order number (for display).
    pub po_number: String,
    /// Current status.
    pub status: ReceivingTaskStatus,
    /// Receiving lines.
    pub lines: Vec<ReceivingLine>,
    /// Assigned to (operator name).
    pub assigned_to: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl ReceivingTask {
    /// Create a new receiving task from a purchase order.
    pub fn from_purchase_order(po: &PurchaseOrder, task_number: String) -> Result<Self> {
        if po.lines.is_empty() {
            return Err(WarehouseError::InvalidReceivingLine(
                "Purchase order has no lines".to_string(),
            ));
        }

        let lines: Vec<ReceivingLine> = po
            .lines
            .iter()
            .map(|po_line| ReceivingLine::from_po_line(po_line))
            .collect();

        Ok(Self {
            id: Uuid::new_v4(),
            task_number,
            po_id: po.id,
            po_number: po.po_number.clone(),
            status: ReceivingTaskStatus::Pending,
            lines,
            assigned_to: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Start the receiving task.
    pub fn start(&mut self) -> Result<()> {
        match self.status {
            ReceivingTaskStatus::Pending => {
                self.status = ReceivingTaskStatus::InProgress;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(WarehouseError::InvalidStateTransition(format!(
                "Cannot start task from status: {:?}",
                self.status
            ))),
        }
    }

    /// Complete the receiving task.
    pub fn complete(&mut self) -> Result<()> {
        match self.status {
            ReceivingTaskStatus::InProgress => {
                // Validate all lines are ready for completion
                for line in &self.lines {
                    if !line.is_ready_for_completion() {
                        return Err(WarehouseError::QCOutcomeRequired);
                    }
                }

                self.status = ReceivingTaskStatus::Completed;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(WarehouseError::InvalidStateTransition(format!(
                "Cannot complete task from status: {:?}",
                self.status
            ))),
        }
    }

    /// Cancel the receiving task.
    pub fn cancel(&mut self) -> Result<()> {
        match self.status {
            ReceivingTaskStatus::Pending | ReceivingTaskStatus::InProgress => {
                self.status = ReceivingTaskStatus::Cancelled;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(WarehouseError::InvalidStateTransition(format!(
                "Cannot cancel task from status: {:?}",
                self.status
            ))),
        }
    }

    /// Record receipt for a line.
    pub fn record_receipt(&mut self, line_id: Uuid, quantity: u32) -> Result<()> {
        if self.status != ReceivingTaskStatus::InProgress {
            return Err(WarehouseError::InvalidStateTransition(
                "Can only record receipt when task is in progress".to_string(),
            ));
        }

        let line = self
            .lines
            .iter_mut()
            .find(|l| l.id == line_id)
            .ok_or_else(|| WarehouseError::InvalidReceivingLine(format!(
                "Line {} not found",
                line_id
            )))?;

        line.record_receipt(quantity)?;

        // Check for discrepancies
        if line.received_quantity != line.expected_quantity {
            let discrepancy_type = if line.received_quantity == 0 {
                DiscrepancyType::ShortReceipt
            } else if line.received_quantity > line.expected_quantity {
                DiscrepancyType::OverReceipt
            } else {
                DiscrepancyType::UnderReceipt
            };

            let discrepancy = ReceivingDiscrepancy::new(
                line.id,
                discrepancy_type,
                line.expected_quantity,
                line.received_quantity,
                None,
            );
            line.discrepancies.push(discrepancy);
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Record QC result for a line.
    pub fn record_qc_result(
        &mut self,
        line_id: Uuid,
        outcome: QCOutcome,
        inspector: String,
        notes: Option<String>,
    ) -> Result<()> {
        if self.status != ReceivingTaskStatus::InProgress {
            return Err(WarehouseError::InvalidStateTransition(
                "Can only record QC result when task is in progress".to_string(),
            ));
        }

        let line = self
            .lines
            .iter_mut()
            .find(|l| l.id == line_id)
            .ok_or_else(|| WarehouseError::InvalidReceivingLine(format!(
                "Line {} not found",
                line_id
            )))?;

        let qc_result = QCResult::new(line.id, outcome, inspector, notes);
        line.qc_result = Some(qc_result);

        // If QC failed, automatically place in quarantine
        if outcome == QCOutcome::Fail {
            line.quarantine_status = QuarantineStatus::Quarantined;
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    /// Place a line in quarantine.
    pub fn place_in_quarantine(&mut self, line_id: Uuid) -> Result<()> {
        if self.status != ReceivingTaskStatus::InProgress {
            return Err(WarehouseError::InvalidStateTransition(
                "Can only place in quarantine when task is in progress".to_string(),
            ));
        }

        let line = self
            .lines
            .iter_mut()
            .find(|l| l.id == line_id)
            .ok_or_else(|| WarehouseError::InvalidReceivingLine(format!(
                "Line {} not found",
                line_id
            )))?;

        if line.quarantine_status != QuarantineStatus::None {
            return Err(WarehouseError::QuarantineOperationFailed(format!(
                "Line is already in quarantine status: {:?}",
                line.quarantine_status
            )));
        }

        line.quarantine_status = QuarantineStatus::Quarantined;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Release a line from quarantine to inventory.
    pub fn release_from_quarantine(&mut self, line_id: Uuid) -> Result<()> {
        if self.status != ReceivingTaskStatus::InProgress {
            return Err(WarehouseError::InvalidStateTransition(
                "Can only release from quarantine when task is in progress".to_string(),
            ));
        }

        let line = self
            .lines
            .iter_mut()
            .find(|l| l.id == line_id)
            .ok_or_else(|| WarehouseError::InvalidReceivingLine(format!(
                "Line {} not found",
                line_id
            )))?;

        if line.quarantine_status != QuarantineStatus::Quarantined {
            return Err(WarehouseError::QuarantineOperationFailed(format!(
                "Line is not in quarantine, current status: {:?}",
                line.quarantine_status
            )));
        }

        line.quarantine_status = QuarantineStatus::Released;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Reject a line (remove from receiving, no inventory update).
    pub fn reject_line(&mut self, line_id: Uuid) -> Result<()> {
        if self.status != ReceivingTaskStatus::InProgress {
            return Err(WarehouseError::InvalidStateTransition(
                "Can only reject line when task is in progress".to_string(),
            ));
        }

        let line = self
            .lines
            .iter_mut()
            .find(|l| l.id == line_id)
            .ok_or_else(|| WarehouseError::InvalidReceivingLine(format!(
                "Line {} not found",
                line_id
            )))?;

        if line.quarantine_status != QuarantineStatus::Quarantined {
            return Err(WarehouseError::QuarantineOperationFailed(
                "Can only reject lines that are in quarantine".to_string(),
            ));
        }

        line.quarantine_status = QuarantineStatus::Rejected;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Acknowledge a discrepancy.
    pub fn acknowledge_discrepancy(&mut self, line_id: Uuid, discrepancy_id: Uuid) -> Result<()> {
        let line = self
            .lines
            .iter_mut()
            .find(|l| l.id == line_id)
            .ok_or_else(|| WarehouseError::InvalidReceivingLine(format!(
                "Line {} not found",
                line_id
            )))?;

        let discrepancy = line
            .discrepancies
            .iter_mut()
            .find(|d| d.id == discrepancy_id)
            .ok_or_else(|| WarehouseError::DiscrepancyValidationFailed(format!(
                "Discrepancy {} not found",
                discrepancy_id
            )))?;

        discrepancy.acknowledge();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if task is ready for completion.
    pub fn is_ready_for_completion(&self) -> bool {
        self.status == ReceivingTaskStatus::InProgress
            && self.lines.iter().all(|l| l.is_ready_for_completion())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use procurement::PurchaseOrder;

    fn create_test_po() -> PurchaseOrder {
        let mut po = PurchaseOrder::new(
            "PO-2026-001".to_string(),
            Uuid::new_v4(),
        );
        po.lines.push(PurchaseOrderLine::new(
            "SKU-001".to_string(),
            10,
            100.0,
        ));
        po.lines.push(PurchaseOrderLine::new(
            "SKU-002".to_string(),
            5,
            50.0,
        ));
        po
    }

    #[test]
    fn test_receiving_task_creation() {
        let po = create_test_po();
        let task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();

        assert_eq!(task.status, ReceivingTaskStatus::Pending);
        assert_eq!(task.lines.len(), 2);
        assert_eq!(task.lines[0].expected_quantity, 10);
        assert_eq!(task.lines[1].expected_quantity, 5);
    }

    #[test]
    fn test_receiving_task_start() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();

        assert_eq!(task.status, ReceivingTaskStatus::Pending);
        task.start().unwrap();
        assert_eq!(task.status, ReceivingTaskStatus::InProgress);
    }

    #[test]
    fn test_receiving_task_start_invalid_state() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.status = ReceivingTaskStatus::Completed;

        assert!(task.start().is_err());
    }

    #[test]
    fn test_record_receipt() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        let line_id = task.lines[0].id;
        task.record_receipt(line_id, 10).unwrap();

        assert_eq!(task.lines[0].received_quantity, 10);
        assert!(!task.lines[0].has_discrepancies());
    }

    #[test]
    fn test_record_receipt_with_discrepancy() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        let line_id = task.lines[0].id;
        task.record_receipt(line_id, 8).unwrap(); // Under-receipt

        assert_eq!(task.lines[0].received_quantity, 8);
        assert!(task.lines[0].has_discrepancies());
        assert_eq!(task.lines[0].discrepancies[0].discrepancy_type, DiscrepancyType::UnderReceipt);
        assert_eq!(task.lines[0].discrepancies[0].discrepancy_amount, 2);
    }

    #[test]
    fn test_record_receipt_over_receipt() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        let line_id = task.lines[0].id;
        task.record_receipt(line_id, 12).unwrap(); // Over-receipt

        assert_eq!(task.lines[0].received_quantity, 12);
        assert!(task.lines[0].has_discrepancies());
        assert_eq!(task.lines[0].discrepancies[0].discrepancy_type, DiscrepancyType::OverReceipt);
        assert_eq!(task.lines[0].discrepancies[0].discrepancy_amount, 2);
    }

    #[test]
    fn test_record_receipt_short_receipt() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        let line_id = task.lines[0].id;
        // Record 0 quantity to indicate short-receipt (nothing received)
        task.record_receipt(line_id, 0).unwrap();

        assert_eq!(task.lines[0].received_quantity, 0);
        assert!(task.lines[0].has_discrepancies());
        assert_eq!(task.lines[0].discrepancies[0].discrepancy_type, DiscrepancyType::ShortReceipt);
        assert_eq!(task.lines[0].discrepancies[0].discrepancy_amount, 10);
    }

    #[test]
    fn test_record_qc_result() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        let line_id = task.lines[0].id;
        task.record_receipt(line_id, 10).unwrap();
        task.record_qc_result(line_id, QCOutcome::Pass, "Inspector A".to_string(), None).unwrap();

        assert!(task.lines[0].has_qc_result());
        assert_eq!(task.lines[0].qc_result.as_ref().unwrap().outcome, QCOutcome::Pass);
    }

    #[test]
    fn test_qc_fail_auto_quarantine() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        let line_id = task.lines[0].id;
        task.record_receipt(line_id, 10).unwrap();
        task.record_qc_result(line_id, QCOutcome::Fail, "Inspector A".to_string(), None).unwrap();

        assert_eq!(task.lines[0].quarantine_status, QuarantineStatus::Quarantined);
    }

    #[test]
    fn test_quarantine_release() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        let line_id = task.lines[0].id;
        task.place_in_quarantine(line_id).unwrap();
        assert_eq!(task.lines[0].quarantine_status, QuarantineStatus::Quarantined);

        task.release_from_quarantine(line_id).unwrap();
        assert_eq!(task.lines[0].quarantine_status, QuarantineStatus::Released);
    }

    #[test]
    fn test_quarantine_reject() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        let line_id = task.lines[0].id;
        task.place_in_quarantine(line_id).unwrap();
        task.reject_line(line_id).unwrap();

        assert_eq!(task.lines[0].quarantine_status, QuarantineStatus::Rejected);
    }

    #[test]
    fn test_acknowledge_discrepancy() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        let line_id = task.lines[0].id;
        task.record_receipt(line_id, 8).unwrap(); // Creates discrepancy

        let discrepancy_id = task.lines[0].discrepancies[0].id;
        assert!(!task.lines[0].discrepancies[0].acknowledged);

        task.acknowledge_discrepancy(line_id, discrepancy_id).unwrap();
        assert!(task.lines[0].discrepancies[0].acknowledged);
    }

    #[test]
    fn test_complete_task_without_qc() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        let line_id = task.lines[0].id;
        task.record_receipt(line_id, 10).unwrap();

        // Cannot complete without QC result
        assert!(task.complete().is_err());
    }

    #[test]
    fn test_complete_task_with_unacknowledged_discrepancy() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        let line_id = task.lines[0].id;
        task.record_receipt(line_id, 8).unwrap(); // Creates discrepancy
        task.record_qc_result(line_id, QCOutcome::Pass, "Inspector A".to_string(), None).unwrap();

        // Cannot complete with unacknowledged discrepancy
        assert!(task.complete().is_err());
    }

    #[test]
    fn test_complete_task_success() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        // Complete first line
        let line_id_1 = task.lines[0].id;
        task.record_receipt(line_id_1, 10).unwrap();
        task.record_qc_result(line_id_1, QCOutcome::Pass, "Inspector A".to_string(), None).unwrap();

        // Complete second line
        let line_id_2 = task.lines[1].id;
        task.record_receipt(line_id_2, 5).unwrap();
        task.record_qc_result(line_id_2, QCOutcome::Pass, "Inspector A".to_string(), None).unwrap();

        task.complete().unwrap();
        assert_eq!(task.status, ReceivingTaskStatus::Completed);
    }

    #[test]
    fn test_cancel_task() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        task.cancel().unwrap();
        assert_eq!(task.status, ReceivingTaskStatus::Cancelled);
    }

    #[test]
    fn test_cancel_completed_task_fails() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        // Complete all lines
        let line_id_1 = task.lines[0].id;
        task.record_receipt(line_id_1, 10).unwrap();
        task.record_qc_result(line_id_1, QCOutcome::Pass, "Inspector A".to_string(), None).unwrap();

        let line_id_2 = task.lines[1].id;
        task.record_receipt(line_id_2, 5).unwrap();
        task.record_qc_result(line_id_2, QCOutcome::Pass, "Inspector A".to_string(), None).unwrap();
        
        task.complete().unwrap();

        // Cannot cancel completed task
        assert!(task.cancel().is_err());
    }

    #[test]
    fn test_is_ready_for_completion() {
        let po = create_test_po();
        let mut task = ReceivingTask::from_purchase_order(&po, "RECV-001".to_string()).unwrap();
        task.start().unwrap();

        // Not ready - no QC results
        assert!(!task.is_ready_for_completion());

        // Add QC results
        let line_id_1 = task.lines[0].id;
        task.record_receipt(line_id_1, 10).unwrap();
        task.record_qc_result(line_id_1, QCOutcome::Pass, "Inspector A".to_string(), None).unwrap();

        let line_id_2 = task.lines[1].id;
        task.record_receipt(line_id_2, 5).unwrap();
        task.record_qc_result(line_id_2, QCOutcome::Pass, "Inspector A".to_string(), None).unwrap();

        // Now ready
        assert!(task.is_ready_for_completion());
    }
}
