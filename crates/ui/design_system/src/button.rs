//! Button component with variants (primary, secondary, danger)

use dioxus::prelude::*;

/// Button variant styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
}

impl Default for ButtonVariant {
    fn default() -> Self {
        ButtonVariant::Primary
    }
}

impl ButtonVariant {
    fn to_classes(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2",
            ButtonVariant::Secondary => "px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2",
            ButtonVariant::Danger => "px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2",
        }
    }
}

/// Button component
#[component]
pub fn Button(
    /// Button text content
    children: Element,
    /// Button variant style
    #[props(default = ButtonVariant::Primary)]
    variant: ButtonVariant,
    /// Click handler
    onclick: Option<EventHandler<MouseEvent>>,
    /// Disabled state
    #[props(default = false)]
    disabled: bool,
    /// Additional CSS classes
    class: Option<String>,
) -> Element {
    let base_classes = variant.to_classes();
    let disabled_classes = if disabled {
        " opacity-50 cursor-not-allowed"
    } else {
        ""
    };
    let custom_classes = class.unwrap_or_default();
    let classes = format!("{} {}{}", base_classes, custom_classes, disabled_classes);

    rsx! {
        button {
            class: "{classes}",
            disabled: disabled,
            onclick: move |e| {
                if let Some(handler) = &onclick {
                    handler.call(e);
                }
            },
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutations;

    #[test]
    fn button_renders_with_primary_variant() {
        let mut vdom = VirtualDom::new(|| rsx! {
            Button {
                variant: ButtonVariant::Primary,
                "Click me"
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("bg-blue-600"));
        assert!(html.contains("Click me"));
    }

    #[test]
    fn button_renders_with_secondary_variant() {
        let mut vdom = VirtualDom::new(|| rsx! {
            Button {
                variant: ButtonVariant::Secondary,
                "Cancel"
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("bg-gray-200"));
        assert!(html.contains("Cancel"));
    }

    #[test]
    fn button_renders_with_danger_variant() {
        let mut vdom = VirtualDom::new(|| rsx! {
            Button {
                variant: ButtonVariant::Danger,
                "Delete"
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("bg-red-600"));
        assert!(html.contains("Delete"));
    }

    #[test]
    fn button_renders_disabled_state() {
        let mut vdom = VirtualDom::new(|| rsx! {
            Button {
                disabled: true,
                "Disabled"
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("opacity-50"));
        assert!(html.contains("disabled"));
    }
}
