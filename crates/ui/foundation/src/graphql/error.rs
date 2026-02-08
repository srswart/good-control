//! GraphQL error types and mapping.
//!
//! This module provides error types that map to the backend's StructuredError format,
//! allowing the frontend to handle different error categories appropriately.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error categories matching backend ErrorCategory enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCategory {
    /// Input validation error
    Validation,
    /// Authorization/permission error
    Authorization,
    /// Compliance gate blocking the operation
    ComplianceBlock,
    /// Resource not found
    NotFound,
    /// Conflict with existing data
    Conflict,
    /// Internal server error
    Internal,
}

/// Field-level error information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldError {
    /// Field name that has the error
    pub field: String,
    /// Error message for this field
    pub message: String,
}

/// Structured error from the backend
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredError {
    /// Error category
    pub category: ErrorCategory,
    /// Main error message
    pub message: String,
    /// Field-level errors (for validation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_errors: Option<Vec<FieldError>>,
    /// Error code (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// GraphQL client errors
#[derive(Debug, Error)]
pub enum GraphQLError {
    /// Network error (connection failed, timeout, etc.)
    #[error("Network error: {0}")]
    Network(String),

    /// HTTP error (non-200 status code)
    #[error("HTTP error {status}: {message}")]
    Http { status: u16, message: String },

    /// GraphQL error from the server
    #[error("GraphQL error: {0}")]
    GraphQL(String),

    /// Structured error from the backend
    #[error("Structured error ({category:?}): {message}")]
    Structured {
        category: ErrorCategory,
        message: String,
        field_errors: Option<Vec<FieldError>>,
    },

    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    Parse(String),

    /// Invalid response format
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

impl GraphQLError {
    /// Check if this is an authorization error
    pub fn is_authorization(&self) -> bool {
        matches!(
            self,
            GraphQLError::Structured {
                category: ErrorCategory::Authorization,
                ..
            }
        )
    }

    /// Check if this is a validation error
    pub fn is_validation(&self) -> bool {
        matches!(
            self,
            GraphQLError::Structured {
                category: ErrorCategory::Validation,
                ..
            }
        )
    }

    /// Check if this is a compliance block error
    pub fn is_compliance_block(&self) -> bool {
        matches!(
            self,
            GraphQLError::Structured {
                category: ErrorCategory::ComplianceBlock,
                ..
            }
        )
    }

    /// Get field errors if this is a validation error
    pub fn field_errors(&self) -> Option<&[FieldError]> {
        match self {
            GraphQLError::Structured { field_errors, .. } => {
                field_errors.as_ref().map(|v| v.as_slice())
            }
            _ => None,
        }
    }
}

// Only implement From<reqwest::Error> for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
impl From<reqwest::Error> for GraphQLError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            GraphQLError::Network("Request timeout".to_string())
        } else if err.is_connect() {
            GraphQLError::Network("Connection failed".to_string())
        } else if let Some(status) = err.status() {
            GraphQLError::Http {
                status: status.as_u16(),
                message: err.to_string(),
            }
        } else {
            GraphQLError::Network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for GraphQLError {
    fn from(err: serde_json::Error) -> Self {
        GraphQLError::Parse(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_serialization() {
        let category = ErrorCategory::Validation;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"VALIDATION\"");

        let category = ErrorCategory::Authorization;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"AUTHORIZATION\"");
    }

    #[test]
    fn test_error_category_deserialization() {
        let json = "\"VALIDATION\"";
        let category: ErrorCategory = serde_json::from_str(json).unwrap();
        assert_eq!(category, ErrorCategory::Validation);

        let json = "\"AUTHORIZATION\"";
        let category: ErrorCategory = serde_json::from_str(json).unwrap();
        assert_eq!(category, ErrorCategory::Authorization);
    }

    #[test]
    fn test_field_error_serialization() {
        let field_error = FieldError {
            field: "sku".to_string(),
            message: "SKU cannot be empty".to_string(),
        };
        let json = serde_json::to_string(&field_error).unwrap();
        assert!(json.contains("\"field\":\"sku\""));
        assert!(json.contains("\"message\":\"SKU cannot be empty\""));
    }

    #[test]
    fn test_structured_error_serialization() {
        let error = StructuredError {
            category: ErrorCategory::Validation,
            message: "Input validation failed".to_string(),
            field_errors: Some(vec![FieldError {
                field: "sku".to_string(),
                message: "SKU cannot be empty".to_string(),
            }]),
            code: Some("VALIDATION_001".to_string()),
        };
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"category\":\"VALIDATION\""));
        assert!(json.contains("\"message\":\"Input validation failed\""));
        assert!(json.contains("\"field_errors\""));
        assert!(json.contains("\"code\":\"VALIDATION_001\""));
    }

    #[test]
    fn test_structured_error_deserialization() {
        let json = r#"{
            "category": "VALIDATION",
            "message": "Input validation failed",
            "field_errors": [
                {"field": "sku", "message": "SKU cannot be empty"}
            ],
            "code": "VALIDATION_001"
        }"#;
        let error: StructuredError = serde_json::from_str(json).unwrap();
        assert_eq!(error.category, ErrorCategory::Validation);
        assert_eq!(error.message, "Input validation failed");
        assert_eq!(error.field_errors.as_ref().unwrap().len(), 1);
        assert_eq!(error.field_errors.as_ref().unwrap()[0].field, "sku");
        assert_eq!(error.code.as_ref().unwrap(), "VALIDATION_001");
    }

    #[test]
    fn test_graphql_error_is_authorization() {
        let error = GraphQLError::Structured {
            category: ErrorCategory::Authorization,
            message: "Permission denied".to_string(),
            field_errors: None,
        };
        assert!(error.is_authorization());
        assert!(!error.is_validation());
        assert!(!error.is_compliance_block());
    }

    #[test]
    fn test_graphql_error_is_validation() {
        let error = GraphQLError::Structured {
            category: ErrorCategory::Validation,
            message: "Validation failed".to_string(),
            field_errors: Some(vec![FieldError {
                field: "sku".to_string(),
                message: "SKU cannot be empty".to_string(),
            }]),
        };
        assert!(error.is_validation());
        assert!(!error.is_authorization());
        assert!(!error.is_compliance_block());
    }

    #[test]
    fn test_graphql_error_is_compliance_block() {
        let error = GraphQLError::Structured {
            category: ErrorCategory::ComplianceBlock,
            message: "Compliance approval required".to_string(),
            field_errors: None,
        };
        assert!(error.is_compliance_block());
        assert!(!error.is_authorization());
        assert!(!error.is_validation());
    }

    #[test]
    fn test_graphql_error_field_errors() {
        let field_errors = vec![
            FieldError {
                field: "sku".to_string(),
                message: "SKU cannot be empty".to_string(),
            },
            FieldError {
                field: "name".to_string(),
                message: "Name is required".to_string(),
            },
        ];
        let error = GraphQLError::Structured {
            category: ErrorCategory::Validation,
            message: "Validation failed".to_string(),
            field_errors: Some(field_errors.clone()),
        };
        let extracted_errors = error.field_errors().unwrap();
        assert_eq!(extracted_errors.len(), 2);
        assert_eq!(extracted_errors[0].field, "sku");
        assert_eq!(extracted_errors[1].field, "name");
    }

    #[test]
    fn test_graphql_error_field_errors_none() {
        let error = GraphQLError::Network("Connection failed".to_string());
        assert!(error.field_errors().is_none());
    }
}

