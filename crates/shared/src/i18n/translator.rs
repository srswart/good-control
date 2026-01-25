//! @arrive component: i18n
//!
//! Translation and message formatting.

use crate::i18n::error::{I18nError, Result};
use crate::i18n::locale::{Locale, SupportedLocale};
use fluent::{FluentArgs, FluentBundle, FluentResource};
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

/// Translator for managing translations across locales.
pub struct Translator {
    bundles: HashMap<SupportedLocale, FluentBundle<FluentResource>>,
    fallback_locale: SupportedLocale,
}

impl Translator {
    /// Get a translated message for the given key and locale.
    pub fn translate(&self, locale: &Locale, key: &str) -> Result<String> {
        self.translate_with_args(locale, key, None)
    }

    /// Get a translated message with arguments.
    pub fn translate_with_args(
        &self,
        locale: &Locale,
        key: &str,
        args: Option<&FluentArgs>,
    ) -> Result<String> {
        let supported_locale = locale.supported_locale();

        // Try the requested locale first
        if let Some(result) = self.try_translate(supported_locale, key, args) {
            return Ok(result);
        }

        // Fall back to the fallback locale
        if supported_locale != self.fallback_locale {
            if let Some(result) = self.try_translate(self.fallback_locale, key, args) {
                return Ok(result);
            }
        }

        // If all else fails, return an error
        Err(I18nError::TranslationNotFound {
            key: key.to_string(),
            locale: supported_locale.code().to_string(),
        })
    }

    fn try_translate(
        &self,
        locale: SupportedLocale,
        key: &str,
        args: Option<&FluentArgs>,
    ) -> Option<String> {
        let bundle = self.bundles.get(&locale)?;
        let message = bundle.get_message(key)?;
        let pattern = message.value()?;

        let mut errors = vec![];
        let value = bundle.format_pattern(pattern, args, &mut errors);

        if errors.is_empty() {
            Some(value.to_string())
        } else {
            None
        }
    }
}

/// Builder for creating a Translator.
pub struct TranslatorBuilder {
    bundles: HashMap<SupportedLocale, FluentBundle<FluentResource>>,
    fallback_locale: SupportedLocale,
}

impl TranslatorBuilder {
    /// Create a new translator builder.
    pub fn new() -> Self {
        Self {
            bundles: HashMap::new(),
            fallback_locale: SupportedLocale::default(),
        }
    }

    /// Set the fallback locale.
    pub fn fallback_locale(mut self, locale: SupportedLocale) -> Self {
        self.fallback_locale = locale;
        self
    }

    /// Add translations from a FTL string.
    pub fn add_ftl(mut self, locale: SupportedLocale, ftl_string: &str) -> Result<Self> {
        let resource = FluentResource::try_new(ftl_string.to_string())
            .map_err(|e| I18nError::Parse(format!("{:?}", e)))?;

        let lang_id: LanguageIdentifier = locale.language_id();
        let mut bundle = FluentBundle::new(vec![lang_id]);

        bundle
            .add_resource(resource)
            .map_err(|e| I18nError::Fluent(format!("{:?}", e)))?;

        self.bundles.insert(locale, bundle);
        Ok(self)
    }

    /// Build the translator.
    pub fn build(self) -> Translator {
        Translator {
            bundles: self.bundles,
            fallback_locale: self.fallback_locale,
        }
    }
}

impl Default for TranslatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EN_FTL: &str = r#"
hello = Hello, World!
greeting = Hello, { $name }!
"#;

    const ZH_FTL: &str = r#"
hello = 你好，世界！
greeting = 你好，{ $name }！
"#;

    #[test]
    fn test_translator_simple_translation() {
        let translator = TranslatorBuilder::new()
            .add_ftl(SupportedLocale::English, EN_FTL)
            .unwrap()
            .add_ftl(SupportedLocale::Chinese, ZH_FTL)
            .unwrap()
            .build();

        let en_locale = Locale::new(SupportedLocale::English);
        let result = translator.translate(&en_locale, "hello").unwrap();
        assert_eq!(result, "Hello, World!");

        let zh_locale = Locale::new(SupportedLocale::Chinese);
        let result = translator.translate(&zh_locale, "hello").unwrap();
        assert_eq!(result, "你好，世界！");
    }

    #[test]
    fn test_translator_with_args() {
        let translator = TranslatorBuilder::new()
            .add_ftl(SupportedLocale::English, EN_FTL)
            .unwrap()
            .build();

        let locale = Locale::new(SupportedLocale::English);
        let mut args = FluentArgs::new();
        args.set("name", "Alice");

        let result = translator
            .translate_with_args(&locale, "greeting", Some(&args))
            .unwrap();
        // Fluent adds Unicode directional markers around interpolated values
        assert!(result.contains("Hello"));
        assert!(result.contains("Alice"));
    }

    #[test]
    fn test_translator_fallback() {
        let translator = TranslatorBuilder::new()
            .fallback_locale(SupportedLocale::English)
            .add_ftl(SupportedLocale::English, EN_FTL)
            .unwrap()
            .build();

        // Request Chinese translation, but only English is available
        let zh_locale = Locale::new(SupportedLocale::Chinese);
        let result = translator.translate(&zh_locale, "hello").unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_translator_missing_key() {
        let translator = TranslatorBuilder::new()
            .add_ftl(SupportedLocale::English, EN_FTL)
            .unwrap()
            .build();

        let locale = Locale::new(SupportedLocale::English);
        let result = translator.translate(&locale, "nonexistent");
        assert!(result.is_err());
    }
}

