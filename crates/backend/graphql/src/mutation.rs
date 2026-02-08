//! GraphQL Mutation root.

use async_graphql::*;
use auth::{
    authenticate, begin_passkey_authentication, begin_passkey_registration, finish_passkey_authentication,
    finish_passkey_registration, AuditEventType, SqliteUserRepository, UserRepository, record_audit_event,
};
use procurement::purchase_order::PurchaseOrder;
use uuid::Uuid;

use crate::context::GraphQLContext;
use crate::types::{
    CreateItemInput, CreateLocationInput, CycleCountLine, CycleCountSession, InventoryMovement,
    Item, Location, LoginResponse, MockDataSummary, PasskeyChallenge, ReserveResponse, User, ReceivingTask, ReceivingLine,
    CreateReceivingTaskInput, RecordQCOutcomeInput, QuarantineLineInput,
    ReleaseFromQuarantineInput, RecordReceiptInput, AcknowledgeDiscrepancyInput,
    FieldError, StructuredError, validate_not_empty,
    AdjustInventoryInput, BulkMutationResult, RowResult,
    PickTask, PickLine, CreatePickTaskInput, AddPickLineInput, RecordPickInput,
    RecordPickExceptionInput, PackTask, PackLine, Package, CreatePackTaskInput,
    AddPackLineInput, CreatePackageInput, RecordPackInput, SetPackageDimensionsInput,
    SetPackageWeightInput, AssignTrackingInput,
    ShipTask, ShipLine, ComplianceBlock, CreateShipTaskInput, AddShipLineInput,
    DispatchShipmentInput,
};
use persistence::InventoryService;

struct MockTaskSummary {
    receiving_tasks: i64,
    pick_tasks: i64,
    pack_tasks: i64,
    ship_tasks: i64,
}

/// GraphQL Mutation root.
#[derive(Default)]
pub struct Mutation;

#[Object]
impl Mutation {
    /// Reload mock data (admin-only).
    async fn reload_mock_data(&self, ctx: &Context<'_>) -> Result<MockDataSummary> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        gql_ctx.require_permission("admin.system").await?;

        let db_summary = persistence::reload_mock_data(&gql_ctx.pool)
            .await
            .map_err(|e| Error::new(format!("Failed to reload mock data: {}", e)))?;

        let task_summary = seed_mock_tasks(gql_ctx)?;

        Ok(MockDataSummary {
            inventory_items: db_summary.items,
            inventory_locations: db_summary.locations,
            inventory_lots: db_summary.lots,
            inventory_serial_units: db_summary.serial_units,
            inventory_positions: db_summary.inventory_positions,
            inventory_movements: db_summary.inventory_movements,
            receiving_tasks: task_summary.receiving_tasks,
            pick_tasks: task_summary.pick_tasks,
            pack_tasks: task_summary.pack_tasks,
            ship_tasks: task_summary.ship_tasks,
        })
    }
    /// Login with username and password.
    async fn login(
        &self,
        ctx: &Context<'_>,
        username: String,
        password: String,
    ) -> Result<LoginResponse> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user_repo = SqliteUserRepository::new(gql_ctx.pool.clone());

        // Authenticate user
        let (user, session) = authenticate(
            &user_repo,
            &gql_ctx.pool,
            &username,
            &password,
            None, // IP address - could be extracted from request
            None, // User agent - could be extracted from request
        )
        .await
        .map_err(|e| Error::new(format!("Authentication failed: {}", e)))?;

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::AuthLogin,
            "User logged in via GraphQL",
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(LoginResponse {
            user: User::from(user),
            token: session.token,
        })
    }

    /// Begin passkey registration for the current user.
    async fn begin_passkey_registration(&self, ctx: &Context<'_>) -> Result<PasskeyChallenge> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        let (challenge_id, options) = begin_passkey_registration(&gql_ctx.pool, user)
            .await
            .map_err(|e| Error::new(format!("Failed to begin passkey registration: {}", e)))?;

        let options_json = serde_json::to_string(&options)
            .map_err(|e| Error::new(format!("Failed to serialize passkey options: {}", e)))?;

        Ok(PasskeyChallenge {
            challenge_id,
            options_json,
        })
    }

    /// Finish passkey registration for the current user.
    async fn finish_passkey_registration(
        &self,
        ctx: &Context<'_>,
        challenge_id: String,
        credential_json: String,
    ) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        finish_passkey_registration(&gql_ctx.pool, user, &challenge_id, &credential_json)
            .await
            .map_err(|e| Error::new(format!("Failed to finish passkey registration: {}", e)))?;

        Ok(true)
    }

    /// Begin passkey authentication (login) for a username or email.
    async fn begin_passkey_authentication(
        &self,
        ctx: &Context<'_>,
        username_or_email: String,
    ) -> Result<PasskeyChallenge> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let (challenge_id, options) = begin_passkey_authentication(&gql_ctx.pool, &username_or_email)
            .await
            .map_err(|e| Error::new(format!("Failed to begin passkey authentication: {}", e)))?;

        let options_json = serde_json::to_string(&options)
            .map_err(|e| Error::new(format!("Failed to serialize passkey options: {}", e)))?;

        Ok(PasskeyChallenge {
            challenge_id,
            options_json,
        })
    }

    /// Finish passkey authentication (login) and create a session.
    async fn finish_passkey_authentication(
        &self,
        ctx: &Context<'_>,
        challenge_id: String,
        credential_json: String,
    ) -> Result<LoginResponse> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        let user = finish_passkey_authentication(&gql_ctx.pool, &challenge_id, &credential_json)
            .await
            .map_err(|e| Error::new(format!("Passkey authentication failed: {}", e)))?;

        let session = auth::create_session(&gql_ctx.pool, user.id, None, None)
            .await
            .map_err(|e| Error::new(format!("Failed to create session: {}", e)))?;

        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::AuthLogin,
            "User logged in via passkey",
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(LoginResponse {
            user: User::from(user),
            token: session.token,
        })
    }

    /// Logout (invalidate current session).
    async fn logout(&self, ctx: &Context<'_>) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::AuthLogout,
            "User logged out via GraphQL",
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

        // Note: In a real implementation, we would invalidate the session token here
        // For now, we just record the audit event
        Ok(true)
    }

    /// Create a new user (requires admin.users permission).
    async fn create_user(
        &self,
        ctx: &Context<'_>,
        username: String,
        email: String,
        password: String,
    ) -> Result<User> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Check permission
        gql_ctx.require_permission("admin.users").await?;

        let user_repo = SqliteUserRepository::new(gql_ctx.pool.clone());
        let user = user_repo.create(&username, &email, &password)
            .await
            .map_err(|e| Error::new(format!("Failed to create user: {}", e)))?;

        // Record audit event
        let current_user = gql_ctx.require_user()?;
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(current_user.id),
            AuditEventType::UserCreated,
            &format!("Created user: {}", username),
            Some("user"),
            Some(&user.id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(User::from(user))
    }

    /// Deactivate a user (requires admin.users permission).
    async fn deactivate_user(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Check permission
        gql_ctx.require_permission("admin.users").await?;

        let user_id: i64 = id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;

        let user_repo = SqliteUserRepository::new(gql_ctx.pool.clone());
        user_repo.deactivate(user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to deactivate user: {}", e)))?;

        // Record audit event
        let current_user = gql_ctx.require_user()?;
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(current_user.id),
            AuditEventType::UserDeactivated,
            &format!("Deactivated user ID: {}", user_id),
            Some("user"),
            Some(&user_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(true)
    }

    /// Activate a user (requires admin.users permission).
    async fn activate_user(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        
        // Check permission
        gql_ctx.require_permission("admin.users").await?;

        let user_id: i64 = id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;

        let user_repo = SqliteUserRepository::new(gql_ctx.pool.clone());
        user_repo.activate(user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to activate user: {}", e)))?;

        // Record audit event
        let current_user = gql_ctx.require_user()?;
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(current_user.id),
            AuditEventType::UserActivated,
            &format!("Activated user ID: {}", user_id),
            Some("user"),
            Some(&user_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(true)
    }

    /// Assign a role to a user (requires admin.roles permission).
    async fn assign_role(&self, ctx: &Context<'_>, user_id: ID, role_id: ID) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Check permission
        gql_ctx.require_permission("admin.roles").await?;

        let user_id: i64 = user_id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;
        let role_id: i64 = role_id.parse()
            .map_err(|_| Error::new("Invalid role ID"))?;

        let current_user = gql_ctx.require_user()?;
        auth::assign_role(&gql_ctx.pool, user_id, role_id, Some(current_user.id))
            .await
            .map_err(|e| Error::new(format!("Failed to assign role: {}", e)))?;

        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(current_user.id),
            AuditEventType::RoleAssigned,
            &format!("Assigned role {} to user {}", role_id, user_id),
            Some("user_role"),
            Some(&format!("{}:{}", user_id, role_id)),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(true)
    }

    /// Remove a role from a user (requires admin.roles permission).
    async fn remove_role(&self, ctx: &Context<'_>, user_id: ID, role_id: ID) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;

        // Check permission
        gql_ctx.require_permission("admin.roles").await?;

        let user_id: i64 = user_id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;
        let role_id: i64 = role_id.parse()
            .map_err(|_| Error::new("Invalid role ID"))?;

        auth::remove_role(&gql_ctx.pool, user_id, role_id)
            .await
            .map_err(|e| Error::new(format!("Failed to remove role: {}", e)))?;

        let current_user = gql_ctx.require_user()?;
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(current_user.id),
            AuditEventType::RoleRemoved,
            &format!("Removed role {} from user {}", role_id, user_id),
            Some("user_role"),
            Some(&format!("{}:{}", user_id, role_id)),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(true)
    }

    /// Reserve inventory, optionally allowing partial allocation.
    async fn reserve_inventory(
        &self,
        ctx: &Context<'_>,
        sku: String,
        location_id: String,
        quantity: i64,
        #[graphql(default = false)] allow_partial: bool,
        reference: Option<String>,
    ) -> Result<ReserveResponse> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        let service = InventoryService::new(gql_ctx.pool.inner().clone());
        let (movement_id, allocated) = service
            .reserve_inventory_with_partial(
                &sku,
                &location_id,
                quantity,
                allow_partial,
                &user.username,
                reference.clone(),
            )
            .await
            .map_err(|e| Error::new(format!("Reservation failed: {}", e)))?;

        Ok(ReserveResponse {
            movement_id,
            allocated,
        })
    }

    /// Start a cycle count session with given ABC class (A/B/C).
    async fn start_cycle_count(&self, ctx: &Context<'_>, abc_class: String) -> Result<CycleCountSession> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        let session_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO cycle_count_sessions (id, abc_class, status, created_at)
               VALUES (?, ?, 'open', datetime('now'))"#,
        )
        .bind(&session_id)
        .bind(&abc_class)
        .execute(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to create cycle count session: {}", e)))?;

        // Return session
        let session = sqlx::query_as::<_, CycleCountSession>(
            r#"SELECT id, abc_class, status, created_at, completed_at FROM cycle_count_sessions WHERE id = ?"#,
        )
        .bind(&session_id)
        .fetch_one(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to load created session: {}", e)))?;

        // Audit
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Started cycle count session {} ({})", session_id, abc_class),
            Some("inventory"),
            Some(&session_id),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(session)
    }

    /// Record a cycle count for a SKU/location within a session.
    async fn record_cycle_count(
        &self,
        ctx: &Context<'_>,
        session_id: String,
        sku: String,
        location_id: String,
        counted_qty: i64,
    ) -> Result<CycleCountLine> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Determine expected from inventory_positions
        let expected: Option<i64> = sqlx::query_scalar(
            r#"SELECT quantity_available FROM inventory_positions
               WHERE sku = ? AND location_id = ? AND status = 'available' AND lot_id IS NULL"#,
        )
        .bind(&sku)
        .bind(&location_id)
        .fetch_optional(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to read expected quantity: {}", e)))?;
        let expected_qty = expected.unwrap_or(0);
        let variance = counted_qty - expected_qty;

        // Upsert line
        let line_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            r#"INSERT INTO cycle_count_lines
               (id, session_id, sku, location_id, expected_qty, counted_qty, variance, status)
               VALUES (?, ?, ?, ?, ?, ?, ?, 'counted')"#,
        )
        .bind(&line_id)
        .bind(&session_id)
        .bind(&sku)
        .bind(&location_id)
        .bind(expected_qty)
        .bind(counted_qty)
        .bind(variance)
        .execute(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to write cycle count line: {}", e)))?;

        // Audit
        let details = format!("{{\"sku\":\"{}\",\"loc\":\"{}\",\"expected\":{},\"counted\":{},\"variance\":{}}}", sku, location_id, expected_qty, counted_qty, variance);
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            "Cycle count recorded",
            Some("inventory"),
            Some(&session_id),
            Some(&details),
            None,
            None,
            None,
        )
        .await;

        // Return line
        let line = sqlx::query_as::<_, CycleCountLine>(
            r#"SELECT id, session_id, sku, location_id, expected_qty, counted_qty, variance, status
               FROM cycle_count_lines WHERE id = ?"#,
        )
        .bind(&line_id)
        .fetch_one(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to load cycle count line: {}", e)))?;

        Ok(line)
    }

    /// Complete a cycle count session.
    async fn complete_cycle_count(&self, ctx: &Context<'_>, session_id: String) -> Result<bool> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;
        sqlx::query(
            r#"UPDATE cycle_count_sessions SET status = 'completed', completed_at = datetime('now') WHERE id = ?"#,
        )
        .bind(&session_id)
        .execute(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to complete cycle count session: {}", e)))?;

        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            "Cycle count session completed",
            Some("inventory"),
            Some(&session_id),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(true)
    }

    // ========================================================================
    // Inventory Domain Mutations
    // ========================================================================

    /// Create a new item (requires warehouse.inventory permission).
    async fn create_item(
        &self,
        ctx: &Context<'_>,
        input: CreateItemInput,
    ) -> Result<Item> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("inventory.write").await
            .map_err(|_| {
                let err = StructuredError::authorization("Permission denied: inventory.write required");
                Error::new(serde_json::to_string(&err).unwrap_or_else(|_| err.message.clone()))
            })?;

        // Validate input
        let mut field_errors = Vec::new();

        if let Some(err) = validate_not_empty("sku", &input.sku) {
            field_errors.push(err);
        }
        if let Some(err) = validate_not_empty("name", &input.name) {
            field_errors.push(err);
        }
        if let Some(err) = validate_not_empty("name_zh", &input.name_zh) {
            field_errors.push(err);
        }
        if let Some(err) = validate_not_empty("item_type", &input.item_type) {
            field_errors.push(err);
        }
        if let Some(err) = validate_not_empty("tracking_mode", &input.tracking_mode) {
            field_errors.push(err);
        }

        // Validate item_type is one of the allowed values
        let valid_item_types = vec!["drone", "battery", "part", "tool", "consumable", "packaging", "service"];
        if !valid_item_types.contains(&input.item_type.as_str()) {
            field_errors.push(FieldError {
                field: "item_type".to_string(),
                message: format!("item_type must be one of: {}", valid_item_types.join(", ")),
            });
        }

        // Validate tracking_mode is one of the allowed values
        let valid_tracking_modes = vec!["none", "serial", "lot"];
        if !valid_tracking_modes.contains(&input.tracking_mode.as_str()) {
            field_errors.push(FieldError {
                field: "tracking_mode".to_string(),
                message: format!("tracking_mode must be one of: {}", valid_tracking_modes.join(", ")),
            });
        }

        // If there are validation errors, return them
        if !field_errors.is_empty() {
            let err = StructuredError::validation("Input validation failed", field_errors);
            return Err(Error::new(serde_json::to_string(&err).unwrap_or_else(|_| err.message.clone())));
        }

        let item_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Check for duplicate SKU
        let existing = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM items WHERE sku = ?")
            .bind(&input.sku)
            .fetch_one(gql_ctx.pool.inner())
            .await
            .map_err(|e| Error::new(format!("Failed to check for duplicate SKU: {}", e)))?;

        if existing > 0 {
            let err = StructuredError::conflict(format!("Item with SKU '{}' already exists", input.sku));
            return Err(Error::new(serde_json::to_string(&err).unwrap_or_else(|_| err.message.clone())));
        }

        sqlx::query(
            r#"
            INSERT INTO items (
                id, sku, name_en, name_zh, description_en, description_zh,
                item_type, tracking_mode, manufacturer_part_no, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&item_id)
        .bind(&input.sku)
        .bind(&input.name)
        .bind(&input.name_zh)
        .bind(&input.description)
        .bind(&input.description_zh)
        .bind(&input.item_type)
        .bind(&input.tracking_mode)
        .bind(&input.manufacturer_part_no)
        .bind(&now)
        .bind(&now)
        .execute(gql_ctx.pool.inner())
        .await
        .map_err(|e| {
            let err = StructuredError::internal(format!("Failed to create item: {}", e));
            Error::new(serde_json::to_string(&err).unwrap_or_else(|_| err.message.clone()))
        })?;

        // Fetch the created item
        let item = sqlx::query_as::<_, persistence::ItemRecord>(
            "SELECT * FROM items WHERE id = ?"
        )
        .bind(&item_id)
        .fetch_one(gql_ctx.pool.inner())
        .await
        .map_err(|e| {
            let err = StructuredError::internal(format!("Failed to fetch created item: {}", e));
            Error::new(serde_json::to_string(&err).unwrap_or_else(|_| err.message.clone()))
        })?;

        // Audit
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Created item: {}", input.sku),
            Some("inventory"),
            Some(&item_id),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(Item::from(item))
    }

    /// Create a new location (requires warehouse.inventory permission).
    async fn create_location(
        &self,
        ctx: &Context<'_>,
        input: CreateLocationInput,
    ) -> Result<Location> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("inventory.write").await?;

        let location_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO locations (id, warehouse, zone, bin, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&location_id)
        .bind(&input.warehouse)
        .bind(&input.zone)
        .bind(&input.bin)
        .bind(&now)
        .execute(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to create location: {}", e)))?;

        // Fetch the created location
        let location = sqlx::query_as::<_, persistence::LocationRecord>(
            "SELECT * FROM locations WHERE id = ?"
        )
        .bind(&location_id)
        .fetch_one(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to fetch created location: {}", e)))?;

        // Audit
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Created location: {}/{}/{}", input.warehouse, input.zone, input.bin),
            Some("inventory"),
            Some(&location_id),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(Location::from(location))
    }

    /// Receive inventory (requires warehouse.execute permission).
    async fn receive_inventory(
        &self,
        ctx: &Context<'_>,
        sku: String,
        location_id: String,
        quantity: i64,
        reference: Option<String>,
    ) -> Result<InventoryMovement> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("inventory.write").await?;

        let service = InventoryService::new(gql_ctx.pool.inner().clone());
        let movement_id = service
            .receive_inventory(&sku, &location_id, quantity, &user.username, reference.clone())
            .await
            .map_err(|e| Error::new(format!("Failed to receive inventory: {}", e)))?;

        // Fetch the movement
        let movement = sqlx::query_as::<_, persistence::InventoryMovementRecord>(
            "SELECT * FROM inventory_movements WHERE id = ?"
        )
        .bind(&movement_id)
        .fetch_one(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to fetch movement: {}", e)))?;

        // Audit
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Received {} units of {} at {}", quantity, sku, location_id),
            Some("inventory"),
            Some(&movement_id),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(InventoryMovement::from(movement))
    }

    /// Transfer inventory between locations (requires warehouse.inventory permission).
    async fn transfer_inventory(
        &self,
        ctx: &Context<'_>,
        sku: String,
        from_location_id: String,
        to_location_id: String,
        quantity: i64,
        reference: Option<String>,
    ) -> Result<InventoryMovement> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("inventory.write").await?;

        let service = InventoryService::new(gql_ctx.pool.inner().clone());
        let movement_id = service
            .transfer_inventory(
                &sku,
                &from_location_id,
                &to_location_id,
                quantity,
                &user.username,
                reference.clone(),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to transfer inventory: {}", e)))?;

        // Fetch the movement
        let movement = sqlx::query_as::<_, persistence::InventoryMovementRecord>(
            "SELECT * FROM inventory_movements WHERE id = ?"
        )
        .bind(&movement_id)
        .fetch_one(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to fetch movement: {}", e)))?;

        // Audit
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!(
                "Transferred {} units of {} from {} to {}",
                quantity, sku, from_location_id, to_location_id
            ),
            Some("inventory"),
            Some(&movement_id),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(InventoryMovement::from(movement))
    }

    /// Adjust inventory quantity (requires warehouse.inventory permission).
    async fn adjust_inventory(
        &self,
        ctx: &Context<'_>,
        sku: String,
        location_id: String,
        quantity: i64,
        reason: String,
        reference: Option<String>,
    ) -> Result<InventoryMovement> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("inventory.adjust").await?;

        let service = InventoryService::new(gql_ctx.pool.inner().clone());
        let movement_id = service
            .adjust_inventory(
                &sku,
                &location_id,
                quantity,
                &reason,
                &user.username,
                reference.clone(),
            )
            .await
            .map_err(|e| Error::new(format!("Failed to adjust inventory: {}", e)))?;

        // Fetch the movement
        let movement = sqlx::query_as::<_, persistence::InventoryMovementRecord>(
            "SELECT * FROM inventory_movements WHERE id = ?"
        )
        .bind(&movement_id)
        .fetch_one(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to fetch movement: {}", e)))?;

        // Audit
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!(
                "Adjusted {} by {} units at {} (reason: {})",
                sku, quantity, location_id, reason
            ),
            Some("inventory"),
            Some(&movement_id),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(InventoryMovement::from(movement))
    }

    /// Bulk adjust inventory (requires inventory.adjust permission).
    ///
    /// Processes multiple inventory adjustments in a single request.
    /// Each adjustment is independent - failures in one row do not affect others.
    /// Returns per-row success/error results.
    async fn bulk_adjust_inventory(
        &self,
        ctx: &Context<'_>,
        inputs: Vec<AdjustInventoryInput>,
    ) -> Result<BulkMutationResult<InventoryMovement>> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission once upfront
        gql_ctx.require_permission("inventory.adjust").await?;

        let service = InventoryService::new(gql_ctx.pool.inner().clone());
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut failure_count = 0;

        // Process each input independently
        for (index, input) in inputs.iter().enumerate() {
            // Validate input
            let mut field_errors = Vec::new();

            if let Some(err) = validate_not_empty("sku", &input.sku) {
                field_errors.push(err);
            }
            if let Some(err) = validate_not_empty("location_id", &input.location_id) {
                field_errors.push(err);
            }
            if let Some(err) = validate_not_empty("reason", &input.reason) {
                field_errors.push(err);
            }
            if input.quantity == 0 {
                field_errors.push(FieldError {
                    field: "quantity".to_string(),
                    message: "Adjustment quantity cannot be zero".to_string(),
                });
            }

            if !field_errors.is_empty() {
                // Validation error for this row
                failure_count += 1;
                results.push(RowResult {
                    index: index as i32,
                    success: false,
                    data: None,
                    error: Some(StructuredError::validation(
                        "Invalid input for inventory adjustment",
                        field_errors,
                    )),
                });
                continue;
            }

            // Attempt the adjustment
            match service
                .adjust_inventory(
                    &input.sku,
                    &input.location_id,
                    input.quantity,
                    &input.reason,
                    &user.username,
                    input.reference.clone(),
                )
                .await
            {
                Ok(movement_id) => {
                    // Fetch the movement
                    match sqlx::query_as::<_, persistence::InventoryMovementRecord>(
                        "SELECT * FROM inventory_movements WHERE id = ?"
                    )
                    .bind(&movement_id)
                    .fetch_one(gql_ctx.pool.inner())
                    .await
                    {
                        Ok(movement) => {
                            success_count += 1;
                            results.push(RowResult {
                                index: index as i32,
                                success: true,
                                data: Some(InventoryMovement::from(movement)),
                                error: None,
                            });
                        }
                        Err(e) => {
                            failure_count += 1;
                            results.push(RowResult {
                                index: index as i32,
                                success: false,
                                data: None,
                                error: Some(StructuredError::internal(
                                    format!("Failed to fetch movement: {}", e)
                                )),
                            });
                        }
                    }
                }
                Err(e) => {
                    // Service error for this row
                    failure_count += 1;
                    let error_msg = e.to_string();
                    let error = if error_msg.contains("negative inventory") {
                        StructuredError::validation(
                            error_msg,
                            vec![FieldError {
                                field: "quantity".to_string(),
                                message: "Adjustment would result in negative inventory".to_string(),
                            }],
                        )
                    } else if error_msg.contains("non-existent inventory") {
                        StructuredError::not_found(error_msg)
                    } else {
                        StructuredError::internal(error_msg)
                    };

                    results.push(RowResult {
                        index: index as i32,
                        success: false,
                        data: None,
                        error: Some(error),
                    });
                }
            }
        }

        // Record audit event for the bulk operation
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!(
                "Bulk inventory adjustment: {} successful, {} failed out of {} total",
                success_count, failure_count, inputs.len()
            ),
            Some("inventory"),
            None,
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(BulkMutationResult {
            success_count,
            failure_count,
            results,
        })
    }

    // ========================================================================
    // Warehouse Receiving Mutations
    // ========================================================================

    /// Create a new receiving task from a purchase order.
    async fn create_receiving_task(
        &self,
        ctx: &Context<'_>,
        input: CreateReceivingTaskInput,
    ) -> Result<ReceivingTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse PO ID
        let po_id = input.po_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid PO ID format"))?;

        // For now, we need to fetch the PO from somewhere
        // Since we don't have a PO repository yet, we'll create a minimal task
        // In a real implementation, this would fetch the PO and create lines from it

        // Create a new receiving task
        let task = warehouse::receiving::ReceivingTask {
            id: uuid::Uuid::new_v4(),
            task_number: input.task_number.clone(),
            po_id,
            po_number: format!("PO-{}", po_id), // Placeholder
            status: warehouse::receiving::ReceivingTaskStatus::Pending,
            lines: Vec::new(), // Would be populated from PO lines
            assigned_to: Some(user.username.clone()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Store the task
        let task_id = task.id;
        gql_ctx.receiving_tasks.write()
            .map_err(|_| Error::new("Failed to write receiving tasks"))?
            .insert(task_id, task.clone());

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Created receiving task {}", input.task_number),
            Some("receiving_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ReceivingTask::from(task))
    }

    /// Record receipt of items for a receiving line.
    async fn record_receipt(
        &self,
        ctx: &Context<'_>,
        input: RecordReceiptInput,
    ) -> Result<ReceivingLine> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let line_id = input.line_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid line ID format"))?;

        // Get and update the task
        let line = {
            let mut tasks = gql_ctx.receiving_tasks.write()
                .map_err(|_| Error::new("Failed to write receiving tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Receiving task not found"))?;

            // Record receipt
            task.record_receipt(line_id, input.quantity as u32)
                .map_err(|e| Error::new(format!("Failed to record receipt: {}", e)))?;

            // Find the updated line
            task.lines.iter()
                .find(|l| l.id == line_id)
                .ok_or_else(|| Error::new("Line not found"))?
                .clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Recorded receipt of {} units for line {}", input.quantity, line_id),
            Some("receiving_lines"),
            Some(&line_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ReceivingLine::from(line))
    }

    /// Record QC outcome for a receiving line.
    async fn record_qc_outcome(
        &self,
        ctx: &Context<'_>,
        input: RecordQCOutcomeInput,
    ) -> Result<ReceivingLine> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let line_id = input.line_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid line ID format"))?;

        // Get and update the task
        let (line, domain_outcome) = {
            let mut tasks = gql_ctx.receiving_tasks.write()
                .map_err(|_| Error::new("Failed to write receiving tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Receiving task not found"))?;

            // Record QC result
            let domain_outcome: warehouse::receiving::QCOutcome = input.outcome.into();
            task.record_qc_result(line_id, domain_outcome, input.inspector.clone(), input.notes.clone())
                .map_err(|e| Error::new(format!("Failed to record QC outcome: {}", e)))?;

            // Find the updated line
            let line = task.lines.iter()
                .find(|l| l.id == line_id)
                .ok_or_else(|| Error::new("Line not found"))?
                .clone();

            (line, domain_outcome)
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Recorded QC outcome {:?} for line {}", domain_outcome, line_id),
            Some("receiving_lines"),
            Some(&line_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ReceivingLine::from(line))
    }

    /// Place a receiving line in quarantine.
    async fn quarantine_line(
        &self,
        ctx: &Context<'_>,
        input: QuarantineLineInput,
    ) -> Result<ReceivingLine> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let line_id = input.line_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid line ID format"))?;

        // Get and update the task
        let line = {
            let mut tasks = gql_ctx.receiving_tasks.write()
                .map_err(|_| Error::new("Failed to write receiving tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Receiving task not found"))?;

            // Place in quarantine
            task.place_in_quarantine(line_id)
                .map_err(|e| Error::new(format!("Failed to quarantine line: {}", e)))?;

            // Find the updated line
            task.lines.iter()
                .find(|l| l.id == line_id)
                .ok_or_else(|| Error::new("Line not found"))?
                .clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Placed line {} in quarantine", line_id),
            Some("receiving_lines"),
            Some(&line_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ReceivingLine::from(line))
    }

    /// Release a receiving line from quarantine and update inventory.
    async fn release_from_quarantine(
        &self,
        ctx: &Context<'_>,
        input: ReleaseFromQuarantineInput,
    ) -> Result<ReceivingLine> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let line_id = input.line_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid line ID format"))?;

        // Get the task for warehouse service (before updating status)
        let task_for_service = {
            let tasks = gql_ctx.receiving_tasks.read()
                .map_err(|_| Error::new("Failed to read receiving tasks"))?;

            tasks.get(&task_id)
                .ok_or_else(|| Error::new("Receiving task not found"))?
                .clone()
        };

        // Update inventory via warehouse service (validates quarantine status)
        let warehouse_service = persistence::WarehouseService::new(gql_ctx.pool.inner().clone());
        warehouse_service
            .release_from_quarantine_and_update_inventory(
                &task_for_service,
                line_id,
                &input.location_id,
                &user.username,
            )
            .await
            .map_err(|e| Error::new(format!("Failed to update inventory: {}", e)))?;

        // Now update the task status
        let line = {
            let mut tasks = gql_ctx.receiving_tasks.write()
                .map_err(|_| Error::new("Failed to write receiving tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Receiving task not found"))?;

            // Release from quarantine
            task.release_from_quarantine(line_id)
                .map_err(|e| Error::new(format!("Failed to release from quarantine: {}", e)))?;

            // Find the updated line
            task.lines.iter()
                .find(|l| l.id == line_id)
                .ok_or_else(|| Error::new("Line not found"))?
                .clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Released line {} from quarantine to location {}", line_id, input.location_id),
            Some("receiving_lines"),
            Some(&line_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ReceivingLine::from(line))
    }

    /// Acknowledge a discrepancy on a receiving line.
    async fn acknowledge_discrepancy(
        &self,
        ctx: &Context<'_>,
        input: AcknowledgeDiscrepancyInput,
    ) -> Result<ReceivingLine> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let line_id = input.line_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid line ID format"))?;
        let discrepancy_id = input.discrepancy_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid discrepancy ID format"))?;

        // Get and update the task
        let line = {
            let mut tasks = gql_ctx.receiving_tasks.write()
                .map_err(|_| Error::new("Failed to write receiving tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Receiving task not found"))?;

            // Acknowledge discrepancy
            task.acknowledge_discrepancy(line_id, discrepancy_id)
                .map_err(|e| Error::new(format!("Failed to acknowledge discrepancy: {}", e)))?;

            // Find the updated line
            task.lines.iter()
                .find(|l| l.id == line_id)
                .ok_or_else(|| Error::new("Line not found"))?
                .clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Acknowledged discrepancy {} on line {}", discrepancy_id, line_id),
            Some("receiving_discrepancies"),
            Some(&discrepancy_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ReceivingLine::from(line))
    }

    /// Complete a receiving task and update inventory.
    async fn complete_receiving_task(
        &self,
        ctx: &Context<'_>,
        task_id: ID,
        location_id: String,
    ) -> Result<ReceivingTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse task ID
        let task_uuid = task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;

        // Get and update the task, then release the lock before async operations
        let completed_task = {
            let mut tasks = gql_ctx.receiving_tasks.write()
                .map_err(|_| Error::new("Failed to write receiving tasks"))?;

            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Receiving task not found"))?;

            // Complete the task
            task.complete()
                .map_err(|e| Error::new(format!("Failed to complete task: {}", e)))?;

            task.clone()
        }; // Lock is dropped here

        // Update inventory via warehouse service
        let warehouse_service = persistence::WarehouseService::new(gql_ctx.pool.inner().clone());
        warehouse_service
            .complete_receiving_task(&completed_task, &location_id, &user.username)
            .await
            .map_err(|e| Error::new(format!("Failed to update inventory: {}", e)))?;

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Completed receiving task {}", completed_task.task_number),
            Some("receiving_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ReceivingTask::from(completed_task))
    }

    // ========================================================================
    // Warehouse Picking Mutations
    // ========================================================================

    /// Create a new pick task.
    async fn create_pick_task(
        &self,
        ctx: &Context<'_>,
        input: CreatePickTaskInput,
    ) -> Result<PickTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse order IDs
        let order_ids: Result<Vec<uuid::Uuid>> = input.order_ids
            .iter()
            .map(|id| id.parse::<uuid::Uuid>()
                .map_err(|_| Error::new("Invalid order ID format")))
            .collect();
        let order_ids = order_ids?;

        // Create a new pick task
        let task = warehouse::picking::PickTask::new(
            input.task_number.clone(),
            input.task_type.into(),
            order_ids,
        )
        .map_err(|e| Error::new(format!("Failed to create pick task: {}", e)))?;

        // Store the task
        let task_id = task.id;
        gql_ctx.pick_tasks.write()
            .map_err(|_| Error::new("Failed to write pick tasks"))?
            .insert(task_id, task.clone());

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Created pick task {}", input.task_number),
            Some("pick_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PickTask::from(task))
    }

    /// Add a line to a pick task.
    async fn add_pick_line(
        &self,
        ctx: &Context<'_>,
        input: AddPickLineInput,
    ) -> Result<PickTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let order_line_id = input.order_line_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid order line ID format"))?;

        // Get and update the task
        let updated_task = {
            let mut tasks = gql_ctx.pick_tasks.write()
                .map_err(|_| Error::new("Failed to write pick tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Pick task not found"))?;

            // Add the line
            task.add_line(
                order_line_id,
                input.sku.clone(),
                input.location_id.clone(),
                input.quantity as u32,
            )
            .map_err(|e| Error::new(format!("Failed to add pick line: {}", e)))?;

            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Added line to pick task {}", updated_task.task_number),
            Some("pick_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PickTask::from(updated_task))
    }

    /// Start a pick task.
    async fn start_pick_task(
        &self,
        ctx: &Context<'_>,
        task_id: ID,
        operator: String,
    ) -> Result<PickTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse task ID
        let task_uuid = task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;

        // Get and update the task
        let updated_task = {
            let mut tasks = gql_ctx.pick_tasks.write()
                .map_err(|_| Error::new("Failed to write pick tasks"))?;

            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Pick task not found"))?;

            // Start the task
            task.start(operator.clone())
                .map_err(|e| Error::new(format!("Failed to start task: {}", e)))?;

            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Started pick task {}", updated_task.task_number),
            Some("pick_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PickTask::from(updated_task))
    }

    /// Record a successful pick.
    async fn record_pick(
        &self,
        ctx: &Context<'_>,
        input: RecordPickInput,
    ) -> Result<PickLine> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let line_id = input.line_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid line ID format"))?;

        // Get and update the task
        let updated_line = {
            let mut tasks = gql_ctx.pick_tasks.write()
                .map_err(|_| Error::new("Failed to write pick tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Pick task not found"))?;

            // Record the pick
            task.record_pick(line_id, input.quantity as u32)
                .map_err(|e| Error::new(format!("Failed to record pick: {}", e)))?;

            // Find and clone the updated line
            task.lines.iter()
                .find(|l| l.id == line_id)
                .ok_or_else(|| Error::new("Pick line not found"))?
                .clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Recorded pick for line {}", line_id),
            Some("pick_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PickLine::from(updated_line))
    }

    /// Record a pick exception.
    async fn record_pick_exception(
        &self,
        ctx: &Context<'_>,
        input: RecordPickExceptionInput,
    ) -> Result<PickLine> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let line_id = input.line_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid line ID format"))?;

        // Get and update the task
        let updated_line = {
            let mut tasks = gql_ctx.pick_tasks.write()
                .map_err(|_| Error::new("Failed to write pick tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Pick task not found"))?;

            // Record the exception
            task.record_exception(
                line_id,
                input.exception_type.into(),
                input.picked_quantity as u32,
                input.notes.clone(),
            )
            .map_err(|e| Error::new(format!("Failed to record exception: {}", e)))?;

            // Find and clone the updated line
            task.lines.iter()
                .find(|l| l.id == line_id)
                .ok_or_else(|| Error::new("Pick line not found"))?
                .clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Recorded pick exception for line {}", line_id),
            Some("pick_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PickLine::from(updated_line))
    }

    /// Complete a pick task.
    async fn complete_pick_task(
        &self,
        ctx: &Context<'_>,
        task_id: ID,
    ) -> Result<PickTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse task ID
        let task_uuid = task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;

        // Get and update the task
        let updated_task = {
            let mut tasks = gql_ctx.pick_tasks.write()
                .map_err(|_| Error::new("Failed to write pick tasks"))?;

            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Pick task not found"))?;

            // Complete the task
            task.complete()
                .map_err(|e| Error::new(format!("Failed to complete task: {}", e)))?;

            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Completed pick task {}", updated_task.task_number),
            Some("pick_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PickTask::from(updated_task))
    }

    /// Cancel a pick task.
    async fn cancel_pick_task(
        &self,
        ctx: &Context<'_>,
        task_id: ID,
    ) -> Result<PickTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse task ID
        let task_uuid = task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;

        // Get and update the task
        let updated_task = {
            let mut tasks = gql_ctx.pick_tasks.write()
                .map_err(|_| Error::new("Failed to write pick tasks"))?;

            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Pick task not found"))?;

            // Cancel the task
            task.cancel()
                .map_err(|e| Error::new(format!("Failed to cancel task: {}", e)))?;

            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Cancelled pick task {}", updated_task.task_number),
            Some("pick_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PickTask::from(updated_task))
    }

    // ========================================================================
    // Warehouse Packing Mutations
    // ========================================================================

    /// Create a new pack task.
    async fn create_pack_task(
        &self,
        ctx: &Context<'_>,
        input: CreatePackTaskInput,
    ) -> Result<PackTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let pick_task_id = input.pick_task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid pick task ID format"))?;
        let order_id = input.order_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid order ID format"))?;

        // Create a new pack task
        let task = warehouse::packing::PackTask::new(
            input.task_number.clone(),
            pick_task_id,
            order_id,
        );

        // Store the task
        let task_id = task.id;
        gql_ctx.pack_tasks.write()
            .map_err(|_| Error::new("Failed to write pack tasks"))?
            .insert(task_id, task.clone());

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Created pack task {}", input.task_number),
            Some("pack_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PackTask::from(task))
    }

    /// Add a line to a pack task.
    async fn add_pack_line(
        &self,
        ctx: &Context<'_>,
        input: AddPackLineInput,
    ) -> Result<PackTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let pick_line_id = input.pick_line_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid pick line ID format"))?;

        // Get and update the task
        let updated_task = {
            let mut tasks = gql_ctx.pack_tasks.write()
                .map_err(|_| Error::new("Failed to write pack tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Pack task not found"))?;

            // Add the line
            task.add_line(
                pick_line_id,
                input.sku.clone(),
                input.quantity as u32,
            )
            .map_err(|e| Error::new(format!("Failed to add pack line: {}", e)))?;

            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Added line to pack task {}", updated_task.task_number),
            Some("pack_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PackTask::from(updated_task))
    }

    /// Start a pack task.
    async fn start_pack_task(
        &self,
        ctx: &Context<'_>,
        task_id: ID,
        operator: String,
    ) -> Result<PackTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse task ID
        let task_uuid = task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;

        // Get and update the task
        let updated_task = {
            let mut tasks = gql_ctx.pack_tasks.write()
                .map_err(|_| Error::new("Failed to write pack tasks"))?;

            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Pack task not found"))?;

            // Start the task
            task.start(operator.clone())
                .map_err(|e| Error::new(format!("Failed to start task: {}", e)))?;

            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Started pack task {}", updated_task.task_number),
            Some("pack_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PackTask::from(updated_task))
    }

    /// Create a package for a pack task.
    async fn create_package(
        &self,
        ctx: &Context<'_>,
        input: CreatePackageInput,
    ) -> Result<Package> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse task ID
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;

        // Get and update the task
        let created_package = {
            let mut tasks = gql_ctx.pack_tasks.write()
                .map_err(|_| Error::new("Failed to write pack tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Pack task not found"))?;

            // Create the package
            let package_id = task.create_package(
                input.package_number.clone(),
                input.carton_type.clone(),
            )
            .map_err(|e| Error::new(format!("Failed to create package: {}", e)))?;

            // Find and clone the created package
            task.packages.iter()
                .find(|p| p.id == package_id)
                .ok_or_else(|| Error::new("Package not found"))?
                .clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Created package {}", input.package_number),
            Some("pack_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(Package::from(created_package))
    }

    /// Record packing of items into a package.
    async fn record_pack(
        &self,
        ctx: &Context<'_>,
        input: RecordPackInput,
    ) -> Result<PackLine> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let line_id = input.line_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid line ID format"))?;
        let package_id = input.package_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid package ID format"))?;

        // Get and update the task
        let updated_line = {
            let mut tasks = gql_ctx.pack_tasks.write()
                .map_err(|_| Error::new("Failed to write pack tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Pack task not found"))?;

            // Record the pack
            task.record_pack(
                line_id,
                package_id,
                input.quantity as u32,
                input.serial_numbers.clone(),
            )
            .map_err(|e| Error::new(format!("Failed to record pack: {}", e)))?;

            // Find and clone the updated line
            task.lines.iter()
                .find(|l| l.id == line_id)
                .ok_or_else(|| Error::new("Pack line not found"))?
                .clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Recorded pack for line {}", line_id),
            Some("pack_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PackLine::from(updated_line))
    }

    /// Set package dimensions.
    async fn set_package_dimensions(
        &self,
        ctx: &Context<'_>,
        input: SetPackageDimensionsInput,
    ) -> Result<Package> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let package_id = input.package_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid package ID format"))?;

        // Get and update the task
        let updated_package = {
            let mut tasks = gql_ctx.pack_tasks.write()
                .map_err(|_| Error::new("Failed to write pack tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Pack task not found"))?;

            // Set dimensions
            task.set_package_dimensions(
                package_id,
                input.length,
                input.width,
                input.height,
            )
            .map_err(|e| Error::new(format!("Failed to set dimensions: {}", e)))?;

            // Find and clone the updated package
            task.packages.iter()
                .find(|p| p.id == package_id)
                .ok_or_else(|| Error::new("Package not found"))?
                .clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Set dimensions for package {}", package_id),
            Some("pack_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(Package::from(updated_package))
    }

    /// Set package weight.
    async fn set_package_weight(
        &self,
        ctx: &Context<'_>,
        input: SetPackageWeightInput,
    ) -> Result<Package> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let package_id = input.package_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid package ID format"))?;

        // Get and update the task
        let updated_package = {
            let mut tasks = gql_ctx.pack_tasks.write()
                .map_err(|_| Error::new("Failed to write pack tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Pack task not found"))?;

            // Set weight
            task.set_package_weight(package_id, input.weight)
                .map_err(|e| Error::new(format!("Failed to set weight: {}", e)))?;

            // Find and clone the updated package
            task.packages.iter()
                .find(|p| p.id == package_id)
                .ok_or_else(|| Error::new("Package not found"))?
                .clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Set weight for package {}", package_id),
            Some("pack_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(Package::from(updated_package))
    }

    /// Assign tracking number to package.
    async fn assign_tracking(
        &self,
        ctx: &Context<'_>,
        input: AssignTrackingInput,
    ) -> Result<Package> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse IDs
        let task_id = input.task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;
        let package_id = input.package_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid package ID format"))?;

        // Get and update the task
        let updated_package = {
            let mut tasks = gql_ctx.pack_tasks.write()
                .map_err(|_| Error::new("Failed to write pack tasks"))?;

            let task = tasks.get_mut(&task_id)
                .ok_or_else(|| Error::new("Pack task not found"))?;

            // Assign tracking
            task.assign_tracking(package_id, input.tracking_number.clone())
                .map_err(|e| Error::new(format!("Failed to assign tracking: {}", e)))?;

            // Find and clone the updated package
            task.packages.iter()
                .find(|p| p.id == package_id)
                .ok_or_else(|| Error::new("Package not found"))?
                .clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Assigned tracking {} to package {}", input.tracking_number, package_id),
            Some("pack_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(Package::from(updated_package))
    }

    /// Complete a pack task.
    async fn complete_pack_task(
        &self,
        ctx: &Context<'_>,
        task_id: ID,
    ) -> Result<PackTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse task ID
        let task_uuid = task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;

        // Get and update the task
        let updated_task = {
            let mut tasks = gql_ctx.pack_tasks.write()
                .map_err(|_| Error::new("Failed to write pack tasks"))?;

            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Pack task not found"))?;

            // Complete the task
            task.complete()
                .map_err(|e| Error::new(format!("Failed to complete task: {}", e)))?;

            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Completed pack task {}", updated_task.task_number),
            Some("pack_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PackTask::from(updated_task))
    }

    /// Cancel a pack task.
    async fn cancel_pack_task(
        &self,
        ctx: &Context<'_>,
        task_id: ID,
    ) -> Result<PackTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;

        // Check permission
        gql_ctx.require_permission("warehouse.execute").await?;

        // Parse task ID
        let task_uuid = task_id.parse::<uuid::Uuid>()
            .map_err(|_| Error::new("Invalid task ID format"))?;

        // Get and update the task
        let updated_task = {
            let mut tasks = gql_ctx.pack_tasks.write()
                .map_err(|_| Error::new("Failed to write pack tasks"))?;

            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Pack task not found"))?;

            // Cancel the task
            task.cancel()
                .map_err(|e| Error::new(format!("Failed to cancel task: {}", e)))?;

            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Cancelled pack task {}", updated_task.task_number),
            Some("pack_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(PackTask::from(updated_task))
    }

    // ========================================================================
    // Ship Mutations
    // ========================================================================

    /// Create a new ship task.
    async fn create_ship_task(
        &self,
        ctx: &Context<'_>,
        input: CreateShipTaskInput,
    ) -> Result<ShipTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;
        gql_ctx.require_permission("warehouse.execute").await?;

        let task = warehouse::shipping::ShipTask::new(input.task_number.clone());
        let task_id = task.id;

        {
            let mut tasks = gql_ctx.ship_tasks.write()
                .map_err(|_| Error::new("Failed to write ship tasks"))?;
            tasks.insert(task_id, task.clone());
        } // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Created ship task {}", input.task_number),
            Some("ship_tasks"),
            Some(&task_id.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ShipTask::from(task))
    }

    /// Add a line to a ship task.
    async fn add_ship_line(
        &self,
        ctx: &Context<'_>,
        input: AddShipLineInput,
    ) -> Result<ShipTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;
        gql_ctx.require_permission("warehouse.execute").await?;

        let task_uuid = Uuid::parse_str(&input.task_id).map_err(|_| Error::new("Invalid task ID"))?;
        let pack_task_uuid = Uuid::parse_str(&input.pack_task_id).map_err(|_| Error::new("Invalid pack task ID"))?;
        let order_uuid = Uuid::parse_str(&input.order_id).map_err(|_| Error::new("Invalid order ID"))?;
        let package_uuids: Result<Vec<Uuid>> = input.package_ids.iter()
            .map(|id| Uuid::parse_str(id).map_err(|_| Error::new("Invalid package ID")))
            .collect();
        let package_uuids = package_uuids?;

        let updated_task = {
            let mut tasks = gql_ctx.ship_tasks.write()
                .map_err(|_| Error::new("Failed to write ship tasks"))?;
            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Ship task not found"))?;
            task.add_line(pack_task_uuid, package_uuids, order_uuid)
                .map_err(|e| Error::new(format!("Failed to add line: {}", e)))?;
            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Added line to ship task {}", updated_task.task_number),
            Some("ship_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ShipTask::from(updated_task))
    }

    /// Start a ship task (move to staging).
    async fn start_ship_task(
        &self,
        ctx: &Context<'_>,
        task_id: ID,
        operator: String,
    ) -> Result<ShipTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;
        gql_ctx.require_permission("warehouse.execute").await?;

        let task_uuid = Uuid::parse_str(&task_id).map_err(|_| Error::new("Invalid task ID"))?;

        let updated_task = {
            let mut tasks = gql_ctx.ship_tasks.write()
                .map_err(|_| Error::new("Failed to write ship tasks"))?;
            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Ship task not found"))?;
            task.start(operator.clone())
                .map_err(|e| Error::new(format!("Failed to start task: {}", e)))?;
            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Started ship task {} (operator: {})", updated_task.task_number, operator),
            Some("ship_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ShipTask::from(updated_task))
    }

    /// Evaluate compliance gates for a ship task.
    async fn evaluate_ship_compliance(
        &self,
        ctx: &Context<'_>,
        task_id: ID,
    ) -> Result<ComplianceBlock> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;
        gql_ctx.require_permission("warehouse.execute").await?;

        let task_uuid = Uuid::parse_str(&task_id).map_err(|_| Error::new("Invalid task ID"))?;

        let compliance_result = {
            let mut tasks = gql_ctx.ship_tasks.write()
                .map_err(|_| Error::new("Failed to write ship tasks"))?;
            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Ship task not found"))?;
            task.evaluate_compliance()
                .map_err(|e| Error::new(format!("Failed to evaluate compliance: {}", e)))?
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Evaluated compliance for ship task (blocked: {})", compliance_result.blocked),
            Some("ship_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ComplianceBlock::from(compliance_result))
    }

    /// Dispatch a shipment.
    async fn dispatch_shipment(
        &self,
        ctx: &Context<'_>,
        input: DispatchShipmentInput,
    ) -> Result<ShipTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;
        gql_ctx.require_permission("warehouse.execute").await?;

        let task_uuid = Uuid::parse_str(&input.task_id).map_err(|_| Error::new("Invalid task ID"))?;

        let updated_task = {
            let mut tasks = gql_ctx.ship_tasks.write()
                .map_err(|_| Error::new("Failed to write ship tasks"))?;
            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Ship task not found"))?;
            task.dispatch(input.carrier.clone(), input.tracking_number.clone())
                .map_err(|e| Error::new(format!("Failed to dispatch: {}", e)))?;
            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Dispatched ship task {} (carrier: {}, tracking: {})",
                updated_task.task_number, input.carrier, input.tracking_number),
            Some("ship_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ShipTask::from(updated_task))
    }

    /// Cancel a ship task.
    async fn cancel_ship_task(
        &self,
        ctx: &Context<'_>,
        task_id: ID,
    ) -> Result<ShipTask> {
        let gql_ctx = ctx.data::<GraphQLContext>()?;
        let user = gql_ctx.require_user()?;
        gql_ctx.require_permission("warehouse.execute").await?;

        let task_uuid = Uuid::parse_str(&task_id).map_err(|_| Error::new("Invalid task ID"))?;

        let updated_task = {
            let mut tasks = gql_ctx.ship_tasks.write()
                .map_err(|_| Error::new("Failed to write ship tasks"))?;
            let task = tasks.get_mut(&task_uuid)
                .ok_or_else(|| Error::new("Ship task not found"))?;
            task.cancel()
                .map_err(|e| Error::new(format!("Failed to cancel task: {}", e)))?;
            task.clone()
        }; // Lock is dropped here

        // Record audit event
        let _ = record_audit_event(
            &gql_ctx.pool,
            Some(user.id),
            AuditEventType::InventoryAdjusted,
            &format!("Cancelled ship task {}", updated_task.task_number),
            Some("ship_tasks"),
            Some(&task_uuid.to_string()),
            None,
            None,
            None,
            None,
        )
        .await;

        Ok(ShipTask::from(updated_task))
    }
}

fn seed_mock_tasks(gql_ctx: &GraphQLContext) -> Result<MockTaskSummary> {
    let supplier_id = mock_uuid(0x201)?;

    let mut receiving_tasks = Vec::new();
    let mut pick_tasks = Vec::new();
    let mut pack_tasks = Vec::new();
    let mut ship_tasks = Vec::new();

    for index in 1..=20 {
        let order_id = mock_uuid(0x300 + index as u16)?;
        let order_line_id = mock_uuid(0x400 + index as u16)?;
        let po_number = format!("PO-{:04}", index);
        let mut purchase_order = PurchaseOrder::new(po_number, supplier_id);
        purchase_order
            .add_line(format!("SKU-{:03}", index), 5, 250.0)
            .map_err(|e| Error::new(format!("Failed to add PO line: {}", e)))?;

        let task_number = format!("RCV-{:04}", index);
        let mut receiving_task = warehouse::receiving::ReceivingTask::from_purchase_order(
            &purchase_order,
            task_number,
        )
        .map_err(|e| Error::new(format!("Failed to build receiving task: {}", e)))?;

        if index % 2 == 0 {
            receiving_task
                .start()
                .map_err(|e| Error::new(format!("Failed to start receiving task: {}", e)))?;
            if let Some(line_id) = receiving_task.lines.first().map(|line| line.id) {
                receiving_task
                    .record_receipt(line_id, 5)
                    .map_err(|e| Error::new(format!("Failed to record receipt: {}", e)))?;
                receiving_task
                    .record_qc_result(
                        line_id,
                        warehouse::receiving::QCOutcome::Pass,
                        "Inspector".to_string(),
                        None,
                    )
                    .map_err(|e| Error::new(format!("Failed to record QC result: {}", e)))?;
            }
        }

        receiving_tasks.push(receiving_task);

        let mut pick_task = warehouse::picking::PickTask::new(
            format!("PICK-{:04}", index),
            warehouse::picking::PickTaskType::SingleOrder,
            vec![order_id],
        )
        .map_err(|e| Error::new(format!("Failed to create pick task: {}", e)))?;
        pick_task
            .add_line(
                order_line_id,
                format!("SKU-{:03}", index),
                format!("LOC-{:03}", index),
                5,
            )
            .map_err(|e| Error::new(format!("Failed to add pick line: {}", e)))?;

        let pick_line_id = pick_task
            .lines
            .first()
            .map(|line| line.id)
            .ok_or_else(|| Error::new("Pick task has no lines"))?;

        let mut pack_task = warehouse::packing::PackTask::new(
            format!("PACK-{:04}", index),
            pick_task.id,
            order_id,
        );
        pack_task
            .add_line(pick_line_id, format!("SKU-{:03}", index), 5)
            .map_err(|e| Error::new(format!("Failed to add pack line: {}", e)))?;
        pack_task
            .start("Casey".to_string())
            .map_err(|e| Error::new(format!("Failed to start pack task: {}", e)))?;
        let package_id = pack_task
            .create_package(format!("PKG-{:04}", index), "Carton-A".to_string())
            .map_err(|e| Error::new(format!("Failed to create package: {}", e)))?;
        let pack_line_id = pack_task
            .lines
            .first()
            .map(|line| line.id)
            .ok_or_else(|| Error::new("Pack task has no lines"))?;
        pack_task
            .record_pack(pack_line_id, package_id, 5, None)
            .map_err(|e| Error::new(format!("Failed to record pack: {}", e)))?;

        let mut ship_task = warehouse::shipping::ShipTask::new(format!("SHIP-{:04}", index));
        ship_task
            .add_line(pack_task.id, vec![package_id], order_id)
            .map_err(|e| Error::new(format!("Failed to add ship line: {}", e)))?;
        ship_task
            .start("Robin".to_string())
            .map_err(|e| Error::new(format!("Failed to start ship task: {}", e)))?;
        let _ = ship_task.evaluate_compliance();

        pick_tasks.push(pick_task);
        pack_tasks.push(pack_task);
        ship_tasks.push(ship_task);
    }

    let mut receiving_store = gql_ctx
        .receiving_tasks
        .write()
        .map_err(|_| Error::new("Failed to lock receiving tasks"))?;
    receiving_store.clear();
    for task in receiving_tasks {
        receiving_store.insert(task.id, task);
    }
    drop(receiving_store);

    let mut pick_store = gql_ctx
        .pick_tasks
        .write()
        .map_err(|_| Error::new("Failed to lock pick tasks"))?;
    pick_store.clear();
    for task in pick_tasks {
        pick_store.insert(task.id, task);
    }
    drop(pick_store);

    let mut pack_store = gql_ctx
        .pack_tasks
        .write()
        .map_err(|_| Error::new("Failed to lock pack tasks"))?;
    pack_store.clear();
    for task in pack_tasks {
        pack_store.insert(task.id, task);
    }
    drop(pack_store);

    let mut ship_store = gql_ctx
        .ship_tasks
        .write()
        .map_err(|_| Error::new("Failed to lock ship tasks"))?;
    ship_store.clear();
    for task in ship_tasks {
        ship_store.insert(task.id, task);
    }
    drop(ship_store);

    Ok(MockTaskSummary {
        receiving_tasks: 20,
        pick_tasks: 20,
        pack_tasks: 20,
        ship_tasks: 20,
    })
}

fn mock_uuid(suffix: u16) -> Result<Uuid> {
    let value = format!("00000000-0000-0000-0000-00000000{:04x}", suffix);
    Uuid::parse_str(&value).map_err(|_| Error::new("Failed to parse mock UUID"))
}
