//! Modal/dialog component

use dioxus::prelude::*;

/// Modal component
/// 
/// A modal dialog component with backdrop, title, and content area.
/// Supports clicking backdrop or close button to dismiss.
#[component]
pub fn Modal(
    /// Whether the modal is open
    open: bool,
    /// Modal title
    title: String,
    /// Modal content
    children: Element,
    /// Close handler (called when backdrop is clicked or close button is pressed)
    onclose: EventHandler,
    /// Additional CSS classes for the modal container
    class: Option<String>,
) -> Element {
    let custom_classes = class.unwrap_or_default();

    if !open {
        return rsx! { };
    }

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50",
            onclick: move |_| {
                // Close on backdrop click
                onclose.call(());
            },
            id: "modal-backdrop",
            div {
                class: "bg-white rounded-lg shadow-xl max-w-md w-full mx-4 {custom_classes}",
                onclick: |e| e.stop_propagation(),
                div {
                    class: "flex items-center justify-between p-4 border-b",
                    h2 {
                        class: "text-lg font-semibold",
                        {title}
                    }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| onclose.call(()),
                        "×"
                    }
                }
                div {
                    class: "p-4",
                    {children}
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
    fn modal_renders_when_open() {
        let mut vdom = VirtualDom::new(|| rsx! {
            Modal {
                open: true,
                title: "Test Modal".to_string(),
                onclose: || {},
                "Modal content"
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Test Modal"));
        assert!(html.contains("Modal content"));
    }

    #[test]
    fn modal_hidden_when_closed() {
        let mut vdom = VirtualDom::new(|| rsx! {
            Modal {
                open: false,
                title: "Test Modal".to_string(),
                onclose: || {},
                "Modal content"
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        // When closed, modal returns empty rsx, so content should not be in HTML
        assert!(!html.contains("Test Modal"));
    }
}
