//! Pick task components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Pick task data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct PickTask {
    pub id: String,
    pub task_id: String,
    pub order_id: String,
    pub sku: String,
    pub quantity: u32,
    pub from_location: String,
    pub status: String,
    pub assigned_to: Option<String>,
    pub created_at: String,
}

/// Pick view component
/// 
/// Displays a table of pick tasks for fulfilling orders.
#[component]
pub fn PickView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let pick_tasks = vec![
        PickTask {
            id: "1".to_string(),
            task_id: "PICK-001".to_string(),
            order_id: "ORD-2026-001".to_string(),
            sku: "DRONE-001".to_string(),
            quantity: 2,
            from_location: "A-01-02".to_string(),
            status: "In Progress".to_string(),
            assigned_to: Some("John Doe".to_string()),
            created_at: "2026-01-25 11:00:00".to_string(),
        },
        PickTask {
            id: "2".to_string(),
            task_id: "PICK-002".to_string(),
            order_id: "ORD-2026-002".to_string(),
            sku: "BATTERY-001".to_string(),
            quantity: 4,
            from_location: "B-02-03".to_string(),
            status: "Pending".to_string(),
            assigned_to: None,
            created_at: "2026-01-25 11:15:00".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Pick"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Task ID".to_string(), key: "task_id".to_string() },
                    TableColumn { header: "Order ID".to_string(), key: "order_id".to_string() },
                    TableColumn { header: "SKU".to_string(), key: "sku".to_string() },
                    TableColumn { header: "Quantity".to_string(), key: "quantity".to_string() },
                    TableColumn { header: "From Location".to_string(), key: "from_location".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "Assigned To".to_string(), key: "assigned_to".to_string() },
                ],
                rows: pick_tasks.iter().map(|task| {
                    vec![
                        task.task_id.clone(),
                        task.order_id.clone(),
                        task.sku.clone(),
                        task.quantity.to_string(),
                        task.from_location.clone(),
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
    fn pick_view_renders() {
        let mut vdom = VirtualDom::new(PickView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Pick"));
        assert!(html.contains("Task ID"));
    }
}
