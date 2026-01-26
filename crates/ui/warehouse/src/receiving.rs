//! Receiving workflow components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Receiving task data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct ReceivingTask {
    pub id: String,
    pub task_id: String,
    pub po_number: Option<String>,
    pub expected_items: u32,
    pub received_items: u32,
    pub status: String,
    pub assigned_to: Option<String>,
    pub created_at: String,
}

/// Receiving view component
/// 
/// Displays a table of receiving tasks with their status and progress.
#[component]
pub fn ReceivingView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let receiving_tasks = vec![
        ReceivingTask {
            id: "1".to_string(),
            task_id: "RECV-001".to_string(),
            po_number: Some("PO-2026-001".to_string()),
            expected_items: 10,
            received_items: 8,
            status: "In Progress".to_string(),
            assigned_to: Some("John Doe".to_string()),
            created_at: "2026-01-25 09:00:00".to_string(),
        },
        ReceivingTask {
            id: "2".to_string(),
            task_id: "RECV-002".to_string(),
            po_number: Some("PO-2026-002".to_string()),
            expected_items: 5,
            received_items: 5,
            status: "Complete".to_string(),
            assigned_to: Some("Jane Smith".to_string()),
            created_at: "2026-01-24 14:30:00".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Receiving"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Task ID".to_string(), key: "task_id".to_string() },
                    TableColumn { header: "PO Number".to_string(), key: "po_number".to_string() },
                    TableColumn { header: "Progress".to_string(), key: "progress".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "Assigned To".to_string(), key: "assigned_to".to_string() },
                    TableColumn { header: "Created".to_string(), key: "created".to_string() },
                ],
                rows: receiving_tasks.iter().map(|task| {
                    let progress = format!("{}/{}", task.received_items, task.expected_items);
                    vec![
                        task.task_id.clone(),
                        task.po_number.clone().unwrap_or_else(|| "-".to_string()),
                        progress,
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
    fn receiving_view_renders() {
        let mut vdom = VirtualDom::new(ReceivingView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Receiving"));
        assert!(html.contains("Task ID"));
    }
}
