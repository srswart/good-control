# Solution Intent — CLI
**Date:** 2026-01-25  
**System:** cli  
**Stack:** Rust, clap (subcommands), shared config/logging  
**Languages:** English (primary), Chinese output optional where needed

## 1) Outcomes
- Provide a reliable, scriptable interface for ops/admin tasks:
  - database migrations and backups
  - bulk import/export
  - compliance validation and document pack generation
  - Kingdee sync trigger/retry/reconcile
  - diagnostics and health checks
- Ensure CLI actions are auditable and align with backend authorization boundaries where appropriate.

## 2) In Scope
- Subcommand structure:
  - `db` (migrate/backup/restore/verify)
  - `import` (items/locations/suppliers/customers/initial stock)
  - `export` (inventory snapshots, audit extracts, shipment manifests)
  - `compliance` (validate gates, list missing docs, build packs)
  - `kingdee` (sync, retry, reconcile, mapping checks)
  - `ops` (health checks, DB integrity checks, tracing toggles)
- Auth modes:
  - service/admin token for calling backend GraphQL
  - local DB mode only for migrations/backup/restore (strictly controlled)

## 3) Out of Scope (initially)
- Interactive TUI; keep it script-first.
- Long-running daemon mode (can be added if job runner moves out of backend).

## 4) Architecture Approach
### 4.1 Crate layout
- `crates/cli/foundation` — clap scaffolding, config, logging, output conventions
- `crates/cli/db_migrations` — migrations + backup/restore + schema verification
- `crates/cli/seed_import` — CSV/JSON import with validation and dry-run
- `crates/cli/export_report` — exports (CSV/JSON), report assembly
- `crates/cli/compliance_tools` — compliance validation commands and doc-pack builders
- `crates/cli/kingdee_sync` — integration operations and reconciliation views
- `crates/cli/ops_diagnostics` — DB integrity checks, connectivity checks, perf probes

### 4.2 Backend interaction
- Prefer calling backend GraphQL for business operations (so RBAC + gates apply).
- Allow direct SQLite access only for:
  - migrations
  - backup/restore
  - low-level integrity checks

## 5) Safety & Audit Intent
- Every command supports:
  - `--dry-run` (where applicable)
  - structured output (`--json`) for automation pipelines
  - explicit confirmation flags for destructive actions (e.g., `--yes-really`)
- Emit correlation IDs and log to the same tracing format as backend.

## 6) i18n Output
- Default to English output.
- Optional Chinese labels for exports or customer-facing extracts (configured per command).

## 7) First Moves (mapped to ARRIVE Advances)
- CLI **foundation**: clap layout, config, logging, consistent exit codes.
- Implement `db migrate` + `db backup/restore`.
- Implement `import items` + `export inventory-snapshot` as the first data pipelines.
