//! Form input components (text input, textarea)

use dioxus::prelude::*;

/// Text input component
#[component]
pub fn TextInput(
    /// Input value (controlled)
    value: String,
    /// Change handler
    oninput: EventHandler<FormEvent>,
    /// Placeholder text
    placeholder: Option<String>,
    /// Input type (text, email, password, etc.)
    #[props(default = String::from("text"))]
    r#type: String,
    /// Disabled state
    #[props(default = false)]
    disabled: bool,
    /// Additional CSS classes
    class: Option<String>,
    /// Input name attribute
    name: Option<String>,
    /// Required field indicator
    #[props(default = false)]
    required: bool,
) -> Element {
    let base_classes = "w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500";
    let disabled_classes = if disabled {
        " bg-gray-100 cursor-not-allowed"
    } else {
        ""
    };
    let custom_classes = class.unwrap_or_default();
    let classes = format!("{} {}{}", base_classes, custom_classes, disabled_classes);

    rsx! {
        input {
            class: "{classes}",
            r#type: r#type,
            value: "{value}",
            placeholder: placeholder,
            disabled: disabled,
            required: required,
            name: name,
            oninput: move |e| oninput.call(e),
        }
    }
}

/// Textarea component
#[component]
pub fn Textarea(
    /// Textarea value (controlled)
    value: String,
    /// Change handler
    oninput: EventHandler<FormEvent>,
    /// Placeholder text
    placeholder: Option<String>,
    /// Number of rows
    #[props(default = 4)]
    rows: u32,
    /// Disabled state
    #[props(default = false)]
    disabled: bool,
    /// Additional CSS classes
    class: Option<String>,
    /// Textarea name attribute
    name: Option<String>,
    /// Required field indicator
    #[props(default = false)]
    required: bool,
) -> Element {
    let base_classes = "w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500";
    let disabled_classes = if disabled {
        " bg-gray-100 cursor-not-allowed"
    } else {
        ""
    };
    let custom_classes = class.unwrap_or_default();
    let classes = format!("{} {}{}", base_classes, custom_classes, disabled_classes);

    rsx! {
        textarea {
            class: "{classes}",
            rows: "{rows}",
            value: "{value}",
            placeholder: placeholder,
            disabled: disabled,
            required: required,
            name: name,
            oninput: move |e| oninput.call(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutations;

    #[test]
    fn text_input_renders_with_value() {
        let mut vdom = VirtualDom::new(|| rsx! {
            TextInput {
                value: "test value".to_string(),
                oninput: |_| {},
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("test value"));
        assert!(html.contains("border-gray-300"));
    }

    #[test]
    fn text_input_renders_with_placeholder() {
        let mut vdom = VirtualDom::new(|| rsx! {
            TextInput {
                value: "".to_string(),
                placeholder: Some("Enter text".to_string()),
                oninput: |_| {},
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Enter text"));
    }

    #[test]
    fn textarea_renders_with_value() {
        let mut vdom = VirtualDom::new(|| rsx! {
            Textarea {
                value: "textarea content".to_string(),
                oninput: |_| {},
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("textarea content"));
    }

    #[test]
    fn text_input_renders_disabled_state() {
        let mut vdom = VirtualDom::new(|| rsx! {
            TextInput {
                value: "".to_string(),
                disabled: true,
                oninput: |_| {},
            }
        });
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("bg-gray-100"));
        assert!(html.contains("disabled"));
    }
}
