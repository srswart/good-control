//! Shipments management components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Shipment data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct Shipment {
    pub id: String,
    pub shipment_id: String,
    pub order_id: String,
    pub carrier: String,
    pub tracking_number: String,
    pub status: String,
    pub origin: String,
    pub destination: String,
    pub created_at: String,
}

/// Shipments view component
/// 
/// Displays a table of shipments with their status and details.
#[component]
pub fn ShipmentsView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let shipments = vec![
        Shipment {
            id: "1".to_string(),
            shipment_id: "SHIP-001".to_string(),
            order_id: "ORD-2026-001".to_string(),
            carrier: "FedEx".to_string(),
            tracking_number: "FX123456789".to_string(),
            status: "In Transit".to_string(),
            origin: "US".to_string(),
            destination: "CA".to_string(),
            created_at: "2026-01-25 10:00:00".to_string(),
        },
        Shipment {
            id: "2".to_string(),
            shipment_id: "SHIP-002".to_string(),
            order_id: "ORD-2026-002".to_string(),
            carrier: "DHL".to_string(),
            tracking_number: "DHL987654321".to_string(),
            status: "Delivered".to_string(),
            origin: "US".to_string(),
            destination: "US".to_string(),
            created_at: "2026-01-24 14:30:00".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Shipments"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Shipment ID".to_string(), key: "shipment_id".to_string() },
                    TableColumn { header: "Order ID".to_string(), key: "order_id".to_string() },
                    TableColumn { header: "Carrier".to_string(), key: "carrier".to_string() },
                    TableColumn { header: "Tracking".to_string(), key: "tracking".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "Route".to_string(), key: "route".to_string() },
                    TableColumn { header: "Created".to_string(), key: "created".to_string() },
                ],
                rows: shipments.iter().map(|shipment| {
                    let route = format!("{} → {}", shipment.origin, shipment.destination);
                    vec![
                        shipment.shipment_id.clone(),
                        shipment.order_id.clone(),
                        shipment.carrier.clone(),
                        shipment.tracking_number.clone(),
                        shipment.status.clone(),
                        route,
                        shipment.created_at.clone(),
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
    fn shipments_view_renders() {
        let mut vdom = VirtualDom::new(ShipmentsView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Shipments"));
        assert!(html.contains("Shipment ID"));
    }
}
