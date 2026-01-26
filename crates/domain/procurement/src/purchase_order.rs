//! Purchase order domain models and logic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ProcurementError;

/// Purchase order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PurchaseOrderStatus {
    /// Purchase order is draft.
    Draft,
    /// Purchase order is pending approval.
    Pending,
    /// Purchase order is approved.
    Approved,
    /// Purchase order is sent to supplier.
    Sent,
    /// Purchase order is partially received.
    PartiallyReceived,
    /// Purchase order is fully received.
    Received,
    /// Purchase order is cancelled.
    Cancelled,
}

/// Purchase order line item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderLine {
    /// Line item ID.
    pub id: Uuid,
    /// SKU.
    pub sku: String,
    /// Quantity ordered.
    pub quantity: u32,
    /// Unit price.
    pub unit_price: f64,
    /// Quantity received.
    pub quantity_received: u32,
}

impl PurchaseOrderLine {
    /// Create a new purchase order line.
    pub fn new(sku: String, quantity: u32, unit_price: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            sku,
            quantity,
            unit_price,
            quantity_received: 0,
        }
    }

    /// Get the line total.
    pub fn line_total(&self) -> f64 {
        self.quantity as f64 * self.unit_price
    }

    /// Record receipt of items.
    pub fn record_receipt(&mut self, quantity: u32) -> Result<(), ProcurementError> {
        if self.quantity_received + quantity > self.quantity {
            return Err(ProcurementError::InvalidPurchaseOrderLine(
                format!("Cannot receive more than ordered. Ordered: {}, Already received: {}, Attempting to receive: {}", 
                    self.quantity, self.quantity_received, quantity)
            ));
        }
        self.quantity_received += quantity;
        Ok(())
    }

    /// Check if line is fully received.
    pub fn is_fully_received(&self) -> bool {
        self.quantity_received >= self.quantity
    }
}

/// Purchase order entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrder {
    /// Unique purchase order ID.
    pub id: Uuid,
    /// Purchase order number (human-readable).
    pub po_number: String,
    /// Supplier ID.
    pub supplier_id: Uuid,
    /// Current status.
    pub status: PurchaseOrderStatus,
    /// Line items.
    pub lines: Vec<PurchaseOrderLine>,
    /// Expected delivery date (optional).
    pub expected_delivery: Option<DateTime<Utc>>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl PurchaseOrder {
    /// Create a new purchase order.
    pub fn new(po_number: String, supplier_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            po_number,
            supplier_id,
            status: PurchaseOrderStatus::Draft,
            lines: Vec::new(),
            expected_delivery: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Add a line item to the purchase order.
    pub fn add_line(&mut self, sku: String, quantity: u32, unit_price: f64) -> Result<(), ProcurementError> {
        if self.status != PurchaseOrderStatus::Draft {
            return Err(ProcurementError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: "add_line".to_string(),
            });
        }
        let line = PurchaseOrderLine::new(sku, quantity, unit_price);
        self.lines.push(line);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Calculate total amount.
    pub fn total_amount(&self) -> f64 {
        self.lines.iter().map(|line| line.line_total()).sum()
    }

    /// Submit purchase order for approval.
    pub fn submit(&mut self) -> Result<(), ProcurementError> {
        if self.status != PurchaseOrderStatus::Draft {
            return Err(ProcurementError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: format!("{:?}", PurchaseOrderStatus::Pending),
            });
        }
        if self.lines.is_empty() {
            return Err(ProcurementError::InvalidPurchaseOrderLine(
                "Cannot submit purchase order with no line items".to_string(),
            ));
        }
        self.status = PurchaseOrderStatus::Pending;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Approve purchase order.
    pub fn approve(&mut self) -> Result<(), ProcurementError> {
        if self.status != PurchaseOrderStatus::Pending {
            return Err(ProcurementError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: format!("{:?}", PurchaseOrderStatus::Approved),
            });
        }
        self.status = PurchaseOrderStatus::Approved;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Send purchase order to supplier.
    pub fn send(&mut self) -> Result<(), ProcurementError> {
        if self.status != PurchaseOrderStatus::Approved {
            return Err(ProcurementError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: format!("{:?}", PurchaseOrderStatus::Sent),
            });
        }
        self.status = PurchaseOrderStatus::Sent;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Record receipt of items for a line.
    pub fn record_receipt(&mut self, line_id: Uuid, quantity: u32) -> Result<(), ProcurementError> {
        if self.status != PurchaseOrderStatus::Sent && self.status != PurchaseOrderStatus::PartiallyReceived {
            return Err(ProcurementError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: "record_receipt".to_string(),
            });
        }
        let line = self.lines.iter_mut()
            .find(|l| l.id == line_id)
            .ok_or_else(|| ProcurementError::InvalidPurchaseOrderLine(
                format!("Line item not found: {}", line_id)
            ))?;
        
        line.record_receipt(quantity)?;
        
        // Update PO status based on receipt
        let all_received = self.lines.iter().all(|l| l.is_fully_received());
        if all_received {
            self.status = PurchaseOrderStatus::Received;
        } else {
            self.status = PurchaseOrderStatus::PartiallyReceived;
        }
        
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Cancel purchase order.
    pub fn cancel(&mut self) -> Result<(), ProcurementError> {
        if self.status == PurchaseOrderStatus::Received || self.status == PurchaseOrderStatus::Cancelled {
            return Err(ProcurementError::InvalidStateTransition {
                from: format!("{:?}", self.status),
                to: format!("{:?}", PurchaseOrderStatus::Cancelled),
            });
        }
        self.status = PurchaseOrderStatus::Cancelled;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_purchase_order() {
        let supplier_id = Uuid::new_v4();
        let po = PurchaseOrder::new("PO-2026-001".to_string(), supplier_id);

        assert_eq!(po.po_number, "PO-2026-001");
        assert_eq!(po.supplier_id, supplier_id);
        assert_eq!(po.status, PurchaseOrderStatus::Draft);
        assert!(po.lines.is_empty());
    }

    #[test]
    fn test_add_line() {
        let supplier_id = Uuid::new_v4();
        let mut po = PurchaseOrder::new("PO-2026-002".to_string(), supplier_id);

        po.add_line("SKU-001".to_string(), 10, 100.0).unwrap();
        assert_eq!(po.lines.len(), 1);
        assert_eq!(po.total_amount(), 1000.0);
    }

    #[test]
    fn test_cannot_add_line_after_submit() {
        let supplier_id = Uuid::new_v4();
        let mut po = PurchaseOrder::new("PO-2026-003".to_string(), supplier_id);
        po.add_line("SKU-001".to_string(), 10, 100.0).unwrap();
        po.submit().unwrap();

        let result = po.add_line("SKU-002".to_string(), 5, 50.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_submit_requires_lines() {
        let supplier_id = Uuid::new_v4();
        let mut po = PurchaseOrder::new("PO-2026-004".to_string(), supplier_id);

        let result = po.submit();
        assert!(result.is_err());
    }

    #[test]
    fn test_po_workflow() {
        let supplier_id = Uuid::new_v4();
        let mut po = PurchaseOrder::new("PO-2026-005".to_string(), supplier_id);
        po.add_line("SKU-001".to_string(), 10, 100.0).unwrap();
        
        po.submit().unwrap();
        assert_eq!(po.status, PurchaseOrderStatus::Pending);

        po.approve().unwrap();
        assert_eq!(po.status, PurchaseOrderStatus::Approved);

        po.send().unwrap();
        assert_eq!(po.status, PurchaseOrderStatus::Sent);
    }

    #[test]
    fn test_record_receipt() {
        let supplier_id = Uuid::new_v4();
        let mut po = PurchaseOrder::new("PO-2026-006".to_string(), supplier_id);
        po.add_line("SKU-001".to_string(), 10, 100.0).unwrap();
        po.submit().unwrap();
        po.approve().unwrap();
        po.send().unwrap();

        let line_id = po.lines[0].id;
        po.record_receipt(line_id, 5).unwrap();
        assert_eq!(po.lines[0].quantity_received, 5);
        assert_eq!(po.status, PurchaseOrderStatus::PartiallyReceived);

        po.record_receipt(line_id, 5).unwrap();
        assert_eq!(po.lines[0].quantity_received, 10);
        assert_eq!(po.status, PurchaseOrderStatus::Received);
    }

    #[test]
    fn test_cannot_receive_more_than_ordered() {
        let supplier_id = Uuid::new_v4();
        let mut po = PurchaseOrder::new("PO-2026-007".to_string(), supplier_id);
        po.add_line("SKU-001".to_string(), 10, 100.0).unwrap();
        po.submit().unwrap();
        po.approve().unwrap();
        po.send().unwrap();

        let line_id = po.lines[0].id;
        let result = po.record_receipt(line_id, 11);
        assert!(result.is_err());
    }

    #[test]
    fn test_purchase_order_serialization() {
        let supplier_id = Uuid::new_v4();
        let mut po = PurchaseOrder::new("PO-2026-008".to_string(), supplier_id);
        po.add_line("SKU-001".to_string(), 10, 100.0).unwrap();

        let json = serde_json::to_string(&po).unwrap();
        let deserialized: PurchaseOrder = serde_json::from_str(&json).unwrap();

        assert_eq!(po.po_number, deserialized.po_number);
        assert_eq!(po.status, deserialized.status);
        assert_eq!(po.lines.len(), deserialized.lines.len());
    }
}
