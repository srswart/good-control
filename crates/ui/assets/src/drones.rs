//! Drones registry and lifecycle components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Drone asset data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct DroneAsset {
    pub serial: String,
    pub model: String,
    pub configuration_version: String,
    pub firmware: String,
    pub readiness_state: String,
    pub current_location: String,
}

/// Drones view component
/// 
/// Displays a table of drone assets with lifecycle information.
#[component]
pub fn DronesView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let drones = vec![
        DroneAsset {
            serial: "DRONE-SN-001".to_string(),
            model: "Industrial Drone Model A".to_string(),
            configuration_version: "v2.1".to_string(),
            firmware: "FW-1.5.3".to_string(),
            readiness_state: "ready".to_string(),
            current_location: "Warehouse A / Zone 1 / Bin A1".to_string(),
        },
        DroneAsset {
            serial: "DRONE-SN-002".to_string(),
            model: "Industrial Drone Model A".to_string(),
            configuration_version: "v2.0".to_string(),
            firmware: "FW-1.4.2".to_string(),
            readiness_state: "in_repair".to_string(),
            current_location: "Repair Area".to_string(),
        },
        DroneAsset {
            serial: "DRONE-SN-003".to_string(),
            model: "Industrial Drone Model B".to_string(),
            configuration_version: "v1.5".to_string(),
            firmware: "FW-1.3.1".to_string(),
            readiness_state: "grounded".to_string(),
            current_location: "Warehouse A / Zone 2 / Bin B2".to_string(),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Drones"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Serial".to_string(), key: "serial".to_string() },
                    TableColumn { header: "Model".to_string(), key: "model".to_string() },
                    TableColumn { header: "Config Version".to_string(), key: "config".to_string() },
                    TableColumn { header: "Firmware".to_string(), key: "firmware".to_string() },
                    TableColumn { header: "Readiness State".to_string(), key: "state".to_string() },
                    TableColumn { header: "Location".to_string(), key: "location".to_string() },
                ],
                rows: drones.iter().map(|drone| {
                    vec![
                        drone.serial.clone(),
                        drone.model.clone(),
                        drone.configuration_version.clone(),
                        drone.firmware.clone(),
                        drone.readiness_state.clone(),
                        drone.current_location.clone(),
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
    fn drones_view_renders() {
        let mut vdom = VirtualDom::new(DronesView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Drones"));
        assert!(html.contains("Serial"));
    }
}
