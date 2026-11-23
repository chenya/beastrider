# Research: M4a Waves and the Brute

Phase 0 of the [plan](plan.md). Each section records a decision, why, and what was rejected. The
owner's technical context is the starting point; where it disagreed with the spec, the owner chose
during planning (see [plan.md, Spec and input alignment](plan.md#spec-and-input-alignment)).

## R1. Versions, platform and hardware

- **Decision**: Everything pinned in M0–M3 stays: Godot 4.7.2, gdext `=0.5.5` with `api-4-6`, Rust
  1.100.0, Compatibility renderer, 1280×720, V-Sync on. No new crate or addon; randomness stays
  with Godot's `RandomNumberGenerator`. The ten-enemy frame-rate check runs on the **UHD 630**.
- **Rationale**: The spec reads the owner's check 12 ("UHD 620") as the 630, per constitution
  1.3.2 and M3's test machine.
- **Alternatives**: none considered; nothing in M4a needs a new dependency.

## R2. One enemy class, two kinds

- **Decision**: M3's `Grunt` becomes `Enemy` (`rust/src/enemy.rs`, replacing `grunt.rs`) with an
  exported `kind: EnemyKind { Grunt, Brute }`. Every value it uses comes from
  `values.enemy(kind) -> EnemyTuning`, one struct with the same fields for both kinds: speeds,
  health, hurtbox, hit-stun time, `knockback_resistance`, `interruptible`, `attack_token_cost`,
  `points` and the nested `attack`. Behaviour code calls `values.enemy(self.kind)` and never matches
  on the kind. `EnemyKind` is matched only in `TuningValues::enemy` (which section to read) and in
  the wave queue (which scene to spawn), both exhaustive. Two scenes, `grunt.tscn` and `brute.tscn`,
  differ only in `kind`, polygon size and colour.
- **Rationale**: The owner's problem statement: the brute must reuse the grunt's hunting, attack,
  stun and hit rules with its own numbers, without copying grunt code (spec FR-009, check 11). A
  kind read through one tuning accessor keeps every rule in one place, so a fix to one kind fixes
  both.
- **Alternatives**: a second `Brute` class sharing a trait (rejected: two Godot classes, two
  `FighterRef` arms, and the copy check 11 forbids); a stats resource per scene (rejected: values
  belong in `tuning.json`, Principle IV).

## R3. The brute's weight: resistance and armored attacks

- **Decision**: Two per-kind values decide how a blow affects an enemy:
  - `knockback_resistance` (0 to 1): the knockback distance and the hit-stun time it takes are
    multiplied by `1 − knockback_resistance` (owner's input). The knockback time is unchanged, so a
    resisted push is shorter, not slower. Freeze, flash and shake play in full.
  - `interruptible` (true or false): when false, a blow during the enemy's wind-up, active part or
    recovery does damage and plays the effects but neither pushes nor stuns it, and its attack
    goes on (spec clarification Q1). Outside an attack, or when true, the blow pushes and stuns as
    scaled above, cancelling any attack, as in M3.
  The pure function `enemy_after_hit(mode, damage, reaction) -> (EnemyMode, push: bool)` applies
  both rules and is unit-tested. Starting values: grunt 0 and true; brute 0.75 and false.
- **Rationale**: The owner chose the spec's armored attack over resistance alone, so a brute's
  strike has to be dodged, not out-swung. Expressing it as a stat keeps it out of kind branches
  (owner's rule: no kind-specific behaviour code).
- **Alternatives**: resistance alone (rejected by the owner: a stunned brute loses its attack, so
  the hero's 0.08 s swing could lock it); a resistance of 1 meaning armored (rejected: the brute
  also needs a little push and stun outside attacks).

## R4. Enemy modes and turns

- **Decision**: `EnemyMode { Entering, Waiting, Approaching, WindUp(Attack), Active(Attack),
  Recovery(Attack), HitStun(Countdown), Dying(Countdown) }`. `Waiting` is new; an enemy goes
  `Entering → Waiting`, and only a granted turn moves it `Waiting → Approaching`. The rest of M3's
  attack flow is unchanged, except that the end of `Recovery` and the end of `HitStun` now lead to
  `Waiting`, not `Approaching`. An enemy holds a turn exactly while its mode is `Approaching`,
  `WindUp`, `Active` or `Recovery` (`EnemyMode::holds_turn()`); there is no separate flag, so a
  turn cannot be held by an enemy that has left those modes.
- **Rationale**: Deriving the turn from the mode makes the owner's release points (end of
  recovery, hit-stun, dying) automatic and testable on the pure transitions.
- **Alternatives**: a `has_token` bool beside the mode (rejected: the constitution's "modes as
  enums" rule, and it can disagree with the mode).

## R5. Attack tokens

- **Decision**: A pure `AttackTokens { capacity, holders: Vec<(InstanceId, cost)> }` in
  `rust/src/crowd.rs`, owned by `WaveDirector`. Each director tick:
  1. `reconcile(still_holding)`: drop holders whose enemy no longer holds a turn (left the modes in
     R4) or no longer exists.
  2. `grant(waiting)`: `waiting` is every `Waiting` enemy with its cost and its distance to its
     target, sorted nearest first. Tokens go to the nearest enemy if its whole cost fits; if not,
     granting stops for this tick, so a far grunt cannot keep jumping ahead of a near brute.
     Granted enemies are told with `Enemy::grant_turn()` (mode `Waiting → Approaching`).
  Invariant (tested): the sum of held costs never exceeds `capacity`. A smaller capacity after F5
  takes nothing away; it applies as tokens free up. Starting pool 2, grunt cost 1, brute cost 2.
- **Rationale**: The owner's input (nearest waiting enemy, chosen over the spec's first come, first
  served). Stopping at the first enemy that doesn't fit keeps "nearest" true for brutes too.
  The director grants and enemies release, so no enemy ever calls into the director: no re-entrant
  binds (M3 R8).
- **Alternatives**: first come, first served (the spec's earlier answer; changed by the owner);
  skipping to the next enemy that fits (rejected: a brute could starve behind a stream of grunts).

## R6. The waiting spot and drift

- **Decision**: At `place()`, an enemy fixes its side (its entry edge: −1 left, +1 right) and its
  waiting depth (its spawn depth). While `Waiting`, its spot is `x = target.x + side ·
  waiting_distance`, `y = waiting depth`, clamped into the lane and screen; it moves towards the
  spot at `crowd.drift_speed` on both axes, never overshooting, and keeps facing the target. With no
  living hero it stands still. The pure `drift_step(position, spot, speed, delta)` is tested.
- **Rationale**: The owner chose a fixed slot on the enemy's own side (planning answer), revising
  clarification Q3's circling. It reads clearly: enemies from the left wait on the left.
- **Alternatives**: circling the hero (rejected by the owner: needs a wander rule, more separation
  work and more tests).

## R7. Keeping apart

- **Decision**: `WaveDirector` runs a separation pass after every enemy has moved in the physics
  frame. The pure `separate(bodies, radius, depth_scale) -> Vec<(dx, dy)>` takes each enemy's
  position and whether it may be pushed. Distance is measured with the depth difference multiplied
  by `depth_scale = hero.horizontal_speed / hero.vertical_speed` (the owner's "scaled the same way
  as the vertical speed"), so a gap in depth counts as much as the time it takes to walk it. Each
  pair closer than `crowd.separation_radius` is pushed apart along the line between them, half each,
  or all of it onto the movable one when the other is attacking (`WindUp`, `Active`, `Recovery`:
  attackers stay put so their aim holds). Bodies at the same spot are split horizontally, the lower
  instance id to the left. After the push each enemy clamps to the lane and screen. Only `Waiting`,
  `Approaching` and `HitStun` enemies move; `Entering` and `Dying` ones are left out. A radius of
  0 turns the pass off. Plain O(n²): at most 10 enemies (R8), 45 pairs.
- **Rationale**: Spec FR-012, the owner's input. One pass a frame settles a crowd within a few
  frames without a physics body; the director already sees every enemy.
- **Alternatives**: Godot physics bodies with collision (rejected: the game has no physics bodies,
  Y-sort and hit rules are hand-made, and collisions would also block the hero); per-enemy steering
  (rejected: each enemy would need every other's position, the same O(n²) spread over n nodes).
- **Physics order**: the director sits after `World` in `main.tscn` and sets
  `process_physics_priority = 1`, so it always runs after the enemies.

## R8. Waves

- **Decision**: `WaveDirector` (`rust/src/wave_director.rs`) replaces `GruntSpawner`. The pure part:
  - `composition(number, waves) -> Composition { grunts, brutes }`: wave `n` (from 1) is the list's
    entry `n` while it exists; past the last entry `L`, `grunts = last.grunts + (n − L) ·
    extra_grunts_per_wave` and `brutes = last.brutes + (n − L) / brute_every_waves` (integer
    division; 0 means no more brutes).
  - `spawn_queue(composition) -> Vec<EnemyKind>`: grunts with the brutes spread evenly through them
    (brute `i` of `b` at position `round((i + 1) · total / (b + 1)) − 1`), so a wave's brutes don't
    all arrive at once or last.
  - `WaveState { Banner(Countdown), Fighting, Breather(Countdown) }` and `WaveProgress { number,
    state, queue, spawn_timer }` with `tick(progress, living, max_on_screen, timing, delta) ->
    (WaveProgress, Option<EnemyKind>)`:

    | From | Condition | To | Spawn |
    | --- | --- | --- | --- |
    | `Banner` | countdown ends | `Fighting`, the wave's queue added after any F3/F7 kinds, timer 0 | — |
    | `Fighting` | queue not empty, timer ran out, `living < max_on_screen` | same, timer = `spawn_interval` | front of queue |
    | `Fighting` | queue empty and `living == 0` | `Breather(breather_time)`; `wave_cleared(n)` | — |
    | `Breather` | `living > 0` (F3/F7) | `Fighting` (same wave, empty queue) | — |
    | `Breather` | countdown ends | `Banner(banner_time)`, `n + 1`; `wave_started(n + 1)` | — |

    The run starts in `Banner` for wave 1 (`wave_started(1)` at once). `living` counts the
    director's enemies that are not `Dying`; entering ones count.
  - Nothing happens without valid tuning or once the run is over.
- **Spawning**: one enemy per spawn, from a random edge just off screen at a random lane depth,
  `grunt_scene` or `brute_scene` by kind, added to `World` before the hero (M3 R6).
- **Signals**: `wave_started(number)` and `wave_cleared(number)`. The banner polls instead (R11).
- **Rationale**: The owner's input, with the spec's on-screen maximum (clarification) and the
  owner's planning answer that enemies enter after the banner. An integer rule keeps every wave
  whole and never smaller than the one before.
- **Alternatives**: a curve or percentage growth (rejected: harder to read in the data; the spec
  asks for a rule that only adds).

## R9. Score, wave and the session best

- **Decision**: `RunRecord` gains `score`, `wave` and `best` (all `i64`). `add_kill(points)` adds one
  kill and the points while playing and emits `score_changed(score)`; `set_wave(n)` is called by
  the director when a banner starts; `end_run()` updates `best = max(best, score)` and emits
  `game_over(kills, score, wave, best)`; `restart()` also folds the score into `best` before
  `reset()`, then resets `score`, `kills` and `wave` but never `best`. The autoload survives the
  scene reload, so the best lasts until the game closes; nothing is written to disk.
- **Rationale**: The owner's input (best kept in the autoload, updated at game over). Folding it in
  on restart too keeps the spec's "highest score of any run" (FR-014) true for a run ended with R.
  The signal carries every value Game Over shows, so its handler calls nothing back (M3 R8).
- **Alternatives**: updating the best on every kill (rejected: Game Over is where it is shown, and
  the owner named game over).

## R10. Tuning file

- **Decision**: New and moved sections ([contracts/tuning-data.md](contracts/tuning-data.md)):
  `enemies.grunt` (M3's `grunt`, plus `knockback_resistance`, `interruptible`, `attack_token_cost`,
  `points`), `enemies.brute` (the same keys), `waves` (`list` of `{ grunts, brutes }`,
  `extra_grunts_per_wave`, `brute_every_waves`, `spawn_interval`, `breather_time`, `banner_time`,
  `max_on_screen`), `crowd` (`max_attack_tokens`, `waiting_distance`, `drift_speed`,
  `separation_radius`). `run.respawn_delay` goes; `run.death_fade_time` stays.
- **Copy stays**: `TuningValues` keeps `Copy` (eight call sites read it every tick). The wave list
  lives in a separate validated `WaveTuning` (`Clone`), returned by `TuningStore::waves()` and read
  only by the director; the loader returns both from one all-or-nothing `validate`.
- **Readers**: `interruptible` is the first boolean (`bool_in`), the wave list the first array
  (`list_in`); counts, costs and points must be whole numbers (`whole_in`).
- **Warnings** (accepted, shown as in M3): each enemy's wind-up not longer than the hero's, and the
  brute's not longer than the grunt's.
- **Rationale**: The owner's sections. Splitting the list out avoids `Clone` calls at every
  `values()` read and keeps M3's call sites unchanged.
- **Alternatives**: `TuningValues: Clone` everywhere (rejected: churn in every module for one
  director's list); a fixed-size wave array (rejected: an arbitrary cap in the file format).

## R11. Presentation

- **Decision**: All GDScript, holding no state:
  - `ui/wave_banner.gd` (a `Label` on the HUD): shows "Wave N" while `director.banner_wave()` is
    above 0, hidden otherwise; polled in `_process`, as M3's bars are.
  - `ui/score_label.gd`: reads `Run.score()` at `_ready` and updates on `Run.score_changed(score)`.
  - `ui/game_over.gd` (extended): `game_over(kills, score, wave, best)` fills score, wave, kills and
    best lines.
  - `world/enemy_body.gd` (renamed from `grunt_body.gd`) and `world/enemy_health_bar.gd` (renamed
    from `grunt_health_bar.gd`) work for both kinds; the brute's look comes from its scene.
- **Rationale**: Polling the banner avoids a duration copy in GDScript (the director already counts
  the banner time); the score label uses the owner's `score_changed` signal.
- **Alternatives**: a banner timer in GDScript on `wave_started` (rejected: a second copy of
  `banner_time` and state in GDScript).

## R12. Debug cheats

- **Decision**: `debug_spawn_brute` (F7) and `debug_next_wave` (F8), new actions. F3 and F7 call
  `WaveDirector.spawn_grunt()` / `spawn_brute()` (a Rust-side `spawn_now(kind)` behind both, so
  GDScript never handles the kind): one enemy joins the current wave, entering at once if the
  on-screen maximum allows, else first in the queue. F8 calls `WaveDirector.skip_wave()`: every
  enemy of the director is freed without points or kills, the queue is emptied, and the next wave's
  banner starts at once (owner's planning answer). All three do nothing at Game Over. The overlay's
  `spawner` export becomes `director`.
- **Rationale**: Spec FR-019; M2a's rule that cheats are actions and absent from release builds.
- **Alternatives**: F8 going to a breather first (the owner's first note; changed by the owner).

## R13. Tests

`cargo test`, pure functions only, beside the M3 tests (owner's list plus the spec's rules):

- Composition from the list and from the escalation rule, including integer brute growth and
  `brute_every_waves = 0`; the spawn queue spreads brutes.
- The wave tick: no next wave while an enemy lives or the queue holds enemies; breather then banner;
  F3/F7 during a breather returns to `Fighting`; the on-screen maximum holds the queue; nothing
  without tuning or after Game Over.
- Tokens never exceed capacity; release on recovery, hit-stun and death (via `holds_turn`); the
  nearest waiting enemy gets the free token; a brute that doesn't fit blocks farther grunts.
- Separation pushes overlapping enemies apart, moves only movable ones, splits identical spots,
  uses the depth scale, and never leaves the lane.
- Drift reaches the spot without overshooting; speed 0 holds still.
- `knockback_resistance` scales distance and stun; a non-interruptible enemy keeps its attack;
  an interruptible one loses it.
- Points per kind; the best survives restart and is folded in at game over and on restart;
  `skip_wave` awards no points (pure: removal does not call `add_kill`).
- Tuning: the new rules and messages ([contracts/tuning-data.md](contracts/tuning-data.md)) and the
  two new warnings.

Everything else (spawning in the scene, the banner, the HUD, armored attacks feeling heavy) is
checked by playing ([quickstart.md](quickstart.md)).
