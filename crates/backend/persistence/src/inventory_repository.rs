//! @arrive component: persistence
//!
//! Inventory repository records and trait definitions.

use crate::error::Result;
use async_trait::async_trait;
use sqlx::SqlitePool;

/// Item record (items table)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ItemRecord {
    pub id: String,
    pub sku: String,
    pub name_en: String,
    pub name_zh: String,
    pub description_en: Option<String>,
    pub description_zh: Option<String>,
    pub item_type: String,
    pub tracking_mode: String,
    pub manufacturer_part_no: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Location record (locations table)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LocationRecord {
    pub id: String,
    pub warehouse: String,
    pub zone: String,
    pub bin: String,
    pub created_at: String,
}

/// Inventory position record (inventory_positions table)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InventoryPositionRecord {
    pub id: String,
    pub sku: String,
    pub location_id: String,
    pub status: String,
    pub lot_id: Option<String>,
    pub quantity_on_hand: i64,
    pub quantity_reserved: i64,
    pub quantity_available: i64,
    pub last_updated: String,
}

/// Inventory movement record (inventory_movements table)
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InventoryMovementRecord {
    pub id: String,
    pub timestamp: String,
    pub actor: String,
    pub reason: String,
    pub from_location_id: Option<String>,
    pub to_location_id: Option<String>,
    pub sku: String,
    pub quantity: i64,
    pub serial_numbers: Option<String>,
    pub lot_id: Option<String>,
    pub status_before: String,
    pub status_after: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
}

/// Inventory repository trait (placeholder for future specialization)
#[async_trait]
pub trait InventoryRepository: Send + Sync {}

/// SQLite implementation placeholder
pub struct SqliteInventoryRepository {
    _pool: SqlitePool,
}

impl SqliteInventoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { _pool: pool }
    }
}

