//! @arrive component: i18n
//!
//! Locale management and detection.

use crate::i18n::error::{I18nError, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use unic_langid::LanguageIdentifier;

/// Supported locales for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportedLocale {
    /// English
    #[serde(rename = "en")]
    English,
    /// Chinese (Simplified)
    #[serde(rename = "zh")]
    Chinese,
}

impl SupportedLocale {
    /// Get all supported locales.
    pub fn all() -> &'static [SupportedLocale] {
        &[SupportedLocale::English, SupportedLocale::Chinese]
    }

    /// Get the language identifier for this locale.
    pub fn language_id(&self) -> LanguageIdentifier {
        match self {
            SupportedLocale::English => "en".parse().unwrap(),
            SupportedLocale::Chinese => "zh".parse().unwrap(),
        }
    }

    /// Get the locale code as a string.
    pub fn code(&self) -> &'static str {
        match self {
            SupportedLocale::English => "en",
            SupportedLocale::Chinese => "zh",
        }
    }

    /// Get the display name in the locale's own language.
    pub fn native_name(&self) -> &'static str {
        match self {
            SupportedLocale::English => "English",
            SupportedLocale::Chinese => "中文",
        }
    }
}

impl Default for SupportedLocale {
    fn default() -> Self {
        SupportedLocale::English
    }
}

impl fmt::Display for SupportedLocale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl FromStr for SupportedLocale {
    type Err = I18nError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "en" | "en-us" | "en-gb" => Ok(SupportedLocale::English),
            "zh" | "zh-cn" | "zh-hans" => Ok(SupportedLocale::Chinese),
            _ => Err(I18nError::UnsupportedLocale(s.to_string())),
        }
    }
}

/// Locale context for the current request/session.
#[derive(Debug, Clone)]
pub struct Locale {
    locale: SupportedLocale,
}

impl Locale {
    /// Create a new locale context.
    pub fn new(locale: SupportedLocale) -> Self {
        Self { locale }
    }

    /// Get the supported locale.
    pub fn supported_locale(&self) -> SupportedLocale {
        self.locale
    }

    /// Get the language identifier.
    pub fn language_id(&self) -> LanguageIdentifier {
        self.locale.language_id()
    }

    /// Detect locale from a string (e.g., from Accept-Language header).
    pub fn detect(s: &str) -> Self {
        s.parse::<SupportedLocale>()
            .map(Self::new)
            .unwrap_or_default()
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new(SupportedLocale::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_locale_all() {
        let locales = SupportedLocale::all();
        assert_eq!(locales.len(), 2);
        assert!(locales.contains(&SupportedLocale::English));
        assert!(locales.contains(&SupportedLocale::Chinese));
    }

    #[test]
    fn test_supported_locale_code() {
        assert_eq!(SupportedLocale::English.code(), "en");
        assert_eq!(SupportedLocale::Chinese.code(), "zh");
    }

    #[test]
    fn test_supported_locale_native_name() {
        assert_eq!(SupportedLocale::English.native_name(), "English");
        assert_eq!(SupportedLocale::Chinese.native_name(), "中文");
    }

    #[test]
    fn test_supported_locale_from_str() {
        assert_eq!("en".parse::<SupportedLocale>().unwrap(), SupportedLocale::English);
        assert_eq!("EN".parse::<SupportedLocale>().unwrap(), SupportedLocale::English);
        assert_eq!("en-US".parse::<SupportedLocale>().unwrap(), SupportedLocale::English);
        assert_eq!("zh".parse::<SupportedLocale>().unwrap(), SupportedLocale::Chinese);
        assert_eq!("zh-CN".parse::<SupportedLocale>().unwrap(), SupportedLocale::Chinese);
        assert!("fr".parse::<SupportedLocale>().is_err());
    }

    #[test]
    fn test_locale_detect() {
        let locale = Locale::detect("en");
        assert_eq!(locale.supported_locale(), SupportedLocale::English);

        let locale = Locale::detect("zh-CN");
        assert_eq!(locale.supported_locale(), SupportedLocale::Chinese);

        let locale = Locale::detect("fr"); // Unsupported, should default to English
        assert_eq!(locale.supported_locale(), SupportedLocale::English);
    }
}

