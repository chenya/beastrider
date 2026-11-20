# Research: M2a Debug Overlay

Decisions for M2a, each with its reason and the options rejected. Most choices come from the
owner's technical context; this file records why they hold up, the details they leave open, and
the two places where the plan departs from the input. Behaviour marked *checked* was confirmed
against the installed Godot 4.7.2 and gdext 0.5.5 on 2026-11-20, in a scratch project outside the
repository.

## R1. Versions: carried over unchanged

- **Decision**: Engine Godot **4.7.2-stable**, gdext `godot = "=0.5.5"` with `api-4-6`, Rust
  1.100.0, the `godot/` + `rust/` layout, JSON tuning data in `godot/data/tuning.json`, the
  Compatibility renderer, 1280×720, V-Sync on. Nothing is upgraded and no crate is added.
- **Rationale**: The owner asked to carry M0 and M1's pins over unchanged. The input says "Godot
  4.6.x", but M0 pinned engine 4.7.2 with API level 4.6, which the owner confirmed during M1.
- **Alternatives considered**: none; changing versions is out of scope.

## R2. One source of truth for tuning: a Rust node as an autoload

- **Problem** (Principle III): in M1 the hero loads its tuning once in `ready` and keeps a copy, so
  a reload could never reach it. The lane band also reads the limits once from the hero.
- **Decision**: A Rust class `TuningStore` (extends `Node`) loads, checks and holds the current
  values. It is the root of a one-node scene, `godot/autoload/tuning.tscn`, registered as the
  autoload **`Tuning`**. Everything that needs a tuning value reads it from `Tuning` when it uses
  it: the hero every physics tick, the lane band and the markers when they are placed or redrawn.
  Nobody else keeps a copy.
- *Checked*: a scene whose root is a GDExtension class works as an autoload in 4.7.2. The node is
  created as the Rust class (`get_class()` returns the Rust class name), its `ready` runs, and it is
  a child of `/root` before the main scene loads, so the hero and the overlay find it ready.
- **Naming, a departure from the input**: the input names the class `Tuning` and the autoload
  `Tuning`. *Checked*: when an autoload has the same name as a class, the name in GDScript means the
  autoload node, and using the class as a type (`var t: Tuning`) then fails at run time with
  "Internal script error! Opcode: 28". So the class is `TuningStore` and the autoload keeps the short name `Tuning`, which keeps
  the owner's call site `Tuning.reload()` in GDScript.
- **Alternatives considered**: class `Tuning` with autoload `TuningData` (keeps the class name but
  changes every GDScript call site); a gdext engine singleton registered from Rust (works, but it is
  not a node, so it cannot emit signals through the scene tree as simply and is not visible in the
  editor's Autoload list); keeping the hero's copy and pushing new values into it (two copies that
  can disagree, which is the bug this milestone removes).

## R3. Reload is all or nothing: parse, validate, then swap

- **Decision**: `TuningStore::reload()` reads the file into a fresh `RawTuning` (four `f64`s,
  parse errors reported as `TuningError`), passes it to a pure function
  `validate(raw, screen_height) -> Result<TuningValues, TuningError>`, and only on `Ok` replaces
  the stored `TuningValues`. On any error the stored values are left untouched. No step panics on
  bad data: every failure is a `TuningError`.
- **Rationale**: Splitting parsing (needs Godot's JSON parser) from checking (plain numbers) lets
  every rule be unit-tested with `cargo test` without the engine, as `movement::step` is in M1.
  The swap at the end makes a partial update impossible by construction (spec edge case
  "Part of the file is bad").
- **Screen height**: the "lane bottom inside the screen" rule needs the screen height, which only
  the running game knows. It is a parameter of `validate` rather than a separate check, so start-up
  and reload apply exactly the same rules (FR-009). `TuningStore` passes its viewport's height.
- **Tests**: one passing and one failing case per rule: horizontal speed `> 0` and finite;
  vertical speed `> 0` and finite; lane top finite and `>= 0`; lane bottom finite; lane top
  `<` lane bottom; lane bottom `<=` screen height. Plus: the error names the field path and the
  reason.
- **Alternatives considered**: validating while parsing (M1's shape; mixes engine calls into the
  rules and cannot run under `cargo test`); applying the valid fields and skipping the bad ones
  (the spec forbids it).

## R4. JSON needs no cache bypass

- **Decision**: Reload reopens and rereads `res://data/tuning.json` with `FileAccess` (through
  gdext's `GFile`), exactly as start-up does. No cache handling is needed.
- *Checked*: writing a file, reading it, rewriting it and reading again returns the new contents
  both times. `FileAccess` reads the disk directly; Godot's resource cache applies to resources
  loaded with `ResourceLoader`/`load()`, and M0 chose JSON, not `.tres`.
- **Alternatives considered**: `ResourceLoader.load(..., CACHE_MODE_IGNORE)`; needed only for a
  Godot Resource format, which the project does not use.

## R5. Result signals on `TuningStore`

- **Decision**: `TuningStore` declares `#[signal] tuning_reloaded()` and
  `#[signal] tuning_rejected(field: GString, reason: GString)`, emitted at the end of every
  `reload()`. The overlay, the lane band and the markers connect to them. Rust does not know who
  listens (Principle III's own example: the reacting display must not be known by the sender).
- **Field for file-level errors**: when the file cannot be read or is not valid JSON there is no
  field; `field` is then the file path (`res://data/tuning.json`) and `reason` says what is wrong,
  so the message still names where to look (FR-008).
- **Re-entrancy**: a handler for `tuning_reloaded` calls back into `Tuning` (for example
  `Tuning.lane_top()`) while `reload()` still holds `&mut self`. *Checked in the gdext 0.5.5
  source*: typed signals emitted with `self.signals().<name>().emit(...)` from a `&mut self`
  method go through `base_mut()`, which allows re-entrant calls into the same object. The handler
  reads the new values, because the swap happens before the emit.
- **Start-up result**: the first load happens in `TuningStore::ready`, before the overlay exists,
  so nobody hears that signal. `TuningStore` therefore also keeps the last rejection and exposes it
  (`rejected_field()`, `rejected_reason()`, empty when the last load succeeded). The overlay reads it
  once in its `_ready` and shows a start-up error the same way as a reload error.
- **Output panel**: every rejection, at start-up or on reload, is also written with
  `godot_error!("Tuning: …")`, as M1 did, so it reaches the output panel (FR-008).

## R6. The hero reads tuning every physics tick

- **Decision**: In `ready`, the hero checks only what tuning cannot fix: player index, input
  actions, the `Body` width, and that the `Tuning` autoload exists (`/root/Tuning` as
  `TuningStore`). It keeps `Gd<TuningStore>`, not the values. Each physics tick it asks
  `Tuning` for the current values (`Option<TuningValues>`, a small `Copy` struct), builds the
  speeds and limits from them and the screen width, and calls the unchanged `movement::step`.
  If there are no values (start-up load failed), the hero stands still that tick.
- **Rationale**: reading a 16-byte struct once a tick costs nothing measurable and means a
  successful reload reaches the hero on the very next tick (acceptance check 4), with no
  notification path to get wrong. It also gives clarification Q1 for free: a hero waiting for
  tuning starts moving on the first tick after a good reload, while a hero stopped by a missing
  action or a zero-width body stays stopped, because its `ready` checks failed (FR-010).
- **Hero outside a new lane**: `movement::step` already clamps every tick, so after a reload that
  moves the lane the hero is put at the nearest point inside on the next tick (spec edge case).
- **Removed**: `Hero::lane_top()` and `lane_bottom()`; the band and markers read `Tuning`
  instead. `check_lane_fits` folds into `validate`.
- **Alternatives considered**: caching in the hero and listening to `tuning_reloaded` (two
  copies, and a Rust class subscribing to its own game's signals for no gain).

## R7. The overlay: a CanvasLayer with a panel and a separate message spot

- **Decision**: `godot/debug/debug_overlay.gd` on a `CanvasLayer` named `DebugOverlay`
  (layer above `Hud`). It holds:
  - a `PanelContainer` in the **top-left** corner (the always-on FPS counter stays top-right,
    clarification Q2), with typed `Label`s for FPS, hero position, player index, horizontal and
    vertical speed, and the last reload result. Hidden at start; F1 toggles it. Its labels are
    updated in `_process` only while it is visible.
  - a message `Label` (the "message spot", clarification Q3) outside the panel, at the top
    centre, visible whether the panel is open or not. "Tuning reloaded" is shown for
    `RELOADED_MESSAGE_SECONDS := 2.0` and then cleared; a rejection message stays until the next
    reload (FR-008, owner's answer during planning).
- **The constant**: the 2-second duration and the marker sizes are named constants in the
  scripts. They are debug-UI presentation, not gameplay tuning, so Principle IV does not require
  them in the data file (the owner's input says the same).
- **Input**: `debug_toggle_overlay` (F1) and `debug_reload_tuning` (F5) are Input Map actions
  with physical keycodes, `device -1`, like the `p1_*` actions. The overlay handles both in
  `_unhandled_input` with `event.is_action_pressed(action)`, whose default `allow_echo = false`
  means holding F5 reloads once (spec edge case). F5 calls `Tuning.reload()` whether or not the
  panel is open. The overlay marks the event handled so nothing else reacts to F1/F5.
- **Values shown**: FPS from `Engine.get_frames_per_second()`, the position from the hero node's
  `position` (rounded to whole pixels), `player_index` from the hero, speeds from `Tuning`. The
  script stores none of them (FR-011); it formats what it reads each frame.
- **The FPS counter**: the input says it moves into the overlay; the owner chose during planning
  to keep it, as clarification Q2 recorded. `Hud/FpsLabel` is unchanged.
- **Alternatives considered**: one script per label (more files for six lines of text); handling
  keys in `_input` (would see keys before GUI controls; nothing in the game uses them, and
  `_unhandled_input` is the documented place for game hotkeys).

## R8. World markers: drawn above the Y-sorted world, redrawn only on change

- **Decision**: `godot/debug/debug_markers.gd` on a `Node2D` named `DebugMarkers`, a sibling
  placed **after** `World` with `z_index = 1`, so it draws over every pillar and the hero
  (FR-005). It draws in `_draw()`:
  - the lane top and lane bottom as horizontal bars across the screen width, 2 px tall,
    straddling each limit (rows `y - 1` and `y`), so the hero's feet, whose bottom edge is at the
    limit, meet the middle of the bar;
  - a small cross at the hero's origin.
  It calls `queue_redraw()` only when the hero's position differs from the one last drawn
  (checked in `_process`) or when `Tuning` emits `tuning_reloaded`. Hidden with the panel.
- **Rationale**: outside `World` it never joins the Y-sort, and `z_index` above 0 puts it over the
  whole world. `_draw` output is cached by the engine, so a still hero costs no redraw.
- **The "last drawn position"** is a presentation cache, not game state: it only decides whether to
  repaint, and the hero's real position stays in Rust.
- **Alternatives considered**: `Line2D` nodes moved from script (three nodes to keep in step);
  drawing from the hero in Rust (moves presentation into Rust, against the split).

## R9. The lane band follows reloads

- **Decision**: `lane_band.gd` reads `Tuning.lane_top()` / `lane_bottom()` in `_ready` and again
  on `Tuning.tuning_reloaded`. It no longer needs the hero or the `await hero.ready`, because the
  autoload is ready before the main scene. When `Tuning.has_values()` is false the band has zero
  height, as in M1.
- **Rationale**: FR-006 requires the band to move with a reload.

## R10. Release builds leave the debug tools out

- **Decision**: In `_ready`, when `OS.is_debug_build()` is false, the overlay frees the markers
  and itself (`queue_free`). The F1/F5 actions stay in the Input Map, but nothing handles them, so
  they do nothing (FR-013). Nothing in Rust refers to the overlay; `TuningStore` still loads the
  file at start-up in every build, because the game needs it.
- *Checked*: `OS.is_debug_build()` is `true` when running from the editor or `godot --path`;
  it is `false` only in a game exported with a **release** export template.
- **Verification needs export setup** (new for M2a): `~/.local/share/godot/export_templates/` is
  empty and the project has no export preset. Checking acceptance check 7 needs, once:
  1. the 4.7.2 export templates installed (Editor → Manage Export Templates → Download);
  2. a Linux export preset in `godot/export_presets.cfg` (committed; it holds no credentials);
  3. `cargo build --release`, because the `.gdextension` file's `linux.release.x86_64` entry
     points at `rust/target/release/libbeastrider.so`;
  4. a release export into `godot/export/` (already ignored by git), then running it.
- **Alternatives considered**: a Rust-side `#[cfg(debug_assertions)]` (ties a presentation choice
  to the Rust build profile, and the overlay is GDScript); a project-setting feature tag
  (`OS.has_feature("debug")` is equivalent but less direct).

## R11. Scene building

- **Decision**: As in M1, the agent builds the scene changes with a headless Godot script
  (`PackedScene.pack` + `ResourceSaver.save`) if the owner hands the task over, and the Input Map
  entries through `ProjectSettings`. The result is an ordinary `.tscn` that opens in the editor.
  Recorded in Complexity Tracking, as M1 did.
- **Alternatives considered**: godot-mcp's scene tools (fail on this project's untyped-declaration
  error, see M1).
