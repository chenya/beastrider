# Research: M4b Ashen Stand

Decisions behind [plan.md](plan.md). Each entry: the decision, why, and what else was weighed.

## R1. Versions, platform and hardware

- **Decision**: Nothing is upgraded: Godot 4.7.2, gdext `=0.5.5` with `api-4-6`, Rust 1.100.0,
  the Compatibility renderer, 1280×720, V-Sync on. The target is the Intel UHD 630.
- **Rationale**: The owner's input pins everything carried over. Check 12 names the UHD 620, but
  constitution 1.3.2 set the UHD 630 and M3 and M4a were checked on it (spec, Assumptions).
- **Alternatives**: None needed.

## R2. The magic meter

- **Decision**: A plain Rust struct `MagicMeter { charges: u32, max: u32 }` in a new pure module
  `rust/src/magic.rs`, held by each `Hero` (so it belongs to that hero's player index). Methods:
  `new(max)` (empty), `add_charge() -> bool` (true if it changed; false at the maximum),
  `refill()`, `spend_all() -> u32` (returns the charges and leaves 0), `set_max(max)` (a lower
  maximum cuts the charges), `charges()`, `max()`, `is_full()`, `is_empty()`. The hero creates it
  on its first tick with tuning, so each run starts empty.
- **Rationale**: The owner's input; a plain struct is unit-tested without the engine, like
  `Health`. Living in the hero ties it to the player index (Principle VII) without a lookup table.
- **Alternatives**: Charges in `Run` (rejected: `Run` is per run, not per player); a separate node
  (rejected: nothing to draw or process that the hero does not already).

## R3. Spell damage and its mode

- **Decision**: `SpellDamageMode { PerCharge, Flat }`, read from `magic.damage_mode` as the
  strings `"per_charge"` or `"flat"`, and a pure `spell_damage(mode, per_charge, flat, charges)
  -> f32`: `per_charge × charges` or `flat`. Every charge is spent in both modes. The starting
  mode is `per_charge`.
- **Rationale**: The owner kept the flat mode during planning (spec, Clarifications). Check 4
  ("damage grows with the charges") holds with the starting mode.
- **Alternatives**: A bool `flat` (rejected: Code Style, "types over strings", modes as enums).

## R4. Casting as a hero mode

- **Decision**: `HeroMode` gains `Casting(Cast)` with `Cast { timer: Countdown, charges: u32 }`. A
  pure `can_cast(mode) -> bool` is true for `Free`, `Attacking` and `HitStun`, false for `Casting`
  and `Dead`. On a press of `p1_magic`:
  - with charges and `can_cast`: `spend_all()`, the swing (and its `hit_this_swing`) or the stun
    is dropped, any knockback slide stops, `Run.begin_cast()`, `HitStop.clear()`, emit
    `cast_started(charges)` and `magic_changed`, enter `Casting`;
  - with no charges (and alive): emit `cast_failed_empty()`; nothing else changes.

  `Casting` makes the hero's target invulnerable, apart from the post-hit `invulnerable`
  countdown, which keeps running. When the cast's countdown (`magic.cast_time`) ends, the impact
  resolves (R6), then `Run.end_cast()`, emit `spell_impact()`, back to `Free`. The hero neither
  moves nor turns while casting. A press during `Casting` does nothing. A magic press made while
  play is frozen (the impact freeze after a hit, or another hero's cast) is kept and acted on at
  the hero's first step after it (found by T028): a press lasts one frame, and the panic press
  right after a hit falls inside the impact freeze, so without this the emergency cast would be
  lost.
- **Rationale**: The owner's input and spec clarification 1 (cast at once from a stun or a swing:
  the emergency escape). Invulnerability as part of the mode means it can't leak past the cast.
- **Alternatives**: Waiting for a stun to end (rejected by the owner); invulnerability through the
  existing countdown (rejected: two meanings in one timer).

## R5. The cast freeze

- **Decision**: `RunTracker` gains a cast-freeze count with `begin_cast()`, `end_cast()` and
  `#[func] is_cast_frozen()`. Every object that now skips its step while `HitStop` is frozen also
  skips it while the cast freeze is on: `Enemy`, `TrainingDummy`, `WaveDirector` (timers, spawns,
  tokens and separation) and the new `Potion` (its lifetime). The `Hero` skips its step while the
  cast freeze is on *unless it is the one casting*; its `Casting` countdown runs on. A count, not a
  flag, so two heroes casting at once (co-op later) end it only when both are done. `restart()`
  resets it to 0. The freeze ends on the tick of the impact; no hit-stop is added at the impact.
- **Rationale**: The owner's input: `HitStop` cannot carry it, because the caster must keep
  running while everything else stops. Putting it in `Run` keeps one place every object already
  links to.
- **Alternatives**: `Engine.time_scale` or pausing the tree (rejected: also stops the flash and the
  caster, as in M2b R-hit-stop); extending `HitStop` with an exception list (rejected: the owner
  asked to keep it apart).

## R6. The impact

- **Decision**: At the impact the hero collects every `Enemy` and `TrainingDummy` child of its
  parent (`World`, where both are added), not the `team_enemies` group, because an enemy joins it
  only once it has fully entered and one whose feet are already on screen must still be hit
  (planning answer). It keeps those whose feet origin lies inside the visible screen rectangle
  (`get_viewport_rect()`, a pure `on_screen(feet, screen) -> bool`), and:
  - an `Enemy` (not `Dying`) takes `take_spell(damage, from_x)`: its health drops; if it dies it
    enters `Dying` and counts as any kill (points, `died`, a drop roll, its turn freed); if it
    survives it enters `KnockedDown` (R7) and is pushed away from the caster by
    `magic.knockdown_push_distance` over `magic.knockdown_push_time` with the shared `Knockback`,
    ignoring `knockback_resistance` and armor. It emits `was_hit(direction, strength)` so the
    flash and shake play as for a blow;
  - a `TrainingDummy`, which has no health, only flashes and shakes the screen as for a blow, with
    no push or knockdown (spec, Edge Cases).

  The spell hits each fighter once; it is not a `resolve_attack` swing.
- **Rationale**: The owner's input and the planning answer (feet on screen). `FighterRef` already
  reaches enemies and dummies through the group (Principle VII, no fixed paths).
- **Alternatives**: Every enemy in play (rejected by the owner during planning); a huge attack box
  through `resolve_attack` (rejected: the depth rule would skip most of the screen).

## R7. Knocked down

- **Decision**: `EnemyMode` gains `KnockedDown(Countdown)` (`magic.knockdown_time`). While down:
  no attack, no turn (`holds_turn` is false, so the director frees its tokens on the next tick,
  as for `HitStun`), `Target::invulnerable = true` so blows cannot land (no juggling), its
  knockback slide runs, and the separation pass treats it as `Give::Yields`. The spell can still
  hit it: a second cast restarts the countdown. When the countdown ends it enters `Retreating`,
  so it goes back to its waiting spot before it can be given a turn (M4a's rotation). A pure
  `enemy_after_spell(mode, damage, knockdown_time, fade) -> EnemyMode` holds the rule, beside
  `enemy_after_hit`.
- **Rationale**: The owner's input (immune to further hits, tokens released) and the clarified
  spec (brutes go down too). The owner's "returns to Waiting" is met through `Retreating`, the
  M4a path back to the waiting spot, so a downed enemy can't snatch a turn the instant it rises.
- **Alternatives**: Straight to `Waiting` (rejected: it would bypass M4a's rule that turns rotate);
  blows allowed on downed enemies (rejected by the owner: juggling).

## R8. Potions

- **Decision**: A Rust class `Potion` (extends `Node2D`, origin at its feet) in a new module
  `rust/src/potion.rs`, with `PotionKind { Health, Magic }` (exported via `GString`, like
  `EnemyKind`). It joins the group `potions` (`POTIONS_GROUP`, a Rust constant) in `ready()`. It
  sits in the Y-sorted `World`, so depth ordering works. `place(kind, position)` sets it up before
  it enters the tree; on its first tick with tuning its lifetime starts at `potions.lifetime`. Each
  tick (unless frozen by `HitStop` or the cast) the lifetime counts down; at 0 it frees itself.
  `#[func]`s: `is_blinking()` (true in the last `potions.blink_time`) and `pickup_box()`; scripts read
  the exported `kind`.
  One scene, `godot/world/potion.tscn`, with a `Body` script that picks its colour from the exported `kind`.
- **Rationale**: The owner's input. A Rust node owns its countdown, so GDScript holds no state;
  `is_blinking()` rather than a remaining fraction keeps `blink_time` out of GDScript.
- **Alternatives**: Two scenes, one per kind (rejected: they differ only in colour, which the
  `Body` script reads from the kind); potions as data in the director (rejected: each needs a node
  to draw and sort by depth anyway).

## R9. Pickup

- **Decision**: Each physics tick (not while `Dead`, and not while frozen unless casting), the
  hero looks at every node in `potions` and collects one when a pure `can_collect(hero_box,
  hero_feet_y, potion_box, potion_feet_y, depth_tolerance)` holds (the boxes overlap horizontally
  and the feet differ by at most `lane.depth_tolerance`, as the hit rule does) **and** a pure
  `potion_useful(kind, health_full, meter_full)` holds (spec clarification: a potion that would do
  nothing stays). Collecting: heal (`Health::heal`, capped) or `add_charge()`, emit
  `potion_collected(kind)` (and `magic_changed` for magic), and free the potion. Only `Hero` runs
  this, so only `team_players` collects.
- **Rationale**: The owner's input (pickup box plus depth tolerance, as in hit detection) and the
  spec's FR-003 and FR-004.
- **Alternatives**: `Area2D` overlaps (rejected: physics bodies were rejected in M2b; the lane rule
  is the shared depth test).

## R10. Drops

- **Decision**: `RunTracker` owns one `RandomNumberGenerator`, reseeded (`randomize()`) in
  `restart()`, and a Rust-side `roll_drop(chance, health_odds) -> Option<PotionKind>` that draws
  two numbers and calls a pure `drop_from_rolls(chance, health_odds, roll_a, roll_b)`. An enemy,
  on entering `Dying` (from a blow or the spell), asks `Run` once with its kind's
  `drop_chance` and `potions.health_odds` and keeps the result as a pending drop. The
  `WaveDirector`, after the enemies tick, takes each pending drop (`take_drop()`) and adds a
  potion from its exported `potion_scene` at the enemy's feet, clamped to the lane, before the
  hero in `World` (as it does for enemies). F8's `skip_wave` frees enemies without a death, so
  they roll nothing.
- **Rationale**: The owner's input puts the roll in `Run` with one generator. The director already
  instantiates scenes into `World`, so it adds the potion; the enemy needs no scene reference.
- **Alternatives**: The enemy instantiating the potion (rejected: every enemy scene would need a
  `potion_scene` export); the director rolling (rejected: the owner named `Run`).

## R11. Presentation

- **Decision**: GDScript, holding no game state:
  - `godot/ui/magic_meter.gd` (`Control` under `Hud`, beside the health bar): draws one pip per
    `hero.max_charges()`, filled up to `hero.charges()`, read each frame; on `cast_failed_empty`
    it shakes for a named presentation time.
  - `godot/ui/spell_flash.gd` (`CanvasLayer` above the HUD with a full-screen `ColorRect`): its
    alpha follows `hero.cast_progress()` (0 to 1) while casting, peaks at `spell_impact`, then
    fades over a named presentation time.
  - `godot/world/potion_body.gd` (on the potion's `Body`): colour by `potion.kind`, hidden every
    other blink period while `potion.is_blinking()`.
  - `godot/world/enemy_body.gd` (extended): a flattened look while `enemy.is_knocked_down()`.
- **Rationale**: The owner's input; Rust holds every rule and time, scripts read and react.
- **Alternatives**: The flash timing itself from `cast_started` (rejected: it would copy
  `cast_time` into GDScript).

## R12. Input and the F9 cheat

- **Decision**: New actions `p1_magic` (physical K and X, any device, deadzone 0.2) and
  `debug_refill_magic` (physical F9). The hero builds its magic action from its player index like
  the others. The debug overlay calls `hero.refill_magic()` on F9; the hero ignores it at Game
  Over. The overlay exists only in debug builds, so F9 does nothing in release.
- **Rationale**: The owner's input and Principle VIII (a cheat arrives with what it acts on).
- **Alternatives**: None.

## R13. Tuning file

- **Decision**: `enemies.<kind>.drop_chance`; new section `potions { health_odds, lifetime,
  blink_time, heal_amount, width, height }` and `magic { max_charges, damage_mode,
  damage_per_charge, flat_damage, cast_time, knockdown_time, knockdown_push_distance,
  knockdown_push_time }`. `TuningValues` gains `PotionTuning` and `MagicTuning` and stays `Copy`.
  Rules and timings in [contracts/tuning-data.md](contracts/tuning-data.md).
- **Rationale**: The owner's input, with `knockdown_push_time` added because the shared
  `Knockback` needs a time as well as a distance, and the owner's "pickup box size" as the
  potion's width and height. The depth half of the pickup uses `lane.depth_tolerance`, the same
  value hits use.
- **Alternatives**: A separate pickup depth value (rejected: one depth rule for hits and pickups).

## R14. Restart and performance

- **Decision**: `Run.restart()` also resets the cast freeze and reseeds the generator; the scene
  reload removes every potion and makes a new hero with an empty meter. Performance: one
  full-screen `ColorRect` and a handful of potion polygons add nothing measurable; the impact
  touches each enemy once.
- **Rationale**: FR-014; Principle V.
- **Alternatives**: None.

## R15. Tests

`cargo test` covers, per the owner's list: `spend_all` returns and clears every charge;
`add_charge` and `set_max` clamp; `spell_damage` in both modes; `can_cast` true from `Free`,
`Attacking` and `HitStun`, false from `Dead` and `Casting`; an empty press changes no state;
`enemy_after_spell` ignores resistance and armor and gives `KnockedDown` or `Dying`;
`KnockedDown` holds no turn and its target is invulnerable; getting up gives `Retreating`;
`drop_from_rolls` gives the expected kinds at the edges of the chance and odds; `can_collect`
respects the depth tolerance; `potion_useful` refuses a potion that would do nothing; `Health`
heals capped at the maximum; a potion lifetime does not count down while frozen; `on_screen` leaves
out feet beyond the edge; the cast-freeze count ends only when every caster is done; the new
tuning rules accept good values and reject bad ones naming the field.
