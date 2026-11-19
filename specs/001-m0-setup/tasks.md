---

description: "Task list for M0 Setup – Toolchain Proof"
---

# Tasks: M0 Setup – Toolchain Proof

**Input**: Design documents from `specs/001-m0-setup/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Rust unit tests are included for the two pure functions chosen in
[research.md R11](research.md) (speed check, `wrap_x`). Everything else is verified by playing,
using the [quickstart.md](quickstart.md) scenarios.

**Organization**: Tasks are grouped by user story so each can be checked on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on unfinished tasks)
- **[Story]**: Which user story the task belongs to (US1–US4 from spec.md)
- **(owner)**: Needs the owner, because it uses the Godot editor GUI (the constitution says
  scenes are built in the editor) or the target hardware. An agent can prepare everything around
  it but cannot tick it.

## Path Conventions

Two folders at the repository root, as in [plan.md](plan.md): `rust/` (the Rust crate) and
`godot/` (the Godot project). Run Rust commands from `rust/`.

## Session plan (about 15 h/week, Principle II)

- **Session 1**: Phases 1–2 (toolchain, empty library loads, tuning module and its tests)
- **Session 2**: Phase 3 (US1: the square drifts on screen). Ends playable.
- **Session 3**: Phases 4–7 (verification on hardware, Quality Gate, review, commit)

Each session ends with something that builds, and from Session 2 on, something you can play.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Toolchain installed, both folders exist, and Godot loads an (empty) Rust library.

- [X] T001 Create the root `.gitignore` containing exactly the two rules `rust/target/` and `godot/.godot/` (one per line, with a short comment above each) in `.gitignore`
- [X] T002 [P] Confirm the installed engine: `godot --version` (resolving to `~/.local/bin/godot`) prints `4.7.2.stable.official.ed1daf0bf` (quickstart.md "Prerequisites" step 2, research.md R1). If it prints anything else, stop and update research.md R1 first (outside the repo, nothing to commit)
- [X] T003 [P] Create `rust/Cargo.toml`: `[package] name = "beastrider"`, `edition = "2024"`, `rust-version = "1.94"`, `publish = false`; `[lib] crate-type = ["cdylib"]`; `[dependencies] godot = { version = "=0.5.5", features = ["api-4-6"] }` (exact pin and explicit API level per research.md R2; no other dependencies)
- [X] T004 [P] Create `rust/rust-toolchain.toml` with `[toolchain] channel = "1.100.0"` and `components = ["rustfmt", "clippy"]` (research.md R3)
- [X] T005 [P] Create `rust/rustfmt.toml` containing `max_width = 100` (constitution, Code Style: Rust)
- [X] T006 Create `rust/src/lib.rs` with the extension entry point: a `///`-documented unit struct `BeastriderExtension` and `#[gdextension] unsafe impl ExtensionLibrary for BeastriderExtension {}` preceded by a `// SAFETY:` comment explaining that gdext requires this `unsafe impl` as the library's entry point and that it adds no unsafe code of its own (the only `unsafe` allowed in game code). Then run `cargo build` in `rust/` and confirm it creates `rust/target/debug/libbeastrider.so` and `rust/Cargo.lock` (depends on T003–T005)
- [X] T007 [P] Create `godot/project.godot` (`config_version=5`) with: `application/config/name="Beastrider"`, `application/config/features=PackedStringArray("4.7")` (the engine version the project is saved with; the Rust API level is separate and stays 4.6), `application/run/main_scene="res://main.tscn"`, `display/window/size/viewport_width=1280`, `display/window/size/viewport_height=720`, V-Sync left at its default, Enabled (no `display/window/vsync/vsync_mode` line: the editor removes settings that equal the default when it saves, research.md R6), `rendering/renderer/rendering_method="gl_compatibility"`, `rendering/renderer/rendering_method.mobile="gl_compatibility"`, `rendering/environment/defaults/default_clear_color=Color(0.05, 0.05, 0.07, 1)` (dark background), and `debug/gdscript/warnings/untyped_declaration=2` (Error). Leave the Linux display driver at its default (X11, research.md R5)
- [X] T008 [P] Create `godot/beastrider.gdextension`: `[configuration]` with `entry_symbol = "gdext_rust_init"`, `compatibility_minimum = 4.6`, `reloadable = true`; `[libraries]` with `linux.debug.x86_64 = "res://../rust/target/debug/libbeastrider.so"` and `linux.release.x86_64 = "res://../rust/target/release/libbeastrider.so"` (research.md R7)
- [X] T009 Import the project once headless with `godot --headless --path godot --editor --quit` and confirm the output has no errors about `beastrider.gdextension` or loading `libbeastrider.so` (FR-011). It is expected to warn that `res://main.tscn` does not exist yet. Confirm `git status` does not show `godot/.godot/` or `rust/target/` (depends on T001, T002, T006–T008)

**Checkpoint**: Godot loads the Rust library without errors. Nothing is on screen yet.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The project-wide tuning data and the Rust code that loads and checks it
([contracts/tuning-data.md](contracts/tuning-data.md)). `DriftingSquare` (US1) cannot get a speed
without it.

**⚠️ CRITICAL**: US1 cannot start until this phase is complete

- [X] T010 [P] Create `godot/data/tuning.json` containing exactly `{ "drifting_square": { "speed": 160 } }`, formatted over several lines with 2-space indentation. 160 px/s crosses 1280 px in about 8 s (spec Assumptions)
- [X] T011 [P] In `rust/src/tuning.rs`, define:
  - a `///`-documented constant `TUNING_PATH: &str = "res://data/tuning.json"` (a file path, not a tuning value);
  - a `TuningError` enum with one variant per cause in contracts/drifting-square.md "Errors": file missing or unreadable (path + read error), invalid JSON (path + line + parser message), missing field (path + field path), and invalid value (path + field path + the bad value). Implement `Display` so every message names the file, the field path where there is one, and the problem, and `impl std::error::Error`;
  - `pub struct SquareTuning { speed: f32 }`, with a checked constructor `SquareTuning::try_new(speed: f64) -> Result<SquareTuning, TuningError>` that accepts only values that are "finite and `>= 0` (`0` = stays still)", plus an accessor `speed(&self) -> f32`. There is no `Default` and no other way to build it, so an invalid `SquareTuning` cannot exist (data-model.md).

  `///` docs on every public item
- [X] T012 Add `#[cfg(test)] mod tests` to `rust/src/tuning.rs` covering `SquareTuning::try_new`: accepts `0.0` and `160.0`; rejects `-5.0`, `f64::NAN`, `f64::INFINITY`; and the `Display` of a rejection contains `res://data/tuning.json`, `drifting_square.speed` and the bad value (depends on T011)
- [X] T013 In `rust/src/tuning.rs`, implement `pub fn load_square_tuning() -> Result<SquareTuning, TuningError>` (or with a `path: &str` parameter, called with `TUNING_PATH`) as follows:
  1. read the whole file through gdext's file access (`GFile` / `FileAccess`);
  2. parse it with a Godot `Json` instance so the error line and message are available;
  3. require a top-level Dictionary with key `drifting_square`, which must be a Dictionary with key `speed`;
  4. require `speed` to be a JSON number (Variant INT or FLOAT; reject strings and other types as an invalid value);
  5. finish with `SquareTuning::try_new`.

  Unknown keys are ignored (contracts/tuning-data.md rule 2). There is no fallback value anywhere (Principle IV). Use `?` and `Result` throughout; no `unwrap()` and no `panic!` (depends on T011)
- [X] T014 Add `mod tuning;` to `rust/src/lib.rs`, then in `rust/` run `cargo build` and `cargo test`. Both pass (dead-code warnings for the not-yet-used loader are acceptable until T017) (depends on T012, T013)

**Checkpoint**: The tuning format exists, and the speed checks are unit-tested.

---

## Phase 3: User Story 1 – Press Play and see it run (Priority: P1) 🎯 MVP

**Goal**: Pressing Play shows the dark 1280×720 window with the drifting, wrapping square, a speed
label read from Rust, and an FPS counter, with no errors in the output.

**Independent Test**: Open the project, press Play, and watch for 30 s. The square drifts and wraps
at least once, the label shows "Speed: 160 px/s", and the FPS counter is visible (quickstart
scenarios 1–3).

### Tests for User Story 1

- [X] T015 [P] [US1] In `rust/src/drifting_square.rs`, write the pure function `fn wrap_x(x: f32, width: f32, viewport_width: f32) -> f32` as `(x + width).rem_euclid(viewport_width + width) - width` (research.md R9; the result is always in `[-width, viewport_width)`), with `#[cfg(test)]` tests:
  - a value already in range stays the same;
  - `x == viewport_width` maps to `-width`;
  - a value just past the right edge lands just after `-width`;
  - a value several screen widths past the edge (the very-high-speed edge case) still lands in range.

  Run `cargo test`

### Implementation for User Story 1

- [X] T016 [US1] In `rust/src/drifting_square.rs`, implement the class from contracts/drifting-square.md:
  - Declaration: `#[derive(GodotClass)] #[class(base = Node2D)] pub struct DriftingSquare` with fields `tuning: Option<SquareTuning>`, `width: f32` and `base: Base<Node2D>`, initialised via `#[class(init)]` or an `INode2D::init` that sets `tuning = None` and `width = 0.0`. These are "not loaded yet" markers, not tuning values.
  - `ready()`: call `tuning::load_square_tuning()`. Then fetch the child `Body` as a `Control` with a non-panicking lookup (`try_get_node_as`), and require its width `> 0`. On any failure, report exactly one `godot_error!` whose message starts with `DriftingSquare:` (for tuning failures, followed by the `TuningError` `Display`; for `Body`, the node path and the problem), and leave `tuning` as `None` (stopped).
  - `process(delta)`: do nothing while stopped. Otherwise `x += speed * delta as f32`, then `x = wrap_x(x, width, viewport width from get_viewport_rect())`. Never touch `position.y`.
  - `#[godot_api] impl DriftingSquare { #[func] pub fn speed(&self) -> f32 }`: returns the tuned speed, or `0.0` while stopped. There is no setter, property or signal.
  - Docs: `///` on the struct and on `speed()`. No literal speed value anywhere (FR-008).

  (depends on T014, T015)
- [X] T017 [US1] Add `mod drifting_square;` to `rust/src/lib.rs`, run `cargo build` in `rust/`, then open the editor (`godot --editor --path godot`) and confirm `DriftingSquare` appears in the Create New Node dialog under `Node2D` (depends on T016)
- [X] T018 [P] [US1] Create `godot/ui/speed_label.gd`:
  - `extends Label`, then a `##` class doc comment.
  - One typed export, `@export var square: DriftingSquare`, with a `##` doc.
  - `func _process(_delta: float) -> void:` setting `text = "Speed: %d px/s" % roundi(square.speed())`.
  - No other variables. Tabs for indentation, and code order as in the constitution's GDScript section.

  (depends on T017, so that the `DriftingSquare` type is known to the editor)
- [X] T019 [P] [US1] Create `godot/ui/fps_label.gd`: `extends Label`, a `##` class doc comment, and `func _process(_delta: float) -> void:` setting `text = "FPS: %d" % Engine.get_frames_per_second()`. No variables. Tabs.
- [X] T020 [US1] (owner) In the Godot editor, build and save `godot/main.tscn`:
  - root `Node2D` named `Main`;
  - child `DriftingSquare` at position `(0, 328)`, with child `ColorRect` named `Body` at `(0, 0)`, size `64×64`, in a bright saturated placeholder colour such as orange `#E8792B`;
  - child `CanvasLayer` named `Hud`, holding a `Label` named `SpeedLabel` (top-left preset with a 16 px margin, script `res://ui/speed_label.gd`, `square` set to the `DriftingSquare` node) and a `Label` named `FpsLabel` (top-right preset with a 16 px margin, script `res://ui/fps_label.gd`).

  Confirm the main scene is `res://main.tscn` in Project Settings (depends on T017–T019)
- [X] T021 [US1] (owner) Press Play and run quickstart.md scenarios 1, 2 and 3: window, square, wrap, label "Speed: 160 px/s", FPS counter; no errors in the Output panel at startup or after 60 s. Fix anything that fails in `rust/src/drifting_square.rs` or `godot/ui/*.gd` before going on (depends on T020)

**Checkpoint**: M0 is playable. This is the MVP: Rust and GDScript work together in Godot.

---

## Phase 4: User Story 2 – Holds 60 fps on the target machine (Priority: P1)

**Goal**: Confirm a steady 60 fps with V-Sync on, on the UHD 630, using the Compatibility renderer.

**Independent Test**: Quickstart scenario 4 (watch the FPS counter for 60 s), backed by scenarios
3 and 9.

- [ ] T022 [US2] (owner) Check the renderer and GPU: the Output panel startup lines show the OpenGL Compatibility renderer on the Intel/Mesa device (not `llvmpipe`); Project Settings show `rendering_method = gl_compatibility` and V-Sync Enabled (quickstart scenarios 3 and 9; values set in `godot/project.godot` by T007)
- [ ] T023 [US2] (owner) Watch the FPS counter for 60 s by the clock (quickstart scenario 4). It must show 60 throughout, except the first second. If it fails, follow quickstart "If something goes wrong" (retry under "Ubuntu on Xorg"), and record the finding and its fix under R5 or R6 in `specs/001-m0-setup/research.md` (depends on T021, T022)

**Checkpoint**: Principle V is proven on the real hardware.

---

## Phase 5: User Story 3 – Tune speed without recompiling (Priority: P2)

**Goal**: Prove that editing `tuning.json` changes the game with no Rust rebuild, and that bad
data gives clear errors with no crash and no hidden fallback.

**Independent Test**: Quickstart scenarios 5, 6 and 7.

- [ ] T024 [US3] (owner) Quickstart scenario 5: time one crossing at speed 160, set `speed` to `320` in `godot/data/tuning.json`, do **not** run cargo, press Play. The label shows 320, and the crossing takes about half the time (±10 %, SC-003)
- [ ] T025 [US3] (owner) Quickstart scenario 6: set `speed` to `0` in `godot/data/tuning.json`. The square stays still and the label shows 0
- [ ] T026 [US3] (owner) Quickstart scenario 7. In `godot/data/tuning.json`, try each of: `-5`; `"fast"`; the `speed` key removed; the `drifting_square` section removed; a broken JSON comma; the file renamed. Each time, expect exactly one error starting `DriftingSquare:` that names the file, the field path (or line, for broken JSON) and the problem; the square stays still; no crash. If a message is unclear, fix its `Display` text in `rust/src/tuning.rs` and re-run `cargo test`
- [ ] T027 [US3] Restore `godot/data/tuning.json` to `speed` `160`, and confirm `git diff godot/data/tuning.json` is empty (depends on T024–T026)

**Checkpoint**: Principle IV is proven end to end.

---

## Phase 6: User Story 4 – Clean, committed, ready for M1 (Priority: P2)

**Goal**: The Quality Gate passes, and the repository holds everything except build output and
caches.

**Independent Test**: Quickstart scenarios 10, 11 and 12.

- [X] T028 [US4] In `rust/`, run `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets` and `cargo test`. Fix every warning in `rust/src/*.rs` until all four pass with 0 warnings (SC-005). Do not silence lints with `#[allow]` unless the reason is written next to it
- [ ] T029 [US4] (owner) In the Godot editor, open `godot/ui/speed_label.gd` and `godot/ui/fps_label.gd` and confirm the Output and Script panels show 0 GDScript errors or warnings, with `untyped_declaration` set to Error (quickstart scenario 11)
- [X] T030 [US4] After a build and a Play, run `git status`. Confirm `rust/target/` and `godot/.godot/` do not appear, and that `rust/Cargo.lock`, `godot/project.godot`, `godot/beastrider.gdextension`, `godot/main.tscn`, `godot/data/tuning.json`, `godot/ui/*.gd` and their `*.uid` files are all present to be committed (quickstart scenario 12, FR-012, research.md R12)

**Checkpoint**: The Quality Gate is green.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Constitution compliance review and the milestone commit.

- [X] T031 [P] Review docs: `///` on every public item in `rust/src/lib.rs`, `rust/src/tuning.rs` and `rust/src/drifting_square.rs` (comments say why, not what); the `// SAFETY:` comment is present on the `ExtensionLibrary` impl; `##` class doc comments are on both scripts in `godot/ui/`
- [X] T032 [P] Check for magic numbers (FR-008): `grep -rnE '\b160\b|\b320\b' rust/src godot/ui` returns nothing. The speed exists only in `godot/data/tuning.json`
- [X] T033 Run the Constitution compliance review required by Governance: check the finished work against Principles I–VIII and the Code Style section, using the table in `specs/001-m0-setup/plan.md` as the checklist. Any departure goes into the Complexity Tracking section of that file
- [ ] T034 (owner) Run the full quickstart.md, scenarios 1–12, in one sitting on the target machine. Every scenario must pass (the milestone is done when the Quality Gate passes)
- [ ] T035 Commit the M0 implementation on branch `feat/m0-setup` in small logical commits (for example: Rust crate and entry point; tuning module and tests; DriftingSquare; Godot project, scene and scripts), staging by explicit path and reviewing `git diff --staged` before each commit, as the owner's git rules require. Ask the owner before committing. Then `git status` must be clean (depends on T028–T034)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies. T002 (installing Godot) can happen at any time before T009.
- **Foundational (Phase 2)**: Needs T006 (the crate builds). Blocks US1.
- **US1 (Phase 3)**: Needs Phase 2. This is the MVP.
- **US2 (Phase 4)**: Needs US1 (something has to be on screen to measure).
- **US3 (Phase 5)**: Needs US1 (the label and the square show the effect of the tuning).
- **US4 (Phase 6)**: Needs US1. Best done after US2 and US3, so their fixes are included.
- **Polish (Phase 7)**: Needs all stories.

### User Story Dependencies

- **US1 (P1)**: Needs only Foundational.
- **US2 (P1)** and **US3 (P2)**: Both only verify US1's build, so they are independent of each
  other and can be done in either order.
- **US4 (P2)**: Independent of US2 and US3, but the final commit (T035) waits for all of them.

### Within Each Story

- In Rust, a pure function and its tests (T015) come before the class that uses it (T016).
- `DriftingSquare` must be built and loaded (T017) before GDScript can use it as a type (T018).
- The scripts (T018, T019) must exist before the scene that uses them (T020).
- Play checks (owner) come last in each story.

### Parallel Opportunities

- **Phase 1**: T002, T003, T004, T005, T007 and T008 touch different files and can all run
  together. T006 waits for T003–T005.
- **Phase 2**: T010 (the JSON file) and T011 (Rust types) can run in parallel.
- **US1**: T015 (`wrap_x`) can run alongside Phase 2's T012–T013. T018 and T019 can run in
  parallel once T017 is done.
- **US2 / US3**: both are owner play sessions on the same machine, so do them one after the other
  in a single sitting.
- **Polish**: T031 and T032 can run in parallel.

---

## Parallel Example: Phase 1 and User Story 1

```bash
# Phase 1 – independent files, all at once:
Task: "Create rust/Cargo.toml (godot =0.5.5, api-4-6, cdylib)"
Task: "Create rust/rust-toolchain.toml (1.100.0, rustfmt, clippy)"
Task: "Create rust/rustfmt.toml (max_width = 100)"
Task: "Create godot/project.godot (Compatibility, 1280x720, V-Sync, untyped_declaration=Error)"
Task: "Create godot/beastrider.gdextension (debug/release linux.x86_64 paths)"

# User Story 1 – after T017, both label scripts at once:
Task: "Create godot/ui/speed_label.gd (typed @export square, reads square.speed())"
Task: "Create godot/ui/fps_label.gd (reads Engine.get_frames_per_second())"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: Setup. The library loads in Godot.
2. Phase 2: Foundational. Tuning loads and is checked, with unit tests.
3. Phase 3: US1. The square drifts on screen.
4. **STOP and VALIDATE**: quickstart scenarios 1–3. The toolchain is now proven to work together.

### Incremental Delivery

1. Setup + Foundational → builds, loads, tests pass (end of session 1)
2. Add US1 → playable (end of session 2) → MVP
3. Add US2 → 60 fps proven on the UHD 630
4. Add US3 → data-driven tuning proven, error paths checked
5. Add US4 + Polish → Quality Gate green, committed (end of session 3) → M0 done

### If a Session Stalls

Shrink the next step instead of pushing through (constitution, Development Workflow). For example,
if `DriftingSquare` misbehaves, get it moving at the tuned speed first and leave wrapping for later.
Every checkpoint above is a safe place to stop.

---

## Notes

- [P] = different files, no dependency on unfinished tasks.
- (owner) tasks need the Godot editor GUI or the target hardware.
- Commit only at the end, with the owner's go-ahead (T035), following the owner's git rules.
- Godot creates `*.uid` files next to scripts (4.4+). Commit them; they are not cache.
