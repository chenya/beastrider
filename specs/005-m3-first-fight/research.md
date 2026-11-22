# Research: M3 First Fight

Phase 0 of the [plan](plan.md). Each section records a decision, why, and what was rejected. The
owner's technical context is the starting point; where it disagreed with the spec, the owner chose
during planning (see [plan.md, Spec and input alignment](plan.md#spec-and-input-alignment)).

## R1. Versions, platform and hardware

- **Decision**: Everything pinned in M0–M2b stays: Godot 4.7.2, gdext `=0.5.5` with `api-4-6`,
  Rust 1.100.0, Compatibility renderer, 1280×720, V-Sync on. No new crate or addon. Restart time is
  measured on the **UHD 630**.
- **Rationale**: The input says "UHD 620"; constitution v1.3.2 corrected the target to the UHD 630,
  and the spec (check 13) names the 630. Randomness comes from Godot's `RandomNumberGenerator`
  (owner's input), so no `rand` crate.
- **Alternatives**: the `rand` crate (rejected: a second dependency for two random choices per
  spawn).

## R2. Shared fighter parts as plain Rust structs

- **Decision**: A new pure module `rust/src/fighter.rs`, tested without the engine, holds what the
  hero and the grunt both need, composed as fields inside each Godot class (no child nodes):
  - `Health { current, max }`: `damage(amount) -> Damage { Survived, Killed, AlreadyDead }`,
    `refill()`, `set_max(max)` (current lowered to a new, smaller max; never raised).
  - `AttackSpec`: wind-up, active and recovery times, reach, box height, damage and a
    `KnockbackSpec { distance, time }`, read from one fighter's `attack` tuning section.
  - `AttackTimer { phase: AttackPhase, elapsed, spec }` with `AttackPhase { WindUp, Active,
    Recovery }` and `advance(delta) -> AttackAdvance { step: Continue | Finished, was_active }`.
    It replaces M2b's `SwingPhase`/`advance_swing`, keeping both of their rules (zero-length parts
    pass in one tick; `was_active` is true if the attack was active at any moment of the tick).
  - `Knockback`: M2b's slide (start x, direction, distance, time, elapsed) moved out of
    `training_dummy.rs`, with `offset()` from `combat::knockback_offset`; clamping stays in
    `combat::clamp_to_area`.
  - `Countdown { remaining }`: one timer type for hit-stun, invulnerability, dying and the respawn
    delay (`start(seconds)`, `tick(delta) -> bool` "just ran out", `is_running()`).
  - `resolve_attack(attacker: &Attacker, targets: &[Target<T>], already_hit: &[T]) -> Vec<Hit<T>>`:
    the one hit rule for everyone. It keeps targets of the other team that are alive and not
    invulnerable, calls M2b's `combat::new_hits` (overlap plus depth tolerance), and returns hit
    records `Hit { target, damage, direction, knockback }`.
- **Rationale**: The owner's problem statement: hero and grunt need the same health, attack phases,
  stun, knockback and hit rule, and duplicating them would let the two drift (Principle III names
  the problem). Plain structs keep every rule under `cargo test`; fields instead of child nodes
  leave no scene wiring to get wrong.
- **The hero's queued press** stays in the hero (`Swing { timer, queued, hit_this_swing }`): it is
  input buffering, not an attack rule, and grunts have no input. M2b's `SwingStep::Restart` becomes
  "`Finished` with `queued` set" inside the hero.
- **Alternatives**: a `Fighter` trait implemented by both classes (rejected: gdext classes are
  reached through `Gd<T>`, and a trait would not remove the per-class glue); child nodes such as a
  `HealthComponent` (rejected by the owner: wiring to get wrong, and harder to test).

## R3. Teams, groups and reaching another fighter

- **Decision**: `enum Team { Players, Enemies }` with `Team::group()` returning one of two Rust
  constants, `"team_players"` and `"team_enemies"`. Each fighter joins its team's group in
  `ready()`; M2b's `hittable` group is removed. An attacker reads only the **other** team's group,
  so grunts never hit grunts and nobody hits itself.
- **Reaching a target of another class**: a Godot-facing helper `rust/src/fighters.rs` turns each
  group member into `enum FighterRef { Hero(Gd<Hero>), Grunt(Gd<Grunt>), Dummy(Gd<TrainingDummy>) }`
  with `target()` (hurtbox, feet y, alive, invulnerable) and `take_hit(&Hit)`. Every use is an
  exhaustive `match`, so a future fighter class (the brute, M4) is a compile error until handled.
- **The training dummy** joins `Enemies` with no `Health`: it takes knockback and flashes but never
  dies or counts as a kill (spec, Edge Cases). Grunts read only `Players`, so they ignore dummies.
- **Rationale**: The owner's input; types over strings for the group names; the enum keeps the
  dispatch in one place.
- **Alternatives**: `call("take_hit")` by name (rejected: strings, no compile-time check); one
  group with a team field (rejected: every attacker would scan its own team too).

## R4. Hero modes, stun and invulnerability

- **Decision**: `HeroMode { Free, Attacking(Swing), HitStun(Countdown), Dead }`, matched
  exhaustively. Beside the mode: `health: Health`, `invulnerable: Countdown` and
  `knockback: Option<Knockback>`.
  - A landed hit (only while alive and not invulnerable): health drops; `Killed` → `Dead`
    (the run ends, R7); otherwise → `HitStun` for `hero.hit_stun_time`, cancelling any swing, and
    invulnerability starts for `hero.invulnerability_time`, counted from the hit (FR-009).
  - Knockback runs in every mode, so a stunned or fallen hero still slides.
  - `is_invulnerable()` is exposed for the GDScript blink; the hero walks and swings while blinking,
    which is why invulnerability is a timer, not a mode (owner's input).
  - A killing blow starts no invulnerability (spec: "no invulnerability blink").
- **Hurtbox**: from `hero.hurtbox_width` and `hurtbox_height` (FR-015). The screen clamp keeps
  using the `Body` polygon's half-width, as in M1; the starting hurtbox is 48×96 px, the same as the
  polygon, so the two agree until the owner changes one.
- **Alternatives**: invulnerability as a mode (rejected: the hero acts while blinking); a `bool`
  for dead (rejected: types over strings, and `Dead` changes every branch).

## R5. The grunt

- **Decision**: Rust class `Grunt` (extends `Node2D`, origin at the feet, in the Y-sorted `World`)
  with `GruntMode { Entering, Approaching, WindUp, Active, Recovery, HitStun, Dying }`, matched
  exhaustively, plus `health`, `facing`, `knockback`, and the current attack's `AttackTimer` and
  `hit_this_attack`. The phase of the attack is the mode itself, so the three attack modes share
  one timer field read only in those arms.
- **Target**: the nearest living fighter in the `Players` group, picked every tick, so co-op needs
  no change (Principle VII). With no living target (the hero fell) the grunt stands still.
- **Approaching** (pure function `fighter::approach_step`, tested on both sides):
  - The slot is at the target's depth, `reach` beyond the target's body edge, on the side the
    grunt is already on (`side = sign(grunt.x − target.x)`, right when equal).
  - While the depth difference is above `lane.depth_tolerance`, it moves only vertically, at
    `grunt.vertical_speed`; once inside the tolerance, it moves horizontally to the slot at
    `grunt.horizontal_speed` and keeps correcting depth. This is the spec's "lines up, then closes
    in" (FR-003); if the target leaves the depth, horizontal movement stops until it lines up again.
  - It faces the target.
- **Starting an attack**: only when the target's hurtbox overlaps the grunt's attack box and the
  depth difference is within tolerance, i.e. `resolve_attack` would land now (FR-004). From then
  until the end of recovery it neither moves nor turns; the strike lands if the target is still
  there during `Active` (FR-005).
- **Hit**: any landed hit, in any mode but `Entering` and `Dying`, switches to `HitStun` for
  `grunt.hit_stun_time`, dropping the attack (clarification Q1); after it, back to `Approaching`.
- **Dying** (`Killed`): emits `died()` once, adds one kill to the run, leaves the `Enemies` group
  at once (cannot be hit, targeted or counted as alive), and frees itself after
  `run.death_fade_time`. GDScript fades the body over the same time from `died` (owner's input).
- **Entering**: a new grunt starts just off screen at its edge (x = −half width or screen width
  + half width) at a random lane depth, walks straight in at `grunt.horizontal_speed`, and joins the
  `Enemies` group only once fully on screen, so it can be neither hit nor attack before
  (spec, Edge Cases). Then `Approaching`; from then on the lane and screen clamp hold it.
- **Stopping**: outside `RunState::Playing`, every mode except `Dying` does nothing (the spec's
  "all grunts stop"; a fading grunt finishes fading).
- **Visible wind-up**: `#[func] is_winding_up()`; the grunt's body script switches the polygon to a
  wind-up colour while it is true (presentation, FR-021).
- **Alternatives**: diagonal approach (rejected: the spec requires lining up first); navigation or
  steering libraries (rejected: one lane, no obstacles).

## R6. Spawning

- **Decision**: Rust class `GruntSpawner` (extends `Node`), one in `main.tscn`, with exported
  `grunt_scene: PackedScene` and `world: Node2D`. Its state is
  `SpawnerState { AwaitingFirst, Watching, Waiting(Countdown) }`:
  - `AwaitingFirst`: as soon as tuning is valid, spawn one grunt with no delay (FR-001), then
    `Watching`. A bad start-up tuning file therefore holds the first grunt back (spec, Edge Cases).
  - `Watching`: when no living grunt is in the `Enemies` group (a dying grunt has left it), start
    the respawn delay → `Waiting`.
  - `Waiting`: counts down `run.respawn_delay` (paused by the hit-stop freeze); at zero, spawn one
    and go back to `Watching`. If a grunt appears meanwhile (F3), go back to `Watching` without
    spawning: the rule is "a new grunt only when none is left" (owner's answer during planning).
  - Nothing counts down or spawns outside `RunState::Playing`.
  - `#[func] spawn_now()` for F3 adds one grunt the normal way: random edge, random depth.
- **Randomness**: one `RandomNumberGenerator`, `randomize()`d in `ready()`; the edge is
  `randi_range(0, 1)`, the depth `randf_range(lane_top, lane_bottom)`. No fixed seed (spec).
- **Placement**: new grunts are added to `World` before the hero (as M2b's spawned dummies), so the
  hero still wins depth ties.
- **Alternatives**: the replace-every-kill rule from `/speckit-specify` (replaced by the owner's
  planning answer); spawning from the `Run` autoload (rejected: the autoload outlives the scene, and
  a spawner in the scene is rebuilt by the restart for free).

## R7. The run and the restart

- **Decision**: Rust class `RunTracker` registered as the autoload **`Run`** (class and autoload
  names must differ; M2b lesson). It holds `state: RunState { Playing, GameOver }` and `kills`.
  - `add_kill()` (from a dying grunt), `end_run()` (from the hero when it falls: sets `GameOver`
    and emits `game_over(kills)` once), `is_playing()`, `kills()`.
  - It reads the `game_restart` action (R) in `process` in every build. `restart()` sets `Playing`,
    `kills = 0`, clears the `HitStop` freeze (new `HitStopTimer::clear()`), and calls
    `SceneTree::reload_current_scene()`. The reload rebuilds the hero, the spawner (which spawns the
    first grunt), the HUD and the Game Over layer; every grunt and dummy of the old run is gone;
    the hit counter resets with the hero. `Tuning` is an autoload and is not reloaded, so F5 values
    survive (owner's input).
  - Restart time is measured from the key press to the first frame of the new run, on the UHD 630,
    against 2 s (SC-007).
- **Signals and re-entrancy**: `game_over(kills)` is emitted while the hero (and the grunt whose
  blow killed it) are bound in Rust, so its GDScript handler must use the argument and call
  nothing on `Run` or the hero during the emit. The same rule holds for every M3 signal; see R8.
- **Alternatives**: resetting every node by hand (rejected: many places to forget, and the spec's
  "nothing carries over"); a GDScript run manager (rejected: run state is game state, FR-021).

## R8. Presentation

- **Decision** (GDScript only reads Rust and reacts to its signals; it holds no game state):
  - **HUD** (`Hud` CanvasLayer, already holding the FPS label): `HealthBar` top-left at (16, 16)
    reading the hero's `health()` and `max_health()` every frame; `KillsLabel` under it reading
    `Run.kills()`. Both stay still during the shake (CanvasLayer) and exist in release builds.
  - **Debug panel** moves down to start below the health bar and kills label (FR-013).
  - **Grunt**: `grunt.tscn` = `Grunt` root (no script, M2b lesson R6) + `Body` polygon with the
    flash material and `grunt_body.gd` (flash on `was_hit`, wind-up colour while `is_winding_up()`,
    fade to transparent over `Tuning.death_fade_time()` on `died`) + `HealthBar` (a `Node2D` drawing
    two rectangles above the head from `health()`/`max_health()` each frame).
  - **Hero**: its `Body` gets the flash material and `hero_body.gd` (flash on `was_hit`, blink while
    `is_invulnerable()`, tip over on `died`).
  - **Shared flash**: M2b's `dummy_flash.gd` becomes `hit_flash.gd` (renamed); `hero_body.gd` and
    `grunt_body.gd` extend it; the dummy's `Body` uses it directly.
  - **Shake**: `screen_shake.gd` listens to `was_hit` of every `Hero`, `Grunt` and `TrainingDummy`
    entering the tree.
  - **Game Over**: a `GameOver` CanvasLayer in `main.tscn`, hidden at start, shown by
    `Run.game_over(kills)` with "Game Over", the kill count and "Press R to restart".
  - Blink rate, colours and the fall angle are looks, kept as GDScript constants (as M2b's colours).
- **Re-entrancy rule**: signal handlers use their arguments and read only *other* objects (the
  `Tuning` autoload) during an emit; bars and labels poll in `_process`, never inside a handler.
  gdext panics if GDScript calls back into an object that Rust holds mutably while it emits.
- **Alternatives**: `ProgressBar` controls for the grunt bars (rejected: a `Control` under a
  `Node2D` adds layout rules for two rectangles); health-changed signals (rejected: polling a few
  bars is cheap and cannot miss an update).

## R9. Debug cheats

- **Decision**: Input Map actions `debug_spawn_grunt` (F3), `debug_refill_health` (F4) and
  `debug_spawn_dummy` moved from F3 to **F6**; `game_restart` (R) is not a debug action. In
  `debug_overlay.gd`: F3 → `spawner.spawn_now()`, F4 → `hero.refill_health()` (the overlay's hero is
  player 1; does nothing while the hero is `Dead`, spec Edge Cases), F6 → M2b's dummy spawn,
  unchanged. All debug-only, freed in release like the rest of the overlay (FR-019).
- **F2** additionally outlines the hero's hurtbox, each grunt's hurtbox, and each grunt's attack box
  while it is `Active` (FR-020), from new `#[func]`s `hurtbox()` and `attack_box()` on both classes.
- **Hero's dummy spawn point** now looks for dummies in the `Enemies` group instead of `hittable`.

## R10. Tuning file restructure and the wind-up warning

- **Decision**: one file, one loader, the same all-or-nothing reload, restructured so both fighters
  use the same key names (owner's input). Sections: `hero`, `grunt` (each with a nested `attack`
  object), `dummy`, `lane`, `effects`, `run`. Field paths gain one level
  (`grunt.attack.wind_up_time`) and error messages name them as before. The full layout and rules
  are in [contracts/tuning-data.md](contracts/tuning-data.md).
  - `lane` keeps `top` and `bottom` and gains `depth_tolerance` (moved from M2b's `attack`): it is a
    property of the lane's fake depth and shared by every attacker (spec Assumptions).
  - `effects` holds the shared freeze, flash and shake (M2b's `impact` without knockback).
  - Knockback distance and time move into each fighter's `attack` (owner's answer during planning).
  - `dummy` keeps its hurtbox; `run` holds `respawn_delay` and `death_fade_time`.
  - One reader builds an `AttackSpec` from either fighter's `attack` object, so the two cannot
    drift.
- **Warning**: a pure `warnings(&TuningValues) -> Vec<TuningWarning>` runs after a successful
  `validate`. Its one rule for M3: `grunt.attack.wind_up_time` not greater than
  `hero.attack.wind_up_time`. The values are still accepted (owner's answer during planning);
  `TuningStore` keeps the warnings, prints them with `godot_warn!`, and exposes `warning()`; the
  overlay shows "Tuning reloaded (warning: …)" and keeps it on screen until the next reload, like a
  rejection. A start-up warning is read, not heard, like a start-up rejection (M2a).
- **Alternatives**: flat keys such as `grunt_wind_up_time` (rejected by the owner's input); a
  rejection (rejected by the owner); keeping `depth_tolerance` in a fighter's attack (rejected: the
  spec makes it one value for everyone).

## R11. Tests

- **`cargo test`** (owner's list, all on pure functions): killing hit reported exactly once and
  overkill clamped at zero (`Health`); invulnerable and dead targets ignored, teams never hit
  themselves, already-hit targets skipped (`resolve_attack`); attack phases and zero-length parts
  (`AttackTimer`, M2b's swing tests moved); a hit during wind-up cancels the attack (pure grunt
  transition `grunt_after_hit`); approach slot on both sides, depth first (`approach_step`); the
  spawner waits the delay and does not spawn while a grunt lives (`SpawnerState::tick`); restart
  resets the run (`RunState`/kills reset as a pure `RunRecord`); the tuning restructure, every new
  rule, and the wind-up warning.
- **Engine-level behaviour** (approach, strikes, death, restart time, cheats, release build,
  60 fps) is checked by windowed scripts and by playing, per [quickstart.md](quickstart.md).
