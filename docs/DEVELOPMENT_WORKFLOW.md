# Development Workflow

This document outlines the development workflow and practices for the drone inventory management system.

## Advance-Based Development

This project follows the ARRIVE methodology, organizing work into discrete "advances" that represent incremental, testable improvements to the system.

### Branch Naming Convention

**Every advance MUST be developed on a dedicated feature branch.**

Branch naming format:
```
adv/<advance-id>-<short-description>
```

Examples:
- `adv/ADV-CLI-001-cli-foundation`
- `adv/ADV-MIG-001-db-migrations`
- `adv/ADV-AUTH-001-auth-rbac-audit`

### Workflow for Implementing an Advance

1. **Create a branch** for the advance:
   ```bash
   git checkout -b adv/ADV-XXX-YYY-short-description
   ```

2. **Implement the advance** following TDD practices:
   - Write tests first (red)
   - Implement minimal code to pass tests (green)
   - Refactor as needed
   - Ensure all tests pass

3. **Update the advance document** in `arrive/systems/<system>/advances/`:
   - Mark status as `complete`
   - Add implementation completion timestamp
   - Update evidence section
   - Document completed tasks

4. **Commit your work** with clear, descriptive commit messages:
   ```bash
   git add .
   git commit -m "feat(ADV-XXX-YYY): Brief description of changes"
   ```

5. **Run the full test suite** to ensure nothing broke:
   ```bash
   cargo test --workspace
   ```

6. **Push the branch** and create a pull request:
   ```bash
   git push origin adv/ADV-XXX-YYY-short-description
   ```

7. **Merge to main** after review and CI passes

### Commit Message Convention

Use conventional commits format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature (maps to an advance)
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `chore`: Maintenance tasks

Scopes should reference the advance ID when applicable:
- `feat(ADV-CLI-001): Add CLI foundation with clap scaffolding`
- `feat(ADV-AUTH-001): Implement RBAC and audit system`

### Testing Requirements

Before merging any advance:

1. **All tests must pass**:
   ```bash
   cargo test --workspace
   ```

2. **No compiler warnings** (except explicitly documented exceptions)

3. **Code builds successfully**:
   ```bash
   cargo build --workspace
   ```

4. **Advance document updated** with:
   - Status: `complete`
   - Implementation completion timestamp
   - Evidence of tests passing
   - Reviewability score

### Branch Protection

- `main` branch is protected
- All changes must go through pull requests
- CI must pass before merging
- At least one approval required (when team grows)

### Example Complete Workflow

```bash
# 1. Create branch for advance
git checkout -b adv/ADV-CLI-001-cli-foundation

# 2. Implement the advance (write tests, code, etc.)
# ... development work ...

# 3. Run tests
cargo test --workspace

# 4. Update advance document
# Edit arrive/systems/cli/advances/ADV-CLI-001.md

# 5. Commit changes
git add .
git commit -m "feat(ADV-CLI-001): Add CLI foundation with clap scaffolding

- Created cli-foundation crate with config, logging, and output modules
- Implemented main CLI app with subcommand structure
- Added 7 unit tests
- Updated workspace Cargo.toml"

# 6. Push and create PR
git push origin adv/ADV-CLI-001-cli-foundation

# 7. After review and CI passes, merge to main
```

## Code Review Guidelines

When reviewing advances:

1. **Check the advance document** is properly updated
2. **Verify tests are comprehensive** and passing
3. **Ensure code follows project conventions**
4. **Check for security issues** (especially in auth, permissions)
5. **Validate database migrations** are reversible where possible
6. **Review error handling** and edge cases

## Continuous Integration

CI pipeline runs on every push and PR:

1. **Build check**: `cargo build --workspace`
2. **Test suite**: `cargo test --workspace`
3. **Linting**: `cargo clippy --workspace`
4. **Format check**: `cargo fmt --check`

## Database Migrations

When adding migrations:

1. **Create migration file** in `migrations/` with timestamp prefix
2. **Test migration** with CLI: `cargo run --package inventory-cli -- db migrate`
3. **Verify rollback** if applicable
4. **Document schema changes** in advance document
5. **Update symlinks** in crates that need migrations

## Release Process

(To be defined as project matures)

## Questions?

For questions about the development workflow, consult:
- Project brief: `docs/project-brief.md`
- Implementation plan: `docs/IMPLEMENTATION_PLAN.md`
- ARRIVE methodology: `arrive/` directory

