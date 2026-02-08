---
advance:
  id: "ADV-FE-004"
  title: "Session handling and passkey login UI"
  system: "frontend"
  primary_component: "app-shell"
  components: ["app-shell", "foundation"]
  started_at: "2026-02-07T16:00:00Z"
  implementation_completed_at: "2026-02-07T17:00:00Z"
  pr_links: []
  reviewability_score: 30
  risk_flags: ["auth"]
  evidence: []
  status: complete
---

## Objective

Add a passkey-based login experience to the UI, persist session tokens, and attach auth headers to GraphQL requests.

## Behavioral Change

After this advance:
- Users can sign in with passkeys
- Session tokens are persisted and reused across refreshes
- GraphQL requests include Bearer tokens when available

## Planned Implementation Tasks

- [x] tidy: centralize auth token handling
- [x] feat: add login route and passkey login UI
- [x] feat: persist session token and rehydrate current user
- [x] feat: add passkey registration button for authenticated users

## Component Impact

```yaml
components: [app-shell, foundation]
```

## Risk + Rollback

- Risk: Bad token persistence could lock users out
- Mitigation: Clear token on unauthorized errors
- Rollback: Remove token persistence and login view

## Evidence

- [ ] tests:unit - Session handling and login view tests
- [ ] ci:passed - CI pipeline passes
- [ ] review:owner - Owner review completed

## Changes Made

### 2026-02-07 - feat: add token storage and GraphQL auth headers
- `crates/ui/foundation/src/graphql/auth_token.rs`: Added auth token storage helpers
- `crates/ui/foundation/src/graphql/client.rs`: Attach Authorization header when token exists
- `crates/ui/foundation/src/graphql/mod.rs`: Exported token helpers
- `crates/ui/foundation/src/lib.rs`: Re-exported token helpers

### 2026-02-07 - feat: passkey login UI and session persistence
- `crates/ui/app_shell/src/login.rs`: Added login view and passkey registration button
- `crates/ui/app_shell/src/passkey_web.rs`: Added WebAuthn helpers for browser
- `crates/ui/app_shell/src/routes.rs`: Added `/login` route
- `crates/ui/app_shell/src/session.rs`: Added token persistence + current user hydration
- `crates/ui/app_shell/src/layout.rs`: Added logout + passkey register actions
- `crates/ui/app_shell/Cargo.toml`: Added serde/wasm-bindgen deps for login flow
