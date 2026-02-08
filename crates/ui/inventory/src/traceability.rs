//! Traceability (movement history) components

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use ui_design_system::{Table, TableColumn};
use ui_foundation::{graphql_endpoint, ErrorCategory, GraphQLClient, GraphQLError};

/// Location data from GraphQL
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Location {
    pub warehouse: String,
    pub zone: String,
    pub bin: String,
}

/// Inventory movement data from GraphQL query
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InventoryMovement {
    pub id: String,
    pub sku: String,
    pub reason: String,
    #[serde(rename = "fromLocation")]
    pub from_location: Option<Location>,
    #[serde(rename = "toLocation")]
    pub to_location: Option<Location>,
    pub quantity: i64,
    pub actor: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Serial genealogy entry from GraphQL
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SerialGenealogyEntry {
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    pub sku: String,
    pub reason: String,
    #[serde(rename = "fromLocation")]
    pub from_location: Option<Location>,
    #[serde(rename = "toLocation")]
    pub to_location: Option<Location>,
    pub quantity: i64,
    pub actor: String,
    pub timestamp: String,
}

/// Lot genealogy entry from GraphQL
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LotGenealogyEntry {
    #[serde(rename = "lotNumber")]
    pub lot_number: String,
    pub sku: String,
    pub reason: String,
    #[serde(rename = "fromLocation")]
    pub from_location: Option<Location>,
    #[serde(rename = "toLocation")]
    pub to_location: Option<Location>,
    pub quantity: i64,
    pub actor: String,
    pub timestamp: String,
}

/// Inventory movements query response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InventoryMovementsQueryResponse {
    #[serde(rename = "inventoryMovements")]
    pub inventory_movements: Vec<InventoryMovement>,
}

/// Serial genealogy query response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SerialGenealogyQueryResponse {
    #[serde(rename = "serialGenealogy")]
    pub serial_genealogy: Vec<SerialGenealogyEntry>,
}

/// Lot genealogy query response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct LotGenealogyQueryResponse {
    #[serde(rename = "lotGenealogy")]
    pub lot_genealogy: Vec<LotGenealogyEntry>,
}

/// Traceability view component
///
/// Displays movement history for inventory items with serial/lot trace lookup.
#[component]
pub fn TraceabilityView() -> Element {
    let mut movements = use_signal(|| Vec::<InventoryMovement>::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut sku_filter = use_signal(|| String::new());
    let mut trace_mode = use_signal(|| String::from("movements"));
    let mut trace_id = use_signal(|| String::new());

    // Fetch inventory movements from GraphQL
    let fetch_movements = move || {
        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());

            let sku_val = sku_filter();

            let query = if !sku_val.is_empty() {
                format!(
                    r#"
                    query {{
                        inventoryMovements(sku: "{}") {{
                            id
                            sku
                            reason
                            fromLocation {{
                                warehouse
                                zone
                                bin
                            }}
                            toLocation {{
                                warehouse
                                zone
                                bin
                            }}
                            quantity
                            actor
                            createdAt
                        }}
                    }}
                    "#,
                    sku_val
                )
            } else {
                r#"
                query {
                    inventoryMovements {
                        id
                        sku
                        reason
                        fromLocation {
                            warehouse
                            zone
                            bin
                        }
                        toLocation {
                            warehouse
                            zone
                            bin
                        }
                        quantity
                        actor
                        createdAt
                    }
                }
                "#
                .to_string()
            };

            match client.query::<InventoryMovementsQueryResponse>(&query, None).await {
                Ok(response) => {
                    movements.set(response.inventory_movements);
                }
                Err(e) => {
                    let error_message = format_error(&e);
                    error.set(Some(error_message));
                }
            }

            loading.set(false);
        });
    };

    // Fetch serial genealogy
    let fetch_serial_genealogy = move || {
        let serial_num = trace_id();
        if serial_num.is_empty() {
            return;
        }

        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());

            let query = format!(
                r#"
                query {{
                    serialGenealogy(serialNumber: "{}") {{
                        serialNumber
                        sku
                        reason
                        fromLocation {{
                            warehouse
                            zone
                            bin
                        }}
                        toLocation {{
                            warehouse
                            zone
                            bin
                        }}
                        quantity
                        actor
                        timestamp
                    }}
                }}
                "#,
                serial_num
            );

            match client.query::<SerialGenealogyQueryResponse>(&query, None).await {
                Ok(response) => {
                    // Convert serial genealogy to movements for display
                    let converted_movements: Vec<InventoryMovement> = response
                        .serial_genealogy
                        .into_iter()
                        .map(|entry| InventoryMovement {
                            id: format!("serial-{}", entry.serial_number),
                            sku: entry.sku,
                            reason: entry.reason,
                            from_location: entry.from_location,
                            to_location: entry.to_location,
                            quantity: entry.quantity,
                            actor: entry.actor,
                            created_at: entry.timestamp,
                        })
                        .collect();
                    movements.set(converted_movements);
                }
                Err(e) => {
                    let error_message = format_error(&e);
                    error.set(Some(error_message));
                }
            }

            loading.set(false);
        });
    };

    // Fetch lot genealogy
    let fetch_lot_genealogy = move || {
        let lot_num = trace_id();
        if lot_num.is_empty() {
            return;
        }

        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());

            let query = format!(
                r#"
                query {{
                    lotGenealogy(lotNumber: "{}") {{
                        lotNumber
                        sku
                        reason
                        fromLocation {{
                            warehouse
                            zone
                            bin
                        }}
                        toLocation {{
                            warehouse
                            zone
                            bin
                        }}
                        quantity
                        actor
                        timestamp
                    }}
                }}
                "#,
                lot_num
            );

            match client.query::<LotGenealogyQueryResponse>(&query, None).await {
                Ok(response) => {
                    // Convert lot genealogy to movements for display
                    let converted_movements: Vec<InventoryMovement> = response
                        .lot_genealogy
                        .into_iter()
                        .map(|entry| InventoryMovement {
                            id: format!("lot-{}", entry.lot_number),
                            sku: entry.sku,
                            reason: entry.reason,
                            from_location: entry.from_location,
                            to_location: entry.to_location,
                            quantity: entry.quantity,
                            actor: entry.actor,
                            created_at: entry.timestamp,
                        })
                        .collect();
                    movements.set(converted_movements);
                }
                Err(e) => {
                    let error_message = format_error(&e);
                    error.set(Some(error_message));
                }
            }

            loading.set(false);
        });
    };

    // Load movements on mount
    use_effect(move || {
        fetch_movements();
    });

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",

            // Header with filters
            div {
                class: "mb-6",
                h2 {
                    class: "text-xl font-semibold text-gray-800 mb-4",
                    "Traceability"
                    if !movements().is_empty() {
                        span {
                            class: "ml-2 text-sm text-gray-500 font-normal",
                            "({movements().len()} movements)"
                        }
                    }
                }

                // Mode selector
                div {
                    class: "flex gap-2 mb-4",
                    button {
                        class: if trace_mode() == "movements" {
                            "px-4 py-2 bg-blue-600 text-white rounded-lg"
                        } else {
                            "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
                        },
                        onclick: move |_| {
                            trace_mode.set("movements".to_string());
                            trace_id.set(String::new());
                            fetch_movements();
                        },
                        "Movements"
                    }
                    button {
                        class: if trace_mode() == "serial" {
                            "px-4 py-2 bg-blue-600 text-white rounded-lg"
                        } else {
                            "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
                        },
                        onclick: move |_| {
                            trace_mode.set("serial".to_string());
                        },
                        "Serial Trace"
                    }
                    button {
                        class: if trace_mode() == "lot" {
                            "px-4 py-2 bg-blue-600 text-white rounded-lg"
                        } else {
                            "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300"
                        },
                        onclick: move |_| {
                            trace_mode.set("lot".to_string());
                        },
                        "Lot Trace"
                    }
                }

                // Filters based on mode
                if trace_mode() == "movements" {
                    div {
                        class: "flex gap-4",
                        input {
                            r#type: "text",
                            placeholder: "Filter by SKU...",
                            class: "flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{sku_filter}",
                            oninput: move |evt| {
                                sku_filter.set(evt.value().clone());
                            }
                        }
                        button {
                            class: "px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400",
                            disabled: loading(),
                            onclick: move |_| {
                                fetch_movements();
                            },
                            "Search"
                        }
                    }
                } else if trace_mode() == "serial" {
                    div {
                        class: "flex gap-4",
                        input {
                            r#type: "text",
                            placeholder: "Enter serial number...",
                            class: "flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{trace_id}",
                            oninput: move |evt| {
                                trace_id.set(evt.value().clone());
                            }
                        }
                        button {
                            class: "px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400",
                            disabled: loading() || trace_id().is_empty(),
                            onclick: move |_| {
                                fetch_serial_genealogy();
                            },
                            "Trace"
                        }
                    }
                } else if trace_mode() == "lot" {
                    div {
                        class: "flex gap-4",
                        input {
                            r#type: "text",
                            placeholder: "Enter lot number...",
                            class: "flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                            value: "{trace_id}",
                            oninput: move |evt| {
                                trace_id.set(evt.value().clone());
                            }
                        }
                        button {
                            class: "px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400",
                            disabled: loading() || trace_id().is_empty(),
                            onclick: move |_| {
                                fetch_lot_genealogy();
                            },
                            "Trace"
                        }
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
                        "Error loading traceability data"
                    }
                    p {
                        class: "text-sm mt-1",
                        "{err_msg}"
                    }
                    button {
                        class: "mt-2 text-sm text-red-600 hover:text-red-800 underline",
                        onclick: move |_| {
                            if trace_mode() == "movements" {
                                fetch_movements();
                            } else if trace_mode() == "serial" {
                                fetch_serial_genealogy();
                            } else {
                                fetch_lot_genealogy();
                            }
                        },
                        "Retry"
                    }
                }
            }

            // Empty state
            if !loading() && error().is_none() && movements().is_empty() {
                div {
                    class: "text-center py-12",
                    p {
                        class: "text-gray-500 text-lg",
                        if trace_mode() == "movements" {
                            "No movements found"
                        } else if trace_mode() == "serial" {
                            "No serial trace found"
                        } else {
                            "No lot trace found"
                        }
                    }
                    p {
                        class: "text-gray-400 text-sm mt-2",
                        if trace_mode() == "movements" {
                            "Try adjusting your filters or perform inventory operations"
                        } else {
                            "Enter a valid serial/lot number to trace"
                        }
                    }
                }
            }

            // Data table
            if !loading() && error().is_none() && !movements().is_empty() {
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
                    rows: movements().iter().map(|mov| {
                        let from_str = mov.from_location.as_ref()
                            .map(|loc| format!("{} / {} / {}", loc.warehouse, loc.zone, loc.bin))
                            .unwrap_or_default();
                        let to_str = mov.to_location.as_ref()
                            .map(|loc| format!("{} / {} / {}", loc.warehouse, loc.zone, loc.bin))
                            .unwrap_or_default();
                        vec![
                            mov.created_at.clone(),
                            mov.sku.clone(),
                            mov.reason.clone(),
                            from_str,
                            to_str,
                            mov.quantity.to_string(),
                            mov.actor.clone(),
                        ]
                    }).collect(),
                }
            }
        }
    }
}

/// Helper function to format GraphQL errors
fn format_error(e: &GraphQLError) -> String {
    match e {
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

        // Check for header
        assert!(html.contains("Traceability"));

        // Check for mode buttons
        assert!(html.contains("Movements"));
        assert!(html.contains("Serial Trace"));
        assert!(html.contains("Lot Trace"));

        // Check for search button
        assert!(html.contains("Search"));
    }

    #[test]
    fn inventory_movement_deserialization() {
        let json = r#"{
            "id": "mov-001",
            "sku": "TEST-001",
            "reason": "receipt",
            "fromLocation": null,
            "toLocation": {
                "warehouse": "WH-01",
                "zone": "A",
                "bin": "001"
            },
            "quantity": 100,
            "actor": "operator1",
            "createdAt": "2026-01-29T10:00:00Z"
        }"#;

        let movement: InventoryMovement = serde_json::from_str(json).unwrap();
        assert_eq!(movement.id, "mov-001");
        assert_eq!(movement.sku, "TEST-001");
        assert_eq!(movement.reason, "receipt");
        assert!(movement.from_location.is_none());
        assert!(movement.to_location.is_some());
        assert_eq!(movement.quantity, 100);
    }

    #[test]
    fn serial_genealogy_entry_deserialization() {
        let json = r#"{
            "serialNumber": "SN-12345",
            "sku": "TEST-001",
            "reason": "transfer",
            "fromLocation": {
                "warehouse": "WH-01",
                "zone": "A",
                "bin": "001"
            },
            "toLocation": {
                "warehouse": "WH-01",
                "zone": "B",
                "bin": "002"
            },
            "quantity": 1,
            "actor": "operator1",
            "timestamp": "2026-01-29T10:00:00Z"
        }"#;

        let entry: SerialGenealogyEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.serial_number, "SN-12345");
        assert_eq!(entry.sku, "TEST-001");
        assert_eq!(entry.reason, "transfer");
        assert_eq!(entry.quantity, 1);
    }

    #[test]
    fn lot_genealogy_entry_deserialization() {
        let json = r#"{
            "lotNumber": "LOT-2026-001",
            "sku": "TEST-001",
            "reason": "receipt",
            "fromLocation": null,
            "toLocation": {
                "warehouse": "WH-01",
                "zone": "A",
                "bin": "001"
            },
            "quantity": 100,
            "actor": "operator1",
            "timestamp": "2026-01-29T10:00:00Z"
        }"#;

        let entry: LotGenealogyEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.lot_number, "LOT-2026-001");
        assert_eq!(entry.sku, "TEST-001");
        assert_eq!(entry.quantity, 100);
    }

    #[test]
    fn format_error_validation() {
        let error = GraphQLError::Structured {
            category: ErrorCategory::Validation,
            message: "Invalid input".to_string(),
            field_errors: Some(vec![
                ui_foundation::FieldError {
                    field: "sku".to_string(),
                    message: "SKU cannot be empty".to_string(),
                },
            ]),
        };

        let formatted = format_error(&error);
        assert!(formatted.contains("Validation Error"));
        assert!(formatted.contains("Invalid input"));
        assert!(formatted.contains("sku: SKU cannot be empty"));
    }

    #[test]
    fn format_error_network() {
        let error = GraphQLError::Network("Connection failed".to_string());
        let formatted = format_error(&error);
        assert!(formatted.contains("Network Error"));
        assert!(formatted.contains("Connection failed"));
    }
}
