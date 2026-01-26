//! Navigation component for the app shell

use dioxus::prelude::*;
use crate::routes::Route;
use crate::language::use_locale;
use crate::i18n::t;

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
                        "block w-full text-left py-3 rounded-r-lg text-sm font-medium transition-all duration-200 bg-gradient-to-r from-blue-600 to-blue-700 shadow-md shadow-blue-500/30 border-l-4 border-blue-400"
                    } else {
                        "block w-full text-left py-3 rounded-r-lg text-sm font-medium transition-all duration-200 text-gray-700 hover:text-gray-900 hover:bg-gray-50 hover:border-l-4 hover:border-gray-300 active:bg-gray-100"
                    },
                    style: if current_route == item.route {
                        "display: block; width: 100%; text-align: left; padding-left: 1rem; padding-right: 1rem; color: white !important; background: linear-gradient(to right, #2563eb, #1d4ed8) !important; border-left: 4px solid #60a5fa !important;"
                    } else {
                        "display: block; width: 100%; text-align: left; padding-left: 1rem; padding-right: 1rem; color: #374151 !important; background: white !important; border-left: none !important;"
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
    ]
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
    }
}
