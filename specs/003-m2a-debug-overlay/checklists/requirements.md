# Specification Quality Checklist: M2a Debug Overlay

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
- [x] Success criteria are technology-agnostic (no implementation details) — *SC-008 excepted, see Notes*
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

- **Technology names are constitution constraints or owner checks.** FR-011 (tuning read and
  checked in Rust, overlay in GDScript) comes from the constitution's Rust/GDScript split and the
  owner's check 6; FR-012 (named input actions `debug_toggle_overlay`, `debug_reload_tuning`)
  is the owner's check 8. SC-008 and User Story 4 name the Quality Gate commands because the
  constitution defines the gate in those terms. Node types, class names and how release builds
  leave the overlay out are left to the plan.
- **"Written for non-technical stakeholders"**: the only stakeholder is the owner, who is the
  developer. The stories are written as what the owner does and sees.
- **Owner's 10 acceptance checks map to the spec**: 1 → US1 scenarios 1–2 / FR-001 / SC-001;
  2 → US1 scenario 3 / FR-003 / SC-002; 3 → US1 scenarios 4–5 / FR-004–005 / SC-003; 4 → US2
  scenarios 1–2 / FR-006 / SC-004; 5 → US2 scenario 4 / FR-008 / SC-005; 6 → FR-011; 7 → US3 /
  FR-013 / SC-006; 8 → FR-012; 9 → US4 scenario 1 / SC-007; 10 → US4 scenarios 2–3 / SC-008.
- **Check 9 names the UHD 620**; the spec uses the UHD 630, the target set by constitution v1.3.2.
  Recorded in Assumptions.
- **Conflict with Principle VIII, not resolved by the spec**: the constitution says "From M2
  onward the game MUST have a debug overlay with cheat keys: show hitboxes, refill health and
  magic, spawn an enemy." The owner's description and the Project Brief
  ([docs/project-brief.md](../../../docs/project-brief.md)) put those three cheats in M2b, M3
  and M4, with what they act on. The spec follows the owner and the brief; the plan must either
  record this as a departure in Complexity Tracking or the constitution must be amended first
  (owner's decision).
- **Defaults chosen instead of clarifications** (see Assumptions and Edge Cases): reload result
  shown even with the overlay hidden; all-or-nothing reload; first bad value reported; F5 recovers
  a hero stopped by bad startup tuning; the always-on FPS counter stays; the origin marker is drawn
  in front of everything.
- Validation passed on the first iteration.
