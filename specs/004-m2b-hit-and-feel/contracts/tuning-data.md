# Contract: tuning data for M2b

`godot/data/tuning.json` keeps M1's `hero` and `lane` sections unchanged and gains three
sections. One file, one loader, all-or-nothing F5 reload (M2a).

```json
{
  "hero": { "horizontal_speed": 300, "vertical_speed": 180 },
  "lane": { "top": 430, "bottom": 700 },
  "attack": {
    "wind_up_time": 0.08,
    "active_time": 0.1,
    "recovery_time": 0.18,
    "reach": 70,
    "height": 60,
    "depth_tolerance": 20
  },
  "impact": {
    "hit_stop_time": 0.06,
    "flash_time": 0.08,
    "shake_strength": 6,
    "shake_time": 0.15,
    "knockback_distance": 60,
    "knockback_time": 0.2
  },
  "dummy": { "hurtbox_width": 56, "hurtbox_height": 120 }
}
```

Units: times in seconds, sizes and distances in pixels.

These starting values are also the shipped ones: the owner landed 30 hits with them and judged the
impact weighty, not floaty (T027, 2026-11-20), so none was retuned.

| Field | Rule (every value must also be a finite number) |
| --- | --- |
| `attack.active_time`, `attack.reach` | `> 0` |
| `impact.knockback_time` | `>= 0`, and `> 0` when `impact.knockback_distance > 0` |
| every other new field | `>= 0` |

Zero turns an effect off: `hit_stop_time`, `flash_time`, `shake_strength`, `shake_time`,
`knockback_distance`.

Errors follow M2a: one message naming the field path (`impact.knockback_time`) and the reason, for
example `must be greater than 0 while impact.knockback_distance is 60, got 0`.
