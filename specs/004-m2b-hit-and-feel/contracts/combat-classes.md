# Contract: combat classes exposed to Godot

What the Rust side exposes in M2b. See [research R2–R7](../research.md) and
[data-model.md](../data-model.md).

## `Hero` (extends `Node2D`, changed)

| Member | Kind | Meaning |
| --- | --- | --- |
| `player_index` | exported property | As M1; also picks `p{index}_attack` |
| `hits_landed()` | `#[func] -> int` | Dummies hit since the game started |
| `attack_box()` | `#[func] -> Rect2` | The swing's reach in screen px while `Active`; an empty `Rect2` otherwise. For F2 only |
| `dummy_spawn_point()` | `#[func] -> Vector2` | Where F3 should put a new dummy (clarification Q4) |

`ready()` now also checks the `p{index}_attack` action and the `HitStop` autoload
(`/root/HitStop` as `HitStopTimer`); a failure is one `Hero:` error and the hero never moves.

## `TrainingDummy` (extends `Node2D`, new)

| Member | Kind | Meaning |
| --- | --- | --- |
| `was_hit(direction: float, strength: float)` | signal | Emitted at the moment of each hit; `direction` −1 or 1 (push direction), `strength` the knockback distance in px |
| `hurtbox()` | `#[func] -> Rect2` | The hurtbox in screen px, from `Tuning`. For F2 only |

Joins the group `"hittable"` (Rust constant `HITTABLE_GROUP`) in `ready`. `take_hit` is Rust-only.

## `HitStopTimer` (extends `Node`, new; autoload `HitStop`)

| Member | Kind | Meaning |
| --- | --- | --- |
| `freeze(seconds: float)` | `#[func]` | Freeze play for at least `seconds`: the longer of the time left and `seconds`, never the sum. `0` does nothing |
| `is_frozen()` | `#[func] -> bool` | True while time is left |

Autoload: `[autoload] HitStop="*res://autoload/hit_stop.tscn"`, after `Tuning`.

## `TuningStore` (extended)

New read-only `#[func]`s for presentation: `flash_time()`, `shake_strength()`, `shake_time()`
(`0.0` without values). All other new values are Rust-only. File format and rules:
[tuning-data.md](tuning-data.md).

## Input Map actions (added)

| Action | Keys (physical, `device -1`, deadzone 0.2) |
| --- | --- |
| `p1_attack` | J, Z |
| `debug_show_hitboxes` | F2 |
| `debug_spawn_dummy` | F3 |
