---
advance:
  id: "ADV-CLI-001"
  title: "CLI Foundation (clap, config, logging, subcommand scaffolding): initial implementation"
  system: "cli"
  primary_component: "foundation"
  components: ["foundation"]
  started_at: "2026-01-25T08:00:00Z"
  implementation_completed_at: "2026-01-25T13:30:00Z"
  pr_links: []
  reviewability_score: 18
  risk_flags: ["new_dependency", "public_api"]
  evidence:
    - type: "tests:unit"
      description: "7 unit tests passing (config defaults, config loading, output formats, logging initialization)"
    - type: "tdd:red-green"
      description: "Tests written for config, logging, and output modules before implementation"
  status: complete
---

## Objective

Create the first working slice of the foundation component and establish its public contracts, tests, and wiring for the cli system.

## Behavioral Change

After this advance:
- The cli system has a defined, test-backed foundation component boundary, with the minimal scaffolding needed to support subsequent feature advances.

## Completed Implementation Tasks

- [x] Created CLI foundation crate structure (`crates/cli/foundation`)
- [x] Implemented CLI configuration module with environment variable support
- [x] Implemented CLI logging module with appropriate formatting for CLI output
- [x] Implemented output conventions module with human-readable and JSON formats
- [x] Created main CLI application entry point with clap-based subcommand scaffolding
- [x] Added comprehensive unit tests (7 tests passing)
- [x] Updated workspace Cargo.toml with CLI crates and dependencies
- [x] Verified CLI builds and runs with all subcommands

## Risk + Rollback

- Risk: Primary risks are incorrect domain invariants (serial/lot/battery lifecycle), missing compliance gates, or data loss during schema evolution.
- Rollback: Feature-flag new behavior, keep migrations reversible where possible, and preserve old GraphQL fields until clients are upgraded.

## Evidence

- [x] tdd:red-green - Tests written for config, logging, and output modules
- [x] tests:unit - 7 unit tests passing covering all foundation modules
- [x] CLI builds successfully with no errors
- [x] CLI runs with --help and all subcommands functional
- [x] JSON and human-readable output formats working correctly
