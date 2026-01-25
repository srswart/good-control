//! Authentication middleware for Axum.

use crate::error::AuthError;
use crate::session::{Session, SessionManager};
use crate::user::User;
use axum::{
    extract::{FromRequestParts, Request},
    http::{header, request::Parts, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use persistence::DbPool;
use std::sync::Arc;

/// Current authenticated user extracted from request.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user: User,
    pub session: Session,
}

/// Auth layer configuration.
#[derive(Clone)]
pub struct AuthLayer {
    pool: DbPool,
}

impl AuthLayer {
    /// Create a new auth layer.
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get the database pool.
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }
}

/// Middleware function to authenticate requests.
pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract auth layer from extensions
    let auth_layer = req.extensions()
        .get::<Arc<AuthLayer>>()
        .ok_or_else(|| AuthError::Internal("AuthLayer not found in extensions".to_string()))?
        .clone();

    // Extract token from Authorization header
    let token = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AuthError::InvalidToken)?;

    // Validate session
    let session_manager = SessionManager::new(auth_layer.pool().clone());
    let session = session_manager.validate_session(token).await?;

    // Get user from session
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, is_active, last_login_at, created_at, updated_at 
         FROM users WHERE id = ?"
    )
    .bind(session.user_id)
    .fetch_one(auth_layer.pool().inner())
    .await
    .map_err(|_| AuthError::UserNotFound(session.user_id.to_string()))?;

    // Check if user is active
    if !user.is_active {
        return Err(AuthError::InvalidCredentials);
    }

    // Insert current user into request extensions
    req.extensions_mut().insert(CurrentUser { user, session });

    Ok(next.run(req).await)
}

/// Extract CurrentUser from request parts.
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Unauthorized: No authenticated user found".to_string(),
            ))
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::InvalidToken | AuthError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            AuthError::PermissionDenied(_) => {
                (StatusCode::FORBIDDEN, self.to_string())
            }
            AuthError::UserNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::create_session;
    use crate::user::{SqliteUserRepository, UserRepository};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use persistence::DbPoolConfig;
    use tower::ServiceExt;

    async fn create_test_pool() -> DbPool {
        let config = DbPoolConfig {
            database_url: "sqlite::memory:".to_string(),
            max_connections: 1,
            connection_timeout_secs: 5,
        };

        let pool = DbPool::new(&config).await.unwrap();

        // Run migrations
        sqlx::migrate!("../../../migrations")
            .run(pool.inner())
            .await
            .unwrap();

        pool
    }

    async fn test_handler(current_user: CurrentUser) -> String {
        format!("Hello, {}!", current_user.user.username)
    }

    #[tokio::test]
    async fn test_auth_middleware_with_valid_token() {
        let pool = create_test_pool().await;
        let user_repo = SqliteUserRepository::new(pool.clone());

        // Create user and session
        let user = user_repo.create("testuser", "test@example.com", "password").await.unwrap();
        let session = create_session(&pool, user.id, None, None).await.unwrap();

        // Create router with auth middleware
        let auth_layer = Arc::new(AuthLayer::new(pool.clone()));
        let app = Router::new()
            .route("/protected", get(test_handler))
            .layer(middleware::from_fn(auth_middleware))
            .layer(axum::Extension(auth_layer));

        // Make request with valid token
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", format!("Bearer {}", session.token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_auth_middleware_without_token() {
        let pool = create_test_pool().await;

        // Create router with auth middleware
        let auth_layer = Arc::new(AuthLayer::new(pool.clone()));
        let app = Router::new()
            .route("/protected", get(test_handler))
            .layer(middleware::from_fn(auth_middleware))
            .layer(axum::Extension(auth_layer));

        // Make request without token
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_auth_middleware_with_invalid_token() {
        let pool = create_test_pool().await;

        // Create router with auth middleware
        let auth_layer = Arc::new(AuthLayer::new(pool.clone()));
        let app = Router::new()
            .route("/protected", get(test_handler))
            .layer(middleware::from_fn(auth_middleware))
            .layer(axum::Extension(auth_layer));

        // Make request with invalid token
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("Authorization", "Bearer invalid_token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        pool.close().await;
    }
}

