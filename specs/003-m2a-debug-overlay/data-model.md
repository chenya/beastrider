# Data Model: M2a Debug Overlay

Entities from the spec, who owns each one, and the rules that keep them valid. "Rules side" is
Rust; "presentation" is GDScript and the scene.

## TuningStore (rules side, Rust class, autoload `Tuning`)

The one place the current tuning lives ([research R2](research.md)).

| Field | Type | Rules |
| --- | --- | --- |
| `values` | `Option<TuningValues>` | `None` only if the start-up load failed and no reload has succeeded since. Replaced only by a fully valid `TuningValues` |
| `rejection` | `Option<TuningError>` | The error from the last load or reload; `None` after a success |

### States

```text
                 start-up load ok                     reload ok
  (created) ───────────────────────▶ Holding ─────────────────────▶ Holding (new values)
      │                              │   ▲
      │ start-up load fails          │   └── reload rejected: values unchanged, rejection set
      ▼                              │
  Empty (values None, rejection set) │
      │ reload ok ───────────────────┘
      └ reload rejected: stays Empty, rejection replaced
```

Two fields cover it; no state machine is needed (Principle III).

## RawTuning (rules side, plain struct)

The four numbers as read from the file, before any rule is applied: `horizontal_speed`,
`vertical_speed`, `lane_top`, `lane_bottom`, all `f64`. Parsing produces it or a `TuningError`
(file unreadable, invalid JSON, a section or field missing, a value that is not a number).

## TuningValues (rules side, plain `Copy` struct)

Replaces M1's `MovementTuning`. Built only by `validate(raw, screen_height)`, so an instance always
holds valid values.

| Field | File path | Rule | Starting value |
| --- | --- | --- | --- |
| `horizontal_speed` | `hero.horizontal_speed` | finite, `> 0` (px/s) | 300 |
| `vertical_speed` | `hero.vertical_speed` | finite, `> 0` (px/s) | 180 |
| `lane_top` | `lane.top` | finite, `>= 0`, `< lane.bottom` | 430 |
| `lane_bottom` | `lane.bottom` | finite, `> lane.top`, `<= screen height` | 700 |

The rules are M1's, unchanged (FR-009); only where they are checked moves.

## TuningError (rules side, enum, kept from M0/M1)

`Unreadable`, `InvalidJson`, `MissingField`, `InvalidValue`. New accessors give the two parts the
signal carries:

- `field()`: the field path (`lane.top`), or the file path for `Unreadable` and `InvalidJson`.
- `reason()`: why it was rejected, including the bad value where there is one
  (`must be less than lane.bottom (700), got 800`).

`Display` stays the full one-line message used in the output panel.

## Hero (rules side, Rust class, changed)

| Field | Type | Source | Rules |
| --- | --- | --- | --- |
| `player_index` | integer, exported | Scene (default 1) | `>= 1` |
| `setup` | `Option<Setup>` | `ready` | `None` if a non-tuning check failed; the hero then never moves (FR-010) |

`Setup` holds `actions` (four `StringName`s, as in M1), `half_width` (from `Body`) and
`tuning: Gd<TuningStore>`. It no longer holds tuning values or limits.

Each physics tick: if `setup` is `Some` and `tuning.values()` is `Some`, build `Speeds` and
`Limits` from the values, the half width and the screen width, and call `movement::step`
(unchanged). Otherwise stand still.

```text
            ready() checks ok                 every physics tick
  Loading ─────────────────────▶ Ready ──▶ Tuning has values? ── yes ──▶ move and clamp
     │                                         │
     │ ready() check fails                     no ──▶ stand still (moves once a reload succeeds)
     ▼
  Stopped (one error; never moves, whatever the tuning)
```

## Movement step (rules side, pure, unchanged)

`movement::step(position, input, speeds, delta, limits)` from M1, with its 11 tests.

## Debug overlay (presentation, `DebugOverlay` CanvasLayer + `debug_overlay.gd`)

Holds no game state. Its only fields are node references and UI state:

| Field | Kind | Meaning |
| --- | --- | --- |
| `hero` | `@export var hero: Hero` | Whose values to show (set in the editor) |
| `markers` | `@export var markers: Node2D` | The world markers it shows, hides and frees with itself |
| panel visible | the panel's `visible` | Hidden at start; F1 flips it |
| message timer | a one-shot timer | Clears "Tuning reloaded" after `RELOADED_MESSAGE_SECONDS` |

Constants: `RELOADED_MESSAGE_SECONDS := 2.0`, the "Tuning reloaded" text.

## World markers (presentation, `DebugMarkers` Node2D + `debug_markers.gd`)

Fields: `@export var hero: Hero`; `_drawn_position: Vector2`, the hero position at the last
redraw (a repaint cache, not game state). Constants: the cross size and bar thickness.

## Lane band (presentation, changed)

Reads `Tuning.lane_top()` / `lane_bottom()` in `_ready` and on `Tuning.tuning_reloaded`. Zero
height when `Tuning.has_values()` is false. No longer refers to the hero.

## Debug actions (Input Map)

| Action | Key (physical) |
| --- | --- |
| `debug_toggle_overlay` | F1 |
| `debug_reload_tuning` | F5 |
