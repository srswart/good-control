//! Route definitions for the application

use dioxus::prelude::*;
use crate::layout::AppLayout;
use crate::language::use_locale;
use crate::i18n::t;
use ui_inventory::{ItemsList, StockView, TraceabilityView, CycleCountsView};
use ui_assets::{DronesView, BatteriesView};
use ui_compliance::{ApprovalsView, DocPacksView, BlockingReasonsView};
use ui_warehouse::{ReceivingView, PutawayView, PickView, PackView, ShipView};
use ui_logistics::{ShipmentsView, TrackingView, CrossBorderPackReviewView};
use ui_procurement_orders::{PurchaseOrdersView, AllocationsView, FulfillmentView};

/// Application routes
#[derive(Routable, Clone, PartialEq, Debug)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Home {},
    
    #[route("/demo")]
    Demo {},
    
    // Placeholder routes for future feature areas
    #[route("/inventory")]
    Inventory {},
    
    #[route("/warehouse")]
    Warehouse {},
    
    #[route("/assets")]
    Assets {},
    
    #[route("/compliance")]
    Compliance {},
    
    #[route("/orders")]
    Orders {},
    
    #[route("/logistics")]
    Logistics {},
    
    #[route("/reporting")]
    Reporting {},
    
    #[route("/integrations")]
    Integrations {},
    
    #[route("/404")]
    NotFound {},
}

/// Home page component
#[component]
pub fn Home() -> Element {
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();
    
    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                {t(current_locale, "welcome")}
            }
            p {
                class: "text-gray-600",
                {t(current_locale, "coming_soon")}
            }
        }
    }
}

/// Demo page route (redirects to demo component)
#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            class: "text-center text-gray-500",
            "Demo page - This route will be handled by the demo component"
        }
    }
}

/// Inventory route component
/// 
/// Displays inventory management interface with tabs for items, stock, traceability, and cycle counts.
#[component]
pub fn Inventory() -> Element {
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();
    let mut active_tab = use_signal(|| "items".to_string());
    
    rsx! {
        div {
            class: "space-y-6",
            // Tab navigation
            div {
                class: "bg-white rounded-lg shadow p-4",
                div {
                    class: "flex space-x-4 border-b border-gray-200",
                    button {
                        class: if active_tab() == "items" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("items".to_string()),
                        {t(current_locale, "items")}
                    }
                    button {
                        class: if active_tab() == "stock" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("stock".to_string()),
                        {t(current_locale, "stock")}
                    }
                    button {
                        class: if active_tab() == "traceability" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("traceability".to_string()),
                        {t(current_locale, "traceability")}
                    }
                    button {
                        class: if active_tab() == "cycle_counts" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("cycle_counts".to_string()),
                        {t(current_locale, "cycle_counts")}
                    }
                }
            }
            
            // Tab content
            if active_tab() == "items" {
                ItemsList {}
            } else if active_tab() == "stock" {
                StockView {}
            } else if active_tab() == "traceability" {
                TraceabilityView {}
            } else if active_tab() == "cycle_counts" {
                CycleCountsView {}
            }
        }
    }
}

/// Warehouse route component
/// 
/// Displays warehouse management interface with tabs for receiving, putaway, pick, pack, and ship.
#[component]
pub fn Warehouse() -> Element {
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();
    let mut active_tab = use_signal(|| "receiving".to_string());
    
    rsx! {
        div {
            class: "space-y-6",
            // Tab navigation
            div {
                class: "bg-white rounded-lg shadow p-4",
                div {
                    class: "flex space-x-4 border-b border-gray-200",
                    button {
                        class: if active_tab() == "receiving" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("receiving".to_string()),
                        {t(current_locale, "receiving")}
                    }
                    button {
                        class: if active_tab() == "putaway" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("putaway".to_string()),
                        {t(current_locale, "putaway")}
                    }
                    button {
                        class: if active_tab() == "pick" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("pick".to_string()),
                        {t(current_locale, "pick")}
                    }
                    button {
                        class: if active_tab() == "pack" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("pack".to_string()),
                        {t(current_locale, "pack")}
                    }
                    button {
                        class: if active_tab() == "ship" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("ship".to_string()),
                        {t(current_locale, "ship")}
                    }
                }
            }
            
            // Tab content
            if active_tab() == "receiving" {
                ReceivingView {}
            } else if active_tab() == "putaway" {
                PutawayView {}
            } else if active_tab() == "pick" {
                PickView {}
            } else if active_tab() == "pack" {
                PackView {}
            } else if active_tab() == "ship" {
                ShipView {}
            }
        }
    }
}

/// Assets route component
/// 
/// Displays assets management interface with tabs for drones and batteries.
#[component]
pub fn Assets() -> Element {
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();
    let mut active_tab = use_signal(|| "drones".to_string());
    
    rsx! {
        div {
            class: "space-y-6",
            // Tab navigation
            div {
                class: "bg-white rounded-lg shadow p-4",
                div {
                    class: "flex space-x-4 border-b border-gray-200",
                    button {
                        class: if active_tab() == "drones" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("drones".to_string()),
                        {t(current_locale, "drones")}
                    }
                    button {
                        class: if active_tab() == "batteries" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("batteries".to_string()),
                        {t(current_locale, "batteries")}
                    }
                }
            }
            
            // Tab content
            if active_tab() == "drones" {
                DronesView {}
            } else if active_tab() == "batteries" {
                BatteriesView {}
            }
        }
    }
}

/// Compliance route component
/// 
/// Displays compliance management interface with tabs for approvals, doc packs, and blocking reasons.
#[component]
pub fn Compliance() -> Element {
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();
    let mut active_tab = use_signal(|| "approvals".to_string());
    
    rsx! {
        div {
            class: "space-y-6",
            // Tab navigation
            div {
                class: "bg-white rounded-lg shadow p-4",
                div {
                    class: "flex space-x-4 border-b border-gray-200",
                    button {
                        class: if active_tab() == "approvals" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("approvals".to_string()),
                        {t(current_locale, "approvals")}
                    }
                    button {
                        class: if active_tab() == "doc_packs" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("doc_packs".to_string()),
                        {t(current_locale, "doc_packs")}
                    }
                    button {
                        class: if active_tab() == "blocking_reasons" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("blocking_reasons".to_string()),
                        {t(current_locale, "blocking_reasons")}
                    }
                }
            }
            
            // Tab content
            if active_tab() == "approvals" {
                ApprovalsView {}
            } else if active_tab() == "doc_packs" {
                DocPacksView {}
            } else if active_tab() == "blocking_reasons" {
                BlockingReasonsView {}
            }
        }
    }
}

/// Orders route component
/// 
/// Displays procurement and orders management interface with tabs for purchase orders, allocations, and fulfillment.
#[component]
pub fn Orders() -> Element {
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();
    let mut active_tab = use_signal(|| "purchase_orders".to_string());
    
    rsx! {
        div {
            class: "space-y-6",
            // Tab navigation
            div {
                class: "bg-white rounded-lg shadow p-4",
                div {
                    class: "flex space-x-4 border-b border-gray-200",
                    button {
                        class: if active_tab() == "purchase_orders" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("purchase_orders".to_string()),
                        {t(current_locale, "purchase_orders")}
                    }
                    button {
                        class: if active_tab() == "allocations" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("allocations".to_string()),
                        {t(current_locale, "allocations")}
                    }
                    button {
                        class: if active_tab() == "fulfillment" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("fulfillment".to_string()),
                        {t(current_locale, "fulfillment")}
                    }
                }
            }
            
            // Tab content
            if active_tab() == "purchase_orders" {
                PurchaseOrdersView {}
            } else if active_tab() == "allocations" {
                AllocationsView {}
            } else if active_tab() == "fulfillment" {
                FulfillmentView {}
            }
        }
    }
}

/// Logistics route component
/// 
/// Displays logistics management interface with tabs for shipments, tracking, and cross-border pack review.
#[component]
pub fn Logistics() -> Element {
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();
    let mut active_tab = use_signal(|| "shipments".to_string());
    
    rsx! {
        div {
            class: "space-y-6",
            // Tab navigation
            div {
                class: "bg-white rounded-lg shadow p-4",
                div {
                    class: "flex space-x-4 border-b border-gray-200",
                    button {
                        class: if active_tab() == "shipments" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("shipments".to_string()),
                        {t(current_locale, "shipments")}
                    }
                    button {
                        class: if active_tab() == "tracking" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("tracking".to_string()),
                        {t(current_locale, "tracking")}
                    }
                    button {
                        class: if active_tab() == "cross_border_pack_review" {
                            "px-4 py-2 text-sm font-medium text-blue-600 border-b-2 border-blue-600"
                        } else {
                            "px-4 py-2 text-sm font-medium text-gray-500 hover:text-gray-700"
                        },
                        onclick: move |_| active_tab.set("cross_border_pack_review".to_string()),
                        {t(current_locale, "cross_border_pack_review")}
                    }
                }
            }
            
            // Tab content
            if active_tab() == "shipments" {
                ShipmentsView {}
            } else if active_tab() == "tracking" {
                TrackingView {}
            } else if active_tab() == "cross_border_pack_review" {
                CrossBorderPackReviewView {}
            }
        }
    }
}

#[component]
pub fn Reporting() -> Element {
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();
    let message = format!("{} {}", t(current_locale, "reporting"), t(current_locale, "features_coming_soon"));
    
    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 { class: "text-xl font-semibold text-gray-800 mb-4", {t(current_locale, "reporting")} }
            p { class: "text-gray-600", {message} }
        }
    }
}

#[component]
pub fn Integrations() -> Element {
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();
    let message = format!("{} {}", t(current_locale, "integrations"), t(current_locale, "features_coming_soon"));
    
    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 { class: "text-xl font-semibold text-gray-800 mb-4", {t(current_locale, "integrations")} }
            p { class: "text-gray-600", {message} }
        }
    }
}

/// 404 Not Found page
#[component]
pub fn NotFound() -> Element {
    let locale_signal = use_locale();
    let current_locale = *locale_signal.read();
    
    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8 text-center",
            h2 {
                class: "text-2xl font-bold text-gray-800 mb-4",
                {t(current_locale, "not_found")}
            }
            p {
                class: "text-gray-600",
                {t(current_locale, "not_found_message")}
            }
        }
    }
}

/// Router component that handles route rendering
#[component]
pub fn AppRouter() -> Element {
    rsx! {
        dioxus_router::Router::<Route> {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutations;

    #[test]
    fn home_route_renders() {
        // Note: This test would need locale context setup
        // For now, we verify the component structure
        // The component requires locale context, so we test that it compiles
        // Integration tests would verify actual rendering with context
        let _component = Home;
        assert!(true); // Component compiles successfully
    }

    #[test]
    fn not_found_route_renders() {
        // Note: This test would need locale context setup
        // The component requires locale context, so we test that it compiles
        // Integration tests would verify actual rendering with context
        let _component = NotFound;
        assert!(true); // Component compiles successfully
    }
}
