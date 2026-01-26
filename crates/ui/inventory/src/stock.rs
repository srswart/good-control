//! Stock/inventory positions components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Inventory position data structure (placeholder)
#[derive(Debug, Clone)]
pub struct InventoryPosition {
    pub sku: String,
    pub location: String,
    pub status: String,
    pub quantity_on_hand: i64,
    pub quantity_available: i64,
}

/// Stock view component
/// 
/// Displays current inventory positions across locations.
#[component]
pub fn StockView() -> Element {
    // Placeholder data
    let positions = vec![
        InventoryPosition {
            sku: "DRONE-001".to_string(),
            location: "Warehouse A / Zone 1 / Bin A1".to_string(),
            status: "available".to_string(),
            quantity_on_hand: 5,
            quantity_available: 3,
        },
        InventoryPosition {
            sku: "BATTERY-001".to_string(),
            location: "Warehouse A / Zone 2 / Bin B1".to_string(),
            status: "available".to_string(),
            quantity_on_hand: 20,
            quantity_available: 18,
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Stock Positions"
            }
            Table {
                columns: vec![
                    TableColumn { header: "SKU".to_string(), key: "sku".to_string() },
                    TableColumn { header: "Location".to_string(), key: "location".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "On Hand".to_string(), key: "on_hand".to_string() },
                    TableColumn { header: "Available".to_string(), key: "available".to_string() },
                ],
                rows: positions.iter().map(|pos| {
                    vec![
                        pos.sku.clone(),
                        pos.location.clone(),
                        pos.status.clone(),
                        pos.quantity_on_hand.to_string(),
                        pos.quantity_available.to_string(),
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
    fn stock_view_renders() {
        let mut vdom = VirtualDom::new(StockView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Stock Positions"));
        assert!(html.contains("SKU"));
    }
}
