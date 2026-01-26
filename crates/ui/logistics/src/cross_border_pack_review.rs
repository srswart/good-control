//! Cross-border pack review components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Cross-border pack review data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct CrossBorderPack {
    pub id: String,
    pub pack_id: String,
    pub order_id: String,
    pub origin_country: String,
    pub destination_country: String,
    pub compliance_status: String,
    pub review_status: String,
    pub reviewed_by: Option<String>,
    pub created_at: String,
}

/// Cross-border pack review view component
/// 
/// Displays a table of cross-border shipments requiring compliance review.
#[component]
pub fn CrossBorderPackReviewView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let cross_border_packs = vec![
        CrossBorderPack {
            id: "1".to_string(),
            pack_id: "PACK-CB-001".to_string(),
            order_id: "ORD-2026-001".to_string(),
            origin_country: "US".to_string(),
            destination_country: "CA".to_string(),
            compliance_status: "Pending Review".to_string(),
            review_status: "Not Reviewed".to_string(),
            reviewed_by: None,
            created_at: "2026-01-25 09:00:00".to_string(),
        },
        CrossBorderPack {
            id: "2".to_string(),
            pack_id: "PACK-CB-002".to_string(),
            order_id: "ORD-2026-003".to_string(),
            origin_country: "US".to_string(),
            destination_country: "MX".to_string(),
            compliance_status: "Approved".to_string(),
            review_status: "Reviewed".to_string(),
            reviewed_by: Some("Compliance Officer".to_string()),
            created_at: "2026-01-24 11:30:00".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Cross-Border Pack Review"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Pack ID".to_string(), key: "pack_id".to_string() },
                    TableColumn { header: "Order ID".to_string(), key: "order_id".to_string() },
                    TableColumn { header: "Route".to_string(), key: "route".to_string() },
                    TableColumn { header: "Compliance Status".to_string(), key: "compliance_status".to_string() },
                    TableColumn { header: "Review Status".to_string(), key: "review_status".to_string() },
                    TableColumn { header: "Reviewed By".to_string(), key: "reviewed_by".to_string() },
                    TableColumn { header: "Created".to_string(), key: "created".to_string() },
                ],
                rows: cross_border_packs.iter().map(|pack| {
                    let route = format!("{} → {}", pack.origin_country, pack.destination_country);
                    vec![
                        pack.pack_id.clone(),
                        pack.order_id.clone(),
                        route,
                        pack.compliance_status.clone(),
                        pack.review_status.clone(),
                        pack.reviewed_by.clone().unwrap_or_else(|| "-".to_string()),
                        pack.created_at.clone(),
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
    fn cross_border_pack_review_view_renders() {
        let mut vdom = VirtualDom::new(CrossBorderPackReviewView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Cross-Border Pack Review"));
        assert!(html.contains("Pack ID"));
    }
}
