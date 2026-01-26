//! Putaway task components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Putaway task data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct PutawayTask {
    pub id: String,
    pub task_id: String,
    pub sku: String,
    pub quantity: u32,
    pub from_location: String,
    pub to_location: Option<String>,
    pub status: String,
    pub assigned_to: Option<String>,
    pub created_at: String,
}

/// Putaway view component
/// 
/// Displays a table of putaway tasks for moving received items to storage locations.
#[component]
pub fn PutawayView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let putaway_tasks = vec![
        PutawayTask {
            id: "1".to_string(),
            task_id: "PUT-001".to_string(),
            sku: "DRONE-001".to_string(),
            quantity: 5,
            from_location: "RECEIVING-01".to_string(),
            to_location: Some("A-01-02".to_string()),
            status: "Assigned".to_string(),
            assigned_to: Some("John Doe".to_string()),
            created_at: "2026-01-25 10:00:00".to_string(),
        },
        PutawayTask {
            id: "2".to_string(),
            task_id: "PUT-002".to_string(),
            sku: "BATTERY-001".to_string(),
            quantity: 10,
            from_location: "RECEIVING-01".to_string(),
            to_location: None,
            status: "Pending".to_string(),
            assigned_to: None,
            created_at: "2026-01-25 10:15:00".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Putaway"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Task ID".to_string(), key: "task_id".to_string() },
                    TableColumn { header: "SKU".to_string(), key: "sku".to_string() },
                    TableColumn { header: "Quantity".to_string(), key: "quantity".to_string() },
                    TableColumn { header: "From".to_string(), key: "from".to_string() },
                    TableColumn { header: "To".to_string(), key: "to".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "Assigned To".to_string(), key: "assigned_to".to_string() },
                ],
                rows: putaway_tasks.iter().map(|task| {
                    vec![
                        task.task_id.clone(),
                        task.sku.clone(),
                        task.quantity.to_string(),
                        task.from_location.clone(),
                        task.to_location.clone().unwrap_or_else(|| "-".to_string()),
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
    fn putaway_view_renders() {
        let mut vdom = VirtualDom::new(PutawayView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Putaway"));
        assert!(html.contains("Task ID"));
    }
}
