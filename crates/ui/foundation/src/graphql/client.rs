//! GraphQL client for making requests to the backend.

use super::auth_token::auth_token;
use super::error::{GraphQLError, StructuredError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Platform-specific HTTP client imports
#[cfg(not(target_arch = "wasm32"))]
use reqwest;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;

/// GraphQL request
#[derive(Debug, Clone, Serialize)]
struct GraphQLRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation_name: Option<String>,
}

/// GraphQL response (raw JSON)
#[derive(Debug, Clone, Deserialize)]
struct GraphQLResponseRaw {
    #[serde(default)]
    data: Option<serde_json::Value>,
    #[serde(default)]
    errors: Vec<GraphQLResponseError>,
}

/// GraphQL error in response
#[derive(Debug, Clone, Deserialize)]
struct GraphQLResponseError {
    message: String,
    #[serde(default)]
    extensions: HashMap<String, serde_json::Value>,
}

/// GraphQL client for making requests to the backend
#[derive(Debug, Clone)]
pub struct GraphQLClient {
    endpoint: String,
    #[cfg(not(target_arch = "wasm32"))]
    client: reqwest::Client,
}

impl GraphQLClient {
    /// Create a new GraphQL client
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            #[cfg(not(target_arch = "wasm32"))]
            client: reqwest::Client::new(),
        }
    }

    /// Execute a GraphQL query
    pub async fn query<T: for<'de> Deserialize<'de>>(
        &self,
        query: impl Into<String>,
        variables: Option<serde_json::Value>,
    ) -> Result<T, GraphQLError> {
        self.execute(query, variables, None).await
    }

    /// Execute a GraphQL mutation
    pub async fn mutate<T: for<'de> Deserialize<'de>>(
        &self,
        mutation: impl Into<String>,
        variables: Option<serde_json::Value>,
    ) -> Result<T, GraphQLError> {
        self.execute(mutation, variables, None).await
    }

    /// Execute a GraphQL operation with optional operation name (native implementation)
    #[cfg(not(target_arch = "wasm32"))]
    async fn execute<T: for<'de> Deserialize<'de>>(
        &self,
        query: impl Into<String>,
        variables: Option<serde_json::Value>,
        operation_name: Option<String>,
    ) -> Result<T, GraphQLError> {
        // Generate correlation ID for request tracing
        let correlation_id = Uuid::new_v4().to_string();

        let request = GraphQLRequest {
            query: query.into(),
            variables,
            operation_name,
        };

        // Make HTTP request with correlation ID header
        let mut request_builder = self
            .client
            .post(&self.endpoint)
            .header("X-Correlation-ID", correlation_id.clone());

        if let Some(token) = auth_token() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        let response = request_builder.json(&request).send().await?;

        // Check HTTP status
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| String::new());
            return Err(GraphQLError::Http {
                status: status.as_u16(),
                message: error_text,
            });
        }

        // Parse response
        let response_text = response.text().await.map_err(|e| {
            GraphQLError::Network(format!("Failed to read response body: {}", e))
        })?;

        Self::parse_response(&response_text)
    }

    /// Execute a GraphQL operation with optional operation name (WASM implementation)
    #[cfg(target_arch = "wasm32")]
    async fn execute<T: for<'de> Deserialize<'de>>(
        &self,
        query: impl Into<String>,
        variables: Option<serde_json::Value>,
        operation_name: Option<String>,
    ) -> Result<T, GraphQLError> {
        // Generate correlation ID for request tracing
        let correlation_id = Uuid::new_v4().to_string();

        let request = GraphQLRequest {
            query: query.into(),
            variables,
            operation_name,
        };

        // Make HTTP request with correlation ID header
        let mut request_builder = Request::post(&self.endpoint)
            .header("X-Correlation-ID", &correlation_id)
            .header("Content-Type", "application/json");

        if let Some(token) = auth_token() {
            request_builder = request_builder.header("Authorization", &format!("Bearer {}", token));
        }

        let response = request_builder
            .json(&request)
            .map_err(|e| GraphQLError::Network(format!("Failed to create request: {}", e)))?
            .send()
            .await
            .map_err(|e| GraphQLError::Network(format!("Failed to send request: {}", e)))?;

        // Check HTTP status
        let status = response.status();
        if status < 200 || status >= 300 {
            let error_text = response.text().await.unwrap_or_else(|_| String::new());
            return Err(GraphQLError::Http {
                status,
                message: error_text,
            });
        }

        // Parse response
        let response_text = response.text().await.map_err(|e| {
            GraphQLError::Network(format!("Failed to read response body: {}", e))
        })?;

        Self::parse_response(&response_text)
    }

    /// Parse GraphQL response (shared between native and WASM)
    fn parse_response<T: for<'de> Deserialize<'de>>(response_text: &str) -> Result<T, GraphQLError> {
        let graphql_response: GraphQLResponseRaw = serde_json::from_str(response_text)?;

        // Check for GraphQL errors
        if !graphql_response.errors.is_empty() {
            let first_error = &graphql_response.errors[0];

            // Try to parse as structured error
            if let Ok(structured_error) = serde_json::from_str::<StructuredError>(&first_error.message) {
                return Err(GraphQLError::Structured {
                    category: structured_error.category,
                    message: structured_error.message,
                    field_errors: structured_error.field_errors,
                });
            }

            // Fall back to plain GraphQL error
            return Err(GraphQLError::GraphQL(first_error.message.clone()));
        }

        // Return data or error if missing
        let data_value = graphql_response.data.ok_or_else(|| {
            GraphQLError::InvalidResponse("Response has no data and no errors".to_string())
        })?;

        // Deserialize the data value into the target type
        serde_json::from_value(data_value).map_err(|e| GraphQLError::Parse(e.to_string()))
    }

    /// Get the correlation ID from the last request (for testing)
    #[cfg(test)]
    pub fn last_correlation_id(&self) -> Option<String> {
        // In a real implementation, we'd store this in the client
        // For now, this is just a placeholder for testing
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GraphQLClient::new("http://localhost:8080/graphql");
        assert_eq!(client.endpoint, "http://localhost:8080/graphql");
    }
}

