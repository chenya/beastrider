---

description: "Task list for M2b Hit and Feel"
---

# Tasks: M2b Hit and Feel

**Input**: Design documents from `specs/004-m2b-hit-and-feel/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Rust unit tests are included, because the owner listed them in the plan input
([research.md R8](research.md)): overlap with and without depth tolerance, mirrored boxes, one hit
per swing, two targets in one swing, knockback clamped at the edges, hit-stop takes the max, phases
advance on time; plus each new tuning rule and the spawn stepping. They are written before the code
they test. Everything else is verified by playing, using [quickstart.md](quickstart.md).

**Organization**: Tasks are grouped by user story. Phases follow priority, except that US4 (P1,
the feel playtest) runs after US3 (P2), because judging feel is easier with the hitboxes, the
counter and extra dummies available.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on unfinished tasks)
- **[Story]**: Which user story the task belongs to (US1–US5 from spec.md)
- **(owner)**: Needs the owner: the Godot editor GUI, or a judgement made by playing. The owner may
  hand an editor task to the agent, which then uses a headless Godot script (`PackedScene.pack` +
  `ResourceSaver.save`) or `ProjectSettings` (`event.device = -1`), as recorded in the plan's
  Complexity Tracking.

## Path Conventions

`rust/` (the crate) and `godot/` (the project), as before. Run Rust commands from `rust/`. Values
in tests must differ from those in `godot/data/tuning.json` (see
[contracts/tuning-data.md](contracts/tuning-data.md)), so the magic-number check in T037 stays
meaningful. Size- and focus-dependent checks need a windowed 1280×720 run, not `--headless`.

## Session plan (about 15 h/week, Principle II)

- **Session 1**: Phases 1–2 (actions, tuning, pure combat rules with tests, `HitStop`).
- **Session 2**: Phase 3 (US1: swing, dummy, four effects). Ends playable: the hero hits the dummy.
- **Session 3**: Phases 4–8 (live tuning checks, F2/F3/counter, feel playtest, 60 fps, gate,
  commit).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm nothing was upgraded and add the three new actions.

- [X] T001 Confirm nothing was upgraded (research.md R1): `godot --version` prints `4.7.2.stable.official.ed1daf0bf`; `rust/Cargo.toml` still has `godot = { version = "=0.5.5", features = ["api-4-6"] }`; `rust/rust-toolchain.toml` still pins `1.100.0`; `cargo build` and `cargo test` in `rust/` pass (20 tests). If anything differs, stop and ask the owner
- [X] T002 (owner) In Project Settings → Input Map, add three actions bound to **physical** keys, deadzone 0.2, and save to `godot/project.godot`: `p1_attack` = J + Z, `debug_show_hitboxes` = F2, `debug_spawn_dummy` = F3 ([contracts/combat-classes.md](contracts/combat-classes.md#input-map-actions-added)). Confirm each event has `"device":-1` and `git diff godot/project.godot` shows nothing else

**Checkpoint**: Toolchain unchanged; the Input Map has the attack and debug actions.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The new tuning values, the pure combat rules, and the `HitStop` autoload, all under
tests where the owner asked. Every story needs them.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete. At the end the game
still plays exactly as M2a.

- [X] T003 [P] Write the tuning tests first in the `#[cfg(test)] mod tests` of `rust/src/tuning.rs` (they will not compile until T004). Extend the test `raw()` helper with values different from the file's (for example wind-up 0.05, active 0.12, recovery 0.2, reach 80, height 50, depth tolerance 15, hit stop 0.05, flash 0.1, shake strength 4, shake time 0.1, knockback 50 over 0.25, hurtbox 40×100). One passing and one failing case per rule in [contracts/tuning-data.md](contracts/tuning-data.md): `attack.active_time` "`> 0`" (fail 0); `attack.reach` "`> 0`" (fail 0); `impact.knockback_time` "`> 0` when `impact.knockback_distance > 0`" (fail time 0 with distance 50; pass time 0 with distance 0); every other new field "`>= 0`" (pass 0, fail −1) and finite (fail NaN). Each rejection's `field()` is the dotted path (`impact.knockback_time`)
- [X] T004 In `rust/src/tuning.rs`, add the attack, impact and dummy values ([data-model.md](data-model.md#tuning-rules-side-tuningvalues-extended)): new section and key constants (`attack`: `wind_up_time`, `active_time`, `recovery_time`, `reach`, `height`, `depth_tolerance`; `impact`: `hit_stop_time`, `flash_time`, `shake_strength`, `shake_time`, `knockback_distance`, `knockback_time`; `dummy`: `hurtbox_width`, `hurtbox_height`); the fourteen `f64` fields in `RawTuning`; `read_raw_tuning` reading them; and in `TuningValues` private `f32` fields with `///`-documented accessors of the same names (group them in `Copy` sub-structs `AttackTuning`, `ImpactTuning`, `DummyTuning` returned by `attack()`, `impact()`, `dummy()`). `validate` applies the rules after M1's, in file order, with no defaults. Then add the same three sections with the starting values to `godot/data/tuning.json` exactly as in [contracts/tuning-data.md](contracts/tuning-data.md). Run `cargo test` until T003 passes (depends on T003)
- [X] T005 [P] Write the combat tests first in a new `rust/src/combat.rs` `#[cfg(test)] mod tests` (they will not compile until T006), covering the owner's list (research.md R8, data-model.md formulas): `attack_box` for `Facing::Right` and `Facing::Left` is mirrored around the origin; `lands` is true for overlapping boxes at equal depth, true for touching edges, true at exactly the tolerance, false just beyond it, false for boxes apart; a tolerance of 0 lands only at equal depth; `advance_swing` moves WindUp → Active → Recovery on time, passes zero-length wind-up or recovery in one tick, returns `Finished` after recovery and `Restart` when recovery was `queued`; a swing that keeps overlapping a target for many ticks yields one hit (use a helper that runs the Active phase against a target list and a `hit_this_swing` list); one swing reaching two targets hits both once; `knockback_offset` is 0 at the start, the full distance at and after the time, monotonic, and 0 for distance 0; clamping a slide at the left and right screen limits and the lane limits keeps the position inside; `extend_freeze` returns the max, not the sum (0.05 then 0.03 → 0.05; 0.02 then 0.06 → 0.06); `spawn_point` starts just inside reach, steps half a width per occupied spot along the facing, and clamps to the screen
- [X] T006 Implement `rust/src/combat.rs` so T005 passes, with no Godot types: `Facing { Left, Right }` with `sign()`; `SwingPhase { WindUp, Active, Recovery { queued: bool } }`; `SwingTimings`; `SwingStep { Continue, Finished, Restart }`; `Rect { left, right, top, bottom }`; `attack_box(origin, facing, half_width, reach, height)`; `hurtbox(origin, width, height)`; `lands(attack, hurt, attacker_y, target_y, depth_tolerance)`; `advance_swing(phase, elapsed, delta, timings) -> (SwingPhase, f32, SwingStep)`; `knockback_offset(distance, time, elapsed)`; `clamp_to_area(position, half_width, screen_width, lane_top, lane_bottom)`; `extend_freeze(remaining, new)`; `spawn_point(...)`. `//!` module doc, `///` on every public item, exhaustive `match`es only. Add `mod combat;` to `rust/src/lib.rs`; `cargo test` (depends on T005)
- [X] T007 Create `rust/src/hit_stop.rs` per [contracts/combat-classes.md](contracts/combat-classes.md#hitstoptimer-extends-node-new-autoload-hitstop): `#[class(base = Node)] pub struct HitStopTimer { remaining: f32, base }` with hand-written `init`; `physics_process` counts `remaining` down to 0; `#[func] freeze(seconds)` sets `remaining = combat::extend_freeze(remaining, seconds)` (0 does nothing); `#[func] is_frozen()`. Docs say why the autoload name differs from the class name (research.md R5). Add `mod hit_stop;` to `rust/src/lib.rs`; `cargo build` and `cargo clippy --all-targets` (depends on T006)
- [X] T008 (owner) Create `godot/autoload/hit_stop.tscn` with one root node of type `HitStopTimer` named `HitStop`, and add it in Project Settings → Autoload as `HitStop` (Global Variable on) **after** `Tuning`, so `godot/project.godot` lists `HitStop="*res://autoload/hit_stop.tscn"` below `Tuning` (depends on T007)
- [X] T009 [P] In `rust/src/tuning_store.rs`, add `#[func]` accessors `flash_time()`, `shake_strength()` and `shake_time()`, each `0.0` without values, with `///` docs (contracts/combat-classes.md) (depends on T004)
- [X] T010 Confirm with a windowed Godot script run (1280×720) that `HitStopTimer` is registered with `freeze` and `is_frozen`; `/root/HitStop` exists after `/root/Tuning`; `Tuning.has_values()` is true with the new file; `HitStop.freeze(0.1)` makes `is_frozen()` true and false again after about 6 physics ticks; the hero still walks as in M2a; and the Output has no errors. Then run `cargo fmt --check`, `cargo clippy --all-targets` (0 warnings except dead code used in Phase 3) and `cargo test` (depends on T008, T009)

**Checkpoint**: Tuning, combat rules and `HitStop` exist and are tested; the game still plays as M2a.

---

## Phase 3: User Story 1 – Swing at the dummy and land a hit (Priority: P1) 🎯 MVP

**Goal**: The hero swings with timed parts, faces the way it last walked, and a hit on the dummy
gives the freeze, the flash, the shake and the knockback.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 1–7.

- [X] T011 [US1] Create `rust/src/training_dummy.rs` per [contracts/combat-classes.md](contracts/combat-classes.md#trainingdummy-extends-node2d-new) and data-model.md: `#[class(base = Node2D)] pub struct TrainingDummy` with `slide: Option<Slide>` and handles to `TuningStore` and `HitStopTimer` found in `ready` (`/root/Tuning`, `/root/HitStop`; missing ⇒ one `godot_error!("TrainingDummy: …")` and the dummy never moves). In `ready`, join the group `combat::HITTABLE_GROUP` (one `pub const` = `"hittable"`). `#[signal] was_hit(direction: f32, strength: f32)`. Rust-only `pub fn take_hit(&mut self, direction: f32)`: if `knockback_distance > 0`, replace `slide` with one starting at the current x; then emit `was_hit(direction, knockback_distance)`. `physics_process`: unless `HitStop.is_frozen()`, advance the slide using `combat::knockback_offset`, then clamp with `combat::clamp_to_area` using `hurtbox_width / 2`, the viewport width and the lane from `Tuning`, ending the slide at a limit or when `elapsed >= time`. `#[func] hurtbox() -> Rect2` from `combat::hurtbox` (contract). `//!` and `///` docs. Add `mod training_dummy;` to `rust/src/lib.rs` (depends on T010)
- [X] T012 [US1] In `rust/src/hero.rs` (research.md R2–R3, data-model.md "Hero"): add `facing: Facing` (`Right` at start), `mode: HeroMode { Free, Attacking { phase: SwingPhase, elapsed: f32, hit_this_swing: Vec<InstanceId> } }` and `hits_landed: i64`; extend `PlayerActions` with `attack` (`p{index}_attack`, checked like the others) and `Setup` with `hit_stop: Gd<HitStopTimer>` (`/root/HitStop`, error `Hero: the HitStop autoload is missing or is not a HitStopTimer`). In `physics_process`: return early if `hit_stop.is_frozen()`; then `match` the mode exhaustively. `Free`: if `is_action_just_pressed(attack)`, start a swing (WindUp, elapsed 0, cleared list) and do not move this tick; otherwise move as before and set `facing` from a non-zero `input.x`. `Attacking`: no movement; advance with `combat::advance_swing` using the attack timings from `Tuning` captured when the swing started (so F5 applies from the next swing, spec edge case); in `Recovery` a just-pressed attack sets `queued`; in `WindUp`/`Active` it is ignored; in `Active`, test every node in `HITTABLE_GROUP` that is a `TrainingDummy` and not in `hit_this_swing` with `combat::lands` (attack box from `combat::attack_box` with the `Body` half width), and for each hit call `take_hit(facing.sign())`, push its `InstanceId`, add 1 to `hits_landed`; if any dummy was hit this tick and `hit_stop_time > 0`, call `HitStop.freeze(hit_stop_time)` once. `Finished` ⇒ `Free`; `Restart` ⇒ a new swing. Update docs (depends on T011)
- [X] T013 [P] [US1] Add to `rust/src/hero.rs` the `#[func]`s `hits_landed() -> i64`, `attack_box() -> Rect2` (empty `Rect2` unless the swing is `Active`) and `dummy_spawn_point() -> Vector2` (calls `combat::spawn_point` with the hero's position, facing, half width, reach, `hurtbox_width`, the x of every `TrainingDummy` in the group, and the viewport width), with `///` docs (contracts/combat-classes.md). Run `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets` (0 warnings) and `cargo test` (depends on T012)
- [X] T014 [P] [US1] Create `godot/world/flash.gdshader`: `shader_type canvas_item;`, `uniform float flash_amount : hint_range(0.0, 1.0) = 0.0;`, and `COLOR.rgb = mix(COLOR.rgb, vec3(1.0), flash_amount);` in `fragment()` (research.md R6)
- [X] T015 [P] [US1] Create `godot/world/dummy_flash.gd` (`extends TrainingDummy`, `##` docs): in `_ready`, connect `was_hit` to a handler that, when `Tuning.flash_time() > 0`, kills any running flash tween and tweens the `Body` material's `flash_amount` from 1.0 to 0.0 over `Tuning.flash_time()`; at 0 it does nothing. Typed; the material reached with an `@onready` typed reference to `$Body` (research.md R6)
- [X] T016 [P] [US1] Create `godot/world/screen_shake.gd` (`extends Camera2D`, `##` docs): in `_enter_tree`, connect `get_tree().node_added` to a handler that connects `was_hit` of every `TrainingDummy` that enters; on `was_hit`, if `Tuning.shake_strength() > 0` and `Tuning.shake_time() > 0`, (re)start a shake with that strength and time; in `_process`, while time is left, set `offset` to a random vector of length up to `strength × time_left / shake_time` and count down with `delta`, then reset `offset` to zero. Several hits on one tick restart one shake, never add up (research.md R6)
- [X] T017 [US1] (owner) Create `godot/world/training_dummy.tscn`: root `TrainingDummy` named `TrainingDummy` with script `res://world/dummy_flash.gd`; child `Polygon2D` `Body` with polygon `(-28, -120), (28, -120), (28, 0), (-28, 0)` in a colour unlike the hero's and the pillars' (for example `#8FB339`), and a `ShaderMaterial` using `res://world/flash.gdshader` with **Local to Scene** on. Then in `godot/main.tscn`: add a `Camera2D` `ShakeCamera` as the **first** child of `Main` (Anchor Mode **Fixed Top Left**, position (0, 0), enabled, script `res://world/screen_shake.gd`), and an instance of the dummy scene named `Dummy` inside `World` **before** `Hero`, standing in the lane (for example at (760, 600)). Save (depends on T013, T014, T015, T016)
- [X] T018 [US1] (owner) Press Play and run quickstart scenarios 1–7: hits with all four effects in order (flash and shake from impact, through the freeze; slide after it), misses at a different depth or out of reach, facing, no movement during a swing, the recovery press, one hit per swing, two dummies (use a second dummy added in the editor for now if F3 is not built yet), edges. No errors in the Output. Fix anything that fails before going on (depends on T017)

**Checkpoint**: The hero hits the dummy with weight. M2b's MVP is playable.

---

## Phase 4: User Story 2 – Tune the impact live, one effect at a time (Priority: P1)

**Goal**: Every attack and impact value changes the next swing on F5, and zero turns an effect off.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 10–12.

- [X] T019 [US2] Confirm by a windowed script run that each of the fourteen values, changed in `godot/data/tuning.json` and reloaded with F5, changes the next swing (timing in ticks, reach, depth tolerance, freeze ticks, knockback distance and time, hurtbox width) and that `flash_time`, `shake_strength`/`shake_time`, `hit_stop_time` and `knockback_distance` at 0 each turn only that effect off; then that `impact.knockback_time` 0 with distance 60, `attack.active_time` 0 and `attack.reach` −5 are each rejected with a message naming the field. Restore the file afterwards (depends on T018)
- [X] T020 [US2] (owner) Quickstart scenarios 10–12 by playing: retune each value live; turn each effect off on its own; try the three bad values. Restore `godot/data/tuning.json` and confirm `git diff godot/data/tuning.json` shows only T004's additions (depends on T019)

**Checkpoint**: The impact can be tuned live, one effect at a time.

---

## Phase 5: User Story 3 – See the hitboxes, spawn dummies, count hits (Priority: P2)

**Goal**: F2 draws the boxes, F3 spawns dummies, the panel counts hits; none of it in release.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 5, 6, 8, 9 and 14.

- [X] T021 [P] [US3] Create `godot/debug/debug_hitboxes.gd` (`extends Node2D`, `##` docs): `@export var hero: Hero` and `@export var world: Node2D` with `##` docs; colour and line-width constants; in `_process`, `queue_redraw()` while visible; in `_draw`, outline `hero.attack_box()` when it has area, and the `hurtbox()` of every `TrainingDummy` child of `world` (research.md R7). No game state
- [X] T022 [US3] In `godot/debug/debug_overlay.gd`: add `## The hitbox drawer shown and hidden with F2.` `@export var hitboxes: Node2D`, `## Where F3 puts new dummies.` `@export var world: Node2D`, `## The scene F3 instances.` `@export var dummy_scene: PackedScene`, and an `@onready` `HitsLine` label. In `_ready`'s release branch also `hitboxes.queue_free()`; otherwise hide `hitboxes`. In `_unhandled_input`: `debug_show_hitboxes` toggles `hitboxes.visible` (panel open or not); `debug_spawn_dummy` instances `dummy_scene`, sets its `position` to `hero.dummy_spawn_point()`, adds it to `world` and `move_child`s it to just before `hero` so the hero stays last; both mark the event handled. In `_process`, `HitsLine` shows `Hits: %d` from `hero.hits_landed()` (depends on T013, T021)
- [X] T023 [US3] (owner) In `godot/main.tscn`: add `Node2D` `DebugHitboxes` after `DebugMarkers` (`z_index` 1, script `res://debug/debug_hitboxes.gd`, `hero` → `World/Hero`, `world` → `World`); add a `Label` `HitsLine` at the end of `DebugOverlay/Panel/Lines`; set the overlay's `hitboxes` → `DebugHitboxes`, `world` → `World`, `dummy_scene` → `res://world/training_dummy.tscn`. Save (depends on T022)
- [X] T024 [US3] (owner) Press Play and run quickstart scenarios 5, 6, 8 and 9: the counter, two dummies in one swing, F2's boxes explaining every hit and miss (with the panel closed too), F3's stepping both ways. No errors (depends on T023)
- [X] T025 [US3] Build and export a release (`cd rust && cargo build --release`, then `godot --headless --path godot --export-release Linux export/beastrider.x86_64`); confirm `godot/export/` holds the binary, `.pck` and `libbeastrider.so` and that a 5-second run prints no errors (depends on T022)
- [X] T026 [US3] (owner) Quickstart scenario 14: run `godot/export/beastrider.x86_64`; swing at the dummy (all four effects work); press F2 and F3 (nothing happens) (depends on T025)

**Checkpoint**: Hitboxes, spawning and the counter work in debug builds only.

---

## Phase 6: User Story 4 – It feels weighty (Priority: P1)

**Goal**: The owner's judgement that 30 hits feel weighty, not floaty.

**Independent Test**: [quickstart.md](quickstart.md) scenario 13.

- [X] T027 [US4] (owner) Play and retune `godot/data/tuning.json` with F5 until the impact feels right, using F2, F3 and the counter as needed. Then land 30 hits with the final values and judge: weighty or floaty (SC-011). If floaty, keep retuning; the milestone is not done until the answer is weighty (depends on T020, T024)
- [X] T028 [US4] Record the final tuned values: confirm `godot/data/tuning.json` holds them and that they pass the rules; update the starting values in `specs/004-m2b-hit-and-feel/contracts/tuning-data.md` to match, so the contract shows what shipped (depends on T027)

**Checkpoint**: The owner says the hit feels weighty.

---

## Phase 7: User Story 5 – Still 60 fps, clean and committed (Priority: P2)

**Goal**: 60 fps with five dummies and hitboxes; the Quality Gate is green.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 15, 17, 18 and 19.

- [X] T029 [US5] (owner) Quickstart scenario 15: press F3 four times (five dummies), F2 on, panel open; walk and swing for 60 s by the clock with the window focused. FPS stays 60 apart from the first second and the impact freezes (depends on T024)
- [X] T030 [US5] In `rust/`, run `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets` and `cargo test`; fix every warning in `rust/src/*.rs` until all pass with 0 warnings. No `#[allow]` without a written reason
- [X] T031 [US5] Check every GDScript file (`godot/world/dummy_flash.gd`, `godot/world/screen_shake.gd`, `godot/world/lane_band.gd`, `godot/debug/debug_overlay.gd`, `godot/debug/debug_markers.gd`, `godot/debug/debug_hitboxes.gd`, `godot/ui/fps_label.gd`) through the editor's language server, as in M2a's T023: 0 errors or warnings, `untyped_declaration` still Error
- [X] T032 [US5] After a build, a Play and the export, run `git status`: `rust/target/`, `godot/.godot/` and `godot/export/` absent; `rust/src/combat.rs`, `hit_stop.rs`, `training_dummy.rs`, the changed `hero.rs`, `tuning.rs`, `tuning_store.rs`, `lib.rs`, `godot/autoload/hit_stop.tscn`, `godot/world/training_dummy.tscn`, `flash.gdshader`, the new `.gd` files with their `.uid` files, `godot/main.tscn`, `godot/project.godot` and `godot/data/tuning.json` present to be committed

**Checkpoint**: The Quality Gate is green.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Constitution review and the milestone commit.

- [X] T033 [P] Review docs: `//!` module docs on `rust/src/combat.rs`, `hit_stop.rs`, `training_dummy.rs`; `///` on every public item and everything exposed to Godot, including `was_hit`; `// SAFETY:` in `rust/src/lib.rs` unchanged; `##` docs on every new script and exported property
- [X] T034 [P] Check the mode rules (FR-012): `grep -rnE 'is_attacking|is_swinging|attacking: bool' rust/src` returns nothing; every `match` on `HeroMode`, `SwingPhase` and `Facing` has no `_ =>` arm
- [X] T035 [P] Check the split (FR-011): no `.gd` file decides a hit, a swing time, a knockback or a freeze (they only read `Tuning`, `hero.attack_box()`, `hurtbox()`, `hits_landed()` and `dummy_spawn_point()`); `grep -rnE 'KEY_|keycode' godot/world godot/debug` returns nothing; `grep -rn '"p1_' rust/src` returns nothing
- [X] T036 [P] Check `Engine.time_scale` and pausing are unused: `grep -rnE 'time_scale|paused' rust/src godot/world godot/debug` returns nothing (research.md R5)
- [X] T037 [P] Check for magic numbers (Principle IV): none of the values in `godot/data/tuning.json` appears as a literal in `rust/src` (outside tests, whose values must differ) or in `godot/world` and `godot/debug` scripts; the only numbers in scripts are named presentation constants
- [X] T038 Run the Constitution compliance review required by Governance: check the finished work against Principles I–VIII (v1.3.3) and the Code Style section, using the table in `specs/004-m2b-hit-and-feel/plan.md` as the checklist; record any departure (for example scenes built by the agent) in that file's Complexity Tracking
- [X] T039 (owner) Run the full quickstart.md, scenarios 1–20, in one sitting on the target machine; scenario 20 re-checks M2a's F1, F5 and rejections
- [X] T040 Commit the M2b implementation on branch `feat/m2b-hit-and-feel` in small logical commits (for example: input actions; tuning values and tests; combat rules and tests; HitStop and autoload; dummy class; hero swing and hits; dummy scene, flash and shake; F2, F3 and the counter; final tuning values; task marks), staging by explicit path and reviewing `git diff --staged` before each, as the owner's git rules require. Ask the owner before committing; then `git status` must be clean (depends on T026–T039)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies. T002 is needed before T012 runs in the game.
- **Foundational (Phase 2)**: Needs T001. Blocks every story.
- **US1 (Phase 3)**: Needs Phase 2. The MVP.
- **US2 (Phase 4)**: Needs US1 (there must be a swing to tune).
- **US3 (Phase 5)**: Needs US1's `#[func]`s (T013); independent of US2.
- **US4 (Phase 6)**: Needs US2 (live tuning) and US3 (tools that help judge).
- **US5 (Phase 7)**: T029 needs US3 (F2, F3); T030–T032 need all code.
- **Polish (Phase 8)**: Needs all stories.

### Within Each Phase

- Tests before the code they test (T003 → T004, T005 → T006).
- Rust classes registered (T007, T011–T013) before scenes use them (T008, T017, T023).
- Scripts (T014–T016, T021–T022) before the scenes that use them.
- Play checks (owner) last in each story.

### Parallel Opportunities

- **Phase 2**: T003 (tuning tests) and T005 (combat tests) together; then T004 and T006 together
  (different files); T009 alongside T007.
- **US1**: T013's `#[func]`s and the three presentation files T014, T015, T016 in parallel.
- **US3**: T021 alongside T013's work.
- **Polish**: T033–T037 in parallel.

---

## Parallel Example: Phase 2 and User Story 1

```bash
# Tests first, together (different files):
Task: "Write the new tuning rule tests in rust/src/tuning.rs"
Task: "Write the combat tests in rust/src/combat.rs"

# Presentation files for US1, together:
Task: "Create godot/world/flash.gdshader"
Task: "Create godot/world/dummy_flash.gd"
Task: "Create godot/world/screen_shake.gd"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: actions. 2. Phase 2: tuning, combat rules, `HitStop` (game still plays as M2a).
3. Phase 3: US1. The hero hits the dummy with all four effects.
4. **STOP and VALIDATE**: quickstart 1–7. The first impression of the hit can be judged here.

### Incremental Delivery

1. Setup + Foundational → tests pass, M2a unchanged (session 1)
2. US1 → hitting works (session 2) → MVP
3. US2 → live, per-effect tuning
4. US3 → F2, F3, counter
5. US4 → weighty, by the owner's judgement
6. US5 + Polish → 60 fps, gate green, committed (session 3) → M2b done

### If a Session Stalls

Shrink the next step (constitution, Development Workflow): for example, get the swing landing with
only the knockback, then add the freeze, then the flash, then the shake, each behind its zero
switch. Never put a tuning value in code as a shortcut. Every checkpoint is a safe place to stop.

---

## Notes

- [P] = different files, no dependency on unfinished tasks.
- (owner) tasks need the Godot editor GUI or a judgement made by playing.
- Commit only at the end, with the owner's go-ahead (T040).
- Godot creates a `*.uid` file next to each new script and shader. Commit it; it is not cache.
