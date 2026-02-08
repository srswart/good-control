---
advance:
  id: "ADV-COMP-002"
  title: "Jurisdiction rule templates + evaluator"
  system: "backend"
  primary_component: "compliance"
  components: ["compliance"]
  started_at: ~
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Implement jurisdiction rule templates and an evaluator that returns structured requirements and block reasons per operation type. Add tests for UAE and KSA example templates.

## Behavioral Change

After this advance:
- Compliance rules can be defined per jurisdiction (country/region)
- Rule templates support different operation types (import, export, domestic transfer)
- Evaluator returns structured requirements (documents, approvals, restrictions)
- Block reasons are clear and actionable
- UAE and KSA jurisdiction templates are implemented and tested

## Planned Implementation Tasks

- [ ] tidy: review compliance domain requirements and operation types
- [ ] test: define jurisdiction rule template tests
- [ ] feat: implement jurisdiction rule template entity and schema
- [ ] test: define rule evaluator tests
- [ ] feat: implement rule evaluator with structured output
- [ ] test: define UAE jurisdiction template tests
- [ ] feat: implement UAE jurisdiction rule template
- [ ] test: define KSA jurisdiction template tests
- [ ] feat: implement KSA jurisdiction rule template
- [ ] test: define operation type tests (import, export, domestic)
- [ ] feat: implement operation type handling in evaluator
- [ ] test: add integration tests for rule evaluation flow

## Risk + Rollback

**Risks:**
- **Regulatory compliance**: Incorrect rules could cause regulatory violations
- **Rule complexity**: Complex rule logic may be difficult to maintain
- **Template accuracy**: Jurisdiction templates must match actual regulations
- **Evaluation performance**: Complex rule evaluation could impact performance

**Rollback Plan:**
- Feature flag jurisdiction rule evaluation
- Manual compliance review available as fallback
- Database migrations are reversible
- Rule templates can be updated without code changes

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Design Decisions

### Jurisdiction Rule Template Structure
- Jurisdiction identifier (ISO country code + optional region)
- Operation types (import, export, domestic_transfer, quarantine_release)
- Required documents per operation type
- Required approvals per operation type
- Item-level restrictions (prohibited items, controlled items)
- Expiry rules for documents and approvals

### Rule Evaluator Output
```rust
struct ComplianceEvaluation {
    allowed: bool,
    block_reasons: Vec<BlockReason>,
    required_documents: Vec<DocumentRequirement>,
    required_approvals: Vec<ApprovalRequirement>,
    warnings: Vec<String>,
}
```

### UAE Jurisdiction Template
- Import operations require: Commercial Invoice, Packing List, Certificate of Origin
- Battery imports require: MSDS, UN38.3 test report
- Drone imports require: GCAA approval
- All documents must be valid (not expired)

### KSA Jurisdiction Template
- Import operations require: Commercial Invoice, Packing List, Certificate of Origin, SABER certificate
- Battery imports require: MSDS, UN38.3 test report, SASO approval
- Drone imports require: GACA approval
- Halal certification required for certain item categories

### Operation Types
- **Import**: Goods entering jurisdiction from outside
- **Export**: Goods leaving jurisdiction
- **Domestic Transfer**: Movement within jurisdiction
- **Quarantine Release**: Release from quarantine to available inventory

