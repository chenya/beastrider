# Data Model: M4a Waves and the Brute

Entities, states, transitions and rules for the [plan](plan.md). Godot-facing members are in
[contracts/wave-classes.md](contracts/wave-classes.md); tuning fields in
[contracts/tuning-data.md](contracts/tuning-data.md). Every closed set below is a Rust enum matched
exhaustively. M3's shared parts (`Health`, `AttackSpec`, `AttackTimer`, `Knockback`, `Countdown`,
`Team`, `resolve_attack`, `approach_step`) are unchanged
([M3 data model](../005-m3-first-fight/data-model.md)).

## EnemyKind and EnemyTuning

`EnemyKind { Grunt, Brute }`, exported on `Enemy` and set in its scene.

`EnemyTuning` (one per kind, same fields):

| Field | Type | Rule |
| --- | --- | --- |
| `horizontal_speed`, `vertical_speed` | f32 | `> 0` |
| `max_health` | f32 | `> 0` |
| `hurtbox_width`, `hurtbox_height` | f32 | `> 0` |
| `hit_stun_time` | f32 | `>= 0` |
| `knockback_resistance` | f32 | `0 <= r <= 1` |
| `interruptible` | bool | — |
| `attack_token_cost` | u32 | whole, `1 <= cost <= crowd.max_attack_tokens` |
| `points` | i64 | whole, `> 0` |
| `attack` | `AttackTuning` | as M3 |

`TuningValues::enemy(kind) -> EnemyTuning` is the only place that matches on the kind for values.

## Hit reaction (`rust/src/enemy.rs`, pure)

`Reaction { stun_time, knockback: KnockbackSpec, interruptible }` is built from the hit and the
enemy's tuning:

- `knockback.distance = hit.knockback.distance · (1 − knockback_resistance)`;
  `knockback.time = hit.knockback.time`.
- `stun_time = hit_stun_time · (1 − knockback_resistance)`.

`enemy_after_hit(mode, damage, reaction, fade_time) -> (EnemyMode, push: bool)`:

| Mode | `Damage` | Result |
| --- | --- | --- |
| any but `Entering`, `Dying` | `Killed` | `Dying(fade_time)`, push |
| `WindUp`, `Active`, `Recovery` | `Survived`, `interruptible = false` | unchanged, no push |
| `Waiting`, `Approaching`, `WindUp`, `Active`, `Recovery`, `HitStun` | `Survived`, otherwise | `HitStun(stun_time)`, push |
| `Entering`, `Dying` | any | unchanged, no push (they cannot be hit) |

## Enemy (was Grunt)

| Field | Type | Notes |
| --- | --- | --- |
| `kind` | `EnemyKind` | export |
| `mode` | `EnemyMode` | see below |
| `facing` | `Facing` | towards its target |
| `health` | `Health` | max from `enemy(kind).max_health` |
| `knockback` | `Option<Knockback>` | |
| `side` | f32 | −1 or +1: its entry edge, fixed at `place()` |
| `waiting_y` | f32 | its spawn depth, fixed at `place()` |

`EnemyMode { Entering, Waiting(waited), Retreating, Approaching, WindUp(Attack), Active(Attack),
Recovery(Attack), HitStun(Countdown), Dying(Countdown) }`; `holds_turn()` is true for
`Approaching`, `WindUp`, `Active` and `Recovery` only. `Waiting` carries the seconds waited.

| From | Event | To |
| --- | --- | --- |
| `Entering` | fully on screen | `Waiting(0)`; joins `team_enemies` |
| `Waiting` | `grant_turn()` from the director | `Approaching` |
| `Waiting`, `Retreating` | each tick | moves to its spot (`spot_step`, below) |
| `Retreating` | `back_at_spot` | `Waiting(0)` |
| `Approaching` | target in reach and depth | `WindUp` (new attack with `enemy(kind).attack`) |
| `WindUp` → `Active` → `Recovery` | timer phases | next phase |
| `Recovery` | timer ends | `Retreating` (turn freed) |
| any hittable | hit lands | `enemy_after_hit` (table above) |
| `HitStun` | countdown ends | `Retreating` |
| `Dying` | countdown ends | frees itself |

Waiting spot: `x = target.x + side · crowd.waiting_distance`, `y = waiting_y`, clamped to the lane
and the screen less half its width. `spot_step(position, spot, walk, drift_speed, radius, delta)`:
farther than `radius` (`crowd.separation_radius`) from the spot it walks at its own speeds; within
it, `drift_step` moves at most `drift_speed · delta` per axis (0 stops it). Never overshoots.
`back_at_spot(position, spot, target_x, waiting_distance, radius)`: within `radius` of the spot,
or `|x − target_x| >= waiting_distance − radius`.

On death it calls `Run.add_kill(points)` once, as M3's grunt called `add_kill()`.

## AttackTokens (`rust/src/crowd.rs`, pure)

`AttackTokens { capacity: u32, holders: Vec<(T, u32)> }`:

- `in_use() = Σ cost`; invariant `in_use() <= capacity` after every call.
- `set_capacity(c)`: takes effect for later grants; holders keep their tokens.
- `reconcile(still_holding: &[T])`: drops every holder not in the list.
- `grant(waiting: &[TurnRequest<T>], max_wait) -> Vec<T>`, `TurnRequest { id, cost, distance,
  waited }`: takes `turn_order` (those with `waited >= max_wait` first, longest first; then nearest
  first; ties: lower id), then for each in order: if `in_use() + cost <= capacity`, holds it and
  returns it; otherwise stops.

## Separation (`rust/src/crowd.rs`, pure)

`Body { x, y, give }`, `Give { Holds, Presses, Yields }`. `separate(bodies, radius, depth_scale)
-> Vec<(dx, dy)>`:

- For each pair with `d = hypot(Δx, Δy · depth_scale) < radius`: overlap `o = radius − d`, along the
  unit line from one to the other in scaled space; all to the one that gives way more, `o/2` each
  between equals, none between two that hold. `d == 0`: split along x, the lower index to the left.
- Offsets are summed per body; depth offsets are divided back by `depth_scale`.
- `radius == 0` returns all zeros.

Yields: `Waiting`, `Retreating`, `HitStun`. Presses: `Approaching`. Holds: `WindUp`, `Active`,
`Recovery`. Left out: `Entering`, `Dying`. The director applies each offset with `Enemy::nudge(dx, dy)`, which clamps.

## Waves (`rust/src/wave_director.rs`, pure)

`WaveSpec { grunts: u32, brutes: u32 }`, `WaveTuning { list: Vec<WaveSpec>, extra_grunts_per_wave,
brute_every_waves, spawn_interval, breather_time, banner_time, max_on_screen }`.

`composition(n, waves) -> WaveSpec` (n from 1):

- `n <= L` (list length): `list[n − 1]`.
- `n > L`: `grunts = list[L − 1].grunts + (n − L) · extra_grunts_per_wave`;
  `brutes = list[L − 1].brutes + (n − L) / brute_every_waves` (integer division; 0 → no extra).

`spawn_queue(spec) -> Vec<EnemyKind>`: length `t = grunts + brutes`; brute `i` (0-based, of `b`)
at index `round((i + 1) · t / (b + 1)) − 1`, grunts elsewhere, in order.

`WaveState { Banner(Countdown), Fighting, Breather(Countdown) }`;
`WaveProgress { number: u32, state: WaveState, queue: Vec<EnemyKind>, spawn_timer: Countdown }`.

`tick(progress, living, ready, waves, delta) -> (WaveProgress, WaveEvent)`, with
`WaveEvent { None, Spawn(EnemyKind), Cleared(n), Started(n) }` and `ready = tuning valid && playing`:

| From | Condition | To | Event |
| --- | --- | --- | --- |
| any | not `ready` | unchanged | `None` |
| `Banner` | countdown ends | `Fighting`, timer 0 | `None` |
| `Fighting` | queue not empty, timer out, `living < max_on_screen` | same, timer = `spawn_interval`, front popped | `Spawn(kind)` |
| `Fighting` | queue empty, `living == 0` | `Breather(breather_time)` | `Cleared(n)` |
| `Breather` | `living > 0` | `Fighting` (same `n`, empty queue) | `None` |
| `Breather` | countdown ends | `Banner(banner_time)`, `n + 1`, `queue = spawn_queue(composition(n + 1))` | `Started(n + 1)` |

Start: `WaveProgress::first()` = `Banner(banner_time)`, `n = 1`, wave 1's queue filled, emitting
`Started(1)` on the first ready tick. A wave's queue is filled as its banner starts, from the values
in use then, so a reload during a banner applies from the next wave (the contract's "the next wave
whose banner starts"); kinds pushed during the banner go in front of it. Outside the table: `push_front(kind)` (F3/F7: joins the queue at the front, timer
set to 0; from `Breather` it returns to `Fighting`) and `skip()` (F8: queue emptied, `Banner`,
`n + 1` with its own queue, `Started(n + 1)`).

`living` = the director's enemies (`Enemy` children of `World`) that are not `Dying`.

## Run (extended)

`RunRecord { state: RunState, kills: i64, score: i64, wave: u32, best: i64 }`:

- `add_kill(points)`: while `Playing`, `kills += 1`, `score += points`; returns true if counted (the
  tracker then emits `score_changed(score)`).
- `set_wave(n)`: while `Playing`, `wave = n`.
- `end_run() -> bool`: `Playing → GameOver` once; `best = max(best, score)`.
- `reset()`: `best = max(best, score)`; then `Playing`, `kills = 0`, `score = 0`, `wave = 0`.
  `best` is never reset.

## Tuning

`validate(raw, screen_height) -> Result<(TuningValues, WaveTuning), TuningError>`, all or nothing.
`TuningValues` stays `Copy` and gains `enemy(kind)` and `crowd() -> CrowdTuning { max_attack_tokens,
waiting_distance, drift_speed, separation_radius, max_wait_time }`; `grunt()` and `run().respawn_delay()` go.
`warnings(values)` returns, in order, `EnemyWindUpNotLonger { kind, hero, enemy }` for each kind and
`BruteWindUpNotLonger { grunt, brute }`. Rules in [contracts/tuning-data.md](contracts/tuning-data.md).
