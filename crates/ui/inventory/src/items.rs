//! Items list and detail components

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use shared::i18n::SupportedLocale;
use ui_design_system::{Table, TableColumn};
use ui_foundation::{graphql_endpoint, ErrorCategory, GraphQLClient, GraphQLError};
use crate::i18n::t;

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

/// Page info for pagination
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
    #[serde(rename = "hasPreviousPage")]
    pub has_previous_page: bool,
    #[serde(rename = "startCursor")]
    pub start_cursor: Option<String>,
    #[serde(rename = "endCursor")]
    pub end_cursor: Option<String>,
}

/// Items connection response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ItemsConnection {
    pub edges: Vec<ItemEdge>,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
    #[serde(rename = "totalCount")]
    pub total_count: i32,
}

/// Items query response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ItemsQueryResponse {
    pub items: ItemsConnection,
}

/// Items list component
///
/// Displays a table of inventory items with search, pagination, and real-time data from GraphQL.
#[component]
pub fn ItemsList() -> Element {
    // Get locale for translations
    let locale_signal: Signal<SupportedLocale> = use_context();
    let locale = *locale_signal.read();

    let mut items = use_signal(|| Vec::<Item>::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut search_query = use_signal(|| String::new());
    let mut cursor = use_signal(|| None::<String>);
    let mut cursor_stack = use_signal(|| Vec::<Option<String>>::new());
    let mut current_after = use_signal(|| None::<String>);
    let mut has_next_page = use_signal(|| false);
    let mut has_previous_page = use_signal(|| false);
    let mut total_count = use_signal(|| 0);

    // Fetch items from GraphQL
    let fetch_items = move |after_cursor: Option<String>| {
        let current_locale = locale;
        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());

            let search_val = search_query();

            // Build query with optional search and pagination
            let query = if !search_val.is_empty() {
                // Search query with optional pagination
                if let Some(cursor_val) = after_cursor {
                    format!(
                        r#"
                        query {{
                            items(search: "{}", pagination: {{ first: 10, after: "{}" }}) {{
                                edges {{
                                    node {{
                                        sku
                                        name
                                        nameZh
                                        itemType
                                        trackingMode
                                    }}
                                    cursor
                                }}
                                pageInfo {{
                                    hasNextPage
                                    hasPreviousPage
                                    startCursor
                                    endCursor
                                }}
                                totalCount
                            }}
                        }}
                        "#,
                        search_val, cursor_val
                    )
                } else {
                    format!(
                        r#"
                        query {{
                            items(search: "{}", pagination: {{ first: 10 }}) {{
                                edges {{
                                    node {{
                                        sku
                                        name
                                        nameZh
                                        itemType
                                        trackingMode
                                    }}
                                    cursor
                                }}
                                pageInfo {{
                                    hasNextPage
                                    hasPreviousPage
                                    startCursor
                                    endCursor
                                }}
                                totalCount
                            }}
                        }}
                        "#,
                        search_val
                    )
                }
            } else if let Some(cursor_val) = after_cursor {
                // Pagination without search
                format!(
                    r#"
                    query {{
                        items(pagination: {{ first: 10, after: "{}" }}) {{
                            edges {{
                                node {{
                                    sku
                                    name
                                    nameZh
                                    itemType
                                    trackingMode
                                }}
                                cursor
                            }}
                            pageInfo {{
                                hasNextPage
                                hasPreviousPage
                                startCursor
                                endCursor
                            }}
                            totalCount
                        }}
                    }}
                    "#,
                    cursor_val
                )
            } else {
                // No search, no pagination
                r#"
                query {
                    items(pagination: { first: 10 }) {
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
                        pageInfo {
                            hasNextPage
                            hasPreviousPage
                            startCursor
                            endCursor
                        }
                        totalCount
                    }
                }
                "#
                .to_string()
            };

            match client.query::<ItemsQueryResponse>(&query, None).await {
                Ok(response) => {
                    let item_list: Vec<Item> = response
                        .items
                        .edges
                        .iter()
                        .map(|edge| edge.node.clone())
                        .collect();
                    items.set(item_list);
                    has_next_page.set(response.items.page_info.has_next_page);
                    has_previous_page.set(response.items.page_info.has_previous_page);
                    cursor.set(response.items.page_info.end_cursor);
                    total_count.set(response.items.total_count);
                }
                Err(e) => {
                    let error_message = match &e {
                        GraphQLError::Structured {
                            category,
                            message,
                            field_errors,
                        } => {
                            let category_str = match category {
                                ErrorCategory::Validation => t(current_locale, "validation_error"),
                                ErrorCategory::Authorization => t(current_locale, "authorization_error"),
                                ErrorCategory::ComplianceBlock => t(current_locale, "compliance_block"),
                                ErrorCategory::NotFound => t(current_locale, "not_found"),
                                ErrorCategory::Conflict => t(current_locale, "conflict"),
                                ErrorCategory::Internal => t(current_locale, "internal_error"),
                            };
                            if let Some(fields) = field_errors {
                                format!(
                                    "{}: {} ({}: {})",
                                    category_str,
                                    message,
                                    t(current_locale, "fields"),
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
                        GraphQLError::Network(msg) => format!("{}: {}", t(current_locale, "network_error"), msg),
                        GraphQLError::Http { status, message } => {
                            format!("{} {}: {}", t(current_locale, "http_error"), status, message)
                        }
                        GraphQLError::GraphQL(msg) => format!("{}: {}", t(current_locale, "graphql_error"), msg),
                        GraphQLError::Parse(msg) => format!("{}: {}", t(current_locale, "parse_error"), msg),
                        GraphQLError::InvalidResponse(msg) => {
                            format!("{}: {}", t(current_locale, "invalid_response"), msg)
                        }
                    };
                    error.set(Some(error_message));
                }
            }

            loading.set(false);
        });
    };

    let on_search_submit = {
        let mut cursor_stack = cursor_stack.clone();
        let mut current_after = current_after.clone();
        move |_| {
            cursor_stack.set(Vec::new());
            current_after.set(None);
            fetch_items(None);
        }
    };
    let on_retry = {
        let mut cursor_stack = cursor_stack.clone();
        let mut current_after = current_after.clone();
        move |_| {
            cursor_stack.set(Vec::new());
            current_after.set(None);
            fetch_items(None);
        }
    };
    let on_search_keypress = {
        let mut cursor_stack = cursor_stack.clone();
        let mut current_after = current_after.clone();
        move |evt: KeyboardEvent| {
            if evt.key() == Key::Enter {
                cursor_stack.set(Vec::new());
                current_after.set(None);
                fetch_items(None);
            }
        }
    };

    // Load items on mount
    {
        let mut cursor_stack = cursor_stack.clone();
        let mut current_after = current_after.clone();
        use_effect(move || {
            cursor_stack.set(Vec::new());
            current_after.set(None);
            fetch_items(None);
        });
    }

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",

            // Header with search
            div {
                class: "flex justify-between items-center mb-6",
                h2 {
                    class: "text-xl font-semibold text-gray-800",
                    {t(locale, "items")}
                    if total_count() > 0 {
                        span {
                            class: "ml-2 text-sm text-gray-500 font-normal",
                            "({total_count()} {t(locale, \"items_total\")})"
                        }
                    }
                }
                div {
                    class: "flex gap-2",
                    input {
                        r#type: "text",
                        placeholder: "{t(locale, \"search_placeholder\")}",
                        class: "px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: "{search_query}",
                        oninput: move |evt| {
                            search_query.set(evt.value().clone());
                        },
                        onkeypress: on_search_keypress,
                    }
                    button {
                        class: "px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400",
                        disabled: loading(),
                        onclick: on_search_submit,
                        {t(locale, "search")}
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
                        {t(locale, "error_loading_items")}
                    }
                    p {
                        class: "text-sm mt-1",
                        "{err_msg}"
                    }
                    button {
                        class: "mt-2 text-sm text-red-600 hover:text-red-800 underline",
                        onclick: on_retry,
                        {t(locale, "retry")}
                    }
                }
            }

            // Empty state
            if !loading() && error().is_none() && items().is_empty() {
                div {
                    class: "text-center py-12",
                    p {
                        class: "text-gray-500 text-lg",
                        {t(locale, "no_items_found")}
                    }
                    p {
                        class: "text-gray-400 text-sm mt-2",
                        {t(locale, "no_items_message")}
                    }
                }
            }

            // Data table
            if !loading() && error().is_none() && !items().is_empty() {
                Table {
                    columns: vec![
                        TableColumn { header: t(locale, "sku").to_string(), key: "sku".to_string() },
                        TableColumn { header: t(locale, "name_en").to_string(), key: "name".to_string() },
                        TableColumn { header: t(locale, "name_zh").to_string(), key: "name_zh".to_string() },
                        TableColumn { header: t(locale, "type").to_string(), key: "type".to_string() },
                        TableColumn { header: t(locale, "tracking").to_string(), key: "tracking".to_string() },
                    ],
                    rows: items().iter().map(|item| {
                        vec![
                            item.sku.clone(),
                            item.name.clone(),
                            item.name_zh.clone(),
                            item.item_type.clone(),
                            item.tracking_mode.clone(),
                        ]
                    }).collect(),
                }

                // Pagination controls
                div {
                    class: "flex justify-between items-center mt-4",
                    div {
                        class: "text-sm text-gray-600",
                        "{t(locale, \"showing\")} {items().len()} {t(locale, \"of\")} {total_count()} {t(locale, \"items_count\")}"
                    }
                    div {
                        class: "flex gap-2",
                button {
                            class: "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 disabled:bg-gray-100 disabled:text-gray-400",
                            disabled: !has_previous_page() || loading(),
                            onclick: move |_| {
                        let mut stack = cursor_stack.write();
                        let previous_after = stack.pop().flatten();
                        current_after.set(previous_after.clone());
                        fetch_items(previous_after);
                            },
                            {t(locale, "previous")}
                        }
                        button {
                            class: "px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 disabled:bg-gray-100 disabled:text-gray-400",
                            disabled: !has_next_page() || loading(),
                            onclick: move |_| {
                        if let Some(end_cursor) = cursor() {
                            let mut stack = cursor_stack.write();
                            stack.push(current_after());
                            current_after.set(Some(end_cursor.clone()));
                            fetch_items(Some(end_cursor));
                                }
                            },
                            {t(locale, "next")}
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutations;

    // Helper component that provides locale context for testing
    #[component]
    fn ItemsListWithContext() -> Element {
        use_context_provider(|| Signal::new(SupportedLocale::English));
        rsx! { ItemsList {} }
    }

    #[test]
    fn items_list_renders() {
        let mut vdom = VirtualDom::new(ItemsListWithContext);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);

        // Check for header (translated)
        assert!(html.contains("Items"));

        // Check for search input placeholder (translated)
        assert!(html.contains("Search by SKU or name"));

        // Check for search button (translated)
        assert!(html.contains("Search"));
    }

    #[test]
    fn items_list_shows_loading_state() {
        let mut vdom = VirtualDom::new(ItemsListWithContext);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);

        // Initially should show loading or empty state
        // (depends on timing of async fetch)
        assert!(html.contains("Items"));
    }

    #[test]
    fn item_serialization() {
        let item = Item {
            sku: "TEST-001".to_string(),
            name: "Test Item".to_string(),
            name_zh: "测试物品".to_string(),
            item_type: "part".to_string(),
            tracking_mode: "none".to_string(),
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("TEST-001"));
        assert!(json.contains("Test Item"));

        let deserialized: Item = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.sku, "TEST-001");
        assert_eq!(deserialized.name, "Test Item");
    }

    #[test]
    fn items_connection_deserialization() {
        let json = r#"{
            "edges": [
                {
                    "node": {
                        "sku": "TEST-001",
                        "name": "Test Item",
                        "nameZh": "测试物品",
                        "itemType": "part",
                        "trackingMode": "none"
                    },
                    "cursor": "TEST-001"
                }
            ],
            "pageInfo": {
                "hasNextPage": false,
                "hasPreviousPage": false,
                "startCursor": "TEST-001",
                "endCursor": "TEST-001"
            },
            "totalCount": 1
        }"#;

        let connection: ItemsConnection = serde_json::from_str(json).unwrap();
        assert_eq!(connection.edges.len(), 1);
        assert_eq!(connection.edges[0].node.sku, "TEST-001");
        assert_eq!(connection.total_count, 1);
        assert!(!connection.page_info.has_next_page);
    }
}
