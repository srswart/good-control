//! Traceability (movement history) components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Inventory movement data structure (placeholder)
#[derive(Debug, Clone)]
pub struct InventoryMovement {
    pub timestamp: String,
    pub sku: String,
    pub reason: String,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
    pub quantity: i64,
    pub actor: String,
}

/// Traceability view component
/// 
/// Displays movement history for inventory items.
#[component]
pub fn TraceabilityView() -> Element {
    // Placeholder data
    let movements = vec![
        InventoryMovement {
            timestamp: "2026-01-25 10:00:00".to_string(),
            sku: "DRONE-001".to_string(),
            reason: "receipt".to_string(),
            from_location: None,
            to_location: Some("Warehouse A / Zone 1 / Bin A1".to_string()),
            quantity: 5,
            actor: "operator1".to_string(),
        },
        InventoryMovement {
            timestamp: "2026-01-25 14:00:00".to_string(),
            sku: "DRONE-001".to_string(),
            reason: "allocation".to_string(),
            from_location: Some("Warehouse A / Zone 1 / Bin A1".to_string()),
            to_location: Some("Staging Area".to_string()),
            quantity: 2,
            actor: "operator2".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Traceability"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Timestamp".to_string(), key: "timestamp".to_string() },
                    TableColumn { header: "SKU".to_string(), key: "sku".to_string() },
                    TableColumn { header: "Reason".to_string(), key: "reason".to_string() },
                    TableColumn { header: "From".to_string(), key: "from".to_string() },
                    TableColumn { header: "To".to_string(), key: "to".to_string() },
                    TableColumn { header: "Quantity".to_string(), key: "quantity".to_string() },
                    TableColumn { header: "Actor".to_string(), key: "actor".to_string() },
                ],
                rows: movements.iter().map(|mov| {
                    vec![
                        mov.timestamp.clone(),
                        mov.sku.clone(),
                        mov.reason.clone(),
                        mov.from_location.clone().unwrap_or_default(),
                        mov.to_location.clone().unwrap_or_default(),
                        mov.quantity.to_string(),
                        mov.actor.clone(),
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
    fn traceability_view_renders() {
        let mut vdom = VirtualDom::new(TraceabilityView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Traceability"));
        assert!(html.contains("Timestamp"));
    }
}
