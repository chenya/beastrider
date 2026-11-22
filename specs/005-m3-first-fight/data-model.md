# Data Model: M3 First Fight

Entities, states, transitions and rules for the [plan](plan.md). Godot-facing members are in
[contracts/fight-classes.md](contracts/fight-classes.md); tuning fields in
[contracts/tuning-data.md](contracts/tuning-data.md). Every closed set below is a Rust enum matched
exhaustively.

## Shared parts (`rust/src/fighter.rs`, pure, unit-tested)

### Health

| Field | Type | Rule |
| --- | --- | --- |
| `current` | f32 | `0 <= current <= max` |
| `max` | f32 | from tuning, `> 0` |

- `damage(amount) -> Damage`: `Survived` if `current` stays above 0; `Killed` the one time it
  reaches 0 (overkill clamps to 0); `AlreadyDead` if it was 0 already. So a death is reported
  exactly once.
- `refill()`: `current = max`. `set_max(m)`: `max = m`, `current = min(current, m)`.

### AttackSpec and KnockbackSpec

`AttackSpec { wind_up, active, recovery, reach, height, damage, knockback: KnockbackSpec }`,
`KnockbackSpec { distance, time }`, built by one reader from either fighter's `attack` section.

### AttackTimer

`AttackTimer { phase: AttackPhase, elapsed, spec: AttackSpec }`,
`AttackPhase { WindUp, Active, Recovery }`. `advance(delta) -> AttackAdvance { step, was_active }`
with `step: Continue | Finished`. Zero-length parts are passed in the same tick; `was_active` is
true if the attack was in `Active` at any moment of the tick (M2b rules, moved). The spec is copied
at the start, so a reload applies from the next attack.

### Knockback

`Knockback { start_x, direction, distance, time, elapsed }`;
`x = start_x + direction · knockback_offset(distance, time, elapsed)` (M2b's even deceleration);
ends when `elapsed >= time` or the clamp stops it. A new hit replaces a running slide. The slide
waits while play is frozen, so it starts when the freeze ends.

### Countdown

`Countdown { remaining }`: `new(s)`, `tick(delta) -> bool` (true on the tick it reaches 0 and on
every tick after, so a countdown of 0 ends at once), `is_running()`. Used for hit-stun,
invulnerability, dying and the respawn delay.

### Team, Attacker, Target, Hit

- `Team { Players, Enemies }`; `other()`; `group()` → `"team_players"` / `"team_enemies"`.
- `Attacker { team, origin, facing, half_width, spec, depth_tolerance }`.
- `Target<T> { id: T, team, hurtbox: Rect, feet_y, alive, invulnerable }`.
- `Hit<T> { target: T, damage, direction, knockback: KnockbackSpec }`, `direction = facing.sign()`.
- `resolve_attack(attacker, targets, already_hit) -> Vec<Hit<T>>`: targets of
  `attacker.team.other()` that are `alive`, not `invulnerable`, not in `already_hit`, and pass M2b's
  `lands` rule (attack box overlaps the hurtbox, touching counts; `|attacker y − target y| <=
  depth_tolerance`).

### approach_step

`approach_step(position, target, side, reach, own_half_width, target_half_width, speeds,
depth_tolerance, delta) -> (x, y)`:

- slot x = `target.x + side · (target_half_width + reach + own_half_width)`: there the grunt's
  attack box just touches the target's hurtbox, which counts as a hit (M2b); slot y = `target.y`.
  No extra margin, so no untuned constant.
- If `|y − target.y| > depth_tolerance`: move only vertically towards `target.y` at the vertical
  speed. Otherwise move towards the slot on both axes (horizontal speed, vertical speed), never
  overshooting.

`side = sign(position.x − target.x)`, `+1` when equal.

## Hero (extended)

| Field | Type | Notes |
| --- | --- | --- |
| `mode` | `HeroMode` | see below |
| `facing` | `Facing` | as M2b |
| `health` | `Health` | max from `hero.max_health` every tick (`set_max`) |
| `invulnerable` | `Countdown` | runs alongside every mode |
| `knockback` | `Option<Knockback>` | runs alongside every mode |
| `hits_landed` | i64 | as M2b |

`HeroMode { Free, Attacking(Swing), HitStun(Countdown), Dead }`;
`Swing { timer: AttackTimer, queued: bool, hit_this_swing: Vec<InstanceId> }`.

| From | Event | To |
| --- | --- | --- |
| `Free` | attack pressed | `Attacking` (new swing with `hero.attack`) |
| `Attacking` | attack pressed in recovery | same, `queued = true` |
| `Attacking` | timer `Finished`, not queued / queued | `Free` / new `Attacking` |
| `Free`, `Attacking`, `HitStun` | hit lands, `Survived` | `HitStun(hero.hit_stun_time)`; invulnerability starts; knockback starts |
| `Free`, `Attacking`, `HitStun` | hit lands, `Killed` | `Dead`; knockback starts; `died()`; `Run.end_run()` |
| `HitStun` | countdown ends | `Free` |
| `Dead` | (anything) | `Dead` until the scene reloads |

A hit lands only if the hero is not `Dead` and not invulnerable (`resolve_attack` filters both).
While frozen by `HitStop`, nothing advances. Outside `Playing`, input is ignored.

## Grunt (new)

| Field | Type | Notes |
| --- | --- | --- |
| `mode` | `GruntMode` | see below |
| `facing` | `Facing` | towards its target |
| `health` | `Health` | max from `grunt.max_health` |
| `knockback` | `Option<Knockback>` | |
| `edge` | `Facing` | the edge it enters from (only used while `Entering`) |

`GruntMode { Entering, Approaching, WindUp(Attack), Active(Attack), Recovery(Attack),
HitStun(Countdown), Dying(Countdown) }`, `Attack { timer: AttackTimer, hit_this_attack: bool }`.

| From | Event | To |
| --- | --- | --- |
| `Entering` | fully on screen | `Approaching`; joins `team_enemies` |
| `Approaching` | target in reach and depth (the attack would land now) | `WindUp` (new attack with `grunt.attack`) |
| `WindUp` → `Active` → `Recovery` | timer phases | next phase; `Recovery` ends → `Approaching` |
| `Active` | the attack lands on the target (once) | same; target takes the `Hit` |
| `Approaching`, `WindUp`, `Active`, `Recovery`, `HitStun` | hit lands, `Survived` | `HitStun(grunt.hit_stun_time)`; attack dropped; knockback starts |
| same | hit lands, `Killed` | `Dying(run.death_fade_time)`; leaves `team_enemies`; `died()`; `Run.add_kill()` |
| `HitStun` | countdown ends | `Approaching` |
| `Dying` | countdown ends | frees itself |

Outside `Playing`, every mode but `Dying` does nothing. No target (no living hero): it stands
still in `Approaching`. The pure transition `grunt_after_hit(mode, damage) -> GruntMode` is
unit-tested (a hit in any attack mode cancels it).

## GruntSpawner (new)

`SpawnerState { AwaitingFirst, Watching, Waiting(Countdown) }`; pure `tick(state, living_grunts,
tuning_ready, playing, delta) -> (SpawnerState, spawn: bool)`, where `living_grunts` counts the
`Grunt` children of `World` that are not `Dying` (entering grunts count):

| From | Condition | To | Spawn |
| --- | --- | --- | --- |
| `AwaitingFirst` | tuning valid and playing | `Watching` | yes |
| `Watching` | no living grunt | `Waiting(run.respawn_delay)` | no |
| `Waiting` | a grunt is alive again (F3) | `Watching` | no |
| `Waiting` | countdown ends | `Watching` | yes |
| any | not playing, or tuning invalid | unchanged | no |

A spawn picks an edge (`randi_range(0, 1)`) and a depth (`randf_range(lane.top, lane.bottom)`).
`spawn_now()` spawns the same way, outside the table.

## Run (new, autoload)

`RunTracker` wraps a pure `RunRecord { state: RunState, kills }`, `RunState { Playing, GameOver }`:

- `add_kill()`: `kills += 1` while `Playing`. A grunt adds its kill when it starts dying, so one
  still fading when the hero falls has already counted.
- `end_run() -> bool`: `Playing → GameOver`, true the first time only (so `game_over` is emitted
  once).
- `reset()`: `Playing`, `kills = 0`. `restart()` = `reset()` + `HitStop.clear()` + reload the
  current scene.

## Tuning (restructured)

`TuningValues` gains `hero()`, `grunt()` (each `FighterTuning { speeds, max_health, hurtbox,
hit_stun_time, invulnerability_time (hero only), attack: AttackSpec }`), `dummy()`, `lane()` (with
`depth_tolerance`), `effects()` and `run()`. `validate` is all or nothing as before;
`warnings(&values)` returns `TuningWarning::GruntWindUpNotLonger { hero, grunt }` when it
applies. Full rules in [contracts/tuning-data.md](contracts/tuning-data.md).
