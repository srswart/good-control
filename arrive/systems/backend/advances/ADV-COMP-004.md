---
advance:
  id: "ADV-COMP-004"
  title: "Document packs + gating integration"
  system: "backend"
  primary_component: "compliance"
  components: ["compliance", "warehouse", "inventory"]
  started_at: ~
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Implement document packs (attachments, versioning, expiry) and wire compliance gating into shipping and quarantine-release operations. Add tests proving operations are blocked when evidence is missing/expired.

## Behavioral Change

After this advance:
- Document packs can be created with multiple attachments
- Documents support versioning and expiry tracking
- Shipping operations are blocked when required documents are missing or expired
- Quarantine release operations are blocked when compliance evidence is insufficient
- Clear block reasons provided to operators
- Comprehensive tests validate gating logic

## Planned Implementation Tasks

- [ ] tidy: review compliance document requirements and warehouse integration points
- [ ] test: define document pack entity tests
- [ ] feat: implement document pack entity with attachments
- [ ] test: define document versioning tests
- [ ] feat: implement document versioning logic
- [ ] test: define document expiry tests
- [ ] feat: implement document expiry tracking and validation
- [ ] test: define shipping gate tests (missing docs, expired docs, valid docs)
- [ ] feat: wire compliance gating into shipping staging workflow
- [ ] test: define quarantine release gate tests
- [ ] feat: wire compliance gating into quarantine release workflow
- [ ] test: add integration tests for blocked shipping scenarios
- [ ] test: add integration tests for blocked quarantine release scenarios
- [ ] test: add integration tests for successful operations with valid documents

## Risk + Rollback

**Risks:**
- **Compliance gaps**: Incorrect gating logic could allow non-compliant operations
- **Blocking legitimate operations**: Overly strict gates may block valid operations
- **Document management**: Complex versioning logic may introduce bugs
- **Integration failures**: Gating failures could leave operations in inconsistent state

**Rollback Plan:**
- Feature flag compliance gating
- Manual override capability for authorized users (with audit trail)
- Database migrations are reversible
- Document packs can be corrected without affecting historical records

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Design Decisions

### Document Pack Structure
- Pack identifier and type (shipment_docs, quarantine_release_docs, etc.)
- Subject reference (shipment, receiving task, item, etc.)
- Multiple documents with metadata (type, version, expiry, upload date)
- Document attachments (file storage integration)
- Audit trail of all document additions/updates

### Document Versioning
- Each document can have multiple versions
- Latest version is used for compliance checks
- Historical versions retained for audit purposes
- Version metadata includes upload date, uploader, and change notes

### Document Expiry
- Expiry date tracked per document
- Automatic expiry status updates
- Expired documents fail compliance checks
- Renewal workflow supported (upload new version with new expiry)

### Shipping Gate Integration
- Evaluate compliance before dispatch confirmation
- Check required documents based on jurisdiction rules
- Validate document expiry dates
- Block dispatch if any requirements not met
- Provide clear block reasons and required actions

### Quarantine Release Gate Integration
- Evaluate compliance before release to inventory
- Check QC documentation and approvals
- Validate all required evidence present and valid
- Block release if compliance requirements not met
- Support manual override with approval and audit trail

### Block Reason Structure
```rust
struct ComplianceBlock {
    operation: String,
    blocked: bool,
    reasons: Vec<BlockReason>,
    required_actions: Vec<String>,
}

struct BlockReason {
    category: BlockCategory, // MissingDocument, ExpiredDocument, MissingApproval, etc.
    message: String,
    details: HashMap<String, String>,
}
```

