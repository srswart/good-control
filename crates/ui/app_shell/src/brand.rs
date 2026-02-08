//! Brand colors and styling constants

/// Brand color palette extracted from the GOOD CONTROL branding
pub struct BrandColors;

impl BrandColors {
    /// Primary brand color - Navy Blue
    /// Used for: backgrounds, headers, primary UI elements
    pub const NAVY: &'static str = "#1e293b";
    
    /// Secondary brand color - Mint Green
    /// Used for: logo text, accents, highlights, CTAs
    pub const MINT: &'static str = "#6ee7b7";
    
    /// Accent brand color - Coral Orange
    /// Used for: decorative elements, warnings, secondary accents
    pub const CORAL: &'static str = "#fb923c";
    
    /// Light variant of navy for subtle backgrounds
    pub const NAVY_LIGHT: &'static str = "#334155";
    
    /// Dark variant of mint for hover states
    pub const MINT_DARK: &'static str = "#5eead4";
    
    /// Light variant of coral for backgrounds
    pub const CORAL_LIGHT: &'static str = "#fdba74";
}

/// CSS gradient for brand header
pub fn brand_header_gradient() -> String {
    format!("linear-gradient(135deg, {} 0%, {} 100%)", BrandColors::NAVY, BrandColors::NAVY_LIGHT)
}

/// CSS gradient for accent elements
pub fn brand_accent_gradient() -> String {
    format!("linear-gradient(90deg, {} 0%, {} 100%)", BrandColors::MINT, BrandColors::MINT_DARK)
}

