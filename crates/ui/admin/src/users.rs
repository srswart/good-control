//! Admin user management UI.

use dioxus::prelude::*;
use serde::Deserialize;
use ui_foundation::{graphql_endpoint, GraphQLClient, GraphQLError};

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct RoleSummary {
    id: String,
    name: String,
    description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct UserRole {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct UserSummary {
    id: String,
    username: String,
    email: String,
    #[serde(rename = "isActive")]
    is_active: bool,
    roles: Vec<UserRole>,
}

#[derive(Debug, Clone, Deserialize)]
struct UsersResponse {
    users: Vec<UserSummary>,
}

#[derive(Debug, Clone, Deserialize)]
struct RolesResponse {
    roles: Vec<RoleSummary>,
}

#[derive(Debug, Clone, Deserialize)]
struct CreateUserResponse {
    #[serde(rename = "createUser")]
    create_user: UserSummary,
}

#[derive(Debug, Clone, Deserialize)]
struct AssignRoleResponse {
    #[serde(rename = "assignRole")]
    assign_role: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct RemoveRoleResponse {
    #[serde(rename = "removeRole")]
    remove_role: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ActivateUserResponse {
    #[serde(rename = "activateUser")]
    activate_user: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct DeactivateUserResponse {
    #[serde(rename = "deactivateUser")]
    deactivate_user: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct MockDataSummary {
    #[serde(rename = "inventoryItems")]
    inventory_items: i64,
    #[serde(rename = "inventoryLocations")]
    inventory_locations: i64,
    #[serde(rename = "inventoryLots")]
    inventory_lots: i64,
    #[serde(rename = "inventorySerialUnits")]
    inventory_serial_units: i64,
    #[serde(rename = "inventoryPositions")]
    inventory_positions: i64,
    #[serde(rename = "inventoryMovements")]
    inventory_movements: i64,
    #[serde(rename = "receivingTasks")]
    receiving_tasks: i64,
    #[serde(rename = "pickTasks")]
    pick_tasks: i64,
    #[serde(rename = "packTasks")]
    pack_tasks: i64,
    #[serde(rename = "shipTasks")]
    ship_tasks: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct ReloadMockDataResponse {
    #[serde(rename = "reloadMockData")]
    reload_mock_data: MockDataSummary,
}

fn format_error(error: GraphQLError) -> String {
    match error {
        GraphQLError::GraphQL(message) => message,
        GraphQLError::Structured { message, .. } => message,
        GraphQLError::Http { status, message } => format!("HTTP {}: {}", status, message),
        GraphQLError::Network(message) => message,
        GraphQLError::InvalidResponse(message) => message,
        GraphQLError::Parse(message) => message,
    }
}

#[component]
pub fn AdminUsersView() -> Element {
    let mut users = use_signal(Vec::<UserSummary>::new);
    let mut roles = use_signal(Vec::<RoleSummary>::new);
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut mock_data_loading = use_signal(|| false);
    let mut mock_data_summary = use_signal(|| None::<MockDataSummary>);

    let mut username = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut initial_role = use_signal(|| None::<String>);

    let fetch_data = std::rc::Rc::new({
        let mut users = users.clone();
        let mut roles = roles.clone();
        let mut loading = loading.clone();
        let mut error = error.clone();

        move || {
            let mut users = users.clone();
            let mut roles = roles.clone();
            let mut loading = loading.clone();
            let mut error = error.clone();

            spawn(async move {
                loading.set(true);
                error.set(None);

                let client = GraphQLClient::new(graphql_endpoint());
                let roles_query = r#"
                    query {
                        roles {
                            id
                            name
                            description
                        }
                    }
                "#;

                let users_query = r#"
                    query {
                        users {
                            id
                            username
                            email
                            isActive
                            roles {
                                id
                                name
                            }
                        }
                    }
                "#;

                let roles_result = client.query::<RolesResponse>(roles_query, None).await;
                let users_result = client.query::<UsersResponse>(users_query, None).await;

                match (roles_result, users_result) {
                    (Ok(roles_response), Ok(users_response)) => {
                        roles.set(roles_response.roles);
                        users.set(users_response.users);
                    }
                    (Err(err), _) | (_, Err(err)) => {
                        error.set(Some(format_error(err)));
                    }
                }

                loading.set(false);
            });
        }
    });

    {
        let fetch_data = fetch_data.clone();
        use_effect(move || {
            fetch_data();
        });
    }

    let on_create_user = {
        let fetch_data = fetch_data.clone();
        move |_| {
            let fetch_data = fetch_data.clone();
            let username_value = username();
            let email_value = email();
            let password_value = password();
            let selected_role = initial_role();

            let mut username = username.clone();
            let mut email = email.clone();
            let mut password = password.clone();
            let mut error = error.clone();
            let mut initial_role = initial_role.clone();

            spawn(async move {
                if username_value.trim().is_empty() || email_value.trim().is_empty() || password_value.is_empty() {
                    error.set(Some("Username, email, and password are required.".to_string()));
                    return;
                }

                let client = GraphQLClient::new(graphql_endpoint());
                let mutation = r#"
                    mutation CreateUser($username: String!, $email: String!, $password: String!) {
                        createUser(username: $username, email: $email, password: $password) {
                            id
                            username
                            email
                            isActive
                            roles { id name }
                        }
                    }
                "#;

                let variables = serde_json::json!({
                    "username": username_value,
                    "email": email_value,
                    "password": password_value,
                });

                match client.mutate::<CreateUserResponse>(mutation, Some(variables)).await {
                    Ok(response) => {
                        if let Some(role_id) = selected_role {
                            let assign_mutation = r#"
                                mutation AssignRole($userId: ID!, $roleId: ID!) {
                                    assignRole(userId: $userId, roleId: $roleId)
                                }
                            "#;
                            let assign_vars = serde_json::json!({
                                "userId": response.create_user.id,
                                "roleId": role_id,
                            });
                            let _ = client.mutate::<AssignRoleResponse>(assign_mutation, Some(assign_vars)).await;
                        }

                        username.set(String::new());
                        email.set(String::new());
                        password.set(String::new());
                        initial_role.set(None);
                        fetch_data();
                    }
                    Err(err) => {
                        error.set(Some(format_error(err)));
                    }
                }
            });
        }
    };

    let on_reload_mock_data = {
        let mut mock_data_loading = mock_data_loading.clone();
        let mut mock_data_summary = mock_data_summary.clone();
        let mut error = error.clone();
        move |_| {
            let mut mock_data_loading = mock_data_loading.clone();
            let mut mock_data_summary = mock_data_summary.clone();
            let mut error = error.clone();

            spawn(async move {
                mock_data_loading.set(true);
                error.set(None);

                let client = GraphQLClient::new(graphql_endpoint());
                let mutation = r#"
                    mutation {
                        reloadMockData {
                            inventoryItems
                            inventoryLocations
                            inventoryLots
                            inventorySerialUnits
                            inventoryPositions
                            inventoryMovements
                            receivingTasks
                            pickTasks
                            packTasks
                            shipTasks
                        }
                    }
                "#;

                match client.mutate::<ReloadMockDataResponse>(mutation, None).await {
                    Ok(response) => {
                        mock_data_summary.set(Some(response.reload_mock_data));
                    }
                    Err(err) => {
                        error.set(Some(format_error(err)));
                    }
                }

                mock_data_loading.set(false);
            });
        }
    };

    rsx! {
        div {
            class: "p-6 space-y-6",
            h1 { class: "text-2xl font-bold text-gray-900", "User Administration" }
            p { class: "text-sm text-gray-600", "Manage users, access status, and role assignments." }

            if let Some(err) = error() {
                div { class: "rounded-lg bg-red-50 border border-red-200 p-3 text-sm text-red-700", "{err}" }
            }

            div {
                class: "bg-white rounded-lg border border-gray-200 p-4 space-y-3",
                h2 { class: "text-lg font-semibold text-gray-900", "Create User" }
                div { class: "grid grid-cols-1 md:grid-cols-4 gap-3",
                    input {
                        class: "border border-gray-300 rounded-md px-3 py-2 text-sm",
                        r#type: "text",
                        placeholder: "Username",
                        value: "{username()}",
                        oninput: move |evt| username.set(evt.value()),
                    }
                    input {
                        class: "border border-gray-300 rounded-md px-3 py-2 text-sm",
                        r#type: "email",
                        placeholder: "Email",
                        value: "{email()}",
                        oninput: move |evt| email.set(evt.value()),
                    }
                    input {
                        class: "border border-gray-300 rounded-md px-3 py-2 text-sm",
                        r#type: "password",
                        placeholder: "Password",
                        value: "{password()}",
                        oninput: move |evt| password.set(evt.value()),
                    }
                    select {
                        class: "border border-gray-300 rounded-md px-3 py-2 text-sm",
                        onchange: move |evt| {
                            let value = evt.value();
                            if value.is_empty() {
                                initial_role.set(None);
                            } else {
                                initial_role.set(Some(value));
                            }
                        },
                        option { value: "", "Initial role (optional)" }
                        for role in roles().iter() {
                            option { value: "{role.id}", "{role.name}" }
                        }
                    }
                }
                button {
                    class: "px-4 py-2 bg-blue-600 text-white rounded-md text-sm hover:bg-blue-700",
                    onclick: on_create_user,
                    "Create user"
                }
            }

            div {
                class: "bg-white rounded-lg border border-gray-200 p-4 space-y-3",
                h2 { class: "text-lg font-semibold text-gray-900", "Mock Data" }
                p { class: "text-sm text-gray-600", "Reloading mock data will delete current inventory and task data before reseeding." }
                button {
                    class: "px-4 py-2 bg-amber-600 text-white rounded-md text-sm hover:bg-amber-700 disabled:bg-amber-300",
                    onclick: on_reload_mock_data,
                    disabled: mock_data_loading(),
                    if mock_data_loading() { "Reloading..." } else { "Reload mock data" }
                }
                if let Some(summary) = mock_data_summary() {
                    div { class: "text-sm text-gray-700",
                        p { "Inventory items: {summary.inventory_items}" }
                        p { "Locations: {summary.inventory_locations}" }
                        p { "Lots: {summary.inventory_lots}" }
                        p { "Serial units: {summary.inventory_serial_units}" }
                        p { "Positions: {summary.inventory_positions}" }
                        p { "Movements: {summary.inventory_movements}" }
                        p { "Receiving tasks: {summary.receiving_tasks}" }
                        p { "Pick tasks: {summary.pick_tasks}" }
                        p { "Pack tasks: {summary.pack_tasks}" }
                        p { "Ship tasks: {summary.ship_tasks}" }
                    }
                }
            }

            div {
                class: "bg-white rounded-lg border border-gray-200 p-4",
                h2 { class: "text-lg font-semibold text-gray-900 mb-3", "Users" }

                if loading() {
                    p { class: "text-sm text-gray-500", "Loading users..." }
                } else {
                    div { class: "overflow-x-auto",
                        table { class: "min-w-full text-sm text-left",
                            thead {
                                tr {
                                    th { class: "px-3 py-2 text-gray-500 uppercase tracking-wide text-xs", "User" }
                                    th { class: "px-3 py-2 text-gray-500 uppercase tracking-wide text-xs", "Email" }
                                    th { class: "px-3 py-2 text-gray-500 uppercase tracking-wide text-xs", "Status" }
                                    th { class: "px-3 py-2 text-gray-500 uppercase tracking-wide text-xs", "Roles" }
                                    th { class: "px-3 py-2 text-gray-500 uppercase tracking-wide text-xs", "Actions" }
                                }
                            }
                            tbody {
                                for user in users().iter() {
                                    {
                                        let user_id = std::rc::Rc::new(user.id.clone());
                                        let is_active = user.is_active;
                                        let user_roles = user.roles.clone();
                                        let mut error = error.clone();
                                        rsx! {
                                            tr { class: "border-t border-gray-100",
                                                td { class: "px-3 py-2 font-medium text-gray-900", "{user.username}" }
                                                td { class: "px-3 py-2 text-gray-600", "{user.email}" }
                                                td { class: "px-3 py-2",
                                                    if user.is_active {
                                                        span { class: "text-green-700 font-medium", "Active" }
                                                    } else {
                                                        span { class: "text-red-700 font-medium", "Inactive" }
                                                    }
                                                }
                                                td { class: "px-3 py-2",
                                                    div { class: "flex flex-wrap gap-3",
                                                        for role in roles().iter() {
                                                            {
                                                                let role_id = role.id.clone();
                                                                let role_name = role.name.clone();
                                                                let has_role = user_roles.iter().any(|r| r.id == role_id);
                                                                let mut error = error.clone();
                                                                let fetch_data = fetch_data.clone();
                                                                let user_id_value = user_id.clone();
                                                                rsx! {
                                                                    span { class: "flex items-center gap-1",
                                                                        input {
                                                                            r#type: "checkbox",
                                                                            checked: has_role,
                                                                            onclick: move |_| {
                                                                                let client = GraphQLClient::new(graphql_endpoint());
                                                                                let user_id = user_id_value.clone();
                                                                                let role_id = role_id.clone();
                                                                                let fetch_data = fetch_data.clone();
                                                                                spawn(async move {
                                                                                    let user_id = (*user_id).clone();
                                                                                    if has_role {
                                                                                        let mutation = r#"
                                                                                            mutation RemoveRole($userId: ID!, $roleId: ID!) {
                                                                                                removeRole(userId: $userId, roleId: $roleId)
                                                                                            }
                                                                                        "#;
                                                                                        let vars = serde_json::json!({
                                                                                            "userId": user_id,
                                                                                            "roleId": role_id,
                                                                                        });
                                                                                        if let Err(err) = client.mutate::<RemoveRoleResponse>(mutation, Some(vars)).await {
                                                                                            error.set(Some(format_error(err)));
                                                                                            return;
                                                                                        }
                                                                                    } else {
                                                                                        let mutation = r#"
                                                                                            mutation AssignRole($userId: ID!, $roleId: ID!) {
                                                                                                assignRole(userId: $userId, roleId: $roleId)
                                                                                            }
                                                                                        "#;
                                                                                        let vars = serde_json::json!({
                                                                                            "userId": user_id,
                                                                                            "roleId": role_id,
                                                                                        });
                                                                                        if let Err(err) = client.mutate::<AssignRoleResponse>(mutation, Some(vars)).await {
                                                                                            error.set(Some(format_error(err)));
                                                                                            return;
                                                                                        }
                                                                                    }
                                                                                    fetch_data();
                                                                                });
                                                                            },
                                                                        }
                                                                        label { class: "text-xs text-gray-600", "{role_name}" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                td { class: "px-3 py-2",
                                                    {
                                                        let fetch_data = fetch_data.clone();
                                                        let user_id_value = user_id.clone();
                                                        rsx! {
                                                            button {
                                                                class: "px-3 py-1 text-xs rounded-md border border-gray-300 hover:bg-gray-100",
                                                                onclick: move |_| {
                                                                    let client = GraphQLClient::new(graphql_endpoint());
                                                                    let user_id = user_id_value.clone();
                                                                    let mut error = error.clone();
                                                                    let fetch_data = fetch_data.clone();
                                                                    spawn(async move {
                                                                                    let user_id = (*user_id).clone();
                                                                        if is_active {
                                                                            let mutation = r#"
                                                                                mutation DeactivateUser($id: ID!) {
                                                                                    deactivateUser(id: $id)
                                                                                }
                                                                            "#;
                                                                            let vars = serde_json::json!({ "id": user_id });
                                                                            if let Err(err) = client.mutate::<DeactivateUserResponse>(mutation, Some(vars)).await {
                                                                                error.set(Some(format_error(err)));
                                                                                return;
                                                                            }
                                                                        } else {
                                                                            let mutation = r#"
                                                                                mutation ActivateUser($id: ID!) {
                                                                                    activateUser(id: $id)
                                                                                }
                                                                            "#;
                                                                            let vars = serde_json::json!({ "id": user_id });
                                                                            if let Err(err) = client.mutate::<ActivateUserResponse>(mutation, Some(vars)).await {
                                                                                error.set(Some(format_error(err)));
                                                                                return;
                                                                            }
                                                                        }
                                                                        fetch_data();
                                                                    });
                                                                },
                                                                if is_active { "Deactivate" } else { "Activate" }
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
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutations;

    #[test]
    fn admin_users_view_renders() {
        let mut vdom = VirtualDom::new(AdminUsersView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);

        assert!(html.contains("User Administration"));
        assert!(html.contains("Create User"));
        assert!(html.contains("Users"));
        assert!(html.contains("Mock Data"));
    }
}
