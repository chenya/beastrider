# Implementation Plan: M1 Lane Movement

**Branch**: `feat/m1-lane-movement` | **Date**: 2026-11-19 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/002-m1-lane-movement/spec.md`

## Summary

The player walks a hero around a lane, with instant start and stop, slower depth movement, hard
limits, and correct overlap with pillars by depth. The approach:

- **Depth** comes from Godot's built-in Y-sort: a `World` node with `y_sort_enabled` holds the
  pillars and the hero, every origin is at the feet, and the hero is the last child so it wins
  ties ([research R2](research.md)).
- **Movement** is a Rust class `Hero` (a `Node2D`, no physics body). Each physics tick it reads
  `Input.get_vector` on its player's four actions (`p1_left` and so on, built once from
  `player_index`), multiplies by the horizontal and vertical speeds, and clamps the result to the
  lane and the screen ([R4–R6](research.md)).
- **The movement step is a pure Rust function** with unit tests for diagonals, the slower vertical
  speed and all four limits ([R9](research.md)).
- **Tuning** adds `hero` and `lane` sections to the M0 JSON file, checked on load
  ([R7](research.md)). A small GDScript places the lane band from the hero's lane limits, so the
  band always matches them ([R8](research.md)).
- **M0's drifting square and speed label are removed**; the FPS counter stays ([R10](research.md)).

## Technical Context

**Language/Version**: Rust 1.100.0 (pinned in `rust/rust-toolchain.toml`) for rules and state;
GDScript (Godot 4.7.2) for presentation. Unchanged from M0.

**Primary Dependencies**: Godot **4.7.2-stable**; gdext `godot = "=0.5.5"` with `api-4-6`. No new
crates or addons. The owner's input says "Godot 4.6.x", but M0 pinned engine 4.7.2 with API level
4.6, and the owner asked to carry M0's pins over unchanged ([R1](research.md)).

**Storage**: `godot/data/tuning.json`, read once per run. No save data.

**Testing**: `cargo test` for the pure movement step and the tuning checks; everything else by
playing, using [quickstart.md](quickstart.md) scenarios 1–14.

**Target Platform**: Ubuntu 24.04 LTS, Intel UHD 630, stock Mesa, through XWayland.

**Project Type**: Desktop game: a Godot project plus a Rust GDExtension library.

**Performance Goals**: A steady 60 fps at 1280×720 with V-Sync on, while walking.

**Constraints**: Compatibility renderer only; no tuning values in code; GDScript holds no game
state; keyboard only; nothing from the spec's out-of-scope list. Sized for 2–3 sessions.

**Scale/Scope**: 1 Rust class (`Hero`), 1 pure module (`movement`), the tuning module extended,
1 new GDScript (`lane_band.gd`), 1 scene rebuilt, 4 Input Map actions, 4 tuning values. M0's
class, script and tuning section removed.

All unknowns are resolved in [research.md](research.md); none remain marked NEEDS CLARIFICATION.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.3.2. ✅ = passes, ➖ = does not apply to M1.

| Rule | Pre-research | Post-design | How M1 meets it |
| --- | --- | --- | --- |
| **I. Fun First, Playable Always** | ✅ | ✅ | M1 ends with a build where the owner walks a hero around a lane. Six of the spec's nine checks are player-visible. Movement feel is judged by playing (SC-009). |
| **II. One Milestone at a Time** | ✅ | ✅ | Only M1's list: walking, limits, depth, tuning. No collision (clarification Q1), no attack, jump, enemies, camera, gamepad, art, sound or overlay (FR-016). About 2–3 sessions. |
| **III. Architecture Serves the Game** | ✅ | ✅ | Each piece names its problem: `Hero` because movement rules must live in Rust; a pure `movement::step` because diagonals and clamping are easy to get subtly wrong and tedious to check by eye; `PlayerActions` because a second player must need no code change; `lane_band.gd` because the band must match the tuned limits. Not added: a physics body (nothing collides), a state machine (one mode, held in an `Option`), signals (nothing reacts to movement yet), a `Lane` class (two numbers). |
| **IV. Data-Driven Tuning** | ✅ | ✅ | Both speeds and both lane limits come from `tuning.json`, with no defaults in code. Hero and pillar sizes, colours and positions are looks and layout, set in the scene (clarification Q2). |
| **V. Runs Well on Weak Hardware** | ✅ | ✅ | Compatibility renderer, V-Sync, 1280×720. A handful of polygons and one Y-sorted node. Checked on the UHD 630 by walking for 60 s (quickstart 10). |
| **VI. Original Art, Cheap Animation** | ✅ | ✅ | Placeholder polygons only. "Golden Axe style" means the movement model; nothing from that game is used. No outside assets. |
| **VII. Co-op Ready, Local Only** | ✅ | ✅ | The hero has a `player_index`; its actions are built from it (`p1_*`); nothing reads "the player". A second player is a second `Hero` plus `p2_*` actions. No networking. |
| **VIII. Always Debuggable** | ➖ | ➖ | The overlay starts at M2, the restart key at M3. Work stays under git with a commit at the end. |
| **Tech: engine and API level** | ✅ | ✅ | Engine 4.7.2, API level 4.6, unchanged. |
| **Tech: Rust/GDScript split** | ✅ | ✅ | Rust owns position, speeds, limits, input reading and tuning checks. GDScript only places the band (reading Rust once) and shows the FPS. Scenes and the Input Map are set up in the editor. |
| **Tech: types over strings** | ✅ | ✅ | Action names are built once and kept as `StringName`s in one struct, not typed around the code. Tuning errors stay a Rust enum. |
| **Tech: dependencies** | ✅ | ✅ | gdext only. |
| **Tech: one tuning-data format** | ✅ | ✅ | The same JSON file and loader; two sections added ([contracts/tuning-data.md](contracts/tuning-data.md)). |
| **Code Style: Rust** | ✅ | ✅ | rustfmt at 100 columns; accessors `lane_top()` / `lane_bottom()`; `Result` for tuning and setup errors; no new `unsafe`; `///` docs on all public items and on everything exposed to Godot. |
| **Code Style: GDScript** | ✅ | ✅ | `lane_band.gd`: typed `@export var hero: Hero`, tabs, `##` class doc, code order per the constitution. Untyped declarations stay errors. |
| **Code Style: shared naming** | ✅ | ✅ | `Hero` in Rust, the editor and GDScript; actions `p1_*` match `player_index`. |
| **Quality Gate** | ✅ | ✅ | `cargo fmt --check`, build and clippy with 0 warnings, `cargo test`, 0 GDScript warnings, and every check passed by playing (quickstart 1–14). |

**Result**: both passes are green and nothing goes in Complexity Tracking.

## Spec alignment

One change to the committed spec, made because the owner's plan input is newer and explicit:

- **Speeds must be greater than 0.** The spec's edge case said a speed of 0 on one axis was valid.
  The owner's technical context requires both speeds to be greater than 0, so the edge case now
  lists 0 as an invalid value ([research R7](research.md)).

## Project Structure

### Documentation (this feature)

```text
specs/002-m1-lane-movement/
├── spec.md              # Feature spec (/speckit-specify, /speckit-clarify)
├── plan.md              # This file
├── research.md          # Phase 0: versions, Y-sort, input, limits, tuning, band, tests
├── data-model.md        # Phase 1: entities, owners, rules, states
├── quickstart.md        # Phase 1: build, play, 14 validation scenarios
├── contracts/
│   ├── hero.md          # Rust → Godot class interface, scene structure, Input Map actions
│   └── tuning-data.md   # The hero and lane sections of the tuning file
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
rust/src/
├── lib.rs                 # mod hero; mod movement; mod tuning  (drifting_square removed)
├── hero.rs                # NEW: Hero class, PlayerActions, ready-time checks
├── movement.rs            # NEW: pure step() with Speeds and Limits; unit tests
├── tuning.rs              # CHANGED: MovementTuning replaces SquareTuning; same error type
└── drifting_square.rs     # DELETED

godot/
├── project.godot          # CHANGED: [input] p1_left/right/up/down (arrows + WASD)
├── main.tscn              # REBUILT: LaneBand, World (Y-sort: pillars, Hero/Body), Hud/FpsLabel
├── data/tuning.json       # CHANGED: hero + lane sections; drifting_square removed
├── world/
│   └── lane_band.gd       # NEW: places the band from hero.lane_top()/lane_bottom()
└── ui/
    ├── fps_label.gd       # kept
    └── speed_label.gd     # DELETED (with its .uid)
```

Scene tree of `main.tscn`:

```text
Main                 (Node2D)
├── LaneBand         (ColorRect, lane_band.gd, hero → World/Hero)   drawn first: behind everything
├── World            (Node2D, y_sort_enabled = true)
│   ├── Pillar1..3   (Polygon2D, origin at base)
│   └── Hero         (Hero, player_index = 1, origin at feet)       last child: wins depth ties
│       └── Body     (Polygon2D, about 48×96 above the origin)
└── Hud              (CanvasLayer)
    └── FpsLabel     (Label, fps_label.gd)
```

**Structure Decision**: The same two-folder layout as M0. New GDScript goes in `godot/world/`
because the band belongs to the play area, not the HUD; `godot/ui/` keeps HUD scripts. Rust
unit tests stay next to the code they test.

## Complexity Tracking

No departures from the constitution, so there are no rows.

## Follow-ups (outside M1)

- **Physics interpolation**: off for now. Turn it on only if quickstart scenario 10 shows uneven
  movement ([R4](research.md)).
- **Collision**: pillars stay pass-through (clarification Q1). A physics body for the hero is
  revisited when combat or enemies need it.
- **Project Brief**: still not in the repository; the spec treats the owner's description as M1's
  must-have list.
