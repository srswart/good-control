//! Backend GraphQL server binary.

use foundation::{init_tracing, AppConfig};
use persistence::{DbPool, DbPoolConfig};
use server::graphql_router;
use std::process;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Load configuration
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };

    // Initialize tracing
    init_tracing(&config.log_level);

    // Create database pool
    let pool_config = match DbPoolConfig::load() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to load database configuration: {}", e);
            process::exit(1);
        }
    };
    let pool = match DbPool::new(&pool_config).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to create database pool: {}", e);
            process::exit(1);
        }
    };

    // Run migrations
    if let Err(e) = persistence::run_migrations(&pool).await {
        tracing::error!("Failed to run migrations: {}", e);
        process::exit(1);
    }

    // Seed admin user if configured
    if let Err(e) = auth::seed_admin_user(&pool).await {
        tracing::error!("Failed to seed admin user: {}", e);
        process::exit(1);
    }

    // Configure CORS to allow requests from web app
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Create router with GraphQL routes and apply CORS to entire app
    let app = server::create_router()
        .merge(graphql_router::routes(pool))
        .layer(cors);

    // Get server address
    let addr = config.server.addr();
    tracing::info!("Starting GraphQL server on {}", addr);
    tracing::info!("GraphQL playground available at http://{}/graphql", addr);

    // Bind and serve
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind to {}: {}", addr, e);
            process::exit(1);
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("Server error: {}", e);
        process::exit(1);
    }
}

