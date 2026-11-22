# Contract: tuning data for M3

`godot/data/tuning.json` is restructured so the hero and the grunt use the same key names
([research R10](../research.md)). One file, one loader, the M2a all-or-nothing F5 reload. Field
paths in messages gain the nesting level, for example `grunt.attack.wind_up_time`.

```json
{
  "hero": {
    "horizontal_speed": 300,
    "vertical_speed": 180,
    "max_health": 100,
    "hurtbox_width": 48,
    "hurtbox_height": 96,
    "hit_stun_time": 0.25,
    "invulnerability_time": 1.0,
    "attack": {
      "wind_up_time": 0.08,
      "active_time": 0.1,
      "recovery_time": 0.18,
      "reach": 70,
      "height": 60,
      "damage": 10,
      "knockback_distance": 60,
      "knockback_time": 0.2
    }
  },
  "grunt": {
    "horizontal_speed": 150,
    "vertical_speed": 90,
    "max_health": 30,
    "hurtbox_width": 52,
    "hurtbox_height": 100,
    "hit_stun_time": 0.3,
    "attack": {
      "wind_up_time": 0.45,
      "active_time": 0.1,
      "recovery_time": 0.4,
      "reach": 60,
      "height": 60,
      "damage": 15,
      "knockback_distance": 80,
      "knockback_time": 0.25
    }
  },
  "dummy": { "hurtbox_width": 56, "hurtbox_height": 120 },
  "lane": { "top": 430, "bottom": 700, "depth_tolerance": 20 },
  "effects": { "hit_stop_time": 0.06, "flash_time": 0.08, "shake_strength": 6, "shake_time": 0.15 },
  "run": { "respawn_delay": 1.5, "death_fade_time": 0.5 }
}
```

Units: times in seconds, sizes, distances and speeds in pixels (per second), health and damage in
points. Starting values are data, not requirements; the owner retunes them (spec, Assumptions).

## Where M2b's values went

| M2b field | M3 field |
| --- | --- |
| `hero.horizontal_speed`, `hero.vertical_speed`, `lane.top`, `lane.bottom` | unchanged |
| `attack.wind_up_time`, `active_time`, `recovery_time`, `reach`, `height` | `hero.attack.*` |
| `attack.depth_tolerance` | `lane.depth_tolerance` |
| `impact.hit_stop_time`, `flash_time`, `shake_strength`, `shake_time` | `effects.*` |
| `impact.knockback_distance`, `impact.knockback_time` | `hero.attack.knockback_*` |
| `dummy.hurtbox_width`, `dummy.hurtbox_height` | unchanged |

## Rules

Every value must also be a finite number; every value is required.

| Field | Rule |
| --- | --- |
| `hero.horizontal_speed`, `hero.vertical_speed`, `lane.top`, `lane.bottom` | as M1 (speeds `> 0`; `0 <= top < bottom <= screen height`) |
| `hero.max_health`, `grunt.max_health` | `> 0` |
| `*.attack.damage` | `> 0` |
| `*.attack.active_time`, `*.attack.reach` | `> 0` |
| `grunt.horizontal_speed`, `grunt.vertical_speed` | `> 0` |
| `*.hurtbox_width`, `*.hurtbox_height`, `dummy.*` | `> 0` |
| `*.attack.knockback_time` | `>= 0`, and `> 0` when that attack's `knockback_distance > 0` |
| every other field | `>= 0` |

Zero turns an effect off, as in M2b: `effects.hit_stop_time`, `flash_time`, `shake_strength`,
`shake_time`, and each attack's `knockback_distance`. Zero is also allowed for wind-up and recovery
times, hit-stun times, the invulnerability time, the respawn delay and the death fade time.

## Warning (accepted, not rejected)

| Condition | Message |
| --- | --- |
| `grunt.attack.wind_up_time <= hero.attack.wind_up_time` | `grunt.attack.wind_up_time: not longer than hero.attack.wind_up_time ({hero}), got {grunt}; the grunt's attack may not be seen coming` |

The values are used. The warning is printed to the output panel and shown by the overlay after
"Tuning reloaded" until the next reload; a start-up warning is shown at start, as a start-up
rejection is (M2a).

## When a reloaded value takes effect (FR-016)

- Speeds, hurtbox sizes, depth tolerance, lane limits: the next tick.
- An attack's values: from the next attack that starts; one already under way finishes with the
  values it started with (as M2b).
- Hit-stun, invulnerability, damage, knockback and effect values: the next hit.
- `max_health`: at once; current health is lowered to a smaller maximum and never raised.
- `respawn_delay`: the next delay that starts. `death_fade_time`: the next death.

Errors follow M2a: one message naming the field path and the reason, for example
`grunt.attack.knockback_time: must be greater than 0 while grunt.attack.knockback_distance is 80,
got 0`.
