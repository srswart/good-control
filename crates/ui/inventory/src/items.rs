//! Items list and detail components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Item data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub sku: String,
    pub name_en: String,
    pub name_zh: String,
    pub item_type: String,
    pub tracking_mode: String,
}

/// Items list component
/// 
/// Displays a table of inventory items with basic information.
#[component]
pub fn ItemsList() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let items = vec![
        Item {
            id: "1".to_string(),
            sku: "DRONE-001".to_string(),
            name_en: "Industrial Drone Model A".to_string(),
            name_zh: "工业无人机型号A".to_string(),
            item_type: "drone".to_string(),
            tracking_mode: "serial".to_string(),
        },
        Item {
            id: "2".to_string(),
            sku: "BATTERY-001".to_string(),
            name_en: "High Capacity Battery".to_string(),
            name_zh: "高容量电池".to_string(),
            item_type: "battery".to_string(),
            tracking_mode: "serial".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Items"
            }
            Table {
                columns: vec![
                    TableColumn { header: "SKU".to_string(), key: "sku".to_string() },
                    TableColumn { header: "Name".to_string(), key: "name".to_string() },
                    TableColumn { header: "Type".to_string(), key: "type".to_string() },
                    TableColumn { header: "Tracking".to_string(), key: "tracking".to_string() },
                ],
                rows: items.iter().map(|item| {
                    vec![
                        item.sku.clone(),
                        item.name_en.clone(),
                        item.item_type.clone(),
                        item.tracking_mode.clone(),
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
    fn items_list_renders() {
        let mut vdom = VirtualDom::new(ItemsList);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Items"));
        assert!(html.contains("SKU"));
    }
}
