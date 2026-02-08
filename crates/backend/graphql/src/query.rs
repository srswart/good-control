//! GraphQL Query root.

use async_graphql::*;
use auth::{SqliteUserRepository, UserRepository};
use uuid::Uuid;

use crate::context::GraphQLContext;
use crate::types::{
    CycleCountSession, HealthResponse, InventoryMovement, InventoryPosition, Item,
    LotGenealogyEntry, Location, SerialGenealogyEntry, User, ReceivingTask, ReceivingTaskStatus,
    PaginationInput, PageInfo, ItemConnection, ItemEdge,
    PickTask, PickTaskStatus, PackTask, PackTaskStatus,
    ShipTask, ShipTaskStatus,
};

/// GraphQL Query root.
#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    /// Health check query.
    async fn health(&self, ctx: &Context<'_>) -> Result<HealthResponse> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Test database connection
        let db_status = match sqlx::query("SELECT 1")
            .fetch_one(gql_ctx.pool.inner())
            .await
        {
            Ok(_) => "connected",
            Err(_) => "disconnected",
        };

        Ok(HealthResponse {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database: db_status.to_string(),
        })
    }

    /// Get the current authenticated user.
    async fn current_user(&self, ctx: &Context<'_>) -> Result<User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let auth_user = gql_ctx.require_user()?;
        Ok(User::from(auth_user.clone()))
    }

    /// Get a user by ID (requires admin.users permission).
    async fn user(&self, ctx: &Context<'_>, id: ID) -> Result<Option<User>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Check permission
        gql_ctx.require_permission("admin.users").await?;

        let user_id: i64 = id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;

        let user_repo = SqliteUserRepository::new(gql_ctx.pool.clone());
        let user = user_repo.find_by_id(user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to find user: {}", e)))?;

        Ok(user.map(User::from))
    }

    /// List all users (requires admin.users permission).
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Check permission
        gql_ctx.require_permission("admin.users").await?;

        let users = sqlx::query_as::<_, auth::User>(
            "SELECT id, username, email, password_hash, is_active, last_login_at, created_at, updated_at 
             FROM users 
             ORDER BY username"
        )
        .fetch_all(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to list users: {}", e)))?;

        Ok(users.into_iter().map(User::from).collect())
    }

    /// Get a cycle count session by ID, with lines via subfield.
    async fn cycle_count_session(&self, ctx: &Context<'_>, id: String) -> Result<Option<CycleCountSession>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let row = sqlx::query_as::<_, CycleCountSession>(
            r#"SELECT id, abc_class, status, created_at, completed_at FROM cycle_count_sessions WHERE id = ?"#,
        )
        .bind(&id)
        .fetch_optional(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to load cycle count session: {}", e)))?;

        Ok(row)
    }

    // ========================================================================
    // Inventory Domain Queries
    // ========================================================================

    /// Get a specific inventory position by SKU, location, and status.
    async fn inventory_position(
        &self,
        ctx: &Context<'_>,
        sku: String,
        location_id: String,
        status: String,
    ) -> Result<Option<InventoryPosition>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let position = persistence::get_current_position(
            gql_ctx.pool.inner(),
            &sku,
            &location_id,
            &status,
        )
        .await
        .map_err(|e| Error::new(format!("Failed to get inventory position: {}", e)))?;

        Ok(position.map(InventoryPosition::from))
    }

    /// List inventory positions with optional filters.
    async fn inventory_positions(
        &self,
        ctx: &Context<'_>,
        sku: Option<String>,
        location_id: Option<String>,
        status: Option<String>,
    ) -> Result<Vec<InventoryPosition>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Build dynamic query based on filters
        let mut query = "SELECT * FROM inventory_positions WHERE 1=1".to_string();
        let mut bindings = Vec::new();

        if let Some(s) = &sku {
            // Use LIKE for partial matching on SKU
            query.push_str(" AND sku LIKE ?");
            bindings.push(format!("%{}%", s));
        }
        if let Some(l) = &location_id {
            query.push_str(" AND location_id = ?");
            bindings.push(l.clone());
        }
        if let Some(st) = &status {
            query.push_str(" AND status = ?");
            bindings.push(st.clone());
        }
        query.push_str(" ORDER BY sku, location_id");

        let mut q = sqlx::query_as::<_, persistence::InventoryPositionRecord>(&query);
        for binding in bindings {
            q = q.bind(binding);
        }

        let positions = q
            .fetch_all(gql_ctx.pool.inner())
            .await
            .map_err(|e| Error::new(format!("Failed to list inventory positions: {}", e)))?;

        Ok(positions.into_iter().map(InventoryPosition::from).collect())
    }

    /// List inventory movements with optional filters.
    async fn inventory_movements(
        &self,
        ctx: &Context<'_>,
        sku: Option<String>,
        location_id: Option<String>,
        limit: Option<i32>,
    ) -> Result<Vec<InventoryMovement>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let mut query = "SELECT * FROM inventory_movements WHERE 1=1".to_string();
        let mut bindings = Vec::new();

        if let Some(s) = &sku {
            // Use LIKE for partial matching on SKU
            query.push_str(" AND sku LIKE ?");
            bindings.push(format!("%{}%", s));
        }
        if let Some(l) = &location_id {
            query.push_str(" AND (from_location_id = ? OR to_location_id = ?)");
            bindings.push(l.clone());
            bindings.push(l.clone());
        }
        query.push_str(" ORDER BY timestamp DESC");

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }

        let mut q = sqlx::query_as::<_, persistence::InventoryMovementRecord>(&query);
        for binding in bindings {
            q = q.bind(binding);
        }

        let movements = q
            .fetch_all(gql_ctx.pool.inner())
            .await
            .map_err(|e| Error::new(format!("Failed to list inventory movements: {}", e)))?;

        Ok(movements.into_iter().map(InventoryMovement::from).collect())
    }

    /// Get an item by SKU.
    async fn item(&self, ctx: &Context<'_>, sku: String) -> Result<Option<Item>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let item = sqlx::query_as::<_, persistence::ItemRecord>(
            "SELECT * FROM items WHERE sku = ?"
        )
        .bind(&sku)
        .fetch_optional(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to get item: {}", e)))?;

        Ok(item.map(Item::from))
    }

    /// List items with optional filters and pagination.
    async fn items(
        &self,
        ctx: &Context<'_>,
        search: Option<String>,
        item_type: Option<String>,
        tracking_mode: Option<String>,
        pagination: Option<PaginationInput>,
    ) -> Result<ItemConnection> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Default pagination: first 20 items
        let pagination = pagination.unwrap_or(PaginationInput {
            first: Some(20),
            after: None,
            last: None,
            before: None,
        });

        // Validate pagination input
        if pagination.first.is_some() && pagination.last.is_some() {
            return Err(Error::new("Cannot specify both 'first' and 'last'"));
        }
        if pagination.after.is_some() && pagination.before.is_some() {
            return Err(Error::new("Cannot specify both 'after' and 'before'"));
        }

        let limit = pagination.first.or(pagination.last).unwrap_or(20);
        if limit <= 0 || limit > 100 {
            return Err(Error::new("Limit must be between 1 and 100"));
        }

        // Build base query
        let mut query = "SELECT * FROM items WHERE 1=1".to_string();
        let mut count_query = "SELECT COUNT(*) as count FROM items WHERE 1=1".to_string();
        let mut bindings = Vec::new();

        if let Some(s) = &search {
            query.push_str(" AND (sku LIKE ? OR name_en LIKE ? OR name_zh LIKE ?)");
            count_query.push_str(" AND (sku LIKE ? OR name_en LIKE ? OR name_zh LIKE ?)");
            let search_pattern = format!("%{}%", s);
            bindings.push(search_pattern.clone());
            bindings.push(search_pattern.clone());
            bindings.push(search_pattern);
        }
        if let Some(t) = &item_type {
            query.push_str(" AND item_type = ?");
            count_query.push_str(" AND item_type = ?");
            bindings.push(t.clone());
        }
        if let Some(tm) = &tracking_mode {
            query.push_str(" AND tracking_mode = ?");
            count_query.push_str(" AND tracking_mode = ?");
            bindings.push(tm.clone());
        }

        // Handle cursor-based pagination
        let cursor_sku = if let Some(after) = &pagination.after {
            // Decode cursor (for simplicity, cursor is just the SKU)
            Some(after.clone())
        } else if let Some(before) = &pagination.before {
            Some(before.clone())
        } else {
            None
        };

        if let Some(cursor) = &cursor_sku {
            if pagination.after.is_some() {
                query.push_str(" AND sku > ?");
            } else {
                query.push_str(" AND sku < ?");
            }
            bindings.push(cursor.clone());
        }

        // Order and limit
        if pagination.last.is_some() {
            query.push_str(" ORDER BY sku DESC");
        } else {
            query.push_str(" ORDER BY sku ASC");
        }
        query.push_str(&format!(" LIMIT {}", limit + 1)); // Fetch one extra to check if there's a next page

        // Execute query
        let mut q = sqlx::query_as::<_, persistence::ItemRecord>(&query);
        for binding in &bindings {
            q = q.bind(binding);
        }

        let mut items = q
            .fetch_all(gql_ctx.pool.inner())
            .await
            .map_err(|e| Error::new(format!("Failed to list items: {}", e)))?;

        // Check if there are more items
        let has_more = items.len() > limit as usize;
        if has_more {
            items.pop(); // Remove the extra item
        }

        // Reverse if using 'last'
        if pagination.last.is_some() {
            items.reverse();
        }

        // Get total count (optional, can be expensive)
        let mut count_q = sqlx::query_scalar::<_, i64>(&count_query);
        for binding in &bindings[..bindings.len() - cursor_sku.as_ref().map(|_| 1).unwrap_or(0)] {
            count_q = count_q.bind(binding);
        }
        let total_count = count_q
            .fetch_one(gql_ctx.pool.inner())
            .await
            .ok();

        // Build edges
        let edges: Vec<ItemEdge> = items
            .into_iter()
            .map(|item| {
                let cursor = item.sku.clone(); // Use SKU as cursor
                ItemEdge {
                    node: Item::from(item),
                    cursor,
                }
            })
            .collect();

        // Build page info
        let page_info = PageInfo {
            has_next_page: if pagination.first.is_some() { has_more } else { pagination.before.is_some() },
            has_previous_page: if pagination.last.is_some() { has_more } else { pagination.after.is_some() },
            start_cursor: edges.first().map(|e| e.cursor.clone()),
            end_cursor: edges.last().map(|e| e.cursor.clone()),
        };

        Ok(ItemConnection {
            edges,
            page_info,
            total_count,
        })
    }

    /// Get a location by ID.
    async fn location(&self, ctx: &Context<'_>, id: String) -> Result<Option<Location>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let location = sqlx::query_as::<_, persistence::LocationRecord>(
            "SELECT * FROM locations WHERE id = ?"
        )
        .bind(&id)
        .fetch_optional(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to get location: {}", e)))?;

        Ok(location.map(Location::from))
    }

    /// List locations with optional filters.
    async fn locations(
        &self,
        ctx: &Context<'_>,
        warehouse: Option<String>,
    ) -> Result<Vec<Location>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let locations = if let Some(wh) = warehouse {
            sqlx::query_as::<_, persistence::LocationRecord>(
                "SELECT * FROM locations WHERE warehouse = ? ORDER BY warehouse, zone, bin"
            )
            .bind(wh)
            .fetch_all(gql_ctx.pool.inner())
            .await
        } else {
            sqlx::query_as::<_, persistence::LocationRecord>(
                "SELECT * FROM locations ORDER BY warehouse, zone, bin"
            )
            .fetch_all(gql_ctx.pool.inner())
            .await
        }
        .map_err(|e| Error::new(format!("Failed to list locations: {}", e)))?;

        Ok(locations.into_iter().map(Location::from).collect())
    }

    /// Get serial number genealogy (full movement history).
    async fn serial_genealogy(
        &self,
        ctx: &Context<'_>,
        serial_number: String,
    ) -> Result<Vec<SerialGenealogyEntry>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let movements = persistence::get_serial_genealogy(
            gql_ctx.pool.inner(),
            &serial_number,
        )
        .await
        .map_err(|e| Error::new(format!("Failed to get serial genealogy: {}", e)))?;

        Ok(movements.into_iter().map(SerialGenealogyEntry::from).collect())
    }

    /// Get lot number genealogy (full movement history).
    async fn lot_genealogy(
        &self,
        ctx: &Context<'_>,
        lot_number: String,
    ) -> Result<Vec<LotGenealogyEntry>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let movements = persistence::get_lot_genealogy_by_number(
            gql_ctx.pool.inner(),
            &lot_number,
        )
        .await
        .map_err(|e| Error::new(format!("Failed to get lot genealogy: {}", e)))?;

        Ok(movements.into_iter().map(LotGenealogyEntry::from).collect())
    }

    // ========================================================================
    // Warehouse Receiving Queries
    // ========================================================================

    /// Get a receiving task by ID.
    async fn receiving_task(&self, ctx: &Context<'_>, id: ID) -> Result<Option<ReceivingTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Check permission
        gql_ctx.require_permission("warehouse.read").await?;

        let task_id = id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;

        let tasks = gql_ctx.receiving_tasks.read()
            .map_err(|_| Error::new("Failed to read receiving tasks"))?;

        Ok(tasks.get(&task_id).cloned().map(ReceivingTask::from))
    }

    /// Get all receiving tasks with optional pagination.
    async fn receiving_tasks(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<ReceivingTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Check permission
        // TODO: Re-enable authentication when auth system is implemented (Bug #9)
        // gql_ctx.require_permission("warehouse.read").await?;

        let tasks = gql_ctx.receiving_tasks.read()
            .map_err(|_| Error::new("Failed to read receiving tasks"))?;

        let mut all_tasks: Vec<_> = tasks.values().cloned().collect();
        all_tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // Most recent first

        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.map(|l| l as usize);

        let result: Vec<_> = all_tasks
            .into_iter()
            .skip(offset)
            .take(limit.unwrap_or(usize::MAX))
            .map(ReceivingTask::from)
            .collect();

        Ok(result)
    }

    /// Get receiving tasks by status.
    async fn receiving_tasks_by_status(
        &self,
        ctx: &Context<'_>,
        status: ReceivingTaskStatus,
    ) -> Result<Vec<ReceivingTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Check permission
        // TODO: Re-enable authentication when auth system is implemented (Bug #9)
        // gql_ctx.require_permission("warehouse.read").await?;

        let tasks = gql_ctx.receiving_tasks.read()
            .map_err(|_| Error::new("Failed to read receiving tasks"))?;

        let domain_status: warehouse::receiving::ReceivingTaskStatus = match status {
            ReceivingTaskStatus::Pending => warehouse::receiving::ReceivingTaskStatus::Pending,
            ReceivingTaskStatus::InProgress => warehouse::receiving::ReceivingTaskStatus::InProgress,
            ReceivingTaskStatus::Completed => warehouse::receiving::ReceivingTaskStatus::Completed,
            ReceivingTaskStatus::Cancelled => warehouse::receiving::ReceivingTaskStatus::Cancelled,
        };

        let mut filtered: Vec<_> = tasks
            .values()
            .filter(|t| t.status == domain_status)
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // Most recent first

        Ok(filtered.into_iter().map(ReceivingTask::from).collect())
    }

    // ========================================================================
    // Warehouse Picking Queries
    // ========================================================================

    /// Get a pick task by ID.
    async fn pick_task(&self, ctx: &Context<'_>, id: ID) -> Result<Option<PickTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Check permission
        // TODO: Re-enable authentication when auth system is implemented (Bug #9)
        // gql_ctx.require_permission("warehouse.read").await?;

        let task_id = id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;

        let tasks = gql_ctx.pick_tasks.read()
            .map_err(|_| Error::new("Failed to read pick tasks"))?;

        Ok(tasks.get(&task_id).cloned().map(PickTask::from))
    }

    /// Get all pick tasks with optional pagination.
    async fn pick_tasks(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<PickTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Check permission
        // TODO: Re-enable authentication when auth system is implemented (Bug #9)
        // gql_ctx.require_permission("warehouse.read").await?;

        let tasks = gql_ctx.pick_tasks.read()
            .map_err(|_| Error::new("Failed to read pick tasks"))?;

        let mut all_tasks: Vec<_> = tasks.values().cloned().collect();
        all_tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // Most recent first

        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.map(|l| l as usize);

        let result: Vec<_> = all_tasks
            .into_iter()
            .skip(offset)
            .take(limit.unwrap_or(usize::MAX))
            .map(PickTask::from)
            .collect();

        Ok(result)
    }

    /// Get pick tasks by status.
    async fn pick_tasks_by_status(
        &self,
        ctx: &Context<'_>,
        status: PickTaskStatus,
    ) -> Result<Vec<PickTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Check permission
        // TODO: Re-enable authentication when auth system is implemented (Bug #9)
        // gql_ctx.require_permission("warehouse.read").await?;

        let tasks = gql_ctx.pick_tasks.read()
            .map_err(|_| Error::new("Failed to read pick tasks"))?;

        let domain_status: warehouse::picking::PickTaskStatus = match status {
            PickTaskStatus::Pending => warehouse::picking::PickTaskStatus::Pending,
            PickTaskStatus::InProgress => warehouse::picking::PickTaskStatus::InProgress,
            PickTaskStatus::Completed => warehouse::picking::PickTaskStatus::Completed,
            PickTaskStatus::Cancelled => warehouse::picking::PickTaskStatus::Cancelled,
        };

        let mut filtered: Vec<_> = tasks
            .values()
            .filter(|t| t.status == domain_status)
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // Most recent first

        Ok(filtered.into_iter().map(PickTask::from).collect())
    }

    // ========================================================================
    // Warehouse Packing Queries
    // ========================================================================

    /// Get a pack task by ID.
    async fn pack_task(&self, ctx: &Context<'_>, id: ID) -> Result<Option<PackTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Check permission
        // TODO: Re-enable authentication when auth system is implemented (Bug #9)
        // gql_ctx.require_permission("warehouse.read").await?;

        let task_id = id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;

        let tasks = gql_ctx.pack_tasks.read()
            .map_err(|_| Error::new("Failed to read pack tasks"))?;

        Ok(tasks.get(&task_id).cloned().map(PackTask::from))
    }

    /// Get all pack tasks with optional pagination.
    async fn pack_tasks(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<PackTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Check permission
        // TODO: Re-enable authentication when auth system is implemented (Bug #9)
        // gql_ctx.require_permission("warehouse.read").await?;

        let tasks = gql_ctx.pack_tasks.read()
            .map_err(|_| Error::new("Failed to read pack tasks"))?;

        let mut all_tasks: Vec<_> = tasks.values().cloned().collect();
        all_tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // Most recent first

        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.map(|l| l as usize);

        let result: Vec<_> = all_tasks
            .into_iter()
            .skip(offset)
            .take(limit.unwrap_or(usize::MAX))
            .map(PackTask::from)
            .collect();

        Ok(result)
    }

    /// Get pack tasks by status.
    async fn pack_tasks_by_status(
        &self,
        ctx: &Context<'_>,
        status: PackTaskStatus,
    ) -> Result<Vec<PackTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Check permission
        // TODO: Re-enable authentication when auth system is implemented (Bug #9)
        // gql_ctx.require_permission("warehouse.read").await?;

        let tasks = gql_ctx.pack_tasks.read()
            .map_err(|_| Error::new("Failed to read pack tasks"))?;

        let domain_status: warehouse::packing::PackTaskStatus = match status {
            PackTaskStatus::Pending => warehouse::packing::PackTaskStatus::Pending,
            PackTaskStatus::InProgress => warehouse::packing::PackTaskStatus::InProgress,
            PackTaskStatus::Completed => warehouse::packing::PackTaskStatus::Completed,
            PackTaskStatus::Cancelled => warehouse::packing::PackTaskStatus::Cancelled,
        };

        let mut filtered: Vec<_> = tasks
            .values()
            .filter(|t| t.status == domain_status)
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // Most recent first

        Ok(filtered.into_iter().map(PackTask::from).collect())
    }

    // ========================================================================
    // Ship Queries
    // ========================================================================

    /// Get a single ship task by ID.
    async fn ship_task(&self, ctx: &Context<'_>, id: ID) -> Result<Option<ShipTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        // TODO: Re-enable authentication when auth system is implemented (Bug #9)
        // gql_ctx.require_permission("warehouse.read").await?;

        let task_uuid = Uuid::parse_str(&id).map_err(|_| Error::new("Invalid task ID"))?;

        let tasks = gql_ctx.ship_tasks.read()
            .map_err(|_| Error::new("Failed to read ship tasks"))?;

        Ok(tasks.get(&task_uuid).cloned().map(ShipTask::from))
    }

    /// Get all ship tasks with pagination.
    async fn ship_tasks(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<ShipTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        // TODO: Re-enable authentication when auth system is implemented (Bug #9)
        // gql_ctx.require_permission("warehouse.read").await?;

        let tasks = gql_ctx.ship_tasks.read()
            .map_err(|_| Error::new("Failed to read ship tasks"))?;

        let mut all_tasks: Vec<_> = tasks.values().cloned().collect();
        all_tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // Most recent first

        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(50) as usize;

        Ok(all_tasks
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(ShipTask::from)
            .collect())
    }

    /// Get ship tasks filtered by status.
    async fn ship_tasks_by_status(
        &self,
        ctx: &Context<'_>,
        status: ShipTaskStatus,
    ) -> Result<Vec<ShipTask>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        // TODO: Re-enable authentication when auth system is implemented (Bug #9)
        // gql_ctx.require_permission("warehouse.read").await?;

        let tasks = gql_ctx.ship_tasks.read()
            .map_err(|_| Error::new("Failed to read ship tasks"))?;

        let domain_status: warehouse::shipping::ShipTaskStatus = match status {
            ShipTaskStatus::Pending => warehouse::shipping::ShipTaskStatus::Pending,
            ShipTaskStatus::Staging => warehouse::shipping::ShipTaskStatus::Staging,
            ShipTaskStatus::ComplianceCheck => warehouse::shipping::ShipTaskStatus::ComplianceCheck,
            ShipTaskStatus::ReadyToDispatch => warehouse::shipping::ShipTaskStatus::ReadyToDispatch,
            ShipTaskStatus::Dispatched => warehouse::shipping::ShipTaskStatus::Dispatched,
            ShipTaskStatus::Blocked => warehouse::shipping::ShipTaskStatus::Blocked,
            ShipTaskStatus::Cancelled => warehouse::shipping::ShipTaskStatus::Cancelled,
        };

        let mut filtered: Vec<_> = tasks
            .values()
            .filter(|t| t.status == domain_status)
            .cloned()
            .collect();
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at)); // Most recent first

        Ok(filtered.into_iter().map(ShipTask::from).collect())
    }
}
