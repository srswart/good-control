//! Demo component showing GraphQL client usage.
//!
//! This module demonstrates how to use the GraphQL client layer to query items
//! from the backend with proper error handling.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use ui_foundation::{graphql_endpoint, ErrorCategory, GraphQLClient, GraphQLError};

/// Item data from GraphQL query
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Item {
    pub sku: String,
    pub name: String,
    #[serde(rename = "nameZh")]
    pub name_zh: String,
    #[serde(rename = "itemType")]
    pub item_type: String,
    #[serde(rename = "trackingMode")]
    pub tracking_mode: String,
}

/// Edge in the connection
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ItemEdge {
    pub node: Item,
    pub cursor: String,
}

/// Items connection response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ItemsConnection {
    pub edges: Vec<ItemEdge>,
}

/// Items query response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ItemsQueryResponse {
    pub items: ItemsConnection,
}

/// Demo component that queries items from the backend
#[component]
pub fn GraphQLDemo() -> Element {
    let mut items = use_signal(|| Vec::<Item>::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let fetch_items = move |_| {
        spawn(async move {
            loading.set(true);
            error.set(None);

            // Create GraphQL client (in a real app, this would be provided via context)
            let client = GraphQLClient::new(graphql_endpoint());

            // GraphQL query using the new pagination structure
            let query = r#"
                query {
                    items {
                        edges {
                            node {
                                sku
                                name
                                nameZh
                                itemType
                                trackingMode
                            }
                            cursor
                        }
                    }
                }
            "#;

            match client.query::<ItemsQueryResponse>(query, None).await {
                Ok(response) => {
                    let item_list: Vec<Item> = response
                        .items
                        .edges
                        .into_iter()
                        .map(|edge| edge.node)
                        .collect();
                    items.set(item_list);
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

    rsx! {
        div { class: "p-4",
            h2 { class: "text-2xl font-bold mb-4", "GraphQL Client Demo" }
            
            button {
                class: "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mb-4",
                onclick: fetch_items,
                disabled: loading(),
                if loading() { "Loading..." } else { "Fetch Items" }
            }

            if let Some(err) = error() {
                div { class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4",
                    strong { "Error: " }
                    span { "{err}" }
                }
            }

            if !items().is_empty() {
                div { class: "overflow-x-auto",
                    table { class: "min-w-full bg-white border border-gray-300",
                        thead {
                            tr { class: "bg-gray-100",
                                th { class: "px-4 py-2 border", "SKU" }
                                th { class: "px-4 py-2 border", "Name (EN)" }
                                th { class: "px-4 py-2 border", "Name (ZH)" }
                                th { class: "px-4 py-2 border", "Type" }
                                th { class: "px-4 py-2 border", "Tracking" }
                            }
                        }
                        tbody {
                            for item in items() {
                                tr { key: "{item.sku}",
                                    td { class: "px-4 py-2 border", "{item.sku}" }
                                    td { class: "px-4 py-2 border", "{item.name}" }
                                    td { class: "px-4 py-2 border", "{item.name_zh}" }
                                    td { class: "px-4 py-2 border", "{item.item_type}" }
                                    td { class: "px-4 py-2 border", "{item.tracking_mode}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

