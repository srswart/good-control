//! Fulfillment components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Fulfillment data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct Fulfillment {
    pub id: String,
    pub order_id: String,
    pub customer: String,
    pub items_count: u32,
    pub status: String,
    pub fulfillment_date: Option<String>,
    pub created_at: String,
}

/// Fulfillment view component
/// 
/// Displays a table of order fulfillment status.
#[component]
pub fn FulfillmentView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let fulfillments = vec![
        Fulfillment {
            id: "1".to_string(),
            order_id: "ORD-2026-001".to_string(),
            customer: "Customer A".to_string(),
            items_count: 3,
            status: "Fulfilled".to_string(),
            fulfillment_date: Some("2026-01-25 15:00:00".to_string()),
            created_at: "2026-01-25 09:00:00".to_string(),
        },
        Fulfillment {
            id: "2".to_string(),
            order_id: "ORD-2026-002".to_string(),
            customer: "Customer B".to_string(),
            items_count: 5,
            status: "In Progress".to_string(),
            fulfillment_date: None,
            created_at: "2026-01-25 10:00:00".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Fulfillment"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Order ID".to_string(), key: "order_id".to_string() },
                    TableColumn { header: "Customer".to_string(), key: "customer".to_string() },
                    TableColumn { header: "Items".to_string(), key: "items".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "Fulfillment Date".to_string(), key: "fulfillment_date".to_string() },
                    TableColumn { header: "Created".to_string(), key: "created".to_string() },
                ],
                rows: fulfillments.iter().map(|fulfillment| {
                    vec![
                        fulfillment.order_id.clone(),
                        fulfillment.customer.clone(),
                        fulfillment.items_count.to_string(),
                        fulfillment.status.clone(),
                        fulfillment.fulfillment_date.clone().unwrap_or_else(|| "-".to_string()),
                        fulfillment.created_at.clone(),
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
    fn fulfillment_view_renders() {
        let mut vdom = VirtualDom::new(FulfillmentView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Fulfillment"));
        assert!(html.contains("Order ID"));
    }
}
