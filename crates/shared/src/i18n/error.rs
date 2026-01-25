//! @arrive component: i18n
//!
//! Error types for i18n operations.

use thiserror::Error;

/// Result type for i18n operations.
pub type Result<T> = std::result::Result<T, I18nError>;

/// Errors that can occur in i18n operations.
#[derive(Debug, Error)]
pub enum I18nError {
    /// Unsupported locale.
    #[error("unsupported locale: {0}")]
    UnsupportedLocale(String),

    /// Translation not found.
    #[error("translation not found for key: {key} in locale: {locale}")]
    TranslationNotFound { key: String, locale: String },

    /// Fluent error.
    #[error("fluent error: {0}")]
    Fluent(String),

    /// Parse error.
    #[error("parse error: {0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_locale_error() {
        let err = I18nError::UnsupportedLocale("fr".to_string());
        assert_eq!(err.to_string(), "unsupported locale: fr");
    }

    #[test]
    fn test_translation_not_found_error() {
        let err = I18nError::TranslationNotFound {
            key: "hello".to_string(),
            locale: "en".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "translation not found for key: hello in locale: en"
        );
    }
}

