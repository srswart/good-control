# Good Control Inventory

Good Control is a Rust workspace that provides a backend API and a Dioxus-based frontend for inventory, warehouse, logistics, procurement, and compliance workflows.

## What is in this repo

- Backend (Axum + GraphQL + SQLite): `apps/server` and `crates/backend/*`
- Frontend UI (Dioxus web/desktop): `apps/web`, `apps/desktop`, and `crates/ui/*`
- Domain logic: `crates/domain/*`
- Governance and change tracking: `arrive/`

## Prerequisites

- Rust toolchain (edition 2024 workspace)
- SQLite (bundled via `sqlx` for dev/test)

## Install

Clone and build the workspace:

```
git clone git@github.com:srswart/good-control.git
cd good-control
cargo build
```

## Run locally

### Backend API

Run the server (Axum + GraphQL):

```
cargo run -p inventory-server
```

The server reads configuration from `APP__*` environment variables. For example:

```
export APP__SERVER__HOST=0.0.0.0
export APP__SERVER__PORT=3000
export APP__DATABASE__URL="sqlite://data/inventory.db"
```

### Web frontend

Run the Dioxus web app in dev mode:

```
cargo run -p inventory-web
```

### Desktop frontend

Run the desktop app:

```
cargo run -p inventory-desktop
```

## Tests

Run the full test suite:

```
cargo test
```

Targeted tests:

```
cargo test -p graphql
cargo test -p ui-admin
```

## ARRIVE governance

This repo uses ARRIVE to track changes, architectural intent, and reviewability.

- **Advances** live under `arrive/systems/<system>/advances/` and document scope, risks, evidence, and change history.
- **System definitions** are under `arrive/systems/*/system.yaml`.
- Each change that affects a system should be linked to an advance.

ARRIVE helps ensure:

- Changes are scoped and reviewable.
- Risk and rollback are documented.
- Tests and evidence are recorded alongside implementation.

## Notes

- Mock data reload is available in the admin UI to seed demo data.
- The application can be deployed on services such as Fly.io.
