//! Navigation component for the app shell

use dioxus::prelude::*;
use crate::routes::Route;
use crate::language::use_locale;
use crate::i18n::t;
use crate::brand::BrandColors;
use crate::session::Session;

/// Navigation item definition
#[derive(Clone, PartialEq, Debug)]
pub struct NavItem {
    /// Route for this navigation item
    pub route: Route,
    /// Display label
    pub label: String,
    /// Icon (optional, for future use)
    pub icon: Option<String>,
}

impl NavItem {
    pub fn new(route: Route, label: impl Into<String>) -> Self {
        Self {
            route,
            label: label.into(),
            icon: None,
        }
    }
}

/// Navigation component
/// 
/// Provides navigation links for the application.
/// Uses Dioxus Router's Link component for client-side navigation.
#[component]
pub fn Navigation(
    /// Navigation items to display
    items: Vec<NavItem>,
    /// Additional CSS classes
    class: Option<String>,
) -> Element {
    let current_route = dioxus_router::use_route::<Route>();
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();

    rsx! {
        nav {
            class: "flex flex-col space-y-1 w-full",
            style: "display: flex; flex-direction: column; width: 100%; padding: 0; margin: 0;",
            for item in &items {
                dioxus_router::Link {
                    to: item.route.clone(),
                    class: if current_route == item.route {
                        "block w-full text-left py-3 rounded-r-lg text-sm font-medium transition-all duration-200 shadow-md border-l-4"
                    } else {
                        "block w-full text-left py-3 rounded-r-lg text-sm font-medium transition-all duration-200 hover:border-l-4"
                    },
                    style: if current_route == item.route {
                        format!("display: block; width: 100%; text-align: left; padding-left: 1rem; padding-right: 1rem; color: {} !important; background: linear-gradient(to right, {}, {}) !important; border-left: 4px solid {} !important;",
                            BrandColors::NAVY, BrandColors::MINT, BrandColors::MINT_DARK, BrandColors::CORAL)
                    } else {
                        format!("display: block; width: 100%; text-align: left; padding-left: 1rem; padding-right: 1rem; color: #e2e8f0 !important; background: transparent !important; border-left: none !important;")
                    },
                    {
                        // Translate the label based on route
                        let label_key = match item.route {
                            Route::Home {} => "home",
                            Route::Inventory {} => "inventory",
                            Route::Warehouse {} => "warehouse",
                            Route::Assets {} => "assets",
                            Route::Compliance {} => "compliance",
                            Route::Orders {} => "orders",
                            Route::Logistics {} => "logistics",
                            Route::Reporting {} => "reporting",
                            Route::Integrations {} => "integrations",
                            Route::AdminUsers {} => "admin_users",
                            _ => "",
                        };
                        if !label_key.is_empty() {
                            t(current_locale, label_key)
                        } else {
                            item.label.as_str()
                        }
                    }
                }
            }
        }
    }
}

/// Default navigation items for the application
pub fn default_nav_items() -> Vec<NavItem> {
    vec![
        NavItem::new(Route::Home {}, "Home"),
        NavItem::new(Route::Inventory {}, "Inventory"),
        NavItem::new(Route::Warehouse {}, "Warehouse"),
        NavItem::new(Route::Assets {}, "Assets"),
        NavItem::new(Route::Compliance {}, "Compliance"),
        NavItem::new(Route::Orders {}, "Orders"),
        NavItem::new(Route::Logistics {}, "Logistics"),
        NavItem::new(Route::Reporting {}, "Reporting"),
        NavItem::new(Route::Integrations {}, "Integrations"),
        NavItem::new(Route::AdminUsers {}, "Admin Users"),
    ]
}

/// Default navigation items filtered by session access.
pub fn nav_items_for_session(session: &Session) -> Vec<NavItem> {
    default_nav_items()
        .into_iter()
        .filter(|item| session.can_access_route(&item.route))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutations;

    #[test]
    fn nav_item_creation() {
        let item = NavItem::new(Route::Home {}, "Home");
        assert_eq!(item.label, "Home");
        assert_eq!(item.route, Route::Home {});
    }

    #[test]
    fn default_nav_items_contains_all_routes() {
        let items = default_nav_items();
        assert!(items.iter().any(|i| matches!(i.route, Route::Home {})));
        assert!(items.iter().any(|i| matches!(i.route, Route::Inventory {})));
        assert!(items.iter().any(|i| matches!(i.route, Route::Warehouse {})));
        assert!(items.iter().any(|i| matches!(i.route, Route::AdminUsers {})));
    }
}
