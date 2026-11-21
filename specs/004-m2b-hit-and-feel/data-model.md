# Data Model: M2b Hit and Feel

Entities from the spec, who owns each one, and the rules that keep them valid. "Rules side" is
Rust; "presentation" is GDScript and the scene. M1 and M2a entities not listed are unchanged.

## Hero (rules side, `Hero`, extended)

| Field | Type | Rules |
| --- | --- | --- |
| `setup` | `Option<Setup>` | As M2a; `Setup` gains the `attack` action and `HitStopTimer` handle |
| `facing` | `Facing { Left, Right }` | `Right` at start; set from the last non-zero horizontal input, only while `Free` |
| `mode` | `HeroMode` | See below |
| `hits_landed` | integer | Starts at 0; +1 per dummy hit; never reset in M2b |

```text
HeroMode::Free ──attack pressed (not frozen)──▶ Attacking { WindUp, elapsed 0, hit_this_swing [] }

Attacking:  WindUp ──wind_up_time──▶ Active ──active_time──▶ Recovery { queued } ──recovery_time──▶
              │                        │                        │
   presses dropped        hits checked every tick    a press sets queued = true
                                                                        │
                    queued = false ──▶ Free          queued = true ──▶ new swing (WindUp, cleared)

While HitStop is frozen: no step at all (no input, no timers, no movement).
```

Parts of zero length are passed through in the same tick (`wind_up_time` or `recovery_time` = 0).

## Swing timings and boxes (rules side, pure `combat.rs`)

| Item | Definition |
| --- | --- |
| `SwingTimings` | `wind_up`, `active`, `recovery` (seconds) |
| `Rect` | `left`, `right`, `top`, `bottom` in screen px |
| Attack box | x from `origin.x + f·half_width` to `origin.x + f·(half_width + reach)` (sorted), y from `origin.y - height` to `origin.y`; `f` = +1 facing right, −1 left |
| Hurtbox | x from `origin.x - w/2` to `origin.x + w/2`, y from `origin.y - h` to `origin.y` |
| `lands(...)` | `attack.left <= hurt.right && hurt.left <= attack.right && |attacker_y - target_y| <= depth_tolerance` |
| `extend_freeze(remaining, new)` | `max(remaining, new)` |
| `knockback_offset(d, t, e)` | `d · (1 - (1 - min(e, t)/t)²)`; `0` if `d == 0` |
| `spawn_point(...)` | x starts at `origin.x + f·(half_width + reach - w/2)`, steps `f·w/2` while any dummy's x is within `w/2`; clamped to `[w/2, screen_width - w/2]`; y = hero's feet y |

## TrainingDummy (rules side, Rust class)

| Field | Type | Rules |
| --- | --- | --- |
| position | `Node2D` position at the feet | Inside the lane and the screen after every tick |
| `slide` | `Option<Slide>` | `Slide { start_x, direction, distance, time, elapsed }`; `None` when at rest |
| `tuning`, `hit_stop` | handles to the autoloads | Found in `ready`; missing ⇒ one `TrainingDummy:` error, never moves |

On `take_hit(direction)`: replace `slide` with a new one from the current x (if `knockback_distance
> 0`), and emit `was_hit(direction, knockback_distance)`. Each tick, unless frozen: advance
`elapsed`, set x = `start_x + direction · knockback_offset`, clamp x and y; drop the slide when
`elapsed >= time` or x hit a limit.

## HitStopTimer (rules side, autoload `HitStop`)

| Field | Type | Rules |
| --- | --- | --- |
| `remaining` | `f32` seconds | `>= 0`; counted down in real physics time; `freeze(s)` sets it to `extend_freeze(remaining, s)` |

## Tuning (rules side, `TuningValues`, extended)

M1's four values plus (file path → rule → starting value):

| Field | Rule | Start |
| --- | --- | --- |
| `attack.wind_up_time` | finite, `>= 0` | 0.08 |
| `attack.active_time` | finite, `> 0` | 0.1 |
| `attack.recovery_time` | finite, `>= 0` | 0.18 |
| `attack.reach` | finite, `> 0` | 70 |
| `attack.height` | finite, `>= 0` | 60 |
| `attack.depth_tolerance` | finite, `>= 0` | 20 |
| `impact.hit_stop_time` | finite, `>= 0` (0 = off) | 0.06 |
| `impact.flash_time` | finite, `>= 0` (0 = off) | 0.08 |
| `impact.shake_strength` | finite, `>= 0` (0 = off) | 6 |
| `impact.shake_time` | finite, `>= 0` (0 = off) | 0.15 |
| `impact.knockback_distance` | finite, `>= 0` (0 = off) | 60 |
| `impact.knockback_time` | finite, `>= 0`, and `> 0` if `knockback_distance > 0` | 0.2 |
| `dummy.hurtbox_width` | finite, `>= 0` | 56 |
| `dummy.hurtbox_height` | finite, `>= 0` | 120 |

## Presentation

- **Flash** (`dummy_flash.gd` on each dummy): tween state only; reads `Tuning.flash_time()`.
- **Shake** (`screen_shake.gd` on `ShakeCamera`): the time left and strength of the current shake
  (UI state); reads `Tuning.shake_strength()` and `shake_time()` when a shake starts.
- **Hitbox drawer** (`debug_hitboxes.gd`): no state; reads boxes from Rust every frame it is shown.
- **Overlay** (extended): `Hits: N` line; F2 toggles the drawer; F3 spawns a dummy.

## Input Map (added)

| Action | Keys (physical) |
| --- | --- |
| `p1_attack` | J, Z |
| `debug_show_hitboxes` | F2 |
| `debug_spawn_dummy` | F3 |
