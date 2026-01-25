---
advance:
  id: "ADV-MIG-001"
  title: "DB migrations + backup/restore (SQLite): initial implementation"
  system: "cli"
  primary_component: "db-migrations"
  components: ["db-migrations"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-01-25T13:45:00Z"
  pr_links: []
  reviewability_score: 22
  risk_flags: []
  evidence:
    - type: "tests:unit"
      description: "14 unit tests passing (backup, restore, verify, migrate with dry-run)"
    - type: "tdd:red-green"
      description: "Tests written before implementation for all modules"
    - type: "tests:integration"
      description: "End-to-end testing of migrate, backup, restore, verify commands"
  status: complete
---

## Objective

Create the first working slice of the db-migrations component and establish its public contracts, tests, and wiring for the cli system.

## Behavioral Change

After this advance:
- The cli system has a defined, test-backed db-migrations component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Completed Implementation Tasks

- [x] Created db-migrations crate structure (`crates/cli/db_migrations`)
- [x] Implemented migration runner with dry-run support and migration listing
- [x] Implemented SQLite backup functionality with WAL/SHM file support
- [x] Implemented SQLite restore functionality with overwrite protection
- [x] Implemented database integrity verification (PRAGMA integrity_check, table count, migration tracking)
- [x] Added comprehensive unit tests (14 tests passing)
- [x] Wired db-migrations into CLI application
- [x] Updated workspace Cargo.toml with db-migrations crate
- [x] Tested all db commands end-to-end (migrate, backup, restore, verify)
- [x] Fixed CLI argument conflict (renamed global --output to --format)

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [x] tdd:red-green - Tests written for backup, restore, verify, and migrate modules
- [x] tests:unit - 14 unit tests passing covering all db-migrations modules
- [x] tests:integration - End-to-end testing of all db commands (migrate, backup, restore, verify)
- [x] CLI builds successfully with no errors
- [x] All db commands functional with both human and JSON output formats
- [x] Database migrations applied successfully
- [x] Backup and restore operations verified
- [x] Database integrity verification working correctly
