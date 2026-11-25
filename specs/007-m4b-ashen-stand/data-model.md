# Data Model: M4b Ashen Stand

The state each new or changed part holds, its rules and how it moves between states. Decisions
are in [research.md](research.md); the Godot-facing surface is in
[contracts/magic-classes.md](contracts/magic-classes.md) and the data file in
[contracts/tuning-data.md](contracts/tuning-data.md).

## Health (extended, `rust/src/fighter.rs`)

- `heal(amount) -> f32`: while alive, `current = min(current + amount, max)`; returns the health
  gained (0 when dead or full).
- `is_full() -> bool`: `current >= max`.

## MagicMeter (`rust/src/magic.rs`, pure)

`MagicMeter { charges: u32, max: u32 }`, one per hero, created empty on the hero's first tick with
tuning.

| Operation | Rule |
| --- | --- |
| `new(max)` | `charges = 0` |
| `add_charge() -> bool` | `charges < max`: `charges + 1`, true; else unchanged, false |
| `refill()` | `charges = max` (F9) |
| `spend_all() -> u32` | returns `charges`, leaves 0 |
| `set_max(max)` | on each tick from `magic.max_charges`; `charges = min(charges, max)` |
| `is_full()`, `is_empty()` | `charges == max`, `charges == 0` |

## Spell damage (`rust/src/magic.rs`, pure)

`SpellDamageMode { PerCharge, Flat }` and
`spell_damage(mode, per_charge, flat, charges) -> f32` = `per_charge × charges` | `flat`.

## Hero (extended)

New fields: `magic: Option<MagicMeter>`. `HeroMode` gains `Casting(Cast)`,
`Cast { timer: Countdown, charges: u32 }`.

| From | Event | To | Effects |
| --- | --- | --- | --- |
| `Free`, `Attacking`, `HitStun` | magic pressed, `charges > 0` | `Casting` | `spend_all`; swing or stun dropped; knockback slide stopped; `Run.begin_cast()`; `HitStop.clear()`; `cast_started(n)`, `magic_changed` |
| `Free`, `Attacking`, `HitStun` | magic pressed, `charges == 0` | unchanged | `cast_failed_empty()` |
| `Casting` | countdown ends | `Free` | impact (below); `Run.end_cast()`; `spell_impact()` |
| `Casting` | magic or attack pressed | unchanged | nothing |
| `Dead` | anything | `Dead` | nothing |

`can_cast(mode)`: `Free | Attacking | HitStun` → true; `Casting | Dead` → false.

While `Casting`: no moving or turning; its target is invulnerable (no blow lands); the post-hit
`invulnerable` countdown keeps its own time. `cast_progress()` = elapsed / `cast_time` (0 outside
a cast).

**Impact**: `damage = spell_damage(...)`; for each `FighterRef` in `team_enemies` whose feet are
`on_screen(feet, viewport_rect)`: an `Enemy` gets `take_spell(damage, caster_x)`, a
`TrainingDummy` takes `damage` as from a blow.

**Pickup** (each tick the hero steps, not while `Dead`): for each node in `potions`, collect when
`can_collect(hero_box, hero_feet_y, potion_box, potion_feet_y, lane.depth_tolerance)` and
`potion_useful(kind, health.is_full(), magic.is_full())`. Health: `heal(potions.heal_amount)`;
Magic: `add_charge()`, `magic_changed`. Then `potion_collected(kind)` and the potion is freed.

`can_collect`: the boxes overlap horizontally (as `lands`) and `|hero_feet_y − potion_feet_y| <=
depth_tolerance`. `potion_useful`: `Health` → not `health_full`; `Magic` → not `meter_full`.

## Enemy (extended)

`EnemyMode` gains `KnockedDown(Countdown)`. New field `drop: Option<PotionKind>` (pending, taken
by the director).

| Mode | `holds_turn` | blows land | crowd `Give` | next |
| --- | --- | --- | --- | --- |
| `KnockedDown` | false | no (target invulnerable) | `Yields` | `Retreating` when the countdown ends |

`enemy_after_spell(mode, damage, knockdown_time, fade) -> EnemyMode`:

| `mode` | `damage` | Result |
| --- | --- | --- |
| `Dying` | any | unchanged |
| any other (`Entering` included) | `Killed` | `Dying(fade)` |
| any other | `Survived` | `KnockedDown(knockdown_time)`, attack cancelled |
| any | `AlreadyDead` | unchanged |

An `Entering` enemy whose feet are on screen takes the damage and, if it survives, is knocked down
like the rest; once it gets up it retreats to its spot, which also ends its entry. Knockdown
ignores `knockback_resistance` and `interruptible`. The push: `Knockback::start(x, away from the
caster, magic.knockdown_push_distance, magic.knockdown_push_time)`.

**Drop**: on entering `Dying` by any path, `drop = Run.roll_drop(own.drop_chance,
potions.health_odds)`. `take_drop() -> Option<(PotionKind, Vector2)>` empties it.

## Drop roll (`rust/src/potion.rs`, pure)

`drop_from_rolls(chance, health_odds, roll_a, roll_b) -> Option<PotionKind>` with rolls in [0, 1):
`roll_a < chance` → a drop; then `roll_b < health_odds` → `Health`, else `Magic`.

## Potion (`rust/src/potion.rs`)

`Potion { kind: PotionKind, life: Option<Countdown>, links }`, in group `potions`.

| State | Rule |
| --- | --- |
| placed | `kind`, position set by `place()` before entering the tree |
| first tick with tuning | `life = potions.lifetime` |
| each tick, not frozen (`HitStop`, cast) | `life` counts down; at 0 the potion frees itself |
| `is_blinking()` | `life <= potions.blink_time` |

`pickup_box()`: `potions.width × potions.height` centred over the feet.

## Run (extended)

New state: `cast_freeze: u32` (casts in progress), `rng: Gd<RandomNumberGenerator>`.
`begin_cast()` +1; `end_cast()` −1 (not below 0); `is_cast_frozen()` = `> 0`. `roll_drop(chance,
odds)`. `restart()` also sets `cast_freeze = 0` and reseeds `rng`. Kills by the spell go through
the same `add_kill(points)` as blows.

## WaveDirector (extended)

Skips its tick while `is_cast_frozen()` as well as `HitStop`. After the enemies tick, for each
enemy with a pending drop, adds a potion from `potion_scene` at its feet, clamped to the lane,
before the hero in `World`.

## Tuning

`TuningValues` gains `PotionTuning { health_odds, lifetime, blink_time, heal_amount, width,
height }` and `MagicTuning { max_charges, damage_mode, damage_per_charge, flat_damage, cast_time,
knockdown_time, knockdown_push_distance, knockdown_push_time }`; `EnemyTuning` gains
`drop_chance`. All `Copy`. Rules in [contracts/tuning-data.md](contracts/tuning-data.md).
