//! Pick task components

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use ui_design_system::{Table, TableColumn};
use ui_foundation::{graphql_endpoint, ErrorCategory, GraphQLClient, GraphQLError};

/// Pick task status enum
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PickTaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl PickTaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "Pending",
            Self::InProgress => "In Progress",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }
}

/// Pick task type enum
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PickTaskType {
    SingleOrder,
    Wave,
    Batch,
}

impl PickTaskType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::SingleOrder => "Single Order",
            Self::Wave => "Wave",
            Self::Batch => "Batch",
        }
    }
}

/// Pick line data structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PickLine {
    pub id: String,
    #[serde(rename = "orderLineId")]
    pub order_line_id: String,
    pub sku: String,
    #[serde(rename = "locationId")]
    pub location_id: String,
    #[serde(rename = "quantityToPick")]
    pub quantity_to_pick: i32,
    #[serde(rename = "quantityPicked")]
    pub quantity_picked: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Pick task data structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PickTask {
    pub id: String,
    #[serde(rename = "taskNumber")]
    pub task_number: String,
    #[serde(rename = "taskType")]
    pub task_type: PickTaskType,
    pub status: PickTaskStatus,
    pub lines: Vec<PickLine>,
    #[serde(rename = "assignedTo")]
    pub assigned_to: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// Pick tasks query response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PickTasksQueryResponse {
    #[serde(rename = "pickTasks")]
    pub pick_tasks: Vec<PickTask>,
}

/// Record pick mutation response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RecordPickMutationResponse {
    #[serde(rename = "recordPick")]
    pub record_pick: PickLine,
}

/// Pick view component
///
/// Displays a table of pick tasks with their status and progress.
/// Fetches data from GraphQL API and provides loading/error/empty states.
#[component]
pub fn PickView() -> Element {
    let mut tasks = use_signal(|| Vec::<PickTask>::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut status_filter = use_signal(|| String::from("all"));

    // Fetch pick tasks from GraphQL
    let fetch_tasks = move || {
        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());

            let query = if status_filter() == "all" {
                r#"
                    query GetPickTasks($limit: Int, $offset: Int) {
                        pickTasks(limit: $limit, offset: $offset) {
                            id
                            taskNumber
                            taskType
                            status
                            lines {
                                id
                                orderLineId
                                sku
                                locationId
                                quantityToPick
                                quantityPicked
                                createdAt
                            }
                            assignedTo
                            createdAt
                            updatedAt
                        }
                    }
                "#
            } else {
                r#"
                    query GetPickTasksByStatus($status: PickTaskStatus!) {
                        pickTasksByStatus(status: $status) {
                            id
                            taskNumber
                            taskType
                            status
                            lines {
                                id
                                orderLineId
                                sku
                                locationId
                                quantityToPick
                                quantityPicked
                                createdAt
                            }
                            assignedTo
                            createdAt
                            updatedAt
                        }
                    }
                "#
            };

            let variables = if status_filter() == "all" {
                serde_json::json!({
                    "limit": 100,
                    "offset": 0
                })
            } else {
                serde_json::json!({
                    "status": status_filter().to_uppercase()
                })
            };

            match client.query::<PickTasksQueryResponse>(query, Some(variables)).await {
                Ok(response) => {
                    tasks.set(response.pick_tasks);
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load pick tasks: {}", e)));
                    loading.set(false);
                }
            }
        });
    };

    // Fetch tasks on mount and when filter changes
    use_effect(move || {
        fetch_tasks();
    });

    // Calculate progress for each task
    let calculate_progress = |task: &PickTask| -> (i32, i32) {
        let total_required: i32 = task.lines.iter().map(|l| l.quantity_to_pick).sum();
        let total_picked: i32 = task.lines.iter().map(|l| l.quantity_picked).sum();
        (total_picked, total_required)
    };

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Pick Tasks"
            }

            // Status filter
            div {
                class: "mb-4 flex gap-2",
                button {
                    class: if status_filter() == "all" { "px-4 py-2 bg-blue-600 text-white rounded" } else { "px-4 py-2 bg-gray-200 text-gray-700 rounded" },
                    onclick: move |_| status_filter.set("all".to_string()),
                    "All"
                }
                button {
                    class: if status_filter() == "PENDING" { "px-4 py-2 bg-blue-600 text-white rounded" } else { "px-4 py-2 bg-gray-200 text-gray-700 rounded" },
                    onclick: move |_| status_filter.set("PENDING".to_string()),
                    "Pending"
                }
                button {
                    class: if status_filter() == "IN_PROGRESS" { "px-4 py-2 bg-blue-600 text-white rounded" } else { "px-4 py-2 bg-gray-200 text-gray-700 rounded" },
                    onclick: move |_| status_filter.set("IN_PROGRESS".to_string()),
                    "In Progress"
                }
                button {
                    class: if status_filter() == "COMPLETED" { "px-4 py-2 bg-blue-600 text-white rounded" } else { "px-4 py-2 bg-gray-200 text-gray-700 rounded" },
                    onclick: move |_| status_filter.set("COMPLETED".to_string()),
                    "Completed"
                }
            }

            // Loading state
            if loading() {
                div {
                    class: "text-center py-8 text-gray-600",
                    "Loading pick tasks..."
                }
            }

            // Error state
            if let Some(err) = error() {
                div {
                    class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4",
                    "{err}"
                }
            }

            // Empty state
            if !loading() && error().is_none() && tasks().is_empty() {
                div {
                    class: "text-center py-8 text-gray-500",
                    "No pick tasks found"
                }
            }

            // Tasks table
            if !loading() && error().is_none() && !tasks().is_empty() {
                Table {
                    columns: vec![
                        TableColumn { header: "Task #".to_string(), key: "task_number".to_string() },
                        TableColumn { header: "Type".to_string(), key: "type".to_string() },
                        TableColumn { header: "Lines".to_string(), key: "lines".to_string() },
                        TableColumn { header: "Progress".to_string(), key: "progress".to_string() },
                        TableColumn { header: "Status".to_string(), key: "status".to_string() },
                        TableColumn { header: "Assigned To".to_string(), key: "assigned_to".to_string() },
                        TableColumn { header: "Created".to_string(), key: "created".to_string() },
                    ],
                    rows: tasks().iter().map(|task| {
                        let (picked, required) = calculate_progress(task);
                        vec![
                            task.task_number.clone(),
                            task.task_type.as_str().to_string(),
                            task.lines.len().to_string(),
                            format!("{}/{}", picked, required),
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

/// Scan-first picking component
///
/// Provides a mobile-optimized interface for scanning items during picking.
/// Features large touch-friendly controls and fast feedback.
#[component]
pub fn ScanPicking(task_id: String) -> Element {
    let mut scan_input = use_signal(|| String::new());
    let mut quantity_input = use_signal(|| String::from("1"));
    let mut loading = use_signal(|| false);
    let mut success_message = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut task = use_signal(|| None::<PickTask>);
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
                    pickTask(id: "{}") {{
                        id
                        taskNumber
                        taskType
                        status
                        lines {{
                            id
                            orderLineId
                            sku
                            locationId
                            quantityToPick
                            quantityPicked
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
                #[serde(rename = "pickTask")]
                pick_task: Option<PickTask>,
            }

            match client.query::<TaskQueryResponse>(&query, None).await {
                Ok(response) => {
                    task.set(response.pick_task);
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
            error.set(Some("Please enter or scan a location".to_string()));
            return;
        }

        // Find matching line by location
        if let Some(current_task) = task() {
            if let Some(line) = current_task.lines.iter().find(|l| l.location_id == scan_val) {
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
                            recordPick(
                                taskId: "{}",
                                lineId: "{}",
                                quantity: {}
                            ) {{
                                id
                                sku
                                locationId
                                quantityToPick
                                quantityPicked
                            }}
                        }}
                        "#,
                        task_id_clone, line_id, qty
                    );

                    match client
                        .mutate::<RecordPickMutationResponse>(&mutation, None)
                        .await
                    {
                        Ok(response) => {
                            let line = response.record_pick;
                            success_message.set(Some(format!(
                                "✓ Picked {} units of {} from {} ({}/{})",
                                qty, line.sku, line.location_id, line.quantity_picked, line.quantity_to_pick
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
                                        pickTask(id: "{}") {{
                                            id
                                            taskNumber
                                            taskType
                                            status
                                            lines {{
                                                id
                                                orderLineId
                                                sku
                                                locationId
                                                quantityToPick
                                                quantityPicked
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
                                    #[serde(rename = "pickTask")]
                                    pick_task: Option<PickTask>,
                                }

                                if let Ok(response) = client.query::<TaskQueryResponse>(&query, None).await {
                                    task.set(response.pick_task);
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
                error.set(Some(format!("Location '{}' not found in this pick task", scan_val)));
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
                    "Scan Picking"
                }
                if let Some(current_task) = task() {
                    p {
                        class: "text-gray-600",
                        "Task: {current_task.task_number}"
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
                    "Scan or Enter Location"
                }
                input {
                    r#type: "text",
                    placeholder: "Scan barcode or type location...",
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
                    "Record Pick"
                }
            }

            // Task lines summary
            if let Some(current_task) = task() {
                div {
                    class: "mt-8 border-t pt-6",
                    h3 {
                        class: "text-lg font-semibold text-gray-800 mb-4",
                        "Pick Items ({current_task.lines.len()})"
                    }
                    div {
                        class: "space-y-2",
                        for line in current_task.lines.iter() {
                            div {
                                class: if line.quantity_picked >= line.quantity_to_pick {
                                    "flex justify-between items-center p-3 bg-green-50 border border-green-200 rounded-lg"
                                } else {
                                    "flex justify-between items-center p-3 bg-gray-50 border border-gray-200 rounded-lg"
                                },
                                div {
                                    p {
                                        class: "font-semibold text-gray-800",
                                        "{line.sku}"
                                    }
                                    p {
                                        class: "text-sm text-gray-600",
                                        "Location: {line.location_id}"
                                    }
                                }
                                div {
                                    class: "text-right",
                                    p {
                                        class: if line.quantity_picked >= line.quantity_to_pick {
                                            "text-green-700 font-semibold"
                                        } else {
                                            "text-gray-700"
                                        },
                                        "{line.quantity_picked} / {line.quantity_to_pick}"
                                    }
                                    if line.quantity_picked >= line.quantity_to_pick {
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
    fn pick_view_renders() {
        let mut vdom = VirtualDom::new(PickView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Pick Tasks"));
        assert!(html.contains("All"));
        assert!(html.contains("Pending"));
    }

    #[test]
    fn pick_task_status_as_str() {
        assert_eq!(PickTaskStatus::Pending.as_str(), "Pending");
        assert_eq!(PickTaskStatus::InProgress.as_str(), "In Progress");
        assert_eq!(PickTaskStatus::Completed.as_str(), "Completed");
        assert_eq!(PickTaskStatus::Cancelled.as_str(), "Cancelled");
    }

    #[test]
    fn pick_task_type_as_str() {
        assert_eq!(PickTaskType::SingleOrder.as_str(), "Single Order");
        assert_eq!(PickTaskType::Wave.as_str(), "Wave");
        assert_eq!(PickTaskType::Batch.as_str(), "Batch");
    }
}
