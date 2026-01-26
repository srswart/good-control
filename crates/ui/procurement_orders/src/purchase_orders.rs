//! Purchase Orders components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Purchase Order data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct PurchaseOrder {
    pub id: String,
    pub po_number: String,
    pub supplier: String,
    pub status: String,
    pub total_amount: String,
    pub created_at: String,
    pub expected_delivery: Option<String>,
}

/// Purchase Orders view component
/// 
/// Displays a table of purchase orders with their status and details.
#[component]
pub fn PurchaseOrdersView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let purchase_orders = vec![
        PurchaseOrder {
            id: "1".to_string(),
            po_number: "PO-2026-001".to_string(),
            supplier: "Supplier A".to_string(),
            status: "Pending".to_string(),
            total_amount: "$5,000.00".to_string(),
            created_at: "2026-01-25 09:00:00".to_string(),
            expected_delivery: Some("2026-02-01".to_string()),
        },
        PurchaseOrder {
            id: "2".to_string(),
            po_number: "PO-2026-002".to_string(),
            supplier: "Supplier B".to_string(),
            status: "Received".to_string(),
            total_amount: "$3,200.00".to_string(),
            created_at: "2026-01-24 14:30:00".to_string(),
            expected_delivery: Some("2026-01-30".to_string()),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Purchase Orders"
            }
            Table {
                columns: vec![
                    TableColumn { header: "PO Number".to_string(), key: "po_number".to_string() },
                    TableColumn { header: "Supplier".to_string(), key: "supplier".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "Total Amount".to_string(), key: "total_amount".to_string() },
                    TableColumn { header: "Created".to_string(), key: "created".to_string() },
                    TableColumn { header: "Expected Delivery".to_string(), key: "expected_delivery".to_string() },
                ],
                rows: purchase_orders.iter().map(|po| {
                    vec![
                        po.po_number.clone(),
                        po.supplier.clone(),
                        po.status.clone(),
                        po.total_amount.clone(),
                        po.created_at.clone(),
                        po.expected_delivery.clone().unwrap_or_else(|| "-".to_string()),
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
    fn purchase_orders_view_renders() {
        let mut vdom = VirtualDom::new(PurchaseOrdersView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Purchase Orders"));
        assert!(html.contains("PO Number"));
    }
}
