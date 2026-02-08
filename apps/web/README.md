# Inventory Web App

Dioxus web frontend for the inventory system.

## Environment

The GraphQL endpoint is provided at runtime using:

- `APP__GRAPHQL_ENDPOINT` (default: `http://localhost:3000/graphql`)

## Local Run

```
dx serve --platform web --package inventory-web
```

## Fly.io Deploy

This app is deployed as its own Fly.io sprite named `good-control`.

```
scripts/deploy-frontend-fly.sh --create-app --region sin
```

Smoke check:

```
curl -f https://good-control.fly.dev/health
```
