# Data Model: M1 Lane Movement

Entities from the spec, who owns each one, and the rules that keep them valid. "Rules side" is
Rust; "presentation" is GDScript and the scene.

## Hero (rules side, Rust class `Hero`)

| Field | Type | Source | Rules |
| --- | --- | --- | --- |
| `position` | `Node2D` position (x, y) | Start: placed in the scene. Then: the movement step | The origin is at the feet. After every tick: `half_width <= x <= viewport_width - half_width` and `lane_top <= y <= lane_bottom` |
| `player_index` | integer, exported | Scene (default 1) | `>= 1`. Picks the input actions `p{index}_left/right/up/down` |
| `actions` | four `StringName`s | Built once in `ready` from `player_index` | Each must exist in the Input Map, or the hero reports an error and stays still |
| `tuning` | `MovementTuning`, or none | Loaded once in `ready` | None until `ready` succeeds, and for good if it fails |
| `half_width` | number | Measured once in `ready` from the `Body` polygon | `> 0`, or the hero reports an error and stays still |

The hero's size and colour belong to its `Body` child (a `Polygon2D`) and are set in the scene.
They are looks, not tuning (Principle IV).

### States

```text
            ready() succeeds
  Loading ───────────────────▶ Active ──(every physics tick)──▶ Active
     │                           (reads input, moves, clamps)
     │ ready() fails
     ▼
  Stopped (one error in the output panel; never moves; lane_top() and lane_bottom() return 0)
```

Two states fit in `Option<MovementTuning>` (none = stopped), as in M0. No state machine is
needed (Principle III).

## Movement step (rules side, pure function in `movement.rs`)

Inputs: current position, input vector (from `Input.get_vector`, length ≤ 1), speeds, `delta`,
limits. Output: the new position.

```text
input   = input with any part that pushes into a limit already reached moved to the other axis
moved   = position + (input.x × horizontal_speed, input.y × vertical_speed) × delta
result  = (clamp(moved.x, limits.min_x, limits.max_x), clamp(moved.y, limits.min_y, limits.max_y))
```

Properties the unit tests hold it to: zero input returns the clamped position unchanged; a
diagonal never covers more distance than the faster straight direction; with V < H a vertical
move covers less distance than a horizontal one in the same time; a result is always inside the
limits, for any delta.

## MovementTuning (rules side, loaded from the tuning file)

| Field | File path | Rules | Starting value |
| --- | --- | --- | --- |
| `horizontal_speed` | `hero.horizontal_speed` | finite, `> 0` (px/s) | 300 |
| `vertical_speed` | `hero.vertical_speed` | finite, `> 0` (px/s) | 180 |
| `lane_top` | `lane.top` | finite, `>= 0`, `< lane.bottom` (px from the top of the screen) | 430 |
| `lane_bottom` | `lane.bottom` | finite, `> lane.top`, `<= viewport height` | 700 |

Built only through a checked constructor, so a value always holds valid numbers. The
`viewport height` rule is checked by the hero in `ready`, where the viewport is known. Every
failure is a `TuningError` naming the file, the field path and the bad value. See
[contracts/tuning-data.md](contracts/tuning-data.md).

## Lane (derived)

The lane is not a separate object: it is `lane_top` and `lane_bottom` from the tuning, with the
screen edges as its left and right limits. Two things read it: the hero's limits, and the lane
band.

## Pillar (presentation, scene only)

A `Polygon2D` child of `World`, origin at the base, shape drawn above it. Position and size are
set in the editor (clarification Q2). It has no script and does not block the hero
(clarification Q1). Its depth is its origin's y.

## Lane band (presentation, `ColorRect` + `lane_band.gd`)

Reads `hero.lane_top()` and `hero.lane_bottom()` once, after the hero is ready, and covers the
full screen width between them. Holds no game state. Zero height (not drawn) when the hero is
stopped.

## FPS counter (presentation, kept from M0)

Unchanged: `godot/ui/fps_label.gd` in the HUD.

## Removed from M0

`DriftingSquare`, its speed label, `SquareTuning` and the `drifting_square` tuning section.
