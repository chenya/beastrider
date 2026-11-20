# Contract: `TuningStore` and the `Tuning` autoload

What the Rust side exposes to Godot for tuning in M2a. GDScript reaches it through the autoload
name `Tuning`; Rust reaches it at `/root/Tuning`. See [research R2–R5](../research.md).

## Autoload

| Item | Value |
| --- | --- |
| Scene | `res://autoload/tuning.tscn`: one node, type `TuningStore`, named `Tuning` |
| Project setting | `[autoload] Tuning="*res://autoload/tuning.tscn"` (enabled as a global) |
| Order | The only autoload; ready before the main scene |

The autoload name must not equal any class name (research R2).

## Class `TuningStore` (extends `Node`)

### Lifecycle

- `ready()`: loads the file once (same as `reload()`, but without emitting a signal: nobody is
  listening yet).

### Methods (`#[func]`)

| Method | Returns | Meaning |
| --- | --- | --- |
| `reload()` | nothing | Rereads and checks the whole file. On success, replaces all values, clears the rejection and emits `tuning_reloaded`. On failure, keeps every value, stores the rejection, writes `Tuning: <message>` to the output panel and emits `tuning_rejected` |
| `has_values()` | `bool` | `false` only while no load has ever succeeded |
| `horizontal_speed()` | `float` | px/s; `0.0` when `has_values()` is false |
| `vertical_speed()` | `float` | px/s; `0.0` when `has_values()` is false |
| `lane_top()` | `float` | px from the top of the screen; `0.0` when `has_values()` is false |
| `lane_bottom()` | `float` | px from the top of the screen; `0.0` when `has_values()` is false |
| `rejected_field()` | `String` | Field path of the last rejection (`lane.top`), the file path for a file-level error, or `""` after a success |
| `rejected_reason()` | `String` | Why it was rejected, or `""` after a success |

All values are read-only from Godot: only the tuning file sets them.

### Signals

| Signal | Arguments | Emitted |
| --- | --- | --- |
| `tuning_reloaded` | none | After a successful `reload()`, once the new values are in place |
| `tuning_rejected` | `field: String`, `reason: String` | After a failed `reload()`, with the old values still in place |

Handlers may call back into `Tuning` (research R5).

### Rust-side API (used by `Hero`)

`values() -> Option<TuningValues>`: the current values, copied. Not exposed to Godot.

## Rules applied on every load

Unchanged from M1 ([M1 contract](../../002-m1-lane-movement/contracts/tuning-data.md)):

| Field | Rule |
| --- | --- |
| `hero.horizontal_speed` | number, finite, `> 0` |
| `hero.vertical_speed` | number, finite, `> 0` |
| `lane.top` | number, finite, `>= 0`, `< lane.bottom` |
| `lane.bottom` | number, finite, `<= screen height` |

The file format and starting values are unchanged. Unknown keys are ignored.

## Message examples

| Situation | `field` | `reason` |
| --- | --- | --- |
| `lane.top` 800, `lane.bottom` 700 | `lane.top` | `must be less than lane.bottom (700), got 800` |
| `hero.vertical_speed` 0 | `hero.vertical_speed` | `must be a finite number > 0, got 0` |
| `hero.horizontal_speed` `"fast"` | `hero.horizontal_speed` | `must be a number, got String` |
| `lane.top` deleted | `lane.top` | `missing` |
| A syntax error on line 4 | `res://data/tuning.json` | `not valid JSON (line 4): <parser message>` |

Exact wording may change in implementation; each message names the field (or file) and the
reason, and the bad value where there is one.

## Hero changes

- `ready()` checks player index, input actions, `Body` width, and that `/root/Tuning` exists and
  is a `TuningStore`. A failure is `Hero: <message>`, and the hero never moves.
- Each physics tick reads `Tuning`'s values; no values means standing still that tick.
- `lane_top()` and `lane_bottom()` are removed from `Hero`.
