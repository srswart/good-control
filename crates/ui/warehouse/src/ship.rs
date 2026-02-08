//! Ship task components

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use ui_design_system::{Table, TableColumn};
use ui_foundation::{graphql_endpoint, ErrorCategory, GraphQLClient, GraphQLError};

/// Ship task status enum
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShipTaskStatus {
    Pending,
    Staging,
    ComplianceCheck,
    ReadyToDispatch,
    Dispatched,
    Blocked,
    Cancelled,
}

impl ShipTaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "Pending",
            Self::Staging => "Staging",
            Self::ComplianceCheck => "Compliance Check",
            Self::ReadyToDispatch => "Ready to Dispatch",
            Self::Dispatched => "Dispatched",
            Self::Blocked => "Blocked",
            Self::Cancelled => "Cancelled",
        }
    }
}

/// Compliance block category enum
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceBlockCategory {
    MissingDocument,
    ExpiredDocument,
    MissingApproval,
    ExpiredApproval,
    ItemRestriction,
    QuarantineBlock,
    BatteryCompliance,
}

impl ComplianceBlockCategory {
    pub fn as_str(&self) -> &str {
        match self {
            Self::MissingDocument => "Missing Document",
            Self::ExpiredDocument => "Expired Document",
            Self::MissingApproval => "Missing Approval",
            Self::ExpiredApproval => "Expired Approval",
            Self::ItemRestriction => "Item Restriction",
            Self::QuarantineBlock => "Quarantine Block",
            Self::BatteryCompliance => "Battery Compliance",
        }
    }
}

/// Block reason data structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct BlockReason {
    pub category: ComplianceBlockCategory,
    pub message: String,
    pub details: Vec<(String, String)>,
}

/// Compliance block data structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ComplianceBlock {
    pub operation: String,
    pub blocked: bool,
    pub reasons: Vec<BlockReason>,
    #[serde(rename = "requiredActions")]
    pub required_actions: Vec<String>,
}

/// Ship line data structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ShipLine {
    pub id: String,
    #[serde(rename = "packTaskId")]
    pub pack_task_id: String,
    #[serde(rename = "packageIds")]
    pub package_ids: Vec<String>,
    #[serde(rename = "orderId")]
    pub order_id: String,
}

/// Ship task data structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ShipTask {
    pub id: String,
    #[serde(rename = "taskNumber")]
    pub task_number: String,
    pub status: ShipTaskStatus,
    pub lines: Vec<ShipLine>,
    #[serde(rename = "complianceCheck")]
    pub compliance_check: Option<ComplianceBlock>,
    pub carrier: Option<String>,
    #[serde(rename = "trackingNumber")]
    pub tracking_number: Option<String>,
    #[serde(rename = "dispatchedAt")]
    pub dispatched_at: Option<String>,
    #[serde(rename = "assignedTo")]
    pub assigned_to: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// Ship tasks query response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ShipTasksQueryResponse {
    #[serde(rename = "shipTasks")]
    pub ship_tasks: Vec<ShipTask>,
}

/// Ship view component
///
/// Displays a table of ship tasks with their status and compliance information.
/// Fetches data from GraphQL API and provides loading/error/empty states.
#[component]
pub fn ShipView() -> Element {
    let mut tasks = use_signal(|| Vec::<ShipTask>::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut status_filter = use_signal(|| String::from("all"));

    // Fetch ship tasks from GraphQL
    let fetch_tasks = move || {
        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());

            let query = if status_filter() == "all" {
                r#"
                    query GetShipTasks($limit: Int, $offset: Int) {
                        shipTasks(limit: $limit, offset: $offset) {
                            id
                            taskNumber
                            status
                            lines {
                                id
                                packTaskId
                                packageIds
                                orderId
                            }
                            complianceCheck {
                                operation
                                blocked
                                reasons {
                                    category
                                    message
                                    details
                                }
                                requiredActions
                            }
                            carrier
                            trackingNumber
                            dispatchedAt
                            assignedTo
                            createdAt
                            updatedAt
                        }
                    }
                "#
            } else {
                r#"
                    query GetShipTasksByStatus($status: ShipTaskStatus!) {
                        shipTasksByStatus(status: $status) {
                            id
                            taskNumber
                            status
                            lines {
                                id
                                packTaskId
                                packageIds
                                orderId
                            }
                            complianceCheck {
                                operation
                                blocked
                                reasons {
                                    category
                                    message
                                    details
                                }
                                requiredActions
                            }
                            carrier
                            trackingNumber
                            dispatchedAt
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

            match client.query::<ShipTasksQueryResponse>(query, Some(variables)).await {
                Ok(response) => {
                    tasks.set(response.ship_tasks);
                    loading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load ship tasks: {}", e)));
                    loading.set(false);
                }
            }
        });
    };

    // Fetch tasks on mount and when filter changes
    use_effect(move || {
        fetch_tasks();
    });

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Ship Tasks"
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
                    class: if status_filter() == "STAGING" { "px-4 py-2 bg-blue-600 text-white rounded" } else { "px-4 py-2 bg-gray-200 text-gray-700 rounded" },
                    onclick: move |_| status_filter.set("STAGING".to_string()),
                    "Staging"
                }
                button {
                    class: if status_filter() == "READY_TO_DISPATCH" { "px-4 py-2 bg-blue-600 text-white rounded" } else { "px-4 py-2 bg-gray-200 text-gray-700 rounded" },
                    onclick: move |_| status_filter.set("READY_TO_DISPATCH".to_string()),
                    "Ready to Dispatch"
                }
                button {
                    class: if status_filter() == "DISPATCHED" { "px-4 py-2 bg-blue-600 text-white rounded" } else { "px-4 py-2 bg-gray-200 text-gray-700 rounded" },
                    onclick: move |_| status_filter.set("DISPATCHED".to_string()),
                    "Dispatched"
                }
                button {
                    class: if status_filter() == "BLOCKED" { "px-4 py-2 bg-blue-600 text-white rounded" } else { "px-4 py-2 bg-gray-200 text-gray-700 rounded" },
                    onclick: move |_| status_filter.set("BLOCKED".to_string()),
                    "Blocked"
                }
            }

            // Loading state
            if loading() {
                div {
                    class: "text-center py-8 text-gray-600",
                    "Loading ship tasks..."
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
                    "No ship tasks found"
                }
            }

            // Tasks table
            if !loading() && error().is_none() && !tasks().is_empty() {
                Table {
                    columns: vec![
                        TableColumn { header: "Task #".to_string(), key: "task_number".to_string() },
                        TableColumn { header: "Lines".to_string(), key: "lines".to_string() },
                        TableColumn { header: "Status".to_string(), key: "status".to_string() },
                        TableColumn { header: "Carrier".to_string(), key: "carrier".to_string() },
                        TableColumn { header: "Tracking".to_string(), key: "tracking".to_string() },
                        TableColumn { header: "Assigned To".to_string(), key: "assigned_to".to_string() },
                        TableColumn { header: "Created".to_string(), key: "created".to_string() },
                    ],
                    rows: tasks().iter().map(|task| {
                        let is_blocked = task.compliance_check.as_ref().map_or(false, |c| c.blocked);
                        let status_display = if is_blocked {
                            format!("⚠️ {}", task.status.as_str())
                        } else {
                            task.status.as_str().to_string()
                        };

                        vec![
                            task.task_number.clone(),
                            task.lines.len().to_string(),
                            status_display,
                            task.carrier.clone().unwrap_or_else(|| "-".to_string()),
                            task.tracking_number.clone().unwrap_or_else(|| "-".to_string()),
                            task.assigned_to.clone().unwrap_or_else(|| "-".to_string()),
                            task.created_at.clone(),
                        ]
                    }).collect(),
                }
            }
        }
    }
}

/// Start ship task mutation response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StartShipTaskMutationResponse {
    #[serde(rename = "startShipTask")]
    pub start_ship_task: ShipTask,
}

/// Evaluate compliance mutation response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct EvaluateComplianceMutationResponse {
    #[serde(rename = "evaluateShipCompliance")]
    pub evaluate_ship_compliance: ShipTask,
}

/// Dispatch shipment mutation response
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DispatchShipmentMutationResponse {
    #[serde(rename = "dispatchShipment")]
    pub dispatch_shipment: ShipTask,
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

/// Scan-first shipping component
///
/// Provides a mobile-optimized interface for shipping workflow.
/// Features large touch-friendly controls and fast feedback.
#[component]
pub fn ScanShipping(task_id: String) -> Element {
    let mut carrier_input = use_signal(|| String::new());
    let mut tracking_input = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut success_message = use_signal(|| None::<String>);
    let mut error = use_signal(|| None::<String>);
    let mut task = use_signal(|| None::<ShipTask>);
    let mut trigger_dispatch = use_signal(|| 0u32); // Counter to trigger dispatch

    // Clone task_id for use in closures
    let task_id_for_effect = task_id.clone();
    let task_id_for_start = task_id.clone();
    let task_id_for_evaluate = task_id.clone();
    let task_id_for_dispatch = task_id.clone();

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
                    shipTask(id: "{}") {{
                        id
                        taskNumber
                        status
                        lines {{
                            id
                            packTaskId
                            packageIds
                            orderId
                        }}
                        complianceCheck {{
                            operation
                            blocked
                            reasons {{
                                category
                                message
                                details
                            }}
                            requiredActions
                        }}
                        carrier
                        trackingNumber
                        dispatchedAt
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
                #[serde(rename = "shipTask")]
                ship_task: Option<ShipTask>,
            }

            match client.query::<TaskQueryResponse>(&query, None).await {
                Ok(response) => {
                    task.set(response.ship_task);
                }
                Err(e) => {
                    let error_message = format_error(&e);
                    error.set(Some(error_message));
                }
            }

            loading.set(false);
        });
    });

    // Handle dispatch submission via effect
    use_effect(move || {
        let trigger_val = trigger_dispatch();
        if trigger_val == 0 {
            return; // Skip initial render
        }

        let carrier_val = carrier_input();
        let tracking_val = tracking_input();

        if carrier_val.is_empty() || tracking_val.is_empty() {
            error.set(Some("Please enter both carrier and tracking number".to_string()));
            return;
        }

        let task_id_clone = task_id_for_dispatch.clone();

        spawn(async move {
            loading.set(true);
            error.set(None);
            success_message.set(None);

            let client = GraphQLClient::new(graphql_endpoint());

            let mutation = format!(
                r#"
                mutation {{
                    dispatchShipment(
                        taskId: "{}",
                        carrier: "{}",
                        trackingNumber: "{}"
                    ) {{
                        id
                        taskNumber
                        status
                        carrier
                        trackingNumber
                        dispatchedAt
                    }}
                }}
                "#,
                task_id_clone, carrier_val, tracking_val
            );

            match client
                .mutate::<DispatchShipmentMutationResponse>(&mutation, None)
                .await
            {
                Ok(response) => {
                    let ship_task = response.dispatch_shipment;
                    success_message.set(Some(format!(
                        "✓ Shipment dispatched via {} - Tracking: {}",
                        ship_task.carrier.unwrap_or_default(),
                        ship_task.tracking_number.unwrap_or_default()
                    )));
                    // Clear inputs
                    carrier_input.set(String::new());
                    tracking_input.set(String::new());

                    // Refresh task data
                    let task_id_refresh = task_id_clone.clone();
                    spawn(async move {
                        let client = GraphQLClient::new(graphql_endpoint());

                        let query = format!(
                            r#"
                            query {{
                                shipTask(id: "{}") {{
                                    id
                                    taskNumber
                                    status
                                    lines {{
                                        id
                                        packTaskId
                                        packageIds
                                        orderId
                                    }}
                                    complianceCheck {{
                                        operation
                                        blocked
                                        reasons {{
                                            category
                                            message
                                            details
                                        }}
                                        requiredActions
                                    }}
                                    carrier
                                    trackingNumber
                                    dispatchedAt
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
                            #[serde(rename = "shipTask")]
                            ship_task: Option<ShipTask>,
                        }

                        if let Ok(response) = client.query::<TaskQueryResponse>(&query, None).await {
                            task.set(response.ship_task);
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
    });

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-6 max-w-2xl mx-auto",

            // Header
            div {
                class: "mb-6",
                h2 {
                    class: "text-2xl font-bold text-gray-800 mb-2",
                    "Scan Shipping"
                }
                if let Some(current_task) = task() {
                    p {
                        class: "text-gray-600",
                        "Task: {current_task.task_number}"
                    }
                    p {
                        class: "text-sm text-gray-500",
                        "Status: {current_task.status.as_str()}"
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

            // Compliance block warning (if blocked)
            if let Some(current_task) = task() {
                if let Some(compliance) = &current_task.compliance_check {
                    if compliance.blocked {
                        div {
                            class: "bg-red-50 border-2 border-red-600 px-6 py-4 rounded-lg mb-6",
                            p {
                                class: "font-bold text-xl text-red-800 mb-3",
                                "⚠️ COMPLIANCE BLOCK"
                            }
                            div {
                                class: "space-y-3",
                                for reason in compliance.reasons.iter() {
                                    div {
                                        class: "bg-white border border-red-300 rounded p-3",
                                        p {
                                            class: "font-semibold text-red-700 text-lg",
                                            "{reason.category.as_str()}"
                                        }
                                        p {
                                            class: "text-red-600 mt-1",
                                            "{reason.message}"
                                        }
                                        if !reason.details.is_empty() {
                                            div {
                                                class: "mt-2 text-sm text-gray-700",
                                                for (key, value) in reason.details.iter() {
                                                    p {
                                                        "{key}: {value}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if !compliance.required_actions.is_empty() {
                                div {
                                    class: "mt-4 bg-yellow-50 border border-yellow-300 rounded p-3",
                                    p {
                                        class: "font-semibold text-yellow-800 mb-2",
                                        "Required Actions:"
                                    }
                                    ul {
                                        class: "list-disc list-inside text-yellow-700 space-y-1",
                                        for action in compliance.required_actions.iter() {
                                            li {
                                                "{action}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Action buttons based on status
            if let Some(current_task) = task() {
                match current_task.status {
                    ShipTaskStatus::Pending => {
                        rsx! {
                            button {
                                class: "w-full px-8 py-6 text-2xl font-bold bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed mb-6",
                                disabled: loading(),
                                onclick: move |_| {
                                    let task_id_clone = task_id_for_start.clone();
                                    spawn(async move {
                                        loading.set(true);
                                        error.set(None);
                                        success_message.set(None);

                                        let client = GraphQLClient::new(graphql_endpoint());

                                        let mutation = format!(
                                            r#"
                                            mutation {{
                                                startShipTask(
                                                    taskId: "{}",
                                                    operator: "warehouse-operator"
                                                ) {{
                                                    id
                                                    taskNumber
                                                    status
                                                }}
                                            }}
                                            "#,
                                            task_id_clone
                                        );

                                        match client.mutate::<StartShipTaskMutationResponse>(&mutation, None).await {
                                            Ok(response) => {
                                                success_message.set(Some(format!(
                                                    "✓ Started staging for task {}",
                                                    response.start_ship_task.task_number
                                                )));
                                                task.set(Some(response.start_ship_task));
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
                                    "Start Staging"
                                }
                            }
                        }
                    }
                    ShipTaskStatus::Staging => {
                        rsx! {
                            button {
                                class: "w-full px-8 py-6 text-2xl font-bold bg-purple-600 text-white rounded-lg hover:bg-purple-700 disabled:bg-gray-400 disabled:cursor-not-allowed mb-6",
                                disabled: loading(),
                                onclick: move |_| {
                                    let task_id_clone = task_id_for_evaluate.clone();
                                    spawn(async move {
                                        loading.set(true);
                                        error.set(None);
                                        success_message.set(None);

                                        let client = GraphQLClient::new(graphql_endpoint());

                                        let mutation = format!(
                                            r#"
                                            mutation {{
                                                evaluateShipCompliance(taskId: "{}") {{
                                                    id
                                                    taskNumber
                                                    status
                                                    complianceCheck {{
                                                        operation
                                                        blocked
                                                        reasons {{
                                                            category
                                                            message
                                                            details
                                                        }}
                                                        requiredActions
                                                    }}
                                                }}
                                            }}
                                            "#,
                                            task_id_clone
                                        );

                                        match client.mutate::<EvaluateComplianceMutationResponse>(&mutation, None).await {
                                            Ok(response) => {
                                                let ship_task = response.evaluate_ship_compliance;
                                                let is_blocked = ship_task.compliance_check.as_ref().map_or(false, |c| c.blocked);
                                                if is_blocked {
                                                    success_message.set(Some("Compliance check complete - BLOCKED".to_string()));
                                                } else {
                                                    success_message.set(Some("✓ Compliance check passed - Ready to dispatch".to_string()));
                                                }
                                                task.set(Some(ship_task));
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
                                    "Evaluate Compliance"
                                }
                            }
                        }
                    }
                    ShipTaskStatus::ComplianceCheck | ShipTaskStatus::ReadyToDispatch => {
                        rsx! {
                            // Carrier input
                            div {
                                class: "mb-6",
                                label {
                                    class: "block text-lg font-semibold text-gray-700 mb-2",
                                    "Carrier"
                                }
                                input {
                                    r#type: "text",
                                    placeholder: "Enter carrier name...",
                                    class: "w-full px-6 py-4 text-2xl border-2 border-gray-300 rounded-lg focus:outline-none focus:ring-4 focus:ring-blue-500 focus:border-blue-500",
                                    value: "{carrier_input}",
                                    oninput: move |evt| {
                                        carrier_input.set(evt.value().clone());
                                        error.set(None);
                                        success_message.set(None);
                                    },
                                    autofocus: true,
                                }
                            }

                            // Tracking number input
                            div {
                                class: "mb-6",
                                label {
                                    class: "block text-lg font-semibold text-gray-700 mb-2",
                                    "Tracking Number"
                                }
                                input {
                                    r#type: "text",
                                    placeholder: "Enter tracking number...",
                                    class: "w-full px-6 py-4 text-2xl border-2 border-gray-300 rounded-lg focus:outline-none focus:ring-4 focus:ring-blue-500 focus:border-blue-500",
                                    value: "{tracking_input}",
                                    oninput: move |evt| {
                                        tracking_input.set(evt.value().clone());
                                        error.set(None);
                                        success_message.set(None);
                                    },
                                    onkeypress: move |evt| {
                                        if evt.key() == Key::Enter {
                                            trigger_dispatch.set(trigger_dispatch() + 1);
                                        }
                                    },
                                }
                            }

                            // Dispatch button
                            button {
                                class: "w-full px-8 py-6 text-2xl font-bold bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed mb-6",
                                disabled: loading() || carrier_input().is_empty() || tracking_input().is_empty(),
                                onclick: move |_| {
                                    trigger_dispatch.set(trigger_dispatch() + 1);
                                },
                                if loading() {
                                    "Processing..."
                                } else {
                                    "Dispatch Shipment"
                                }
                            }
                        }
                    }
                    ShipTaskStatus::Dispatched => {
                        rsx! {
                            div {
                                class: "bg-green-50 border-2 border-green-500 px-6 py-4 rounded-lg mb-6",
                                p {
                                    class: "font-bold text-xl text-green-800 mb-2",
                                    "✓ Shipment Dispatched"
                                }
                                if let Some(carrier) = &current_task.carrier {
                                    p {
                                        class: "text-green-700 text-lg",
                                        "Carrier: {carrier}"
                                    }
                                }
                                if let Some(tracking) = &current_task.tracking_number {
                                    p {
                                        class: "text-green-700 text-lg",
                                        "Tracking: {tracking}"
                                    }
                                }
                                if let Some(dispatched) = &current_task.dispatched_at {
                                    p {
                                        class: "text-green-600 text-sm mt-2",
                                        "Dispatched at: {dispatched}"
                                    }
                                }
                            }
                        }
                    }
                    ShipTaskStatus::Blocked => {
                        rsx! {
                            div {
                                class: "bg-red-50 border-2 border-red-600 px-6 py-4 rounded-lg mb-6",
                                p {
                                    class: "font-bold text-xl text-red-800",
                                    "⚠️ Shipment Blocked - Cannot Proceed"
                                }
                                p {
                                    class: "text-red-600 mt-2",
                                    "Please resolve compliance issues before dispatching."
                                }
                            }
                        }
                    }
                    ShipTaskStatus::Cancelled => {
                        rsx! {
                            div {
                                class: "bg-gray-50 border-2 border-gray-400 px-6 py-4 rounded-lg mb-6",
                                p {
                                    class: "font-bold text-xl text-gray-700",
                                    "Task Cancelled"
                                }
                            }
                        }
                    }
                }
            }

            // Task lines summary
            if let Some(current_task) = task() {
                div {
                    class: "mt-8 border-t pt-6",
                    h3 {
                        class: "text-lg font-semibold text-gray-800 mb-4",
                        "Shipment Lines ({current_task.lines.len()})"
                    }
                    div {
                        class: "space-y-2",
                        for line in current_task.lines.iter() {
                            div {
                                class: "flex justify-between items-center p-3 bg-gray-50 border border-gray-200 rounded-lg",
                                div {
                                    p {
                                        class: "font-semibold text-gray-800",
                                        "Order: {line.order_id}"
                                    }
                                    p {
                                        class: "text-sm text-gray-600",
                                        "Packages: {line.package_ids.len()}"
                                    }
                                }
                                div {
                                    class: "text-right text-sm text-gray-600",
                                    for (idx, pkg_id) in line.package_ids.iter().enumerate() {
                                        p {
                                            "Pkg {idx + 1}: {pkg_id}"
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
    fn ship_view_renders() {
        let mut vdom = VirtualDom::new(ShipView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Ship Tasks"));
        assert!(html.contains("All"));
        assert!(html.contains("Pending"));
    }

    #[test]
    fn ship_task_status_as_str() {
        assert_eq!(ShipTaskStatus::Pending.as_str(), "Pending");
        assert_eq!(ShipTaskStatus::Staging.as_str(), "Staging");
        assert_eq!(ShipTaskStatus::ComplianceCheck.as_str(), "Compliance Check");
        assert_eq!(ShipTaskStatus::ReadyToDispatch.as_str(), "Ready to Dispatch");
        assert_eq!(ShipTaskStatus::Dispatched.as_str(), "Dispatched");
        assert_eq!(ShipTaskStatus::Blocked.as_str(), "Blocked");
        assert_eq!(ShipTaskStatus::Cancelled.as_str(), "Cancelled");
    }

    #[test]
    fn compliance_block_category_as_str() {
        assert_eq!(ComplianceBlockCategory::MissingDocument.as_str(), "Missing Document");
        assert_eq!(ComplianceBlockCategory::ExpiredDocument.as_str(), "Expired Document");
        assert_eq!(ComplianceBlockCategory::MissingApproval.as_str(), "Missing Approval");
        assert_eq!(ComplianceBlockCategory::ExpiredApproval.as_str(), "Expired Approval");
        assert_eq!(ComplianceBlockCategory::ItemRestriction.as_str(), "Item Restriction");
        assert_eq!(ComplianceBlockCategory::QuarantineBlock.as_str(), "Quarantine Block");
        assert_eq!(ComplianceBlockCategory::BatteryCompliance.as_str(), "Battery Compliance");
    }
}
