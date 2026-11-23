# Implementation Plan: M4a Waves and the Brute

**Branch**: `feat/m4a-waves-and-brute` | **Date**: 2026-11-23 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/006-m4a-waves-and-brute/spec.md`

## Summary

Escalating waves replace M3's single respawning grunt; a slow, heavy brute joins from wave 4;
enemies take turns through attack tokens and keep apart; a score and a session best give each run
an outcome. The approach:

- **One `Enemy` class, two kinds**: M3's `Grunt` generalised with an exported `EnemyKind { Grunt,
  Brute }`; every value comes from `enemies.<kind>`, and behaviour never branches on the kind
  ([research R2](research.md)).
- **The brute's weight as two stats**: `knockback_resistance` scales the push and stun it takes;
  `interruptible = false` makes its attacks armored, so blows do damage but don't stop them
  ([R3](research.md)).
- **Turns as attack tokens**: a new `Waiting` mode; a pure `AttackTokens` pool in the director hands
  free tokens to the waiting enemy nearest its target, by each kind's cost; a turn is held exactly
  while the mode is `Approaching` to `Recovery` ([R4, R5](research.md)).
- **Waiting spots and separation**: each enemy waits on its own side at least the waiting distance
  away and drifts there; a pure O(n²) pass pushes enemies apart after they move ([R6, R7](research.md)).
- **`WaveDirector`** replaces `GruntSpawner`: `WaveState { Banner, Fighting, Breather }`, the wave
  list plus an integer escalation rule, a stream of spawns `spawn_interval` apart, an on-screen
  maximum, `spawn_now(kind)` and `skip_wave()` ([R8](research.md)).
- **Score and best in `Run`**: points per kind, `score_changed`, `game_over(kills, score, wave,
  best)`, the best kept across restarts in the autoload ([R9](research.md)).
- **Tuning** gains `enemies`, `waves` and `crowd`; `TuningValues` stays `Copy` and the wave list
  is a separate `WaveTuning` ([R10](research.md), [contracts/tuning-data.md](contracts/tuning-data.md)).
- **Presentation in GDScript**: wave banner, score label, extended Game Over, enemy body and bar
  scripts shared by both kinds ([R11](research.md)).

## Technical Context

**Language/Version**: Rust 1.100.0 for rules and state; GDScript (Godot 4.7.2) for presentation.
Unchanged.

**Primary Dependencies**: Godot **4.7.2-stable**; gdext `=0.5.5` with `api-4-6`. No new crates or
addons ([R1](research.md)).

**Storage**: `godot/data/tuning.json`, extended, read at start-up and on F5. The session best lives
in memory only.

**Testing**: `cargo test` for every pure rule ([R13](research.md)); everything else by windowed
check scripts and by playing ([quickstart.md](quickstart.md) scenarios 1–26).

**Target Platform**: Ubuntu 24.04 LTS, Intel UHD 630, stock Mesa; release export as in M2a.

**Project Type**: Desktop game: a Godot project plus a Rust GDExtension library.

**Performance Goals**: A steady 60 fps at 1280×720 with V-Sync on, with ten enemies on screen,
hitboxes shown and the panel open (apart from the intended freezes); restart in under 2 s.

**Constraints**: Compatibility renderer only; no gameplay value in code; GDScript holds no game
state; signal handlers never call back into the emitter; no behaviour branch on the enemy kind;
keyboard only; nothing from the spec's out-of-scope list.

**Scale/Scope**: 3 new Rust modules (`enemy` replacing `grunt`, `crowd`, `wave_director` replacing
`grunt_spawner`); `fighters`, `hero`, `run`, `tuning`, `tuning_store`, `lib` changed; 1 new scene
(`brute.tscn`), `grunt.tscn` and `main.tscn` changed; 2 new and 2 renamed GDScripts, 4 changed;
2 Input Map actions added; tuning grows from 40 to 71 values plus the wave list. About three
sessions, inside the constitution's two-week limit, with the game playable after each phase.

All unknowns are resolved in [research.md](research.md); none remain marked NEEDS CLARIFICATION.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.3.3. ✅ = passes.

| Rule | Pre-research | Post-design | How M4a meets it |
| --- | --- | --- | --- |
| **I. Fun First, Playable Always** | ✅ | ✅ | Waves, a second enemy and a score turn one fight into a run with a curve. Ten of the thirteen checks are player-visible, and the milestone ends only when the owner judges five runs (US6, SC-012). Tasks keep the game runnable after each phase: waves on grunts first, then turns, then the brute, then score. |
| **II. One Milestone at a Time** | ✅ | ✅ | Only the brief's M4a row: escalating waves, the brute, an attacker limit and a score. Potions, magic, jumping and saving the best stay out (FR-024). M4 was split into M4a and M4b to stay inside two weeks; M4a is sized at about three sessions. |
| **III. Architecture Serves the Game** | ✅ | ✅ | Each piece names its problem: one `Enemy` with kinds because the brute must share every grunt rule without a copy; tokens because a crowd striking at once makes hits unseeable; waiting spots and separation because enemies must not stack; the director because waves, turns and spacing all need every enemy at once; `WaveTuning` apart because a list would break `Copy` at eight call sites. Not added: a trait per kind, physics bodies, navigation, an event bus, a behaviour-tree library. |
| **IV. Data-Driven Tuning** | ✅ | ✅ | Every new value (brute stats, resistance, armor, token costs and pool, points, wave list, growth rule, interval, breather, banner, on-screen maximum, waiting distance, drift speed, separation) in `tuning.json`, checked on load, reloaded live. The depth scale is derived from the hero's speeds; colours and the banner's place are looks. |
| **V. Runs Well on Weak Hardware** | ✅ | ✅ | Placeholder polygons, the M2b flash shader, one O(n²) pass over at most 10 enemies (45 pairs) a frame; no lights or particles. Ten enemies checked on the UHD 630 (quickstart 20); the on-screen maximum caps late waves. |
| **VI. Original Art, Cheap Animation** | ✅ | ✅ | Placeholder polygons; the brute is a bigger shape in its own colour. No outside assets. |
| **VII. Co-op Ready, Local Only** | ✅ | ✅ | Enemies target the nearest living member of `team_players`; tokens are granted by distance to that target and waiting spots are measured from it; health stays per hero. No networking. |
| **VIII. Always Debuggable** | ✅ | ✅ | F7 spawns the new enemy and F8 skips a wave, arriving with what they act on; F3 now joins a wave; F2 outlines brutes too; debug builds only. R restarts in under 2 s in every build and keeps the session best. |
| **Tech: engine and API level** | ✅ | ✅ | 4.7.2, API level 4.6, unchanged. |
| **Tech: Rust/GDScript split** | ✅ | ✅ | Rust: waves, tokens, waiting, separation, enemy behaviour, the brute's reaction, score, best, restart. GDScript: banner, score label, Game Over lines, enemy body and bar, cheats' key handling. |
| **Tech: types over strings** | ✅ | ✅ | `EnemyKind`, `EnemyMode`, `WaveState`, `WaveEvent`, `RunState`, `Team`, `Damage`, `FighterRef` enums, matched exhaustively; group and action names are Rust constants. |
| **Tech: dependencies** | ✅ | ✅ | gdext only. |
| **Tech: one tuning-data format** | ✅ | ✅ | The same JSON file and loader; one reader for both enemy sections; the first boolean and array readers added to it. |
| **Code Style: Rust** | ✅ | ✅ | rustfmt at 100; `try_new`/accessor naming; `Result` for data; no new `unsafe`; `///` docs on all public items and exposed members; pure rules in their own modules with tests. |
| **Code Style: GDScript** | ✅ | ✅ | Typed, tabs, `##` docs, code order per the constitution; constants for looks; enemy scripts shared by both kinds. |
| **Code Style: shared naming** | ✅ | ✅ | `Enemy`, `EnemyKind`, `WaveDirector`, `wave_started`, `score_changed` the same in Rust, the editor and GDScript; `Run` stays the autoload of `RunTracker`. |
| **Quality Gate** | ✅ | ✅ | fmt, build and clippy with 0 warnings, `cargo test`, 0 GDScript warnings, every check passed by playing (quickstart 1–26), including the owner's five-run playtest. |

**Result**: both passes are green. The expected departure (scenes and settings changed by the
agent, if the owner hands those tasks over, as in M0–M3) is recorded below. No kind-specific
behaviour branch was needed: the brute's difference is entirely in `enemies.brute`, including its
armored attacks (`interruptible`).

## Spec and input alignment

Decided by the owner during planning; [spec.md](spec.md) was updated to match:

- **Who gets a freed turn**: the waiting enemy nearest its target (the input's rule), revising
  clarification Q4's "waited longest". FR-011, US3 sc. 3, SC-004, Key Entities and Edge Cases
  updated; a brute that doesn't fit blocks farther grunts so it cannot starve ([R5](research.md)).
- **Waiting**: a fixed spot on the enemy's own side (the input's rule), revising clarification Q5's
  circling. FR-010 and US3 updated ([R6](research.md)).
- **Brute armor**: the spec's armored attacks are kept (clarification Q1); the input's
  `knockback_resistance` is added beside them as the `interruptible` stat ([R3](research.md)).
- **F8**: starts the next wave's banner at once (spec FR-019), not a breather (the input's note).

Resolved in the plan, with the spec updated to match:

- **Attacker limit as tokens** (the input's `AttackTokens` with per-kind costs): FR-010, FR-017,
  FR-018, SC-004, Edge Cases and Assumptions now speak of a token pool and costs; starting pool 2,
  grunt 1, brute 2.
- **Banner then enemies** (the input's order): a wave's enemies start entering when its banner
  ends. FR-002 and Assumptions updated.
- **Wave reached during a breather**: the wave just cleared. US4 sc. 2 now matches the Edge Cases.

Resolved in the plan without a spec change:

- **UHD 620 → UHD 630** ([R1](research.md)).
- **On-screen maximum** (spec clarification, not in the input): `waves.max_on_screen`, starting
  at 10 ([R8](research.md)).
- **Drift speed** (spec clarification, not in the input): `crowd.drift_speed` ([R6](research.md)).
- **Best on restart**: folded in at game over (the input) and on R mid-run, so FR-014's "any run"
  holds ([R9](research.md)).
- **Banner polls** `director.banner_wave()` rather than timing itself from `wave_started`, so
  GDScript holds no copy of the banner time; the signal is still emitted ([R11](research.md)).
- **`WaveTuning` apart from `TuningValues`**, so `TuningValues` stays `Copy` ([R10](research.md)).

## Project Structure

### Documentation (this feature)

```text
specs/006-m4a-waves-and-brute/
├── spec.md              # Feature spec (/speckit-specify, /speckit-clarify, planning answers)
├── plan.md              # This file
├── research.md          # Phase 0: enemy kinds, brute weight, modes, tokens, waiting, separation,
│                        #          waves, score, tuning, presentation, cheats, tests
├── data-model.md        # Phase 1: structs, modes and transitions, formulas, rules
├── quickstart.md        # Phase 1: build, play, 26 validation scenarios
├── contracts/
│   ├── wave-classes.md  # Enemy, WaveDirector, RunTracker, TuningStore, actions, scenes, scripts
│   └── tuning-data.md   # The extended file, rules, warnings, when values apply
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
rust/src/
├── lib.rs                 # − mod grunt, grunt_spawner; + mod enemy, crowd, wave_director
├── enemy.rs               # NEW (replaces grunt.rs): Enemy, EnemyKind, EnemyMode (+ Waiting),
│                          #   holds_turn(), Reaction, enemy_after_hit(), drift_step(), grant_turn(),
│                          #   nudge(); unit tests
├── crowd.rs               # NEW (pure): AttackTokens, Body, separate(); unit tests
├── wave_director.rs       # NEW (replaces grunt_spawner.rs): WaveDirector; pure composition(),
│                          #   spawn_queue(), WaveState, WaveProgress::tick(); unit tests
├── fighters.rs            # CHANGED: FighterRef::Grunt → FighterRef::Enemy
├── hero.rs                # CHANGED: reaches enemies through FighterRef::Enemy (no rule change)
├── run.rs                 # CHANGED: score, wave, best; score_changed; game_over(kills, score,
│                          #   wave, best); set_wave(); tests
├── tuning.rs              # CHANGED: enemies.<kind>, waves, crowd; bool, array and whole-number
│                          #   readers; WaveTuning; new rules and warnings; tests
├── tuning_store.rs        # CHANGED: waves(); first warning of several
├── fighter.rs, combat.rs, movement.rs, training_dummy.rs, hit_stop.rs   # unchanged

godot/
├── project.godot          # CHANGED: actions debug_spawn_brute (F7), debug_next_wave (F8)
├── main.tscn              # CHANGED: GruntSpawner → WaveDirector (after World); Hud + ScoreLabel,
│                          #   WaveBanner; GameOver/Box + score, wave, best; overlay director export
├── world/
│   ├── grunt.tscn         # CHANGED: root type Enemy, kind Grunt, renamed scripts
│   ├── brute.tscn         # NEW: Enemy, kind Brute, bigger body, own colour
│   ├── enemy_body.gd      # RENAMED from grunt_body.gd: typed Enemy
│   ├── enemy_health_bar.gd # RENAMED from grunt_health_bar.gd: typed Enemy
│   └── screen_shake.gd    # CHANGED: Grunt → Enemy
├── ui/
│   ├── wave_banner.gd     # NEW: "Wave N" while director.banner_wave() > 0
│   ├── score_label.gd     # NEW: score from Run
│   └── game_over.gd       # CHANGED: score, wave reached, kills, best
├── debug/
│   ├── debug_overlay.gd   # CHANGED: director export; F3/F7 spawn_now(kind); F8 skip_wave()
│   └── debug_hitboxes.gd  # CHANGED: Grunt → Enemy
└── data/tuning.json       # CHANGED: extended (contracts/tuning-data.md)
```

**Structure Decision**: Same layout as M3. Pure crowd rules (tokens, separation) get their own
module so they are tested without the engine, as `fighter.rs` is; the wave rules live beside the
director that runs them, as M3's spawner rule did. The grunt's files are renamed to the class they
now hold (`enemy.rs`, `enemy_body.gd`, `enemy_health_bar.gd`); the scene keeps the name
`grunt.tscn` because it is the grunt.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
| --- | --- | --- |
| `main.tscn`, `grunt.tscn`, `brute.tscn` and the Input Map may be changed by the agent rather than by hand in the editor (Tech: "Scenes, nodes and animation are built in the Godot editor"), as in M0–M3, if the owner hands those tasks over at the start of `/speckit-implement`. | The agent cannot use the editor GUI; M3's method (`ProjectSettings` for actions, `PackedScene.pack` + `ResourceSaver.save` for scenes, small text edits for type and script paths) produces ordinary `.tscn` files that open in the editor. | Building by hand in the editor stays the default for the owner's own tasks. |

## Follow-ups (outside M4a)

- **Potions and magic (M4b)**: potion drops come from `Enemy` deaths; the magic meter belongs to the
  hero by player index; the replay test and go or no-go decision follow.
- **A third enemy kind** (later milestones) is a new `EnemyKind` arm, a tuning section and a scene,
  with no behaviour change, if its difference fits the stats; otherwise the plan for that milestone
  records the branch in Complexity Tracking.
- **Brief "Status"** in `docs/project-brief.md` and the live brief: update both when M4a is merged.
