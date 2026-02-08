---
advance:
  id: "ADV-BATT-002"
  title: "Battery registry + lifecycle states"
  system: "backend"
  primary_component: "battery"
  components: ["battery", "inventory", "assets"]
  started_at: ~
  implementation_completed_at: ~
  pr_links: []
  reviewability_score: 0
  risk_flags: []
  evidence: []
  status: planned
---

## Objective

Implement battery registry, lifecycle states (ready/training-only/grounded/retired), and basic policy checks. Integrate with inventory/asset linking and add tests.

## Behavioral Change

After this advance:
- Batteries can be registered with unique serial numbers
- Lifecycle states track operational status and usage restrictions
- Policy checks enforce battery usage rules (training-only, grounded, etc.)
- Integration with inventory for battery tracking
- Integration with assets for battery-drone pairing
- Comprehensive tests validate state machine and policy enforcement

## Planned Implementation Tasks

- [ ] tidy: review battery domain requirements and safety regulations
- [ ] test: define battery entity tests
- [ ] feat: implement battery entity with serial identity
- [ ] test: define lifecycle state machine tests
- [ ] feat: implement lifecycle state machine with validation
- [ ] test: define policy check tests (training-only, grounded, cycle limits)
- [ ] feat: implement policy check logic
- [ ] test: define inventory integration tests
- [ ] feat: integrate battery registry with inventory positions
- [ ] test: define asset linking tests (battery-drone pairing)
- [ ] feat: implement battery-drone pairing logic
- [ ] test: add integration tests for battery lifecycle

## Risk + Rollback

**Risks:**
- **Safety compliance**: Incorrect lifecycle states could allow unsafe battery usage
- **Policy enforcement**: Weak policy checks could violate safety regulations
- **Serial tracking**: Duplicate serial numbers could cause tracking issues
- **Asset pairing**: Incorrect battery-drone pairing could impact operations

**Rollback Plan:**
- Feature flag battery registry and policy checks
- Database migrations are reversible
- Manual battery corrections available via admin tools
- Conservative policy defaults (block if uncertain)

## Evidence

- [ ] tidy:preparatory
- [ ] tdd:red-green
- [ ] tests:unit
- [ ] tests:integration
- [ ] ci:passed
- [ ] review:owner

## Design Decisions

### Battery Entity
- Serial number (unique identifier)
- Model (battery type/chemistry)
- Capacity (mAh), voltage, chemistry (LiPo, Li-ion, etc.)
- Manufacture date, first use date
- Cycle count, charge cycles remaining
- Lifecycle state
- Health metrics (capacity degradation, internal resistance)

### Lifecycle States
- `Ready`: Operational and available for all uses
- `TrainingOnly`: Degraded but safe for training flights only
- `Grounded`: Not operational due to safety concerns
- `Retired`: Permanently removed from service

### State Transitions
- `Ready` → `TrainingOnly` (capacity degraded below threshold)
- `Ready` → `Grounded` (safety issue detected)
- `TrainingOnly` → `Grounded` (further degradation)
- `Grounded` → `Retired` (cannot be repaired)
- Any state → `Retired` (end of life)

### Policy Checks
- **Training-only enforcement**: Block operational flights for `TrainingOnly` batteries
- **Cycle limit**: Block usage if cycle count exceeds safe limit
- **Expiry**: Block usage if battery past expiry date
- **Health threshold**: Transition to `TrainingOnly` if capacity < 80% of rated
- **Grounding triggers**: Swelling, overheating, short circuit history

### Inventory Integration
- Batteries tracked as serialized inventory items
- Lifecycle state affects inventory availability
- Only `Ready` batteries available for operational allocation
- `TrainingOnly` batteries available only for training operations
- State transitions update inventory positions

### Asset Linking (Battery-Drone Pairing)
- Track which battery is paired with which drone
- Validate battery compatibility with drone model
- Enforce lifecycle state restrictions on pairing
- Track pairing history for maintenance and compliance
- Support multiple batteries per drone (swappable)

### Health Monitoring
- Track charge cycles and capacity degradation
- Monitor internal resistance and voltage sag
- Record temperature extremes and abuse events
- Automatic state transitions based on health metrics
- Alerts for batteries approaching lifecycle thresholds

