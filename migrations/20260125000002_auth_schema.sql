-- Auth schema for authentication, authorization, and audit
-- @arrive component: auth

-- Extend users table with auth fields
ALTER TABLE users ADD COLUMN password_hash TEXT;
ALTER TABLE users ADD COLUMN is_active INTEGER NOT NULL DEFAULT 1;
ALTER TABLE users ADD COLUMN last_login_at TEXT;

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Permissions table
CREATE TABLE IF NOT EXISTS permissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    resource TEXT NOT NULL,
    action TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- User-Role assignments (many-to-many)
CREATE TABLE IF NOT EXISTS user_roles (
    user_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    assigned_at TEXT NOT NULL DEFAULT (datetime('now')),
    assigned_by INTEGER,
    PRIMARY KEY (user_id, role_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (assigned_by) REFERENCES users(id) ON DELETE SET NULL
);

-- Role-Permission assignments (many-to-many)
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id INTEGER NOT NULL,
    permission_id INTEGER NOT NULL,
    granted_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

-- Sessions table for token-based authentication
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    token TEXT NOT NULL UNIQUE,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_accessed_at TEXT NOT NULL DEFAULT (datetime('now')),
    ip_address TEXT,
    user_agent TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Audit events table
CREATE TABLE IF NOT EXISTS audit_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER,
    event_type TEXT NOT NULL,
    resource_type TEXT,
    resource_id TEXT,
    action TEXT NOT NULL,
    details TEXT,
    ip_address TEXT,
    user_agent TEXT,
    request_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_audit_events_user_id ON audit_events(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_events_event_type ON audit_events(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_events_created_at ON audit_events(created_at);
CREATE INDEX IF NOT EXISTS idx_audit_events_resource ON audit_events(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_permissions_resource_action ON permissions(resource, action);

-- Insert default roles
INSERT INTO roles (name, description) VALUES
    ('admin', 'System administrator with full access'),
    ('warehouse_operator', 'Warehouse operator with scan and task execution permissions'),
    ('warehouse_supervisor', 'Warehouse supervisor with queue management permissions'),
    ('compliance_officer', 'Compliance officer with approval and document management permissions');

-- Insert default permissions
INSERT INTO permissions (name, description, resource, action) VALUES
    -- Inventory permissions
    ('inventory.read', 'Read inventory data', 'inventory', 'read'),
    ('inventory.write', 'Create and update inventory', 'inventory', 'write'),
    ('inventory.adjust', 'Adjust inventory quantities', 'inventory', 'adjust'),
    ('inventory.scrap', 'Scrap inventory items', 'inventory', 'scrap'),
    
    -- Warehouse permissions
    ('warehouse.read', 'Read warehouse tasks and data', 'warehouse', 'read'),
    ('warehouse.execute', 'Execute warehouse tasks', 'warehouse', 'execute'),
    ('warehouse.manage', 'Manage warehouse queues and assignments', 'warehouse', 'manage'),
    
    -- Compliance permissions
    ('compliance.read', 'Read compliance data', 'compliance', 'read'),
    ('compliance.approve', 'Approve compliance documents', 'compliance', 'approve'),
    ('compliance.manage', 'Manage compliance rules and gates', 'compliance', 'manage'),
    
    -- Admin permissions
    ('admin.users', 'Manage users', 'admin', 'users'),
    ('admin.roles', 'Manage roles and permissions', 'admin', 'roles'),
    ('admin.audit', 'Access audit logs', 'admin', 'audit'),
    ('admin.system', 'System administration', 'admin', 'system');

-- Assign permissions to admin role
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin';

-- Assign permissions to warehouse_operator role
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON p.name IN (
    'inventory.read',
    'warehouse.read',
    'warehouse.execute'
)
WHERE r.name = 'warehouse_operator';

-- Assign permissions to warehouse_supervisor role
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON p.name IN (
    'inventory.read',
    'inventory.write',
    'warehouse.read',
    'warehouse.execute',
    'warehouse.manage'
)
WHERE r.name = 'warehouse_supervisor';

-- Assign permissions to compliance_officer role
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
JOIN permissions p ON p.name IN (
    'inventory.read',
    'compliance.read',
    'compliance.approve',
    'compliance.manage'
)
WHERE r.name = 'compliance_officer';

