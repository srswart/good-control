//! Toast/notification component

use dioxus::prelude::*;

/// Toast notification level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastLevel {
    fn to_classes(&self) -> &'static str {
        match self {
            ToastLevel::Info => "bg-blue-50 border-blue-200 text-blue-800",
            ToastLevel::Success => "bg-green-50 border-green-200 text-green-800",
            ToastLevel::Warning => "bg-yellow-50 border-yellow-200 text-yellow-800",
            ToastLevel::Error => "bg-red-50 border-red-200 text-red-800",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            ToastLevel::Info => "ℹ",
            ToastLevel::Success => "✓",
            ToastLevel::Warning => "⚠",
            ToastLevel::Error => "✕",
        }
    }
}

/// Toast notification component
/// 
/// Displays temporary notifications (info, success, warning, error).
/// Typically auto-dismisses after a few seconds or can be manually closed.
#[component]
pub fn Toast(
    /// Whether the toast is visible
    visible: bool,
    /// Toast message
    message: String,
    /// Toast level (info, success, warning, error)
    #[props(default = ToastLevel::Info)]
    level: ToastLevel,
    /// Close handler
    onclose: EventHandler,
    /// Additional CSS classes
    class: Option<String>,
) -> Element {
    if !visible {
        return rsx! { };
    }

    let level_classes = level.to_classes();
    let icon = level.icon();
    let custom_classes = class.unwrap_or_default();
    let classes = format!("{} {}", level_classes, custom_classes);

    rsx! {
        div {
            class: "fixed top-4 right-4 z-50 max-w-sm w-full shadow-lg rounded-lg border p-4 {classes}",
            role: "alert",
            div {
                class: "flex items-start",
                span {
                    class: "mr-2 text-lg",
                    {icon}
                }
                p {
                    class: "flex-1",
                    {message}
                }
                button {
                    class: "ml-2 text-gray-400 hover:text-gray-600",
                    onclick: move |_| onclose.call(()),
                    "×"
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
    fn toast_renders_when_visible() {
        let mut vdom = VirtualDom::new(|| rsx! {
            Toast {
                visible: true,
                message: "Test message".to_string(),
                level: ToastLevel::Info,
                onclose: || {},
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Test message"));
        assert!(html.contains("bg-blue-50"));
    }

    #[test]
    fn toast_renders_success_level() {
        let mut vdom = VirtualDom::new(|| rsx! {
            Toast {
                visible: true,
                message: "Success!".to_string(),
                level: ToastLevel::Success,
                onclose: || {},
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("bg-green-50"));
        assert!(html.contains("Success!"));
    }

    #[test]
    fn toast_hidden_when_not_visible() {
        let mut vdom = VirtualDom::new(|| rsx! {
            Toast {
                visible: false,
                message: "Hidden message".to_string(),
                level: ToastLevel::Info,
                onclose: || {},
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(!html.contains("Hidden message"));
    }
}
