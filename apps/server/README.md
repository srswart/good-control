# Inventory Backend Server

Backend GraphQL server for the inventory system.

## Environment

The server reads settings from environment variables using the `APP__` prefix.

Required:
- `APP__SERVER__HOST` (default `127.0.0.1`)
- `APP__SERVER__PORT` (default `3000`)
- `APP__LOG_LEVEL` (default `info`)
- `APP__DATABASE__URL` (default `sqlite:data/drone-inventory.db`)
- `APP__DATABASE__MAX_CONNECTIONS` (default `5`)
- `APP__DATABASE__CONNECTION_TIMEOUT_SECS` (default `30`)

## Local Run

```
cargo run -p inventory-server
```

## Fly.io Deploy

This app is deployed as its own Fly.io sprite named `good-control-api`.

```
scripts/deploy-backend-fly.sh --create-app --create-volume --region sin
```

Smoke check:

```
curl -f https://good-control-api.fly.dev/health
```
