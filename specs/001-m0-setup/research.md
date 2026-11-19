# Research: M0 Setup – Toolchain Proof

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Date**: 2026-11-19

Each entry records the decision, why it was chosen, and what else was considered. Facts that were
looked up are marked with where they came from.

---

## R1. Godot version

- **Decision**: Godot **4.7.2-stable**, standard (non-.NET) build, `linux.x86_64`. This is the
  official binary already installed on the target machine as `~/.local/bin/godot`
  (`godot --version` → `4.7.2.stable.official.ed1daf0bf`). The game is built against the **4.6
  API** (R2), which is older than the engine. That is allowed.
- **Rationale**: The owner's choice (2026-11-19). The plan first pinned 4.6.3, the latest 4.6
  patch. When it turned out that 4.7.2 was already installed, the owner chose to run on 4.7.2 and
  keep the 4.6 API level. This means:
  - no second Godot install, and the plain `godot` command is the right one;
  - the constitution rule "the gdext API level MUST NOT be newer than the Godot version that runs
    the game" holds (4.6 ≤ 4.7.2);
  - the extension stays loadable by any Godot from 4.6 up, since `compatibility_minimum = 4.6`
    (R7).

  The cost is that 4.7-only engine APIs cannot be called from Rust until the API level is raised.
- **Alternatives considered**:
  - *Godot 4.6.3-stable* (latest 4.6 patch, 2026-05-20). This was the original pin. Rejected:
    it would mean a second binary next to the installed 4.7.2, and an easy way to open the project
    with the wrong one.
  - *4.7.2 with `api-4-7`*. Matches the engine exactly. Not chosen by the owner. Raising the API
    level later is a one-line `Cargo.toml` change plus a rebuild, and it is needed only when a 4.7
    API is wanted.
  - *Ubuntu snap / Flathub packages*. Rejected: the version can change under you, and sandboxing
    can get in the way of loading a library from `../rust/target/`.
- **Install note**: The binary is not part of the repository. If `~/.local/bin/godot` is ever
  replaced with a different version, update this entry and re-run the quickstart.
- **Sources**: [GitHub releases](https://github.com/godotengine/godot/releases),
  [Maintenance release: Godot 4.6.3](https://godotengine.org/article/maintenance-release-godot-4-6-3/).

## R2. gdext version and API level

- **Decision**: `godot = { version = "=0.5.5", features = ["api-4-6"] }`.
- **Rationale**:
  - 0.5.5 is the newest 0.5.x on crates.io (checked with `cargo search` and `cargo info`). The
    `=` pins it exactly, and `Cargo.lock` is committed.
  - **The API level is set explicitly.** In 0.5.5, the default when no `api-*` feature is given is
    4.6 (`godot-bindings-0.5.5/src/import.rs` uses `version_4_6` as the default), which matches
    what the owner expected. But the same release already supports 4.7 (`LATEST_API_VERSION =
    (4, 7, 0)`), so a future 0.5.x could move the default to 4.7 or later. Writing `api-4-6`
    keeps the owner's chosen API level fixed no matter what the default becomes. It also keeps
    the constitution rule ("the gdext API level MUST NOT be newer than the Godot version that
    runs the game") true by construction: the engine is 4.7.2 (R1), and a default moving past it,
    for example to 4.8, would otherwise break the rule silently.
  - Licence: MPL-2.0 (from `cargo info`). gdext is the constitution's one approved dependency.
- **Alternatives considered**: relying on the default (rejected for the reason above);
  `api-custom` (needs a Godot binary at build time, which adds a step for no gain).

## R3. Rust toolchain

- **Decision**: `rust/rust-toolchain.toml` pins `channel = "1.100.0"` with the `rustfmt` and
  `clippy` components.
- **Rationale**: gdext 0.5.5 declares `rust-version: 1.94` (from `cargo info`), so its minimum is
  1.94, as the owner noted. 1.100.0 is the current stable already installed on the target machine,
  so the pin costs no download. Pinning an exact version keeps clippy's lint set fixed, so the
  "no warnings" gate does not break just because a new Rust release added a lint.
- **Alternatives considered**: `channel = "stable"` (does not pin anything); `1.94.0` (works, but
  is an older compiler with no benefit for a new project).
- **Upgrading**: change the pin on purpose, then run the Quality Gate again.

## R4. Tuning data format (project-wide decision)

- **Decision**: **JSON**, one file at `godot/data/tuning.json`, with one top-level section per
  tuned thing (for M0, `"drifting_square": { "speed": 160 }`). Rust reads it with Godot's own
  file access and JSON parser through gdext, so no new crates are added. See
  [contracts/tuning-data.md](contracts/tuning-data.md).
- **Rationale**:
  1. **Every value is visible in the file.** A Godot `.tres` file leaves out any property whose
     value equals the class default. So with a Rust-defined Resource, the "real" value could live
     only in Rust code as the class default. That is exactly the magic number Principle IV
     forbids. With JSON, the file is the only source, and a missing value is an error rather than
     a silent fallback, which is what FR-009 asks for.
  2. **Errors can name the file, line, field and problem** (FR-009). Godot's JSON parser reports
     the error line and message, and Rust checks each field after parsing.
  3. **Easy to edit and review.** Any text editor works, and a balance change shows up as a
     one-line diff.
  4. **Rust stays in charge of loading and checking**, as the constitution's language split
     requires. GDScript never touches the file.
- **Alternatives considered**:
  - *Godot Resource (`.tres`) with a Rust-defined Resource class.* Pros: typed fields, editing in
    the inspector with range hints, no parsing code. Rejected because of point 1 (hidden defaults
    in code), and because every data shape would need a registered Rust class, which is more
    code than M0 needs.
  - *JSON parsed with `serde` / `serde_json`.* Pure Rust, derive-based, easy to unit-test.
    Rejected for now (Principle III): one value does not justify a second and third dependency.
    **Watch for**: when hand-checking fields from a Godot Dictionary gets repetitive (likely
    M4 or M6, with enemy types and waves), a plan can propose serde/serde_json (MIT OR
    Apache-2.0), naming that problem.
  - *One JSON file per tuned thing.* Possible later. One file is simpler while there are only a
    handful of values; sections keep the door open to splitting it.
- **Export note** (not needed for M0, which has no exports): when exporting is first done, confirm
  that `data/*.json` is included in the export, and add it to the export filter if not.

## R5. Wayland vs X11 on Ubuntu 24.04

- **Decision**: **No setting is needed.** Leave the project's Linux display driver at its default
  (X11). On Ubuntu 24.04's default GNOME Wayland session, Godot 4.7 then runs through XWayland.
- **Rationale**: The Godot 4.6 and 4.7 docs both say X11 is still the default display driver for projects,
  because Wayland support is still a work in progress. A project set to Wayland falls back to X11
  automatically when Wayland is not available. The target machine's session is Wayland
  (`XDG_SESSION_TYPE=wayland`), which XWayland handles.
- **Fallback plan**: if V-Sync, frame pacing or window placement misbehave, first try logging in
  with "Ubuntu on Xorg" from the login screen and re-run the 60 fps check. That tells you whether
  the problem is XWayland or something else. `--display-server x11` / `wayland` on the command
  line can switch drivers for a single run without changing the project.
- **Alternatives considered**: forcing native Wayland in project settings (rejected: still marked
  as in progress in 4.6 and 4.7, and it buys nothing for M0).
- **Sources**: [Godot 4.6 docs: Wayland and X11](https://docs.godotengine.org/en/4.6/tutorials/platform/linux/wayland_x11.html),
  [Godot 4.7 docs: Wayland and X11](https://docs.godotengine.org/en/4.7/tutorials/platform/linux/wayland_x11.html).

## R6. Compatibility renderer on the UHD 620

- **Decision**: In project settings, set `rendering/renderer/rendering_method` to
  `gl_compatibility` for both desktop and mobile. Keep V-Sync at its default, Enabled
  (`display/window/vsync/vsync_mode`).
- **Rationale**: Required by Principle V. The UHD 620 on Mesa's `iris` driver supports OpenGL
  well above the 3.3 that the Compatibility renderer needs. M0 draws one rectangle and two labels,
  so 60 fps has a huge margin. The point of the check is to prove the setup (driver, V-Sync,
  XWayland), not the content.
- **Check**: the Godot output panel prints the renderer and GPU at startup. It must show
  `OpenGL API … Compatibility` and the Intel/Mesa device. If it shows `llvmpipe`, hardware
  acceleration is not active, and the 60 fps check means nothing until that is fixed.

## R7. How the Godot project finds the Rust library

- **Decision**: `godot/beastrider.gdextension` with:
  - `entry_symbol = "gdext_rust_init"`, `compatibility_minimum = 4.6` (matches the API level, so
    the library loads in Godot 4.6 and newer, including the installed 4.7.2), `reloadable = true`;
  - `linux.debug.x86_64 = "res://../rust/target/debug/libbeastrider.so"`;
  - `linux.release.x86_64 = "res://../rust/target/release/libbeastrider.so"`.
- **Rationale**: This is the godot-rust book's standard setup for a `godot/` + `rust/` layout. The
  `res://../` path reaches outside the Godot project folder, so the build output can stay in
  `rust/target/` and be ignored by git. With `reloadable = true`, rebuilding the Rust library while
  the editor is open reloads it, so the editor does not need to be restarted.
- **Fresh checkout**: the `.so` does not exist until `cargo build` has run once. The quickstart
  lists this as step 1 (FR-013). Before that, Godot shows its own "library not found" error, which
  the spec accepts as an edge case.

## R8. Code split and naming

- **Decision**:
  - **Rust** `DriftingSquare` (extends `Node2D`): loads and checks its tuning section in
    `ready()`, moves itself in `process(delta)`, wraps at the viewport edge, and exposes a
    read-only `speed()` method to Godot. The square's width is read from its child `Body` node
    (a `ColorRect` built in the editor), so the scene is the only place the size is set.
  - **GDScript** `speed_label.gd` (extends `Label`): has one typed `@export var square:
    DriftingSquare` node reference set in the editor, and sets its text from `square.speed()` every
    frame. It keeps no values of its own.
  - **GDScript** `fps_label.gd` (extends `Label`): sets its text from
    `Engine.get_frames_per_second()` every frame.
- **Rationale**: This follows the constitution's language split exactly: Rust owns the rules and
  the state, and GDScript only shows them. The Rust method is named `speed()`, not `get_speed()`,
  because the constitution's Rust naming rule follows the standard library (`foo()` /
  `set_foo()`), and the same name is then used from GDScript. There is no setter, because nothing
  outside Rust may change the speed.
- **Alternatives considered**:
  - *Rust draws the square itself with `draw_rect`.* Rejected: drawing is presentation, and an
    editor-built `ColorRect` is zero code.
  - *An exported `width` property on `DriftingSquare`.* Rejected: the width would then be set in
    two places (the property and the `ColorRect`), which could drift apart.
  - *A Godot property (`#[var]`) instead of a method.* Workable, but a `#[func]` getter makes "read
    only" obvious without depending on the exact `#[var]` attribute options.
  - *A signal when the speed changes.* Rejected (Principle III): the speed never changes after
    `ready()`, so there is nothing to signal. Polling every frame is what the owner described.

## R9. Wrap-around maths

- **Decision**: Use a pure Rust function, roughly `wrap_x(x, width, viewport_width) -> f32`,
  computed as `(x + width).rem_euclid(viewport_width + width) - width`. The square fully leaves the
  right edge, then enters from the left with its right edge at x = 0.
- **Rationale**: This handles very high speeds (several screen widths in one frame) without
  getting stuck off-screen (spec edge case), and is easy to unit-test without Godot.

## R10. Error reporting for bad tuning data

- **Decision**: Use Rust `Result` with a small error type that names the file, the field and the
  problem. On error, `DriftingSquare` reports it with gdext's `godot_error!` (which shows in the
  editor's output and debugger panels), keeps no tuning loaded, does not move, and `speed()`
  returns `0.0`.
- **Rationale**: This follows the constitution's error rules (`Result` for a malformed tuning file;
  `panic!` only for bugs) and FR-009. No fallback speed lives in code (Principle IV).

## R11. Tests

- **Decision**: Rust unit tests (`cargo test`) cover the two pure pieces: `wrap_x` and the speed
  check (accepts 0 and positive values, rejects negative, NaN and infinity). Everything that
  touches Godot is checked by playing, as the spec's acceptance scenarios describe.
- **Rationale**: These are the two places where M0 can be wrong in ways that are hard to see by
  watching (a rare wrap glitch, or a bad value being accepted). The tests need no engine and no new
  dependencies.
- **Note**: `cargo test` is not part of the constitution's Quality Gate. The plan runs it anyway.
  Adding it to the gate would be a constitution amendment, so it is listed as a follow-up.

## R12. Git ignore rules

- **Decision**: The root `.gitignore` ignores `rust/target/` and `godot/.godot/`. Commit
  `rust/Cargo.lock`, Godot's `*.import` and `*.uid` files, and `project.godot`.
- **Rationale**: These are the owner's two ignore rules (acceptance check 8). Godot 4 expects
  `.import` and `.uid` files to be committed, because they hold import settings and stable
  resource IDs. `Cargo.lock` pins the exact crate graph.
