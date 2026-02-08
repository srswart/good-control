//! Shipping staging workflow domain models and logic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{WarehouseError, Result};

/// Ship task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipTaskStatus {
    /// Ship task is pending (not yet started).
    Pending,
    /// Ship task is in staging area.
    Staging,
    /// Ship task is undergoing compliance check.
    ComplianceCheck,
    /// Ship task is ready to dispatch.
    ReadyToDispatch,
    /// Ship task has been dispatched.
    Dispatched,
    /// Ship task is blocked by compliance issues.
    Blocked,
    /// Ship task is cancelled.
    Cancelled,
}

/// Compliance block category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceBlockCategory {
    /// Missing required document.
    MissingDocument,
    /// Document has expired.
    ExpiredDocument,
    /// Missing required approval.
    MissingApproval,
    /// Approval has expired.
    ExpiredApproval,
    /// Item restriction violation.
    ItemRestriction,
    /// Item is in quarantine.
    QuarantineBlock,
    /// Battery compliance violation.
    BatteryCompliance,
}

/// Block reason detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockReason {
    /// Block category.
    pub category: ComplianceBlockCategory,
    /// Human-readable message.
    pub message: String,
    /// Additional details (key-value pairs).
    pub details: HashMap<String, String>,
}

impl BlockReason {
    /// Create a new block reason.
    pub fn new(category: ComplianceBlockCategory, message: String) -> Self {
        Self {
            category,
            message,
            details: HashMap::new(),
        }
    }

    /// Add a detail to the block reason.
    pub fn with_detail(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }
}

/// Compliance block result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceBlock {
    /// Operation being evaluated.
    pub operation: String,
    /// Whether the operation is blocked.
    pub blocked: bool,
    /// List of block reasons (if blocked).
    pub reasons: Vec<BlockReason>,
    /// Required actions to resolve blocks.
    pub required_actions: Vec<String>,
}

impl ComplianceBlock {
    /// Create a new compliance block result (not blocked).
    pub fn allowed(operation: String) -> Self {
        Self {
            operation,
            blocked: false,
            reasons: Vec::new(),
            required_actions: Vec::new(),
        }
    }

    /// Create a new compliance block result (blocked).
    pub fn blocked(operation: String, reasons: Vec<BlockReason>, required_actions: Vec<String>) -> Self {
        Self {
            operation,
            blocked: true,
            reasons,
            required_actions,
        }
    }
}

/// Ship line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipLine {
    /// Line ID.
    pub id: Uuid,
    /// Pack task ID reference.
    pub pack_task_id: Uuid,
    /// Package IDs from pack task.
    pub package_ids: Vec<Uuid>,
    /// Order ID.
    pub order_id: Uuid,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
}

impl ShipLine {
    /// Create a new ship line.
    pub fn new(pack_task_id: Uuid, package_ids: Vec<Uuid>, order_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            pack_task_id,
            package_ids,
            order_id,
            created_at: Utc::now(),
        }
    }
}

/// Ship task entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipTask {
    /// Task ID.
    pub id: Uuid,
    /// Task number (human-readable).
    pub task_number: String,
    /// Current status.
    pub status: ShipTaskStatus,
    /// Ship lines.
    pub lines: Vec<ShipLine>,
    /// Compliance check result (if evaluated).
    pub compliance_check: Option<ComplianceBlock>,
    /// Carrier name (assigned on dispatch).
    pub carrier: Option<String>,
    /// Tracking number (assigned on dispatch).
    pub tracking_number: Option<String>,
    /// Dispatch timestamp (when dispatched).
    pub dispatched_at: Option<DateTime<Utc>>,
    /// Assigned operator.
    pub assigned_to: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl ShipTask {
    /// Create a new ship task.
    pub fn new(task_number: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            task_number,
            status: ShipTaskStatus::Pending,
            lines: Vec::new(),
            compliance_check: None,
            carrier: None,
            tracking_number: None,
            dispatched_at: None,
            assigned_to: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a ship line.
    pub fn add_line(&mut self, pack_task_id: Uuid, package_ids: Vec<Uuid>, order_id: Uuid) -> Result<()> {
        if self.status != ShipTaskStatus::Pending {
            return Err(WarehouseError::InvalidStateTransition(
                format!("Cannot add lines to ship task in status {:?}", self.status)
            ));
        }

        let line = ShipLine::new(pack_task_id, package_ids, order_id);
        self.lines.push(line);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Start the ship task (move to staging).
    pub fn start(&mut self, operator: String) -> Result<()> {
        if self.status != ShipTaskStatus::Pending {
            return Err(WarehouseError::InvalidStateTransition(
                format!("Cannot start ship task in status {:?}", self.status)
            ));
        }

        if self.lines.is_empty() {
            return Err(WarehouseError::InvalidQuantity(
                "Cannot start ship task with no lines".to_string()
            ));
        }

        self.status = ShipTaskStatus::Staging;
        self.assigned_to = Some(operator);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Evaluate compliance gates.
    /// This is a simplified implementation - in production, this would call the compliance domain.
    pub fn evaluate_compliance(&mut self) -> Result<ComplianceBlock> {
        if self.status != ShipTaskStatus::Staging {
            return Err(WarehouseError::InvalidStateTransition(
                format!("Cannot evaluate compliance for ship task in status {:?}", self.status)
            ));
        }

        // Simplified compliance check - in production, this would integrate with compliance domain
        // For now, we'll just check if there are any lines (basic validation)
        let result = if self.lines.is_empty() {
            let reason = BlockReason::new(
                ComplianceBlockCategory::ItemRestriction,
                "No packages to ship".to_string(),
            );
            ComplianceBlock::blocked(
                "ship_dispatch".to_string(),
                vec![reason],
                vec!["Add packages to ship task".to_string()],
            )
        } else {
            ComplianceBlock::allowed("ship_dispatch".to_string())
        };

        self.compliance_check = Some(result.clone());

        if result.blocked {
            self.status = ShipTaskStatus::Blocked;
        } else {
            self.status = ShipTaskStatus::ReadyToDispatch;
        }

        self.updated_at = Utc::now();
        Ok(result)
    }

    /// Dispatch the shipment.
    pub fn dispatch(&mut self, carrier: String, tracking_number: String) -> Result<()> {
        if self.status != ShipTaskStatus::ReadyToDispatch {
            return Err(WarehouseError::InvalidStateTransition(
                format!("Cannot dispatch ship task in status {:?}", self.status)
            ));
        }

        self.status = ShipTaskStatus::Dispatched;
        self.carrier = Some(carrier);
        self.tracking_number = Some(tracking_number);
        self.dispatched_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Block the ship task with compliance reasons.
    pub fn block(&mut self, compliance_block: ComplianceBlock) -> Result<()> {
        if !compliance_block.blocked {
            return Err(WarehouseError::DiscrepancyValidationFailed(
                "Cannot block with a non-blocking compliance result".to_string()
            ));
        }

        self.status = ShipTaskStatus::Blocked;
        self.compliance_check = Some(compliance_block);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Cancel the ship task.
    pub fn cancel(&mut self) -> Result<()> {
        if self.status == ShipTaskStatus::Dispatched {
            return Err(WarehouseError::InvalidStateTransition(
                "Cannot cancel dispatched ship task".to_string()
            ));
        }

        self.status = ShipTaskStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Get task statistics.
    pub fn statistics(&self) -> ShipTaskStatistics {
        ShipTaskStatistics {
            total_lines: self.lines.len(),
            total_packages: self.lines.iter().map(|l| l.package_ids.len()).sum(),
            is_blocked: self.status == ShipTaskStatus::Blocked,
            block_count: self.compliance_check.as_ref()
                .map(|c| c.reasons.len())
                .unwrap_or(0),
        }
    }
}

/// Ship task statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipTaskStatistics {
    /// Total number of lines.
    pub total_lines: usize,
    /// Total number of packages.
    pub total_packages: usize,
    /// Whether the task is blocked.
    pub is_blocked: bool,
    /// Number of block reasons.
    pub block_count: usize,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ship_task() {
        let task = ShipTask::new("SHIP-001".to_string());

        assert_eq!(task.task_number, "SHIP-001");
        assert_eq!(task.status, ShipTaskStatus::Pending);
        assert!(task.lines.is_empty());
        assert!(task.compliance_check.is_none());
        assert!(task.carrier.is_none());
        assert!(task.tracking_number.is_none());
        assert!(task.dispatched_at.is_none());
        assert!(task.assigned_to.is_none());
    }

    #[test]
    fn test_add_line() {
        let mut task = ShipTask::new("SHIP-002".to_string());
        let pack_task_id = Uuid::new_v4();
        let package_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        let order_id = Uuid::new_v4();

        let result = task.add_line(pack_task_id, package_ids.clone(), order_id);
        assert!(result.is_ok());
        assert_eq!(task.lines.len(), 1);
        assert_eq!(task.lines[0].pack_task_id, pack_task_id);
        assert_eq!(task.lines[0].package_ids, package_ids);
        assert_eq!(task.lines[0].order_id, order_id);
    }

    #[test]
    fn test_cannot_add_line_after_start() {
        let mut task = ShipTask::new("SHIP-003".to_string());
        let pack_task_id = Uuid::new_v4();
        let package_ids = vec![Uuid::new_v4()];
        let order_id = Uuid::new_v4();

        task.add_line(pack_task_id, package_ids.clone(), order_id).unwrap();
        task.start("operator1".to_string()).unwrap();

        let result = task.add_line(pack_task_id, package_ids, order_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_start_ship_task() {
        let mut task = ShipTask::new("SHIP-004".to_string());
        let pack_task_id = Uuid::new_v4();
        let package_ids = vec![Uuid::new_v4()];
        let order_id = Uuid::new_v4();

        task.add_line(pack_task_id, package_ids, order_id).unwrap();

        let result = task.start("operator1".to_string());
        assert!(result.is_ok());
        assert_eq!(task.status, ShipTaskStatus::Staging);
        assert_eq!(task.assigned_to, Some("operator1".to_string()));
    }

    #[test]
    fn test_cannot_start_without_lines() {
        let mut task = ShipTask::new("SHIP-005".to_string());

        let result = task.start("operator1".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_compliance_pass() {
        let mut task = ShipTask::new("SHIP-006".to_string());
        let pack_task_id = Uuid::new_v4();
        let package_ids = vec![Uuid::new_v4()];
        let order_id = Uuid::new_v4();

        task.add_line(pack_task_id, package_ids, order_id).unwrap();
        task.start("operator1".to_string()).unwrap();

        let result = task.evaluate_compliance();
        assert!(result.is_ok());

        let compliance_block = result.unwrap();
        assert!(!compliance_block.blocked);
        assert_eq!(task.status, ShipTaskStatus::ReadyToDispatch);
    }

    #[test]
    fn test_evaluate_compliance_blocked() {
        let mut task = ShipTask::new("SHIP-007".to_string());
        // Start without adding lines (will fail compliance)
        task.status = ShipTaskStatus::Staging; // Force status for test

        let result = task.evaluate_compliance();
        assert!(result.is_ok());

        let compliance_block = result.unwrap();
        assert!(compliance_block.blocked);
        assert_eq!(task.status, ShipTaskStatus::Blocked);
        assert!(!compliance_block.reasons.is_empty());
        assert!(!compliance_block.required_actions.is_empty());
    }

    #[test]
    fn test_dispatch() {
        let mut task = ShipTask::new("SHIP-008".to_string());
        let pack_task_id = Uuid::new_v4();
        let package_ids = vec![Uuid::new_v4()];
        let order_id = Uuid::new_v4();

        task.add_line(pack_task_id, package_ids, order_id).unwrap();
        task.start("operator1".to_string()).unwrap();
        task.evaluate_compliance().unwrap();

        let result = task.dispatch("DHL".to_string(), "1234567890".to_string());
        assert!(result.is_ok());
        assert_eq!(task.status, ShipTaskStatus::Dispatched);
        assert_eq!(task.carrier, Some("DHL".to_string()));
        assert_eq!(task.tracking_number, Some("1234567890".to_string()));
        assert!(task.dispatched_at.is_some());
    }

    #[test]
    fn test_cannot_dispatch_if_not_ready() {
        let mut task = ShipTask::new("SHIP-009".to_string());

        let result = task.dispatch("DHL".to_string(), "1234567890".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_block_ship_task() {
        let mut task = ShipTask::new("SHIP-010".to_string());

        let reason = BlockReason::new(
            ComplianceBlockCategory::MissingDocument,
            "Missing COO certificate".to_string(),
        ).with_detail("document_type".to_string(), "COO".to_string());

        let compliance_block = ComplianceBlock::blocked(
            "ship_dispatch".to_string(),
            vec![reason],
            vec!["Upload COO certificate".to_string()],
        );

        let result = task.block(compliance_block);
        assert!(result.is_ok());
        assert_eq!(task.status, ShipTaskStatus::Blocked);
        assert!(task.compliance_check.is_some());

        let check = task.compliance_check.unwrap();
        assert!(check.blocked);
        assert_eq!(check.reasons.len(), 1);
        assert_eq!(check.required_actions.len(), 1);
    }

    #[test]
    fn test_cancel_ship_task() {
        let mut task = ShipTask::new("SHIP-011".to_string());

        let result = task.cancel();
        assert!(result.is_ok());
        assert_eq!(task.status, ShipTaskStatus::Cancelled);
    }

    #[test]
    fn test_cannot_cancel_dispatched_task() {
        let mut task = ShipTask::new("SHIP-012".to_string());
        task.status = ShipTaskStatus::Dispatched; // Force status for test

        let result = task.cancel();
        assert!(result.is_err());
    }

    #[test]
    fn test_statistics() {
        let mut task = ShipTask::new("SHIP-013".to_string());
        let pack_task_id = Uuid::new_v4();
        let package_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let order_id = Uuid::new_v4();

        task.add_line(pack_task_id, package_ids, order_id).unwrap();

        let stats = task.statistics();
        assert_eq!(stats.total_lines, 1);
        assert_eq!(stats.total_packages, 3);
        assert!(!stats.is_blocked);
        assert_eq!(stats.block_count, 0);
    }

    #[test]
    fn test_complete_workflow() {
        let mut task = ShipTask::new("SHIP-014".to_string());
        let pack_task_id = Uuid::new_v4();
        let package_ids = vec![Uuid::new_v4()];
        let order_id = Uuid::new_v4();

        // Add line
        task.add_line(pack_task_id, package_ids, order_id).unwrap();
        assert_eq!(task.status, ShipTaskStatus::Pending);

        // Start task
        task.start("operator1".to_string()).unwrap();
        assert_eq!(task.status, ShipTaskStatus::Staging);

        // Evaluate compliance
        let compliance_result = task.evaluate_compliance().unwrap();
        assert!(!compliance_result.blocked);
        assert_eq!(task.status, ShipTaskStatus::ReadyToDispatch);

        // Dispatch
        task.dispatch("FedEx".to_string(), "TRACK123".to_string()).unwrap();
        assert_eq!(task.status, ShipTaskStatus::Dispatched);
        assert_eq!(task.carrier, Some("FedEx".to_string()));
        assert_eq!(task.tracking_number, Some("TRACK123".to_string()));
    }
}
