# Implementation Plan: M0 Setup – Toolchain Proof

**Branch**: `feat/m0-setup` | **Date**: 2026-11-19 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/001-m0-setup/spec.md`

## Summary

Prove the fixed stack works end to end on the target machine before any gameplay exists. This
milestone sets up:

- a Godot project in `godot/`, run on the installed Godot 4.7.2 (Compatibility renderer,
  1280×720, V-Sync on);
- a Rust `cdylib` crate in `rust/` using gdext 0.5.5, pinned to API level 4.6. That is older than
  the engine, which is allowed, so the library loads in any Godot from 4.6 up.

The Rust class `DriftingSquare` loads its speed from `godot/data/tuning.json`, moves and wraps
itself, and exposes `speed()`. Two GDScript labels show that speed and the FPS, holding no state of
their own. Tuning data is **JSON**, read and checked by Rust, so every value lives in the file and
a missing value is an error rather than a hidden default ([research.md R4](research.md)). The
milestone ends with the Quality Gate passing and a clean `git status`.

## Technical Context

**Language/Version**: Rust 1.100.0 (pinned in `rust/rust-toolchain.toml`; gdext 0.5.5 needs at
least 1.94) for rules and state; GDScript (Godot 4.7.2) for presentation and glue.

**Primary Dependencies**: Godot **4.7.2-stable**, standard build, linux.x86_64, installed as
`godot` ([research.md R1](research.md)); godot-rust gdext,
crate `godot` **=0.5.5** with feature `api-4-6` (MPL-2.0). No other crates or addons.

**Storage**: One JSON tuning file, `godot/data/tuning.json`, read once per run. No save data.

**Testing**: `cargo test` for pure Rust logic (wrap maths, speed checks); everything else is
checked by playing, using the scenarios in [quickstart.md](quickstart.md).

**Target Platform**: Ubuntu 24.04 LTS desktop, Intel UHD 630 with the stock Mesa driver. Runs
through XWayland in the default Wayland session; no setting needed ([research.md R5](research.md)).

**Project Type**: Desktop game. A Godot project plus a Rust GDExtension library.

**Performance Goals**: A steady 60 fps at 1280×720 with V-Sync on, on the UHD 630.

**Constraints**: Compatibility renderer only. No magic tuning numbers in code. GDScript holds no
game state. Offline, single machine. The milestone is sized for 2–3 sessions.

**Scale/Scope**: 1 Rust class, 1 Rust tuning module, 2 GDScript scripts, 1 scene, 1 data file.

All unknowns were resolved in [research.md](research.md); none remain marked NEEDS
CLARIFICATION.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.3.2. ✅ = passes, ➖ = does not apply to M0.

| Rule | Pre-research | Post-design | How M0 meets it |
|---|---|---|---|
| **I. Fun First, Playable Always** | ✅ | ✅ | M0 ends with a build the owner runs by pressing Play. The spec has 3 player-visible checks next to its technical ones. There is no gameplay loop to playtest yet. |
| **II. One Milestone at a Time** | ✅ | ✅ | Only M0 items are included. Out-of-scope items (input, lane, enemies, art, sound, debug overlay) are excluded by FR-014. Nothing from the Not-doing list. About 2–3 sessions of work. |
| **III. Architecture Serves the Game** | ✅ | ✅ | Each piece names its problem: `DriftingSquare` because rules and state must live in Rust; a `tuning` module because loading and checking data is a separate job that later sections will reuse; a pure `wrap_x` function because the wrap must work at any speed and be testable without Godot. No state machine (two states fit in an `Option`), no signals (the speed never changes after start), no serde (one value). |
| **IV. Data-Driven Tuning** | ✅ | ✅ | Speed comes from `tuning.json`, with no default in code. The square's size and colour are placeholder looks set in the scene, not tuning values. |
| **V. Runs Well on Weak Hardware** | ✅ | ✅ | `gl_compatibility`, V-Sync on, 1280×720. No shaders, lights or particles. The 60 fps check is done on the UHD 630, and the startup log must show the Intel/Mesa GPU. |
| **VI. Original Art, Cheap Animation** | ✅ | ✅ | Placeholder only: one `ColorRect` and default-font labels. No outside assets, so the assets log is not needed yet (it is created with the first non-placeholder asset). |
| **VII. Co-op Ready, Local Only** | ✅ | ✅ | No player, no input, no camera in M0, so nothing assumes "the player". No networking. |
| **VIII. Always Debuggable** | ✅ | ✅ | The debug overlay is due at M2 and the restart key at M3, so neither is needed yet. Under git from the first session; M0 ends with a commit. |
| **Tech: engine and API level** | ✅ | ✅ | Engine: Godot 4.7.2. gdext is pinned at API level 4.6 (`api-4-6` set explicitly). 4.6 ≤ 4.7.2, so the API is not newer than the engine, and an explicit feature means a new default cannot change that. |
| **Tech: Rust/GDScript split** | ✅ | ✅ | Rust owns position, speed, wrapping, and tuning loading and checks. GDScript only reads `speed()` and the FPS. Scene and layout are built in the editor. |
| **Tech: toolchain ≥ gdext minimum** | ✅ | ✅ | 1.100.0 ≥ 1.94, pinned in `rust/rust-toolchain.toml`. |
| **Tech: types over strings** | ✅ | ✅ | No modes in M0. Tuning errors use a Rust error enum, not strings that callers have to compare. |
| **Tech: dependencies** | ✅ | ✅ | gdext only. |
| **Tech: one tuning-data format** | ✅ | ✅ | JSON, recorded in research R4 and [contracts/tuning-data.md](contracts/tuning-data.md). |
| **Code Style: Rust** | ✅ | ✅ | `rust/rustfmt.toml` with `max_width = 100`. Accessor named `speed()`. Tuning errors use `Result`. The only `unsafe` is `unsafe impl ExtensionLibrary`, with a `// SAFETY:` comment. `///` docs on all public items and on everything exposed to Godot. |
| **Code Style: GDScript** | ✅ | ✅ | `debug/gdscript/warnings/untyped_declaration` is set to Error. Typed `@export var square: DriftingSquare`. Tabs, snake_case file names, code order as in the constitution, and `##` class doc comments. |
| **Code Style: shared naming** | ✅ | ✅ | `DriftingSquare` uses the same name in Rust, the editor and GDScript. |
| **Quality Gate** | ✅ | ✅ | `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets` (0 warnings), 0 GDScript warnings, and every acceptance check passes by playing. Quickstart scenarios 1–12. |

**Result**: both passes are green and nothing goes in Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/001-m0-setup/
├── spec.md              # Feature spec (/speckit-specify)
├── plan.md              # This file
├── research.md          # Phase 0: versions, tuning format, Wayland, design decisions
├── data-model.md        # Phase 1: entities, owners, state transitions
├── quickstart.md        # Phase 1: fresh checkout → Play, plus 12 validation scenarios
├── contracts/
│   ├── drifting-square.md   # Rust → Godot class interface
│   └── tuning-data.md       # Project-wide tuning file format
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
.gitignore                     # rust/target/, godot/.godot/

rust/                          # Rust GDExtension crate "beastrider" (cdylib)
├── Cargo.toml                 # godot = { version = "=0.5.5", features = ["api-4-6"] }
├── Cargo.lock                 # committed
├── rust-toolchain.toml        # channel = "1.100.0", components rustfmt + clippy
├── rustfmt.toml               # max_width = 100
└── src/
    ├── lib.rs                 # ExtensionLibrary entry point (the one SAFETY-commented unsafe)
    ├── drifting_square.rs     # DriftingSquare class; wrap_x + its unit tests
    └── tuning.rs              # read + check tuning.json; SquareTuning, TuningError; unit tests

godot/                         # Godot project (engine 4.7.2, extension API 4.6)
├── project.godot              # Compatibility renderer, 1280×720, V-Sync, dark clear colour,
│                              #   untyped_declaration = Error, main scene
├── beastrider.gdextension     # points to ../rust/target/{debug,release}/libbeastrider.so
├── main.tscn                  # DriftingSquare (+ Body ColorRect), SpeedLabel, FpsLabel
├── data/
│   └── tuning.json            # { "drifting_square": { "speed": 160 } }
└── ui/
    ├── speed_label.gd         # reads square.speed() each frame; no state
    └── fps_label.gd           # reads Engine.get_frames_per_second() each frame
```

**Structure Decision**: Two folders in one repository, as the owner asked: `godot/` is the Godot
project and `rust/` is the library crate. They are connected only by `beastrider.gdextension`
(see [research.md R7](research.md)). Rust unit tests sit next to the code they test, as `#[cfg(test)]`
modules, because there are only two small pure functions to test. A separate `tests/` folder can
come when there are integration tests. GDScript goes in `ui/` because both M0 scripts are display
only.

## Complexity Tracking

No departures from the constitution, so there are no rows. If a later change to this plan
departs from a principle, add a table here with columns *Violation*, *Why Needed* and *Simpler
Alternative Rejected Because*.

## Follow-ups (outside M0)

- **Project Brief**: still not in the repository. The spec treats the owner's description as M0's
  must-have list.
- **`cargo test` in the Quality Gate**: this plan runs it, but the constitution's gate does not
  list it. Adding it is a constitution amendment (MINOR) if the owner wants it.
- **serde/serde_json**: revisit when checking Dictionary fields by hand gets repetitive (likely
  M4 or M6). See research R4.
- **API level 4.7**: the engine is already 4.7.2, but Rust targets the 4.6 API (the owner's
  choice, research R1). Switch `api-4-6` to `api-4-7` (and `compatibility_minimum` to 4.7) only
  when a milestone needs a 4.7-only engine API, and record the change in that milestone's plan.
