---

description: "Task list for M3 First Fight"
---

# Tasks: M3 First Fight

**Input**: Design documents from `specs/005-m3-first-fight/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Rust unit tests are included, because the owner listed them in the plan input
([research.md R11](research.md)): killing hit reported exactly once; overkill clamps at zero;
invulnerable targets ignore hits; teams never hit themselves; a hit during wind-up cancels the
attack; approach slot maths on both sides; the spawner waits the delay; restart resets the run;
plus every tuning rule and the wind-up warning. They are written before the code they test.
Everything else is verified by windowed check scripts and by playing
([quickstart.md](quickstart.md)).

**Organization**: Tasks are grouped by user story. Phases follow priority, except that US5 (P1, the
fairness playtest) runs after US4 (P2), because judging fairness needs live tuning and the cheats.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on unfinished tasks)
- **[Story]**: Which user story the task belongs to (US1–US6 from spec.md)
- **(owner)**: Needs the owner: the Godot editor GUI, or a judgement made by playing. The owner may
  hand an editor task to the agent, which then uses a headless Godot script (`PackedScene.pack` +
  `ResourceSaver.save`) or `ProjectSettings` (`event.device = -1`), as recorded in the plan's
  Complexity Tracking.

## Path Conventions

`rust/` (the crate) and `godot/` (the project), as before. Run Rust commands from `rust/`. Values
in tests must differ from those in `godot/data/tuning.json` (see
[contracts/tuning-data.md](contracts/tuning-data.md)), so the magic-number check in T051 stays
meaningful. Size- and focus-dependent checks need a windowed 1280×720 run, not `--headless`.
Never put a script on a Rust-class root node (`Grunt`, `Hero`, `TrainingDummy`, `RunTracker`): a
GDScript `_ready` replaces the Rust `ready()` (M2b research R6). Signal handlers use their
arguments and never call back into the emitter during the emit (research R8).

## Session plan (about 15 h/week, Principle II)

- **Session 1**: Phases 1–2 (actions, tuning restructure, shared fighter parts with tests, refactor
  of hero and dummy, `Run` autoload). Ends playing exactly as M2b.
- **Session 2**: Phases 3–4 (US1 and US2: the grunt hunts, strikes, takes hits, dies, respawns).
  Ends playable: a fight you can win or lose health in.
- **Session 3**: Phases 5–9 (game over and restart, live tuning and cheats, the fairness playtest,
  60 fps, gate, commit).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm nothing was upgraded and set up the new and moved actions.

- [X] T001 Confirm nothing was upgraded (research.md R1): `godot --version` prints `4.7.2.stable.official.ed1daf0bf`; `rust/Cargo.toml` still has `godot = { version = "=0.5.5", features = ["api-4-6"] }`; `rust/rust-toolchain.toml` still pins `1.100.0`; `cargo build` and `cargo test` in `rust/` pass (41 tests). If anything differs, stop and ask the owner
- [X] T002 (owner) In Project Settings → Input Map, bound to **physical** keys, deadzone 0.2, saved to `godot/project.godot`: add `game_restart` = R, `debug_spawn_grunt` = F3, `debug_refill_health` = F4; change `debug_spawn_dummy` from F3 to **F6** ([contracts/fight-classes.md](contracts/fight-classes.md#input-map)). Confirm each event has `"device":-1` and `git diff godot/project.godot` shows nothing else. F6 now spawns the M2b dummy; F3 and F4 do nothing until T042

**Checkpoint**: The Input Map has the restart and the M3 debug actions.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The restructured tuning file, the shared fighter parts under tests, the hero and dummy
moved onto them with teams, and the `Run` autoload. Every story needs them.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete. At the end the game
plays exactly as M2b (hero, dummy, four effects, F2, F6 spawning dummies, F1, F5).

- [X] T003 [P] Write the tuning tests first in the `#[cfg(test)] mod tests` of `rust/src/tuning.rs` (they will not compile until T004). Rebuild the test `raw()` helper for the new layout with values different from the file's. One passing and one failing case per rule in [contracts/tuning-data.md](contracts/tuning-data.md#rules): `hero.max_health`, `grunt.max_health` "`> 0`"; `*.attack.damage` "`> 0`"; `*.attack.active_time`, `*.attack.reach` "`> 0`"; `grunt.horizontal_speed`, `grunt.vertical_speed` "`> 0`"; `*.hurtbox_width`, `*.hurtbox_height`, `dummy.*` "`> 0`"; `*.attack.knockback_time` "`>= 0`, and `> 0` when that attack's `knockback_distance > 0`" (for both `hero` and `grunt`); every other field "`>= 0`" (pass 0, fail −1) and finite (fail NaN); M1's speed and lane rules still hold. Each rejection's `field()` is the full path (`grunt.attack.knockback_time`); a missing nested object reports its path (`hero.attack`). Warning tests: `warnings()` returns `GruntWindUpNotLonger { hero, grunt }` when `grunt.attack.wind_up_time <= hero.attack.wind_up_time` (equal and smaller) and nothing when longer; such values still pass `validate`
- [X] T004 In `rust/src/tuning.rs`, restructure per [contracts/tuning-data.md](contracts/tuning-data.md) and [data-model.md](data-model.md#tuning-restructured): section and key constants for `hero`, `grunt`, `dummy`, `lane`, `effects`, `run` and the nested `attack` object; generalise the readers so a nested object is read with the dotted path (`hero.attack.reach`); one function reads an `attack` object into `fighter`-independent raw values so the hero's and the grunt's cannot drift; `TuningValues` exposes `hero()`, `grunt()` (`FighterTuning`), `dummy()`, `lane()` (with `depth_tolerance`), `effects()`, `run()`; `validate` applies the rules in file order with no defaults; add `pub enum TuningWarning { GruntWindUpNotLonger { hero: f32, grunt: f32 } }` with `Display` giving the contract's message, and `pub fn warnings(&TuningValues) -> Vec<TuningWarning>`. Rewrite `godot/data/tuning.json` exactly as the contract's JSON. Run `cargo test` until T003 passes (callers fixed in T009) (depends on T003)
- [X] T005 [P] Write the fighter tests first in a new `rust/src/fighter.rs` `#[cfg(test)] mod tests` (they will not compile until T006), covering [data-model.md](data-model.md#shared-parts-rustsrcfighterrs-pure-unit-tested): `Health::damage` returns `Survived`, then `Killed` exactly once, then `AlreadyDead`; overkill clamps `current` to 0; `refill` restores `max`; `set_max` lowers `current` to a smaller max and never raises it. `AttackTimer`: M2b's `advance_swing` tests moved over (WindUp → Active → Recovery on time, zero-length parts in one tick, `Finished` after recovery, `was_active` true for an active part shorter than a tick). `Knockback` offset 0 at start, full distance at and after its time. `Countdown::tick` true exactly on the tick it reaches 0. `resolve_attack`: a target of the same team is never hit; an `invulnerable` or not `alive` target is ignored; an already-hit target is skipped; two targets in reach are both hit; a target beyond `depth_tolerance` is missed; each `Hit` carries the attacker's damage, `facing.sign()` and knockback. `approach_step` on both sides: depth difference above tolerance moves only vertically; inside it, moves to the slot at `target.x ± (target_half_width + reach + own_half_width)` without overshooting; `side` is +1 when x is equal
- [X] T006 Implement `rust/src/fighter.rs` so T005 passes, with no Godot types except `InstanceId`-agnostic generics: `Health`, `Damage { Survived, Killed, AlreadyDead }`, `AttackSpec`, `KnockbackSpec`, `AttackPhase { WindUp, Active, Recovery }`, `AttackTimer` with `advance`, `AttackStep { Continue, Finished }`, `Knockback`, `Countdown`, `Team { Players, Enemies }` with `other()` and `group()` (constants `"team_players"`, `"team_enemies"`), `Attacker`, `Target<T>`, `Hit<T>`, `resolve_attack`, `approach_step`. Build `AttackSpec` from the tuning reader of T004. Move `SwingPhase`, `SwingTimings`, `SwingStep`, `advance_swing` and their tests out of `rust/src/combat.rs`; keep `Facing`, `Rect`, boxes, `lands`, `new_hits`, `knockback_offset`, `clamp_to_area`, `extend_freeze`, `spawn_point`. `//!` module doc, `///` on every public item, exhaustive `match`es only. Add `mod fighter;` to `rust/src/lib.rs`; `cargo test` (depends on T004, T005)
- [X] T007 [P] In `rust/src/hit_stop.rs`, add `#[func] clear()` that ends any freeze at once, with `///` docs (contracts/fight-classes.md)
- [X] T008 [P] In `rust/src/tuning_store.rs`, read the M2b accessors from `effects()` (`flash_time()`, `shake_strength()`, `shake_time()`), add `#[func] death_fade_time()` (`0.0` without values), and keep the warnings of the last successful load or reload: print each with `godot_warn!`, expose `#[func] warning() -> GString` (the first warning's message, or `""`), all with `///` docs (depends on T004)
- [X] T009 Move the hero and the dummy onto the new parts with no change in behaviour (research R2–R3): in `rust/src/hero.rs`, `Swing { timer: AttackTimer, queued: bool, hit_this_swing: Vec<InstanceId> }` (a queued press restarts on `Finished`), attack values from `values.hero().attack`, depth tolerance from `values.lane()`; join `Team::Players.group()` in `ready`; land blows with `fighter::resolve_attack` over the `Enemies` group, reached through a new `rust/src/fighters.rs` `enum FighterRef { Hero(Gd<Hero>), Dummy(Gd<TrainingDummy>) }` with `target()` and `take_hit(&Hit<InstanceId>)` (the `Grunt` arm comes in T029); `dummy_spawn_point()` finds dummies in the `Enemies` group. In `rust/src/training_dummy.rs`, join `Team::Enemies.group()`, use `fighter::Knockback`, and `take_hit(&Hit)` takes the knockback from the hit (the attacker's), emitting `was_hit(direction, distance)`. Remove `HITTABLE_GROUP` from `rust/src/combat.rs`. Add `mod fighters;` to `rust/src/lib.rs`; `cargo build`, `cargo test` (depends on T006, T008)
- [X] T010 [P] Write the run tests first in a new `rust/src/run.rs` `#[cfg(test)] mod tests` (data-model.md "Run"): `RunRecord::add_kill` counts while `Playing` and not after `GameOver`; `end_run` returns true once, then false; `reset` gives `Playing` with 0 kills
- [X] T011 Implement `rust/src/run.rs` so T010 passes (research R7, [contracts/fight-classes.md](contracts/fight-classes.md#runtracker-new-autoload-run)): pure `RunRecord { state: RunState, kills }`, `RunState { Playing, GameOver }`; `#[class(base = Node)] pub struct RunTracker` wrapping it, with `#[signal] game_over(kills: i64)`, `#[func] kills()`, `#[func] is_playing()`, and Rust-side `add_kill()` and `end_run()` (emits `game_over` only when `end_run` returns true). `restart()` comes in T034. Docs say why the autoload name differs from the class name. Add `mod run;` to `rust/src/lib.rs`; `cargo test` (depends on T010)
- [X] T012 (owner) Create `godot/autoload/run.tscn` with one root node of type `RunTracker` named `Run`, and add it in Project Settings → Autoload as `Run` (Global Variable on) **after** `HitStop` (depends on T011)
- [X] T013 Rename `godot/world/dummy_flash.gd` to `godot/world/hit_flash.gd` (with its `.uid`), keep its behaviour (flash on the parent's `was_hit`, the parent reached as a `Node2D` with `was_hit` connected through `Signal(get_parent(), &"was_hit")`), and update the script path in `godot/world/training_dummy.tscn` (research R8)
- [X] T014 Confirm with windowed Godot script runs (1280×720) that the game plays exactly as M2b: `/root/Tuning`, `/root/HitStop`, `/root/Run` in that order; tuning loads with the new file and no warning; M2b scenario checks (hits with all four effects, misses by depth, one hit per swing, two dummies, edge clamp) pass against dummies spawned with F6; F2 and F1 work; F5 rejects `grunt.max_health` 0 naming `grunt.max_health`. Then `cargo fmt --check`, `cargo clippy --all-targets` (0 warnings except dead code used in Phase 3) and `cargo test` (depends on T007, T009, T012, T013)

**Checkpoint**: Shared parts tested, hero and dummy rebuilt on them, `Run` registered; the game
still plays as M2b.

---

## Phase 3: User Story 1 – A grunt hunts the hero and strikes (Priority: P1) 🎯 MVP

**Goal**: A grunt enters, lines up, closes in, winds up visibly and strikes; the hero loses health,
is stunned and blinks.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 1–4.

- [X] T015 [US1] In `rust/src/hero.rs` (research R4, [data-model.md](data-model.md#hero-extended)): add `health: Health` (max from `values.hero().max_health` each tick via `set_max`), `invulnerable: Countdown`, `knockback: Option<Knockback>`; extend `HeroMode` with `HitStun(Countdown)` and `Dead`, every `match` exhaustive; `#[signal] was_hit(direction: f32, strength: f32)`, `#[signal] died()`; Rust-side `take_hit(&Hit)` per the data-model table (only while not `Dead` and not invulnerable: `Survived` → `HitStun(hero.hit_stun_time)`, swing cancelled, invulnerability `hero.invulnerability_time`, knockback; `Killed` → `Dead`, knockback, `died()`, `Run.end_run()`); knockback advances and is clamped in every mode; nothing advances while frozen; input ignored unless `Free`/`Attacking` and `Run.is_playing()`. `#[func]`s `health()`, `max_health()`, `is_invulnerable()`, `is_dead()`, `hurtbox()` (from `hero.hurtbox_width`/`height`). `FighterRef::Hero` `target()` reports `alive = !is_dead`, `invulnerable`. Setup finds `/root/Run` (error `Hero: the Run autoload is missing or is not a RunTracker`). `///` docs (depends on T014)
- [X] T016 [US1] Create `rust/src/grunt.rs` (research R5, [data-model.md](data-model.md#grunt-new), contracts/fight-classes.md): `#[class(base = Node2D)] pub struct Grunt` with `mode: GruntMode { Entering, Approaching, WindUp(Attack), Active(Attack), Recovery(Attack), HitStun(Countdown), Dying(Countdown) }`, `Attack { timer: AttackTimer, hit_this_attack: bool }`, `health`, `facing`, `knockback`, `edge`; handles to `Tuning`, `HitStop`, `Run` found in `ready` (missing ⇒ one `godot_error!("Grunt: …")`, never moves). `physics_process`: nothing while frozen; outside `Run.is_playing()` only `Dying` advances. `Entering`: walk in from `edge` at `grunt.horizontal_speed` until fully on screen, then `Approaching`. `Approaching`: target = nearest living `Players` member (via `FighterRef`); face it; `fighter::approach_step`; start `WindUp` (attack from `values.grunt().attack`) when `resolve_attack` with the grunt's attacker would land on it now. `WindUp`/`Active`/`Recovery`: no movement or turning; during `Active`, `resolve_attack` against `Players` at most once per attack, the hit applied through `FighterRef::take_hit`, one `HitStop.freeze(effects.hit_stop_time)` if > 0; recovery ends → `Approaching`. Clamp to lane and screen once entered. `#[func]`s `is_winding_up()`, `is_alive()`, `health()`, `max_health()`, `hurtbox()` (empty while `Entering`/`Dying`), `attack_box()` (only while `Active`). `#[signal] was_hit`, `#[signal] died` declared now. The grunt does **not** join the `Enemies` group yet (T029). Rust-side `pub fn place(edge, depth)` for the spawner. Add `mod grunt;` to `rust/src/lib.rs` (depends on T015)
- [X] T017 [P] [US1] Write the spawner tests first in a new `rust/src/grunt_spawner.rs` `#[cfg(test)] mod tests` (data-model.md "GruntSpawner"): `AwaitingFirst` spawns once tuning is valid and playing; `Watching` with a living grunt stays; with none starts `Waiting(respawn_delay)`; `Waiting` does not spawn before the delay and spawns on the tick it ends; a grunt appearing during `Waiting` returns to `Watching` without spawning; nothing happens while not playing or without tuning
- [X] T018 [US1] Implement `rust/src/grunt_spawner.rs` so T017 passes (research R6, contracts/fight-classes.md): pure `SpawnerState { AwaitingFirst, Watching, Waiting(Countdown) }` with `tick(state, living_grunts, tuning_ready, playing, delta) -> (SpawnerState, bool)`; `#[class(base = Node)] pub struct GruntSpawner` with `#[export] grunt_scene: Option<Gd<PackedScene>>`, `#[export] world: Option<Gd<Node2D>>`, a `RandomNumberGenerator` `randomize()`d in `ready`; each physics tick (not while frozen) count living grunts (`Grunt` children of `world` with `is_alive()`), call `tick`, and spawn: instance the scene, `place(edge = randi_range(0, 1), depth = randf_range(lane.top, lane.bottom))`, add to `world`, then `move_child` it before the hero (the last child) so the hero wins depth ties. `#[func] spawn_now()` spawns the same way. Missing exports ⇒ one `godot_error!("GruntSpawner: …")`. Add `mod grunt_spawner;` to `rust/src/lib.rs`; `cargo test` (depends on T016, T017)
- [X] T019 [P] [US1] Create `godot/world/grunt_body.gd` (`extends "res://world/hit_flash.gd"`, `##` docs): a `WIND_UP_COLOR` constant and the body's starting colour remembered in `_ready`; in `_process`, `color` is `WIND_UP_COLOR` while `(get_parent() as Grunt).is_winding_up()`, else the starting colour (research R8)
- [X] T020 [P] [US1] Create `godot/world/hero_body.gd` (`extends "res://world/hit_flash.gd"`, `##` docs): a `BLINK_SECONDS` constant; in `_process`, while `(get_parent() as Hero).is_invulnerable()`, toggle `visible` every `BLINK_SECONDS`, else `visible = true` (research R8)
- [X] T021 [P] [US1] In `godot/world/screen_shake.gd`, connect `was_hit` of every node entering the tree that `is Hero`, `is Grunt` or `is TrainingDummy` (research R8)
- [X] T022 [P] [US1] Create `godot/ui/health_bar.gd` (`extends Control`, `##` docs): `## The hero whose health is shown.` `@export var hero: Hero`; size and colour constants; `_draw` draws a background rectangle and a filled part `hero.health() / hero.max_health()` wide (0 when max is 0); `_process` calls `queue_redraw()`. No game state
- [X] T023 [P] [US1] In `godot/debug/debug_hitboxes.gd`, also outline `hero.hurtbox()`, and for every `Grunt` child of `world` its `hurtbox()` and, when it has area, its `attack_box()` (FR-020); colour constants for each
- [X] T024 [US1] (owner) Create `godot/world/grunt.tscn`: root `Grunt` named `Grunt` (**no script**); child `Polygon2D` `Body` with polygon `(-26, -100), (26, -100), (26, 0), (-26, 0)` in a colour unlike the hero's, the dummy's and the pillars' (for example `#7A3B8F`), a `ShaderMaterial` using `res://world/flash.gdshader` with **Local to Scene** on, and script `res://world/grunt_body.gd`. Save (depends on T016, T019)
- [X] T025 [US1] (owner) In `godot/main.tscn` (contracts/fight-classes.md "Scenes"): delete `World/Dummy` (clarification Q3); add `Node` `GruntSpawner` (type `GruntSpawner`) with `grunt_scene` → `res://world/grunt.tscn` and `world` → `World`; give `World/Hero/Body` a `ShaderMaterial` with `flash.gdshader` (Local to Scene) and script `res://world/hero_body.gd`; add `Control` `HealthBar` to `Hud` at (16, 16) with script `res://ui/health_bar.gd`, `hero` → `World/Hero`; move `DebugOverlay/Panel` down so it starts below the health bar and the kills label to come (for example `offset_top` 64). Save (depends on T018, T020, T021, T022, T024)
- [X] T026 [US1] Confirm by windowed script runs: at start no dummy, one grunt enters from an edge at a lane depth and is not hittable or targetable while entering; it reaches the hero's depth (within `lane.depth_tolerance`) before its x approaches; it stops at the slot and starts no attack while out of reach or depth; its wind-up lasts longer than the hero's (in ticks) and `is_winding_up()` is true throughout; stepping out of depth during the wind-up makes the strike miss (health unchanged); a landed strike lowers health by `grunt.attack.damage`, emits `was_hit`, knocks the hero away from the grunt, freezes once, and makes `is_invulnerable()` true for about `hero.invulnerability_time` during which strikes do nothing; a strike during the hero's swing cancels it and the hero cannot move for `hero.hit_stun_time`. No errors in the Output (depends on T023, T025)
- [X] T027 [US1] (owner) Press Play and run quickstart scenarios 1–4. Fix anything that fails before going on (depends on T026)

**Checkpoint**: The grunt hunts and strikes fairly; the hero can be hurt. M3's MVP is playable.

---

## Phase 4: User Story 2 – Beat the grunt, and another comes (Priority: P1)

**Goal**: The hero's blows hurt, stun and kill grunts; a dead grunt fades, counts, and a new one
comes when none is left.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 5–7.

- [X] T028 [P] [US2] Write tests first in `rust/src/grunt.rs` `#[cfg(test)] mod tests` for the pure `grunt_after_hit(mode, damage: Damage, hit_stun_time, death_fade_time) -> GruntMode` (data-model.md): `Survived` in `Approaching`, `WindUp`, `Active`, `Recovery` or `HitStun` → `HitStun(hit_stun_time)` (the attack dropped); `Killed` → `Dying(death_fade_time)`; `Entering` and `Dying` are unchanged
- [X] T029 [US2] In `rust/src/grunt.rs` (research R5): implement `grunt_after_hit` so T028 passes; join `Team::Enemies.group()` when `Entering` ends and leave it when `Dying` starts; Rust-side `take_hit(&Hit)`: damage `health` (max from `grunt.max_health` each tick), apply `grunt_after_hit`, start the knockback from the hit, emit `was_hit`; on `Killed` emit `died()` once and call `Run.add_kill()`; `HitStun` ends → `Approaching`; `Dying` ends → `queue_free()`. Add the `Grunt(Gd<Grunt>)` arm to `FighterRef` in `rust/src/fighters.rs` (`alive = is_alive()`, never invulnerable) so the hero's blows reach grunts and the hit counter counts them. `cargo test`, `cargo clippy --all-targets` (depends on T026, T028)
- [X] T030 [P] [US2] Create `godot/world/grunt_health_bar.gd` (`extends Node2D`, `##` docs): size, offset-above-head and colour constants; `_draw` draws a background and a filled part from the parent `Grunt`'s `health() / max_health()`; `_process` calls `queue_redraw()`; hidden once the grunt is not `is_alive()` (research R8)
- [X] T031 [P] [US2] In `godot/world/grunt_body.gd`, on the parent's `died` signal tween `modulate:a` from 1 to 0 over `Tuning.death_fade_time()` (the flash still plays at the start); no call back into the grunt inside the handler
- [X] T032 [P] [US2] Create `godot/ui/kills_label.gd` (`extends Label`, `##` docs): in `_process`, `text = "Kills: %d" % Run.kills()`
- [X] T033 [US2] (owner) Add a `Node2D` `HealthBar` child to the `Grunt` root in `godot/world/grunt.tscn` with script `res://world/grunt_health_bar.gd`; add a `Label` `KillsLabel` to `Hud` in `godot/main.tscn` under the health bar (for example at (16, 36)) with script `res://ui/kills_label.gd`. Save (depends on T030, T032)
- [X] T034 [US2] Confirm by windowed script runs: a hero blow on a grunt lowers its health by `hero.attack.damage`, flashes it, freezes, shakes, knocks it back and stuns it for `grunt.hit_stun_time`; a blow during its wind-up cancels the strike (no hit follows); after the stun it lines up and approaches again; three blows kill it: `died` once, `Run.kills()` +1, it leaves `team_enemies` at once (further blows do nothing), fades over `run.death_fade_time` and is freed; a new grunt enters `run.respawn_delay` after the last one died; with three grunts (spawned through `GruntSpawner.spawn_now()`), killing one brings no replacement until none is left; the dummy from F6 is ignored by grunts and never counts as a kill (depends on T029, T031, T033)
- [X] T035 [US2] (owner) Press Play and run quickstart scenarios 5–6. Scenario 7 needs F3 for extra grunts, so it is played in T045 (depends on T034)

**Checkpoint**: A fight you can win, grunt after grunt.

---

## Phase 5: User Story 3 – Game over and a fast restart (Priority: P1)

**Goal**: The hero falls, Game Over shows the kills, and R starts a fresh run in under 2 s.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 8–9.

- [X] T036 [US3] In `rust/src/run.rs` (research R7): the `game_restart` action name as a constant; `process` (every build, not affected by `HitStop`) calls `restart()` when `Input.is_action_just_pressed(game_restart)`; `#[func] restart()`: `RunRecord::reset()`, `HitStop.clear()` (found at `/root/HitStop`), then `get_tree().reload_current_scene()`; a missing action or autoload ⇒ one `godot_error!("Run: …")`. `cargo test`, `cargo clippy --all-targets` (depends on T029)
- [X] T037 [P] [US3] In `godot/world/hero_body.gd`, on the parent's `died` signal tween `rotation` to a `FALL_ANGLE` constant (tipping over) and stop blinking; no call back into the hero inside the handler
- [X] T038 [P] [US3] Create `godot/ui/game_over.gd` (`extends CanvasLayer`, `##` docs): hidden in `_ready`; connects `Run.game_over`; the handler shows the layer and sets its labels to "Game Over", `"Kills: %d" % kills` (from the argument) and "Press R to restart"; it calls nothing on `Run` during the emit
- [X] T039 [US3] (owner) In `godot/main.tscn`, add a `CanvasLayer` `GameOver` (layer above `Hud`, below `DebugOverlay`) with script `res://ui/game_over.gd` and a centred `VBoxContainer` holding three `Label`s (`Title`, `Kills`, `Hint`). Save (depends on T037, T038)
- [X] T040 [US3] Confirm by windowed script runs: at zero health the hero emits `died` once, tips over, `Run.is_playing()` is false, every grunt stops (a fading one finishes), no new grunt spawns, and Game Over shows the same kill count as `Run.kills()` had; walk and attack keys do nothing; pressing R at Game Over and mid-run (with extra grunts, F6 dummies, a fading grunt, during a freeze) gives within 2 s (measured from the press to the new scene's first frame): full health, kills 0, hit counter 0, no leftover grunts or dummies, one grunt entering; F5-tuned values survive the restart (depends on T036, T039)
- [X] T041 [US3] (owner) Press Play and run quickstart scenarios 8–9 on the UHD 630, timing R (depends on T040)

**Checkpoint**: A whole run, lost and restarted. The loop is complete.

---

## Phase 6: User Story 4 – Tune the fight live, and cheat (Priority: P2)

**Goal**: Every fight value applies with F5; F3, F4, F6 work in debug builds only; the wind-up
warning shows.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 7 and 10–14.

- [X] T042 [US4] In `godot/debug/debug_overlay.gd`: `## The spawner F3 asks for a grunt.` `@export var spawner: GruntSpawner`; in `_unhandled_input`, `debug_spawn_grunt` calls `spawner.spawn_now()`, `debug_refill_health` calls `hero.refill_health()` (add `#[func] refill_health()` to `rust/src/hero.rs`: health to max, nothing while `Dead`), both marking the event handled; `debug_spawn_dummy` (now F6) unchanged. After a reload, and at start, if `Tuning.warning()` is not empty, the message spot shows `"Tuning reloaded (warning: %s)"` and stays until the next reload. Then (owner) set the overlay's `spawner` → `GruntSpawner` in `godot/main.tscn` (depends on T040)
- [X] T043 [US4] Confirm by windowed script runs: each new value in [contracts/tuning-data.md](contracts/tuning-data.md) changed and reloaded with F5 takes effect at the moment the contract gives (next tick, next attack, next hit, `max_health` at once with current lowered, `respawn_delay` and `death_fade_time` the next time); `grunt.max_health` 0, `grunt.attack.knockback_time` 0 with distance 80, and `run.respawn_delay` −1 are each rejected naming the field; `grunt.attack.wind_up_time` 0.05 is accepted with the warning, and restoring clears it; F3 adds a grunt from an edge, F4 refills (not at Game Over), F6 adds a dummy, F2 outlines everything. Restore `godot/data/tuning.json` afterwards (depends on T042)
- [X] T044 [US4] Build and export a release (`cd rust && cargo build --release`, then `godot --headless --path godot --export-release Linux export/beastrider.x86_64`); confirm `godot/export/` holds the binary, `.pck` and `libbeastrider.so` and that a 5-second run prints no errors (depends on T042)
- [ ] T045 [US4] (owner) Quickstart scenarios 7 and 10–14: no replacement while a grunt lives, retune live, the bad values, the warning, the cheats, and the release build (fight, bars, kills, Game Over and R work; F1–F6 do nothing) (depends on T043, T044)

**Checkpoint**: The fight can be tuned live; cheats exist in debug builds only.

---

## Phase 7: User Story 5 – The fight is fair (Priority: P1)

**Goal**: The owner's judgement that every hit taken could have been seen coming.

**Independent Test**: [quickstart.md](quickstart.md) scenario 16.

- [ ] T046 [US5] (owner) Play and retune `godot/data/tuning.json` with F5 until the fight feels fair, using F2, F3, F4 and F6 as needed. Then play five runs to Game Over with the final values and judge: could every hit taken have been seen coming (SC-012)? If not, keep retuning; the milestone is not done until the answer is yes (depends on T045)
- [ ] T047 [US5] Record the final values: confirm `godot/data/tuning.json` holds them, passes the rules and gives no warning; update the starting values in `specs/005-m3-first-fight/contracts/tuning-data.md` to match, so the contract shows what shipped (depends on T046)

**Checkpoint**: The owner says the fight is fair.

---

## Phase 8: User Story 6 – Still 60 fps, clean and committed (Priority: P2)

**Goal**: 60 fps with five grunts; the Quality Gate is green.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 15, 18, 19 and 21.

- [ ] T048 [US6] (owner) Quickstart scenario 15: F3 to five grunts, F2 on, panel open; fight for 60 s by the clock, F4 to stay alive and F3 to keep five, window focused. FPS stays 60 apart from the first second and the impact freezes (depends on T045)
- [X] T049 [US6] In `rust/`, run `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets` and `cargo test`; fix every warning in `rust/src/*.rs` until all pass with 0 warnings. No `#[allow]` without a written reason
- [X] T050 [US6] Check every GDScript file (`godot/world/hit_flash.gd`, `hero_body.gd`, `grunt_body.gd`, `grunt_health_bar.gd`, `screen_shake.gd`, `lane_band.gd`, `godot/ui/fps_label.gd`, `health_bar.gd`, `kills_label.gd`, `game_over.gd`, `godot/debug/debug_overlay.gd`, `debug_markers.gd`, `debug_hitboxes.gd`) through the editor's language server, as in M2b's T031: 0 errors or warnings, `untyped_declaration` still Error

**Checkpoint**: The Quality Gate is green.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Constitution review, docs and the milestone commit.

- [X] T051 [P] Check for magic numbers (Principle IV): none of the values in `godot/data/tuning.json` appears as a literal in `rust/src` (outside tests, whose values must differ) or in `godot/world`, `godot/ui` and `godot/debug` scripts; the only numbers in scripts are named presentation constants
- [X] T052 [P] Check the mode rules (FR-021): `grep -rnE 'is_attacking: bool|dead: bool|stunned: bool|game_over: bool' rust/src` returns nothing; every `match` on `HeroMode`, `GruntMode`, `AttackPhase`, `SpawnerState`, `RunState`, `Team`, `Damage` and `FighterRef` has no `_ =>` arm
- [X] T053 [P] Check the split (FR-021) and co-op (FR-014): no `.gd` file decides a hit, damage, death, a spawn, a kill or the run state (they only read Rust and react to `was_hit`, `died`, `game_over`); grunts target through `team_players`, never a fixed node path; `grep -rnE 'KEY_|keycode' godot/world godot/ui godot/debug` returns nothing; `grep -rn '"p1_' rust/src` returns nothing
- [X] T054 [P] Review docs: `//!` module docs on `rust/src/fighter.rs`, `fighters.rs`, `grunt.rs`, `grunt_spawner.rs`, `run.rs`; `///` on every public item and everything exposed to Godot, including the new signals; `// SAFETY:` in `rust/src/lib.rs` unchanged; `##` docs on every new script and exported property
- [X] T055 Run the Constitution compliance review required by Governance: check the finished work against Principles I–VIII (v1.3.3) and the Code Style section, using the table in `specs/005-m3-first-fight/plan.md` as the checklist; record any departure (for example scenes built by the agent) in that file's Complexity Tracking
- [X] T056 After a build, a Play and the export, run `git status`: `rust/target/`, `godot/.godot/` and `godot/export/` absent; the new and changed Rust modules, `godot/autoload/run.tscn`, `godot/world/grunt.tscn`, the new and renamed `.gd` files with their `.uid` files, `godot/main.tscn`, `godot/world/training_dummy.tscn`, `godot/project.godot` and `godot/data/tuning.json` present to be committed
- [ ] T057 (owner) Run the full quickstart.md, scenarios 1–21, in one sitting on the target machine; scenario 20 re-checks M2b and M2a
- [ ] T058 Commit the M3 implementation on branch `feat/m3-first-fight` in small logical commits that each pass the gate on their own staged tree (for example: input actions; tuning restructure with the shared fighter parts and the refactored hero and dummy; `Run` autoload; the grunt, spawner and their scenes; hits on grunts, death and kills; game over and restart; cheats and warning; final values and docs), staging by explicit path and reviewing `git diff --staged` before each, as the owner's git rules require. Ask the owner before committing; then `git status` must be clean (depends on T046–T057)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies. T002 is needed before T036 (R) and T042 (F3, F4) run.
- **Foundational (Phase 2)**: Needs T001. Blocks every story. Ends playing as M2b.
- **US1 (Phase 3)**: Needs Phase 2. The MVP: something fights back.
- **US2 (Phase 4)**: Needs US1 (there must be a grunt to hit).
- **US3 (Phase 5)**: Needs US1 (the hero can die) and US2's `FighterRef::Grunt` (T029).
- **US4 (Phase 6)**: Needs US3 (F4 at Game Over, values surviving restart).
- **US5 (Phase 7)**: Needs US4 (live tuning and cheats to judge with).
- **US6 (Phase 8)**: T048 needs US4 (F3, F4); T049–T050 need all code.
- **Polish (Phase 9)**: Needs all stories.

### Within Each Phase

- Tests before the code they test (T003 → T004, T005 → T006, T010 → T011, T017 → T018, T028 → T029).
- Rust classes registered (T011, T016, T018) before scenes use them (T012, T024, T025).
- Scripts before the scenes that use them (T019–T022 → T024–T025; T030, T032 → T033;
  T037–T038 → T039).
- Windowed checks, then the owner's play checks, last in each story.

### Parallel Opportunities

- **Phase 2**: T003 (tuning tests), T005 (fighter tests) and T010 (run tests) together; T007 and
  T008 alongside T006.
- **US1**: T017 (spawner tests) alongside T016; the presentation files T019–T023 together.
- **US2**: T028 first; T030, T031, T032 together.
- **US3**: T037 and T038 together.
- **Polish**: T051–T054 in parallel.

---

## Parallel Example: Phase 2 and User Story 1

```bash
# Tests first, together (different files):
Task: "Write the tuning tests in rust/src/tuning.rs"
Task: "Write the fighter tests in rust/src/fighter.rs"
Task: "Write the run tests in rust/src/run.rs"

# Presentation files for US1, together:
Task: "Create godot/world/grunt_body.gd"
Task: "Create godot/world/hero_body.gd"
Task: "Extend godot/world/screen_shake.gd"
Task: "Create godot/ui/health_bar.gd"
Task: "Extend godot/debug/debug_hitboxes.gd"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: actions. 2. Phase 2: tuning, shared parts, refactor, `Run` (game still plays as M2b).
3. Phase 3: US1. A grunt hunts, telegraphs and strikes; the hero loses health.
4. **STOP and VALIDATE**: quickstart 1–4. A first feel for fairness can be judged here.

### Incremental Delivery

1. Setup + Foundational → tests pass, M2b unchanged (session 1)
2. US1 → something fights back → MVP
3. US2 → grunts can be beaten, and more come (session 2)
4. US3 → game over and restart: the full loop
5. US4 → live tuning of the fight, cheats
6. US5 → fair, by the owner's judgement
7. US6 + Polish → 60 fps, gate green, committed (session 3) → M3 done

### If a Session Stalls

Shrink the next step (constitution, Development Workflow): for example, get the grunt walking
straight at the hero before lining up by depth, or let it strike with no wind-up colour, then add
each piece behind its tuning value. Never put a tuning value in code as a shortcut. Every
checkpoint is a safe place to stop.

---

## Notes

- [P] = different files, no dependency on unfinished tasks.
- (owner) tasks need the Godot editor GUI or a judgement made by playing.
- Commit only at the end, with the owner's go-ahead (T058). Rust parts that are unused until a
  later phase would fail clippy's dead-code check on their own, so commits group them with their
  first user, as in M2b.
- Godot creates a `*.uid` file next to each new script. Commit it; it is not cache.
- After the merge, update the brief's "Next step" (`docs/project-brief.md`) to M4.
