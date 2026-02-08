---
advance:
  id: "ADV-PROC-003"
  title: "Supplier compliance documents"
  system: "backend"
  primary_component: "procurement"
  components: ["procurement", "compliance"]
  started_at: ~
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Require supplier documentation for certain item classes and block receiving when missing. Add tests for required vs optional cases.

## Behavioral Change

After this advance:
- Supplier documentation requirements can be defined per item class
- Receiving is blocked when required supplier documents are missing or expired
- Optional documents are tracked but do not block receiving
- Clear block reasons provided to operators
- Comprehensive tests validate required vs optional document scenarios

## Planned Implementation Tasks

- [ ] tidy: review compliance document requirements and item classification
- [ ] test: define supplier document requirement tests
- [ ] feat: implement supplier document requirement configuration per item class
- [ ] test: define document validation tests
- [ ] feat: implement document validation logic (presence, expiry)
- [ ] test: define receiving gate tests (required docs missing, expired, valid)
- [ ] feat: wire supplier document validation into receiving workflow
- [ ] test: define optional document tests
- [ ] feat: implement optional document tracking without blocking
- [ ] test: add integration tests for blocked receiving scenarios
- [ ] test: add integration tests for successful receiving with valid documents

## Risk + Rollback

**Risks:**
- **Compliance gaps**: Incorrect document requirements could allow non-compliant receipts
- **Blocking legitimate receipts**: Overly strict requirements may block valid receipts
- **Document management**: Complex document tracking may introduce bugs
- **Integration failures**: Document validation failures could prevent receiving

**Rollback Plan:**
- Feature flag supplier document validation
- Manual override capability for authorized users (with audit trail)
- Database migrations are reversible
- Document requirements can be adjusted without code changes

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Design Decisions

### Supplier Document Requirements
- Configured per item class (batteries, drones, electronics, etc.)
- Document types: MSDS, COO, COC, test reports, certifications
- Required vs optional designation
- Expiry tracking for time-sensitive documents

### Document Validation
- Check document presence for required documents
- Validate expiry dates for time-sensitive documents
- Support document versioning (latest version used)
- Audit trail of all document uploads and validations

### Receiving Gate Integration
- Evaluate supplier documents before receiving task creation
- Block receiving if required documents missing or expired
- Provide clear block reasons and required actions
- Allow receiving to proceed if only optional documents missing

### Item Class Examples
- **Batteries**: Require MSDS, UN38.3 test report, COO
- **Drones**: Require COO, COC, regulatory approval certificate
- **Electronics**: Require COO, RoHS certificate (optional)
- **General**: Require COO only

### Block Reason Structure
```rust
struct SupplierDocumentBlock {
    blocked: bool,
    missing_required_documents: Vec<DocumentType>,
    expired_documents: Vec<DocumentInfo>,
    missing_optional_documents: Vec<DocumentType>,
    required_actions: Vec<String>,
}
```

### Integration Points
- **Compliance**: Document pack management and validation
- **Warehouse**: Receiving task creation and validation
- **Procurement**: Supplier performance tracking (document compliance rate)

