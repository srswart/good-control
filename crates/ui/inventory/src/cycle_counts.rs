//! Cycle counts components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Cycle count data structure (placeholder)
#[derive(Debug, Clone)]
pub struct CycleCount {
    pub id: String,
    pub location: String,
    pub scheduled_date: String,
    pub status: String,
    pub counted_by: Option<String>,
}

/// Cycle counts view component
/// 
/// Displays cycle count tasks and results.
#[component]
pub fn CycleCountsView() -> Element {
    // Placeholder data
    let counts = vec![
        CycleCount {
            id: "1".to_string(),
            location: "Warehouse A / Zone 1 / Bin A1".to_string(),
            scheduled_date: "2026-01-26".to_string(),
            status: "pending".to_string(),
            counted_by: None,
        },
        CycleCount {
            id: "2".to_string(),
            location: "Warehouse A / Zone 2 / Bin B1".to_string(),
            scheduled_date: "2026-01-25".to_string(),
            status: "completed".to_string(),
            counted_by: Some("operator1".to_string()),
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Cycle Counts"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Location".to_string(), key: "location".to_string() },
                    TableColumn { header: "Scheduled Date".to_string(), key: "scheduled_date".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "Counted By".to_string(), key: "counted_by".to_string() },
                ],
                rows: counts.iter().map(|count| {
                    vec![
                        count.location.clone(),
                        count.scheduled_date.clone(),
                        count.status.clone(),
                        count.counted_by.clone().unwrap_or_default(),
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
    fn cycle_counts_view_renders() {
        let mut vdom = VirtualDom::new(CycleCountsView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Cycle Counts"));
        assert!(html.contains("Location"));
    }
}
