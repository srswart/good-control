//! Allocations components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Allocation data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct Allocation {
    pub id: String,
    pub order_id: String,
    pub sku: String,
    pub quantity_requested: u32,
    pub quantity_allocated: u32,
    pub status: String,
    pub allocated_from: String,
    pub created_at: String,
}

/// Allocations view component
/// 
/// Displays a table of order allocations showing inventory allocation status.
#[component]
pub fn AllocationsView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let allocations = vec![
        Allocation {
            id: "1".to_string(),
            order_id: "ORD-2026-001".to_string(),
            sku: "DRONE-001".to_string(),
            quantity_requested: 5,
            quantity_allocated: 5,
            status: "Fully Allocated".to_string(),
            allocated_from: "A-01-02".to_string(),
            created_at: "2026-01-25 10:00:00".to_string(),
        },
        Allocation {
            id: "2".to_string(),
            order_id: "ORD-2026-002".to_string(),
            sku: "BATTERY-001".to_string(),
            quantity_requested: 10,
            quantity_allocated: 7,
            status: "Partially Allocated".to_string(),
            allocated_from: "B-02-03".to_string(),
            created_at: "2026-01-25 10:15:00".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Allocations"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Order ID".to_string(), key: "order_id".to_string() },
                    TableColumn { header: "SKU".to_string(), key: "sku".to_string() },
                    TableColumn { header: "Requested".to_string(), key: "requested".to_string() },
                    TableColumn { header: "Allocated".to_string(), key: "allocated".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "From Location".to_string(), key: "from_location".to_string() },
                    TableColumn { header: "Created".to_string(), key: "created".to_string() },
                ],
                rows: allocations.iter().map(|allocation| {
                    vec![
                        allocation.order_id.clone(),
                        allocation.sku.clone(),
                        allocation.quantity_requested.to_string(),
                        allocation.quantity_allocated.to_string(),
                        allocation.status.clone(),
                        allocation.allocated_from.clone(),
                        allocation.created_at.clone(),
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
    fn allocations_view_renders() {
        let mut vdom = VirtualDom::new(AllocationsView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Allocations"));
        assert!(html.contains("Order ID"));
    }
}
