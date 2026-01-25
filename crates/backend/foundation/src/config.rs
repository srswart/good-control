//! Configuration loading and types.

use serde::Deserialize;

/// Server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Host to bind to.
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to bind to.
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl ServerConfig {
    /// Returns the socket address string.
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Application configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Server configuration.
    #[serde(default)]
    pub server: ServerConfig,

    /// Log level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            log_level: default_log_level(),
        }
    }
}

impl AppConfig {
    /// Load configuration from environment and optional config file.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration cannot be loaded or parsed.
    pub fn load() -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            // Start with defaults
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 3000)?
            .set_default("log_level", "info")?
            // Add optional config file
            .add_source(config::File::with_name("config").required(false))
            // Add environment variables with prefix
            .add_source(
                config::Environment::with_prefix("APP")
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
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_server_config_addr() {
        let config = ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
        };
        assert_eq!(config.addr(), "0.0.0.0:8080");
    }

    #[test]
    fn test_app_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.log_level, "info");
    }
}

