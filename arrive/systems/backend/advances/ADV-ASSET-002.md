---
advance:
  id: "ADV-ASSET-002"
  title: "Drone registry + readiness states"
  system: "backend"
  primary_component: "assets"
  components: ["assets", "inventory"]
  started_at: ~
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Implement a drone registry with serial identity, configuration metadata, and readiness states. Include tests for state transitions and persistence integration.

## Behavioral Change

After this advance:
- Drones can be registered with unique serial numbers
- Configuration metadata tracked per drone (model, firmware, sensors, etc.)
- Readiness states track operational status (ready, maintenance, grounded, retired)
- State transitions validated and audited
- Integration with inventory for drone tracking
- Comprehensive tests validate state machine and persistence

## Planned Implementation Tasks

- [ ] tidy: review asset domain requirements and inventory integration
- [ ] test: define drone entity tests
- [ ] feat: implement drone entity with serial identity
- [ ] test: define configuration metadata tests
- [ ] feat: implement configuration metadata storage and retrieval
- [ ] test: define readiness state machine tests
- [ ] feat: implement readiness state machine with validation
- [ ] test: define state transition tests
- [ ] feat: implement state transition logic with audit trail
- [ ] test: define inventory integration tests
- [ ] feat: integrate drone registry with inventory positions
- [ ] test: add integration tests for drone lifecycle

## Risk + Rollback

**Risks:**
- **Serial number conflicts**: Duplicate serial numbers could cause tracking issues
- **State machine bugs**: Invalid transitions could corrupt drone state
- **Configuration accuracy**: Incorrect metadata could impact operations
- **Inventory sync**: Drone registry and inventory positions could become out of sync

**Rollback Plan:**
- Feature flag drone registry
- Database migrations are reversible
- Manual drone corrections available via admin tools
- Inventory positions can be reconciled if sync fails

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Design Decisions

### Drone Entity
- Serial number (unique identifier)
- Model (drone type/variant)
- Firmware version
- Configuration metadata (sensors, payloads, capabilities)
- Readiness state
- Registration date, last maintenance date
- Owner/operator information

### Readiness States
- `Ready`: Operational and available for deployment
- `Maintenance`: Undergoing scheduled or unscheduled maintenance
- `Grounded`: Not operational due to safety/compliance issues
- `Retired`: Permanently removed from service

### State Transitions
- `Ready` ↔ `Maintenance` (scheduled maintenance)
- `Ready` → `Grounded` (safety/compliance issue)
- `Grounded` → `Maintenance` (repair required)
- `Maintenance` → `Ready` (maintenance complete, passed checks)
- Any state → `Retired` (permanent removal)

### Configuration Metadata
- Model specifications (weight, dimensions, flight time)
- Sensor configuration (cameras, LiDAR, thermal, etc.)
- Payload capacity and current payload
- Firmware version and update history
- Calibration dates and certificates
- Regulatory approvals and certifications

### Inventory Integration
- Drones tracked as serialized inventory items
- Readiness state affects inventory availability
- Only `Ready` drones available for allocation
- State transitions update inventory positions
- Movement ledger tracks drone location changes

### Audit Trail
- All state transitions logged with timestamp, user, and reason
- Configuration changes tracked with version history
- Maintenance records linked to state transitions
- Immutable audit records for compliance

