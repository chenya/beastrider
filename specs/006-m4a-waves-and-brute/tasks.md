---

description: "Task list for M4a Waves and the Brute"
---

# Tasks: M4a Waves and the Brute

**Input**: Design documents from `specs/006-m4a-waves-and-brute/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Rust unit tests are included, because the owner listed them in the plan input
([research.md R13](research.md)): composition from the list and from the escalation rule; next wave
only after clear plus breather; tokens never exceed capacity and are released on recovery,
hit-stun and death; the nearest waiting enemy gets the free token; separation pushes overlapping
enemies apart and never out of the lane; `knockback_resistance` scaling; points per kind; session
best survives restart; `skip_wave` awards no points; plus every new tuning rule and warning. They
are written before the code they test. Everything else is verified by windowed check scripts and
by playing ([quickstart.md](quickstart.md)).

**Organization**: Tasks are grouped by user story. Phases follow the spec's order, except that US6
(P1, the five-run playtest) runs after US5 (P2), because judging the curve needs live tuning and
the cheats. The cheat each story's test needs arrives with that story: F3 and F8 with US1, F7 with
US2.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on unfinished tasks)
- **[Story]**: Which user story the task belongs to (US1–US7 from spec.md)
- **(owner)**: Needs the owner: the Godot editor GUI, or a judgement made by playing. The owner may
  hand an editor task to the agent, which then uses a headless Godot script (`PackedScene.pack` +
  `ResourceSaver.save`) or `ProjectSettings` (`event.device = -1`), as recorded in the plan's
  Complexity Tracking.

## Path Conventions

`rust/` (the crate) and `godot/` (the project), as before. Run Rust commands from `rust/`. Values
in tests must differ from those in `godot/data/tuning.json` (see
[contracts/tuning-data.md](contracts/tuning-data.md)), so the magic-number check in T059 stays
meaningful. Size- and focus-dependent checks need a windowed 1280×720 run, not `--headless`
(a headless run reports a 64 px screen and rejects `lane.bottom`). Never put a script on a
Rust-class root node (`Enemy`, `Hero`, `TrainingDummy`, `RunTracker`, `WaveDirector`): a GDScript
`_ready` replaces the Rust `ready()` (M2b research R6). Signal handlers use their arguments and
never call back into the emitter during the emit (M3 research R8). Behaviour code never matches on
`EnemyKind` (research R2); the only two matches are in `TuningValues::enemy` and the director's
scene choice.

## Session plan (about 15 h/week, Principle II)

- **Session 1**: Phases 1–3 (actions, tuning extension with tests, `Grunt` → `Enemy` with kinds,
  the brute scene, then US1 waves). Ends playable: waves of grunts and brutes, every enemy
  attacking at once.
- **Session 2**: Phases 4–6 (US2 the brute's weight, US3 turns and spacing, US4 score and best).
  Ends with the full M4a loop.
- **Session 3**: Phases 7–10 (live tuning and cheats, the five-run playtest, 60 fps, gate,
  commit). Mostly playing and retuning.

Session 1 is the heaviest (14 tasks). If it runs long, shrink the next step as the constitution's
Development Workflow says (for example, spawn a wave all at once before adding the stream).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm nothing was upgraded and add the two new actions.

- [X] T001 Confirm nothing was upgraded (research.md R1): `godot --version` prints `4.7.2.stable.official.ed1daf0bf`; `rust/Cargo.toml` still has `godot = { version = "=0.5.5", features = ["api-4-6"] }`; `rust/rust-toolchain.toml` still pins `1.100.0`; `cargo build` and `cargo test` in `rust/` pass (59 tests). If anything differs, stop and ask the owner
- [X] T002 (owner) In Project Settings → Input Map, bound to **physical** keys, deadzone 0.2, saved to `godot/project.godot`: add `debug_spawn_brute` = F7 and `debug_next_wave` = F8 ([contracts/wave-classes.md](contracts/wave-classes.md#input-map)). Confirm each event has `"device":-1` and `git diff godot/project.godot` shows nothing else. F8 does nothing until T013 and F7 until T019

**Checkpoint**: The Input Map has the M4a debug actions.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The extended tuning file under tests, and M3's `Grunt` generalised into `Enemy` with
two kinds and a brute scene. Every story needs them.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete. At the end the game
plays exactly as M3 (one grunt at a time, respawning; F3, F4, F6, F1, F2, F5; Game Over and R).

- [X] T003 [P] Write the tuning tests first in the `#[cfg(test)] mod tests` of `rust/src/tuning.rs` (they will not compile until T004). Rebuild the test `raw()` helper for the new layout with values different from the file's, including a three-entry wave list. One passing and one failing case per new rule in [contracts/tuning-data.md](contracts/tuning-data.md#rules): `enemies.<kind>.knockback_resistance` "`0 <= r <= 1`" (pass 0 and 1, fail −0.1 and 1.1); `interruptible` "`true` or `false`" (fail a number and a string); `attack_token_cost` "a whole number, `1 <= cost <= crowd.max_attack_tokens`" (fail 0, 1.5 and pool + 1); `points` "a whole number, `> 0`"; `waves.list` "an array with at least one entry" (fail `[]` and a non-array); `waves.list[i].grunts`, `brutes` "whole numbers, `>= 0`, and `grunts + brutes >= 1`" (fail −1, 0.5 and `0 + 0`, with the field path `waves.list[1].grunts`); `extra_grunts_per_wave`, `brute_every_waves` "whole numbers, `>= 0`, not both 0"; `max_on_screen` "a whole number, `>= 1`"; `spawn_interval`, `breather_time`, `banner_time` "`>= 0`"; `crowd.max_attack_tokens` "a whole number, `>= 1`"; `crowd.waiting_distance` "`> 0`"; `crowd.drift_speed`, `crowd.separation_radius` "`>= 0`". M3's rules for each `enemies.<kind>` as they were for `grunt` (paths `enemies.brute.attack.knockback_time` etc.). Warning tests: `EnemyWindUpNotLonger` for each kind when its wind-up `<=` the hero's (equal and smaller), `BruteWindUpNotLonger` when the brute's `<=` the grunt's, none when longer; such values still pass `validate`
- [X] T004 In `rust/src/tuning.rs`, extend per [contracts/tuning-data.md](contracts/tuning-data.md) and [data-model.md](data-model.md#tuning): section and key constants for `enemies` (`grunt`, `brute`), `waves`, `crowd`; remove `grunt` and `run.respawn_delay`; one reader for an enemy section (M3's fighter reader plus the four new keys) used for both kinds; new readers `bool_in`, `whole_in` and `list_in` with field paths like `waves.list[2].grunts`; `pub enum EnemyKind { Grunt, Brute }` in `rust/src/enemy.rs` (re-exported for `tuning.rs`), `#[derive(GodotConvert, Var, Export)]` with `#[godot(via = GString)]` so the editor shows a drop-down for the scenes' `kind`; `EnemyTuning` (fields per [data-model.md](data-model.md#enemykind-and-enemytuning)); `TuningValues` stays `Copy` and gains `enemy(kind) -> EnemyTuning` (the one exhaustive match on the kind for values) and `crowd() -> CrowdTuning`; a separate `#[derive(Clone)] pub struct WaveTuning { list: Vec<WaveSpec>, … }` with accessors; `validate` returns `(TuningValues, WaveTuning)` all or nothing, rules in file order, no defaults; `TuningWarning` gains `EnemyWindUpNotLonger { kind, hero, enemy }` and `BruteWindUpNotLonger { grunt, brute }` with the contract's messages (the M3 `GruntWindUpNotLonger` becomes the grunt case). Rewrite `godot/data/tuning.json` exactly as the contract's JSON, keeping M3's `hero`, `dummy`, `lane` and `effects` values. Run `cargo test` until T003 passes (callers fixed in T005–T006) (depends on T003)
- [X] T005 In `rust/src/tuning_store.rs`: hold the `WaveTuning` beside the values; Rust-side `pub fn waves(&self) -> Option<WaveTuning>` (a clone, for the director); `warning()` returns the first of the warnings; `death_fade_time()` unchanged. `///` docs (depends on T004)
- [X] T006 Generalise the grunt (research R2): `git mv rust/src/grunt.rs rust/src/enemy.rs`; class `Grunt` → `Enemy` with `#[export] kind: EnemyKind` (default `Grunt`); `GruntMode` → `EnemyMode`, `grunt_after_hit` → `enemy_after_hit` (same rules for now); every value read through `values.enemy(self.kind)` instead of `values.grunt()`, with **no** `match` on the kind anywhere in the file; error messages say `Enemy: …`. In `rust/src/fighters.rs`, `FighterRef::Grunt` → `FighterRef::Enemy(Gd<Enemy>)`. In `rust/src/grunt_spawner.rs`, spawn `Enemy` and, until T011 replaces the spawner, use `waves.breather_time` from `Tuning.waves()` as the respawn delay (a `// TODO(T011)` note says so). Update `rust/src/lib.rs` (`mod enemy;` instead of `mod grunt;`) and `rust/src/hero.rs` if it names the grunt. `cargo build`, `cargo test` (depends on T005)
- [X] T007 [P] Rename the grunt scripts for both kinds (research R11): `git mv godot/world/grunt_body.gd godot/world/enemy_body.gd` and `git mv godot/world/grunt_health_bar.gd godot/world/enemy_health_bar.gd` (with their `.uid` files), typed against `Enemy` instead of `Grunt`, `##` docs saying they serve every kind; in `godot/world/screen_shake.gd` and `godot/debug/debug_hitboxes.gd`, `Grunt` → `Enemy` (depends on T006)
- [X] T008 (owner) In `godot/world/grunt.tscn`, change the root's type from `Grunt` to `Enemy` with `kind = Grunt`, and point `Body` and `HealthBar` at `res://world/enemy_body.gd` and `res://world/enemy_health_bar.gd`. Create `godot/world/brute.tscn` the same way ([contracts/wave-classes.md](contracts/wave-classes.md#scenes)): root `Enemy` named `Brute`, `kind = Brute`, **no script**; `Body` polygon `(-36, -140), (36, -140), (36, 0), (-36, 0)` (the brute's hurtbox, 72 × 140) in a colour unlike the hero's, the grunt's `#7A3B8F`, the dummy's and the pillars' (for example `#3F6B3A`), a `ShaderMaterial` with `res://world/flash.gdshader` (**Local to Scene**), script `enemy_body.gd`; `HealthBar` with `enemy_health_bar.gd`, raised to sit above the taller body if its offset is a constant (else note it for T025). Save (depends on T006, T007)
- [X] T009 Confirm with windowed Godot script runs (1280×720) that the game plays exactly as M3: tuning loads with the new file and no warning; one grunt (an `Enemy` of kind `Grunt`) enters, lines up, winds up, strikes, takes three blows, fades, counts a kill, and is replaced after `waves.breather_time`; F3, F4, F6, F1, F2 work; Game Over and R work; F5 rejects `enemies.brute.attack_token_cost` 3 naming that path. Instancing `brute.tscn` by script gives an enemy with the brute's health (120) and size. Then `cargo fmt --check`, `cargo clippy --all-targets` (0 warnings except dead code used in later phases) and `cargo test` (depends on T008)

**Checkpoint**: Tuning extended and tested, one `Enemy` class with two kinds, the brute scene
exists; the game still plays as M3.

---

## Phase 3: User Story 1 – Waves that clear and come again (Priority: P1) 🎯 MVP

**Goal**: A banner, a stream of the wave's enemies, a breather when it is cleared, and waves that
keep growing past the list.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 1–4.

- [X] T010 [P] [US1] Write the wave tests first in a new `rust/src/wave_director.rs` `#[cfg(test)] mod tests` (they will not compile until T011), per [data-model.md](data-model.md#waves-rustsrcwave_directorrs-pure): `composition(n)` returns the list's entry for `n <= L`; past it, `grunts = last.grunts + (n − L) · extra_grunts_per_wave` and `brutes = last.brutes + (n − L) / brute_every_waves` (integer division), with `brute_every_waves = 0` adding none; no wave has fewer enemies than the one before for 20 waves. `spawn_queue` has `grunts + brutes` entries with brute `i` at `round((i + 1) · t / (b + 1)) − 1` (for 3 + 1: `[G, B, G, G]`). The tick: `first()` is `Banner` for wave 1 and reports `Started(1)`; `Banner` ends → `Fighting` with the queue filled; `Fighting` spawns the front of the queue only when the timer is out and `living < max_on_screen`, then waits `spawn_interval`; no `Cleared` while the queue holds enemies or `living > 0`; queue empty and `living == 0` → `Breather` and `Cleared(n)`; `living > 0` during `Breather` → back to `Fighting` for the same wave; `Breather` ends → `Banner` for `n + 1` and `Started(n + 1)`; `push_front` puts a kind first and zeroes the timer; a kind pushed during `Banner` is still first in the queue after the banner ends, followed by the wave's own queue; `skip()` empties the queue and starts `Banner` for `n + 1`; nothing changes while not ready (no tuning, or not playing)
- [X] T011 [US1] Implement `rust/src/wave_director.rs` so T010 passes (research R8, [contracts/wave-classes.md](contracts/wave-classes.md#wavedirector-new-extends-node-replaces-gruntspawner)): pure `composition`, `spawn_queue`, `WaveState { Banner(Countdown), Fighting, Breather(Countdown) }`, `WaveProgress` with `first`, `tick`, `push_front`, `skip`, and `WaveEvent { None, Spawn(EnemyKind), Cleared(u32), Started(u32) }`; `#[class(base = Node)] pub struct WaveDirector` with `#[export] grunt_scene`, `#[export] brute_scene`, `#[export] world`, a `RandomNumberGenerator` `randomize()`d in `ready`, and `process_physics_priority` set to 1 in `ready`; handles to `Tuning`, `HitStop`, `Run` (missing ⇒ one `godot_error!("WaveDirector: …")`). Each physics tick, not while frozen: count living enemies (`Enemy` children of `world` with `is_alive()`), call `tick` with `ready = values present && Run.is_playing()`, and act on the event: `Spawn(kind)` instances the kind's scene (the director's one match on `EnemyKind`), `place`s it at a random edge and lane depth and `move_child`s it before the hero, as M3's spawner did; `Started(n)` emits `wave_started(n)`; `Cleared(n)` emits `wave_cleared(n)`. `#[func]`s `wave()`, `banner_wave()` (the number while in `Banner`, else 0), `#[func] spawn_grunt()` and `#[func] spawn_brute()`, both calling a Rust-side `spawn_now(kind)` (`push_front`; nothing at Game Over), `skip_wave()` (frees every `Enemy` child of `world` with `queue_free()` directly, never through a hit, so no kill or points are counted; then `skip()`; nothing at Game Over). Delete `rust/src/grunt_spawner.rs`; in `rust/src/lib.rs` replace `mod grunt_spawner;` with `mod wave_director;`. `cargo test`, `cargo clippy --all-targets` (depends on T009, T010)
- [X] T012 [P] [US1] Create `godot/ui/wave_banner.gd` (`extends Label`, `##` docs): `## The director whose banner is shown.` `@export var director: WaveDirector`; in `_process`, `visible = director.banner_wave() > 0` and, while visible, `text = "Wave %d" % director.banner_wave()`. No game state (research R11)
- [X] T013 [P] [US1] In `godot/debug/debug_overlay.gd`: `@export var spawner: GruntSpawner` → `## The director F3, F7 and F8 act on.` `@export var director: WaveDirector`; `debug_spawn_grunt` calls `director.spawn_grunt()`; add `debug_next_wave` → `director.skip_wave()`; both mark the event handled (depends on T011)
- [X] T014 [US1] (owner) In `godot/main.tscn` (contracts/wave-classes.md "Scenes"): delete `GruntSpawner`; add a `Node` `WaveDirector` (type `WaveDirector`) **after** `World` with `grunt_scene` → `res://world/grunt.tscn`, `brute_scene` → `res://world/brute.tscn`, `world` → `World`; add a `Label` `WaveBanner` to `Hud`, centred at the top of the play area (for example anchors top-centre, `offset_top` 96, font size 40), script `res://ui/wave_banner.gd`, `director` → `WaveDirector`; set the overlay's `director` → `WaveDirector`. Save (depends on T011, T012, T013)
- [X] T015 [US1] Confirm by windowed script runs: "Wave 1" is shown for `waves.banner_time`, then wave 1's two grunts enter `spawn_interval` apart from random edges at lane depths; `wave_started(1)` fires once; no new wave while an enemy lives or is still queued; after the last death, `wave_cleared(1)`, `breather_time` with nothing, then "Wave 2" and three grunts; F8 removes every enemy with `Run.kills()` unchanged and starts the next banner at once; skipping to wave 10 gives the counts of quickstart scenario 3 (wave 8: 7 grunts and 2 brutes) and never more than `max_on_screen` alive; F3 during a breather returns the director to `Fighting` for the same wave. No errors in the Output (depends on T014)
- [X] T016 [US1] (owner) Press Play and run quickstart scenarios 1–4. Fix anything that fails before going on (depends on T015)

**Checkpoint**: Waves come, clear and grow. Every enemy still hunts at once (turns come in US3).
M4a's MVP is playable.

---

## Phase 4: User Story 2 – The brute (Priority: P1)

**Goal**: A brute that takes many blows, hits hard, telegraphs longer, barely flinches, and cannot
be stopped mid-attack.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 5–7.

- [X] T017 [P] [US2] Write tests first in `rust/src/enemy.rs` `#[cfg(test)] mod tests` for the hit reaction ([data-model.md](data-model.md#hit-reaction-rustsrcenemyrs-pure)): `Reaction` from a hit and an enemy's tuning scales knockback distance and stun time by `1 − knockback_resistance` and keeps the knockback time (resistance 0: unchanged; 0.75: a quarter; 1: zero); `enemy_after_hit` follows the table: `Killed` in any hittable mode → `Dying(fade)` with push; `Survived` in `WindUp`, `Active` or `Recovery` with `interruptible = false` → mode unchanged, no push; `Survived` otherwise → `HitStun(stun_time)` with push (the attack dropped); `Entering` and `Dying` unchanged, no push
- [X] T018 [US2] In `rust/src/enemy.rs` (research R3): `Reaction` and the new `enemy_after_hit(mode, damage, reaction, fade_time) -> (EnemyMode, bool)` so T017 passes; `take_hit` builds the reaction from `values.enemy(self.kind)`, starts the knockback only when the result says push (with the scaled distance), and emits `was_hit(direction, strength)` with the scaled distance, or 0 for an armored blow. `cargo test`, `cargo clippy --all-targets` (depends on T011, T017)
- [X] T019 [P] [US2] In `godot/debug/debug_overlay.gd`, `debug_spawn_brute` calls `director.spawn_brute()` and marks the event handled; update the `##` docs listing the keys (F3 grunt, F7 brute, F8 next wave) (depends on T013)
- [X] T020 [US2] Confirm by windowed script runs: a brute from F7 enters as a bigger shape with its own bar; 12 hero blows kill it (a grunt 3); its strike costs the hero `enemies.brute.attack.damage` (35); `is_winding_up()` lasts `enemies.brute.attack.wind_up_time` (longer than the grunt's); a blow while it waits or approaches pushes it 15 px (60 · 0.25) and stuns it 0.1 s (0.4 · 0.25); ten blows during its wind-up, the hero staying in reach, leave it winding up and its strike still lands (10 of 10); a grunt is still stunned and cancelled by every blow. Wave 4 brings 3 grunts and 1 brute. No errors (depends on T018, T019)
- [X] T021 [US2] (owner) Press Play and run quickstart scenarios 5–7 (depends on T020)

**Checkpoint**: Two enemies that ask for different play.

---

## Phase 5: User Story 3 – Enemies take turns (Priority: P1)

**Goal**: Attack tokens limit who closes in; the rest wait on their own side; nobody stacks.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 8–11.

- [X] T022 [P] [US3] Write the crowd tests first in a new `rust/src/crowd.rs` `#[cfg(test)] mod tests` (they will not compile until T023), per [data-model.md](data-model.md#attacktokens-rustsrccrowdrs-pure): `AttackTokens` "`in_use() <= capacity` after every call" over a random-ish sequence of grants and reconciles; `reconcile` drops holders not in the list (release on recovery, hit-stun and death is the enemy leaving the holding modes); `grant` serves nearest first, ties by lower id; a nearest brute (cost 2) with one token free gets nothing and no farther grunt is served; `set_capacity` lower keeps holders and blocks new grants until enough are free. `separate`: two bodies closer than `radius` end at least `radius` apart (scaled) after applying the offsets; a fixed body does not move and its movable neighbour takes the whole push; two fixed bodies get no offsets; identical positions split along x with the lower index to the left; `depth_scale` makes a depth gap count more; `radius = 0` gives zeros; offsets followed by the lane clamp never leave `[lane.top, lane.bottom]`
- [X] T023 [US3] Implement `rust/src/crowd.rs` so T022 passes (research R5, R7): generic `AttackTokens<T: Copy + PartialEq + Ord>` with `new(capacity)`, `set_capacity`, `in_use`, `reconcile`, `grant`; `Body { x, y, movable }` and `separate(bodies, radius, depth_scale) -> Vec<(f32, f32)>`. No Godot types. `//!` and `///` docs. Add `mod crowd;` to `rust/src/lib.rs`; `cargo test` (depends on T022)
- [X] T024 [P] [US3] Add tests in `rust/src/enemy.rs` for `EnemyMode::holds_turn()` (true only for `Approaching`, `WindUp`, `Active`, `Recovery`) and `drift_step(position, spot, speed, delta)` (reaches the spot without overshooting on both axes; speed 0 stays put) (depends on T018)
- [X] T025 [US3] In `rust/src/enemy.rs` (research R4, R6, [data-model.md](data-model.md#enemy-was-grunt)): add `Waiting` to `EnemyMode` with every `match` exhaustive; `Entering` ends → `Waiting`; `Recovery` ends and `HitStun` ends → `Waiting`; `place()` fixes `side` (−1 left edge, +1 right) and `waiting_y` (its spawn depth); `Waiting` each tick: face the nearest living hero, drift with `drift_step` at `crowd.drift_speed` to `x = target.x + side · crowd.waiting_distance`, `y = waiting_y`, clamped to the lane and the screen less half its width; no target → stand still; never start an attack. `holds_turn()` and `drift_step` so T024 passes. Rust-side `pub fn grant_turn(&mut self)` (`Waiting` → `Approaching`, else nothing), `pub fn nudge(&mut self, dx: f32, dy: f32)` (moves and clamps), `pub fn turn_cost(&self) -> u32` and `pub fn turn_request(&self) -> Option<f32>` (its distance to its target while `Waiting`, else `None`); `#[func] is_waiting()`. `cargo test` (depends on T023, T024)
- [X] T026 [US3] In `rust/src/wave_director.rs` (research R5, R7): own an `AttackTokens<InstanceId>` with capacity from `crowd.max_attack_tokens` (`set_capacity` each tick). After the wave tick, each physics tick: (1) gather the living enemies; (2) `reconcile` with those whose `holds_turn()` is true; (3) `grant` among those with a `turn_request()`, then `grant_turn()` each granted enemy; (4) build `Body`s (movable for `Waiting`, `Approaching`, `HitStun`; fixed for `WindUp`, `Active`, `Recovery`; `Entering` and `Dying` left out), call `separate` with `crowd.separation_radius` and `depth_scale = hero.horizontal_speed / hero.vertical_speed`, and `nudge` each movable enemy by its offset. The director never runs while an enemy holds a bind on it (it is never called by enemies). `cargo test`, `cargo clippy --all-targets` (depends on T025)
- [X] T027 [P] [US3] In `godot/debug/debug_hitboxes.gd`, draw waiting enemies' hurtbox outlines in a distinct constant colour (`Enemy.is_waiting()`), so turns can be seen with F2 (depends on T025)
- [X] T028 [US3] Confirm by windowed script runs: with six grunts from F3 and a pool of 2, the sum of costs held by enemies in `Approaching`–`Recovery` never exceeds 2 over 60 s; each freed turn goes to the waiting grunt nearest the hero; a brute attacking alone blocks every grunt; a nearest waiting brute with one token free blocks farther grunts; waiting enemies sit on their entry side about `waiting_distance` from the hero at their spawn depth and drift back when the hero moves; walking up to a waiting enemy never makes it attack; no two enemies end a frame closer than `separation_radius` (scaled) while waiting or approaching, including at the lane edges; `drift_speed` 0 holds them still. No errors (depends on T026, T027)
- [X] T029 [US3] (owner) Press Play and run quickstart scenarios 8–11 (depends on T028)

**Checkpoint**: A crowd that takes turns and keeps apart: hits can be seen coming again.

---

## Phase 6: User Story 4 – Score and the session best (Priority: P1)

**Goal**: Points per kill, a HUD score, Game Over with score, wave, kills and best; the best
survives R.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 12–14.

- [X] T030 [P] [US4] Write the run tests first in `rust/src/run.rs` `#[cfg(test)] mod tests` (data-model.md "Run"): `add_kill(points)` adds one kill and the points while `Playing` and nothing after `GameOver`; `set_wave` only while `Playing`; `end_run` returns true once and sets `best = max(best, score)`; `reset` folds the score into `best`, then gives `Playing`, 0 kills, 0 score, wave 0, and never lowers `best` (three runs with scores 50, 20, 80 leave best 50, 50, 80); a run reset mid-way with a new high score keeps it as `best`; and a record that has counted kills is unchanged in kills and score by anything but `add_kill` (the guarantee `skip_wave` relies on: it frees enemies directly and never calls `add_kill`)
- [X] T031 [US4] Implement in `rust/src/run.rs` so T030 passes (research R9, [contracts/wave-classes.md](contracts/wave-classes.md#runtracker-extended-autoload-run)): `RunRecord { state, kills, score, wave, best }`; `#[signal] score_changed(score: i64)`, emitted when `add_kill` counts; `#[signal] game_over(kills: i64, score: i64, wave: i64, best: i64)` replacing `game_over(kills)`; `#[func]`s `score()`, `wave()`, `best()`; Rust-side `add_kill(points)` and `set_wave(n)`; `restart()` keeps the best. In `rust/src/enemy.rs`, the kill calls `run.bind_mut().add_kill(values.enemy(self.kind).points())`. In `rust/src/wave_director.rs`, `Started(n)` also calls `Run.set_wave(n)` before emitting `wave_started`. `skip_wave` frees enemies with `queue_free()` only and never calls `take_hit` or `add_kill` (T030 covers the record; T035 checks it in play). `cargo test`, `cargo clippy --all-targets` (depends on T026, T030)
- [X] T032 [P] [US4] Create `godot/ui/score_label.gd` (`extends Label`, `##` docs): `text = "Score: %d" % Run.score()` in `_ready`; connect `Run.score_changed` to a handler that sets the text from its argument and calls nothing on `Run` (research R11)
- [X] T033 [P] [US4] In `godot/ui/game_over.gd`: the handler takes `(kills: int, score: int, wave: int, best: int)` and fills the labels "Game Over", `"Score: %d"`, `"Wave reached: %d"`, `"Kills: %d"`, `"Best this session: %d"` and "Press R to restart" from its arguments only (depends on T031)
- [X] T034 [US4] (owner) In `godot/main.tscn`: add a `Label` `ScoreLabel` to `Hud` under the kills label (for example at (16, 56)) with script `res://ui/score_label.gd`, and move `DebugOverlay/Panel` down so it starts below it (for example `offset_top` 84); in `GameOver/Box`, add `Label`s `Score`, `Wave` and `Best` between `Title` and `Hint` (centred). Save (depends on T032, T033)
- [X] T035 [US4] Confirm by windowed script runs: a grunt kill raises `Run.score()` by `enemies.grunt.points` and a brute kill by `enemies.brute.points`, each emitting `score_changed` once; F8 changes neither score nor kills; at Game Over mid-wave the screen shows the HUD's score, that wave, the kills and the best; at Game Over during a breather the wave just cleared; R at Game Over and mid-run gives within 2 s (press to first frame) wave 1's banner, score 0, kills 0, no leftover enemies or dummies; the best equals the highest score of the session, including a run ended with R. No errors (depends on T034)
- [X] T036 [US4] (owner) Press Play and run quickstart scenarios 12–14 on the UHD 630, timing R (depends on T035)

**Checkpoint**: The full M4a loop: waves, two enemies, turns, a score and a best to beat.

---

## Phase 7: User Story 5 – Tune the waves live, and cheat (Priority: P2)

**Goal**: Every new value applies with F5 at the contract's moment; bad values are rejected; the
warnings show; the cheats are absent from release builds.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 15–19.

- [X] T037 [US5] Confirm by windowed script runs: each new value in [contracts/tuning-data.md](contracts/tuning-data.md) changed and reloaded with F5 takes effect at the moment the contract gives (wave list and growth at the next banner, the current wave keeping its queue; `spawn_interval` and `max_on_screen` the next spawn; `breather_time`, `banner_time` the next one; token pool and costs the next grant with holders kept; `waiting_distance`, `drift_speed`, `separation_radius` the next tick; resistance and `interruptible` the next hit; points the next kill); the bad values of quickstart scenario 16 are each rejected naming the field; `enemies.brute.attack.wind_up_time` 0.4 is accepted with the brute warning, and restoring clears it. F3, F7 and F8 do nothing at Game Over. Restore `godot/data/tuning.json` afterwards (depends on T035)
- [X] T038 [US5] Build and export a release (`cd rust && cargo build --release`, then `godot --headless --path godot --export-release Linux export/beastrider.x86_64`); confirm `godot/export/` holds the binary, `.pck` and `libbeastrider.so` and that a 5-second windowed run prints no errors (depends on T035)
- [X] T039 [US5] (owner) Quickstart scenarios 15–19: retune live, the bad values, the warning, the cheats during a wave, a breather and Game Over, and the release build (waves, brutes, score, Game Over with best and R work; F1–F8 do nothing) (depends on T037, T038)

**Checkpoint**: The difficulty curve can be tuned live; cheats exist in debug builds only.

---

## Phase 8: User Story 6 – The curve feels right (Priority: P1)

**Goal**: The owner's judgement that wave 1 is easy, difficulty rises noticeably, and every death
felt earned.

**Independent Test**: [quickstart.md](quickstart.md) scenario 21.

- [X] T040 [US6] (owner) Play and retune `godot/data/tuning.json` with F5 until the curve feels right, using F2, F3, F4, F7 and F8 as needed. Then play five runs to Game Over with the final values and judge (SC-012): was wave 1 easy, did each wave feel noticeably harder than the last, and did every death feel earned? If not, keep retuning; the milestone is not done until the answer is yes (depends on T039)
- [X] T041 [US6] Record the final values: confirm `godot/data/tuning.json` holds them, passes the rules and gives no warning; update the JSON and the starting-values sentence in `specs/006-m4a-waves-and-brute/contracts/tuning-data.md` to match, so the contract shows what shipped (depends on T040)

**Checkpoint**: The owner says the curve is right.

---

## Phase 9: User Story 7 – Still 60 fps, clean and committed (Priority: P2)

**Goal**: 60 fps with ten enemies; the Quality Gate is green.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 20, 23, 24 and 26.

- [X] T042 [US7] (owner) Quickstart scenario 20: F3 and F7 to ten enemies (mixed), F2 on, panel open; fight for 60 s by the clock, F4 to stay alive, window focused. FPS stays 60 apart from the first second and the impact freezes (depends on T039)
- [X] T043 [US7] In `rust/`, run `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets` and `cargo test`; fix every warning in `rust/src/*.rs` until all pass with 0 warnings. No `#[allow]` without a written reason
- [X] T044 [US7] Check every GDScript file (`godot/world/hit_flash.gd`, `hero_body.gd`, `enemy_body.gd`, `enemy_health_bar.gd`, `screen_shake.gd`, `lane_band.gd`, `godot/ui/fps_label.gd`, `health_bar.gd`, `kills_label.gd`, `score_label.gd`, `wave_banner.gd`, `game_over.gd`, `godot/debug/debug_overlay.gd`, `debug_markers.gd`, `debug_hitboxes.gd`) through the editor's language server, as in M2b's T031: 0 errors or warnings, `untyped_declaration` still Error

**Checkpoint**: The Quality Gate is green.

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Constitution review, docs and the milestone commit.

- [X] T045 [P] Check for magic numbers (Principle IV): none of the values in `godot/data/tuning.json` appears as a literal in `rust/src` (outside tests, whose values must differ) or in `godot/world`, `godot/ui` and `godot/debug` scripts; the only numbers in scripts are named presentation constants
- [X] T046 [P] Check the mode and kind rules (FR-009, FR-022): `grep -rnE 'is_attacking: bool|dead: bool|stunned: bool|has_token: bool|waiting: bool' rust/src` returns nothing; every `match` on `EnemyMode`, `WaveState`, `WaveEvent`, `RunState`, `Team`, `Damage`, `FighterRef` and `EnemyKind` has no `_ =>` arm; `grep -rn 'EnemyKind::' rust/src` outside tests shows only `TuningValues::enemy`, the director's scene choice and `Export` defaults
- [X] T047 [P] Check the split (FR-022) and co-op (FR-023): no `.gd` file decides a wave, a turn, a spawn, a hit, a kill, the score or the best (they only read Rust and react to `was_hit`, `died`, `score_changed`, `game_over`); enemies target and are granted tokens through `team_players`, never a fixed node path; `grep -rnE 'KEY_|keycode' godot/world godot/ui godot/debug` returns nothing; `grep -rn '"p1_' rust/src` returns nothing
- [X] T048 [P] Review docs: `//!` module docs on `rust/src/enemy.rs`, `crowd.rs`, `wave_director.rs`, `run.rs`, `tuning.rs` (pointing at `specs/006-m4a-waves-and-brute/contracts/tuning-data.md`); `///` on every public item and everything exposed to Godot, including the new signals; `// SAFETY:` in `rust/src/lib.rs` unchanged; `##` docs on every new or renamed script and exported property
- [X] T049 Run the Constitution compliance review required by Governance: check the finished work against Principles I–VIII (v1.3.3) and the Code Style section, using the table in `specs/006-m4a-waves-and-brute/plan.md` as the checklist; record any departure (for example scenes built by the agent) in that file's Complexity Tracking
- [X] T050 After a build, a Play and the export, run `git status`: `rust/target/`, `godot/.godot/` and `godot/export/` absent; `rust/src/grunt.rs` and `grunt_spawner.rs` gone, `enemy.rs`, `crowd.rs` and `wave_director.rs` present; the new and renamed `.gd` files with their `.uid` files, `godot/world/brute.tscn`, `godot/world/grunt.tscn`, `godot/main.tscn`, `godot/project.godot` and `godot/data/tuning.json` present to be committed
- [X] T051 (owner) Run the full quickstart.md, scenarios 1–26, in one sitting on the target machine; scenario 25 re-checks M3 and M2b
- [X] T052 Commit the M4a implementation on branch `feat/m4a-waves-and-brute` in small logical commits that each pass the gate on their own staged tree (for example: input actions; tuning extension with `Enemy` and the brute scene; waves and the director; the brute's weight; turns and spacing; score and best; final values and docs), staging by explicit path and reviewing `git diff --staged` before each, as the owner's git rules require. Ask the owner before committing; then `git status` must be clean (depends on T040–T051)
- [X] T053 After the merge, update the status line in `docs/project-brief.md` and the live brief (M4a done, M4b next) on the M4b branch, as was done after M3

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies. T002 is needed before T019 (F7) and T013 (F8) can be tried.
- **Foundational (Phase 2)**: Needs T001. Blocks every story. Ends playing as M3.
- **US1 (Phase 3)**: Needs Phase 2. The MVP: waves.
- **US2 (Phase 4)**: Needs US1 (the director spawns brutes and F7 calls it).
- **US3 (Phase 5)**: Needs US2's `enemy_after_hit` (T018) and US1's director (T011).
- **US4 (Phase 6)**: Needs US3's director tick (T026) and the enemy kill path.
- **US5 (Phase 7)**: Needs US4 (every value and cheat in place).
- **US6 (Phase 8)**: Needs US5 (live tuning and cheats to judge with).
- **US7 (Phase 9)**: T042 needs US5; T043–T044 need all code.
- **Polish (Phase 10)**: Needs all stories.

### Within Each Phase

- Tests before the code they test (T003 → T004, T010 → T011, T017 → T018, T022 → T023,
  T024 → T025, T030 → T031).
- Rust classes registered (T006, T011) before scenes use them (T008, T014).
- Scripts before the scenes that use them (T007 → T008; T012, T013 → T014; T032, T033 → T034).
- Windowed checks, then the owner's play checks, last in each story.

### Parallel Opportunities

- **Phase 2**: T003 first; T007 alongside T008's preparation once T006 builds.
- **US1**: T010 (wave tests) first; T012 and T013 together once T011 builds.
- **US2**: T017 first; T019 alongside T018.
- **US3**: T022 and T024 together (different files); T027 alongside T026.
- **US4**: T030 first; T032 and T033 together.
- **Polish**: T045–T048 in parallel.

---

## Parallel Example: User Story 3

```bash
# Tests first, together (different files):
Task: "Write the crowd tests in rust/src/crowd.rs"
Task: "Add holds_turn and drift_step tests in rust/src/enemy.rs"

# Once the director grants turns:
Task: "Draw waiting enemies in godot/debug/debug_hitboxes.gd"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: actions. 2. Phase 2: tuning, `Enemy` with kinds, the brute scene (game still plays
as M3).
3. Phase 3: US1. Waves with banners, a breather and growth past the list.
4. **STOP and VALIDATE**: quickstart 1–4. A first feel for the curve can be judged here, with every
   enemy attacking at once.

### Incremental Delivery

1. Setup + Foundational → tests pass, M3 unchanged
2. US1 → waves → MVP (session 1)
3. US2 → the brute weighs more and cannot be out-swung
4. US3 → turns and spacing: crowds become readable
5. US4 → score and best: an outcome to beat (session 2)
6. US5 → live tuning of the curve, cheats
7. US6 → the curve is right, by the owner's judgement
8. US7 + Polish → 60 fps, gate green, committed (session 3) → M4a done

### If a Session Stalls

Shrink the next step (constitution, Development Workflow): for example, let waves spawn all at once
before adding the stream and the on-screen maximum; give the brute only resistance before its
armored attacks; hand out tokens to any waiting enemy before ordering by distance; skip separation
until the turn order works. Add each piece behind its tuning value. Never put a tuning value in
code as a shortcut. Every checkpoint is a safe place to stop.

---

## Notes

- [P] = different files, no dependency on unfinished tasks.
- (owner) tasks need the Godot editor GUI or a judgement made by playing.
- Commit only at the end, with the owner's go-ahead (T052). Rust parts that are unused until a
  later phase would fail clippy's dead-code check on their own, so commits group them with their
  first user, as in M3.
- Godot creates a `*.uid` file next to each new script. Commit it; it is not cache. `git mv` keeps
  the renamed scripts' history; move their `.uid` files with them.
- The `grunt.tscn` root changes type (`Grunt` → `Enemy`); open it once in the editor after T006 so
  Godot rewrites nothing unexpected, and check `git diff` shows only the type, `kind` and the
  script paths.
