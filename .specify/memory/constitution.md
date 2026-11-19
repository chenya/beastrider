# Beastrider Constitution

<!-- The Beastrider Project Brief is the source of truth for scope (milestones, Later list,
Not-doing list). If the brief and this file ever disagree, stop and ask the owner which one is
current. -->

## Core Principles

### I. Fun First, Playable Always

Every milestone MUST end with a build the owner can run and play. A rough playable game beats an
elegant unplayable one. Every spec MUST include at least one acceptance check stated as something
the player does and sees (for example: "hit the dummy and it flashes, slides back, and the screen
freezes briefly"). Technical checks (it builds, it loads, no warnings) MAY sit alongside these but
never replace them. If it is unclear whether the current loop is fun, the next step MUST be a
playtest, not more features.

### II. One Milestone at a Time

Scope is the enemy. Work follows the milestone order M0 to M8 in the project brief, and only the
current milestone's must-have list is in scope. Anything else goes onto the Later list, never into
the current spec. A Later-list item enters scope only when the project brief promotes it. The
Not-doing list is binding. It includes online multiplayer of any kind, procedurally generated
levels, a custom engine, 3D, save systems and cutscenes, and removing any of these from it
requires amending this constitution first. A milestone MUST be sized for two or three sessions.
At about 15 hours a week, a milestone that cannot be finished in roughly two weeks MUST be split.

### III. Architecture Serves the Game

A pattern, abstraction or system MAY be introduced only when a concrete problem demands it, and
the plan MUST name that problem in one sentence. Valid examples: a state machine because the hero
has several modes (idle, attack, hurt); a shared component because several enemy types need the
same behaviour; signals because the score display must react to kills without the enemy knowing
about it. "We might need it later" is not a problem. Exception: if the owner states that a piece
of engineering is itself the fun, it MAY proceed, and the plan states once what it costs in time
to a playable game.

### IV. Data-Driven Tuning

Tuning values and content (damage, speeds, health, wave sizes, drop rates, hit-feel timings such
as freeze length and shake strength) MUST live in data files, so balancing means editing numbers,
not code. Neither Rust nor GDScript code may contain magic numbers for these values.

### V. Runs Well on Weak Hardware

The game MUST hold 60 fps at 1280x720 on an Intel UHD 630 integrated GPU using Godot's
Compatibility renderer (OpenGL). Plans MUST NOT depend on features that exist only in the Forward+
or Mobile renderers. Heavy shaders, many dynamic lights and unbounded particle effects are
avoided. The mood comes from palette, silhouette, a simple vignette and a few lights that each
earn their place. Any acceptance check about speed or visuals MUST be verified on the target
hardware, not only on a stronger machine.

### VI. Original Art, Cheap Animation

Characters, creatures and scenes MUST be original. The Frazetta-inspired mood is achieved through
lighting, colour and silhouette; no specific painting is recreated, and no Golden Axe character,
creature or asset is copied. Placeholder art (coloured shapes) is used until the loop is fun.
Characters use cutout animation: a few separate pieces posed with Godot's animation tools
(AnimationPlayer, and Skeleton2D bones where useful), not frame-by-frame drawing. Every
non-placeholder art or audio asset MUST have its source and licence recorded in an assets log in
the repository.

### VII. Co-op Ready, Local Only

Gameplay code MUST NOT assume a single hard-coded "the player". Input, health, magic and camera
logic MUST refer to a player index, so that local two-player co-op can be added at M7 without a
rewrite. Networking code is out of scope and MUST NOT be added.

### VIII. Always Debuggable

From M2 onward the game MUST have a debug overlay with cheat keys: show hitboxes, refill health
and magic, spawn an enemy. From M3 onward, when runs and game over exist, a single key MUST
restart a run in under two seconds. The project is under version control from the first session,
with a commit at the end of every passing milestone.

## Technology and Constraints

- Engine: Godot 4.x, stable release. The exact version is pinned in the M0 plan. The gdext API
  level MUST NOT be newer than the Godot version that runs the game.
- Languages: Rust (through the godot-rust gdext bindings) and GDScript, with a fixed split. Rust
  owns the rules and the state: hero and enemy state machines, combat and damage, hit detection,
  wave spawning, and loading and checking tuning data. GDScript owns presentation and glue: HUD,
  menus, debug overlay, hit-feel effects (flash, shake, particles), and wiring scenes together.
  GDScript MUST NOT hold game state; it reads from Rust objects and reacts to their signals.
  Scenes, nodes and animation are built in the Godot editor.
- Platform: Ubuntu 24.04 LTS desktop, single machine, offline. Target hardware: Intel UHD 630
  integrated GPU with the stock Mesa driver. 60 fps is measured with V-Sync on, on that machine.
- Rendering: Compatibility renderer.
- Data format: one format for all tuning data (JSON or Godot Resources), chosen in the M0 plan
  and used everywhere.
- Rust toolchain: stable Rust, at least the minimum version the chosen gdext release requires,
  checked in the M0 plan.
- Types over strings: modes such as hero and enemy states are Rust enums with exhaustive matches,
  so adding a state is a compile error until every case is handled.
- Dependencies: gdext is the one approved third-party dependency. Any other Rust crate, Godot
  addon or plugin needs a plan that names the problem it solves and its licence.
- Time budget: about 15 hours a week, so plans and tasks are sized for sessions of a few hours.

## Code Style

### Rust

Rust code MUST follow the [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html),
which itself defers to the official Rust style rules and the Rust API Guidelines where it is
silent. In practice for this project:

- Formatting is rustfmt's job. A `rustfmt.toml` at the crate root sets `max_width = 100`, as the
  guide recommends. `#[rustfmt::skip]` is allowed only for deliberate manual alignment, such as a
  table of tuning defaults.
- Naming follows the standard library: `new()` and `try_new()` constructors, `foo()` /
  `set_foo()` accessors, `as_` / `to_` / `into_` conversions, and no abbreviations an outsider
  would not know (`knockback_speed`, not `kb_spd`).
- Errors: `Option` for simple absence, `Result` for recoverable failures that need context (a
  malformed tuning file), `panic!` only for bugs and broken invariants. Prefer
  `expect("why this cannot fail")` over `unwrap()`.
- Unsafe: none in game code. The one exception is the `unsafe impl ExtensionLibrary` entry point
  that gdext requires; it carries a `// SAFETY:` comment, and any other `unsafe` needs a plan
  entry and its own `SAFETY:` comment.
- Documentation: `///` doc comments on every public item, including every class and method
  exposed to Godot, so the GDScript side knows what it is calling. Comments explain why, not
  what.
- The guide's codebase-specific sections (Chromium, Android, Fuchsia) do not apply.

### GDScript

GDScript MUST follow the
[official GDScript style guide](https://github.com/godotengine/godot-docs/blob/master/tutorials/scripting/gdscript/gdscript_styleguide.rst).
In practice for this project:

- Layout: tabs for indentation; wrapped function-call lines take two indent levels, wrapped
  arrays, dictionaries and enums take one. Lines under 100 characters, ideally under 80. Two blank
  lines around functions and inner classes, one blank line to separate logical sections inside a
  function.
- Style: double quotes unless single quotes avoid escapes; `and`, `or` and `not` instead of `&&`,
  `||` and `!`.
- Naming: snake_case for file names, functions and variables; PascalCase for classes and nodes;
  PascalCase, singular, for enum names; CONSTANT_CASE for constants and enum members; signals in
  snake_case and the past tense (`hit_landed`, `wave_cleared`); a single leading underscore for
  private members and for virtual methods meant to be overridden.
- Code order: (1) `@tool`, `@icon`, `@static_unload`; (2) `class_name`; (3) `extends`; (4) `##`
  class doc comment; (5) signals; (6) enums; (7) constants; (8) static variables; (9) `@export`
  variables; (10) other variables; (11) `@onready` variables; (12) `_static_init()`; (13) other
  static methods; (14) overridden built-in virtuals in the order `_init()`, `_enter_tree()`,
  `_ready()`, `_process()`, `_physics_process()`, then the rest; (15) overridden custom methods;
  (16) other methods; (17) inner classes. Within each group, public before private.
- Static typing everywhere: the untyped-declaration warning is set to Error in the project
  settings. Use `:=` when the type is obvious from the same line; write the type explicitly when
  it is ambiguous or cannot be inferred, such as the result of `get_node()`. Avoid `as` casts for
  inference, since a mismatch silently gives `null`.
- Comments: `#` and `##` are followed by a space (commented-out code is not); prefer comments on
  their own line; `##` doc comments on every class, exported property and public method that
  other scripts call.

### Shared naming

Rust classes exposed to Godot are PascalCase (`Hero`, `WaveSpawner`) and appear under those names
in the editor and in GDScript, so both languages use one vocabulary. Signal names are the same in
both languages.

## Quality Gate

At the end of every milestone, all of these hold:

- `cargo fmt --check` passes.
- `cargo build` and `cargo clippy` report no warnings.
- The Godot editor shows no GDScript errors or warnings.
- Every acceptance check in the milestone's spec passes by playing, on the target hardware where
  Principle V applies.

## Development Workflow

Spec Kit commands are run in the coding agent's chat. Run `/speckit-constitution` once for the
project. Then, for each milestone, run `/speckit-specify`, `/speckit-plan`, `/speckit-tasks` and
`/speckit-implement`. Use `/speckit-clarify`, `/speckit-checklist`, `/speckit-analyze` and
`/speckit-converge` on larger milestones (M4, M6) or whenever a spec feels ambiguous.

- One Spec Kit feature per milestone, named after it (for example `m1-lane-movement`).
- A spec describes what the player experiences and why. Technology choices belong in the plan.
- Every plan includes a Constitution Check against Principles I to VIII and the Code Style
  section, before research and again after design.
- Tasks are small enough for one session, and each task list ends with the game runnable.
- A milestone is done when the Quality Gate passes. Then commit, then update the project brief if
  the plan changed.
- If a session stalls, shrink the next step instead of pushing through.

## Governance

This constitution supersedes other practices within the project. The owner is the only decision
maker.

- Amendments: change this file, bump the version, update the Last Amended date, then check that
  the plan, spec and tasks templates still match.
- Versioning: MAJOR when a principle is removed or redefined; MINOR when a principle or section is
  added or materially expanded; PATCH for clarifications and typos.
- Violations: any plan that departs from a principle MUST record the departure in its Complexity
  Tracking table, with the reason and the simpler option that was rejected. An unrecorded
  departure is a defect.
- Compliance review: at the end of every milestone, check the finished work against Principles I
  to VIII and the Code Style section before starting the next milestone.
- Scope changes: moving an item between the must-have and Later lists is done in the project
  brief. Removing an item from the Not-doing list also requires amending Principle II.
- Amendment log:
  - 1.1.0 (2026-11-19): language changed from GDScript to Rust through gdext; toolchain, type
    and quality-gate constraints added.
  - 1.2.0 (2026-11-19): GDScript added for presentation and glue with a fixed Rust/GDScript
    split; typed GDScript required; platform set to Ubuntu 24.04 LTS.
  - 1.3.0 (2026-11-19): Code Style section added (Google Rust Style Guide, official GDScript
    style guide); Quality Gate made its own section with `cargo fmt --check`. Fixes: Principle I
    now allows technical checks beside player-visible ones (M0 needed this); the restart rule in
    Principle VIII starts at M3, when runs exist; cutout animation is described as posed in
    Godot's animation tools rather than by code; the dependency rule now names gdext as
    approved; the scope-change rules in Principle II and Governance no longer contradict each
    other.
  - 1.3.1 (2026-11-19): GDScript style rules checked against the guide's source file; code order
    corrected (annotations before `class_name`, public before private within each group), and
    layout, quote, operator, typing and comment rules added.
  - 1.3.2 (2026-11-19): target GPU corrected from Intel UHD 620 to Intel UHD 630, the GPU in
    the owner's machine, as reported by Godot at startup.

**Version**: 1.3.2 | **Ratified**: 2026-11-19 | **Last Amended**: 2026-11-19
