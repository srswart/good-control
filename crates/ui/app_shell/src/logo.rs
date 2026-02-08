//! Brand logo component

use dioxus::prelude::*;
use crate::brand::BrandColors;

/// Logo size variants
#[derive(Clone, Copy, PartialEq)]
pub enum LogoSize {
    /// Small logo (for sidebar, mobile)
    Small,
    /// Medium logo (for header)
    Medium,
    /// Large logo (for home page, splash screens)
    Large,
}

impl LogoSize {
    fn container_class(&self) -> &'static str {
        match self {
            LogoSize::Small => "h-12",
            LogoSize::Medium => "h-16",
            LogoSize::Large => "h-24",
        }
    }
    
    fn text_size(&self) -> &'static str {
        match self {
            LogoSize::Small => "text-lg",
            LogoSize::Medium => "text-2xl",
            LogoSize::Large => "text-4xl",
        }
    }
    
    fn stripe_height(&self) -> &'static str {
        match self {
            LogoSize::Small => "h-1",
            LogoSize::Medium => "h-1.5",
            LogoSize::Large => "h-2",
        }
    }
}

/// Brand logo component
/// 
/// Displays the "GOOD CONTROL" logo with brand colors and styling.
#[component]
pub fn Logo(
    /// Size variant of the logo
    #[props(default = LogoSize::Medium)]
    size: LogoSize,
    /// Whether to show the decorative stripes
    #[props(default = true)]
    show_stripes: bool,
) -> Element {
    let text_size = size.text_size();
    let stripe_height = size.stripe_height();
    
    rsx! {
        div {
            class: "flex flex-col items-start gap-1",
            // Logo text
            div {
                class: format!("font-bold tracking-tight {}", text_size),
                style: format!("color: {}; font-family: system-ui, -apple-system, sans-serif;", BrandColors::MINT),
                "GOOD CONTROL"
            }
            // Decorative stripes
            if show_stripes {
                div {
                    class: "flex gap-1 w-full",
                    div {
                        class: format!("flex-1 {} rounded-full", stripe_height),
                        style: format!("background-color: {};", BrandColors::CORAL),
                    }
                    div {
                        class: format!("flex-1 {} rounded-full", stripe_height),
                        style: format!("background-color: {};", BrandColors::MINT),
                    }
                    div {
                        class: format!("flex-1 {} rounded-full", stripe_height),
                        style: format!("background-color: {};", BrandColors::CORAL),
                    }
                }
            }
        }
    }
}

/// Compact logo icon (just "GC" initials)
/// 
/// Used for small spaces like mobile headers or favicons.
#[component]
pub fn LogoIcon(
    /// Size in pixels
    #[props(default = 40)]
    size: u32,
) -> Element {
    rsx! {
        div {
            class: "rounded-lg flex items-center justify-center shadow-md",
            style: format!(
                "width: {}px; height: {}px; background: linear-gradient(135deg, {} 0%, {} 100%);",
                size, size, BrandColors::MINT, BrandColors::MINT_DARK
            ),
            span {
                class: "font-bold",
                style: format!("color: {}; font-size: {}px;", BrandColors::NAVY, size / 2),
                "GC"
            }
        }
    }
}

