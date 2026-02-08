-- Sample inventory data for testing
-- This script creates sample items, locations, and inventory positions

-- Delete existing sample data (in reverse order of foreign key dependencies)
-- Note: inventory_movements cannot be deleted due to append-only trigger
-- To reload seed data, delete the database and restart the server
DELETE FROM lots;
DELETE FROM serial_units;
DELETE FROM inventory_positions;
DELETE FROM locations;
DELETE FROM items;

-- Sample Items (Drones, Batteries, Parts)
INSERT INTO items (id, sku, name_en, name_zh, description_en, description_zh, item_type, tracking_mode, manufacturer_part_no, created_at, updated_at)
VALUES 
    ('item-001', 'DJI-M300-RTK', 'DJI Matrice 300 RTK', 'DJI 经纬 M300 RTK', 'Professional drone for surveying and inspection', '专业测绘和巡检无人机', 'drone', 'serial', 'M300-RTK', datetime('now'), datetime('now')),
    ('item-002', 'DJI-M30T', 'DJI Matrice 30T', 'DJI 经纬 M30T', 'Compact thermal imaging drone', '紧凑型热成像无人机', 'drone', 'serial', 'M30T', datetime('now'), datetime('now')),
    ('item-003', 'TB60-BATTERY', 'TB60 Intelligent Flight Battery', 'TB60 智能飞行电池', 'High-capacity battery for M300 RTK', 'M300 RTK 高容量电池', 'battery', 'serial', 'TB60', datetime('now'), datetime('now')),
    ('item-004', 'TB30-BATTERY', 'TB30 Intelligent Flight Battery', 'TB30 智能飞行电池', 'Battery for M30 series', 'M30 系列电池', 'battery', 'serial', 'TB30', datetime('now'), datetime('now')),
    ('item-005', 'PROP-2110', 'Propeller 2110', '螺旋桨 2110', 'Replacement propeller for M300', 'M300 替换螺旋桨', 'part', 'lot', 'PROP-2110', datetime('now'), datetime('now')),
    ('item-006', 'LANDING-GEAR', 'Landing Gear Set', '起落架套装', 'Landing gear for M300 RTK', 'M300 RTK 起落架', 'part', 'none', 'LG-M300', datetime('now'), datetime('now')),
    ('item-007', 'GIMBAL-CAMERA', 'Zenmuse H20T Camera', '禅思 H20T 相机', 'Hybrid thermal and zoom camera', '混合热成像和变焦相机', 'part', 'serial', 'H20T', datetime('now'), datetime('now')),
    ('item-008', 'CHARGING-HUB', 'BS60 Charging Station', 'BS60 充电站', 'Intelligent charging hub', '智能充电站', 'tool', 'none', 'BS60', datetime('now'), datetime('now')),
    ('item-009', 'CASE-M300', 'Hard Case for M300', 'M300 硬壳箱', 'Protective transport case', '防护运输箱', 'packaging', 'none', 'CASE-M300', datetime('now'), datetime('now')),
    ('item-010', 'CLEANING-KIT', 'Drone Cleaning Kit', '无人机清洁套装', 'Maintenance cleaning supplies', '维护清洁用品', 'consumable', 'none', 'CLEAN-01', datetime('now'), datetime('now'));

-- Sample Locations (Warehouses and Zones)
INSERT INTO locations (id, warehouse, zone, bin, created_at)
VALUES
    ('loc-001', 'WH-SZ', 'RECEIVING', 'R-001', datetime('now')),
    ('loc-002', 'WH-SZ', 'RECEIVING', 'R-002', datetime('now')),
    ('loc-003', 'WH-SZ', 'STORAGE', 'A-101', datetime('now')),
    ('loc-004', 'WH-SZ', 'STORAGE', 'A-102', datetime('now')),
    ('loc-005', 'WH-SZ', 'STORAGE', 'B-201', datetime('now')),
    ('loc-006', 'WH-SZ', 'STORAGE', 'B-202', datetime('now')),
    ('loc-007', 'WH-SZ', 'QUARANTINE', 'Q-001', datetime('now')),
    ('loc-008', 'WH-SZ', 'SHIPPING', 'S-001', datetime('now')),
    ('loc-009', 'WH-BJ', 'STORAGE', 'A-001', datetime('now')),
    ('loc-010', 'WH-BJ', 'STORAGE', 'A-002', datetime('now'));

-- Sample Inventory Movements (Receipts to create initial stock)
INSERT INTO inventory_movements (id, timestamp, actor, reason, from_location_id, to_location_id, sku, quantity, status_before, status_after, reference)
VALUES
    ('mov-001', datetime('now', '-30 days'), 'system', 'receipt', NULL, 'loc-003', 'DJI-M300-RTK', 5, 'available', 'available', 'PO-2024-001'),
    ('mov-002', datetime('now', '-30 days'), 'system', 'receipt', NULL, 'loc-003', 'DJI-M30T', 8, 'available', 'available', 'PO-2024-001'),
    ('mov-003', datetime('now', '-25 days'), 'system', 'receipt', NULL, 'loc-004', 'TB60-BATTERY', 20, 'available', 'available', 'PO-2024-002'),
    ('mov-004', datetime('now', '-25 days'), 'system', 'receipt', NULL, 'loc-004', 'TB30-BATTERY', 30, 'available', 'available', 'PO-2024-002'),
    ('mov-005', datetime('now', '-20 days'), 'system', 'receipt', NULL, 'loc-005', 'PROP-2110', 100, 'available', 'available', 'PO-2024-003'),
    ('mov-006', datetime('now', '-20 days'), 'system', 'receipt', NULL, 'loc-005', 'LANDING-GEAR', 15, 'available', 'available', 'PO-2024-003'),
    ('mov-007', datetime('now', '-15 days'), 'system', 'receipt', NULL, 'loc-006', 'GIMBAL-CAMERA', 10, 'available', 'available', 'PO-2024-004'),
    ('mov-008', datetime('now', '-15 days'), 'system', 'receipt', NULL, 'loc-006', 'CHARGING-HUB', 12, 'available', 'available', 'PO-2024-004'),
    ('mov-009', datetime('now', '-10 days'), 'system', 'receipt', NULL, 'loc-005', 'CASE-M300', 20, 'available', 'available', 'PO-2024-005'),
    ('mov-010', datetime('now', '-10 days'), 'system', 'receipt', NULL, 'loc-005', 'CLEANING-KIT', 50, 'available', 'available', 'PO-2024-005');

-- Sample Inventory Positions (Current stock levels with various statuses)
INSERT INTO inventory_positions (id, sku, location_id, status, lot_id, quantity_on_hand, quantity_reserved, quantity_available, last_updated)
VALUES
    ('pos-001', 'DJI-M300-RTK', 'loc-003', 'available', NULL, 5, 0, 5, datetime('now', '-1 day')),
    ('pos-002', 'DJI-M30T', 'loc-003', 'available', NULL, 8, 0, 8, datetime('now', '-1 day')),
    ('pos-003', 'TB60-BATTERY', 'loc-004', 'available', NULL, 20, 2, 18, datetime('now', '-1 day')),
    ('pos-004', 'TB30-BATTERY', 'loc-004', 'reserved', NULL, 30, 30, 0, datetime('now', '-1 day')),
    ('pos-005', 'PROP-2110', 'loc-005', 'available', NULL, 100, 10, 90, datetime('now', '-1 day')),
    ('pos-006', 'LANDING-GEAR', 'loc-005', 'quarantine', NULL, 15, 0, 0, datetime('now', '-1 day')),
    ('pos-007', 'GIMBAL-CAMERA', 'loc-006', 'available', NULL, 10, 1, 9, datetime('now', '-1 day')),
    ('pos-008', 'CHARGING-HUB', 'loc-006', 'damaged', NULL, 12, 0, 0, datetime('now', '-1 day')),
    ('pos-009', 'CASE-M300', 'loc-005', 'available', NULL, 20, 3, 17, datetime('now', '-1 day')),
    ('pos-010', 'CLEANING-KIT', 'loc-005', 'available', NULL, 50, 0, 50, datetime('now', '-1 day')),
    ('pos-011', 'DJI-M300-RTK', 'loc-009', 'reserved', NULL, 3, 3, 0, datetime('now', '-2 days')),
    ('pos-012', 'TB60-BATTERY', 'loc-007', 'quarantine', NULL, 5, 0, 0, datetime('now', '-3 days'));

-- Sample Serial Units (for serialized items)
INSERT INTO serial_units (id, serial_number, sku, current_status, current_location_id, created_at, updated_at)
VALUES
    ('serial-001', 'M300-2024-001', 'DJI-M300-RTK', 'available', 'loc-003', datetime('now'), datetime('now')),
    ('serial-002', 'M300-2024-002', 'DJI-M300-RTK', 'available', 'loc-003', datetime('now'), datetime('now')),
    ('serial-003', 'M300-2024-003', 'DJI-M300-RTK', 'available', 'loc-003', datetime('now'), datetime('now')),
    ('serial-004', 'M30T-2024-001', 'DJI-M30T', 'available', 'loc-003', datetime('now'), datetime('now')),
    ('serial-005', 'M30T-2024-002', 'DJI-M30T', 'available', 'loc-003', datetime('now'), datetime('now')),
    ('serial-006', 'TB60-2024-001', 'TB60-BATTERY', 'available', 'loc-004', datetime('now'), datetime('now')),
    ('serial-007', 'TB60-2024-002', 'TB60-BATTERY', 'available', 'loc-004', datetime('now'), datetime('now')),
    ('serial-008', 'TB30-2024-001', 'TB30-BATTERY', 'available', 'loc-004', datetime('now'), datetime('now')),
    ('serial-009', 'H20T-2024-001', 'GIMBAL-CAMERA', 'available', 'loc-006', datetime('now'), datetime('now')),
    ('serial-010', 'H20T-2024-002', 'GIMBAL-CAMERA', 'available', 'loc-006', datetime('now'), datetime('now'));

-- Sample Lots (for lot-tracked items)
INSERT INTO lots (id, lot_number, sku, expiry_date, created_at)
VALUES
    ('lot-001', 'PROP-LOT-2024-01', 'PROP-2110', NULL, datetime('now')),
    ('lot-002', 'PROP-LOT-2024-02', 'PROP-2110', NULL, datetime('now'));

