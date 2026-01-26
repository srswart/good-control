//! App shell layout component

use dioxus::prelude::*;
use crate::nav::Navigation;
use crate::routes::Route;
use crate::LayoutContext;
use crate::LanguageSwitcher;
use crate::language::use_locale;
use crate::i18n::t;

/// App shell layout component
/// 
/// Provides the main application layout with header, navigation sidebar, and content area.
/// This component is used as a layout route in the router.
#[component]
pub fn AppLayout() -> Element {
    // Get layout context from AppShell
    let context_signal = use_context::<Signal<LayoutContext>>();
    let ctx = context_signal.read();
    let items = ctx.nav_items.clone().unwrap_or_else(crate::nav::default_nav_items);
    let header_content = ctx.header_content.clone();
    let sidebar_footer = ctx.sidebar_footer.clone();
    
    // Get current locale for translations
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();

    rsx! {
        div {
            class: "min-h-screen bg-gray-50 flex flex-col",
            style: "margin: 0; padding: 0; width: 100%;",
            // Header
            header {
                class: "bg-gradient-to-r from-gray-900 to-gray-800 text-white shadow-lg border-b-2 border-gray-700",
                div {
                    class: "max-w-full mx-auto px-6 sm:px-8 lg:px-10 py-5",
                    div {
                        class: "flex justify-between items-center",
                        div {
                            class: "flex items-center space-x-3",
                            div {
                                class: "w-10 h-10 bg-gradient-to-br from-blue-500 to-blue-600 rounded-lg flex items-center justify-center shadow-md",
                                span {
                                    class: "text-white font-bold text-lg",
                                    "GC"
                                }
                            }
                            h1 {
                                class: "text-2xl font-bold text-white tracking-tight",
                                {t(current_locale, "app_title")}
                            }
                        }
                        if let Some(content) = header_content {
                            {content}
                        }
                    }
                }
            }
            
            // Main layout with sidebar and content
            div {
                class: "flex flex-row flex-1 min-h-0",
                style: "display: flex; flex-direction: row; margin: 0; padding: 0; width: 100%;",
                // Sidebar navigation - vertical menu on left side
                aside {
                    class: "w-64 bg-white border-r-2 border-gray-300 flex-shrink-0 flex flex-col",
                    style: "width: 16rem; min-width: 16rem; max-width: 16rem; display: flex; flex-direction: column; background-color: white; padding: 0; margin: 0; position: relative; left: 0;",
                    div {
                        class: "flex-1 overflow-y-auto",
                        div {
                            class: "pt-6 pb-2",
                            style: "padding-top: 1.5rem; padding-left: 0; padding-right: 0;",
                            div {
                                class: "px-4 mb-4",
                                LanguageSwitcher {}
                            }
                            h2 {
                                class: "text-xs font-semibold uppercase tracking-wider text-gray-500 mb-4",
                                style: "padding-left: 1rem; padding-right: 1rem;",
                                {t(current_locale, "navigation")}
                            }
                            Navigation {
                                items: items,
                            }
                        }
                    }
                    // Footer area at bottom of sidebar
                    if let Some(footer) = sidebar_footer {
                        div {
                            class: "mt-auto pt-4 pb-4 flex justify-center",
                            style: "padding-left: 0; padding-right: 0;",
                            {footer}
                        }
                    }
                }
                
                // Main content area
                main {
                    class: "flex-1 overflow-y-auto bg-blue-50 min-w-0",
                    style: "flex: 1; overflow-y: auto; background-color: #eff6ff;",
                    div {
                        class: "max-w-7xl mx-auto px-6 sm:px-8 lg:px-10",
                        style: "padding-top: 2rem; padding-bottom: 2rem;",
                        dioxus_router::Outlet::<Route> {}
                    }
                }
            }
        }
    }
}
