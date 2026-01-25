---
advance:
  id: "ADV-DB-001"
  title: "SQLx + SQLite persistence, migrations, repositories: initial implementation"
  system: "backend"
  primary_component: "persistence"
  components: ["persistence"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-01-25T17:05:00Z"
  pr_links: []
  reviewability_score: 28
  risk_flags: ["migration", "concurrency"]
  evidence:
    - type: "tdd:red-green"
      description: "Tests written before implementation for pool, migrations, and repository"
    - type: "tests:unit"
      description: "8 unit tests passing (error handling, pool creation, migrations, repository trait)"
    - type: "tests:integration"
      description: "Integration tests with SQLite in-memory and file-based databases"
  status: complete
---

## Objective

Create the first working slice of the persistence component and establish its public contracts, tests, and wiring for the backend system.

## Behavioral Change

After this advance:
- The backend system has a defined, test-backed persistence component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Planned Implementation Tasks

- [ ] tidy: preparatory refactoring (no behavior change)
- [ ] test: add/adjust tests describing expected behavior
- [ ] feat/fix: implement minimal changes to pass tests

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [x] tidy:preparatory
- [x] tdd:red-green
- [x] tests:unit
- [x] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Changes Made

### Files Created (9 files, ~450 LOC)

**Workspace Configuration:**
- `Cargo.toml` - Added persistence crate to workspace members, added SQLx and async-trait dependencies

**Persistence Crate:**
- `crates/backend/persistence/Cargo.toml` - Persistence crate manifest with SQLx, async-trait, and testing dependencies
- `crates/backend/persistence/src/lib.rs` - Module exports and component annotation
- `crates/backend/persistence/src/error.rs` - Error types (PersistenceError enum with Connection, Migration, NotFound, Configuration variants)
- `crates/backend/persistence/src/pool.rs` - Database pool management (DbPool, DbPoolConfig with connection pooling)
- `crates/backend/persistence/src/migrations.rs` - Migration runner using SQLx migrate macro
- `crates/backend/persistence/src/repository.rs` - Generic Repository trait for CRUD operations
- `crates/backend/persistence/migrations` - Symlink to workspace migrations directory

**Migrations:**
- `migrations/20260125000001_initial_schema.sql` - Initial database schema with users table and indexes

### Key Features Implemented

1. **Database Pool Management**
   - Configurable connection pool with max connections and timeout settings
   - Support for both in-memory and file-based SQLite databases
   - Automatic database creation if missing
   - Graceful pool shutdown

2. **Migration System**
   - SQLx-based migration runner
   - Migrations stored in workspace-level `migrations/` directory
   - Initial schema with users table for future auth component

3. **Repository Pattern**
   - Generic `Repository<T, ID>` trait with async methods
   - Standard CRUD operations: find_by_id, find_all, save, delete
   - Extensible for domain-specific repositories

4. **Error Handling**
   - Comprehensive error types using thiserror
   - Proper error propagation from SQLx
   - Domain-specific errors (NotFound, Configuration)

### Test Coverage

- **8 unit tests** covering:
  - Error message formatting
  - Default configuration values
  - Pool creation with in-memory database
  - Pool creation with file-based database
  - Invalid database URL handling
  - Migration execution
  - Repository trait compilation and basic operations

### Commit Sequence

Following Tidy First + TDD:
1. `tidy:` Added persistence crate to workspace with dependencies
2. `test:` Wrote failing tests for pool, migrations, and repository
3. `feat:` Implemented database pool, migrations, and repository trait
