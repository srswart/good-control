//! Session state management

use dioxus::prelude::*;

/// Session state for the current user
#[derive(Debug, Clone, PartialEq)]
pub struct Session {
    /// User ID (if authenticated)
    pub user_id: Option<String>,
    /// Username (if authenticated)
    pub username: Option<String>,
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
    pub fn authenticated(user_id: String, username: String, roles: Vec<String>) -> Self {
        Self {
            user_id: Some(user_id),
            username: Some(username),
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
        );
        session.clear();
        assert!(!session.is_authenticated);
        assert!(session.user_id.is_none());
    }
}
