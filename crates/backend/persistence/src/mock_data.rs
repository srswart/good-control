//! Mock data seeding helpers for demos.

use crate::{DbPool, Result};

#[derive(Debug, Clone)]
pub struct MockDataSummary {
    pub items: i64,
    pub locations: i64,
    pub lots: i64,
    pub serial_units: i64,
    pub inventory_positions: i64,
    pub inventory_movements: i64,
}

pub async fn reload_mock_data(pool: &DbPool) -> Result<MockDataSummary> {
    let mut tx = pool.inner().begin().await?;

    clear_inventory_data(&mut tx).await?;
    seed_inventory_data(&mut tx).await?;

    let summary = MockDataSummary {
        items: count_rows(&mut tx, "items").await?,
        locations: count_rows(&mut tx, "locations").await?,
        lots: count_rows(&mut tx, "lots").await?,
        serial_units: count_rows(&mut tx, "serial_units").await?,
        inventory_positions: count_rows(&mut tx, "inventory_positions").await?,
        inventory_movements: count_rows(&mut tx, "inventory_movements").await?,
    };

    tx.commit().await?;

    Ok(summary)
}

async fn clear_inventory_data(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<()> {
    drop_inventory_triggers(tx).await?;
    sqlx::query("DELETE FROM inventory_movements").execute(tx.as_mut()).await?;
    sqlx::query("DELETE FROM inventory_positions").execute(tx.as_mut()).await?;
    sqlx::query("DELETE FROM serial_units").execute(tx.as_mut()).await?;
    sqlx::query("DELETE FROM lots").execute(tx.as_mut()).await?;
    sqlx::query("DELETE FROM items").execute(tx.as_mut()).await?;
    sqlx::query("DELETE FROM locations").execute(tx.as_mut()).await?;
    create_inventory_triggers(tx).await?;
    Ok(())
}

async fn seed_inventory_data(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<()> {
    let mut location_ids = Vec::new();
    for index in 1..=20 {
        let id = format!("LOC-{:03}", index);
        let warehouse = "WH1";
        let zone = format!("Z{}", (index - 1) % 4 + 1);
        let bin = format!("BIN-{:02}", index);
        location_ids.push(id.clone());
        sqlx::query(
            "INSERT INTO locations (id, warehouse, zone, bin) VALUES (?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(warehouse)
        .bind(&zone)
        .bind(&bin)
        .execute(tx.as_mut())
        .await?;
    }

    let item_types = ["drone", "battery", "part", "tool", "consumable"];
    let tracking_modes = ["serial", "lot", "none", "serial", "lot"];
    let mut skus = Vec::new();
    let mut serial_skus = Vec::new();
    let mut lot_skus = Vec::new();

    for index in 1..=20 {
        let id = format!("ITEM-{:03}", index);
        let sku = format!("SKU-{:03}", index);
        let item_type = item_types[(index - 1) % item_types.len()];
        let tracking_mode = tracking_modes[(index - 1) % tracking_modes.len()];
        let name_en = format!("Mock Item {}", index);
        let name_zh = format!("示例物品{}", index);
        skus.push(sku.clone());
        if tracking_mode == "serial" {
            serial_skus.push(sku.clone());
        } else if tracking_mode == "lot" {
            lot_skus.push(sku.clone());
        }

        sqlx::query(
            "INSERT INTO items (id, sku, name_en, name_zh, item_type, tracking_mode) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&sku)
        .bind(&name_en)
        .bind(&name_zh)
        .bind(item_type)
        .bind(tracking_mode)
        .execute(tx.as_mut())
        .await?;
    }

    let mut lot_ids = Vec::new();
    for index in 1..=20 {
        let lot_id = format!("LOT-{:04}", index);
        let lot_number = format!("LOT-2026-{:02}", index);
        let sku = lot_skus
            .get((index - 1) % lot_skus.len())
            .cloned()
            .unwrap_or_else(|| skus[0].clone());
        let expiry_date = format!("2026-12-{:02}", (index - 1) % 28 + 1);
        lot_ids.push((lot_id.clone(), sku.clone()));

        sqlx::query(
            "INSERT INTO lots (id, lot_number, sku, expiry_date) VALUES (?, ?, ?, ?)"
        )
        .bind(&lot_id)
        .bind(&lot_number)
        .bind(&sku)
        .bind(&expiry_date)
        .execute(tx.as_mut())
        .await?;
    }

    for index in 1..=20 {
        let id = format!("SERIAL-{:04}", index);
        let serial_number = format!("SN-{:04}", index);
        let sku = serial_skus
            .get((index - 1) % serial_skus.len())
            .cloned()
            .unwrap_or_else(|| skus[0].clone());
        let location_id = &location_ids[(index - 1) % location_ids.len()];
        let status = "available";

        sqlx::query(
            "INSERT INTO serial_units (id, serial_number, sku, current_location_id, current_status) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&serial_number)
        .bind(&sku)
        .bind(location_id)
        .bind(status)
        .execute(tx.as_mut())
        .await?;
    }

    for index in 1..=20 {
        let id = format!("POS-{:04}", index);
        let sku = skus[(index - 1) % skus.len()].clone();
        let location_id = &location_ids[(index - 1) % location_ids.len()];
        let status = "available";
        let lot_id = lot_ids
            .iter()
            .find(|(_, lot_sku)| *lot_sku == sku)
            .map(|(lot_id, _)| lot_id.as_str());
        let on_hand = 10 + (index as i64);
        let reserved = (index % 3) as i64;
        let available = on_hand - reserved;

        sqlx::query(
            "INSERT INTO inventory_positions (id, sku, location_id, status, lot_id, quantity_on_hand, quantity_reserved, quantity_available) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&sku)
        .bind(location_id)
        .bind(status)
        .bind(lot_id)
        .bind(on_hand)
        .bind(reserved)
        .bind(available)
        .execute(tx.as_mut())
        .await?;
    }

    for index in 1..=20 {
        let id = format!("MOVE-{:04}", index);
        let actor = "mock-seed";
        let reason = "receipt";
        let from_location_id: Option<&str> = None;
        let to_location_id = Some(&location_ids[(index - 1) % location_ids.len()]);
        let sku = skus[(index - 1) % skus.len()].clone();
        let quantity = 5 + index as i64;
        let lot_id = lot_ids
            .iter()
            .find(|(_, lot_sku)| *lot_sku == sku)
            .map(|(lot_id, _)| lot_id.as_str());
        let status_before = "in_transit";
        let status_after = "available";
        let reference = Some(format!("RCV-{:03}", index));

        sqlx::query(
            "INSERT INTO inventory_movements (id, actor, reason, from_location_id, to_location_id, sku, quantity, lot_id, status_before, status_after, reference) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(actor)
        .bind(reason)
        .bind(from_location_id)
        .bind(to_location_id)
        .bind(&sku)
        .bind(quantity)
        .bind(lot_id)
        .bind(status_before)
        .bind(status_after)
        .bind(reference)
        .execute(tx.as_mut())
        .await?;
    }

    Ok(())
}

async fn count_rows(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, table: &str) -> Result<i64> {
    let query = format!("SELECT COUNT(*) FROM {}", table);
    let count = sqlx::query_scalar::<_, i64>(&query)
        .fetch_one(tx.as_mut())
        .await?;
    Ok(count)
}

async fn drop_inventory_triggers(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<()> {
    sqlx::query("DROP TRIGGER IF EXISTS prevent_movement_updates")
        .execute(tx.as_mut())
        .await?;
    sqlx::query("DROP TRIGGER IF EXISTS prevent_movement_deletes")
        .execute(tx.as_mut())
        .await?;
    Ok(())
}

async fn create_inventory_triggers(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<()> {
    sqlx::query(
        "CREATE TRIGGER IF NOT EXISTS prevent_movement_updates
         BEFORE UPDATE ON inventory_movements
         BEGIN
             SELECT RAISE(ABORT, 'Inventory movements are append-only and cannot be updated');
         END;"
    )
    .execute(tx.as_mut())
    .await?;
    sqlx::query(
        "CREATE TRIGGER IF NOT EXISTS prevent_movement_deletes
         BEFORE DELETE ON inventory_movements
         BEGIN
             SELECT RAISE(ABORT, 'Inventory movements are append-only and cannot be deleted');
         END;"
    )
    .execute(tx.as_mut())
    .await?;
    Ok(())
}
