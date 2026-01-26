//! Language switcher component

use dioxus::prelude::*;
use shared::i18n::SupportedLocale;
use ui_design_system::{Button, ButtonVariant, Modal};

/// Hook to access locale signal
pub fn use_locale() -> Signal<SupportedLocale> {
    use_context()
}

/// Language switcher component
/// 
/// Displays a language icon button that opens a modal to select language preference.
#[component]
pub fn LanguageSwitcher() -> Element {
    let mut show_modal = use_signal(|| false);
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();

    rsx! {
        div {
            Button {
                variant: ButtonVariant::Secondary,
                class: Some("text-xs px-3 py-2 w-full".to_string()),
                onclick: move |_| show_modal.set(true),
                "🌐 Language"
            }
            Modal {
                open: show_modal(),
                title: {
                    if current_locale == SupportedLocale::English {
                        "Select Language"
                    } else {
                        "选择语言"
                    }.to_string()
                },
                onclose: move || show_modal.set(false),
                div {
                    class: "p-4 space-y-2",
                    Button {
                        variant: if current_locale == SupportedLocale::English {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Secondary
                        },
                        class: Some("w-full justify-start".to_string()),
                        onclick: move |_| {
                            use_locale().set(SupportedLocale::English);
                            show_modal.set(false);
                        },
                        "English"
                    }
                    Button {
                        variant: if current_locale == SupportedLocale::Chinese {
                            ButtonVariant::Primary
                        } else {
                            ButtonVariant::Secondary
                        },
                        class: Some("w-full justify-start".to_string()),
                        onclick: move |_| {
                            use_locale().set(SupportedLocale::Chinese);
                            show_modal.set(false);
                        },
                        "中文"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_switcher_renders() {
        // Note: This test requires context setup which is complex in unit tests
        // Integration tests would be more appropriate for component testing
        // This is a placeholder to show test structure
    }
}
