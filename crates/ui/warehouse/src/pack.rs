//! Pack task components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Pack task data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct PackTask {
    pub id: String,
    pub task_id: String,
    pub order_id: String,
    pub items_count: u32,
    pub status: String,
    pub assigned_to: Option<String>,
    pub created_at: String,
}

/// Pack view component
/// 
/// Displays a table of pack tasks for packaging picked items.
#[component]
pub fn PackView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let pack_tasks = vec![
        PackTask {
            id: "1".to_string(),
            task_id: "PACK-001".to_string(),
            order_id: "ORD-2026-001".to_string(),
            items_count: 3,
            status: "Ready".to_string(),
            assigned_to: Some("Jane Smith".to_string()),
            created_at: "2026-01-25 12:00:00".to_string(),
        },
        PackTask {
            id: "2".to_string(),
            task_id: "PACK-002".to_string(),
            order_id: "ORD-2026-002".to_string(),
            items_count: 5,
            status: "In Progress".to_string(),
            assigned_to: Some("John Doe".to_string()),
            created_at: "2026-01-25 12:15:00".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Pack"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Task ID".to_string(), key: "task_id".to_string() },
                    TableColumn { header: "Order ID".to_string(), key: "order_id".to_string() },
                    TableColumn { header: "Items".to_string(), key: "items".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "Assigned To".to_string(), key: "assigned_to".to_string() },
                    TableColumn { header: "Created".to_string(), key: "created".to_string() },
                ],
                rows: pack_tasks.iter().map(|task| {
                    vec![
                        task.task_id.clone(),
                        task.order_id.clone(),
                        task.items_count.to_string(),
                        task.status.clone(),
                        task.assigned_to.clone().unwrap_or_else(|| "-".to_string()),
                        task.created_at.clone(),
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
    fn pack_view_renders() {
        let mut vdom = VirtualDom::new(PackView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Pack"));
        assert!(html.contains("Task ID"));
    }
}
