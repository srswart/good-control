//! Shipment domain models and logic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Shipment status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipmentStatus {
    /// Shipment is being prepared.
    Preparing,
    /// Shipment is ready for pickup.
    ReadyForPickup,
    /// Shipment has been picked up by carrier.
    InTransit,
    /// Shipment is out for delivery.
    OutForDelivery,
    /// Shipment has been delivered.
    Delivered,
    /// Shipment has been cancelled.
    Cancelled,
    /// Shipment has an exception.
    Exception,
}

/// Shipment entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipment {
    /// Unique shipment ID.
    pub id: Uuid,
    /// Shipment reference number (human-readable).
    pub reference: String,
    /// Current status.
    pub status: ShipmentStatus,
    /// Origin location.
    pub origin: String,
    /// Destination location.
    pub destination: String,
    /// Carrier name (optional).
    pub carrier: Option<String>,
    /// Tracking number (optional).
    pub tracking_number: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl Shipment {
    /// Create a new shipment.
    pub fn new(reference: String, origin: String, destination: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            reference,
            status: ShipmentStatus::Preparing,
            origin,
            destination,
            carrier: None,
            tracking_number: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update shipment status.
    pub fn update_status(&mut self, new_status: ShipmentStatus) {
        self.status = new_status;
        self.updated_at = Utc::now();
    }

    /// Assign carrier and tracking number.
    pub fn assign_carrier(&mut self, carrier: String, tracking_number: String) {
        self.carrier = Some(carrier);
        self.tracking_number = Some(tracking_number);
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_shipment() {
        let shipment = Shipment::new(
            "SHP-001".to_string(),
            "Warehouse A".to_string(),
            "Customer Site B".to_string(),
        );

        assert_eq!(shipment.reference, "SHP-001");
        assert_eq!(shipment.origin, "Warehouse A");
        assert_eq!(shipment.destination, "Customer Site B");
        assert_eq!(shipment.status, ShipmentStatus::Preparing);
        assert!(shipment.carrier.is_none());
        assert!(shipment.tracking_number.is_none());
    }

    #[test]
    fn test_update_status() {
        let mut shipment = Shipment::new(
            "SHP-002".to_string(),
            "Origin".to_string(),
            "Destination".to_string(),
        );

        let initial_updated_at = shipment.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));

        shipment.update_status(ShipmentStatus::InTransit);

        assert_eq!(shipment.status, ShipmentStatus::InTransit);
        assert!(shipment.updated_at > initial_updated_at);
    }

    #[test]
    fn test_assign_carrier() {
        let mut shipment = Shipment::new(
            "SHP-003".to_string(),
            "Origin".to_string(),
            "Destination".to_string(),
        );

        shipment.assign_carrier("DHL".to_string(), "1234567890".to_string());

        assert_eq!(shipment.carrier, Some("DHL".to_string()));
        assert_eq!(shipment.tracking_number, Some("1234567890".to_string()));
    }

    #[test]
    fn test_shipment_serialization() {
        let shipment = Shipment::new(
            "SHP-004".to_string(),
            "Origin".to_string(),
            "Destination".to_string(),
        );

        let json = serde_json::to_string(&shipment).unwrap();
        let deserialized: Shipment = serde_json::from_str(&json).unwrap();

        assert_eq!(shipment.reference, deserialized.reference);
        assert_eq!(shipment.status, deserialized.status);
    }
}

