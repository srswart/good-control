//! Approvals workflow components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Approval data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct Approval {
    pub id: String,
    pub request_id: String,
    pub request_type: String,
    pub requester: String,
    pub status: String,
    pub submitted_at: String,
    pub reviewed_by: Option<String>,
}

/// Approvals view component
/// 
/// Displays a table of compliance approvals with their status and details.
#[component]
pub fn ApprovalsView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let approvals = vec![
        Approval {
            id: "1".to_string(),
            request_id: "REQ-001".to_string(),
            request_type: "Export Authorization".to_string(),
            requester: "John Doe".to_string(),
            status: "Pending".to_string(),
            submitted_at: "2026-01-25 10:00:00".to_string(),
            reviewed_by: None,
        },
        Approval {
            id: "2".to_string(),
            request_id: "REQ-002".to_string(),
            request_type: "Transfer Authorization".to_string(),
            requester: "Jane Smith".to_string(),
            status: "Approved".to_string(),
            submitted_at: "2026-01-24 14:30:00".to_string(),
            reviewed_by: Some("Admin User".to_string()),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Approvals"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Request ID".to_string(), key: "request_id".to_string() },
                    TableColumn { header: "Type".to_string(), key: "type".to_string() },
                    TableColumn { header: "Requester".to_string(), key: "requester".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "Submitted".to_string(), key: "submitted".to_string() },
                    TableColumn { header: "Reviewed By".to_string(), key: "reviewed_by".to_string() },
                ],
                rows: approvals.iter().map(|approval| {
                    vec![
                        approval.request_id.clone(),
                        approval.request_type.clone(),
                        approval.requester.clone(),
                        approval.status.clone(),
                        approval.submitted_at.clone(),
                        approval.reviewed_by.clone().unwrap_or_else(|| "-".to_string()),
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
    fn approvals_view_renders() {
        let mut vdom = VirtualDom::new(ApprovalsView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Approvals"));
        assert!(html.contains("Request ID"));
    }
}
