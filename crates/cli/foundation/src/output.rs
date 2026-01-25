//! CLI output formatting and conventions.

use serde::Serialize;
use std::fmt;
use std::io;

/// Output format for CLI commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Human-readable output (default).
    Human,
    /// JSON output for automation.
    Json,
}

impl OutputFormat {
    /// Parse output format from string.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Human => write!(f, "human"),
            Self::Json => write!(f, "json"),
        }
    }
}

/// Writer for CLI output with format support.
pub struct OutputWriter {
    format: OutputFormat,
}

impl OutputWriter {
    /// Create a new output writer with the specified format.
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Write a success message.
    pub fn success(&self, message: &str) -> io::Result<()> {
        match self.format {
            OutputFormat::Human => {
                println!("✓ {}", message);
                Ok(())
            }
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "status": "success",
                    "message": message,
                });
                self.write_json(&output)
            }
        }
    }

    /// Write an error message.
    pub fn error(&self, message: &str) -> io::Result<()> {
        match self.format {
            OutputFormat::Human => {
                eprintln!("✗ Error: {}", message);
                Ok(())
            }
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "status": "error",
                    "message": message,
                });
                self.write_json_stderr(&output)
            }
        }
    }

    /// Write a warning message.
    pub fn warning(&self, message: &str) -> io::Result<()> {
        match self.format {
            OutputFormat::Human => {
                println!("⚠ Warning: {}", message);
                Ok(())
            }
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "status": "warning",
                    "message": message,
                });
                self.write_json(&output)
            }
        }
    }

    /// Write an info message.
    pub fn info(&self, message: &str) -> io::Result<()> {
        match self.format {
            OutputFormat::Human => {
                println!("{}", message);
                Ok(())
            }
            OutputFormat::Json => {
                let output = serde_json::json!({
                    "status": "info",
                    "message": message,
                });
                self.write_json(&output)
            }
        }
    }

    /// Write structured data.
    pub fn data<T: Serialize>(&self, data: &T) -> io::Result<()> {
        match self.format {
            OutputFormat::Human => {
                // For human format, use pretty JSON as fallback
                let json = serde_json::to_string_pretty(data)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                println!("{}", json);
                Ok(())
            }
            OutputFormat::Json => self.write_json(data),
        }
    }

    /// Write a table header (human format only).
    pub fn table_header(&self, columns: &[&str]) -> io::Result<()> {
        if self.format == OutputFormat::Human {
            println!("{}", columns.join(" | "));
            println!("{}", "-".repeat(columns.join(" | ").len()));
        }
        Ok(())
    }

    /// Write a table row (human format only).
    pub fn table_row(&self, values: &[&str]) -> io::Result<()> {
        if self.format == OutputFormat::Human {
            println!("{}", values.join(" | "));
        }
        Ok(())
    }

    /// Write JSON to stdout.
    fn write_json<T: Serialize>(&self, data: &T) -> io::Result<()> {
        let json = serde_json::to_string(data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", json);
        Ok(())
    }

    /// Write JSON to stderr.
    fn write_json_stderr<T: Serialize>(&self, data: &T) -> io::Result<()> {
        let json = serde_json::to_string(data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        eprintln!("{}", json);
        Ok(())
    }

    /// Get the current output format.
    pub fn format(&self) -> OutputFormat {
        self.format
    }
}

impl Default for OutputWriter {
    fn default() -> Self {
        Self::new(OutputFormat::Human)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("human").unwrap(), OutputFormat::Human);
        assert_eq!(OutputFormat::from_str("Human").unwrap(), OutputFormat::Human);
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str("JSON").unwrap(), OutputFormat::Json);
        assert!(OutputFormat::from_str("invalid").is_err());
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(OutputFormat::Human.to_string(), "human");
        assert_eq!(OutputFormat::Json.to_string(), "json");
    }

    #[test]
    fn test_output_writer_creation() {
        let writer = OutputWriter::new(OutputFormat::Human);
        assert_eq!(writer.format(), OutputFormat::Human);

        let writer = OutputWriter::new(OutputFormat::Json);
        assert_eq!(writer.format(), OutputFormat::Json);
    }

    #[test]
    fn test_output_writer_default() {
        let writer = OutputWriter::default();
        assert_eq!(writer.format(), OutputFormat::Human);
    }
}

