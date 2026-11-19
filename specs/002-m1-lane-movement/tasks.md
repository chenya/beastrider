---

description: "Task list for M1 Lane Movement"
---

# Tasks: M1 Lane Movement

**Input**: Design documents from `specs/002-m1-lane-movement/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Rust unit tests are included, because the owner asked for them in the plan input: the
pure movement step ([research.md R9](research.md)) and the tuning checks. They are written before
the code they test. Everything else is verified by playing, using the
[quickstart.md](quickstart.md) scenarios.

**Organization**: Tasks are grouped by user story so each can be checked on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on unfinished tasks)
- **[Story]**: Which user story the task belongs to (US1–US4 from spec.md)
- **(owner)**: Needs the owner, because it uses the Godot editor GUI (the constitution says
  scenes are built in the editor) or a judgement made by playing. The owner may ask the agent to
  do an editor task instead, using the godot-mcp tools or a headless Godot script.

## Path Conventions

The same two folders as M0 ([plan.md](plan.md)): `rust/` (the crate) and `godot/` (the project).
Run Rust commands from `rust/`. Starting values in tests must differ from those in
`godot/data/tuning.json` (300, 180, 430, 700), so the magic-number check in T031 stays meaningful.

## Session plan (about 15 h/week, Principle II)

- **Session 1**: Phases 1–2 (input actions, movement core and tuning with their tests, M0 removed)
- **Session 2**: Phase 3 (US1: the hero walks the lane). Ends playable.
- **Session 3**: Phases 4–7 (pillars, tuning checks, 60 fps, Quality Gate, review, commit)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the M0 toolchain is unchanged and add player 1's input actions.

- [ ] T001 Confirm nothing was upgraded (research.md R1): `godot --version` prints `4.7.2.stable.official.ed1daf0bf`; `rust/Cargo.toml` still has `godot = { version = "=0.5.5", features = ["api-4-6"] }`; `rust/rust-toolchain.toml` still pins `1.100.0`; and `cargo build` in `rust/` passes. If anything differs, stop and ask the owner
- [ ] T002 (owner) In Project Settings → Input Map, add four actions bound to **physical** keys and save, so they are written to `godot/project.godot`: `p1_left` = Left arrow + A, `p1_right` = Right arrow + D, `p1_up` = Up arrow + W, `p1_down` = Down arrow + S ([contracts/hero.md](contracts/hero.md#input-actions-input-map-project-settings), research.md R5). Then confirm `godot/project.godot` has an `[input]` section with exactly these four actions and that `git diff godot/project.godot` shows no other change

**Checkpoint**: The toolchain is as in M0 and the Input Map has player 1's actions.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The pure movement step and the new tuning, both under unit tests, and M0's square
removed. Every user story needs these.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T003 [P] Write the unit tests first, in a `#[cfg(test)] mod tests` at the bottom of a new `rust/src/movement.rs` (they will not compile until T004). Cover, with speeds and limits different from the tuning file's starting values: (1) zero input returns the position unchanged when inside the limits; (2) a unit diagonal (0.70710677, -0.70710677) for 1 s covers no more distance than input (1, 0) for 1 s, with V < H, and no more than the faster straight direction with V > H; (3) with V < H, input (0, 1) for 1 s covers less distance than (1, 0) for 1 s; (4) one test per limit (min_x, max_x, min_y, max_y): moving into it lands exactly on it; (5) a position on a limit with input into it stays bit-for-bit unchanged over 100 ticks; (6) a delta of 1000 s with diagonal input still ends inside all four limits; (7) a position outside the limits with zero input is moved to the nearest point inside
- [ ] T004 Implement `rust/src/movement.rs` so T003's tests pass. Contents: a `//!` module doc; `pub struct Speeds { pub horizontal: f32, pub vertical: f32 }`; `pub struct Limits { pub min_x: f32, pub max_x: f32, pub min_y: f32, pub max_y: f32 }`; and `pub fn step(position: (f32, f32), input: (f32, f32), speeds: Speeds, delta: f32, limits: Limits) -> (f32, f32)` computing `moved = position + (input.0 × horizontal, input.1 × vertical) × delta` and then clamping each axis to its limits (data-model.md, "Movement step"). No Godot types in this file. `///` docs on every public item. Add `mod movement;` to `rust/src/lib.rs` and run `cargo test` (depends on T003)
- [ ] T005 [P] Write the tuning tests first, replacing M0's `SquareTuning` tests in `rust/src/tuning.rs` (they will not compile until T006). Cover `MovementTuning::try_new(horizontal_speed, vertical_speed, lane_top, lane_bottom)`: accepts valid values; rejects each speed at `0`, negative, NaN and infinity; rejects `lane_top < 0`, `lane_top == lane_bottom` and `lane_top > lane_bottom`; rejects a non-finite lane value; and the message of a rejection contains `res://data/tuning.json`, the field path (`hero.vertical_speed`, `lane.top`) and the bad value. Also cover `check_lane_fits(&tuning, screen_height)`: accepts `lane_bottom == screen_height`, rejects `lane_bottom > screen_height` with a message naming `lane.bottom`
- [ ] T006 In `rust/src/tuning.rs`, replace `SquareTuning` and `load_square_tuning` with `MovementTuning` (fields `horizontal_speed`, `vertical_speed`, `lane_top`, `lane_bottom`, private, with `///`-documented accessors of the same names) and `pub fn load_movement_tuning() -> Result<MovementTuning, TuningError>`, so T005's tests pass. Rules, from data-model.md: `hero.horizontal_speed` "finite, `> 0` (px/s)"; `hero.vertical_speed` "finite, `> 0` (px/s)"; `lane.top` "finite, `>= 0`, `< lane.bottom`"; `lane.bottom` "finite, `> lane.top`, `<= viewport height`" (the last rule as `pub fn check_lane_fits(tuning: &MovementTuning, screen_height: f32) -> Result<(), TuningError>`, called by the hero). Keep `TuningError`, `read_json_object`, `object_field`, `number_field` and `required` from M0; replace the `drifting_square`/`speed` constants with `HERO_SECTION`, `LANE_SECTION` and the four field keys. No default values anywhere (depends on T005)
- [ ] T007 [P] Replace the contents of `godot/data/tuning.json` with the M1 shape from [contracts/tuning-data.md](contracts/tuning-data.md): `hero.horizontal_speed` 300, `hero.vertical_speed` 180, `lane.top` 430, `lane.bottom` 700, two-space indentation, and no `drifting_square` section
- [ ] T008 Remove M0's square (research.md R10, FR-014): delete `rust/src/drifting_square.rs` and its `mod drifting_square;` line in `rust/src/lib.rs`; delete `godot/ui/speed_label.gd` and `godot/ui/speed_label.gd.uid`. `godot/main.tscn` will reference missing items until T013 rebuilds it; that is expected. Run `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets` and `cargo test` (dead-code warnings for the not-yet-used loader and `step` are acceptable until T010) (depends on T004, T006)

**Checkpoint**: `cargo test` passes for movement and tuning; M0's square is gone from the code.

---

## Phase 3: User Story 1 – Walk the hero around the lane (Priority: P1) 🎯 MVP

**Goal**: The hero walks the lane with instant start and stop, normalised diagonals, slower depth
movement and hard limits.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 1–6 (pillars may still be
missing; they come in US2).

- [ ] T009 [US1] Create `rust/src/hero.rs` implementing [contracts/hero.md](contracts/hero.md): `#[derive(GodotClass)] #[class(base = Node2D)] pub struct Hero` with `#[export] player_index: i32` (default 1, data-model.md: "`>= 1`"), `actions: Option<PlayerActions>`, `tuning: Option<MovementTuning>`, `limits: Option<Limits>`, `base: Base<Node2D>`; a hand-written `init` (as in M0, to avoid clippy's `redundant_field_names`); a private `struct PlayerActions { left, right, up, down: StringName }` built once from `player_index` as `p{index}_left` and so on. In `ready()`, in this order, stopping at the first failure with one `godot_error!("Hero: {message}")`: check `player_index >= 1`; build the actions and check each with `InputMap::singleton().has_action`, naming the action and the index if missing; call `load_movement_tuning()`; call `check_lane_fits` with the viewport height; get `Body` with `try_get_node_as::<Polygon2D>("Body")` and measure its half width from the polygon's min and max x plus `Body`'s x position, requiring it `> 0`; then build `Limits { min_x: half_width, max_x: viewport_width - half_width, min_y: lane_top, max_y: lane_bottom }`. `///` docs on the class and every exposed item; a module `//!` doc
- [ ] T010 [US1] In `rust/src/hero.rs`, implement `physics_process(delta)`: when active, read `Input::singleton().get_vector(left, right, up, down)` on the cached actions, call `movement::step` with the position, the input, `Speeds` from the tuning, `delta as f32` and the limits, and set the new position; when stopped, do nothing. Add `#[func] pub fn lane_top(&self) -> f32` and `#[func] pub fn lane_bottom(&self) -> f32`, returning `0.0` when stopped (contracts/hero.md). Add `mod hero;` to `rust/src/lib.rs`, then run `cargo build`, `cargo clippy --all-targets` (0 warnings) and `cargo test` (depends on T009)
- [ ] T011 [US1] Confirm with a headless Godot script that `Hero` is registered with parent class `Node2D` and has the methods `lane_top` and `lane_bottom` and the property `player_index` (FR-010, contracts/hero.md) (depends on T010)
- [ ] T012 [US1] Create `godot/world/lane_band.gd`: `extends ColorRect`, a `##` class doc comment, `## The hero whose lane limits this band shows. Set in the editor.` above `@export var hero: Hero`, and `_ready()` that does `if not hero.is_node_ready(): await hero.ready`, then sets `position = Vector2(0.0, hero.lane_top())` and `size = Vector2(get_viewport_rect().size.x, hero.lane_bottom() - hero.lane_top())`. No other variables; tabs; typed (research.md R8, FR-008) (depends on T011)
- [ ] T013 [US1] (owner) In the Godot editor, rebuild `godot/main.tscn` to the tree in [plan.md](plan.md#source-code-repository-root), without pillars yet: root `Node2D` `Main`; first child `ColorRect` `LaneBand` (a lighter colour than the background, for example `#2a2a33`, script `res://world/lane_band.gd`, `hero` → `World/Hero`); then `Node2D` `World` with **Y Sort Enabled** on; inside it `Hero` (`player_index` 1, position inside the lane, for example (640, 560)) with child `Polygon2D` `Body` whose polygon is `(-24, -96), (24, -96), (24, 0), (-24, 0)` in a bright colour (for example `#E8792B`); then `CanvasLayer` `Hud` with `Label` `FpsLabel` (top-right, 16 px margin, `res://ui/fps_label.gd`). Remove the old `DriftingSquare` and `SpeedLabel` nodes. Save (depends on T002, T007, T008, T012)
- [ ] T014 [US1] (owner) Press Play and run quickstart scenarios 1–6: window, band, hero and FPS counter; instant start and stop; diagonal no faster; up and down slower; limits with no shaking and sliding along them; opposite keys, both key sets, and focus loss. No errors in the Output panel. Fix anything that fails in `rust/src/hero.rs` or `rust/src/movement.rs` before going on (depends on T013)

**Checkpoint**: The hero walks the lane. M1's MVP is playable.

---

## Phase 4: User Story 2 – Walk in front of and behind pillars (Priority: P1)

**Goal**: The hero and the pillars overlap in depth order, with no flicker.

**Independent Test**: [quickstart.md](quickstart.md) scenario 7.

- [ ] T015 [US2] (owner) In `godot/main.tscn`, add three `Polygon2D` pillars as children of `World`, **before** `Hero` in the tree (so the hero stays last and wins ties, research.md R2): each with its origin at its base and a polygon above it, for example `(-32, -260), (32, -260), (32, 0), (-32, 0)`, in a placeholder colour different from the hero's. Place their bases at three different depths inside the lane, for example y = 470, 560 and 660, spread across the screen. Save (clarification Q2: layout lives in the scene) (depends on T013)
- [ ] T016 [US2] (owner) Press Play and run quickstart scenario 7 for each pillar: the pillar covers the hero when the hero's feet are above its base and the hero covers the pillar when below; the switch happens once per crossing, never flickers, and with the feet exactly on the base line the hero is in front. If the order is wrong, check the three points in quickstart "If something goes wrong" (depends on T015)

**Checkpoint**: Depth sorting is proven.

---

## Phase 5: User Story 3 – Tune movement and the lane without recompiling (Priority: P2)

**Goal**: The four tuning values drive the game, with clear errors for bad values.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 8 and 9.

- [ ] T017 [US3] (owner) Quickstart scenario 8: double `hero.horizontal_speed`, halve `hero.vertical_speed` and add 50 to `lane.top` in `godot/data/tuning.json`; do **not** run cargo; press Play. Sideways is twice as fast, up and down half as fast, and both the band's top edge and the hero's upper limit moved down 50 px (depends on T014)
- [ ] T018 [US3] (owner) Quickstart scenario 9: one at a time, set a speed to `0`, `-5` and `"fast"`; delete `lane.top`; set `lane.top` above `lane.bottom`; set `lane.bottom` to `800`. Press Play after each. Each time: exactly one error starting `Hero:` that names the file, field and problem; the hero stays still; the band is not drawn; no crash. If a message is unclear, fix its text in `rust/src/tuning.rs` or `rust/src/hero.rs` and re-run `cargo test` (depends on T014)
- [ ] T019 [US3] Restore `godot/data/tuning.json` to the values in T007 and confirm `git diff godot/data/tuning.json` shows no change from T007 (depends on T017, T018)

**Checkpoint**: Principle IV holds for movement.

---

## Phase 6: User Story 4 – Still 60 fps, clean and committed (Priority: P2)

**Goal**: The frame rate holds while walking, and the Quality Gate is green.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 10, 12, 13 and 14.

- [ ] T020 [US4] (owner) Quickstart scenario 10: walk the hero continuously for 60 s by the clock with the window in front and focused. The FPS counter shows 60 throughout (the first second excepted) and movement looks even. If there is a regular hitch, follow research.md R4 (turn on physics interpolation) and record the change in `specs/002-m1-lane-movement/research.md` (depends on T016)
- [ ] T021 [US4] In `rust/`, run `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets` and `cargo test`. Fix every warning in `rust/src/*.rs` until all four pass with 0 warnings (SC-008). Do not silence lints with `#[allow]` unless the reason is written next to it
- [ ] T022 [US4] Check every GDScript file (`godot/world/lane_band.gd`, `godot/ui/fps_label.gd`) through the editor's language server, as in M0's T029: 0 errors or warnings, with `untyped_declaration` still Error (quickstart scenario 13)
- [ ] T023 [US4] After a build and a Play, run `git status`. Confirm `rust/target/` and `godot/.godot/` do not appear, `rust/src/drifting_square.rs` and `godot/ui/speed_label.gd` are deleted, and `rust/src/hero.rs`, `rust/src/movement.rs`, `godot/world/lane_band.gd` with its `.uid`, `godot/main.tscn`, `godot/project.godot` and `godot/data/tuning.json` are all present to be committed (quickstart scenario 14)

**Checkpoint**: The Quality Gate is green.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Constitution review and the milestone commit.

- [ ] T024 [P] Review docs: `//!` module docs on `rust/src/movement.rs` and `rust/src/hero.rs`; `///` on every public item in `rust/src/movement.rs`, `rust/src/tuning.rs` and `rust/src/hero.rs` and on everything exposed to Godot (comments say why, not what); the `// SAFETY:` comment in `rust/src/lib.rs` unchanged; `##` doc comments on `godot/world/lane_band.gd` and its exported `hero`
- [ ] T025 [P] Check for magic numbers (FR-012, Principle IV): `grep -rnE '\b(300|180|430|700)\b' rust/src godot/world godot/ui` returns nothing. The speeds and lane limits exist only in `godot/data/tuning.json`
- [ ] T026 [P] Check the Principle VII rule: `grep -rn '"p1_' rust/src` returns nothing (action names are built from `player_index`), and no code refers to a single global player
- [ ] T027 Run the Constitution compliance review required by Governance: check the finished work against Principles I–VIII and the Code Style section, using the table in `specs/002-m1-lane-movement/plan.md` as the checklist. Any departure goes into that file's Complexity Tracking section
- [ ] T028 (owner) Run the full quickstart.md, scenarios 1–14, in one sitting on the target machine, and after a few minutes of walking confirm SC-009: "moving feels immediate and the depth looks right"
- [ ] T029 Commit the M1 implementation on branch `feat/m1-lane-movement` in small logical commits (for example: input actions; movement core and tests; tuning change and tests; M0 removal; Hero; lane band; scene; task marks), staging by explicit path and reviewing `git diff --staged` before each commit, as the owner's git rules require. Ask the owner before committing. Then `git status` must be clean (depends on T020–T028)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies. T002 can happen any time before T013.
- **Foundational (Phase 2)**: Needs T001. Blocks every story.
- **US1 (Phase 3)**: Needs Phase 2. This is the MVP.
- **US2 (Phase 4)**: Needs US1's scene (T013).
- **US3 (Phase 5)**: Needs US1 (T014). Independent of US2.
- **US4 (Phase 6)**: T020 needs US2 (the pillars are part of the frame being measured). T021–T023
  need only US1.
- **Polish (Phase 7)**: Needs all stories.

### User Story Dependencies

- **US1 (P1)**: Needs only Foundational.
- **US2 (P1)**: Adds pillars to US1's scene; no Rust change.
- **US3 (P2)**: Verifies US1's tuning; independent of US2.
- **US4 (P2)**: Final checks; the commit (T029) waits for all stories.

### Within Each Story

- Tests come before the code they test (T003 → T004, T005 → T006).
- `Hero` must be built and registered (T011) before GDScript can use it as a type (T012).
- The band script (T012) must exist before the scene uses it (T013).
- Play checks (owner) come last in each story.

### Parallel Opportunities

- **Phase 2**: T003 (movement tests), T005 (tuning tests) and T007 (JSON file) touch different
  files and can run together. T004 and T006 then run in parallel too (different files).
- **US1**: none; each task builds on the previous one.
- **US2 / US3 / US4 play checks**: all owner play sessions on the same machine, so do them one
  after another in a single sitting.
- **Polish**: T024, T025 and T026 can run in parallel.

---

## Parallel Example: Phase 2

```bash
# Tests and data first, all at once (different files):
Task: "Write movement tests in rust/src/movement.rs"
Task: "Write MovementTuning tests in rust/src/tuning.rs"
Task: "Replace godot/data/tuning.json with the hero and lane sections"

# Then both implementations at once:
Task: "Implement movement::step in rust/src/movement.rs"
Task: "Implement MovementTuning and load_movement_tuning in rust/src/tuning.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: Setup. Input actions in place.
2. Phase 2: Foundational. Movement and tuning tested; M0's square gone.
3. Phase 3: US1. The hero walks the lane.
4. **STOP and VALIDATE**: quickstart scenarios 1–6. Movement feel can be judged from here.

### Incremental Delivery

1. Setup + Foundational → tests pass (end of session 1)
2. Add US1 → playable (end of session 2) → MVP
3. Add US2 → depth sorting proven
4. Add US3 → tuning and error paths proven
5. Add US4 + Polish → 60 fps, Quality Gate green, committed (end of session 3) → M1 done

### If a Session Stalls

Shrink the next step instead of pushing through (constitution, Development Workflow). For example,
if the hero misbehaves, get it moving with the lane limits only (y clamped, x free) first, and
add the `Body` measurement and the x limits afterwards. Never put a tuning value in code as a
shortcut (Principle IV). Every checkpoint above is a safe place to stop.

---

## Notes

- [P] = different files, no dependency on unfinished tasks.
- (owner) tasks need the Godot editor GUI or a judgement made by playing.
- Commit only at the end, with the owner's go-ahead (T029), following the owner's git rules.
- Godot creates a `*.uid` file next to each new script. Commit it; it is not cache.
