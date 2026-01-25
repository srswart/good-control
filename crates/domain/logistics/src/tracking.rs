//! Tracking event domain models and logic.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ShipmentStatus;

/// Type of tracking event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackingEventType {
    /// Shipment created.
    Created,
    /// Shipment ready for pickup.
    ReadyForPickup,
    /// Picked up by carrier.
    PickedUp,
    /// In transit.
    InTransit,
    /// Arrived at facility.
    ArrivedAtFacility,
    /// Out for delivery.
    OutForDelivery,
    /// Delivered.
    Delivered,
    /// Delivery attempted.
    DeliveryAttempted,
    /// Exception occurred.
    Exception,
    /// Cancelled.
    Cancelled,
}

impl TrackingEventType {
    /// Get the corresponding shipment status for this event type.
    pub fn to_shipment_status(&self) -> ShipmentStatus {
        match self {
            TrackingEventType::Created => ShipmentStatus::Preparing,
            TrackingEventType::ReadyForPickup => ShipmentStatus::ReadyForPickup,
            TrackingEventType::PickedUp => ShipmentStatus::InTransit,
            TrackingEventType::InTransit => ShipmentStatus::InTransit,
            TrackingEventType::ArrivedAtFacility => ShipmentStatus::InTransit,
            TrackingEventType::OutForDelivery => ShipmentStatus::OutForDelivery,
            TrackingEventType::Delivered => ShipmentStatus::Delivered,
            TrackingEventType::DeliveryAttempted => ShipmentStatus::OutForDelivery,
            TrackingEventType::Exception => ShipmentStatus::Exception,
            TrackingEventType::Cancelled => ShipmentStatus::Cancelled,
        }
    }
}

/// Tracking event entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingEvent {
    /// Unique event ID.
    pub id: Uuid,
    /// Associated shipment ID.
    pub shipment_id: Uuid,
    /// Event type.
    pub event_type: TrackingEventType,
    /// Event timestamp.
    pub timestamp: DateTime<Utc>,
    /// Location where event occurred (optional).
    pub location: Option<String>,
    /// Event description/notes.
    pub description: String,
    /// Additional metadata (optional).
    pub metadata: Option<serde_json::Value>,
}

impl TrackingEvent {
    /// Create a new tracking event.
    pub fn new(shipment_id: Uuid, event_type: TrackingEventType, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            shipment_id,
            event_type,
            timestamp: Utc::now(),
            location: None,
            description,
            metadata: None,
        }
    }

    /// Create a new tracking event with location.
    pub fn with_location(
        shipment_id: Uuid,
        event_type: TrackingEventType,
        description: String,
        location: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            shipment_id,
            event_type,
            timestamp: Utc::now(),
            location: Some(location),
            description,
            metadata: None,
        }
    }

    /// Add metadata to the event.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Timeline of tracking events for a shipment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingTimeline {
    /// Shipment ID.
    pub shipment_id: Uuid,
    /// Events in chronological order.
    pub events: Vec<TrackingEvent>,
}

impl TrackingTimeline {
    /// Create a new empty timeline for a shipment.
    pub fn new(shipment_id: Uuid) -> Self {
        Self {
            shipment_id,
            events: Vec::new(),
        }
    }

    /// Add an event to the timeline.
    pub fn add_event(&mut self, event: TrackingEvent) {
        self.events.push(event);
        // Keep events sorted by timestamp
        self.events.sort_by_key(|e| e.timestamp);
    }

    /// Get the most recent event.
    pub fn latest_event(&self) -> Option<&TrackingEvent> {
        self.events.last()
    }

    /// Get the current status based on the latest event.
    pub fn current_status(&self) -> Option<ShipmentStatus> {
        self.latest_event()
            .map(|e| e.event_type.to_shipment_status())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_tracking_event() {
        let shipment_id = Uuid::new_v4();
        let event = TrackingEvent::new(
            shipment_id,
            TrackingEventType::Created,
            "Shipment created".to_string(),
        );

        assert_eq!(event.shipment_id, shipment_id);
        assert_eq!(event.event_type, TrackingEventType::Created);
        assert_eq!(event.description, "Shipment created");
        assert!(event.location.is_none());
        assert!(event.metadata.is_none());
    }

    #[test]
    fn test_tracking_event_with_location() {
        let shipment_id = Uuid::new_v4();
        let event = TrackingEvent::with_location(
            shipment_id,
            TrackingEventType::ArrivedAtFacility,
            "Arrived at sorting facility".to_string(),
            "Shanghai Distribution Center".to_string(),
        );

        assert_eq!(event.shipment_id, shipment_id);
        assert_eq!(event.event_type, TrackingEventType::ArrivedAtFacility);
        assert_eq!(
            event.location,
            Some("Shanghai Distribution Center".to_string())
        );
    }

    #[test]
    fn test_tracking_event_with_metadata() {
        let shipment_id = Uuid::new_v4();
        let metadata = json!({
            "carrier": "DHL",
            "tracking_number": "1234567890",
            "temperature": "22C"
        });

        let event = TrackingEvent::new(
            shipment_id,
            TrackingEventType::InTransit,
            "Package in transit".to_string(),
        )
        .with_metadata(metadata.clone());

        assert_eq!(event.metadata, Some(metadata));
    }

    #[test]
    fn test_event_type_to_shipment_status() {
        assert_eq!(
            TrackingEventType::Created.to_shipment_status(),
            ShipmentStatus::Preparing
        );
        assert_eq!(
            TrackingEventType::PickedUp.to_shipment_status(),
            ShipmentStatus::InTransit
        );
        assert_eq!(
            TrackingEventType::Delivered.to_shipment_status(),
            ShipmentStatus::Delivered
        );
        assert_eq!(
            TrackingEventType::Exception.to_shipment_status(),
            ShipmentStatus::Exception
        );
    }

    #[test]
    fn test_tracking_timeline_new() {
        let shipment_id = Uuid::new_v4();
        let timeline = TrackingTimeline::new(shipment_id);

        assert_eq!(timeline.shipment_id, shipment_id);
        assert!(timeline.events.is_empty());
        assert!(timeline.latest_event().is_none());
        assert!(timeline.current_status().is_none());
    }

    #[test]
    fn test_tracking_timeline_add_event() {
        let shipment_id = Uuid::new_v4();
        let mut timeline = TrackingTimeline::new(shipment_id);

        let event1 = TrackingEvent::new(
            shipment_id,
            TrackingEventType::Created,
            "Shipment created".to_string(),
        );
        timeline.add_event(event1);

        assert_eq!(timeline.events.len(), 1);

        let event2 = TrackingEvent::new(
            shipment_id,
            TrackingEventType::PickedUp,
            "Picked up".to_string(),
        );
        timeline.add_event(event2);

        assert_eq!(timeline.events.len(), 2);
    }

    #[test]
    fn test_tracking_timeline_latest_event() {
        let shipment_id = Uuid::new_v4();
        let mut timeline = TrackingTimeline::new(shipment_id);

        timeline.add_event(TrackingEvent::new(
            shipment_id,
            TrackingEventType::Created,
            "Created".to_string(),
        ));

        std::thread::sleep(std::time::Duration::from_millis(10));

        timeline.add_event(TrackingEvent::new(
            shipment_id,
            TrackingEventType::InTransit,
            "In transit".to_string(),
        ));

        let latest = timeline.latest_event().unwrap();
        assert_eq!(latest.event_type, TrackingEventType::InTransit);
    }

    #[test]
    fn test_tracking_timeline_current_status() {
        let shipment_id = Uuid::new_v4();
        let mut timeline = TrackingTimeline::new(shipment_id);

        timeline.add_event(TrackingEvent::new(
            shipment_id,
            TrackingEventType::Created,
            "Created".to_string(),
        ));

        assert_eq!(timeline.current_status(), Some(ShipmentStatus::Preparing));

        timeline.add_event(TrackingEvent::new(
            shipment_id,
            TrackingEventType::Delivered,
            "Delivered".to_string(),
        ));

        assert_eq!(timeline.current_status(), Some(ShipmentStatus::Delivered));
    }

    #[test]
    fn test_tracking_event_serialization() {
        let shipment_id = Uuid::new_v4();
        let event = TrackingEvent::with_location(
            shipment_id,
            TrackingEventType::OutForDelivery,
            "Out for delivery".to_string(),
            "Beijing".to_string(),
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: TrackingEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.shipment_id, deserialized.shipment_id);
        assert_eq!(event.event_type, deserialized.event_type);
        assert_eq!(event.location, deserialized.location);
    }

    #[test]
    fn test_timeline_serialization() {
        let shipment_id = Uuid::new_v4();
        let mut timeline = TrackingTimeline::new(shipment_id);

        timeline.add_event(TrackingEvent::new(
            shipment_id,
            TrackingEventType::Created,
            "Created".to_string(),
        ));

        let json = serde_json::to_string(&timeline).unwrap();
        let deserialized: TrackingTimeline = serde_json::from_str(&json).unwrap();

        assert_eq!(timeline.shipment_id, deserialized.shipment_id);
        assert_eq!(timeline.events.len(), deserialized.events.len());
    }
}
