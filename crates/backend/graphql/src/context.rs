//! GraphQL request context.

use auth::User;
use persistence::DbPool;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use warehouse::receiving::ReceivingTask;
use warehouse::picking::PickTask;
use warehouse::packing::PackTask;
use warehouse::shipping::ShipTask;

/// In-memory store for receiving tasks (temporary until persistence layer is added).
pub type ReceivingTaskStore = Arc<RwLock<HashMap<Uuid, ReceivingTask>>>;

/// In-memory store for pick tasks (temporary until persistence layer is added).
pub type PickTaskStore = Arc<RwLock<HashMap<Uuid, PickTask>>>;

/// In-memory store for pack tasks (temporary until persistence layer is added).
pub type PackTaskStore = Arc<RwLock<HashMap<Uuid, PackTask>>>;

/// In-memory store for ship tasks (temporary until persistence layer is added).
pub type ShipTaskStore = Arc<RwLock<HashMap<Uuid, ShipTask>>>;

/// GraphQL request context containing user session and database pool.
#[derive(Clone)]
pub struct GraphQLContext {
    /// Database pool for queries and mutations.
    pub pool: DbPool,
    /// Current authenticated user (if any).
    pub current_user: Option<User>,
    /// In-memory store for receiving tasks.
    pub receiving_tasks: ReceivingTaskStore,
    /// In-memory store for pick tasks.
    pub pick_tasks: PickTaskStore,
    /// In-memory store for pack tasks.
    pub pack_tasks: PackTaskStore,
    /// In-memory store for ship tasks.
    pub ship_tasks: ShipTaskStore,
}

impl GraphQLContext {
    /// Create a new GraphQL context.
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool,
            current_user: None,
            receiving_tasks: Arc::new(RwLock::new(HashMap::new())),
            pick_tasks: Arc::new(RwLock::new(HashMap::new())),
            pack_tasks: Arc::new(RwLock::new(HashMap::new())),
            ship_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new GraphQL context with an authenticated user.
    pub fn with_user(pool: DbPool, user: User) -> Self {
        Self {
            pool,
            current_user: Some(user),
            receiving_tasks: Arc::new(RwLock::new(HashMap::new())),
            pick_tasks: Arc::new(RwLock::new(HashMap::new())),
            pack_tasks: Arc::new(RwLock::new(HashMap::new())),
            ship_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new GraphQL context with an authenticated user and task stores.
    pub fn with_user_and_tasks(
        pool: DbPool,
        user: User,
        receiving_tasks: ReceivingTaskStore,
        pick_tasks: PickTaskStore,
        pack_tasks: PackTaskStore,
        ship_tasks: ShipTaskStore,
    ) -> Self {
        Self {
            pool,
            current_user: Some(user),
            receiving_tasks,
            pick_tasks,
            pack_tasks,
            ship_tasks,
        }
    }

    /// Get the current user, returning an error if not authenticated.
    pub fn require_user(&self) -> async_graphql::Result<&User> {
        self.current_user
            .as_ref()
            .ok_or_else(|| async_graphql::Error::new("Authentication required"))
    }

    /// Check if the current user has a specific permission.
    pub async fn check_permission(&self, permission: &str) -> async_graphql::Result<bool> {
        let user = self.require_user()?;
        auth::check_permission(&self.pool, user.id, permission)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Permission check failed: {}", e)))
    }

    /// Require the current user to have a specific permission.
    pub async fn require_permission(&self, permission: &str) -> async_graphql::Result<()> {
        let user = self.require_user()?;
        auth::require_permission(&self.pool, user.id, permission)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Permission denied: {}", e)))
    }
}

