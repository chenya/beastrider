# Contract: wave classes, signals and actions

What GDScript and the scenes may rely on. Everything else in Rust is private. Signals carry the
data their handlers need; a handler must not call back into the emitting object (or `Run`) during
the emit ([M3 research R8](../../005-m3-first-fight/research.md)). Unchanged M3 members (`Hero`,
`TrainingDummy`, `HitStopTimer`) are in the
[M3 contract](../../005-m3-first-fight/contracts/fight-classes.md).

## Autoloads (in order, unchanged)

`Tuning` (`TuningStore`), `HitStop` (`HitStopTimer`), `Run` (`RunTracker`).

## `Enemy` (was `Grunt`, extends `Node2D`)

Placed only by `WaveDirector`. Never give the root a script (M2b lesson); scripts go on children.

| Member | Kind | Contract |
| --- | --- | --- |
| `kind: EnemyKind` | export | `Grunt` or `Brute`; set in `grunt.tscn` / `brute.tscn`. Every value comes from `enemies.<kind>`. |
| `was_hit(direction: f32, strength: f32)` | signal | A blow landed; `strength` is the knockback after resistance (0 for an armored attack). |
| `died()` | signal | Emitted once, when its health reaches zero; it frees itself `run.death_fade_time` later. |
| `health() -> f32`, `max_health() -> f32` | func | For its bar. |
| `is_winding_up() -> bool` | func | True during the wind-up; drives the wind-up colour. |
| `is_alive() -> bool` | func | False once dying. |
| `is_waiting() -> bool` | func | True while it waits for a turn. For F2 and tests by playing. |
| `hurtbox() -> Rect2` | func | For F2; empty while entering or dying. |
| `attack_box() -> Rect2` | func | Its reach while its attack is active, else empty. For F2. |

## `WaveDirector` (new, extends `Node`; replaces `GruntSpawner`)

| Member | Kind | Contract |
| --- | --- | --- |
| `grunt_scene: PackedScene`, `brute_scene: PackedScene` | export | `res://world/grunt.tscn`, `res://world/brute.tscn`. |
| `world: Node2D` | export | The Y-sorted `World`; enemies are added before the hero. |
| `wave_started(number: int)` | signal | A wave's banner starts. |
| `wave_cleared(number: int)` | signal | The last enemy of a fully spawned wave died; the breather starts. |
| `wave() -> int` | func | The current wave number (1 from the start). |
| `banner_wave() -> int` | func | The wave number while its banner shows, else 0. For the banner. |
| `spawn_grunt()`, `spawn_brute()` | func | F3/F7: one more grunt or brute joins the current wave (Rust-side `spawn_now(kind)` behind both). Nothing at Game Over. |
| `skip_wave()` | func | F8: every enemy removed with no points or kills, queue emptied, next wave's banner at once. Nothing at Game Over. |

Runs after the enemies each physics frame (`process_physics_priority = 1`, placed after `World`),
and owns the attack tokens and the separation pass.

## `RunTracker` (extended, autoload `Run`)

| Member | Kind | Contract |
| --- | --- | --- |
| `game_over(kills: int, score: int, wave: int, best: int)` | signal | Once per run, when the hero falls. Was `game_over(kills)`. |
| `score_changed(score: int)` | signal | A kill added points. |
| `kills() -> int`, `score() -> int`, `wave() -> int`, `best() -> int` | func | For the HUD at start-up. `best` is the session best. |
| `is_playing() -> bool` | func | As M3. |
| `restart()` | func | As M3, and the best is kept (folded with this run's score first). |

## `TuningStore` (extended)

No new GDScript accessors. `warning()` now returns the first of the warnings in
[tuning-data.md](tuning-data.md). Rust-only: `waves() -> Option<WaveTuning>`.

## Groups (unchanged)

`team_players` (every `Hero`), `team_enemies` (every `Enemy` that has entered and is not dying,
every `TrainingDummy`).

## Input Map

| Action | Keys | Builds |
| --- | --- | --- |
| M3 actions | unchanged | as M3 |
| `debug_spawn_grunt` | F3 | debug |
| `debug_spawn_brute` | **F7** (new) | debug |
| `debug_next_wave` | **F8** (new) | debug |

## Scenes

- `godot/world/grunt.tscn`: root type `Enemy`, `kind = Grunt`; `Body` (`enemy_body.gd`, M3 shape and
  colour); `HealthBar` (`enemy_health_bar.gd`).
- `godot/world/brute.tscn` (new): the same nodes, `kind = Brute`, a body about 1.4 times the grunt's
  in a colour unlike the hero's, the grunt's and the dummy's, the bar raised to match.
- `godot/main.tscn`: `GruntSpawner` replaced by `WaveDirector` (after `World`); `Hud` gains
  `ScoreLabel` and `WaveBanner`; the overlay's `spawner` becomes `director`; `GameOver/Box` gains
  score, wave and best lines.

## Presentation scripts

| Script | Reads | Shows |
| --- | --- | --- |
| `ui/wave_banner.gd` | `director.banner_wave()` each frame | "Wave N" top centre while above 0 |
| `ui/score_label.gd` | `Run.score()` at ready, `Run.score_changed(score)` | "Score: N" near the kills |
| `ui/game_over.gd` | `Run.game_over(kills, score, wave, best)` | score, wave reached, kills, best, "Press R to restart" |
| `world/enemy_body.gd` | `Enemy.is_winding_up()`, `died` | wind-up colour, fade (was `grunt_body.gd`) |
| `world/enemy_health_bar.gd` | `Enemy.health()`, `max_health()` | bar above the head (was `grunt_health_bar.gd`) |
| `debug/debug_overlay.gd` | — | F3 → `director.spawn_grunt()`, F7 → `director.spawn_brute()`, F8 → `director.skip_wave()` |
| `debug/debug_hitboxes.gd` | `Enemy.hurtbox()`, `attack_box()` | outlines for both kinds |
| `world/screen_shake.gd` | `Enemy.was_hit` | shake for any enemy (was `Grunt`) |
