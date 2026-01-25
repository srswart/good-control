//! CLI configuration loading and types.

use serde::Deserialize;
use std::path::PathBuf;

/// CLI configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct CliConfig {
    /// Backend GraphQL endpoint URL.
    #[serde(default = "default_backend_url")]
    pub backend_url: String,

    /// Service/admin token for backend authentication.
    pub auth_token: Option<String>,

    /// Database path for local operations (migrations, backups).
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,

    /// Log level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Default output format (human, json).
    #[serde(default = "default_output_format")]
    pub output_format: String,
}

fn default_backend_url() -> String {
    "http://127.0.0.1:3000/graphql".to_string()
}

fn default_db_path() -> PathBuf {
    PathBuf::from("./data/inventory.db")
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_output_format() -> String {
    "human".to_string()
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            backend_url: default_backend_url(),
            auth_token: None,
            db_path: default_db_path(),
            log_level: default_log_level(),
            output_format: default_output_format(),
        }
    }
}

impl CliConfig {
    /// Load configuration from environment and optional config file.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration cannot be loaded or parsed.
    pub fn load() -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            // Start with defaults
            .set_default("backend_url", "http://127.0.0.1:3000/graphql")?
            .set_default("db_path", "./data/inventory.db")?
            .set_default("log_level", "info")?
            .set_default("output_format", "human")?
            // Add optional config file
            .add_source(config::File::with_name("cli-config").required(false))
            // Add environment variables with prefix CLI_
            .add_source(
                config::Environment::with_prefix("CLI")
                    .separator("__")
                    .try_parsing(true),
            );

        builder.build()?.try_deserialize()
    }

    /// Load configuration with custom config file path.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration cannot be loaded or parsed.
    pub fn load_from_file(path: &str) -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            // Start with defaults
            .set_default("backend_url", "http://127.0.0.1:3000/graphql")?
            .set_default("db_path", "./data/inventory.db")?
            .set_default("log_level", "info")?
            .set_default("output_format", "human")?
            // Add specified config file
            .add_source(config::File::with_name(path))
            // Add environment variables with prefix CLI_
            .add_source(
                config::Environment::with_prefix("CLI")
                    .separator("__")
                    .try_parsing(true),
            );

        builder.build()?.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_config_defaults() {
        let config = CliConfig::default();
        assert_eq!(config.backend_url, "http://127.0.0.1:3000/graphql");
        assert_eq!(config.db_path, PathBuf::from("./data/inventory.db"));
        assert_eq!(config.log_level, "info");
        assert_eq!(config.output_format, "human");
        assert!(config.auth_token.is_none());
    }

    #[test]
    fn test_cli_config_load_defaults() {
        // Should load with defaults when no config file or env vars
        let result = CliConfig::load();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.backend_url, "http://127.0.0.1:3000/graphql");
    }
}

