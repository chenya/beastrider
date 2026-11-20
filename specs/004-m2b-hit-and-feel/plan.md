# Implementation Plan: M2b Hit and Feel

**Branch**: `feat/m2b-hit-and-feel` | **Date**: 2026-11-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/004-m2b-hit-and-feel/spec.md`

## Summary

The hero gets its first swing and a training dummy to hit, with four impact effects tuned live,
plus the F2 hitbox and F3 spawn cheats. The approach:

- **Modes as enums**: `HeroMode { Free, Attacking { phase, elapsed, hit_this_swing } }`,
  `SwingPhase { WindUp, Active, Recovery { queued } }` and `Facing { Left, Right }`, matched
  exhaustively in `physics_process` ([research R2](research.md)).
- **Hit detection in plain Rust**: attack box and hurtboxes built from the feet origins and tuning,
  mirrored by facing; a hit needs horizontal overlap **and** a depth difference within tolerance;
  targets come from one Godot group ([R3](research.md)).
- **`TrainingDummy`** in Rust: knockback covering a distance in a time with even deceleration, from
  a closed-form offset, clamped to the lane and screen ([R4](research.md)).
- **Hit-stop**: a `HitStopTimer` autoload (`HitStop`) that the hero and dummies check each tick; a
  new freeze takes the longer time, never the sum; no `Engine.time_scale` ([R5](research.md)).
- **Presentation reacts to `was_hit`**: a `canvas_item` flash shader (checked on the Compatibility
  renderer) tweened per dummy, and a `Camera2D` offset shake running through the freeze
  ([R6](research.md)).
- **Debug**: hit counter in Rust, F2 draws boxes read from Rust, F3 spawns at a Rust-computed
  point; all freed in release builds ([R7](research.md)).
- **Tuning**: three new sections, fourteen values, same loader and reload ([R8](research.md)).

## Technical Context

**Language/Version**: Rust 1.100.0 for rules and state; GDScript (Godot 4.7.2) for presentation,
plus one `canvas_item` shader. Unchanged.

**Primary Dependencies**: Godot **4.7.2-stable**; gdext `=0.5.5` with `api-4-6`. No new crates or
addons. The input's "Godot 4.6.x" means the pinned 4.7.2 with API level 4.6 ([R1](research.md)).

**Storage**: `godot/data/tuning.json`, read at start-up and on F5.

**Testing**: `cargo test` for the pure `combat` functions and the new tuning rules (owner's list in
[R8](research.md)); everything else by playing ([quickstart.md](quickstart.md) scenarios 1–20).

**Target Platform**: Ubuntu 24.04 LTS, Intel UHD 630, stock Mesa; release export set up in M2a.

**Project Type**: Desktop game: a Godot project plus a Rust GDExtension library.

**Performance Goals**: A steady 60 fps at 1280×720 with V-Sync on, with five dummies and hitboxes
shown (apart from the intended freezes).

**Constraints**: Compatibility renderer only; no gameplay value in code; GDScript holds no game
state; keyboard only; nothing from the spec's out-of-scope list. Sized for 2–3 sessions.

**Scale/Scope**: 3 new Rust modules (`combat`, `training_dummy`, `hit_stop`), `hero` and `tuning`
extended; 1 shader and 4 new GDScripts (`dummy_flash`, `screen_shake`, `debug_hitboxes`, and the
overlay extended); 2 new scenes (`training_dummy.tscn`, `autoload/hit_stop.tscn`); `main.tscn`
gains a camera, a dummy and the hitbox drawer; 3 Input Map actions; 14 tuning values.

All unknowns are resolved in [research.md](research.md); none remain marked NEEDS CLARIFICATION.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.3.3. ✅ = passes, ➖ = does not apply to M2b.

| Rule | Pre-research | Post-design | How M2b meets it |
| --- | --- | --- | --- |
| **I. Fun First, Playable Always** | ✅ | ✅ | The hero can hit something for the first time; nine of the thirteen checks are player-visible, and the milestone ends only when the owner judges 30 hits weighty (US4, SC-011). |
| **II. One Milestone at a Time** | ✅ | ✅ | Only the brief's M2b row: a dummy to hit, the four effects tunable, show-hitboxes and spawn cheats. No health, death, moving enemies, combos, jump attacks, magic, sound or art (FR-019). 2–3 sessions. |
| **III. Architecture Serves the Game** | ✅ | ✅ | Each piece names its problem: `HeroMode`/`SwingPhase` enums because the hero now has modes with different rules; `HitStopTimer` because every object must stop together and freezes must not add up; a pure `combat` module because overlap, depth, mirroring and knockback are easy to get subtly wrong; the `was_hit` signal because flash and shake must react without the dummy knowing them; the `hittable` group because spawned dummies must be found with no wiring. Not added: `Area2D` physics, a state-machine library, health, a general effects system. |
| **IV. Data-Driven Tuning** | ✅ | ✅ | All fourteen attack, impact and hurtbox values in `tuning.json`, checked on load, reloaded live; zero turns each effect off. The dummy's drawn colour, the shader and the hitbox outline colours are looks; the spawn step is derived from the hurtbox width. |
| **V. Runs Well on Weak Hardware** | ✅ | ✅ | One tiny shader (a single `mix`), a handful of boxes, no particles or lights. Checked with five dummies and hitboxes shown on the UHD 630 (quickstart 15). The shader runs on the Compatibility renderer (*checked*, [R6](research.md)). |
| **VI. Original Art, Cheap Animation** | ✅ | ✅ | Placeholder polygons; the flash shader is written to keep working on art later. No outside assets. |
| **VII. Co-op Ready, Local Only** | ✅ | ✅ | `p{index}_attack` built from `player_index`; facing, mode and hit counter belong to each hero; dummies react to whoever hit them via `direction`. No networking. |
| **VIII. Always Debuggable** | ✅ | ✅ | v1.3.3 schedules show-hitboxes and spawn-enemy for M2b: F2 and F3, in debug builds only. Refill health and magic wait for M3 and M4. |
| **Tech: engine and API level** | ✅ | ✅ | 4.7.2, API level 4.6, unchanged. |
| **Tech: Rust/GDScript split** | ✅ | ✅ | Rust: attack timing, hits, depth rule, facing, knockback, freeze, hit counter, spawn point. GDScript: flash, shake, hitbox drawing, key handling and instancing. The constitution lists "hit detection" and "hit-feel effects (flash, shake)" on exactly these sides. |
| **Tech: types over strings** | ✅ | ✅ | `HeroMode`, `SwingPhase`, `Facing` enums with exhaustive matches; no `is_attacking` booleans; the group name is one Rust constant. |
| **Tech: dependencies** | ✅ | ✅ | gdext only. |
| **Tech: one tuning-data format** | ✅ | ✅ | The same JSON file and loader; three sections added. |
| **Code Style: Rust** | ✅ | ✅ | rustfmt at 100; `try_new`/accessor naming; `Result` for data, no panics on data; no new `unsafe`; `///` docs on all public items and exposed members. |
| **Code Style: GDScript** | ✅ | ✅ | Typed, tabs, `##` docs, code order per the constitution; constants for colours. |
| **Code Style: shared naming** | ✅ | ✅ | `TrainingDummy`, `HitStopTimer`, `Hero` and `was_hit` are the same in Rust, the editor and GDScript; autoload names differ from class names on purpose ([R5](research.md)). |
| **Quality Gate** | ✅ | ✅ | fmt, build and clippy with 0 warnings, `cargo test`, 0 GDScript warnings, every check passed by playing (quickstart 1–20), including the feel playtest. |

**Result**: both passes are green. One expected departure (scenes built by script, if the owner
hands that task over) is recorded below, as in M1 and M2a.

## Spec and input alignment

- **Knockback time 0** (owner's answer during planning): a knockback time of 0 with a distance
  above 0 is rejected by F5. The spec's clarification Q1 and FR-010 were updated to match.
- **Hit areas in tuning** (owner's input): attack-box height and the dummy's hurtbox width and
  height are tuning values, making fourteen; the spec's FR-009, Key Entities, SC-006 and Assumptions
  were updated to match.
- **F2 action name**: `debug_show_hitboxes` (owner's input); the spec's Key Entities were updated.
- **`attack.reach > 0`**: the input lists sizes as `>= 0`; the plan keeps the spec's FR-010
  ([R8](research.md)).
- **F3 spawn point**: the input says "in front of the hero, clamped to the lane"; the plan follows
  clarification Q4 (stepping) and computes the point in Rust ([R7](research.md)).
- **Class `HitStopTimer`, autoload `HitStop`**: a class and autoload may not share a name
  ([R5](research.md)).
- **Shake for spawned dummies**: the camera listens to every `TrainingDummy` that enters the tree,
  so F3's dummies shake the screen too ([R6](research.md)).

## Project Structure

### Documentation (this feature)

```text
specs/004-m2b-hit-and-feel/
├── spec.md              # Feature spec (/speckit-specify, /speckit-clarify)
├── plan.md              # This file
├── research.md          # Phase 0: modes, hits, dummy, hit-stop, signal and effects, debug, tuning
├── data-model.md        # Phase 1: entities, states, formulas, rules
├── quickstart.md        # Phase 1: build, play, 20 validation scenarios
├── contracts/
│   ├── combat-classes.md # Hero, TrainingDummy, HitStopTimer, TuningStore additions, actions
│   └── tuning-data.md    # The attack, impact and dummy sections
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
rust/src/
├── lib.rs                 # + mod combat; mod hit_stop; mod training_dummy
├── combat.rs              # NEW: Facing, SwingPhase, timings, Rect, boxes, lands(), advance_swing(),
│                          #      knockback_offset(), extend_freeze(), spawn_point(); unit tests
├── hit_stop.rs            # NEW: HitStopTimer (autoload HitStop)
├── training_dummy.rs      # NEW: TrainingDummy, was_hit, slide, clamping, hurtbox()
├── hero.rs                # CHANGED: HeroMode, facing, swing, hits, hits_landed, attack_box(), spawn point
├── tuning.rs              # CHANGED: attack, impact and dummy values in RawTuning/TuningValues; rules; tests
├── tuning_store.rs        # CHANGED: flash_time(), shake_strength(), shake_time() accessors
└── movement.rs            # unchanged

godot/
├── project.godot          # CHANGED: [autoload] HitStop; [input] p1_attack, debug_show_hitboxes, debug_spawn_dummy
├── main.tscn              # CHANGED: + ShakeCamera, + Dummy in World (before Hero), + DebugHitboxes
├── autoload/
│   └── hit_stop.tscn      # NEW: one HitStopTimer node
├── world/
│   ├── training_dummy.tscn # NEW: TrainingDummy + Body (flash material) + dummy_flash.gd
│   ├── flash.gdshader     # NEW: canvas_item flash towards white
│   ├── dummy_flash.gd     # NEW: tweens flash_amount on was_hit
│   └── screen_shake.gd    # NEW: Camera2D offset jitter on was_hit
├── debug/
│   ├── debug_overlay.gd   # CHANGED: Hits line, F2, F3, frees the hitbox drawer in release
│   └── debug_hitboxes.gd  # NEW: draws attack box and hurtboxes
└── data/tuning.json       # CHANGED: attack, impact, dummy sections
```

**Structure Decision**: Same layout. Game presentation (flash, shake, the dummy scene) goes in
`godot/world/`; debug-only scripts stay in `godot/debug/`; the new autoload scene sits beside
`tuning.tscn`. Pure combat rules get their own Rust module so they are tested without the engine,
as `movement.rs` is.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
| --- | --- | --- |
| `main.tscn`, `training_dummy.tscn`, `autoload/hit_stop.tscn` and the Input Map may be changed by the agent rather than by hand in the editor (Tech: "Scenes, nodes and animation are built in the Godot editor"), as in M0–M2a. Applies only if the owner hands those tasks over. | The agent cannot use the editor GUI, and godot-mcp's scene tools fail on this project's untyped-declaration error. The result is ordinary `.tscn` files that open in the editor. | Building by hand in the editor stays the default; relaxing the untyped-declaration error was rejected because it would break the GDScript style rule. |

## Follow-ups (outside M2b)

- **Hero hurtbox**: when enemies fight back (M3), the hero gets a hurtbox from the same `combat`
  functions.
- **Hit counter reset**: arrives with the restart key in M3.
- **Effect order**: if the feel playtest suggests a different order, it is a spec change first
  (clarification Q2), then a code change.
