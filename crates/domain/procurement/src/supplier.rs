//! Supplier domain models and logic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supplier entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    /// Unique supplier ID.
    pub id: Uuid,
    /// Supplier code (human-readable identifier).
    pub code: String,
    /// Supplier name (English).
    pub name_en: String,
    /// Supplier name (Chinese).
    pub name_zh: String,
    /// Contact email.
    pub email: Option<String>,
    /// Contact phone.
    pub phone: Option<String>,
    /// Address.
    pub address: Option<String>,
    /// Active status.
    pub active: bool,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl Supplier {
    /// Create a new supplier.
    pub fn new(code: String, name_en: String, name_zh: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            code,
            name_en,
            name_zh,
            email: None,
            phone: None,
            address: None,
            active: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update supplier contact information.
    pub fn update_contact(&mut self, email: Option<String>, phone: Option<String>, address: Option<String>) {
        self.email = email;
        self.phone = phone;
        self.address = address;
        self.updated_at = Utc::now();
    }

    /// Deactivate supplier.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = Utc::now();
    }

    /// Activate supplier.
    pub fn activate(&mut self) {
        self.active = true;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_supplier() {
        let supplier = Supplier::new(
            "SUP-001".to_string(),
            "Supplier A".to_string(),
            "供应商A".to_string(),
        );

        assert_eq!(supplier.code, "SUP-001");
        assert_eq!(supplier.name_en, "Supplier A");
        assert_eq!(supplier.name_zh, "供应商A");
        assert!(supplier.active);
        assert!(supplier.email.is_none());
        assert!(supplier.phone.is_none());
    }

    #[test]
    fn test_update_contact() {
        let mut supplier = Supplier::new(
            "SUP-002".to_string(),
            "Supplier B".to_string(),
            "供应商B".to_string(),
        );

        let initial_updated_at = supplier.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));

        supplier.update_contact(
            Some("contact@supplierb.com".to_string()),
            Some("+1-555-0123".to_string()),
            Some("123 Main St".to_string()),
        );

        assert_eq!(supplier.email, Some("contact@supplierb.com".to_string()));
        assert_eq!(supplier.phone, Some("+1-555-0123".to_string()));
        assert_eq!(supplier.address, Some("123 Main St".to_string()));
        assert!(supplier.updated_at > initial_updated_at);
    }

    #[test]
    fn test_deactivate_supplier() {
        let mut supplier = Supplier::new(
            "SUP-003".to_string(),
            "Supplier C".to_string(),
            "供应商C".to_string(),
        );

        assert!(supplier.active);
        supplier.deactivate();
        assert!(!supplier.active);
    }

    #[test]
    fn test_activate_supplier() {
        let mut supplier = Supplier::new(
            "SUP-004".to_string(),
            "Supplier D".to_string(),
            "供应商D".to_string(),
        );

        supplier.deactivate();
        assert!(!supplier.active);

        supplier.activate();
        assert!(supplier.active);
    }

    #[test]
    fn test_supplier_serialization() {
        let supplier = Supplier::new(
            "SUP-005".to_string(),
            "Supplier E".to_string(),
            "供应商E".to_string(),
        );

        let json = serde_json::to_string(&supplier).unwrap();
        let deserialized: Supplier = serde_json::from_str(&json).unwrap();

        assert_eq!(supplier.code, deserialized.code);
        assert_eq!(supplier.name_en, deserialized.name_en);
        assert_eq!(supplier.name_zh, deserialized.name_zh);
    }
}
