---

description: "Task list for M2a Debug Overlay"
---

# Tasks: M2a Debug Overlay

**Input**: Design documents from `specs/003-m2a-debug-overlay/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Rust unit tests are included, because the owner asked for them in the plan input: the
pure `validate` function, with one passing and one failing case per rule
([research.md R3](research.md)). They are written before the code they test. M1's movement tests
stay unchanged. Everything else is verified by playing, using the [quickstart.md](quickstart.md)
scenarios.

**Organization**: Tasks are grouped by user story so each can be checked on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on unfinished tasks)
- **[Story]**: Which user story the task belongs to (US1–US4 from spec.md)
- **(owner)**: Needs the owner, because it uses the Godot editor GUI (the constitution says
  scenes are built in the editor) or a judgement made by playing. The owner may hand an editor
  task to the agent, which then uses a headless Godot script (`PackedScene.pack` +
  `ResourceSaver.save`) or `ProjectSettings` (with `event.device = -1`), as recorded in the
  plan's Complexity Tracking.

## Path Conventions

The same two folders as M0 and M1 ([plan.md](plan.md)): `rust/` (the crate) and `godot/` (the
project). Run Rust commands from `rust/`. Values in tests must differ from those in
`godot/data/tuning.json` (300, 180, 430, 700), so the magic-number check in T026 stays
meaningful. Size- and focus-dependent checks need a windowed run, not `--headless` (the headless
window is 64×64, which the lane rule rejects).

## Session plan (about 15 h/week; the brief sizes M2a at about one session)

- **Session 1**: Phases 1–3 (actions, `TuningStore` with tests, hero and band moved to it, the
  overlay panel and markers). Ends playable with F1 working.
- **Session 2** (if needed): Phases 4–7 (F5 reload, release check, 60 fps, Quality Gate, review,
  commit).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm nothing was upgraded and add the two debug actions.

- [X] T001 Confirm nothing was upgraded (research.md R1): `godot --version` prints `4.7.2.stable.official.ed1daf0bf`; `rust/Cargo.toml` still has `godot = { version = "=0.5.5", features = ["api-4-6"] }`; `rust/rust-toolchain.toml` still pins `1.100.0`; `~/.local/share/godot/export_templates/4.7.2.stable/linux_release.x86_64` exists; and `cargo build` and `cargo test` in `rust/` pass (18 tests). If anything differs, stop and ask the owner
- [X] T002 (owner) In Project Settings → Input Map, add two actions bound to **physical** keys, deadzone 0.2, and save, so they are written to `godot/project.godot`: `debug_toggle_overlay` = F1, `debug_reload_tuning` = F5 ([contracts/debug-overlay.md](contracts/debug-overlay.md#input-map-actions-project-settings--input-map), FR-012). Then confirm `godot/project.godot` lists them beside the four `p1_*` actions with `"device":-1`, and that `git diff godot/project.godot` shows no other change

**Checkpoint**: The toolchain is as before and the Input Map has the debug actions.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: One source of truth for tuning (the problem this milestone solves, plan.md): the
pure `validate` under tests, the `TuningStore` class as the `Tuning` autoload, and the hero and
lane band reading from it. Every user story needs this.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete. At the end the game
must play exactly as M1.

- [X] T003 Write the `validate` tests first, added to the `#[cfg(test)] mod tests` of `rust/src/tuning.rs` (they will not compile until T004; the old `MovementTuning` tests go in T007). Use a valid `RawTuning { horizontal_speed: 250.0, vertical_speed: 150.0, lane_top: 400.0, lane_bottom: 650.0 }` and screen height `720.0`. One passing and one failing case for each rule in data-model.md: `hero.horizontal_speed` "finite, `> 0`" (fail: `0.0`, `-5.0`, NaN, infinity); `hero.vertical_speed` "finite, `> 0`" (same fails); `lane.top` "finite, `>= 0`" (pass `0.0`, fail `-1.0` and NaN); `lane.bottom` "finite" (fail infinity); `lane.top` "`< lane.bottom`" (fail equal and greater); `lane.bottom` "`<= screen height`" (pass equal, fail `721.0` against `720.0`). Also: a huge finite value that overflows `f32` (`1e300`) is rejected; a rejection's `field()` is the dotted path (`lane.top`, `hero.vertical_speed`) and its `reason()` contains the rule and the bad value; `to_string()` still contains `res://data/tuning.json`, the field path and the value
- [X] T004 In `rust/src/tuning.rs`, add (data-model.md), keeping `MovementTuning`, `check_lane_fits` and `load_movement_tuning` for now so `hero.rs` still builds (T007 removes them): `pub struct RawTuning { pub horizontal_speed: f64, pub vertical_speed: f64, pub lane_top: f64, pub lane_bottom: f64 }`; `#[derive(Clone, Copy, Debug, PartialEq)] pub struct TuningValues` with private `f32` fields and `///`-documented accessors `horizontal_speed()`, `vertical_speed()`, `lane_top()`, `lane_bottom()`; `pub fn validate(raw: &RawTuning, screen_height: f32) -> Result<TuningValues, TuningError>` applying the M1 rules in the order speeds, bottom, top, top < bottom, bottom <= screen height; and `pub fn read_raw_tuning() -> Result<RawTuning, TuningError>` (the existing file reading, JSON parsing and field lookup, now returning raw numbers). Add `pub fn field(&self) -> &str` and `pub fn reason(&self) -> String` to `TuningError`: `field()` is the field path, or the file path for `Unreadable` and `InvalidJson`; `reason()` is the message without the file and field (for `InvalidValue`, the rule and the bad value; for `MissingField`, `missing`). Keep `Display` as the full one-line message. No defaults, no panics on data. Run `cargo test` until T003 passes (depends on T003)
- [X] T005 Create `rust/src/tuning_store.rs` implementing [contracts/tuning-store.md](contracts/tuning-store.md): `#[derive(GodotClass)] #[class(base = Node)] pub struct TuningStore { values: Option<TuningValues>, rejection: Option<TuningError>, base: Base<Node> }` with a hand-written `init`; `ready()` calls a private `load(&mut self) -> Result<(), TuningError>` that does `read_raw_tuning()` then `validate(&raw, viewport height)` and swaps `values` only on `Ok` (all or nothing, research R3), setting or clearing `rejection` and writing `godot_error!("Tuning: {error}")` on failure; `#[func] pub fn reload(&mut self)` calls `load` and then emits `tuning_reloaded` on success or `tuning_rejected(field, reason)` on failure through `self.signals()` (research R5); `#[signal] fn tuning_reloaded();` and `#[signal] fn tuning_rejected(field: GString, reason: GString);`; `#[func]` accessors `has_values()`, `horizontal_speed()`, `vertical_speed()`, `lane_top()`, `lane_bottom()` (each `0.0` when `values` is `None`), `rejected_field()` and `rejected_reason()` (`""` when `rejection` is `None`); and a Rust-only `pub fn values(&self) -> Option<TuningValues>`. `//!` module doc and `///` docs on the class, every `#[func]` and both signals. Add `mod tuning_store;` to `rust/src/lib.rs`. Run `cargo build` and `cargo clippy --all-targets` (dead-code warnings for unused accessors are acceptable until T007) (depends on T004)
- [X] T006 (owner) Create `godot/autoload/tuning.tscn` with one root node of type `TuningStore` named `Tuning`, and in Project Settings → Autoload add it as `Tuning` (Global Variable on), so `godot/project.godot` gains `[autoload]` with `Tuning="*res://autoload/tuning.tscn"`. The autoload name must not equal any class name (research R2). Save (depends on T005)
- [X] T007 In `rust/src/hero.rs`, implement the hero changes in [contracts/tuning-store.md](contracts/tuning-store.md#hero-changes) and data-model.md: replace `Active` with `Setup { actions: PlayerActions, half_width: f32, tuning: Gd<TuningStore> }` and the field `active` with `setup: Option<Setup>`. In `ready()`, check in order: `player_index >= 1`; the input actions; `Body` half width `> 0`; and `/root/Tuning` with `try_get_node_as::<TuningStore>`, failing with `Hero: the Tuning autoload is missing or is not a TuningStore`. No tuning is loaded here any more. In `physics_process`, if `setup` is `Some` and `setup.tuning.bind().values()` is `Some`, build `Speeds` and `Limits { min_x: half_width, max_x: viewport_width - half_width, min_y: lane_top, max_y: lane_bottom }` from them and call `movement::step` as before; otherwise do nothing that tick (FR-010: a hero without a `Setup` never moves). Remove `lane_top()` and `lane_bottom()` from `Hero`. Then delete the now-unused `MovementTuning`, `check_lane_fits`, `load_movement_tuning` and their tests from `rust/src/tuning.rs`. Update the `//!` and `///` docs. Run `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets` (0 warnings) and `cargo test` (depends on T005)
- [X] T008 [P] In `godot/world/lane_band.gd`, drop the `hero` export and the `await hero.ready`; add a private `_place()` that sets `position = Vector2(0.0, Tuning.lane_top())` and `size = Vector2(get_viewport_rect().size.x, Tuning.lane_bottom() - Tuning.lane_top())`; call it from `_ready()` and connect `Tuning.tuning_reloaded` to it (research R9). Update the `##` class doc (the band reads `Tuning`; zero height while `Tuning.has_values()` is false). Then (owner) in `godot/main.tscn` clear the `LaneBand` node's now-unused `hero` property so the scene loads without warnings. Save (depends on T006)
- [X] T009 Confirm with a windowed Godot script run (1280×720) that: `TuningStore` is registered with parent class `Node` and has the methods and both signals in contracts/tuning-store.md; `/root/Tuning` exists and is a `TuningStore` before `Main` is ready; `Tuning.has_values()` is true and returns 300, 180, 430, 700; the band's top and height match 430 and 270; the hero moves under simulated `p1_right` input; and the Output has no errors (depends on T007, T008)

**Checkpoint**: The game plays exactly as M1, but every tuning value now comes from `Tuning`.

---

## Phase 3: User Story 1 – See what the game is doing (Priority: P1) 🎯 MVP

**Goal**: F1 shows and hides a live panel, the lane bars and the origin cross.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 1–4.

- [X] T010 [P] [US1] Create `godot/debug/debug_markers.gd` (research R8): `extends Node2D`, `##` class doc; constants `BAR_THICKNESS := 2.0`, `CROSS_HALF_SIZE := 6.0` and a marker colour; `## The hero whose origin the cross marks. Set in the editor.` above `@export var hero: Hero`; `var _drawn_position: Vector2`. `_ready()` connects `Tuning.tuning_reloaded` to `queue_redraw`. `_process()` calls `queue_redraw()` only when `visible` and `hero.position != _drawn_position`. `_draw()` stores `_drawn_position = hero.position`; draws two bars across the viewport width straddling `Tuning.lane_top()` and `Tuning.lane_bottom()` (rows `y - 1` and `y`, so the hero's bottom edge meets the middle), but only when `Tuning.has_values()`; and a cross of `CROSS_HALF_SIZE` at `hero.position`. Typed, tabs, code order per the constitution (depends on T009)
- [X] T011 [P] [US1] Create `godot/debug/debug_overlay.gd` (research R7, [contracts/debug-overlay.md](contracts/debug-overlay.md)): `extends CanvasLayer`, `##` class doc; `## The hero whose values the panel shows. Set in the editor.` above `@export var hero: Hero`; `## The world markers shown and hidden with the panel. Set in the editor.` above `@export var markers: Node2D`; `@onready` typed references to `Panel` and its six line labels. `_ready()` hides the panel and the markers. `_unhandled_input(event)` toggles both when `event.is_action_pressed("debug_toggle_overlay")` and marks the event handled. `_process()` returns early when the panel is hidden; otherwise sets `FPS: %d` from `Engine.get_frames_per_second()`, `Position: (%d, %d)` from `hero.position` (rounded), `Player: %d` from `hero.player_index`, `Horizontal speed: %d px/s` and `Vertical speed: %d px/s` from `Tuning`, and `Tuning: ok` or `Tuning: rejected (<field>)` from `Tuning.rejected_field()`. No game state stored (FR-011). Typed, tabs, code order per the constitution (depends on T009)
- [X] T012 [US1] (owner) In `godot/main.tscn`, add the nodes in [contracts/debug-overlay.md](contracts/debug-overlay.md#scene-tree-of-maintscn): `DebugMarkers` (`Node2D`, after `World`, `z_index` 1, script `res://debug/debug_markers.gd`, `hero` → `World/Hero`); `DebugOverlay` (`CanvasLayer`, layer 2, after `Hud`, script `res://debug/debug_overlay.gd`, `hero` → `World/Hero`, `markers` → `DebugMarkers`) with children `Panel` (`PanelContainer`, top-left, 16 px margin, a translucent dark style, mouse filter Ignore) → `Lines` (`VBoxContainer`) → six `Label`s `FpsLine`, `PositionLine`, `PlayerLine`, `HorizontalSpeedLine`, `VerticalSpeedLine`, `TuningLine`; and `Message` (`Label`, top centre, empty, mouse filter Ignore). Leave `Hud/FpsLabel` as it is (clarification Q2). Save (depends on T010, T011)
- [X] T013 [US1] (owner) Press Play and run quickstart scenarios 1–4: F1 ten times; live values; the hero's feet on the bars at the lane top and bottom; the cross at the feet, visible behind pillars. No errors in the Output panel. Fix anything that fails in `godot/debug/*.gd` before going on (depends on T012)

**Checkpoint**: F1 works. The MVP of M2a is playable.

---

## Phase 4: User Story 2 – Retune without restarting (Priority: P1)

**Goal**: F5 applies a good file at once, or rejects a bad one with a message and changes nothing.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 5–11.

- [X] T014 [US2] In `godot/debug/debug_overlay.gd` (research R5, R7): add `const RELOADED_MESSAGE_SECONDS := 2.0` and the text `Tuning reloaded` as a constant; `@onready` reference to `Message`; in `_unhandled_input`, call `Tuning.reload()` when `event.is_action_pressed("debug_reload_tuning")` (default no echo, so holding F5 reloads once), whether or not the panel is open, and mark the event handled; connect `Tuning.tuning_reloaded` to show `Tuning reloaded` and clear it after `RELOADED_MESSAGE_SECONDS` with a one-shot timer (a newer reload restarts it); connect `Tuning.tuning_rejected(field, reason)` to show `Tuning rejected: <field>: <reason>` with no timeout, cancelling any pending clear (FR-008: it stays until the next reload); in `_ready`, if `Tuning.rejected_field()` is not empty, show the start-up rejection the same way (spec edge case) (depends on T013)
- [X] T015 [US2] (owner) Press Play and run quickstart scenarios 5–9 and 11: double `hero.horizontal_speed` and F5; halve `hero.vertical_speed`, move `lane.top` and `lane.bottom`, F5 after each; `lane.top` 800; each bad value in scenario 8; restore and F5; hold F5, no-change F5, quick F1. The output panel shows one `Tuning:` error per rejection. If a message is unclear, fix its text in `rust/src/tuning.rs` and rerun `cargo test` (depends on T014)
- [X] T016 [US2] (owner) Quickstart scenario 10: start with `hero.vertical_speed` 0, then fix and F5 (the hero starts walking); then remove `p1_left` from the Input Map, Play, F5 (the hero stays stopped with a `Hero:` error); restore it (depends on T014)
- [X] T017 [US2] Restore `godot/data/tuning.json` and the Input Map; confirm `git diff godot/data/tuning.json` is empty and `git diff godot/project.godot` shows only T002's and T006's additions (depends on T015, T016)

**Checkpoint**: Live retuning works and bad values never reach the game.

---

## Phase 5: User Story 3 – Debug tools stay out of the released game (Priority: P2)

**Goal**: A release export has no overlay, markers, message or reload.

**Independent Test**: [quickstart.md](quickstart.md) scenario 12.

- [X] T018 [US3] In `godot/debug/debug_overlay.gd` `_ready()`, as its first step: if `not OS.is_debug_build()`, call `markers.queue_free()` and `queue_free()` and return (research R10, FR-013). Nothing in `rust/src/` may refer to the overlay or the markers (depends on T014)
- [X] T019 [US3] Build and export a release: `cd rust && cargo build --release`, then `godot --headless --path godot --export-release Linux export/beastrider.x86_64`. Confirm `godot/export/` holds `beastrider.x86_64`, `beastrider.pck` and `libbeastrider.so`, and that a 5-second run prints no errors (depends on T018)
- [X] T020 [US3] (owner) Quickstart scenario 12: run `godot/export/beastrider.x86_64`, walk, press F1 and F5 several times. No panel, bars, cross or message; the game plays as in M1 (depends on T019)

**Checkpoint**: Debug tools exist only in debug builds.

---

## Phase 6: User Story 4 – Still 60 fps, clean and committed (Priority: P2)

**Goal**: The frame rate holds with the overlay open, and the Quality Gate is green.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 13, 15, 16 and 17.

- [X] T021 [US4] (owner) Quickstart scenario 13: open the panel and walk continuously for 60 s by the clock, window in front and focused. FPS shows 60 throughout (the first second excepted) (depends on T013)
- [X] T022 [US4] In `rust/`, run `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets` and `cargo test`. Fix every warning in `rust/src/*.rs` until all four pass with 0 warnings (SC-008). No `#[allow]` without a written reason
- [X] T023 [US4] Check every GDScript file (`godot/debug/debug_overlay.gd`, `godot/debug/debug_markers.gd`, `godot/world/lane_band.gd`, `godot/ui/fps_label.gd`) through the editor's language server, as in M1's T022: 0 errors or warnings, with `untyped_declaration` still Error (quickstart scenario 16)
- [X] T024 [US4] After a build, a Play and the export, run `git status`. Confirm `rust/target/`, `godot/.godot/` and `godot/export/` do not appear, and that `rust/src/tuning.rs`, `rust/src/tuning_store.rs`, `rust/src/hero.rs`, `rust/src/lib.rs`, `godot/autoload/tuning.tscn`, `godot/debug/*.gd` with their `.uid` files, `godot/world/lane_band.gd`, `godot/main.tscn` and `godot/project.godot` are present to be committed (quickstart scenario 17)

**Checkpoint**: The Quality Gate is green.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Constitution review and the milestone commit.

- [X] T025 [P] Review docs: `//!` module docs on `rust/src/tuning.rs`, `rust/src/tuning_store.rs` and `rust/src/hero.rs`; `///` on every public item and on everything exposed to Godot, including both signals (comments say why, not what); the `// SAFETY:` comment in `rust/src/lib.rs` unchanged; `##` doc comments on both debug scripts, their exported properties, and `godot/world/lane_band.gd`
- [X] T026 [P] Check for magic numbers (FR-012 of M1, Principle IV): `grep -rnE '\b(300|180|430|700)\b' rust/src godot/debug godot/world godot/ui` returns nothing (test values in `rust/src` must differ). The only numbers in `godot/debug/*.gd` are named debug-UI constants (research R7)
- [X] T027 [P] Check FR-011 and FR-012: `grep -rnE 'KEY_F1|KEY_F5|keycode' godot/debug godot/world godot/ui` returns nothing (keys come only from the Input Map); no `.gd` file reads `tuning.json` or holds a speed or limit in a variable (they read `Tuning` each time); `grep -rn 'debug_' rust/src` returns nothing (Rust does not know about the overlay)
- [X] T028 Run the Constitution compliance review required by Governance: check the finished work against Principles I–VIII (v1.3.3) and the Code Style section, using the table in `specs/003-m2a-debug-overlay/plan.md` as the checklist. Any departure goes into that file's Complexity Tracking section (for example, if the agent built scenes by script)
- [X] T029 (owner) Run the full quickstart.md, scenarios 1–17, in one sitting on the target machine, and after a short tuning session confirm SC-009: "I can retune without restarting and see why the hero stops where it does"
- [X] T030 Commit the M2a implementation on branch `feat/m2a-debug-overlay` in small logical commits (for example: debug actions; validate and tests; TuningStore and autoload; hero and band on Tuning; markers; overlay panel; reload messages; release check; task marks), staging by explicit path and reviewing `git diff --staged` before each commit, as the owner's git rules require. Ask the owner before committing. Then `git status` must be clean (depends on T020–T029)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies. T002 can happen any time before T013.
- **Foundational (Phase 2)**: Needs T001. Blocks every story.
- **US1 (Phase 3)**: Needs Phase 2. This is the MVP.
- **US2 (Phase 4)**: Needs US1's overlay script and scene (T013), because the message spot lives
  in the overlay.
- **US3 (Phase 5)**: Needs the overlay (T014). Independent of US2's play checks.
- **US4 (Phase 6)**: T021 needs US1; T022–T024 need all code tasks.
- **Polish (Phase 7)**: Needs all stories.

### User Story Dependencies

- **US1 (P1)**: Needs only Foundational.
- **US2 (P1)**: Adds F5 and messages to US1's overlay; the reload itself is foundational.
- **US3 (P2)**: Adds the release check to the overlay.
- **US4 (P2)**: Final checks; the commit (T030) waits for all stories.

### Within Each Story

- Tests come before the code they test (T003 → T004).
- `TuningStore` must be built and registered (T005) before the autoload scene can use it (T006),
  and the autoload must exist before GDScript refers to `Tuning` (T008, T010, T011).
- Scripts (T010, T011) exist before the scene uses them (T012).
- Play checks (owner) come last in each story.

### Parallel Opportunities

- **Phase 2**: T008 (band script) can run alongside T007 (hero) once T006 is done; different files.
- **US1**: T010 (markers) and T011 (overlay) are different files and can run together.
- **US2 / US3**: T014 and T018 touch the same file, so they run one after the other.
- **Play checks**: all owner sessions on the same machine; do them in one sitting.
- **Polish**: T025, T026 and T027 can run in parallel.

---

## Parallel Example: User Story 1

```bash
# Both debug scripts at once (different files):
Task: "Create godot/debug/debug_markers.gd: lane bars and origin cross"
Task: "Create godot/debug/debug_overlay.gd: panel, F1 toggle, live values"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: Setup. Debug actions in place.
2. Phase 2: Foundational. `validate` tested; `Tuning` autoload is the one source; game plays as M1.
3. Phase 3: US1. F1 shows the panel, bars and cross.
4. **STOP and VALIDATE**: quickstart scenarios 1–4. The invisible limits are now visible.

### Incremental Delivery

1. Setup + Foundational → tests pass, M1 behaviour unchanged
2. Add US1 → F1 overlay (end of session 1) → MVP
3. Add US2 → live F5 retuning
4. Add US3 → absent from release builds
5. Add US4 + Polish → 60 fps, Quality Gate green, committed → M2a done

### If a Session Stalls

Shrink the next step instead of pushing through (constitution, Development Workflow). For
example, if the signals misbehave, get F5 working with the panel's `Tuning:` line only (read each
frame), and add the separate message spot afterwards. Never put a tuning value in code as a
shortcut (Principle IV). Every checkpoint above is a safe place to stop.

---

## Notes

- [P] = different files, no dependency on unfinished tasks.
- (owner) tasks need the Godot editor GUI or a judgement made by playing.
- Commit only at the end, with the owner's go-ahead (T030), following the owner's git rules.
- Godot creates a `*.uid` file next to each new script. Commit it; it is not cache.
