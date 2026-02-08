//! GraphQL types and models.

use async_graphql::*;
use auth::{Permission as AuthPermission, Role as AuthRole, User as AuthUser};
use sqlx::FromRow;

/// Role type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct Role {
    pub id: ID,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<AuthRole> for Role {
    fn from(role: AuthRole) -> Self {
        Self {
            id: ID(role.id.to_string()),
            name: role.name,
            description: role.description,
            created_at: role.created_at,
            updated_at: role.updated_at,
        }
    }
}

/// Permission type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct Permission {
    pub id: ID,
    pub name: String,
    pub description: Option<String>,
    pub resource: String,
    pub action: String,
    pub created_at: String,
}

impl From<AuthPermission> for Permission {
    fn from(permission: AuthPermission) -> Self {
        Self {
            id: ID(permission.id.to_string()),
            name: permission.name,
            description: permission.description,
            resource: permission.resource,
            action: permission.action,
            created_at: permission.created_at,
        }
    }
}

/// User type for GraphQL.
#[derive(Clone, SimpleObject)]
#[graphql(complex)]
pub struct User {
    pub id: ID,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[ComplexObject]
impl User {
    /// Get the roles assigned to this user.
    async fn roles(&self, ctx: &Context<'_>) -> Result<Vec<Role>> {
        let gql_ctx = ctx.data::<crate::GraphQLContext>()?;
        let user_id: i64 = self.id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;

        let auth_roles = auth::get_user_roles(&gql_ctx.pool, user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to get user roles: {}", e)))?;

        Ok(auth_roles.into_iter().map(Role::from).collect())
    }

    /// Get the permissions assigned to this user.
    async fn permissions(&self, ctx: &Context<'_>) -> Result<Vec<Permission>> {
        let gql_ctx = ctx.data::<crate::GraphQLContext>()?;
        let user_id: i64 = self.id.parse()
            .map_err(|_| Error::new("Invalid user ID"))?;

        let auth_permissions = auth::get_user_permissions(&gql_ctx.pool, user_id)
            .await
            .map_err(|e| Error::new(format!("Failed to get user permissions: {}", e)))?;

        Ok(auth_permissions.into_iter().map(Permission::from).collect())
    }
}

impl From<AuthUser> for User {
    fn from(user: AuthUser) -> Self {
        Self {
            id: ID(user.id.to_string()),
            username: user.username,
            email: user.email,
            is_active: user.is_active,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Login response containing user and session token.
#[derive(SimpleObject)]
pub struct LoginResponse {
    /// The authenticated user.
    pub user: User,
    /// Session token for subsequent requests.
    pub token: String,
}

/// Passkey challenge response for WebAuthn.
#[derive(SimpleObject)]
pub struct PasskeyChallenge {
    /// Challenge ID for completing the flow.
    #[graphql(name = "challengeId")]
    pub challenge_id: String,
    /// WebAuthn options JSON for the client.
    #[graphql(name = "optionsJson")]
    pub options_json: String,
}

/// Health check response.
#[derive(SimpleObject)]
pub struct HealthResponse {
    /// Status of the service.
    pub status: String,
    /// Version of the service.
    pub version: String,
    /// Database connection status.
    pub database: String,
}

/// Reserve inventory response.
#[derive(SimpleObject)]
pub struct ReserveResponse {
    /// Movement ID created for this reservation.
    pub movement_id: String,
    /// Quantity actually allocated (may be less than requested when allowPartial=true).
    pub allocated: i64,
}

/// Cycle count line type.
#[derive(Clone, SimpleObject, FromRow)]
pub struct CycleCountLine {
    pub id: String,
    pub session_id: String,
    pub sku: String,
    pub location_id: String,
    pub expected_qty: i64,
    pub counted_qty: i64,
    pub variance: i64,
    pub status: String,
}

/// Cycle count session type.
#[derive(Clone, SimpleObject, FromRow)]
#[graphql(complex)]
pub struct CycleCountSession {
    pub id: String,
    pub abc_class: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

#[ComplexObject]
impl CycleCountSession {
    async fn lines(&self, ctx: &Context<'_>) -> Result<Vec<CycleCountLine>> {
        let gql_ctx = ctx.data::<crate::GraphQLContext>()?;
        let rows = sqlx::query_as::<_, CycleCountLine>(
            r#"SELECT id, session_id, sku, location_id, expected_qty, counted_qty, variance, status
               FROM cycle_count_lines WHERE session_id = ? ORDER BY sku, location_id"#,
        )
        .bind(&self.id)
        .fetch_all(gql_ctx.pool.inner())
        .await
        .map_err(|e| Error::new(format!("Failed to load cycle count lines: {}", e)))?;

        Ok(rows)
    }
}

// ============================================================================
// Inventory Domain Types
// ============================================================================

/// Item type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct Item {
    pub id: ID,
    pub sku: String,
    pub name: String,
    pub name_zh: String,
    pub description: Option<String>,
    pub description_zh: Option<String>,
    pub item_type: String,
    pub tracking_mode: String,
    pub manufacturer_part_no: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<persistence::ItemRecord> for Item {
    fn from(record: persistence::ItemRecord) -> Self {
        Self {
            id: ID(record.id),
            sku: record.sku,
            name: record.name_en,
            name_zh: record.name_zh,
            description: record.description_en,
            description_zh: record.description_zh,
            item_type: record.item_type,
            tracking_mode: record.tracking_mode,
            manufacturer_part_no: record.manufacturer_part_no,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// Location type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct Location {
    pub id: ID,
    pub warehouse: String,
    pub zone: String,
    pub bin: String,
    pub created_at: String,
}

impl From<persistence::LocationRecord> for Location {
    fn from(record: persistence::LocationRecord) -> Self {
        Self {
            id: ID(record.id),
            warehouse: record.warehouse,
            zone: record.zone,
            bin: record.bin,
            created_at: record.created_at,
        }
    }
}

/// Inventory position type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct InventoryPosition {
    pub id: ID,
    pub sku: String,
    pub location_id: String,
    pub status: String,
    pub lot_id: Option<String>,
    pub quantity_on_hand: i64,
    pub quantity_reserved: i64,
    pub quantity_available: i64,
    pub last_updated: String,
}

impl From<persistence::InventoryPositionRecord> for InventoryPosition {
    fn from(record: persistence::InventoryPositionRecord) -> Self {
        Self {
            id: ID(record.id),
            sku: record.sku,
            location_id: record.location_id,
            status: record.status,
            lot_id: record.lot_id,
            quantity_on_hand: record.quantity_on_hand,
            quantity_reserved: record.quantity_reserved,
            quantity_available: record.quantity_available,
            last_updated: record.last_updated,
        }
    }
}

/// Inventory movement type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct InventoryMovement {
    pub id: ID,
    pub timestamp: String,
    pub actor: String,
    pub reason: String,
    pub from_location_id: Option<String>,
    pub to_location_id: Option<String>,
    pub sku: String,
    pub quantity: i64,
    pub serial_numbers: Option<Vec<String>>,
    pub lot_id: Option<String>,
    pub status_before: String,
    pub status_after: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

impl From<persistence::InventoryMovementRecord> for InventoryMovement {
    fn from(record: persistence::InventoryMovementRecord) -> Self {
        // Parse serial_numbers from JSON string if present
        let serial_numbers = record.serial_numbers.and_then(|s| {
            serde_json::from_str::<Vec<String>>(&s).ok()
        });

        Self {
            id: ID(record.id),
            timestamp: record.timestamp,
            actor: record.actor,
            reason: record.reason,
            from_location_id: record.from_location_id,
            to_location_id: record.to_location_id,
            sku: record.sku,
            quantity: record.quantity,
            serial_numbers,
            lot_id: record.lot_id,
            status_before: record.status_before,
            status_after: record.status_after,
            reference: record.reference,
            notes: record.notes,
        }
    }
}

/// Serial genealogy entry type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct SerialGenealogyEntry {
    pub movement_id: ID,
    pub timestamp: String,
    pub reason: String,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
    pub sku: String,
    pub status_before: String,
    pub status_after: String,
}

impl From<persistence::InventoryMovementRecord> for SerialGenealogyEntry {
    fn from(record: persistence::InventoryMovementRecord) -> Self {
        Self {
            movement_id: ID(record.id),
            timestamp: record.timestamp,
            reason: record.reason,
            from_location: record.from_location_id,
            to_location: record.to_location_id,
            sku: record.sku,
            status_before: record.status_before,
            status_after: record.status_after,
        }
    }
}

/// Lot genealogy entry type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct LotGenealogyEntry {
    pub movement_id: ID,
    pub timestamp: String,
    pub reason: String,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
    pub sku: String,
    pub quantity: i64,
    pub status_before: String,
    pub status_after: String,
}

impl From<persistence::InventoryMovementRecord> for LotGenealogyEntry {
    fn from(record: persistence::InventoryMovementRecord) -> Self {
        Self {
            movement_id: ID(record.id),
            timestamp: record.timestamp,
            reason: record.reason,
            from_location: record.from_location_id,
            to_location: record.to_location_id,
            sku: record.sku,
            quantity: record.quantity,
            status_before: record.status_before,
            status_after: record.status_after,
        }
    }
}

// ============================================================================
// Input Types for Mutations
// ============================================================================

/// Input for creating an item.
#[derive(InputObject)]
pub struct CreateItemInput {
    pub sku: String,
    pub name: String,
    pub name_zh: String,
    pub description: Option<String>,
    pub description_zh: Option<String>,
    pub item_type: String,
    pub tracking_mode: String,
    pub manufacturer_part_no: Option<String>,
}

/// Input for creating a location.
#[derive(InputObject)]
pub struct CreateLocationInput {
    pub warehouse: String,
    pub zone: String,
    pub bin: String,
}

// ============================================================================
// Warehouse Receiving Types
// ============================================================================

/// Receiving task status enum for GraphQL.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ReceivingTaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl From<warehouse::receiving::ReceivingTaskStatus> for ReceivingTaskStatus {
    fn from(status: warehouse::receiving::ReceivingTaskStatus) -> Self {
        match status {
            warehouse::receiving::ReceivingTaskStatus::Pending => Self::Pending,
            warehouse::receiving::ReceivingTaskStatus::InProgress => Self::InProgress,
            warehouse::receiving::ReceivingTaskStatus::Completed => Self::Completed,
            warehouse::receiving::ReceivingTaskStatus::Cancelled => Self::Cancelled,
        }
    }
}

/// Discrepancy type enum for GraphQL.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum DiscrepancyType {
    OverReceipt,
    UnderReceipt,
    ShortReceipt,
}

impl From<warehouse::receiving::DiscrepancyType> for DiscrepancyType {
    fn from(dt: warehouse::receiving::DiscrepancyType) -> Self {
        match dt {
            warehouse::receiving::DiscrepancyType::OverReceipt => Self::OverReceipt,
            warehouse::receiving::DiscrepancyType::UnderReceipt => Self::UnderReceipt,
            warehouse::receiving::DiscrepancyType::ShortReceipt => Self::ShortReceipt,
        }
    }
}

/// QC outcome enum for GraphQL.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum QCOutcome {
    Pass,
    Fail,
    Rework,
}

impl From<warehouse::receiving::QCOutcome> for QCOutcome {
    fn from(outcome: warehouse::receiving::QCOutcome) -> Self {
        match outcome {
            warehouse::receiving::QCOutcome::Pass => Self::Pass,
            warehouse::receiving::QCOutcome::Fail => Self::Fail,
            warehouse::receiving::QCOutcome::Rework => Self::Rework,
        }
    }
}

impl From<QCOutcome> for warehouse::receiving::QCOutcome {
    fn from(outcome: QCOutcome) -> Self {
        match outcome {
            QCOutcome::Pass => Self::Pass,
            QCOutcome::Fail => Self::Fail,
            QCOutcome::Rework => Self::Rework,
        }
    }
}

/// Quarantine status enum for GraphQL.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum QuarantineStatus {
    None,
    Quarantined,
    Released,
    Rejected,
}

impl From<warehouse::receiving::QuarantineStatus> for QuarantineStatus {
    fn from(status: warehouse::receiving::QuarantineStatus) -> Self {
        match status {
            warehouse::receiving::QuarantineStatus::None => Self::None,
            warehouse::receiving::QuarantineStatus::Quarantined => Self::Quarantined,
            warehouse::receiving::QuarantineStatus::Released => Self::Released,
            warehouse::receiving::QuarantineStatus::Rejected => Self::Rejected,
        }
    }
}

/// Receiving discrepancy type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct ReceivingDiscrepancy {
    pub id: ID,
    pub receiving_line_id: ID,
    pub discrepancy_type: DiscrepancyType,
    pub expected_quantity: i32,
    pub received_quantity: i32,
    pub discrepancy_amount: i32,
    pub notes: Option<String>,
    pub acknowledged: bool,
    pub created_at: String,
}

impl From<warehouse::receiving::ReceivingDiscrepancy> for ReceivingDiscrepancy {
    fn from(disc: warehouse::receiving::ReceivingDiscrepancy) -> Self {
        Self {
            id: ID(disc.id.to_string()),
            receiving_line_id: ID(disc.receiving_line_id.to_string()),
            discrepancy_type: disc.discrepancy_type.into(),
            expected_quantity: disc.expected_quantity as i32,
            received_quantity: disc.received_quantity as i32,
            discrepancy_amount: disc.discrepancy_amount as i32,
            notes: disc.notes,
            acknowledged: disc.acknowledged,
            created_at: disc.created_at.to_rfc3339(),
        }
    }
}

/// QC result type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct QCResult {
    pub id: ID,
    pub receiving_line_id: ID,
    pub outcome: QCOutcome,
    pub inspector: String,
    pub notes: Option<String>,
    pub created_at: String,
}

impl From<warehouse::receiving::QCResult> for QCResult {
    fn from(qc: warehouse::receiving::QCResult) -> Self {
        Self {
            id: ID(qc.id.to_string()),
            receiving_line_id: ID(qc.receiving_line_id.to_string()),
            outcome: qc.outcome.into(),
            inspector: qc.inspector,
            notes: qc.notes,
            created_at: qc.created_at.to_rfc3339(),
        }
    }
}

/// Receiving line type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct ReceivingLine {
    pub id: ID,
    pub po_line_id: ID,
    pub sku: String,
    pub expected_quantity: i32,
    pub received_quantity: i32,
    pub discrepancies: Vec<ReceivingDiscrepancy>,
    pub qc_result: Option<QCResult>,
    pub quarantine_status: QuarantineStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl From<warehouse::receiving::ReceivingLine> for ReceivingLine {
    fn from(line: warehouse::receiving::ReceivingLine) -> Self {
        Self {
            id: ID(line.id.to_string()),
            po_line_id: ID(line.po_line_id.to_string()),
            sku: line.sku,
            expected_quantity: line.expected_quantity as i32,
            received_quantity: line.received_quantity as i32,
            discrepancies: line.discrepancies.into_iter().map(Into::into).collect(),
            qc_result: line.qc_result.map(Into::into),
            quarantine_status: line.quarantine_status.into(),
            created_at: line.created_at.to_rfc3339(),
            updated_at: line.updated_at.to_rfc3339(),
        }
    }
}

/// Receiving task type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct ReceivingTask {
    pub id: ID,
    pub task_number: String,
    pub po_id: ID,
    pub po_number: String,
    pub status: ReceivingTaskStatus,
    pub lines: Vec<ReceivingLine>,
    pub assigned_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<warehouse::receiving::ReceivingTask> for ReceivingTask {
    fn from(task: warehouse::receiving::ReceivingTask) -> Self {
        Self {
            id: ID(task.id.to_string()),
            task_number: task.task_number,
            po_id: ID(task.po_id.to_string()),
            po_number: task.po_number,
            status: task.status.into(),
            lines: task.lines.into_iter().map(Into::into).collect(),
            assigned_to: task.assigned_to,
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
        }
    }
}

/// Input for creating a receiving task.
#[derive(InputObject)]
pub struct CreateReceivingTaskInput {
    pub po_id: ID,
    pub task_number: String,
}

/// Input for recording QC outcome.
#[derive(InputObject)]
pub struct RecordQCOutcomeInput {
    pub task_id: ID,
    pub line_id: ID,
    pub outcome: QCOutcome,
    pub inspector: String,
    pub notes: Option<String>,
}

/// Input for quarantining a line.
#[derive(InputObject)]
pub struct QuarantineLineInput {
    pub task_id: ID,
    pub line_id: ID,
}

/// Input for releasing from quarantine.
#[derive(InputObject)]
pub struct ReleaseFromQuarantineInput {
    pub task_id: ID,
    pub line_id: ID,
    pub location_id: String,
}

/// Input for recording receipt.
#[derive(InputObject)]
pub struct RecordReceiptInput {
    pub task_id: ID,
    pub line_id: ID,
    pub quantity: i32,
}

/// Input for acknowledging discrepancy.
#[derive(InputObject)]
pub struct AcknowledgeDiscrepancyInput {
    pub task_id: ID,
    pub line_id: ID,
    pub discrepancy_id: ID,
}

// ============================================================================
// Picking Domain Types
// ============================================================================

/// Pick task type enum.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum PickTaskType {
    #[graphql(name = "SINGLE_ORDER")]
    SingleOrder,
    #[graphql(name = "WAVE")]
    Wave,
    #[graphql(name = "BATCH")]
    Batch,
}

impl From<warehouse::picking::PickTaskType> for PickTaskType {
    fn from(t: warehouse::picking::PickTaskType) -> Self {
        match t {
            warehouse::picking::PickTaskType::SingleOrder => PickTaskType::SingleOrder,
            warehouse::picking::PickTaskType::Wave => PickTaskType::Wave,
            warehouse::picking::PickTaskType::Batch => PickTaskType::Batch,
        }
    }
}

impl From<PickTaskType> for warehouse::picking::PickTaskType {
    fn from(t: PickTaskType) -> Self {
        match t {
            PickTaskType::SingleOrder => warehouse::picking::PickTaskType::SingleOrder,
            PickTaskType::Wave => warehouse::picking::PickTaskType::Wave,
            PickTaskType::Batch => warehouse::picking::PickTaskType::Batch,
        }
    }
}

/// Pick task status enum.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum PickTaskStatus {
    #[graphql(name = "PENDING")]
    Pending,
    #[graphql(name = "IN_PROGRESS")]
    InProgress,
    #[graphql(name = "COMPLETED")]
    Completed,
    #[graphql(name = "CANCELLED")]
    Cancelled,
}

impl From<warehouse::picking::PickTaskStatus> for PickTaskStatus {
    fn from(s: warehouse::picking::PickTaskStatus) -> Self {
        match s {
            warehouse::picking::PickTaskStatus::Pending => PickTaskStatus::Pending,
            warehouse::picking::PickTaskStatus::InProgress => PickTaskStatus::InProgress,
            warehouse::picking::PickTaskStatus::Completed => PickTaskStatus::Completed,
            warehouse::picking::PickTaskStatus::Cancelled => PickTaskStatus::Cancelled,
        }
    }
}

/// Pick exception type enum.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum PickExceptionType {
    #[graphql(name = "SHORT_PICK")]
    ShortPick,
    #[graphql(name = "DAMAGED")]
    Damaged,
    #[graphql(name = "NOT_FOUND")]
    NotFound,
    #[graphql(name = "WRONG_ITEM")]
    WrongItem,
}

impl From<warehouse::picking::PickExceptionType> for PickExceptionType {
    fn from(t: warehouse::picking::PickExceptionType) -> Self {
        match t {
            warehouse::picking::PickExceptionType::ShortPick => PickExceptionType::ShortPick,
            warehouse::picking::PickExceptionType::Damaged => PickExceptionType::Damaged,
            warehouse::picking::PickExceptionType::NotFound => PickExceptionType::NotFound,
            warehouse::picking::PickExceptionType::WrongItem => PickExceptionType::WrongItem,
        }
    }
}

impl From<PickExceptionType> for warehouse::picking::PickExceptionType {
    fn from(t: PickExceptionType) -> Self {
        match t {
            PickExceptionType::ShortPick => warehouse::picking::PickExceptionType::ShortPick,
            PickExceptionType::Damaged => warehouse::picking::PickExceptionType::Damaged,
            PickExceptionType::NotFound => warehouse::picking::PickExceptionType::NotFound,
            PickExceptionType::WrongItem => warehouse::picking::PickExceptionType::WrongItem,
        }
    }
}

/// Pick exception type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct PickException {
    pub id: ID,
    pub pick_line_id: ID,
    pub exception_type: PickExceptionType,
    pub expected_quantity: i32,
    pub picked_quantity: i32,
    pub notes: Option<String>,
    pub created_at: String,
}

impl From<warehouse::picking::PickException> for PickException {
    fn from(e: warehouse::picking::PickException) -> Self {
        Self {
            id: ID(e.id.to_string()),
            pick_line_id: ID(e.pick_line_id.to_string()),
            exception_type: e.exception_type.into(),
            expected_quantity: e.expected_quantity as i32,
            picked_quantity: e.picked_quantity as i32,
            notes: e.notes,
            created_at: e.created_at.to_rfc3339(),
        }
    }
}

/// Pick line type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct PickLine {
    pub id: ID,
    pub order_line_id: ID,
    pub sku: String,
    pub location_id: String,
    pub quantity_to_pick: i32,
    pub quantity_picked: i32,
    pub exceptions: Vec<PickException>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<warehouse::picking::PickLine> for PickLine {
    fn from(line: warehouse::picking::PickLine) -> Self {
        Self {
            id: ID(line.id.to_string()),
            order_line_id: ID(line.order_line_id.to_string()),
            sku: line.sku,
            location_id: line.location_id,
            quantity_to_pick: line.quantity_to_pick as i32,
            quantity_picked: line.quantity_picked as i32,
            exceptions: line.exceptions.into_iter().map(Into::into).collect(),
            created_at: line.created_at.to_rfc3339(),
            updated_at: line.updated_at.to_rfc3339(),
        }
    }
}

/// Pick task type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct PickTask {
    pub id: ID,
    pub task_number: String,
    pub task_type: PickTaskType,
    pub order_ids: Vec<ID>,
    pub status: PickTaskStatus,
    pub lines: Vec<PickLine>,
    pub assigned_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<warehouse::picking::PickTask> for PickTask {
    fn from(task: warehouse::picking::PickTask) -> Self {
        Self {
            id: ID(task.id.to_string()),
            task_number: task.task_number,
            task_type: task.task_type.into(),
            order_ids: task.order_ids.into_iter().map(|id| ID(id.to_string())).collect(),
            status: task.status.into(),
            lines: task.lines.into_iter().map(Into::into).collect(),
            assigned_to: task.assigned_to,
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
        }
    }
}

// ============================================================================
// Packing Domain Types
// ============================================================================

/// Pack task status enum.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum PackTaskStatus {
    #[graphql(name = "PENDING")]
    Pending,
    #[graphql(name = "IN_PROGRESS")]
    InProgress,
    #[graphql(name = "COMPLETED")]
    Completed,
    #[graphql(name = "CANCELLED")]
    Cancelled,
}

impl From<warehouse::packing::PackTaskStatus> for PackTaskStatus {
    fn from(s: warehouse::packing::PackTaskStatus) -> Self {
        match s {
            warehouse::packing::PackTaskStatus::Pending => PackTaskStatus::Pending,
            warehouse::packing::PackTaskStatus::InProgress => PackTaskStatus::InProgress,
            warehouse::packing::PackTaskStatus::Completed => PackTaskStatus::Completed,
            warehouse::packing::PackTaskStatus::Cancelled => PackTaskStatus::Cancelled,
        }
    }
}

/// Packed item type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct PackedItem {
    pub id: ID,
    pub sku: String,
    pub quantity: i32,
    pub serial_numbers: Option<Vec<String>>,
}

impl From<warehouse::packing::PackedItem> for PackedItem {
    fn from(item: warehouse::packing::PackedItem) -> Self {
        Self {
            id: ID(item.id.to_string()),
            sku: item.sku,
            quantity: item.quantity as i32,
            serial_numbers: item.serial_numbers,
        }
    }
}

/// Package type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct Package {
    pub id: ID,
    pub package_number: String,
    pub carton_type: String,
    pub dimensions: Option<PackageDimensions>,
    pub weight: Option<f64>,
    pub tracking_number: Option<String>,
    pub items: Vec<PackedItem>,
    pub created_at: String,
}

/// Package dimensions type.
#[derive(Clone, SimpleObject)]
pub struct PackageDimensions {
    pub length: f64,
    pub width: f64,
    pub height: f64,
}

impl From<warehouse::packing::Package> for Package {
    fn from(pkg: warehouse::packing::Package) -> Self {
        Self {
            id: ID(pkg.id.to_string()),
            package_number: pkg.package_number,
            carton_type: pkg.carton_type,
            dimensions: pkg.dimensions.map(|(l, w, h)| PackageDimensions {
                length: l,
                width: w,
                height: h,
            }),
            weight: pkg.weight,
            tracking_number: pkg.tracking_number,
            items: pkg.items.into_iter().map(Into::into).collect(),
            created_at: pkg.created_at.to_rfc3339(),
        }
    }
}

/// Pack line type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct PackLine {
    pub id: ID,
    pub pick_line_id: ID,
    pub sku: String,
    pub quantity_to_pack: i32,
    pub quantity_packed: i32,
    pub serial_numbers: Option<Vec<String>>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<warehouse::packing::PackLine> for PackLine {
    fn from(line: warehouse::packing::PackLine) -> Self {
        Self {
            id: ID(line.id.to_string()),
            pick_line_id: ID(line.pick_line_id.to_string()),
            sku: line.sku,
            quantity_to_pack: line.quantity_to_pack as i32,
            quantity_packed: line.quantity_packed as i32,
            serial_numbers: line.serial_numbers,
            created_at: line.created_at.to_rfc3339(),
            updated_at: line.updated_at.to_rfc3339(),
        }
    }
}

/// Pack task type for GraphQL.
#[derive(Clone, SimpleObject)]
pub struct PackTask {
    pub id: ID,
    pub task_number: String,
    pub pick_task_id: ID,
    pub order_id: ID,
    pub status: PackTaskStatus,
    pub lines: Vec<PackLine>,
    pub packages: Vec<Package>,
    pub assigned_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<warehouse::packing::PackTask> for PackTask {
    fn from(task: warehouse::packing::PackTask) -> Self {
        Self {
            id: ID(task.id.to_string()),
            task_number: task.task_number,
            pick_task_id: ID(task.pick_task_id.to_string()),
            order_id: ID(task.order_id.to_string()),
            status: task.status.into(),
            lines: task.lines.into_iter().map(Into::into).collect(),
            packages: task.packages.into_iter().map(Into::into).collect(),
            assigned_to: task.assigned_to,
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
        }
    }
}

// ============================================================================
// Picking Input Types
// ============================================================================

/// Input for creating a pick task.
#[derive(InputObject)]
pub struct CreatePickTaskInput {
    pub task_number: String,
    pub task_type: PickTaskType,
    pub order_ids: Vec<ID>,
}

/// Input for adding a pick line.
#[derive(InputObject)]
pub struct AddPickLineInput {
    pub task_id: ID,
    pub order_line_id: ID,
    pub sku: String,
    pub location_id: String,
    pub quantity: i32,
}

/// Input for recording a pick.
#[derive(InputObject)]
pub struct RecordPickInput {
    pub task_id: ID,
    pub line_id: ID,
    pub quantity: i32,
}

/// Input for recording a pick exception.
#[derive(InputObject)]
pub struct RecordPickExceptionInput {
    pub task_id: ID,
    pub line_id: ID,
    pub exception_type: PickExceptionType,
    pub picked_quantity: i32,
    pub notes: Option<String>,
}

// ============================================================================
// Packing Input Types
// ============================================================================

/// Input for creating a pack task.
#[derive(InputObject)]
pub struct CreatePackTaskInput {
    pub task_number: String,
    pub pick_task_id: ID,
    pub order_id: ID,
}

/// Input for adding a pack line.
#[derive(InputObject)]
pub struct AddPackLineInput {
    pub task_id: ID,
    pub pick_line_id: ID,
    pub sku: String,
    pub quantity: i32,
}

/// Input for creating a package.
#[derive(InputObject)]
pub struct CreatePackageInput {
    pub task_id: ID,
    pub package_number: String,
    pub carton_type: String,
}

/// Input for recording packing.
#[derive(InputObject)]
pub struct RecordPackInput {
    pub task_id: ID,
    pub line_id: ID,
    pub package_id: ID,
    pub quantity: i32,
    pub serial_numbers: Option<Vec<String>>,
}

/// Input for setting package dimensions.
#[derive(InputObject)]
pub struct SetPackageDimensionsInput {
    pub task_id: ID,
    pub package_id: ID,
    pub length: f64,
    pub width: f64,
    pub height: f64,
}

/// Input for setting package weight.
#[derive(InputObject)]
pub struct SetPackageWeightInput {
    pub task_id: ID,
    pub package_id: ID,
    pub weight: f64,
}

/// Input for assigning tracking number.
#[derive(InputObject)]
pub struct AssignTrackingInput {
    pub task_id: ID,
    pub package_id: ID,
    pub tracking_number: String,
}

// ============================================================================
// Shipping Types
// ============================================================================

/// Ship task status.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ShipTaskStatus {
    Pending,
    Staging,
    ComplianceCheck,
    ReadyToDispatch,
    Dispatched,
    Blocked,
    Cancelled,
}

impl From<warehouse::ShipTaskStatus> for ShipTaskStatus {
    fn from(status: warehouse::ShipTaskStatus) -> Self {
        match status {
            warehouse::ShipTaskStatus::Pending => ShipTaskStatus::Pending,
            warehouse::ShipTaskStatus::Staging => ShipTaskStatus::Staging,
            warehouse::ShipTaskStatus::ComplianceCheck => ShipTaskStatus::ComplianceCheck,
            warehouse::ShipTaskStatus::ReadyToDispatch => ShipTaskStatus::ReadyToDispatch,
            warehouse::ShipTaskStatus::Dispatched => ShipTaskStatus::Dispatched,
            warehouse::ShipTaskStatus::Blocked => ShipTaskStatus::Blocked,
            warehouse::ShipTaskStatus::Cancelled => ShipTaskStatus::Cancelled,
        }
    }
}

/// Compliance block category.
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum ComplianceBlockCategory {
    MissingDocument,
    ExpiredDocument,
    MissingApproval,
    ExpiredApproval,
    ItemRestriction,
    QuarantineBlock,
    BatteryCompliance,
}

impl From<warehouse::ComplianceBlockCategory> for ComplianceBlockCategory {
    fn from(category: warehouse::ComplianceBlockCategory) -> Self {
        match category {
            warehouse::ComplianceBlockCategory::MissingDocument => ComplianceBlockCategory::MissingDocument,
            warehouse::ComplianceBlockCategory::ExpiredDocument => ComplianceBlockCategory::ExpiredDocument,
            warehouse::ComplianceBlockCategory::MissingApproval => ComplianceBlockCategory::MissingApproval,
            warehouse::ComplianceBlockCategory::ExpiredApproval => ComplianceBlockCategory::ExpiredApproval,
            warehouse::ComplianceBlockCategory::ItemRestriction => ComplianceBlockCategory::ItemRestriction,
            warehouse::ComplianceBlockCategory::QuarantineBlock => ComplianceBlockCategory::QuarantineBlock,
            warehouse::ComplianceBlockCategory::BatteryCompliance => ComplianceBlockCategory::BatteryCompliance,
        }
    }
}

/// Block reason detail.
#[derive(SimpleObject, Clone)]
pub struct BlockReason {
    pub category: ComplianceBlockCategory,
    pub message: String,
    pub details: Vec<KeyValuePair>,
}

/// Key-value pair for details.
#[derive(SimpleObject, Clone)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

impl From<warehouse::BlockReason> for BlockReason {
    fn from(reason: warehouse::BlockReason) -> Self {
        Self {
            category: reason.category.into(),
            message: reason.message,
            details: reason.details.into_iter()
                .map(|(k, v)| KeyValuePair { key: k, value: v })
                .collect(),
        }
    }
}

/// Compliance block result.
#[derive(SimpleObject, Clone)]
pub struct ComplianceBlock {
    pub operation: String,
    pub blocked: bool,
    pub reasons: Vec<BlockReason>,
    pub required_actions: Vec<String>,
}

impl From<warehouse::ComplianceBlock> for ComplianceBlock {
    fn from(block: warehouse::ComplianceBlock) -> Self {
        Self {
            operation: block.operation,
            blocked: block.blocked,
            reasons: block.reasons.into_iter().map(Into::into).collect(),
            required_actions: block.required_actions,
        }
    }
}

/// Ship line item.
#[derive(SimpleObject, Clone)]
pub struct ShipLine {
    pub id: ID,
    pub pack_task_id: ID,
    pub package_ids: Vec<ID>,
    pub order_id: ID,
    pub created_at: String,
}

impl From<warehouse::ShipLine> for ShipLine {
    fn from(line: warehouse::ShipLine) -> Self {
        Self {
            id: line.id.to_string().into(),
            pack_task_id: line.pack_task_id.to_string().into(),
            package_ids: line.package_ids.into_iter().map(|id| id.to_string().into()).collect(),
            order_id: line.order_id.to_string().into(),
            created_at: line.created_at.to_rfc3339(),
        }
    }
}

/// Ship task entity.
#[derive(SimpleObject, Clone)]
pub struct ShipTask {
    pub id: ID,
    pub task_number: String,
    pub status: ShipTaskStatus,
    pub lines: Vec<ShipLine>,
    pub compliance_check: Option<ComplianceBlock>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub dispatched_at: Option<String>,
    pub assigned_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<warehouse::ShipTask> for ShipTask {
    fn from(task: warehouse::ShipTask) -> Self {
        Self {
            id: task.id.to_string().into(),
            task_number: task.task_number,
            status: task.status.into(),
            lines: task.lines.into_iter().map(Into::into).collect(),
            compliance_check: task.compliance_check.map(Into::into),
            carrier: task.carrier,
            tracking_number: task.tracking_number,
            dispatched_at: task.dispatched_at.map(|dt| dt.to_rfc3339()),
            assigned_to: task.assigned_to,
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
        }
    }
}

// ============================================================================
// Shipping Input Types
// ============================================================================

/// Input for creating a ship task.
#[derive(InputObject)]
pub struct CreateShipTaskInput {
    pub task_number: String,
}

/// Input for adding a ship line.
#[derive(InputObject)]
pub struct AddShipLineInput {
    pub task_id: ID,
    pub pack_task_id: ID,
    pub package_ids: Vec<ID>,
    pub order_id: ID,
}

/// Input for dispatching a shipment.
#[derive(InputObject)]
pub struct DispatchShipmentInput {
    pub task_id: ID,
    pub carrier: String,
    pub tracking_number: String,
}

// ============================================================================
// Pagination Types (Relay-style cursor-based pagination)
// ============================================================================

/// Pagination input for cursor-based pagination.
#[derive(InputObject, Clone)]
pub struct PaginationInput {
    /// Number of items to return from the start.
    pub first: Option<i32>,
    /// Cursor to start from (for forward pagination).
    pub after: Option<String>,
    /// Number of items to return from the end.
    pub last: Option<i32>,
    /// Cursor to start from (for backward pagination).
    pub before: Option<String>,
}

/// Page information for cursor-based pagination.
#[derive(SimpleObject, Clone)]
pub struct PageInfo {
    /// Whether there are more items when paginating forward.
    pub has_next_page: bool,
    /// Whether there are more items when paginating backward.
    pub has_previous_page: bool,
    /// Cursor of the first item in the current page.
    pub start_cursor: Option<String>,
    /// Cursor of the last item in the current page.
    pub end_cursor: Option<String>,
}

/// Edge type for Item connections.
#[derive(SimpleObject, Clone)]
pub struct ItemEdge {
    /// The item node.
    pub node: Item,
    /// Cursor for this item.
    pub cursor: String,
}

/// Connection type for Item pagination.
#[derive(SimpleObject, Clone)]
pub struct ItemConnection {
    /// List of item edges.
    pub edges: Vec<ItemEdge>,
    /// Page information.
    pub page_info: PageInfo,
    /// Total count of items (optional, can be expensive to compute).
    pub total_count: Option<i64>,
}

// ============================================================================
// Structured Error Types
// ============================================================================

/// Error category for structured error handling.
#[derive(Enum, Copy, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ErrorCategory {
    /// Validation error (invalid input).
    Validation,
    /// Authorization error (permission denied).
    Authorization,
    /// Compliance block (operation blocked by compliance rules).
    ComplianceBlock,
    /// Resource not found.
    NotFound,
    /// Conflict (e.g., duplicate key, concurrent modification).
    Conflict,
    /// Internal server error.
    Internal,
}

/// Field-level error for validation.
#[derive(SimpleObject, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldError {
    /// Field name that has the error.
    pub field: String,
    /// Error message for this field.
    pub message: String,
}

/// Structured error with category and field-level details.
#[derive(SimpleObject, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructuredError {
    /// Error category.
    pub category: ErrorCategory,
    /// Main error message.
    pub message: String,
    /// Field-level errors (for validation errors).
    pub field_errors: Option<Vec<FieldError>>,
    /// Error code (optional, for client-side error handling).
    pub code: Option<String>,
}

impl StructuredError {
    /// Create a validation error.
    pub fn validation(message: impl Into<String>, field_errors: Vec<FieldError>) -> Self {
        Self {
            category: ErrorCategory::Validation,
            message: message.into(),
            field_errors: Some(field_errors),
            code: Some("VALIDATION_ERROR".to_string()),
        }
    }

    /// Create an authorization error.
    pub fn authorization(message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::Authorization,
            message: message.into(),
            field_errors: None,
            code: Some("AUTHORIZATION_ERROR".to_string()),
        }
    }

    /// Create a compliance block error.
    pub fn compliance_block(message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::ComplianceBlock,
            message: message.into(),
            field_errors: None,
            code: Some("COMPLIANCE_BLOCK".to_string()),
        }
    }

    /// Create a not found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::NotFound,
            message: message.into(),
            field_errors: None,
            code: Some("NOT_FOUND".to_string()),
        }
    }

    /// Create a conflict error.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::Conflict,
            message: message.into(),
            field_errors: None,
            code: Some("CONFLICT".to_string()),
        }
    }

    /// Create an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            category: ErrorCategory::Internal,
            message: message.into(),
            field_errors: None,
            code: Some("INTERNAL_ERROR".to_string()),
        }
    }
}

// ============================================================================
// Input Validation Helpers
// ============================================================================

/// Validate that a string is not empty.
pub fn validate_not_empty(field: &str, value: &str) -> Option<FieldError> {
    if value.trim().is_empty() {
        Some(FieldError {
            field: field.to_string(),
            message: format!("{} cannot be empty", field),
        })
    } else {
        None
    }
}

/// Validate that a number is positive.
pub fn validate_positive(field: &str, value: i64) -> Option<FieldError> {
    if value <= 0 {
        Some(FieldError {
            field: field.to_string(),
            message: format!("{} must be positive", field),
        })
    } else {
        None
    }
}

/// Validate that a number is non-negative.
pub fn validate_non_negative(field: &str, value: i64) -> Option<FieldError> {
    if value < 0 {
        Some(FieldError {
            field: field.to_string(),
            message: format!("{} cannot be negative", field),
        })
    } else {
        None
    }
}

// ============================================================================
// Bulk Mutation Types
// ============================================================================

/// Result of a bulk mutation operation.
#[derive(SimpleObject, Clone)]
pub struct BulkMutationResult<T: async_graphql::OutputType> {
    /// Number of successful operations.
    pub success_count: i32,
    /// Number of failed operations.
    pub failure_count: i32,
    /// Per-row results in the same order as input.
    pub results: Vec<RowResult<T>>,
}

/// Result for a single row in a bulk operation.
#[derive(SimpleObject, Clone)]
pub struct RowResult<T: async_graphql::OutputType> {
    /// Index of this row in the input array (0-based).
    pub index: i32,
    /// Whether this row succeeded.
    pub success: bool,
    /// The result data (if successful).
    pub data: Option<T>,
    /// The error (if failed).
    pub error: Option<StructuredError>,
}

/// Input for a single inventory adjustment in a bulk operation.
#[derive(InputObject, Clone)]
pub struct AdjustInventoryInput {
    /// SKU to adjust.
    pub sku: String,
    /// Location ID where the adjustment occurs.
    pub location_id: String,
    /// Quantity to adjust (positive = increase, negative = decrease).
    pub quantity: i64,
    /// Reason for the adjustment.
    pub reason: String,
    /// Optional reference (e.g., cycle count ID).
    pub reference: Option<String>,
}
