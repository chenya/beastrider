# Contract: fight classes, signals and actions

What GDScript and the scenes may rely on. Everything else in Rust is private. Signals carry the
data their handlers need; a handler must not call back into the emitting object (or `Run`) during
the emit ([research R8](../research.md)).

## Autoloads (in order)

| Autoload | Class | Scene |
| --- | --- | --- |
| `Tuning` | `TuningStore` | `res://autoload/tuning.tscn` (M2a) |
| `HitStop` | `HitStopTimer` | `res://autoload/hit_stop.tscn` (M2b) |
| `Run` | `RunTracker` | `res://autoload/run.tscn` (new) |

## `Hero` (extended)

| Member | Kind | Contract |
| --- | --- | --- |
| `player_index` | export | As M1. Health and damage belong to this hero (FR-014). |
| `was_hit(direction: f32, strength: f32)` | signal | A blow landed on this hero; `direction` is the push (−1 or 1), `strength` the knockback distance. |
| `died()` | signal | Emitted once, when the hero falls. |
| `health() -> f32`, `max_health() -> f32` | func | For the health bar. |
| `is_invulnerable() -> bool` | func | True while the post-hit invulnerability runs; drives the blink. |
| `is_dead() -> bool` | func | True once fallen. |
| `refill_health()` | func | F4: sets health to the maximum; does nothing while dead. |
| `hurtbox() -> Rect2` | func | Its body in screen pixels; empty without tuning. For F2. |
| `attack_box() -> Rect2` | func | As M2b: the reach while the swing is active, else empty. |
| `hits_landed() -> int` | func | As M2b; counts blows on grunts and dummies; resets with the scene. |
| `dummy_spawn_point() -> Vector2` | func | As M2b (F6 now); dummies are found in the `Enemies` group. |

## `Grunt` (new, extends `Node2D`)

Placed only by `GruntSpawner`. Never give the root a script (M2b lesson: a GDScript `_ready`
replaces the Rust `ready()`); scripts go on its children.

| Member | Kind | Contract |
| --- | --- | --- |
| `was_hit(direction: f32, strength: f32)` | signal | A blow landed on this grunt. |
| `died()` | signal | Emitted once, when its health reaches zero; it frees itself `run.death_fade_time` later. |
| `health() -> f32`, `max_health() -> f32` | func | For its bar. |
| `is_winding_up() -> bool` | func | True during the wind-up; drives the wind-up colour. |
| `is_alive() -> bool` | func | False once dying; the spawner counts living grunts with it. |
| `hurtbox() -> Rect2` | func | For F2; empty while entering or dying. |
| `attack_box() -> Rect2` | func | Its reach while its attack is active, else empty. For F2. |

## `TrainingDummy` (changed)

Joins the `Enemies` group instead of `hittable`. `was_hit` and `hurtbox()` as M2b. Its knockback
comes from the attacker's `attack.knockback_*` values.

## `GruntSpawner` (new, extends `Node`)

| Member | Kind | Contract |
| --- | --- | --- |
| `grunt_scene: PackedScene` | export | `res://world/grunt.tscn`. |
| `world: Node2D` | export | The Y-sorted `World`; grunts are added before the hero. |
| `spawn_now()` | func | F3: one more grunt, entering from a random edge at a random depth. |

## `RunTracker` (new, autoload `Run`)

| Member | Kind | Contract |
| --- | --- | --- |
| `game_over(kills: int)` | signal | Emitted once per run, when the hero falls. |
| `kills() -> int` | func | Grunts killed this run. |
| `is_playing() -> bool` | func | False after the hero has fallen, until the restart. |
| `restart()` | func | A fresh run: state and kills reset, freeze cleared, current scene reloaded. Called on `game_restart` in every build. |

## `HitStopTimer` (extended)

`clear()`: ends any freeze at once; used by `restart()`. `freeze()` and `is_frozen()` as M2b.

## `TuningStore` (extended)

New value accessors for presentation: `death_fade_time()`. `flash_time()`, `shake_strength()`,
`shake_time()` as M2b (now from `effects`). New `warning() -> String`: the warning of the last
successful load or reload, or `""` ([tuning-data.md](tuning-data.md)). `tuning_reloaded()` and
`tuning_rejected(field, reason)` unchanged.

## Groups (Rust constants)

| Group | Members |
| --- | --- |
| `team_players` | every `Hero` |
| `team_enemies` | every `Grunt` that has entered and is not dying, every `TrainingDummy` |

## Input Map

| Action | Keys | Builds |
| --- | --- | --- |
| `p1_left`, `p1_right`, `p1_up`, `p1_down`, `p1_attack` | as M1/M2b | all |
| `game_restart` | R | all |
| `debug_toggle_overlay`, `debug_reload_tuning`, `debug_show_hitboxes` | F1, F5, F2 | debug |
| `debug_spawn_grunt` | F3 | debug |
| `debug_refill_health` | F4 | debug |
| `debug_spawn_dummy` | **F6** (was F3) | debug |

All bind on any device (`device -1`), as before.

## Scenes

- `godot/world/grunt.tscn`: `Grunt` root; `Body` (`Polygon2D`, own colour, flash material local to
  scene, `grunt_body.gd`); `HealthBar` (`Node2D`, `grunt_health_bar.gd`).
- `godot/autoload/run.tscn`: one `RunTracker` node named `Run`.
- `godot/main.tscn`: the scene `Dummy` removed (clarification Q3); `GruntSpawner` added; `Hero/Body`
  gains the flash material and `hero_body.gd`; `Hud` gains `HealthBar` and `KillsLabel`; a
  `GameOver` CanvasLayer (hidden) is added; the debug `Panel` moves below the HUD; the overlay gains
  `spawner` and keeps `dummy_scene`.
