-- Inventory Schema: Items, Locations, Movements, and Positions
-- ADV-PERSIST-001

-- Items/SKUs table with bilingual support
CREATE TABLE IF NOT EXISTS items (
    id TEXT PRIMARY KEY NOT NULL,
    sku TEXT UNIQUE NOT NULL,
    name_en TEXT NOT NULL,
    name_zh TEXT NOT NULL,
    description_en TEXT,
    description_zh TEXT,
    item_type TEXT NOT NULL CHECK (item_type IN ('drone', 'battery', 'part', 'tool', 'consumable', 'packaging', 'service')),
    tracking_mode TEXT NOT NULL CHECK (tracking_mode IN ('none', 'serial', 'lot')),
    manufacturer_part_no TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_items_sku ON items(sku);
CREATE INDEX IF NOT EXISTS idx_items_item_type ON items(item_type);
CREATE INDEX IF NOT EXISTS idx_items_tracking_mode ON items(tracking_mode);

-- Locations table (warehouse/zone/bin hierarchy)
CREATE TABLE IF NOT EXISTS locations (
    id TEXT PRIMARY KEY NOT NULL,
    warehouse TEXT NOT NULL,
    zone TEXT NOT NULL,
    bin TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE (warehouse, zone, bin)
);

CREATE INDEX IF NOT EXISTS idx_locations_warehouse ON locations(warehouse);
CREATE INDEX IF NOT EXISTS idx_locations_warehouse_zone ON locations(warehouse, zone);

-- Serial units table for serial-tracked items
CREATE TABLE IF NOT EXISTS serial_units (
    id TEXT PRIMARY KEY NOT NULL,
    serial_number TEXT UNIQUE NOT NULL,
    sku TEXT NOT NULL,
    current_location_id TEXT NOT NULL,
    current_status TEXT NOT NULL CHECK (current_status IN ('available', 'reserved', 'quarantine', 'damaged', 'in_transit', 'on_hold', 'scrapped')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (sku) REFERENCES items(sku) ON DELETE RESTRICT,
    FOREIGN KEY (current_location_id) REFERENCES locations(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_serial_units_serial_number ON serial_units(serial_number);
CREATE INDEX IF NOT EXISTS idx_serial_units_sku ON serial_units(sku);
CREATE INDEX IF NOT EXISTS idx_serial_units_location ON serial_units(current_location_id);
CREATE INDEX IF NOT EXISTS idx_serial_units_status ON serial_units(current_status);

-- Lots table for lot-tracked items
CREATE TABLE IF NOT EXISTS lots (
    id TEXT PRIMARY KEY NOT NULL,
    lot_number TEXT UNIQUE NOT NULL,
    sku TEXT NOT NULL,
    expiry_date TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (sku) REFERENCES items(sku) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_lots_lot_number ON lots(lot_number);
CREATE INDEX IF NOT EXISTS idx_lots_sku ON lots(sku);
CREATE INDEX IF NOT EXISTS idx_lots_expiry ON lots(expiry_date);

-- Inventory movements table (append-only ledger)
CREATE TABLE IF NOT EXISTS inventory_movements (
    id TEXT PRIMARY KEY NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    actor TEXT NOT NULL,
    reason TEXT NOT NULL CHECK (reason IN (
        'receipt', 'transfer', 'adjustment', 'cycle_count', 'pick', 'putaway',
        'shipment', 'return', 'scrap', 'quarantine_hold', 'quarantine_release',
        'damage', 'reservation', 'reservation_release'
    )),
    from_location_id TEXT,
    to_location_id TEXT,
    sku TEXT NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity != 0),
    serial_numbers TEXT, -- JSON array of serial numbers
    lot_id TEXT,
    status_before TEXT NOT NULL CHECK (status_before IN ('available', 'reserved', 'quarantine', 'damaged', 'in_transit', 'on_hold', 'scrapped')),
    status_after TEXT NOT NULL CHECK (status_after IN ('available', 'reserved', 'quarantine', 'damaged', 'in_transit', 'on_hold', 'scrapped')),
    reference TEXT,
    notes TEXT,
    FOREIGN KEY (from_location_id) REFERENCES locations(id) ON DELETE RESTRICT,
    FOREIGN KEY (to_location_id) REFERENCES locations(id) ON DELETE RESTRICT,
    FOREIGN KEY (sku) REFERENCES items(sku) ON DELETE RESTRICT,
    FOREIGN KEY (lot_id) REFERENCES lots(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_movements_timestamp ON inventory_movements(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_movements_sku ON inventory_movements(sku);
CREATE INDEX IF NOT EXISTS idx_movements_from_location ON inventory_movements(from_location_id);
CREATE INDEX IF NOT EXISTS idx_movements_to_location ON inventory_movements(to_location_id);
CREATE INDEX IF NOT EXISTS idx_movements_reason ON inventory_movements(reason);
CREATE INDEX IF NOT EXISTS idx_movements_lot ON inventory_movements(lot_id);

-- Inventory positions table (derived from movements, but cached for performance)
CREATE TABLE IF NOT EXISTS inventory_positions (
    id TEXT PRIMARY KEY NOT NULL,
    sku TEXT NOT NULL,
    location_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('available', 'reserved', 'quarantine', 'damaged', 'in_transit', 'on_hold', 'scrapped')),
    lot_id TEXT,
    quantity_on_hand INTEGER NOT NULL DEFAULT 0 CHECK (quantity_on_hand >= 0),
    quantity_reserved INTEGER NOT NULL DEFAULT 0 CHECK (quantity_reserved >= 0),
    quantity_available INTEGER NOT NULL DEFAULT 0 CHECK (quantity_available >= 0),
    last_updated TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (sku) REFERENCES items(sku) ON DELETE RESTRICT,
    FOREIGN KEY (location_id) REFERENCES locations(id) ON DELETE RESTRICT,
    FOREIGN KEY (lot_id) REFERENCES lots(id) ON DELETE RESTRICT,
    UNIQUE (sku, location_id, status, lot_id)
);

CREATE INDEX IF NOT EXISTS idx_positions_sku ON inventory_positions(sku);
CREATE INDEX IF NOT EXISTS idx_positions_location ON inventory_positions(location_id);
CREATE INDEX IF NOT EXISTS idx_positions_status ON inventory_positions(status);
CREATE INDEX IF NOT EXISTS idx_positions_lot ON inventory_positions(lot_id);
CREATE INDEX IF NOT EXISTS idx_positions_available ON inventory_positions(quantity_available) WHERE quantity_available > 0;

-- Trigger to prevent updates/deletes on inventory_movements (append-only enforcement)
CREATE TRIGGER IF NOT EXISTS prevent_movement_updates
BEFORE UPDATE ON inventory_movements
BEGIN
    SELECT RAISE(ABORT, 'Inventory movements are append-only and cannot be updated');
END;

CREATE TRIGGER IF NOT EXISTS prevent_movement_deletes
BEFORE DELETE ON inventory_movements
BEGIN
    SELECT RAISE(ABORT, 'Inventory movements are append-only and cannot be deleted');
END;
