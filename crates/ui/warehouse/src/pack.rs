//! Pack task components

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use ui_design_system::{Table, TableColumn};
use ui_foundation::{graphql_endpoint, ErrorCategory, GraphQLClient, GraphQLError};

/// Pack task status enum
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PackTaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl PackTaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "Pending",
            Self::InProgress => "In Progress",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }
}

/// Pack line data structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PackLine {
    pub id: String,
    #[serde(rename = "pickTaskId")]
    pub pick_task_id: String,
    pub sku: String,
    #[serde(rename = "quantityRequired")]
    pub quantity_required: i32,
    #[serde(rename = "quantityPacked")]
    pub quantity_packed: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Package data structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Package {
    pub id: String,
    #[serde(rename = "cartonType")]
    pub carton_type: String,
    #[serde(rename = "trackingNumber")]
    pub tracking_number: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Pack task data structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PackTask {
    pub id: String,
    #[serde(rename = "taskNumber")]
    pub task_number: String,
    pub status: PackTaskStatus,
    pub lines: Vec<PackLine>,
    pub packages: Vec<Package>,
    #[serde(rename = "assignedTo")]
    pub assigned_to: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// Pack tasks query response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PackTasksQueryResponse {
    #[serde(rename = "packTasks")]
    pub pack_tasks: Vec<PackTask>,
}

/// Verify pack item mutation response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct VerifyPackItemMutationResponse {
    #[serde(rename = "verifyPackItem")]
    pub verify_pack_item: PackLine,
}

/// Add package mutation response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AddPackageMutationResponse {
    #[serde(rename = "addPackage")]
    pub add_package: Package,
}

/// Pack view component
///
/// Displays a table of pack tasks with their status and progress.
/// Fetches data from GraphQL API and provides loading/error/empty states.
#[component]
pub fn PackView() -> Element {
    let mut tasks = use_signal(|| Vec::<PackTask>::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut status_filter = use_signal(|| String::from("all"));

    // Fetch pack tasks from GraphQL
    let fetch_tasks = move || {
        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());

            let query = if status_filter() == "all" {
                r#"
                    query GetPackTasks($limit: Int, $offset: Int) {
                        packTasks(limit: $limit, offset: $offset) {
                            id
                            taskNumber
                            status
                            lines {
                                id
                                pickTaskId
                                sku
                                quantityRequired
                                quantityPacked
                                createdAt
                            }
                            packages {
                                id
                                cartonType
                                trackingNumber
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
                    query GetPackTasksByStatus($status: PackTaskStatus!) {
                        packTasksByStatus(status: $status) {
                            id
                            taskNumber
                            status
                            lines {
                                id
                                pickTaskId
                                sku
                                quantityRequired
                                quantityPacked
                                createdAt
                            }
                            packages {
                                id
                                cartonType
                                trackingNumber
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

            match client.query::<PackTasksQueryResponse>(query, Some(variables)).await {
                Ok(response) => {
                    tasks.set(response.pack_tasks);
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load pack tasks: {}", e)));
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
    let calculate_progress = |task: &PackTask| -> (i32, i32) {
        let total_required: i32 = task.lines.iter().map(|l| l.quantity_required).sum();
        let total_packed: i32 = task.lines.iter().map(|l| l.quantity_packed).sum();
        (total_packed, total_required)
    };

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Pack Tasks"
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
                    "Loading pack tasks..."
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
                    "No pack tasks found"
                }
            }

            // Tasks table
            if !loading() && error().is_none() && !tasks().is_empty() {
                Table {
                    columns: vec![
                        TableColumn { header: "Task #".to_string(), key: "task_number".to_string() },
                        TableColumn { header: "Lines".to_string(), key: "lines".to_string() },
                        TableColumn { header: "Packages".to_string(), key: "packages".to_string() },
                        TableColumn { header: "Progress".to_string(), key: "progress".to_string() },
                        TableColumn { header: "Status".to_string(), key: "status".to_string() },
                        TableColumn { header: "Assigned To".to_string(), key: "assigned_to".to_string() },
                        TableColumn { header: "Created".to_string(), key: "created".to_string() },
                    ],
                    rows: tasks().iter().map(|task| {
                        let (packed, required) = calculate_progress(task);
                        vec![
                            task.task_number.clone(),
                            task.lines.len().to_string(),
                            task.packages.len().to_string(),
                            format!("{}/{}", packed, required),
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

/// Scan-first packing component
///
/// Provides a mobile-optimized interface for scanning items during packing.
/// Features large touch-friendly controls and fast feedback.
#[component]
pub fn ScanPacking(task_id: String) -> Element {
    let mut scan_input = use_signal(|| String::new());
    let mut package_input = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut success_message = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut task = use_signal(|| None::<PackTask>);
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
                    packTask(id: "{}") {{
                        id
                        taskNumber
                        status
                        lines {{
                            id
                            pickTaskId
                            sku
                            quantityRequired
                            quantityPacked
                            createdAt
                        }}
                        packages {{
                            id
                            cartonType
                            trackingNumber
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
                #[serde(rename = "packTask")]
                pack_task: Option<PackTask>,
            }

            match client.query::<TaskQueryResponse>(&query, None).await {
                Ok(response) => {
                    task.set(response.pack_task);
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

                spawn(async move {
                    loading.set(true);
                    error.set(None);
                    success_message.set(None);

                    let client = GraphQLClient::new(graphql_endpoint());

                    let mutation = format!(
                        r#"
                        mutation {{
                            verifyPackItem(
                                taskId: "{}",
                                lineId: "{}",
                                sku: "{}"
                            ) {{
                                id
                                sku
                                quantityRequired
                                quantityPacked
                            }}
                        }}
                        "#,
                        task_id_clone, line_id, scan_val
                    );

                    match client
                        .mutate::<VerifyPackItemMutationResponse>(&mutation, None)
                        .await
                    {
                        Ok(response) => {
                            let line = response.verify_pack_item;
                            success_message.set(Some(format!(
                                "✓ Verified {} ({}/{})",
                                line.sku, line.quantity_packed, line.quantity_required
                            )));
                            // Clear inputs
                            scan_input.set(String::new());

                            // Refresh task data
                            let task_id_refresh = task_id_clone.clone();
                            spawn(async move {
                                let client = GraphQLClient::new(graphql_endpoint());

                                let query = format!(
                                    r#"
                                    query {{
                                        packTask(id: "{}") {{
                                            id
                                            taskNumber
                                            status
                                            lines {{
                                                id
                                                pickTaskId
                                                sku
                                                quantityRequired
                                                quantityPacked
                                                createdAt
                                            }}
                                            packages {{
                                                id
                                                cartonType
                                                trackingNumber
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
                                    #[serde(rename = "packTask")]
                                    pack_task: Option<PackTask>,
                                }

                                if let Ok(response) = client.query::<TaskQueryResponse>(&query, None).await {
                                    task.set(response.pack_task);
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
                error.set(Some(format!("SKU '{}' not found in this pack task", scan_val)));
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
                    "Scan Packing"
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

            // Submit button (large, touch-friendly)
            button {
                class: "w-full px-8 py-6 text-2xl font-bold bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed mb-6",
                disabled: loading() || scan_input().is_empty(),
                onclick: move |_| {
                    trigger_scan.set(trigger_scan() + 1);
                },
                if loading() {
                    "Processing..."
                } else {
                    "Verify Item"
                }
            }

            // Add Package section
            div {
                class: "mb-6 border-t pt-6",
                h3 {
                    class: "text-lg font-semibold text-gray-800 mb-4",
                    "Add Package"
                }
                div {
                    class: "mb-4",
                    label {
                        class: "block text-lg font-semibold text-gray-700 mb-2",
                        "Carton Type"
                    }
                    input {
                        r#type: "text",
                        placeholder: "Enter carton type...",
                        class: "w-full px-6 py-4 text-2xl border-2 border-gray-300 rounded-lg focus:outline-none focus:ring-4 focus:ring-blue-500 focus:border-blue-500",
                        value: "{package_input}",
                        oninput: move |evt| {
                            package_input.set(evt.value().clone());
                        },
                    }
                }
                button {
                    class: "w-full px-8 py-6 text-2xl font-bold bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed",
                    disabled: loading() || package_input().is_empty(),
                    onclick: move |_| {
                        let carton_type = package_input();
                        let task_id_clone = task_id.clone();

                        spawn(async move {
                            loading.set(true);
                            error.set(None);
                            success_message.set(None);

                            let client = GraphQLClient::new(graphql_endpoint());

                            let mutation = format!(
                                r#"
                                mutation {{
                                    addPackage(
                                        taskId: "{}",
                                        cartonType: "{}",
                                        trackingNumber: null
                                    ) {{
                                        id
                                        cartonType
                                        trackingNumber
                                        createdAt
                                    }}
                                }}
                                "#,
                                task_id_clone, carton_type
                            );

                            match client
                                .mutate::<AddPackageMutationResponse>(&mutation, None)
                                .await
                            {
                                Ok(response) => {
                                    let package = response.add_package;
                                    success_message.set(Some(format!(
                                        "✓ Added package: {}",
                                        package.carton_type
                                    )));
                                    // Clear input
                                    package_input.set(String::new());

                                    // Refresh task data
                                    let task_id_refresh = task_id_clone.clone();
                                    spawn(async move {
                                        let client = GraphQLClient::new(graphql_endpoint());

                                        let query = format!(
                                            r#"
                                            query {{
                                                packTask(id: "{}") {{
                                                    id
                                                    taskNumber
                                                    status
                                                    lines {{
                                                        id
                                                        pickTaskId
                                                        sku
                                                        quantityRequired
                                                        quantityPacked
                                                        createdAt
                                                    }}
                                                    packages {{
                                                        id
                                                        cartonType
                                                        trackingNumber
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
                                            #[serde(rename = "packTask")]
                                            pack_task: Option<PackTask>,
                                        }

                                        if let Ok(response) = client.query::<TaskQueryResponse>(&query, None).await {
                                            task.set(response.pack_task);
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
                    },
                    if loading() {
                        "Processing..."
                    } else {
                        "Add Package"
                    }
                }
            }

            // Task lines summary
            if let Some(current_task) = task() {
                div {
                    class: "mt-8 border-t pt-6",
                    h3 {
                        class: "text-lg font-semibold text-gray-800 mb-4",
                        "Pack Items ({current_task.lines.len()})"
                    }
                    div {
                        class: "space-y-2",
                        for line in current_task.lines.iter() {
                            div {
                                class: if line.quantity_packed >= line.quantity_required {
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
                                        class: if line.quantity_packed >= line.quantity_required {
                                            "text-green-700 font-semibold"
                                        } else {
                                            "text-gray-700"
                                        },
                                        "{line.quantity_packed} / {line.quantity_required}"
                                    }
                                    if line.quantity_packed >= line.quantity_required {
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

                // Packages list
                if !current_task.packages.is_empty() {
                    div {
                        class: "mt-6 border-t pt-6",
                        h3 {
                            class: "text-lg font-semibold text-gray-800 mb-4",
                            "Packages ({current_task.packages.len()})"
                        }
                        div {
                            class: "space-y-2",
                            for package in current_task.packages.iter() {
                                div {
                                    class: "flex justify-between items-center p-3 bg-blue-50 border border-blue-200 rounded-lg",
                                    div {
                                        p {
                                            class: "font-semibold text-gray-800",
                                            "{package.carton_type}"
                                        }
                                        if let Some(tracking) = &package.tracking_number {
                                            p {
                                                class: "text-sm text-gray-600",
                                                "Tracking: {tracking}"
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
    fn pack_view_renders() {
        let mut vdom = VirtualDom::new(PackView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Pack Tasks"));
        assert!(html.contains("All"));
        assert!(html.contains("Pending"));
    }

    #[test]
    fn pack_task_status_as_str() {
        assert_eq!(PackTaskStatus::Pending.as_str(), "Pending");
        assert_eq!(PackTaskStatus::InProgress.as_str(), "In Progress");
        assert_eq!(PackTaskStatus::Completed.as_str(), "Completed");
        assert_eq!(PackTaskStatus::Cancelled.as_str(), "Cancelled");
    }
}
