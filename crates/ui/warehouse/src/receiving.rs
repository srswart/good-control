//! Receiving workflow components

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use ui_design_system::{Table, TableColumn};
use ui_foundation::{graphql_endpoint, ErrorCategory, GraphQLClient, GraphQLError};

/// Receiving task status enum
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReceivingTaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl ReceivingTaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "Pending",
            Self::InProgress => "In Progress",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }
}

/// Receiving line data structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReceivingLine {
    pub id: String,
    #[serde(rename = "poLineId")]
    pub po_line_id: String,
    pub sku: String,
    #[serde(rename = "expectedQuantity")]
    pub expected_quantity: i32,
    #[serde(rename = "receivedQuantity")]
    pub received_quantity: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Receiving task data structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReceivingTask {
    pub id: String,
    #[serde(rename = "taskNumber")]
    pub task_number: String,
    #[serde(rename = "poId")]
    pub po_id: String,
    #[serde(rename = "poNumber")]
    pub po_number: String,
    pub status: ReceivingTaskStatus,
    pub lines: Vec<ReceivingLine>,
    #[serde(rename = "assignedTo")]
    pub assigned_to: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// Receiving tasks query response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ReceivingTasksQueryResponse {
    #[serde(rename = "receivingTasks")]
    pub receiving_tasks: Vec<ReceivingTask>,
}

/// Record receipt input for mutation
#[derive(Debug, Clone, Serialize)]
pub struct RecordReceiptInput {
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(rename = "lineId")]
    pub line_id: String,
    pub quantity: i32,
}

/// Record receipt mutation response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RecordReceiptMutationResponse {
    #[serde(rename = "recordReceipt")]
    pub record_receipt: ReceivingLine,
}

/// Receiving view component
///
/// Displays a table of receiving tasks with their status and progress.
/// Fetches data from GraphQL API and provides loading/error/empty states.
#[component]
pub fn ReceivingView() -> Element {
    let mut tasks = use_signal(|| Vec::<ReceivingTask>::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut status_filter = use_signal(|| String::from("all"));

    // Fetch receiving tasks from GraphQL
    let fetch_tasks = move || {
        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());

            let query = r#"
                query {
                    receivingTasks {
                        id
                        taskNumber
                        poId
                        poNumber
                        status
                        lines {
                            id
                            poLineId
                            sku
                            expectedQuantity
                            receivedQuantity
                            createdAt
                        }
                        assignedTo
                        createdAt
                        updatedAt
                    }
                }
            "#
            .to_string();

            match client
                .query::<ReceivingTasksQueryResponse>(&query, None)
                .await
            {
                Ok(response) => {
                    tasks.set(response.receiving_tasks);
                }
                Err(e) => {
                    let error_message = format_error(&e);
                    error.set(Some(error_message));
                }
            }

            loading.set(false);
        });
    };

    // Load tasks on mount
    use_effect(move || {
        fetch_tasks();
    });

    // Filter tasks by status
    let filtered_tasks = use_memo(move || {
        let status_val = status_filter();
        let all_tasks = tasks();

        if status_val == "all" {
            all_tasks
        } else {
            all_tasks
                .into_iter()
                .filter(|t| {
                    let task_status = match t.status {
                        ReceivingTaskStatus::Pending => "pending",
                        ReceivingTaskStatus::InProgress => "in_progress",
                        ReceivingTaskStatus::Completed => "completed",
                        ReceivingTaskStatus::Cancelled => "cancelled",
                    };
                    task_status == status_val
                })
                .collect()
        }
    });

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",

            // Header with filters
            div {
                class: "mb-6",
                h2 {
                    class: "text-xl font-semibold text-gray-800 mb-4",
                    "Receiving Tasks"
                    if !tasks().is_empty() {
                        span {
                            class: "ml-2 text-sm text-gray-500 font-normal",
                            "({tasks().len()} total)"
                        }
                    }
                }

                // Status filter
                div {
                    class: "flex gap-4 items-center",
                    label {
                        class: "text-sm font-medium text-gray-700",
                        "Filter by Status:"
                    }
                    select {
                        class: "px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500",
                        value: "{status_filter}",
                        onchange: move |evt| {
                            status_filter.set(evt.value().clone());
                        },
                        option { value: "all", "All Statuses" }
                        option { value: "pending", "Pending" }
                        option { value: "in_progress", "In Progress" }
                        option { value: "completed", "Completed" }
                        option { value: "cancelled", "Cancelled" }
                    }
                    button {
                        class: "px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400",
                        disabled: loading(),
                        onclick: move |_| {
                            fetch_tasks();
                        },
                        "Refresh"
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
                        "Error loading receiving tasks"
                    }
                    p {
                        class: "text-sm mt-1",
                        "{err_msg}"
                    }
                    button {
                        class: "mt-2 text-sm text-red-600 hover:text-red-800 underline",
                        onclick: move |_| {
                            fetch_tasks();
                        },
                        "Retry"
                    }
                }
            }

            // Empty state
            if !loading() && error().is_none() && filtered_tasks().is_empty() {
                div {
                    class: "text-center py-12",
                    p {
                        class: "text-gray-500 text-lg",
                        if status_filter() == "all" {
                            "No receiving tasks found"
                        } else {
                            "No tasks with this status"
                        }
                    }
                    p {
                        class: "text-gray-400 text-sm mt-2",
                        "Create a new receiving task to get started"
                    }
                }
            }

            // Data table
            if !loading() && error().is_none() && !filtered_tasks().is_empty() {
                Table {
                    columns: vec![
                        TableColumn { header: "Task #".to_string(), key: "task_number".to_string() },
                        TableColumn { header: "PO Number".to_string(), key: "po_number".to_string() },
                        TableColumn { header: "Lines".to_string(), key: "lines".to_string() },
                        TableColumn { header: "Progress".to_string(), key: "progress".to_string() },
                        TableColumn { header: "Status".to_string(), key: "status".to_string() },
                        TableColumn { header: "Assigned To".to_string(), key: "assigned_to".to_string() },
                        TableColumn { header: "Created".to_string(), key: "created".to_string() },
                    ],
                    rows: filtered_tasks().iter().map(|task| {
                        let total_expected: i32 = task.lines.iter().map(|l| l.expected_quantity).sum();
                        let total_received: i32 = task.lines.iter().map(|l| l.received_quantity).sum();
                        let progress = format!("{}/{}", total_received, total_expected);
                        let lines_count = task.lines.len().to_string();

                        vec![
                            task.task_number.clone(),
                            task.po_number.clone(),
                            lines_count,
                            progress,
                            task.status.as_str().to_string(),
                            task.assigned_to.clone().unwrap_or_else(|| "-".to_string()),
                            task.created_at.clone(),
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

/// Scan-first receiving component
///
/// Provides a mobile-optimized interface for scanning items during receiving.
/// Features large touch-friendly controls and fast feedback.
#[component]
pub fn ScanReceiving(task_id: String) -> Element {
    let mut scan_input = use_signal(|| String::new());
    let mut quantity_input = use_signal(|| String::from("1"));
    let mut loading = use_signal(|| false);
    let mut success_message = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut task = use_signal(|| None::<ReceivingTask>);
    let mut trigger_scan = use_signal(|| 0u32); // Counter to trigger scan

    // Clone task_id for use in closures
    let task_id_for_effect = task_id.clone();
    let task_id_for_scan = task_id.clone();

    // Load task on mount and when task_id changes
    use_effect(move || {
        let task_id_clone = task_id_for_effect.clone();
        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());

            let query = format!(
                r#"
                query {{
                    receivingTask(id: "{}") {{
                        id
                        taskNumber
                        poId
                        poNumber
                        status
                        lines {{
                            id
                            poLineId
                            sku
                            expectedQuantity
                            receivedQuantity
                            createdAt
                        }}
                        assignedTo
                        createdAt
                        updatedAt
                    }}
                }}
                "#,
                task_id_clone
            );

            #[derive(Debug, Clone, Deserialize)]
            struct TaskQueryResponse {
                #[serde(rename = "receivingTask")]
                receiving_task: Option<ReceivingTask>,
            }

            match client.query::<TaskQueryResponse>(&query, None).await {
                Ok(response) => {
                    task.set(response.receiving_task);
                }
                Err(e) => {
                    let error_message = format_error(&e);
                    error.set(Some(error_message));
                }
            }

            loading.set(false);
        });
    });

    // Handle scan submission via effect
    use_effect(move || {
        let trigger_val = trigger_scan();
        if trigger_val == 0 {
            return; // Skip initial render
        }

        let scan_val = scan_input();
        if scan_val.is_empty() {
            error.set(Some("Please enter or scan a SKU".to_string()));
            return;
        }

        // Find matching line by SKU
        if let Some(current_task) = task() {
            if let Some(line) = current_task.lines.iter().find(|l| l.sku == scan_val) {
                let line_id = line.id.clone();
                let task_id_clone = task_id_for_scan.clone();
                let qty_str = quantity_input();
                let qty: i32 = qty_str.parse().unwrap_or(1);

                spawn(async move {
                    loading.set(true);
                    error.set(None);
                    success_message.set(None);

                    let client = GraphQLClient::new(graphql_endpoint());

                    let mutation = format!(
                        r#"
                        mutation {{
                            recordReceipt(
                                taskId: "{}",
                                lineId: "{}",
                                quantity: {}
                            ) {{
                                id
                                sku
                                expectedQuantity
                                receivedQuantity
                            }}
                        }}
                        "#,
                        task_id_clone, line_id, qty
                    );

                    match client
                        .mutate::<RecordReceiptMutationResponse>(&mutation, None)
                        .await
                    {
                        Ok(response) => {
                            let line = response.record_receipt;
                            success_message.set(Some(format!(
                                "✓ Recorded {} units of {} ({}/{})",
                                qty, line.sku, line.received_quantity, line.expected_quantity
                            )));
                            // Clear inputs
                            scan_input.set(String::new());
                            quantity_input.set(String::from("1"));

                            // Refresh task data
                            let task_id_refresh = task_id_clone.clone();
                            spawn(async move {
                                let client = GraphQLClient::new(graphql_endpoint());

                                let query = format!(
                                    r#"
                                    query {{
                                        receivingTask(id: "{}") {{
                                            id
                                            taskNumber
                                            poId
                                            poNumber
                                            status
                                            lines {{
                                                id
                                                poLineId
                                                sku
                                                expectedQuantity
                                                receivedQuantity
                                                createdAt
                                            }}
                                            assignedTo
                                            createdAt
                                            updatedAt
                                        }}
                                    }}
                                    "#,
                                    task_id_refresh
                                );

                                #[derive(Debug, Clone, Deserialize)]
                                struct TaskQueryResponse {
                                    #[serde(rename = "receivingTask")]
                                    receiving_task: Option<ReceivingTask>,
                                }

                                if let Ok(response) = client.query::<TaskQueryResponse>(&query, None).await {
                                    task.set(response.receiving_task);
                                }
                            });
                        }
                        Err(e) => {
                            let error_message = format_error(&e);
                            error.set(Some(error_message));
                        }
                    }

                    loading.set(false);
                });
            } else {
                error.set(Some(format!("SKU '{}' not found in this receiving task", scan_val)));
            }
        }
    });

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-6 max-w-2xl mx-auto",

            // Header
            div {
                class: "mb-6",
                h2 {
                    class: "text-2xl font-bold text-gray-800 mb-2",
                    "Scan Receiving"
                }
                if let Some(current_task) = task() {
                    p {
                        class: "text-gray-600",
                        "Task: {current_task.task_number} | PO: {current_task.po_number}"
                    }
                }
            }

            // Success message
            if let Some(msg) = success_message() {
                div {
                    class: "bg-green-50 border-2 border-green-500 text-green-800 px-6 py-4 rounded-lg mb-6 text-lg font-semibold",
                    "{msg}"
                }
            }

            // Error message
            if let Some(err_msg) = error() {
                div {
                    class: "bg-red-50 border-2 border-red-500 text-red-800 px-6 py-4 rounded-lg mb-6",
                    p {
                        class: "font-semibold text-lg",
                        "Error"
                    }
                    p {
                        class: "mt-1",
                        "{err_msg}"
                    }
                }
            }

            // Scan input (large, touch-friendly)
            div {
                class: "mb-6",
                label {
                    class: "block text-lg font-semibold text-gray-700 mb-2",
                    "Scan or Enter SKU"
                }
                input {
                    r#type: "text",
                    placeholder: "Scan barcode or type SKU...",
                    class: "w-full px-6 py-4 text-2xl border-2 border-gray-300 rounded-lg focus:outline-none focus:ring-4 focus:ring-blue-500 focus:border-blue-500",
                    value: "{scan_input}",
                    oninput: move |evt| {
                        scan_input.set(evt.value().clone());
                        // Clear previous messages
                        error.set(None);
                        success_message.set(None);
                    },
                    onkeypress: move |evt| {
                        if evt.key() == Key::Enter {
                            trigger_scan.set(trigger_scan() + 1);
                        }
                    },
                    autofocus: true,
                }
            }

            // Quantity input (large, touch-friendly)
            div {
                class: "mb-6",
                label {
                    class: "block text-lg font-semibold text-gray-700 mb-2",
                    "Quantity"
                }
                input {
                    r#type: "number",
                    min: "1",
                    class: "w-full px-6 py-4 text-2xl border-2 border-gray-300 rounded-lg focus:outline-none focus:ring-4 focus:ring-blue-500 focus:border-blue-500",
                    value: "{quantity_input}",
                    oninput: move |evt| {
                        quantity_input.set(evt.value().clone());
                    },
                }
            }

            // Submit button (large, touch-friendly)
            button {
                class: "w-full px-8 py-6 text-2xl font-bold bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed",
                disabled: loading() || scan_input().is_empty(),
                onclick: move |_| {
                    trigger_scan.set(trigger_scan() + 1);
                },
                if loading() {
                    "Processing..."
                } else {
                    "Record Receipt"
                }
            }

            // Task lines summary
            if let Some(current_task) = task() {
                div {
                    class: "mt-8 border-t pt-6",
                    h3 {
                        class: "text-lg font-semibold text-gray-800 mb-4",
                        "Expected Items ({current_task.lines.len()})"
                    }
                    div {
                        class: "space-y-2",
                        for line in current_task.lines.iter() {
                            div {
                                class: if line.received_quantity >= line.expected_quantity {
                                    "flex justify-between items-center p-3 bg-green-50 border border-green-200 rounded-lg"
                                } else {
                                    "flex justify-between items-center p-3 bg-gray-50 border border-gray-200 rounded-lg"
                                },
                                div {
                                    p {
                                        class: "font-semibold text-gray-800",
                                        "{line.sku}"
                                    }
                                }
                                div {
                                    class: "text-right",
                                    p {
                                        class: if line.received_quantity >= line.expected_quantity {
                                            "text-green-700 font-semibold"
                                        } else {
                                            "text-gray-700"
                                        },
                                        "{line.received_quantity} / {line.expected_quantity}"
                                    }
                                    if line.received_quantity >= line.expected_quantity {
                                        p {
                                            class: "text-sm text-green-600",
                                            "✓ Complete"
                                        }
                                    }
                                }
                            }
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

    #[test]
    fn receiving_view_renders() {
        let mut vdom = VirtualDom::new(ReceivingView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);

        // Check for header
        assert!(html.contains("Receiving Tasks"));

        // Check for filter controls
        assert!(html.contains("Filter by Status"));
        assert!(html.contains("All Statuses"));

        // Check for refresh button
        assert!(html.contains("Refresh"));
    }

    #[test]
    fn receiving_task_status_as_str() {
        assert_eq!(ReceivingTaskStatus::Pending.as_str(), "Pending");
        assert_eq!(ReceivingTaskStatus::InProgress.as_str(), "In Progress");
        assert_eq!(ReceivingTaskStatus::Completed.as_str(), "Completed");
        assert_eq!(ReceivingTaskStatus::Cancelled.as_str(), "Cancelled");
    }

    #[test]
    fn receiving_task_deserialization() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "taskNumber": "RECV-001",
            "poId": "123e4567-e89b-12d3-a456-426614174001",
            "poNumber": "PO-2026-001",
            "status": "IN_PROGRESS",
            "lines": [
                {
                    "id": "123e4567-e89b-12d3-a456-426614174002",
                    "poLineId": "123e4567-e89b-12d3-a456-426614174003",
                    "sku": "TEST-001",
                    "expectedQuantity": 10,
                    "receivedQuantity": 8,
                    "createdAt": "2026-01-29T10:00:00Z"
                }
            ],
            "assignedTo": "operator1",
            "createdAt": "2026-01-29T09:00:00Z",
            "updatedAt": "2026-01-29T10:00:00Z"
        }"#;

        let task: ReceivingTask = serde_json::from_str(json).unwrap();
        assert_eq!(task.task_number, "RECV-001");
        assert_eq!(task.po_number, "PO-2026-001");
        assert_eq!(task.status, ReceivingTaskStatus::InProgress);
        assert_eq!(task.lines.len(), 1);
        assert_eq!(task.lines[0].sku, "TEST-001");
        assert_eq!(task.lines[0].expected_quantity, 10);
        assert_eq!(task.lines[0].received_quantity, 8);
    }

    #[test]
    fn receiving_tasks_query_response_deserialization() {
        let json = r#"{
            "receivingTasks": [
                {
                    "id": "123e4567-e89b-12d3-a456-426614174000",
                    "taskNumber": "RECV-001",
                    "poId": "123e4567-e89b-12d3-a456-426614174001",
                    "poNumber": "PO-2026-001",
                    "status": "PENDING",
                    "lines": [],
                    "assignedTo": null,
                    "createdAt": "2026-01-29T09:00:00Z",
                    "updatedAt": "2026-01-29T09:00:00Z"
                }
            ]
        }"#;

        let response: ReceivingTasksQueryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.receiving_tasks.len(), 1);
        assert_eq!(response.receiving_tasks[0].task_number, "RECV-001");
        assert_eq!(response.receiving_tasks[0].status, ReceivingTaskStatus::Pending);
    }

    #[test]
    fn format_error_network() {
        let error = GraphQLError::Network("Connection failed".to_string());
        let formatted = format_error(&error);
        assert!(formatted.contains("Network Error"));
        assert!(formatted.contains("Connection failed"));
    }
}
