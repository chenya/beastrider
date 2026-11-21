# Research: M2b Hit and Feel

Decisions for M2b, each with its reason and the options rejected. Most choices come from the
owner's technical context; this file records why they hold up, the details they leave open, and
where the plan departs from the input. Behaviour marked *checked* was confirmed against the
installed Godot 4.7.2 and gdext 0.5.5 on 2026-11-20.

## R1. Versions and earlier decisions: carried over

- **Decision**: Engine Godot **4.7.2-stable**, gdext `=0.5.5` with `api-4-6`, Rust 1.100.0, the
  `godot/` + `rust/` layout, JSON tuning in `godot/data/tuning.json` loaded by `TuningStore` (the
  `Tuning` autoload) with M2a's all-or-nothing F5 reload, Compatibility renderer, 1280×720, V-Sync
  on. No new crate or addon.
- **Rationale**: The owner asked to carry everything over. "Godot 4.6.x" in the input means the
  pinned 4.7.2 with API level 4.6, as confirmed in M1 and M2a.

## R2. Hero modes as enums

- **Problem** (Principle III): the hero now has two modes with different rules (free to move, or
  swinging in one of three parts), and a remembered press during recovery.
- **Decision**: In a pure module `rust/src/combat.rs`:
  - `enum Facing { Left, Right }`, updated from the last non-zero horizontal input while free;
    `Right` at start (spec FR-002).
  - `enum SwingPhase { WindUp, Active, Recovery { queued: bool } }`. The `queued` flag is the one
    remembered press of clarification Q3; it exists only inside `Recovery`, so it cannot be set in
    any other phase.
  - In `rust/src/hero.rs`: `enum HeroMode { Free, Attacking { phase: SwingPhase, elapsed: f32,
    hit_this_swing: Vec<InstanceId> } }`.
  - A pure `advance_swing(phase, elapsed, delta, timings) -> SwingAdvance` moves through the
    phases on time, passing through zero-length parts in one tick (wind-up and recovery may be 0,
    FR-010), and reports `Continue`, `Finished` or `Restart` (a queued press). It also reports
    whether the swing was active at any moment of the tick, so an active part shorter than one
    tick still gets checked for hits (added during implementation).
  Every use is an exhaustive `match`; there is no `is_attacking` flag (FR-012). Movement input is
  read only in `Free`. Everything runs in `physics_process`.
- **Attack press**: read with `Input.is_action_just_pressed(p1_attack)` in `physics_process`, which
  reports a press once, so holding the key cannot repeat a swing (FR-003). A press in `Free` starts
  a swing; in `Recovery` it sets `queued`; in `WindUp` or `Active` it is dropped. While frozen (R5)
  the hero skips its step, so a press during a freeze is dropped too.
- **`hit_this_swing`**: a small `Vec` rather than a set: with a handful of dummies a linear search
  is simplest, and it is cleared at the start of each swing.
- **Alternatives considered**: booleans (`is_attacking`, `buffered`), rejected by the constitution's
  "types over strings" rule and the owner; a general state-machine crate (a dependency for two
  modes).

## R3. Hit detection in plain Rust

- **Decision**: Axis-aligned boxes in screen coordinates, built from each fighter's feet origin:
  - **Attack box**: starts at the hero's front edge (its `Body` half width, measured as in M1) and
    reaches `attack.reach` further in the facing direction; it rises `attack.height` above the
    feet. `Facing` mirrors it.
  - **Hurtbox**: `dummy.hurtbox_width` wide, centred on the dummy's origin, `dummy.hurtbox_height`
    tall above the feet.
  - A pure `lands(attack, hurtbox, attacker_feet_y, target_feet_y, depth_tolerance) -> bool` is true
    when the boxes overlap **horizontally**, touching edges included, **and**
    `|attacker_feet_y - target_feet_y| <= depth_tolerance` (FR-005, spec edge cases). The heights
    only matter for drawing (F2): depth, not box height, decides a hit on a lane.
  - Targets are the nodes in one Godot group, `HITTABLE_GROUP = "hittable"`, a single Rust constant.
    Each `TrainingDummy` joins it in `ready`, so spawned dummies are found with no extra wiring.
- **Rationale**: the owner's: a handful of fighters, deterministic, testable without the engine,
  and the depth rule is custom anyway. `Area2D` overlaps would arrive a physics frame late and still
  need the depth check.
- **Alternatives considered**: `Area2D`/`CollisionShape2D` (engine timing, harder to test, still
  custom depth); testing vertical overlap of the boxes (wrong for a lane: two fighters at the same
  depth always "overlap" vertically).

## R4. The training dummy and its knockback

- **Decision**: Rust class `TrainingDummy` (extends `Node2D`, origin at its feet), placed in the
  Y-sorted `World` like the pillars, with a `Body` `Polygon2D` drawn above the origin. It joins
  `HITTABLE_GROUP`, has no health, and moves only while sliding.
- **Knockback** (clarification Q1, revised by the owner): a hit gives a slide that covers
  `impact.knockback_distance` in `impact.knockback_time`, decelerating evenly to rest. With `d` and
  `t`, the start speed is `2d/t` and the displacement after `e` seconds is `d · (1 - (1 - e/t)²)`.
  The pure `knockback_offset(distance, time, elapsed)` uses this closed form, so the slide ends
  exactly `d` away with no rounding drift, whatever the frame times. Direction is the hero's facing
  (away from the hero). A new hit replaces a running slide from where the dummy is (spec edge case).
- **Clamping**: every tick the dummy's x is clamped to `[hurtbox_width/2, screen_width -
  hurtbox_width/2]` and its y to the lane from `Tuning` (FR-008). A slide that meets a limit stops
  there.
- **Alternatives considered**: a velocity with per-tick friction (drifts with frame time and needs a
  second number that is not the owner's "distance in time"); a `Tween` on the position (moves a rule
  into presentation).

## R5. Hit-stop: a Rust autoload, not `Engine.time_scale`

- **Problem** (Principle III): every gameplay object must stop together for the same freeze, and two
  freezes close together must not add up.
- **Decision**: Rust class `HitStopTimer` (extends `Node`), the root of
  `godot/autoload/hit_stop.tscn`, registered as the autoload **`HitStop`** after `Tuning`. It
  holds `remaining: f32`, counts it down in its own `physics_process` in real time, and offers
  `freeze(seconds)` and `is_frozen()`. A pure `extend_freeze(remaining, new) -> f32` returns the
  **larger** of the two, never the sum. The hero and every dummy skip their
  gameplay step while `is_frozen()` (FR-007). A freeze of 0 does nothing (FR-010).
- **Naming**: the input names the autoload `HitStop`. As M2a found (M2a research R2), a class and an
  autoload may not share a name without breaking typed GDScript, so the class is `HitStopTimer`.
- **Order**: autoloads are earlier children of `/root`, so `HitStop` counts down before the scene's
  nodes run each tick; a freeze started by the hero on one tick holds the hero and dummies from the
  next tick.
- **Why not `time_scale`**: it would stop the shake, the flash tween and the overlay too; the flash
  and shake must run through the freeze (clarification Q2).
- **Alternatives considered**: `get_tree().paused` (stops everything not marked to process while
  paused, including the dummies' presentation); a freeze counter on each object (two copies that can
  disagree).

## R6. `was_hit` and the presentation

- **Decision**: `TrainingDummy` declares `#[signal] was_hit(direction: f32, strength: f32)`:
  `direction` is `-1.0` or `1.0` (the push direction), `strength` the knockback distance in px (for
  later presentation; unused by M2b's effects). Emitted once per hit, at the moment of impact.
- **Flash**: `godot/world/flash.gdshader`, a `canvas_item` shader with `uniform float flash_amount`
  mixing `COLOR.rgb` towards white, on the dummy's `Body` through a `ShaderMaterial`. A small script
  on the dummy's `Body` (`godot/world/dummy_flash.gd`, extends `Polygon2D`) answers its parent's
  `was_hit` by tweening `flash_amount` from 1 to 0 over `Tuning.flash_time()`; at 0 it does
  nothing. *Found during implementation*: the script must not sit on the `TrainingDummy` root. A
  GDScript `_ready` on a node whose class is a Rust class replaces the Rust `ready()`, so the dummy
  never joined the `hittable` group or found its autoloads and could not be hit. *Checked*: on the
  Compatibility renderer (`gl_compatibility`, OpenGL 3) the shader compiles and a pixel goes from
  the body colour to half-way to pure white at 0.5 and to `(1, 1, 1)` at 1.0, and a tween drives
  the uniform.
  Reason for a shader (owner's): it keeps working when the placeholder becomes art at M5. The
  material is set **local to scene** so each dummy flashes on its own.
- **Shake**: a `Camera2D` named `ShakeCamera`, first child of `Main`, `anchor_mode = FIXED_TOP_LEFT`
  at (0, 0), so the view is unchanged. Its script (`godot/world/screen_shake.gd`) sets `offset` to a
  random jitter of up to `Tuning.shake_strength()` that decays linearly to 0 over
  `Tuning.shake_time()`, in `_process`, so it runs in real time through the freeze. `CanvasLayer`s
  (HUD, overlay) do not follow the camera, so only the play area shakes (spec assumption). Several
  dummies hit on one tick restart one shake; they do not add up.
- **Finding dummies**: the camera script connects to `get_tree().node_added` in `_enter_tree` and
  listens to every `TrainingDummy` that enters the tree, so dummies in the scene and dummies spawned
  by F3 both shake the screen, with no group name repeated in GDScript.
- **Tuning reads**: `TuningStore` gains `#[func]` accessors for `flash_time`, `shake_strength` and
  `shake_time`, the only new values presentation needs. GDScript keeps no copy.

## R7. Hit counter and debug cheats

- **Decision**: The hero owns `hits_landed` (one per dummy hit), exposed as a read-only `#[func]`.
  The overlay panel shows `Hits: N` (FR-016).
- **F2** (`debug_show_hitboxes`, the owner's name): `godot/debug/debug_hitboxes.gd` on a `Node2D`
  after `DebugMarkers`, `z_index` 1. While visible it redraws every frame: the hero's attack box
  from `hero.attack_box()` (a `Rect2`, empty unless the swing is `Active`) and each dummy's
  `hurtbox()` (a `Rect2`). It finds dummies as the `TrainingDummy` children of `World`.
- **F3** (`debug_spawn_dummy`): the overlay instances `res://world/training_dummy.tscn` into `World`
  at `hero.dummy_spawn_point()` and moves it just before the hero, so the hero stays the last child
  and still wins depth ties (M1 research R2). The spawn point is a game rule (clarification Q4:
  start just inside reach, step half a dummy's width along the facing while a dummy stands there,
  keep inside lane and screen), so it is a pure Rust function `spawn_point(...)` behind the hero's
  `#[func]`; GDScript only instances and places.
- **Input**: F2 and F3 are Input Map actions handled in the overlay's `_unhandled_input`, like F1
  and F5. F2 works with the panel closed (spec edge case).
- **Release builds**: the overlay frees the markers, the hitbox drawer and itself, as in M2a; F2 and
  F3 then do nothing (FR-017). The dummy, the attack, the flash and the shake are game, not debug,
  and stay.
- **Departure from the input**: the input says F3 places the dummy "in front of the hero, clamped to
  the lane"; the spec (clarification Q4) adds the half-width stepping. The plan follows the spec,
  and moves the computation into Rust.

## R8. Tuning additions

- **Decision**: Three new sections in `godot/data/tuning.json`, read by the same loader and checked
  by the same `validate`:

  ```json
  "attack": { "wind_up_time": 0.08, "active_time": 0.1, "recovery_time": 0.18,
              "reach": 70, "height": 60, "depth_tolerance": 20 },
  "impact": { "hit_stop_time": 0.06, "flash_time": 0.08, "shake_strength": 6,
              "shake_time": 0.15, "knockback_distance": 60, "knockback_time": 0.2 },
  "dummy":  { "hurtbox_width": 56, "hurtbox_height": 120 }
  ```

- **Rules** (spec FR-010 as revised): every value finite and `>= 0`; `attack.active_time > 0`;
  `attack.reach > 0`; `impact.knockback_time > 0` whenever `impact.knockback_distance > 0`. Zero
  turns off `hit_stop_time`, `flash_time`, `shake_strength`, `shake_time` and `knockback_distance`.
- **`reach > 0`**: the input lists sizes as `>= 0`; the spec's FR-010 requires reach above 0 (a
  zero reach could never be seen to work). The plan follows the spec.
- **Tests** (owner's list): overlap with and without the depth tolerance; mirrored boxes for both
  facings; one hit per swing; two targets in one swing; knockback clamped at lane and screen edges;
  hit-stop takes the max, not the sum; phases advance on time (including zero-length parts and the
  queued restart); plus one passing and one failing case per new tuning rule, and the spawn-point
  stepping.

## R9. Scene changes

- **Decision**: New scene `godot/world/training_dummy.tscn`: `TrainingDummy` root, `Body`
  `Polygon2D` (−28,−120)…(28,0) in a colour unlike the hero's, with the flash `ShaderMaterial` and
  `dummy_flash.gd`. `main.tscn` gains `ShakeCamera` (first child of `Main`), one `Dummy` instance in
  `World` before `Hero`, and `DebugHitboxes` after `DebugMarkers`; the overlay gains exports for
  the hitbox drawer, `World` and the dummy scene, and a `HitsLine` label. Input Map gains
  `p1_attack` (J, Z), `debug_show_hitboxes` (F2) and `debug_spawn_dummy` (F3).
- **Building**: as in M1 and M2a, the agent may build these with headless scripts if the owner hands
  the editor tasks over; recorded in Complexity Tracking.
