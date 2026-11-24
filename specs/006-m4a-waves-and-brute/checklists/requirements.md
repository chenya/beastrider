# Specification Quality Checklist: M4a Waves and the Brute

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-11-23
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details)
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification

## Notes

- Iteration 1 (2026-11-23): two [NEEDS CLARIFICATION] markers remain, both waiting on the owner:
  whether a blow cancels a brute's attack (US2 sc. 5, FR-008), and whether the hero heals between
  waves (Assumptions).
- Iteration 2 (2026-11-23): the owner answered both (Q1: B, Q2: A). A brute keeps attacking through
  blows; health carries over the whole run. Both are recorded under Clarifications and in US2,
  FR-008, SC-003 and Assumptions. All items pass.
- Technology names (Rust, GDScript, the shared M3 fighter parts) appear only in the note at the top
  and in FR-009 and FR-022. They restate constitution rules and the owner's check 11, as in the M3
  spec, so the content-quality items pass.
- The success criteria name `cargo` and the editor (SC-011) because the owner's check 13 and the
  constitution's Quality Gate define success that way; this matches M0–M3.
- Items marked incomplete require spec updates before `/speckit-clarify` or `/speckit-plan`
