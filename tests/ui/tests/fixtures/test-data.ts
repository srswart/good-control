/**
 * Test data fixtures for UI tests
 */

export interface TestItem {
  sku: string;
  name_en: string;
  name_zh: string;
  item_type: string;
  tracking_mode: string;
}

export interface TestLocation {
  code: string;
  name_en: string;
  name_zh: string;
  location_type: string;
}

export interface TestInventoryPosition {
  sku: string;
  location_id: string;
  status: 'available' | 'reserved' | 'quarantine' | 'damaged';
  quantity_on_hand: number;
  quantity_reserved: number;
  quantity_available: number;
}

/**
 * Sample inventory items for testing
 */
export const TEST_ITEMS: TestItem[] = [
  {
    sku: 'DJI-M300-001',
    name_en: 'DJI Matrice 300 RTK',
    name_zh: 'DJI 经纬 M300 RTK',
    item_type: 'drone',
    tracking_mode: 'serial'
  },
  {
    sku: 'DJI-TB60-001',
    name_en: 'TB60 Intelligent Flight Battery',
    name_zh: 'TB60 智能飞行电池',
    item_type: 'battery',
    tracking_mode: 'serial'
  },
  {
    sku: 'PROP-9455-001',
    name_en: '9455S Quick-Release Propellers',
    name_zh: '9455S 快拆桨',
    item_type: 'part',
    tracking_mode: 'lot'
  },
  {
    sku: 'DJI-M30T-001',
    name_en: 'DJI Matrice 30T',
    name_zh: 'DJI 经纬 M30T',
    item_type: 'drone',
    tracking_mode: 'serial'
  },
  {
    sku: 'CAM-H20T-001',
    name_en: 'Zenmuse H20T Camera',
    name_zh: '禅思 H20T 云台相机',
    item_type: 'camera',
    tracking_mode: 'serial'
  }
];

/**
 * Sample locations for testing
 */
export const TEST_LOCATIONS: TestLocation[] = [
  {
    code: 'WH-A-01-01',
    name_en: 'Warehouse A - Aisle 1 - Bin 1',
    name_zh: '仓库 A - 通道 1 - 货位 1',
    location_type: 'bin'
  },
  {
    code: 'WH-A-01-02',
    name_en: 'Warehouse A - Aisle 1 - Bin 2',
    name_zh: '仓库 A - 通道 1 - 货位 2',
    location_type: 'bin'
  },
  {
    code: 'STAGING-01',
    name_en: 'Staging Area 1',
    name_zh: '暂存区 1',
    location_type: 'staging'
  }
];

/**
 * Search queries for testing
 */
export const SEARCH_QUERIES = {
  validSku: 'DJI-M300',
  validNameEn: 'Battery',
  validNameZh: '电池',
  validPartial: 'DJI',
  noResults: 'NONEXISTENT-SKU-999',
  specialChars: 'M30T'
};

/**
 * User credentials for testing (if authentication is implemented)
 */
export const TEST_USERS = {
  admin: {
    username: 'admin',
    password: 'admin123'
  },
  operator: {
    username: 'operator',
    password: 'operator123'
  },
  viewer: {
    username: 'viewer',
    password: 'viewer123'
  }
};

/**
 * Common test timeouts
 */
export const TIMEOUTS = {
  short: 1000,
  medium: 3000,
  long: 5000,
  veryLong: 10000
};

/**
 * Pagination test data
 */
export const PAGINATION = {
  defaultPageSize: 20,
  smallPageSize: 5,
  largePageSize: 50
};

/**
 * Get a random test item
 */
export function getRandomItem(): TestItem {
  return TEST_ITEMS[Math.floor(Math.random() * TEST_ITEMS.length)];
}

/**
 * Get a random test location
 */
export function getRandomLocation(): TestLocation {
  return TEST_LOCATIONS[Math.floor(Math.random() * TEST_LOCATIONS.length)];
}

/**
 * Get test items by type
 */
export function getItemsByType(itemType: string): TestItem[] {
  return TEST_ITEMS.filter(item => item.item_type === itemType);
}

/**
 * Get test items by tracking mode
 */
export function getItemsByTrackingMode(trackingMode: string): TestItem[] {
  return TEST_ITEMS.filter(item => item.tracking_mode === trackingMode);
}

/**
 * Sample inventory positions for testing (based on seed data)
 */
export const TEST_INVENTORY_POSITIONS: Record<string, TestInventoryPosition> = {
  available_m300: {
    sku: 'DJI-M300-RTK',
    location_id: 'loc-003',
    status: 'available',
    quantity_on_hand: 5,
    quantity_reserved: 0,
    quantity_available: 5
  },
  available_m30t: {
    sku: 'DJI-M30T',
    location_id: 'loc-003',
    status: 'available',
    quantity_on_hand: 8,
    quantity_reserved: 0,
    quantity_available: 8
  },
  available_tb60: {
    sku: 'TB60-BATTERY',
    location_id: 'loc-004',
    status: 'available',
    quantity_on_hand: 20,
    quantity_reserved: 2,
    quantity_available: 18
  },
  reserved_tb30: {
    sku: 'TB30-BATTERY',
    location_id: 'loc-004',
    status: 'reserved',
    quantity_on_hand: 30,
    quantity_reserved: 30,
    quantity_available: 0
  },
  quarantine_landing_gear: {
    sku: 'LANDING-GEAR',
    location_id: 'loc-005',
    status: 'quarantine',
    quantity_on_hand: 15,
    quantity_reserved: 0,
    quantity_available: 0
  },
  damaged_charging_hub: {
    sku: 'CHARGING-HUB',
    location_id: 'loc-006',
    status: 'damaged',
    quantity_on_hand: 12,
    quantity_reserved: 0,
    quantity_available: 0
  },
  reserved_m300_bj: {
    sku: 'DJI-M300-RTK',
    location_id: 'loc-009',
    status: 'reserved',
    quantity_on_hand: 3,
    quantity_reserved: 3,
    quantity_available: 0
  },
  quarantine_tb60_q: {
    sku: 'TB60-BATTERY',
    location_id: 'loc-007',
    status: 'quarantine',
    quantity_on_hand: 5,
    quantity_reserved: 0,
    quantity_available: 0
  }
};

/**
 * Stock view filter queries
 */
export const STOCK_QUERIES = {
  skuFull: 'DJI-M300-RTK',
  skuPartial: 'TB60',
  skuNoResults: 'NONEXISTENT-SKU-999',
  statusAvailable: 'available',
  statusReserved: 'reserved',
  statusQuarantine: 'quarantine',
  statusDamaged: 'damaged'
};

