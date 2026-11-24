# Contract: tuning data for M4a

`godot/data/tuning.json` gains `enemies`, `waves` and `crowd`; M3's `grunt` moves to
`enemies.grunt` ([research R10](../research.md)). One file, one loader, the M2a all-or-nothing F5
reload. Field paths in messages follow the nesting, for example `enemies.brute.attack.wind_up_time`
or `waves.list[2].grunts`.

```json
{
  "hero": { "...": "unchanged from M3" },
  "enemies": {
    "grunt": {
      "horizontal_speed": 150,
      "vertical_speed": 90,
      "max_health": 30,
      "hurtbox_width": 52,
      "hurtbox_height": 100,
      "hit_stun_time": 0.3,
      "knockback_resistance": 0,
      "interruptible": true,
      "attack_token_cost": 1,
      "points": 10,
      "attack": {
        "wind_up_time": 0.45, "active_time": 0.1, "recovery_time": 0.4, "reach": 60,
        "height": 60, "damage": 15, "knockback_distance": 80, "knockback_time": 0.25
      }
    },
    "brute": {
      "horizontal_speed": 100,
      "vertical_speed": 60,
      "max_health": 120,
      "hurtbox_width": 72,
      "hurtbox_height": 140,
      "hit_stun_time": 0.4,
      "knockback_resistance": 0.75,
      "interruptible": false,
      "attack_token_cost": 2,
      "points": 30,
      "attack": {
        "wind_up_time": 0.7, "active_time": 0.12, "recovery_time": 0.6, "reach": 75,
        "height": 80, "damage": 35, "knockback_distance": 120, "knockback_time": 0.3
      }
    }
  },
  "dummy": { "...": "unchanged from M3" },
  "lane": { "...": "unchanged from M3" },
  "effects": { "...": "unchanged from M3" },
  "run": { "death_fade_time": 0.5 },
  "waves": {
    "list": [
      { "grunts": 2, "brutes": 0 },
      { "grunts": 3, "brutes": 0 },
      { "grunts": 4, "brutes": 0 },
      { "grunts": 3, "brutes": 1 },
      { "grunts": 4, "brutes": 1 }
    ],
    "extra_grunts_per_wave": 1,
    "brute_every_waves": 3,
    "spawn_interval": 0.6,
    "breather_time": 2.0,
    "banner_time": 1.5,
    "max_on_screen": 10
  },
  "crowd": {
    "max_attack_tokens": 2,
    "waiting_distance": 180,
    "drift_speed": 40,
    "separation_radius": 56,
    "max_wait_time": 4.0
  }
}
```

Units: times in seconds, sizes, distances and speeds in pixels (per second), health and damage in
points, resistance a fraction. Values are data, not requirements (spec, Assumptions). These are the
values M4a shipped with: the owner's balance playtest (T040) kept the starting values unchanged. With them the brute takes 12 hero blows (grunt 3), hits for 35 (grunt 15),
winds up 0.7 s (grunt 0.45 s), and a blow outside its attack pushes it 15 px and stuns it 0.1 s.

## Where M3's values went

| M3 field | M4a field |
| --- | --- |
| `grunt.*` | `enemies.grunt.*` |
| `run.respawn_delay` | removed (waves replace the respawn; `waves.breather_time` is the nearest equivalent) |
| everything else | unchanged |

## Rules

Every number must be finite; every value is required. M3's rules stay for `hero`, `dummy`, `lane`,
`effects` and `run`, and apply to each `enemies.<kind>` as they did to `grunt`.

| Field | Rule |
| --- | --- |
| `enemies.<kind>.knockback_resistance` | `0 <= r <= 1` |
| `enemies.<kind>.interruptible` | `true` or `false` |
| `enemies.<kind>.attack_token_cost` | a whole number, `1 <= cost <= crowd.max_attack_tokens` |
| `enemies.<kind>.points` | a whole number, `> 0` |
| `waves.list` | an array with at least one entry |
| `waves.list[i].grunts`, `waves.list[i].brutes` | whole numbers, `>= 0`, and `grunts + brutes >= 1` |
| `waves.extra_grunts_per_wave`, `waves.brute_every_waves` | whole numbers, `>= 0`, not both 0 (waves must keep growing) |
| `waves.max_on_screen` | a whole number, `>= 1` |
| `waves.spawn_interval`, `breather_time`, `banner_time` | `>= 0` |
| `crowd.max_attack_tokens` | a whole number, `>= 1` |
| `crowd.waiting_distance` | `> 0` |
| `crowd.drift_speed`, `crowd.separation_radius` | `>= 0` (0 stops a waiting enemy once at its spot / turns the pass off) |
| `crowd.max_wait_time` | `>= 0` (0: turns go to the longest waiting) |

## Warnings (accepted, not rejected)

| Condition | Message |
| --- | --- |
| `enemies.<kind>.attack.wind_up_time <= hero.attack.wind_up_time` | `enemies.<kind>.attack.wind_up_time: not longer than hero.attack.wind_up_time ({hero}), got {enemy}; the {kind}'s attack may not be seen coming` |
| `enemies.brute.attack.wind_up_time <= enemies.grunt.attack.wind_up_time` | `enemies.brute.attack.wind_up_time: not longer than enemies.grunt.attack.wind_up_time ({grunt}), got {brute}; the brute should telegraph longer than the grunt` |

Shown as in M3: printed to the output panel and after "Tuning reloaded" until the next reload; the
overlay shows the first.

## When a reloaded value takes effect

- `enemies.<kind>` values: as M3's grunt values (speeds the next tick, attacks from the next attack,
  hit values the next hit, `max_health` at once). `knockback_resistance` and `interruptible`: the
  next hit. `attack_token_cost`: the next grant. `points`: the next kill.
- `waves.list` and the escalation rule: the next wave whose banner starts; the current wave keeps
  its queue. `spawn_interval`: the next spawn. `breather_time`, `banner_time`: the next breather or
  banner. `max_on_screen`: the next spawn.
- `crowd.max_attack_tokens`: the next grant; holders keep their tokens. `waiting_distance`,
  `drift_speed`, `separation_radius`: the next tick. `max_wait_time`: the next grant.

Errors follow M2a: one message naming the field path and the reason, for example
`enemies.brute.attack_token_cost: must be at most crowd.max_attack_tokens (2), got 3`.
