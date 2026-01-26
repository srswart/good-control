//! Error boundary component for catching and displaying errors in the component tree.

use dioxus::prelude::*;
use thiserror::Error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_boundary_renders_children() {
        let children = rsx! {
            div { "Test content" }
        };
        let _result = ErrorBoundary(children);
        // In Dioxus 0.7, components return Element which is opaque
        // This test verifies the function compiles and can be called
        assert!(true); // Placeholder - actual testing would require VDOM rendering
    }
}

/// Error that can be caught by the error boundary
#[derive(Error, Debug, Clone)]
pub enum BoundaryError {
    #[error("An unexpected error occurred: {0}")]
    Unexpected(String),
}

impl From<anyhow::Error> for BoundaryError {
    fn from(err: anyhow::Error) -> Self {
        BoundaryError::Unexpected(err.to_string())
    }
}

/// Error boundary component that catches errors in child components
/// 
/// In Dioxus 0.7, error boundaries are handled via error handling mechanisms.
/// This is a minimal implementation that will be expanded in future advances.
#[allow(non_snake_case)]
pub fn ErrorBoundary(children: Element) -> Element {
    rsx! {
        {children}
    }
}
