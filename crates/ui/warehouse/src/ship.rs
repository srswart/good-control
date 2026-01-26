//! Ship task components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Ship task data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct ShipTask {
    pub id: String,
    pub task_id: String,
    pub order_id: String,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub status: String,
    pub assigned_to: Option<String>,
    pub created_at: String,
}

/// Ship view component
/// 
/// Displays a table of ship tasks for shipping packed orders.
#[component]
pub fn ShipView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let ship_tasks = vec![
        ShipTask {
            id: "1".to_string(),
            task_id: "SHIP-001".to_string(),
            order_id: "ORD-2026-001".to_string(),
            carrier: Some("FedEx".to_string()),
            tracking_number: Some("FX123456789".to_string()),
            status: "Shipped".to_string(),
            assigned_to: Some("Jane Smith".to_string()),
            created_at: "2026-01-25 13:00:00".to_string(),
        },
        ShipTask {
            id: "2".to_string(),
            task_id: "SHIP-002".to_string(),
            order_id: "ORD-2026-002".to_string(),
            carrier: None,
            tracking_number: None,
            status: "Ready".to_string(),
            assigned_to: None,
            created_at: "2026-01-25 13:15:00".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Ship"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Task ID".to_string(), key: "task_id".to_string() },
                    TableColumn { header: "Order ID".to_string(), key: "order_id".to_string() },
                    TableColumn { header: "Carrier".to_string(), key: "carrier".to_string() },
                    TableColumn { header: "Tracking".to_string(), key: "tracking".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "Assigned To".to_string(), key: "assigned_to".to_string() },
                ],
                rows: ship_tasks.iter().map(|task| {
                    vec![
                        task.task_id.clone(),
                        task.order_id.clone(),
                        task.carrier.clone().unwrap_or_else(|| "-".to_string()),
                        task.tracking_number.clone().unwrap_or_else(|| "-".to_string()),
                        task.status.clone(),
                        task.assigned_to.clone().unwrap_or_else(|| "-".to_string()),
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
    fn ship_view_renders() {
        let mut vdom = VirtualDom::new(ShipView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Ship"));
        assert!(html.contains("Task ID"));
    }
}
