//! Stock/inventory positions components

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use ui_design_system::{Table, TableColumn};
use ui_foundation::{graphql_endpoint, ErrorCategory, GraphQLClient, GraphQLError};

/// Inventory position data from GraphQL query
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InventoryPosition {
    pub sku: String,
    #[serde(rename = "locationId")]
    pub location_id: String,
    pub status: String,
    #[serde(rename = "quantityOnHand")]
    pub quantity_on_hand: i64,
    #[serde(rename = "quantityReserved")]
    pub quantity_reserved: i64,
    #[serde(rename = "quantityAvailable")]
    pub quantity_available: i64,
}

/// Inventory positions query response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InventoryPositionsQueryResponse {
    #[serde(rename = "inventoryPositions")]
    pub inventory_positions: Vec<InventoryPosition>,
}

/// Stock view component
///
/// Displays current inventory positions across locations with filtering and real-time data.
#[component]
pub fn StockView() -> Element {
    let mut positions = use_signal(|| Vec::<InventoryPosition>::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut sku_filter = use_signal(|| String::new());
    let mut status_filter = use_signal(|| String::from("all"));

    // Fetch inventory positions from GraphQL
    let fetch_positions = move || {
        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());

            // Build query with optional filters
            let sku_val = sku_filter();
            let status_val = status_filter();

            let query = if !sku_val.is_empty() {
                format!(
                    r#"
                    query {{
                        inventoryPositions(sku: "{}") {{
                            sku
                            locationId
                            status
                            quantityOnHand
                            quantityReserved
                            quantityAvailable
                        }}
                    }}
                    "#,
                    sku_val
                )
            } else {
                r#"
                query {
                    inventoryPositions {
                        sku
                        locationId
                        status
                        quantityOnHand
                        quantityReserved
                        quantityAvailable
                    }
                }
                "#
                .to_string()
            };

            match client.query::<InventoryPositionsQueryResponse>(&query, None).await {
                Ok(response) => {
                    let mut pos_list = response.inventory_positions;

                    // Apply status filter client-side
                    if status_val != "all" {
                        pos_list.retain(|p| p.status == status_val);
                    }

                    positions.set(pos_list);
                }
                Err(e) => {
                    let error_message = match &e {
                        GraphQLError::Structured {
                            category,
                            message,
                            field_errors,
                        } => {
                            let category_str = match category {
                                ErrorCategory::Validation => "Validation Error",
                                ErrorCategory::Authorization => "Authorization Error",
                                ErrorCategory::ComplianceBlock => "Compliance Block",
                                ErrorCategory::NotFound => "Not Found",
                                ErrorCategory::Conflict => "Conflict",
                                ErrorCategory::Internal => "Internal Error",
                            };
                            if let Some(fields) = field_errors {
                                format!(
                                    "{}: {} (Fields: {})",
                                    category_str,
                                    message,
                                    fields
                                        .iter()
                                        .map(|f| format!("{}: {}", f.field, f.message))
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            } else {
                                format!("{}: {}", category_str, message)
                            }
                        }
                        GraphQLError::Network(msg) => format!("Network Error: {}", msg),
                        GraphQLError::Http { status, message } => {
                            format!("HTTP Error {}: {}", status, message)
                        }
                        GraphQLError::GraphQL(msg) => format!("GraphQL Error: {}", msg),
                        GraphQLError::Parse(msg) => format!("Parse Error: {}", msg),
                        GraphQLError::InvalidResponse(msg) => {
                            format!("Invalid Response: {}", msg)
                        }
                    };
                    error.set(Some(error_message));
                }
            }

            loading.set(false);
        });
    };

    // Load positions on mount
    use_effect(move || {
        fetch_positions();
    });

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",

            // Header with filters
            div {
                class: "mb-6",
                h2 {
                    class: "text-xl font-semibold text-gray-800 mb-4",
                    "Stock Positions"
                    if !positions().is_empty() {
                        span {
                            class: "ml-2 text-sm text-gray-500 font-normal",
                            "({positions().len()} positions)"
                        }
                    }
                }
                div {
                    class: "flex gap-4",
                    div {
                        class: "flex-1",
                        input {
                            r#type: "text",
                            placeholder: "Filter by SKU...",
                            class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{sku_filter}",
                            oninput: move |evt| {
                                sku_filter.set(evt.value().clone());
                            }
                        }
                    }
                    div {
                        class: "flex-1",
                        select {
                            class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{status_filter}",
                            onchange: move |evt| {
                                status_filter.set(evt.value().clone());
                                fetch_positions();
                            },
                            option { value: "all", "All Statuses" }
                            option { value: "available", "Available" }
                            option { value: "reserved", "Reserved" }
                            option { value: "quarantine", "Quarantine" }
                            option { value: "damaged", "Damaged" }
                        }
                    }
                    button {
                        class: "px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400",
                        disabled: loading(),
                        onclick: move |_| {
                            fetch_positions();
                        },
                        "Search"
                    }
                }
            }

            // Loading state
            if loading() {
                div {
                    class: "flex justify-center items-center py-12",
                    div {
                        class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"
                    }
                }
            }

            // Error state
            if let Some(err_msg) = error() {
                div {
                    class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4",
                    p {
                        class: "font-semibold",
                        "Error loading stock positions"
                    }
                    p {
                        class: "text-sm mt-1",
                        "{err_msg}"
                    }
                    button {
                        class: "mt-2 text-sm text-red-600 hover:text-red-800 underline",
                        onclick: move |_| {
                            fetch_positions();
                        },
                        "Retry"
                    }
                }
            }

            // Empty state
            if !loading() && error().is_none() && positions().is_empty() {
                div {
                    class: "text-center py-12",
                    p {
                        class: "text-gray-500 text-lg",
                        "No stock positions found"
                    }
                    p {
                        class: "text-gray-400 text-sm mt-2",
                        "Try adjusting your filters or receive inventory"
                    }
                }
            }

            // Data table
            if !loading() && error().is_none() && !positions().is_empty() {
                Table {
                    columns: vec![
                        TableColumn { header: "SKU".to_string(), key: "sku".to_string() },
                        TableColumn { header: "Location".to_string(), key: "location".to_string() },
                        TableColumn { header: "Status".to_string(), key: "status".to_string() },
                        TableColumn { header: "On Hand".to_string(), key: "on_hand".to_string() },
                        TableColumn { header: "Reserved".to_string(), key: "reserved".to_string() },
                        TableColumn { header: "Available".to_string(), key: "available".to_string() },
                    ],
                    rows: positions().iter().map(|pos| {
                        vec![
                            pos.sku.clone(),
                            pos.location_id.clone(),
                            pos.status.clone(),
                            pos.quantity_on_hand.to_string(),
                            pos.quantity_reserved.to_string(),
                            pos.quantity_available.to_string(),
                        ]
                    }).collect(),
                }
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

        // Check for header
        assert!(html.contains("Stock Positions"));

        // Check for filter inputs
        assert!(html.contains("Filter by SKU"));
        assert!(html.contains("All Statuses"));

        // Check for search button
        assert!(html.contains("Search"));
    }

    #[test]
    fn inventory_position_deserialization() {
        let json = r#"{
            "sku": "TEST-001",
            "locationId": "WH-01-A-001",
            "status": "available",
            "quantityOnHand": 100,
            "quantityReserved": 20,
            "quantityAvailable": 80
        }"#;

        let position: InventoryPosition = serde_json::from_str(json).unwrap();
        assert_eq!(position.sku, "TEST-001");
        assert_eq!(position.location_id, "WH-01-A-001");
        assert_eq!(position.quantity_on_hand, 100);
        assert_eq!(position.quantity_reserved, 20);
        assert_eq!(position.quantity_available, 80);
    }

    #[test]
    fn inventory_positions_query_response_deserialization() {
        let json = r#"{
            "inventoryPositions": [
                {
                    "sku": "TEST-001",
                    "locationId": "WH-01-A-001",
                    "status": "available",
                    "quantityOnHand": 100,
                    "quantityReserved": 20,
                    "quantityAvailable": 80
                }
            ]
        }"#;

        let response: InventoryPositionsQueryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.inventory_positions.len(), 1);
        assert_eq!(response.inventory_positions[0].sku, "TEST-001");
        assert_eq!(response.inventory_positions[0].location_id, "WH-01-A-001");
    }
}
