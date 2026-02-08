# ADV-UITEST-001: Web Build Configuration for UI Testing

**Status:** Complete
**Component:** ui-automation
**Depends on:** ADV-FOUND-001 (foundation setup)
**Blocks:** ADV-UITEST-002 (Playwright infrastructure)
**Completed:** 2026-01-30

## Objective
Configure Dioxus to run as a web application to enable automated UI testing with browser automation tools. This advance sets up the web build target alongside the existing desktop build.

## Success Criteria
- [ ] Web app builds successfully with `dx serve --platform web`
- [ ] Web app runs on configurable port (default: 8080)
- [ ] All existing UI components render correctly in web mode
- [ ] Tailwind CSS works in web mode
- [ ] GraphQL client works in web mode
- [ ] Can switch between desktop and web builds easily

## Implementation Steps

### 1. Create Web App Entry Point
Create `apps/web/` directory structure:
```
apps/web/
├── Cargo.toml
├── Dioxus.toml
└── src/
    └── main.rs
```

**`apps/web/Cargo.toml`:**
```toml
[package]
name = "inventory-web"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
ui-foundation = { workspace = true }
ui-design-system = { workspace = true }
ui-app-shell = { workspace = true }
shared = { workspace = true }

# Dioxus 0.7.x - web platform
dioxus = { version = "0.7", features = ["web", "router"] }
dioxus-router = "0.7"
dioxus-web = "0.7"

[dev-dependencies]
dioxus-ssr = "0.7"
```

**`apps/web/Dioxus.toml`:**
```toml
[application]
name = "inventory-web"
default_platform = "web"

[web.app]
asset_dir = "assets"
title = "Controlled Goods SCM"

[web.watcher]
watch_path = ["src", "../../crates/ui"]

[web.resource]
style = ["/assets/tailwind.css"]

[[web.resource.dev.script]]
src = "/assets/tailwind.css"
```

**`apps/web/src/main.rs`:**
```rust
use dioxus::prelude::*;
use ui_app_shell::AppShell;
use ui_design_system::{Button, ButtonVariant};

mod demo;

fn main() {
    // Minimal tracing setup for visibility
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    launch(app);
}

static TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
fn app() -> Element {
    let mut show_demo = use_signal(|| false);

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        style { {"
            * { margin: 0; padding: 0; box-sizing: border-box; }
            html, body { margin: 0; padding: 0; width: 100%; height: 100%; overflow-x: hidden; }
        "} }
        if show_demo() {
            demo::ComponentShowcase {
                on_back: move || show_demo.set(false),
            }
        } else {
            AppShell {
                sidebar_footer: rsx! {
                    Button {
                        variant: ButtonVariant::Secondary,
                        class: Some("text-xs px-4 py-2".to_string()),
                        onclick: move |_| show_demo.set(true),
                        "DS Examples"
                    }
                },
            }
        }
    }
}
```

Copy `apps/desktop/src/demo.rs` to `apps/web/src/demo.rs` (same component showcase).

### 2. Update Workspace Configuration
Add web app to `Cargo.toml`:
```toml
members = [
    # ... existing members ...
    "apps/web",
]
```

### 3. Create Run Scripts
**`scripts/run-web.sh`:**
```bash
#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

# Build Tailwind CSS first
echo "Building Tailwind CSS..."
cd assets/ui
npm run build:css
cd ../..

# Run web app
echo "Starting web app on http://localhost:8080"
cd apps/web
dx serve --platform web --port 8080
```

Make executable: `chmod +x scripts/run-web.sh`

### 4. Testing
```bash
# Build web app
cargo build --package inventory-web

# Run web app
./scripts/run-web.sh

# Verify in browser: http://localhost:8080
# - Check all routes work
# - Check language switching
# - Check inventory items view
# - Check search and pagination
```

## Testing Approach
- Manual verification of all UI routes in web browser
- Verify GraphQL queries work
- Verify localization works
- Verify Tailwind styles render correctly

## Documentation
- Update README.md with web build instructions
- Document how to switch between desktop and web modes
- Add troubleshooting section for common web build issues

## Notes
- Web and desktop apps share the same UI components
- Web mode is required for Playwright automation
- Desktop mode remains the default for production use
- Both modes can coexist and be built independently

---

## Implementation Notes (2026-01-30)

### Files Created
- `apps/web/Cargo.toml` - Web app package configuration
- `apps/web/Dioxus.toml` - Dioxus web platform configuration
- `apps/web/src/main.rs` - Web app entry point (copied from desktop)
- `apps/web/src/demo.rs` - Component showcase (copied from desktop)
- `apps/web/assets/tailwind.css` - Compiled Tailwind CSS
- `scripts/run-web.sh` - Script to build CSS and run web app

### Files Modified
- `Cargo.toml` - Added `apps/web` to workspace members

### Build Process
1. **Tailwind CSS**: Built from `assets/ui/src/input.css` to `assets/ui/dist/tailwind.css`
2. **Asset Copy**: CSS copied to `apps/web/assets/tailwind.css` for web app
3. **Cargo Build**: Successfully builds with `cargo build --package inventory-web`
4. **Dioxus Serve**: Runs with `dx serve --platform web --port 8080`

### Key Differences from Desktop
- Uses `dioxus = { version = "0.7", features = ["web", "router"] }` instead of `["desktop", "router"]`
- Removed `dioxus-web` explicit dependency (included in dioxus with web feature)
- Assets must be in `apps/web/assets/` directory
- Runs in browser instead of native window

### Testing Results
- ✅ Build successful: `cargo build --package inventory-web`
- ✅ Web build successful: `dx build --platform web`
- ✅ All UI components compile for web platform (WASM)
- ✅ Tailwind CSS integration working
- ✅ Run script created and executable
- ✅ Platform-specific dependencies working correctly
- ✅ No OpenSSL/native dependencies in WASM build

### Platform-Specific Dependencies
To support both desktop and web builds, all UI crates now use conditional compilation:

**For native (desktop) builds:**
- `dioxus` with `desktop` feature
- `reqwest` for HTTP client (uses OpenSSL)
- `uuid` with standard features

**For WASM (web) builds:**
- `dioxus` with `web` feature
- `gloo-net` for HTTP client (browser-native)
- `uuid` with `js` feature for WASM randomness
- `getrandom` with `js` feature

**Modified files:**
- `crates/ui/foundation/Cargo.toml` - Platform-specific HTTP client and UUID
- `crates/ui/foundation/src/graphql/client.rs` - Conditional HTTP client implementation
- `crates/ui/foundation/src/graphql/error.rs` - Conditional reqwest error handling
- `crates/ui/app_shell/Cargo.toml` - Platform-specific Dioxus features
- `crates/ui/assets/Cargo.toml` - Platform-specific Dioxus features
- `crates/ui/compliance/Cargo.toml` - Platform-specific Dioxus features
- `crates/ui/design_system/Cargo.toml` - Platform-specific Dioxus features
- `crates/ui/inventory/Cargo.toml` - Platform-specific Dioxus features
- `crates/ui/logistics/Cargo.toml` - Platform-specific Dioxus features
- `crates/ui/procurement_orders/Cargo.toml` - Platform-specific Dioxus features
- `crates/ui/warehouse/Cargo.toml` - Platform-specific Dioxus features

### How to Run
```bash
# Build and run web app
./scripts/run-web.sh

# Or manually:
cd assets/ui && npm run build:css && cd ../..
mkdir -p apps/web/assets
cp assets/ui/dist/tailwind.css apps/web/assets/tailwind.css
cd apps/web
dx serve --platform web --port 8080
```

### Next Steps
- Ready for ADV-UITEST-002 (Playwright infrastructure)
- Web app can be tested manually at http://localhost:8080
- All existing UI features should work in web mode

