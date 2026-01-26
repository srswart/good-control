//! Blocking reasons components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Blocking reason data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct BlockingReason {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub reason: String,
    pub severity: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

/// Blocking reasons view component
/// 
/// Displays a table of compliance blocking reasons that prevent operations.
#[component]
pub fn BlockingReasonsView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let blocking_reasons = vec![
        BlockingReason {
            id: "1".to_string(),
            entity_type: "Item".to_string(),
            entity_id: "DRONE-001".to_string(),
            reason: "Missing export authorization".to_string(),
            severity: "High".to_string(),
            created_at: "2026-01-25 08:00:00".to_string(),
            resolved_at: None,
        },
        BlockingReason {
            id: "2".to_string(),
            entity_type: "Transfer".to_string(),
            entity_id: "TRANS-002".to_string(),
            reason: "Jurisdiction mismatch".to_string(),
            severity: "Medium".to_string(),
            created_at: "2026-01-24 15:30:00".to_string(),
            resolved_at: Some("2026-01-25 10:00:00".to_string()),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Blocking Reasons"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Entity Type".to_string(), key: "entity_type".to_string() },
                    TableColumn { header: "Entity ID".to_string(), key: "entity_id".to_string() },
                    TableColumn { header: "Reason".to_string(), key: "reason".to_string() },
                    TableColumn { header: "Severity".to_string(), key: "severity".to_string() },
                    TableColumn { header: "Created".to_string(), key: "created".to_string() },
                    TableColumn { header: "Resolved".to_string(), key: "resolved".to_string() },
                ],
                rows: blocking_reasons.iter().map(|reason| {
                    vec![
                        reason.entity_type.clone(),
                        reason.entity_id.clone(),
                        reason.reason.clone(),
                        reason.severity.clone(),
                        reason.created_at.clone(),
                        reason.resolved_at.clone().unwrap_or_else(|| "-".to_string()),
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
    fn blocking_reasons_view_renders() {
        let mut vdom = VirtualDom::new(BlockingReasonsView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Blocking Reasons"));
        assert!(html.contains("Entity Type"));
    }
}
