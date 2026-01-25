//! @arrive component: foundation
//! Axum HTTP server for drone-inventory backend.
//!
//! This crate provides the HTTP server infrastructure:
//! - Router setup with health check endpoints
//! - Middleware configuration (tracing, CORS)
//! - Server lifecycle management

pub mod router;
pub mod health;

pub use router::create_router;
pub use health::health_check;

use foundation::AppConfig;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

/// Run the server with the given configuration.
///
/// # Arguments
///
/// * `config` - Application configuration.
///
/// # Errors
///
/// Returns an error if the server fails to bind or run.
pub async fn run(config: &AppConfig) -> Result<(), std::io::Error> {
    let app = create_router();
    let addr: SocketAddr = config.server.addr().parse().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid address: {e}"))
    })?;

    info!("Starting server on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Run the server with the given configuration, returning when a shutdown signal is received.
///
/// # Arguments
///
/// * `config` - Application configuration.
/// * `shutdown` - A future that completes when the server should shut down.
///
/// # Errors
///
/// Returns an error if the server fails to bind or run.
pub async fn run_with_shutdown<F>(config: &AppConfig, shutdown: F) -> Result<(), std::io::Error>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let app = create_router();
    let addr: SocketAddr = config.server.addr().parse().map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid address: {e}"))
    })?;

    info!("Starting server on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    Ok(())
}

/// Create a test server that binds to a random port.
/// Returns the router and the bound address.
#[cfg(test)]
pub async fn create_test_server() -> (axum::Router, SocketAddr) {
    let app = create_router();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    (app, addr)
}

