use dioxus::prelude::*;
use ui_app_shell::AppShell;
use ui_design_system::{Button, ButtonVariant};

mod demo;

fn main() {
    // Minimal tracing setup for visibility
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    launch(app);
}

static TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
fn app() -> Element {
    let mut show_demo = use_signal(|| false);

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        style { {"
            * { margin: 0; padding: 0; box-sizing: border-box; }
            html, body { margin: 0; padding: 0; width: 100%; height: 100%; overflow-x: hidden; }
        "} }
        if show_demo() {
            demo::ComponentShowcase {
                on_back: move || show_demo.set(false),
            }
        } else {
            AppShell {
                sidebar_footer: rsx! {
                    Button {
                        variant: ButtonVariant::Secondary,
                        class: Some("text-xs px-4 py-2".to_string()),
                        onclick: move |_| show_demo.set(true),
                        "DS Examples"
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutations;

    #[test]
    fn renders_main_app() {
        let mut vdom = VirtualDom::new(app);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Controlled Goods SCM"));
    }
}
