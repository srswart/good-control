-- Rollback Inventory Schema
-- ADV-PERSIST-001

-- Drop triggers first
DROP TRIGGER IF EXISTS prevent_movement_deletes;
DROP TRIGGER IF EXISTS prevent_movement_updates;

-- Drop tables in reverse order (respecting foreign keys)
DROP TABLE IF EXISTS inventory_positions;
DROP TABLE IF EXISTS inventory_movements;
DROP TABLE IF EXISTS lots;
DROP TABLE IF EXISTS serial_units;
DROP TABLE IF EXISTS locations;
DROP TABLE IF EXISTS items;
