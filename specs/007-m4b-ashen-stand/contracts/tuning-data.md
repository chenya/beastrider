# Contract: tuning data for M4b

`godot/data/tuning.json` gains `enemies.<kind>.drop_chance` and two sections, `potions` and
`magic`. One file, one loader, the M2a all-or-nothing F5 reload. Field paths in messages follow
the nesting, for example `magic.damage_mode` or `enemies.brute.drop_chance`.

```json
{
  "enemies": {
    "grunt": { "...": "unchanged from M4a", "drop_chance": 0.15 },
    "brute": { "...": "unchanged from M4a", "drop_chance": 0.5 }
  },
  "potions": {
    "health_odds": 0.6,
    "lifetime": 8.0,
    "blink_time": 2.0,
    "heal_amount": 30,
    "width": 24,
    "height": 32
  },
  "magic": {
    "max_charges": 3,
    "damage_mode": "per_charge",
    "damage_per_charge": 20,
    "flat_damage": 40,
    "cast_time": 0.6,
    "knockdown_time": 1.2,
    "knockdown_push_distance": 60,
    "knockdown_push_time": 0.25
  }
}
```

Every other section is unchanged from M4a. Units: times in seconds, sizes and distances in pixels,
health and damage in points, chances and odds as fractions. Starting values are data, not
requirements; the owner retunes them (US4). With them, a 1-charge cast nearly kills a grunt (20
of 30), 3 charges kill a grunt and take half a brute's health (60 of 120), and a red potion
restores 30 of the hero's 100.

## Rules

Every number must be finite; every value is required.

| Field | Rule |
| --- | --- |
| `enemies.<kind>.drop_chance` | `0 <= c <= 1` |
| `potions.health_odds` | `0 <= o <= 1` (the share of drops that are red; the rest are blue) |
| `potions.lifetime` | `> 0` |
| `potions.blink_time` | `0 <= b <= potions.lifetime` |
| `potions.heal_amount` | `> 0` |
| `potions.width`, `potions.height` | `> 0` |
| `magic.max_charges` | a whole number, `>= 1` |
| `magic.damage_mode` | `"per_charge"` or `"flat"` |
| `magic.damage_per_charge`, `magic.flat_damage` | `> 0` (both always required) |
| `magic.cast_time` | `> 0` |
| `magic.knockdown_time`, `magic.knockdown_push_time` | `> 0` |
| `magic.knockdown_push_distance` | `>= 0` (0: knocked down where it stands) |

## Warnings (accepted, not rejected)

| Condition | Message |
| --- | --- |
| `enemies.brute.drop_chance <= enemies.grunt.drop_chance` | `enemies.brute.drop_chance: not higher than enemies.grunt.drop_chance ({grunt}), got {brute}; brutes should drop more often` |

## When a reloaded value takes effect

- `drop_chance`, `health_odds`: the next death.
- `potions.lifetime`: potions dropped after the reload; `blink_time`, `width`, `height`: the next
  tick, for every potion. `heal_amount`: the next pickup.
- `magic.max_charges`: the next tick (charges above it are cut). `damage_mode`,
  `damage_per_charge`, `flat_damage`: the next impact. `cast_time`: the next cast.
  `knockdown_time`, `knockdown_push_distance`, `knockdown_push_time`: the next impact.

Errors follow M2a: one message naming the field path and the reason, for example
`magic.damage_mode: must be "per_charge" or "flat", got "big"`.
