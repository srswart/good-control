---
advance:
  id: "ADV-COMP-003"
  title: "Approval workflow"
  system: "backend"
  primary_component: "compliance"
  components: ["compliance", "graphql"]
  started_at: ~
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Implement submit/review/approve/reject flows with audit records and expiry tracking. Expose GraphQL endpoints and add tests for transitions and permissions.

## Behavioral Change

After this advance:
- Users can submit items/operations for compliance approval
- Reviewers can approve or reject submissions with comments
- Approval workflow tracks state transitions with audit trail
- Approvals have expiry dates and automatic expiry handling
- GraphQL API exposes approval workflow operations
- Comprehensive tests validate state transitions and permissions

## Planned Implementation Tasks

- [ ] tidy: review compliance domain and permission requirements
- [ ] test: define approval entity and state machine tests
- [ ] feat: implement approval entity with status lifecycle
- [ ] test: define submit approval request tests
- [ ] feat: implement submit approval request logic
- [ ] test: define review/approve/reject transition tests
- [ ] feat: implement review, approve, and reject operations
- [ ] test: define expiry tracking tests
- [ ] feat: implement expiry date tracking and automatic expiry
- [ ] test: define audit trail tests
- [ ] feat: implement audit record creation for all state transitions
- [ ] test: define permission tests (who can submit, review, approve)
- [ ] feat: implement permission checks for approval operations
- [ ] test: define GraphQL mutation tests
- [ ] feat: expose GraphQL mutations for approval workflow
- [ ] test: add integration tests for complete approval lifecycle

## Risk + Rollback

**Risks:**
- **Permission bypass**: Incorrect permission checks could allow unauthorized approvals
- **Audit gaps**: Missing audit records could cause compliance issues
- **Expiry handling**: Incorrect expiry logic could allow expired approvals
- **State machine bugs**: Invalid transitions could corrupt approval state

**Rollback Plan:**
- Feature flag approval workflow
- Database migrations are reversible
- Manual approval corrections available via admin tools
- Audit trail is append-only and cannot be deleted

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Design Decisions

### Approval States
- `Pending` → `UnderReview` → `Approved` / `Rejected`
- `Approved` → `Expired` (automatic based on expiry date)
- `Rejected` → `Pending` (resubmission allowed)

### Approval Entity
- Approval type (import, export, quarantine_release, etc.)
- Subject (item, shipment, operation)
- Submitter, reviewer, approver (user references)
- Submission date, review date, approval/rejection date
- Expiry date (for approved approvals)
- Comments and attachments
- Audit trail of all state transitions

### Permission Model
- **Submit**: Any authenticated user with appropriate role
- **Review**: Compliance reviewer role
- **Approve/Reject**: Compliance approver role (may be same as reviewer)
- **View**: Submitter, reviewer, approver, and admin roles

### GraphQL API
```graphql
mutation submitApprovalRequest(input: SubmitApprovalInput!): ApprovalResult!
mutation reviewApproval(id: ID!, comment: String): ApprovalResult!
mutation approveApproval(id: ID!, expiryDate: DateTime, comment: String): ApprovalResult!
mutation rejectApproval(id: ID!, reason: String!): ApprovalResult!
query approvalRequests(filter: ApprovalFilter, pagination: PaginationInput): ApprovalConnection!
query approval(id: ID!): Approval
```

### Audit Trail
- All state transitions logged with timestamp, user, and reason
- Immutable audit records (append-only)
- Includes before/after state for all changes
- Supports compliance reporting and investigation

