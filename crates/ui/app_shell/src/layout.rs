//! App shell layout component

use dioxus::prelude::*;
use crate::nav::{Navigation, nav_items_for_session};
use crate::routes::Route;
use crate::session::use_session;
use crate::LayoutContext;
use crate::LanguageSwitcher;
use crate::language::use_locale;
use crate::i18n::t;
use crate::logo::{Logo, LogoSize};
use crate::brand::BrandColors;
use crate::login::PasskeyRegisterButton;
use ui_foundation::graphql::auth_token::set_auth_token;

/// App shell layout component
/// 
/// Provides the main application layout with header, navigation sidebar, and content area.
/// This component is used as a layout route in the router.
#[component]
pub fn AppLayout() -> Element {
    let mut session_signal = use_session();
    let session = session_signal.read();
    let navigator = dioxus_router::use_navigator();
    let current_route = dioxus_router::use_route::<Route>();

    use_effect(move || {
        if !session_signal.read().is_authenticated {
            navigator.push(Route::Login {});
        }
    });

    // Get layout context from AppShell
    let context_signal = use_context::<Signal<LayoutContext>>();
    let ctx = context_signal.read();
    let items = ctx
        .nav_items
        .clone()
        .unwrap_or_else(|| nav_items_for_session(&session));
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
                class: "text-white shadow-lg border-b-2",
                style: format!("background: linear-gradient(135deg, {} 0%, {} 100%); border-color: {};",
                    BrandColors::NAVY, BrandColors::NAVY_LIGHT, BrandColors::CORAL),
                div {
                    class: "max-w-full mx-auto px-6 sm:px-8 lg:px-10 py-5",
                    div {
                        class: "flex justify-between items-center",
                        div {
                            class: "flex items-center space-x-4",
                            Logo {
                                size: LogoSize::Medium,
                                show_stripes: true,
                            }
                        }
                        div {
                            class: "flex items-center space-x-3",
                            if let Some(content) = header_content {
                                {content}
                            }
                            if session.is_authenticated {
                                PasskeyRegisterButton {}
                                button {
                                    class: "text-xs text-white bg-rose-600 rounded px-3 py-2",
                                    onclick: move |_| {
                                        session_signal.write().clear();
                                        set_auth_token(None);
                                        navigator.push(Route::Login {});
                                    },
                                    "Logout"
                                }
                            }
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
                    class: "w-64 border-r-2 flex-shrink-0 flex flex-col",
                    style: format!("width: 16rem; min-width: 16rem; max-width: 16rem; display: flex; flex-direction: column; background-color: {}; border-color: {}; padding: 0; margin: 0; position: relative; left: 0;",
                        BrandColors::NAVY, BrandColors::CORAL),
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
                                class: "text-xs font-semibold uppercase tracking-wider mb-4",
                                style: format!("padding-left: 1rem; padding-right: 1rem; color: {};", BrandColors::MINT),
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
                            class: "mt-auto pt-4 pb-4 flex justify-center border-t",
                            style: format!("padding-left: 0; padding-right: 0; border-color: {};", BrandColors::NAVY_LIGHT),
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
                        if session.can_access_route(&current_route) {
                            dioxus_router::Outlet::<Route> {}
                        } else {
                            div {
                                class: "bg-white rounded-lg shadow p-8 text-center",
                                h2 { class: "text-xl font-semibold text-gray-800 mb-2", "Access denied" }
                                p { class: "text-gray-600", "You do not have permission to view this module." }
                            }
                        }
                    }
                }
            }
        }
    }
}
