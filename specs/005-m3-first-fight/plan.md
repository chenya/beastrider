# Implementation Plan: M3 First Fight

**Branch**: `feat/m3-first-fight` | **Date**: 2026-11-22 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/005-m3-first-fight/spec.md`

## Summary

A grunt hunts the hero, both sides have health and die, and a run ends in Game Over and restarts in
under two seconds. The approach:

- **Shared fighter parts as plain Rust structs** in a new pure module `fighter.rs`: `Health`,
  `AttackSpec`/`AttackTimer`, `Knockback`, `Countdown`, `Team`, and one `resolve_attack` hit rule,
  composed as fields into `Hero` and `Grunt`, each unit-tested ([research R2](research.md)).
- **Teams as groups** (`team_players`, `team_enemies`); an attacker only looks at the other team; a
  `FighterRef` enum reaches the hero, a grunt or a dummy with exhaustive matches
  ([R3](research.md)).
- **Hero modes** `Free, Attacking, HitStun, Dead`, with invulnerability as a timer beside the mode
  ([R4](research.md)).
- **`Grunt`** with `Entering, Approaching, WindUp, Active, Recovery, HitStun, Dying`; it lines up at
  the nearest player's depth, then closes to a slot at reach ([R5](research.md)).
- **`GruntSpawner`**: the first grunt at once, then a new one only when none is left, after the
  respawn delay; `spawn_now()` for F3 ([R6](research.md)).
- **`Run` autoload** (`RunTracker`): kills, `Playing`/`GameOver`, `game_over(kills)`, and R reloads
  the scene with tuning kept ([R7](research.md)).
- **Presentation in GDScript**: health bars, kills, Game Over, blink, wind-up colour, fall and fade,
  all reading Rust or reacting to its signals ([R8](research.md)).
- **Tuning restructured** into `hero`, `grunt` (each with an `attack` object), `dummy`, `lane`,
  `effects`, `run`; a grunt wind-up not longer than the hero's is accepted with a warning
  ([R10](research.md), [contracts/tuning-data.md](contracts/tuning-data.md)).

## Technical Context

**Language/Version**: Rust 1.100.0 for rules and state; GDScript (Godot 4.7.2) for presentation.
Unchanged.

**Primary Dependencies**: Godot **4.7.2-stable**; gdext `=0.5.5` with `api-4-6`. No new crates or
addons; randomness from Godot's `RandomNumberGenerator` ([R1](research.md)).

**Storage**: `godot/data/tuning.json`, restructured, read at start-up and on F5.

**Testing**: `cargo test` for every pure rule (owner's list in [R11](research.md)); everything else
by windowed check scripts and by playing ([quickstart.md](quickstart.md) scenarios 1–21).

**Target Platform**: Ubuntu 24.04 LTS, Intel UHD 630, stock Mesa; release export as in M2a.

**Project Type**: Desktop game: a Godot project plus a Rust GDExtension library.

**Performance Goals**: A steady 60 fps at 1280×720 with V-Sync on, with five grunts, hitboxes shown
and the panel open (apart from the intended freezes); restart in under 2 s.

**Constraints**: Compatibility renderer only; no gameplay value in code; GDScript holds no game
state; signal handlers never call back into the emitter ([R8](research.md)); keyboard only; nothing
from the spec's out-of-scope list.

**Scale/Scope**: 5 new Rust modules (`fighter`, `fighters`, `grunt`, `grunt_spawner`, `run`);
`hero`, `training_dummy`, `combat`, `hit_stop`, `tuning`, `tuning_store` changed; 2 new scenes
(`grunt.tscn`, `autoload/run.tscn`); about 7 new or renamed GDScripts; 4 Input Map actions added or
moved; tuning grows from 18 to 40 values. The largest milestone so far: about three sessions,
inside the constitution's two-week limit, with the game playable after each phase of tasks.

All unknowns are resolved in [research.md](research.md); none remain marked NEEDS CLARIFICATION.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.3.3. ✅ = passes.

| Rule | Pre-research | Post-design | How M3 meets it |
| --- | --- | --- | --- |
| **I. Fun First, Playable Always** | ✅ | ✅ | The first full run: fight, lose, restart. Ten of the fourteen checks are player-visible, and the milestone ends only when the owner judges five runs fair (US5, SC-012). Tasks keep the game runnable after each phase. |
| **II. One Milestone at a Time** | ✅ | ✅ | Only the brief's M3 row: a chasing grunt, health bars, death, game over, restart, refill-health cheat. No waves, brute, potions, magic, jumping or score beyond kills (FR-022). One milestone by the owner's choice; sized at about three sessions, inside two weeks. |
| **III. Architecture Serves the Game** | ✅ | ✅ | Each piece names its problem: shared structs because hero and grunt need the same health, attack, stun and knockback rules and must not drift; teams because grunts must not hit grunts; `FighterRef` because an attack must reach three different classes; `Run` because game over and restart must outlive and rebuild the scene; the spawner because grunts must come back; mode enums because both fighters now have several modes. Not added: a trait hierarchy, component nodes, navigation, an event bus, a state-machine library. |
| **IV. Data-Driven Tuning** | ✅ | ✅ | All 22 new values (health, damage, speeds, attack specs, knockback, stun, invulnerability, respawn delay, fade) in `tuning.json`, checked on load, reloaded live. The approach slot is derived from reach and body widths; colours, blink rate and fall angle are looks. |
| **V. Runs Well on Weak Hardware** | ✅ | ✅ | A handful of polygons, the M2b flash shader, two rectangles per grunt bar; no lights or particles. Five grunts checked on the UHD 630 (quickstart 15); restart timed there (quickstart 9). |
| **VI. Original Art, Cheap Animation** | ✅ | ✅ | Placeholder polygons; the fall and fade are tweens on placeholders. No outside assets. |
| **VII. Co-op Ready, Local Only** | ✅ | ✅ | Health belongs to each hero; grunts target the nearest living member of `team_players`, not "the hero"; F4 refills the overlay's player-1 hero by reference; actions stay `p{index}_*`. No networking. |
| **VIII. Always Debuggable** | ✅ | ✅ | F4 refill health arrives as v1.3.3 schedules; F3 spawns the new enemy, F6 keeps the dummy, F2 shows every box; debug builds only. R restarts a run in under 2 s in every build, as required from M3. |
| **Tech: engine and API level** | ✅ | ✅ | 4.7.2, API level 4.6, unchanged. |
| **Tech: Rust/GDScript split** | ✅ | ✅ | Rust: enemy behaviour, health, damage, death, kills, run state, spawning, restart, hit rules. GDScript: bars, kills label, Game Over, blink, wind-up colour, fall, fade, flash, shake, cheats' key handling. |
| **Tech: types over strings** | ✅ | ✅ | `HeroMode`, `GruntMode`, `AttackPhase`, `SpawnerState`, `RunState`, `Team`, `Damage`, `FighterRef` enums, matched exhaustively; group and action names are Rust constants. |
| **Tech: dependencies** | ✅ | ✅ | gdext only. |
| **Tech: one tuning-data format** | ✅ | ✅ | The same JSON file and loader, restructured; nested `attack` objects read by one function. |
| **Code Style: Rust** | ✅ | ✅ | rustfmt at 100; `try_new`/accessor naming; `Result` for data; no new `unsafe`; `///` docs on all public items and exposed members. |
| **Code Style: GDScript** | ✅ | ✅ | Typed, tabs, `##` docs, code order per the constitution; constants for looks; body scripts extend `hit_flash.gd`. |
| **Code Style: shared naming** | ✅ | ✅ | `Grunt`, `GruntSpawner`, `RunTracker`, `died`, `game_over` the same in Rust, the editor and GDScript; autoload `Run` differs from class `RunTracker` on purpose (M2b lesson). |
| **Quality Gate** | ✅ | ✅ | fmt, build and clippy with 0 warnings, `cargo test`, 0 GDScript warnings, every check passed by playing (quickstart 1–21), including the fairness playtest. |

**Result**: both passes are green. The T055 review after implementation found the work matches
every row above; its one departure (scenes and settings changed by the agent, at the owner's
request) is recorded below, as in M0–M2b. Details settled while building are recorded in research:
a grunt does not start an attack while its target is invulnerable, so it waits out the blink (R5);
`Countdown::tick` reports "run out" on every tick after it reaches 0, so a zero-length stun or delay
ends at once (R2); the shared flash script is the class `HitFlash`, which the hero's and the grunt's
body scripts extend (R8); and the Game Over layer sits between the HUD (layer 1) and the debug
overlay, which moved to layer 3 (R8).

## Spec and input alignment

Decided by the owner during planning; [spec.md](spec.md) was updated to match:

- **Respawn rule**: a new grunt comes only when none is left (the input's rule), replacing the
  `/speckit-specify` answer "every kill is replaced". Clarifications, US2 sc. 4, Edge Cases, FR-010,
  SC-005 and the US6 test (F3 keeps five in play) updated.
- **Wind-up check**: a grunt wind-up not longer than the hero's is accepted with a warning (the
  input's rule), revising clarification Q2's "no rule". FR-006 updated.
- **Knockback per attacker** (the input's `AttackSpec`): each fighter's attack has its own knockback
  distance and time; freeze, flash and shake stay shared. FR-007, FR-015 and Assumptions updated.

Resolved in the plan without a spec change:

- **UHD 620 → UHD 630**: the input names the 620; the constitution and the spec's check 13 name the
  630 ([R1](research.md)).
- **Tuning sections**: the input lists `hero, grunt, effects, run`; `lane` and `dummy` are kept, and
  `depth_tolerance` moves to `lane` because it is shared by every attacker ([R10](research.md)).
- **Class `RunTracker`, autoload `Run`**: a class and its autoload may not share a name. The input's
  `RunState { Playing, GameOver }` is kept as the Rust enum.
- **Hero's queued press** stays in the hero, not in `AttackTimer`: grunts have no input
  ([R2](research.md)).
- **Approach**: the input's slot approach also keeps the spec's "line up first, then close in"
  (FR-003): horizontal movement waits until the depth is within tolerance ([R5](research.md)).
- **Bars poll; signals carry data**: health bars and kills read every frame, because gdext forbids a
  handler calling back into an object Rust holds during an emit ([R8](research.md)).

## Project Structure

### Documentation (this feature)

```text
specs/005-m3-first-fight/
├── spec.md              # Feature spec (/speckit-specify, /speckit-clarify, planning answers)
├── plan.md              # This file
├── research.md          # Phase 0: shared parts, teams, hero, grunt, spawner, run, presentation,
│                        #          cheats, tuning, tests
├── data-model.md        # Phase 1: structs, modes and transitions, formulas, rules
├── quickstart.md        # Phase 1: build, play, 21 validation scenarios
├── contracts/
│   ├── fight-classes.md # Hero, Grunt, TrainingDummy, GruntSpawner, RunTracker, HitStop, Tuning,
│   │                    # groups, actions, scenes
│   └── tuning-data.md   # The restructured file, rules, warning, when values apply
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
rust/src/
├── lib.rs                 # + mod fighter, fighters, grunt, grunt_spawner, run
├── fighter.rs             # NEW (pure): Health, AttackSpec, AttackTimer, Knockback, Countdown, Team,
│                          #   resolve_attack(), approach_step(), grunt_after_hit(); unit tests
├── fighters.rs            # NEW: FighterRef { Hero, Grunt, Dummy }: target(), take_hit()
├── grunt.rs               # NEW: Grunt, GruntMode, was_hit, died, health, is_winding_up, boxes
├── grunt_spawner.rs       # NEW: GruntSpawner, SpawnerState::tick (pure, tested), spawn_now()
├── run.rs                 # NEW: RunTracker (autoload Run), RunRecord/RunState (pure, tested)
├── combat.rs              # CHANGED: SwingPhase/advance_swing move to fighter.rs; HITTABLE_GROUP
│                          #   removed; boxes, lands, new_hits, knockback_offset, clamp, spawn_point stay
├── hero.rs                # CHANGED: HeroMode + HitStun/Dead, health, invulnerability, knockback,
│                          #   team, was_hit/died, health accessors, refill_health, hurtbox()
├── training_dummy.rs      # CHANGED: team_enemies, Knockback from fighter.rs, attacker's knockback
├── hit_stop.rs            # CHANGED: clear()
├── tuning.rs              # CHANGED: restructured sections, nested attack objects, warnings(); tests
├── tuning_store.rs        # CHANGED: effects/run accessors, death_fade_time(), warning()
└── movement.rs            # unchanged

godot/
├── project.godot          # CHANGED: autoload Run; actions game_restart, debug_spawn_grunt,
│                          #   debug_refill_health; debug_spawn_dummy → F6
├── main.tscn              # CHANGED: − Dummy; + GruntSpawner; Hero/Body flash + hero_body.gd;
│                          #   Hud + HealthBar, KillsLabel; + GameOver layer; Panel moved down
├── autoload/
│   └── run.tscn           # NEW: one RunTracker node named Run
├── world/
│   ├── grunt.tscn         # NEW: Grunt + Body (flash, grunt_body.gd) + HealthBar
│   ├── hit_flash.gd       # RENAMED from dummy_flash.gd: flash on the parent's was_hit
│   ├── hero_body.gd       # NEW: extends hit_flash.gd; blink, fall
│   ├── grunt_body.gd      # NEW: extends hit_flash.gd; wind-up colour, fade on died
│   ├── grunt_health_bar.gd # NEW: two rectangles above the head
│   ├── screen_shake.gd    # CHANGED: also listens to Hero and Grunt
│   └── training_dummy.tscn # CHANGED: script path hit_flash.gd
├── ui/
│   ├── health_bar.gd      # NEW: hero bar, top-left
│   ├── kills_label.gd     # NEW: kills during play
│   └── game_over.gd       # NEW: shows on Run.game_over(kills)
├── debug/
│   ├── debug_overlay.gd   # CHANGED: F3 grunt, F4 refill, F6 dummy, tuning warning
│   └── debug_hitboxes.gd  # CHANGED: hero and grunt bodies, grunt attack boxes
└── data/tuning.json       # CHANGED: restructured (contracts/tuning-data.md)
```

**Structure Decision**: Same layout. Pure rules shared by fighters get their own module
(`fighter.rs`) so they are tested without the engine, as `movement.rs` and `combat.rs` are;
Godot classes get one module each. HUD and Game Over scripts go in `godot/ui/` beside the FPS label;
fighter presentation in `godot/world/`; debug-only scripts stay in `godot/debug/`.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
| --- | --- | --- |
| `main.tscn`, `grunt.tscn`, `autoload/run.tscn`, `training_dummy.tscn` and the Input Map were changed by the agent rather than by hand in the editor (Tech: "Scenes, nodes and animation are built in the Godot editor"), as in M0–M2b. The owner handed T002, T012, T024, T025, T033, T039 and T042's scene part over at the start of `/speckit-implement`. They were done with `ProjectSettings` (actions), `PackedScene.pack` + `ResourceSaver.save` (`grunt.tscn`, `main.tscn`), and direct text edits for the one-node `autoload/run.tscn`, its autoload line, the renamed flash script's path in `training_dummy.tscn`, and two `null` shader parameters the packer wrote. | The agent cannot use the editor GUI, and godot-mcp's scene tools fail on this project's untyped-declaration error. The result is ordinary `.tscn` files that open in the editor. | Building by hand in the editor stays the default; relaxing the untyped-declaration error was rejected because it would break the GDScript style rule. |

## Follow-ups (outside M3)

- **Grunts piling into one spot** with several in play (low impact, F3 only): spacing them out is a
  Later-list candidate if the five-grunt test looks messy.
- **The brute (M4)** becomes a third `FighterRef` arm and reuses `Health`, `AttackTimer`,
  `Knockback` and `resolve_attack` with its own tuning section.
- **Brief "Next step"** in `docs/project-brief.md` still points at M2a; update it when M3 is merged.
