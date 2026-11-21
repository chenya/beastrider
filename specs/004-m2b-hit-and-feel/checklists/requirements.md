# Specification Quality Checklist: M2b Hit and Feel

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-11-20
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs) — *accepted exception, see Notes*
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details) — *SC-010 excepted, see Notes*
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification — *accepted exception, see Notes*

## Notes

- **Technology names are constitution constraints or owner checks.** FR-011 (rules in Rust, flash
  and shake in GDScript reacting to a signal) is the constitution's Rust/GDScript split and the
  owner's check 10; FR-012 (modes as a closed set checked exhaustively) is the constitution's
  "types over strings" rule and check 10; FR-013 (`p1_attack`) is Principle VII and check 11.
  SC-010 and User Story 5 name the Quality Gate commands because the constitution defines the gate
  in those terms. Node types, class names, data layout and how the freeze is done are left to the
  plan.
- **"Written for non-technical stakeholders"**: the only stakeholder is the owner, who is the
  developer. The stories are written as what the owner does and sees.
- **Owner's 13 acceptance checks map to the spec**: 1 → US1 sc. 1 / FR-005, FR-007 / SC-001;
  2 → US1 sc. 2 / FR-005 / SC-001; 3 → US1 sc. 6 / FR-006 / SC-002; 4 → US1 sc. 7 / FR-006 /
  SC-003; 5 → US1 sc. 8 / FR-008 / SC-004; 6 → US1 sc. 4–5 / FR-002, FR-004 / SC-005; 7 → US2 /
  FR-009, FR-010 / SC-006; 8 → US3 / FR-014, FR-015, FR-017 / SC-007, SC-008; 9 → US4 / SC-011;
  10 → FR-011, FR-012; 11 → FR-013; 12 → US5 sc. 1 / SC-009; 13 → US5 sc. 2–3 / SC-010.
- **Check 12 names the UHD 620**; the spec uses the UHD 630, the target set by constitution v1.3.2.
  Recorded in Assumptions.
- **Principle VIII (v1.3.3)**: M2b brings the show-hitboxes and spawn cheats, as the constitution
  and the brief require. Refill health and magic wait for M3 and M4.
- **Defaults chosen instead of clarifications** (see Assumptions and Edge Cases): attack ignored
  during a swing or freeze (no buffering); facing right at start; one freeze per swing; flash
  during the freeze, shake and knockback after it; horizontal-only knockback; a new hit replaces a
  running slide; touching edges and exact tolerance count as hits; F3 spawns in front of the hero
  within reach; reach drawn only while active; F2 works with the panel closed; dummies do not block
  the hero; zero allowed for wind-up, recovery and depth tolerance but not active time or reach.
- Validation passed on the first iteration.
