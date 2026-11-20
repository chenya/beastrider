# Implementation Plan: M2a Debug Overlay

**Branch**: `feat/m2a-debug-overlay` | **Date**: 2026-11-20 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/003-m2a-debug-overlay/spec.md`

## Summary

F1 shows a debug panel and world markers; F5 rereads the tuning file and applies it at once, or
rejects it with a message and keeps the old values. The approach:

- **One source of truth for tuning** (the problem: the M1 hero copies its tuning once, so a
  reload could never reach it). A Rust class `TuningStore` loads, checks and holds the current
  values and is the root of a one-node autoload scene named `Tuning`. The hero reads its speeds
  and limits from `Tuning` every physics tick; the lane band and markers read it when they draw
  ([research R2, R6](research.md)).
- **All-or-nothing reload**: parse into a raw struct, check it with a pure
  `validate(raw, screen_height)` covered by `cargo test`, and swap only on success
  ([R3](research.md)). JSON needs no cache bypass ([R4](research.md)).
- **Signals** `tuning_reloaded()` and `tuning_rejected(field, reason)` tell the presentation side
  what happened; Rust does not know who listens ([R5](research.md)).
- **Overlay in GDScript**: a `CanvasLayer` with a top-left panel (FPS, position, player, speeds,
  tuning state) and a separate message spot; a `Node2D` above the Y-sorted world draws the lane
  bars and the origin cross with `_draw()`, redrawing only on change ([R7, R8](research.md)).
- **Keys** are the Input Map actions `debug_toggle_overlay` (F1) and `debug_reload_tuning` (F5),
  handled in `_unhandled_input` ([R7](research.md)).
- **Release builds**: the overlay frees itself and the markers when `OS.is_debug_build()` is
  false ([R10](research.md)).

## Technical Context

**Language/Version**: Rust 1.100.0 for rules and state; GDScript (Godot 4.7.2) for presentation.
Unchanged.

**Primary Dependencies**: Godot **4.7.2-stable**; gdext `godot = "=0.5.5"` with `api-4-6`. No new
crates or addons. The owner's input says "Godot 4.6.x"; M0 pinned 4.7.2 with API level 4.6, as
confirmed during M1 ([R1](research.md)).

**Storage**: `godot/data/tuning.json`, read at start-up and on each F5. Format unchanged.

**Testing**: `cargo test` for `validate` (one pass and one fail per rule) and the unchanged
movement tests; everything else by playing ([quickstart.md](quickstart.md) scenarios 1–17).

**Target Platform**: Ubuntu 24.04 LTS, Intel UHD 630, stock Mesa, through XWayland. Release-build
check needs the 4.7.2 export templates (one-time setup, [R10](research.md)).

**Project Type**: Desktop game: a Godot project plus a Rust GDExtension library.

**Performance Goals**: A steady 60 fps at 1280×720 with V-Sync on, with the overlay open.

**Constraints**: Compatibility renderer only; no gameplay tuning values in code; GDScript holds no
game state; keyboard only; nothing from the spec's out-of-scope list. Sized for about one session
(the brief's estimate for M2a), two at most.

**Scale/Scope**: 1 new Rust class (`TuningStore`), `tuning.rs` restructured (`RawTuning`,
`TuningValues`, `validate`), `Hero` simplified; 2 new GDScripts (`debug_overlay.gd`,
`debug_markers.gd`), `lane_band.gd` changed; 1 new autoload scene; `main.tscn` gains 2 nodes and
their children; 2 Input Map actions; an export preset.

All unknowns are resolved in [research.md](research.md); none remain marked NEEDS CLARIFICATION.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.3.3. ✅ = passes, ➖ = does not apply to M2a.

| Rule | Pre-research | Post-design | How M2a meets it |
| --- | --- | --- | --- |
| **I. Fun First, Playable Always** | ✅ | ✅ | The game stays playable as in M1; five of the ten checks are player-visible (F1, live values, bars match the stops, F5 changes speed, a bad value is rejected). The point is faster tuning by feel (SC-009). |
| **II. One Milestone at a Time** | ✅ | ✅ | Only the brief's M2a row: overlay, markers, live reload, absent from release. No hitboxes, enemies, health, magic, dummy or in-overlay editing (FR-015). About one session. |
| **III. Architecture Serves the Game** | ✅ | ✅ | Each piece names its problem: `TuningStore` autoload because the hero's one-time copy means a reload could never reach it; `validate` as a pure function because the rules must be the same at start-up and reload and testable without the engine; signals because the overlay and band must react to a reload without Rust knowing they exist. Not added: a notification path into the hero (it reads every tick), a state machine (two `Option`s), a debug framework for future cheats (they arrive with their milestones). |
| **IV. Data-Driven Tuning** | ✅ | ✅ | Gameplay values stay in `tuning.json` with no defaults in code. The 2-s message time, bar thickness and cross size are debug-UI presentation, kept as named constants in the scripts ([R7](research.md)). |
| **V. Runs Well on Weak Hardware** | ✅ | ✅ | Six labels, two bars and a cross; markers redraw only on change; no shaders. Checked on the UHD 630 with the overlay open for 60 s (quickstart 13). |
| **VI. Original Art, Cheap Animation** | ✅ | ✅ | Placeholder UI only; no outside assets. |
| **VII. Co-op Ready, Local Only** | ✅ | ✅ | The hero keeps `player_index`; the overlay shows the player index of the hero it is linked to, not "the player". Tuning is shared game data, not per player. Debug actions are not player actions, so they carry no index. No networking. |
| **VIII. Always Debuggable** | ✅ | ✅ | v1.3.3: the overlay is required from M2a, shown and hidden by a key, absent from release builds (R10). Cheats wait for M2b, M3, M4. Restart key starts at M3. Work committed on the branch. |
| **Tech: engine and API level** | ✅ | ✅ | Engine 4.7.2, API level 4.6, unchanged. |
| **Tech: Rust/GDScript split** | ✅ | ✅ | Rust parses, checks and holds tuning and moves the hero. GDScript shows the overlay, draws markers, places the band and turns keys into calls; it stores no game state (the markers' last-drawn position is a repaint cache). The constitution names the debug overlay as GDScript. |
| **Tech: types over strings** | ✅ | ✅ | Tuning errors stay a Rust enum; the signal carries its two parts as strings only at the GDScript boundary. Action names appear once each in the overlay script. |
| **Tech: dependencies** | ✅ | ✅ | gdext only. Export templates are Godot's own and are not a dependency of the game. |
| **Tech: one tuning-data format** | ✅ | ✅ | Same JSON file, same sections, same rules. |
| **Code Style: Rust** | ✅ | ✅ | rustfmt at 100 columns; `try_new`/accessor naming; `Result` for every load failure, no panics on data; no new `unsafe`; `///` docs on `TuningStore`, its methods and signals. |
| **Code Style: GDScript** | ✅ | ✅ | Typed exports (`hero: Hero`, `markers: Node2D`), constants in CONSTANT_CASE, `##` docs, code order per the constitution, untyped declarations still errors. |
| **Code Style: shared naming** | ✅ | ✅ | `TuningStore`, `Hero` in Rust, the editor and GDScript; signal names identical on both sides. The autoload name `Tuning` differs from the class on purpose ([R2](research.md)). |
| **Quality Gate** | ✅ | ✅ | `cargo fmt --check`, build and clippy with 0 warnings, `cargo test`, 0 GDScript warnings, and every check passed by playing (quickstart 1–17). |

**Result**: both passes are green. The T028 review after implementation found the work matches
every row above; its one departure (scenes and settings changed by the agent, at the owner's
request) is recorded below, as in M1. It also fixed an M0 bug the on-screen messages exposed:
Godot counts JSON error lines from 0, so messages now add 1 to match the editor.

## Spec and input alignment

The plan follows the spec and the owner's answers where the technical context disagreed:

- **FPS counter stays** (clarification Q2, confirmed by the owner during planning). The input
  said it moves into the overlay; `Hud/FpsLabel` is unchanged and the overlay also lists FPS.
- **A rejection message stays until the next reload** (FR-008, confirmed by the owner during
  planning). Only "Tuning reloaded" fades, after `RELOADED_MESSAGE_SECONDS`.
- **The message spot is separate from the panel** (clarification Q3), so F5 gives feedback with
  the panel closed.
- **Class name `TuningStore`, autoload `Tuning`**: the input names both `Tuning`, which breaks
  typed GDScript ([R2](research.md)).
- **`validate` takes the screen height** as well as the values, so the "lane inside the screen"
  rule is applied identically at start-up and reload ([R3](research.md)).

## Project Structure

### Documentation (this feature)

```text
specs/003-m2a-debug-overlay/
├── spec.md              # Feature spec (/speckit-specify, /speckit-clarify)
├── plan.md              # This file
├── research.md          # Phase 0: versions, autoload, reload, signals, overlay, markers, release
├── data-model.md        # Phase 1: entities, owners, rules, states
├── quickstart.md        # Phase 1: build, export setup, 17 validation scenarios
├── contracts/
│   ├── tuning-store.md  # TuningStore class, Tuning autoload, signals, messages, Hero changes
│   └── debug-overlay.md # Input actions, scene tree, overlay and marker behaviour
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
rust/src/
├── lib.rs                 # + mod tuning_store
├── tuning.rs              # CHANGED: RawTuning, TuningValues, validate(); parsing kept; tests per rule
├── tuning_store.rs        # NEW: TuningStore class, reload(), accessors, signals
├── hero.rs                # CHANGED: reads Tuning every tick; lane_top/lane_bottom removed
└── movement.rs            # unchanged

godot/
├── project.godot          # CHANGED: [autoload] Tuning; [input] debug_toggle_overlay, debug_reload_tuning
├── export_presets.cfg     # NEW: Linux preset (for the release-build check)
├── main.tscn              # CHANGED: + DebugMarkers, + DebugOverlay (Panel, Message)
├── autoload/
│   └── tuning.tscn        # NEW: one TuningStore node
├── debug/
│   ├── debug_overlay.gd   # NEW: panel, message spot, F1/F5, release check
│   └── debug_markers.gd   # NEW: lane bars and origin cross
├── world/
│   └── lane_band.gd       # CHANGED: reads Tuning, follows tuning_reloaded
├── ui/
│   └── fps_label.gd       # unchanged
└── data/tuning.json       # unchanged
```

**Structure Decision**: Same two-folder layout. Debug scripts get their own `godot/debug/` folder
so they are easy to find and clearly not gameplay; the autoload scene gets `godot/autoload/`.
Rust gets one new module so `tuning.rs` stays the pure data-and-rules file.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
| --- | --- | --- |
| `main.tscn`, `autoload/tuning.tscn` and the Input Map were changed by the agent rather than by hand in the editor (Tech: "Scenes, nodes and animation are built in the Godot editor"), as in M0 and M1. The owner handed T002, T006, T008 and T012 over at the start of `/speckit-implement`. T002 (actions) and T006 (autoload) used `ProjectSettings`; T006 and T012 built scenes with `PackedScene.pack` + `ResourceSaver.save`; T008's two-line removal of `LaneBand.hero` was a direct text edit of `main.tscn`. | The agent cannot use the editor GUI, and godot-mcp's scene tools fail on this project's untyped-declaration error. The result is an ordinary `.tscn` that opens and edits in the editor. | Building by hand in the editor is the default and stays available; relaxing the untyped-declaration error to let godot-mcp run was rejected because it would break the GDScript style rule. |

## Follow-ups (outside M2a)

- **Cheats**: show hitboxes and spawn an enemy in M2b, refill health in M3, refill magic in M4
  (constitution v1.3.3).
- **Automatic reload on file save**: not requested; F5 stays the only trigger.
- **Export preset**: kept for later packaged builds (the brief's Later list).
