//! @arrive component: i18n
//!
//! Internationalization support for EN+ZH locales.
//!
//! This module provides:
//! - Locale management and detection
//! - Translation loading and formatting
//! - Bilingual text support for master data

mod error;
mod locale;
mod translator;

pub use error::{I18nError, Result};
pub use locale::{Locale, SupportedLocale};
pub use translator::{Translator, TranslatorBuilder};

