//! Tracking information components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Tracking event data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct TrackingEvent {
    pub id: String,
    pub tracking_number: String,
    pub event_type: String,
    pub location: String,
    pub timestamp: String,
    pub description: String,
    pub status: String,
}

/// Tracking view component
/// 
/// Displays a table of tracking events for shipments.
#[component]
pub fn TrackingView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let tracking_events = vec![
        TrackingEvent {
            id: "1".to_string(),
            tracking_number: "FX123456789".to_string(),
            event_type: "In Transit".to_string(),
            location: "Chicago, IL, US".to_string(),
            timestamp: "2026-01-25 14:30:00".to_string(),
            description: "Package in transit to destination".to_string(),
            status: "Active".to_string(),
        },
        TrackingEvent {
            id: "2".to_string(),
            tracking_number: "FX123456789".to_string(),
            event_type: "Departed".to_string(),
            location: "New York, NY, US".to_string(),
            timestamp: "2026-01-25 10:15:00".to_string(),
            description: "Package departed from origin facility".to_string(),
            status: "Active".to_string(),
        },
        TrackingEvent {
            id: "3".to_string(),
            tracking_number: "DHL987654321".to_string(),
            event_type: "Delivered".to_string(),
            location: "Los Angeles, CA, US".to_string(),
            timestamp: "2026-01-24 16:00:00".to_string(),
            description: "Package delivered to recipient".to_string(),
            status: "Complete".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Tracking"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Tracking Number".to_string(), key: "tracking_number".to_string() },
                    TableColumn { header: "Event Type".to_string(), key: "event_type".to_string() },
                    TableColumn { header: "Location".to_string(), key: "location".to_string() },
                    TableColumn { header: "Timestamp".to_string(), key: "timestamp".to_string() },
                    TableColumn { header: "Description".to_string(), key: "description".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                ],
                rows: tracking_events.iter().map(|event| {
                    vec![
                        event.tracking_number.clone(),
                        event.event_type.clone(),
                        event.location.clone(),
                        event.timestamp.clone(),
                        event.description.clone(),
                        event.status.clone(),
                    ]
                }).collect(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutations;

    #[test]
    fn tracking_view_renders() {
        let mut vdom = VirtualDom::new(TrackingView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Tracking"));
        assert!(html.contains("Tracking Number"));
    }
}
