//! Inbound document domain models and logic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of inbound document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InboundDocumentType {
    /// Purchase order.
    PurchaseOrder,
    /// Invoice.
    Invoice,
    /// Packing list.
    PackingList,
    /// Certificate of origin.
    CertificateOfOrigin,
    /// Customs declaration.
    CustomsDeclaration,
    /// Other document.
    Other,
}

/// Inbound document status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InboundDocumentStatus {
    /// Document is pending review.
    Pending,
    /// Document is approved.
    Approved,
    /// Document is rejected.
    Rejected,
    /// Document is archived.
    Archived,
}

/// Inbound document entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundDocument {
    /// Unique document ID.
    pub id: Uuid,
    /// Document reference number.
    pub reference: String,
    /// Document type.
    pub document_type: InboundDocumentType,
    /// Associated purchase order ID (optional).
    pub purchase_order_id: Option<Uuid>,
    /// Supplier ID.
    pub supplier_id: Uuid,
    /// Current status.
    pub status: InboundDocumentStatus,
    /// Document file path or storage reference (optional).
    pub file_path: Option<String>,
    /// Document metadata (optional).
    pub metadata: Option<serde_json::Value>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl InboundDocument {
    /// Create a new inbound document.
    pub fn new(
        reference: String,
        document_type: InboundDocumentType,
        supplier_id: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            reference,
            document_type,
            purchase_order_id: None,
            supplier_id,
            status: InboundDocumentStatus::Pending,
            file_path: None,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Associate document with a purchase order.
    pub fn associate_with_po(&mut self, purchase_order_id: Uuid) {
        self.purchase_order_id = Some(purchase_order_id);
        self.updated_at = Utc::now();
    }

    /// Set document file path.
    pub fn set_file_path(&mut self, file_path: String) {
        self.file_path = Some(file_path);
        self.updated_at = Utc::now();
    }

    /// Set document metadata.
    pub fn set_metadata(&mut self, metadata: serde_json::Value) {
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
    }

    /// Approve document.
    pub fn approve(&mut self) {
        self.status = InboundDocumentStatus::Approved;
        self.updated_at = Utc::now();
    }

    /// Reject document.
    pub fn reject(&mut self) {
        self.status = InboundDocumentStatus::Rejected;
        self.updated_at = Utc::now();
    }

    /// Archive document.
    pub fn archive(&mut self) {
        self.status = InboundDocumentStatus::Archived;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_inbound_document() {
        let supplier_id = Uuid::new_v4();
        let doc = InboundDocument::new(
            "DOC-001".to_string(),
            InboundDocumentType::Invoice,
            supplier_id,
        );

        assert_eq!(doc.reference, "DOC-001");
        assert_eq!(doc.document_type, InboundDocumentType::Invoice);
        assert_eq!(doc.supplier_id, supplier_id);
        assert_eq!(doc.status, InboundDocumentStatus::Pending);
        assert!(doc.purchase_order_id.is_none());
        assert!(doc.file_path.is_none());
    }

    #[test]
    fn test_associate_with_po() {
        let supplier_id = Uuid::new_v4();
        let po_id = Uuid::new_v4();
        let mut doc = InboundDocument::new(
            "DOC-002".to_string(),
            InboundDocumentType::PackingList,
            supplier_id,
        );

        doc.associate_with_po(po_id);
        assert_eq!(doc.purchase_order_id, Some(po_id));
    }

    #[test]
    fn test_set_file_path() {
        let supplier_id = Uuid::new_v4();
        let mut doc = InboundDocument::new(
            "DOC-003".to_string(),
            InboundDocumentType::Invoice,
            supplier_id,
        );

        doc.set_file_path("/documents/invoice-001.pdf".to_string());
        assert_eq!(doc.file_path, Some("/documents/invoice-001.pdf".to_string()));
    }

    #[test]
    fn test_set_metadata() {
        let supplier_id = Uuid::new_v4();
        let mut doc = InboundDocument::new(
            "DOC-004".to_string(),
            InboundDocumentType::Invoice,
            supplier_id,
        );

        let metadata = json!({
            "invoice_number": "INV-001",
            "total_amount": 5000.00,
            "currency": "USD"
        });
        doc.set_metadata(metadata.clone());
        assert_eq!(doc.metadata, Some(metadata));
    }

    #[test]
    fn test_approve_document() {
        let supplier_id = Uuid::new_v4();
        let mut doc = InboundDocument::new(
            "DOC-005".to_string(),
            InboundDocumentType::Invoice,
            supplier_id,
        );

        assert_eq!(doc.status, InboundDocumentStatus::Pending);
        doc.approve();
        assert_eq!(doc.status, InboundDocumentStatus::Approved);
    }

    #[test]
    fn test_reject_document() {
        let supplier_id = Uuid::new_v4();
        let mut doc = InboundDocument::new(
            "DOC-006".to_string(),
            InboundDocumentType::Invoice,
            supplier_id,
        );

        doc.reject();
        assert_eq!(doc.status, InboundDocumentStatus::Rejected);
    }

    #[test]
    fn test_archive_document() {
        let supplier_id = Uuid::new_v4();
        let mut doc = InboundDocument::new(
            "DOC-007".to_string(),
            InboundDocumentType::Invoice,
            supplier_id,
        );

        doc.approve();
        doc.archive();
        assert_eq!(doc.status, InboundDocumentStatus::Archived);
    }

    #[test]
    fn test_inbound_document_serialization() {
        let supplier_id = Uuid::new_v4();
        let doc = InboundDocument::new(
            "DOC-008".to_string(),
            InboundDocumentType::PurchaseOrder,
            supplier_id,
        );

        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: InboundDocument = serde_json::from_str(&json).unwrap();

        assert_eq!(doc.reference, deserialized.reference);
        assert_eq!(doc.document_type, deserialized.document_type);
        assert_eq!(doc.status, deserialized.status);
    }
}
