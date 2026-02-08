//! Session state management

use dioxus::prelude::*;
use ui_foundation::graphql::{graphql_endpoint, GraphQLClient, GraphQLError};
use ui_foundation::graphql::auth_token::{auth_token, set_auth_token};
use crate::routes::Route;
use serde::{Deserialize, Serialize};

/// Session state for the current user
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    /// User ID (if authenticated)
    pub user_id: Option<String>,
    /// Username (if authenticated)
    pub username: Option<String>,
    /// Session token (if authenticated)
    pub token: Option<String>,
    /// User roles/permissions (to be expanded in ADV-SHELL-002)
    pub roles: Vec<String>,
    /// Whether the user is authenticated
    pub is_authenticated: bool,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            user_id: None,
            username: None,
            token: None,
            roles: vec![],
            is_authenticated: false,
        }
    }
}

impl Session {
    /// Create a new unauthenticated session
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an authenticated session
    pub fn authenticated(user_id: String, username: String, roles: Vec<String>, token: String) -> Self {
        Self {
            user_id: Some(user_id),
            username: Some(username),
            token: Some(token),
            roles,
            is_authenticated: true,
        }
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Clear session (logout)
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Determine if the session can access a given route.
    pub fn can_access_route(&self, route: &Route) -> bool {
        if !self.is_authenticated {
            return matches!(route, Route::Login {});
        }

        if self.has_role("admin") {
            return true;
        }

        match route {
            Route::Home {} => true,
            Route::Inventory {} => self.has_role("warehouse_operator")
                || self.has_role("warehouse_supervisor")
                || self.has_role("compliance_officer"),
            Route::Warehouse {} => self.has_role("warehouse_operator") || self.has_role("warehouse_supervisor"),
            Route::Assets {} => self.has_role("warehouse_supervisor"),
            Route::Compliance {} => self.has_role("compliance_officer"),
            Route::Orders {} => self.has_role("warehouse_supervisor"),
            Route::Logistics {} => self.has_role("warehouse_supervisor"),
            Route::Reporting {} => self.has_role("compliance_officer"),
            Route::Integrations {} => false,
            Route::Demo {} => self.has_role("warehouse_supervisor"),
            Route::NotFound {} => true,
            Route::Login {} => true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CurrentUserResponse {
    #[serde(rename = "currentUser")]
    current_user: CurrentUser,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct CurrentUser {
    id: String,
    username: String,
    roles: Vec<UserRole>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct UserRole {
    name: String,
}

/// Session state management hook
/// 
/// Provides access to session state throughout the application.
/// This is a minimal implementation; full session management will be added in ADV-SHELL-002.
pub fn use_session() -> Signal<Session> {
    use_context()
}

/// Session provider component
/// 
/// Wraps the app to provide session context.
/// Should be placed at the root of the application.
#[component]
pub fn SessionProvider(children: Element) -> Element {
    let session = use_signal(Session::new);

    provide_context(session);

    // Restore session from stored token
    let mut session_for_load = session.clone();
    use_effect(move || {
        if let Some(token) = auth_token() {
            spawn(async move {
                let client = GraphQLClient::new(graphql_endpoint());
                let query = r#"
                    query {
                        currentUser {
                            id
                            username
                            roles { name }
                        }
                    }
                "#;

                match client.query::<CurrentUserResponse>(query, None).await {
                    Ok(response) => {
                        let roles = response
                            .current_user
                            .roles
                            .into_iter()
                            .map(|role| role.name)
                            .collect();
                        session_for_load.set(Session::authenticated(
                            response.current_user.id,
                            response.current_user.username,
                            roles,
                            token,
                        ));
                    }
                    Err(GraphQLError::Http { status, .. }) if status == 401 || status == 403 => {
                        set_auth_token(None);
                    }
                    Err(_) => {
                        // Keep unauthenticated if request fails
                    }
                }
            });
        }
    });

    rsx! {
        {children}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_default_is_unauthenticated() {
        let session = Session::new();
        assert!(!session.is_authenticated);
        assert!(session.user_id.is_none());
    }

    #[test]
    fn session_authenticated_has_user_info() {
        let session = Session::authenticated(
            "user123".to_string(),
            "testuser".to_string(),
            vec!["admin".to_string()],
            "token".to_string(),
        );
        assert!(session.is_authenticated);
        assert_eq!(session.user_id, Some("user123".to_string()));
        assert_eq!(session.username, Some("testuser".to_string()));
        assert!(session.has_role("admin"));
    }

    #[test]
    fn session_clear_resets_to_default() {
        let mut session = Session::authenticated(
            "user123".to_string(),
            "testuser".to_string(),
            vec!["admin".to_string()],
            "token".to_string(),
        );
        session.clear();
        assert!(!session.is_authenticated);
        assert!(session.user_id.is_none());
    }
}
