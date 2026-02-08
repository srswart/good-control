//! @arrive component: app-shell
//! App shell: routing, navigation, session, permission-aware rendering.
//!
//! This crate provides the app shell infrastructure:
//! - Routing and navigation
//! - Session state management
//! - Navigation components
//! - Permission-aware rendering helpers

use dioxus::prelude::*;
use shared::i18n::SupportedLocale;

pub mod routes;
pub mod nav;
pub mod session;
pub mod passkey_web;
pub mod login;
pub mod layout;
pub mod language;
pub mod i18n;
pub mod brand;
pub mod logo;

pub use routes::Route;
pub use nav::{Navigation, NavItem, default_nav_items, nav_items_for_session};
pub use session::{Session, SessionProvider, use_session};
pub use passkey_web::{webauthn_authenticate, webauthn_register};
pub use login::{LoginView, PasskeyRegisterButton};
pub use layout::AppLayout;
pub use language::LanguageSwitcher;
pub use logo::{Logo, LogoIcon, LogoSize};
pub use brand::BrandColors;

/// App shell component that initializes Router and Session
/// 
/// This should be used at the root of your application.
/// It provides Router context and Session context to all child components.
#[component]
pub fn AppShell(
    /// Additional content to display in header (e.g., user menu, language switcher)
    header_content: Option<Element>,
    /// Additional content to display at bottom of sidebar (e.g., DS Examples button)
    sidebar_footer: Option<Element>,
    /// Navigation items (defaults to default_nav_items if not provided)
    nav_items: Option<Vec<NavItem>>,
) -> Element {
    // Store header_content, sidebar_footer, and nav_items in context so AppLayout can access them
    use_context_provider(|| {
        Signal::new(LayoutContext {
            header_content: header_content.clone(),
            sidebar_footer: sidebar_footer.clone(),
            nav_items: nav_items.clone(),
        })
    });

    // Provide locale context for language switcher
    let locale_state = use_signal(|| SupportedLocale::English);
    provide_context(locale_state);

    rsx! {
        SessionProvider {
            dioxus_router::Router::<Route> {}
        }
    }
}

/// Context for passing layout props to AppLayout
#[derive(Clone)]
pub struct LayoutContext {
    pub header_content: Option<Element>,
    pub sidebar_footer: Option<Element>,
    pub nav_items: Option<Vec<NavItem>>,
}
