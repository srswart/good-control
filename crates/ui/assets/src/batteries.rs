//! Batteries registry and lifecycle components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Battery data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct Battery {
    pub serial: String,
    pub model: String,
    pub cycles: i32,
    pub last_test_date: Option<String>,
    pub lifecycle_state: String,
    pub assignment_pool: Option<String>,
    pub current_location: String,
}

/// Batteries view component
/// 
/// Displays a table of batteries with lifecycle and health information.
#[component]
pub fn BatteriesView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let batteries = vec![
        Battery {
            serial: "BAT-SN-001".to_string(),
            model: "High Capacity Battery".to_string(),
            cycles: 45,
            last_test_date: Some("2026-01-20".to_string()),
            lifecycle_state: "ready".to_string(),
            assignment_pool: Some("Pool A".to_string()),
            current_location: "Warehouse A / Zone 2 / Bin B1".to_string(),
        },
        Battery {
            serial: "BAT-SN-002".to_string(),
            model: "High Capacity Battery".to_string(),
            cycles: 120,
            last_test_date: Some("2026-01-15".to_string()),
            lifecycle_state: "grounded".to_string(),
            assignment_pool: None,
            current_location: "Repair Area".to_string(),
        },
        Battery {
            serial: "BAT-SN-003".to_string(),
            model: "Standard Battery".to_string(),
            cycles: 200,
            last_test_date: Some("2026-01-10".to_string()),
            lifecycle_state: "retired".to_string(),
            assignment_pool: None,
            current_location: "Scrap Area".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Batteries"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Serial".to_string(), key: "serial".to_string() },
                    TableColumn { header: "Model".to_string(), key: "model".to_string() },
                    TableColumn { header: "Cycles".to_string(), key: "cycles".to_string() },
                    TableColumn { header: "Last Test".to_string(), key: "last_test".to_string() },
                    TableColumn { header: "Lifecycle State".to_string(), key: "state".to_string() },
                    TableColumn { header: "Assignment Pool".to_string(), key: "pool".to_string() },
                    TableColumn { header: "Location".to_string(), key: "location".to_string() },
                ],
                rows: batteries.iter().map(|battery| {
                    vec![
                        battery.serial.clone(),
                        battery.model.clone(),
                        battery.cycles.to_string(),
                        battery.last_test_date.clone().unwrap_or_default(),
                        battery.lifecycle_state.clone(),
                        battery.assignment_pool.clone().unwrap_or_default(),
                        battery.current_location.clone(),
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
    fn batteries_view_renders() {
        let mut vdom = VirtualDom::new(BatteriesView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Batteries"));
        assert!(html.contains("Serial"));
    }
}
