//! Validation error display component

use dioxus::prelude::*;

/// Validation error display component
/// 
/// Displays validation errors in a consistent, accessible format.
/// Used for form field errors and compliance blocking messages.
#[component]
pub fn ValidationError(
    /// Error message to display
    message: String,
    /// Additional CSS classes
    class: Option<String>,
) -> Element {
    let base_classes = "mt-1 text-sm text-red-600";
    let custom_classes = class.unwrap_or_default();
    let classes = format!("{} {}", base_classes, custom_classes);

    rsx! {
        p {
            class: "{classes}",
            role: "alert",
            {message}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutations;

    #[test]
    fn validation_error_renders_message() {
        let mut vdom = VirtualDom::new(|| rsx! {
            ValidationError {
                message: "This field is required".to_string()
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("This field is required"));
        assert!(html.contains("text-red-600"));
        assert!(html.contains("role=\"alert\""));
    }

    #[test]
    fn validation_error_has_accessible_role() {
        let mut vdom = VirtualDom::new(|| rsx! {
            ValidationError {
                message: "Error message".to_string()
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("role=\"alert\""));
    }
}
