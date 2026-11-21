# Quickstart: M2b Hit and Feel

How to build and play M2b, then check every acceptance item, on the target machine (Ubuntu 24.04
LTS, Intel UHD 630). Prerequisites are unchanged from [M0](../001-m0-setup/quickstart.md); the
export setup is the one from [M2a](../003-m2a-debug-overlay/quickstart.md).

## Build and run

```sh
cd rust && cargo build && cd ..
godot --editor --path godot          # then Play, and click the game window
```

Keys: arrows/WASD walk, **J** or **Z** swing, **F1** panel, **F2** hitboxes, **F3** spawn a dummy,
**F5** reload tuning. Before editing `godot/data/tuning.json`, note its starting values
([contracts/tuning-data.md](contracts/tuning-data.md)) and restore them afterwards.

## Validation scenarios

| # | Spec | Do | Expect |
| --- | --- | --- | --- |
| 1 | US1 sc. 1, FR-001, FR-007, SC-001 | Walk next to the dummy, face it at the same depth, swing 20 times | Every swing: after a short wind-up, a short freeze, the dummy flashes white, the play area shakes (HUD and panel still), then the dummy slides away from the hero and slows to a stop |
| 2 | US1 sc. 2–3, FR-005, SC-001 | Stand at the same horizontal distance but clearly above or below the dummy's depth; swing 20 times. Then swing facing away, and from beyond reach | No freeze, flash, shake or slide on any of them |
| 3 | US1 sc. 4–5, FR-002, FR-004, SC-005 | Walk left, stop, swing; walk right, stop, swing. Hold a direction while swinging | The swing goes the way the hero last walked; the hero does not move or turn until the swing ends, then moves if the key is still held |
| 4 | Edge cases, FR-003 | Press attack during wind-up; during recovery; hold it down | Wind-up press: nothing extra. Recovery press: a second swing starts the moment the first ends. Held: one swing |
| 5 | US1 sc. 6, FR-006, SC-002 | F1 open. Swing 20 times so the reach overlaps the dummy | `Hits` rises by exactly 20 |
| 6 | US1 sc. 7, FR-006, SC-003 | F3 once (a second dummy appears beside the first); swing through both | Both flash and slide; `Hits` rises by 2; one freeze |
| 7 | US1 sc. 8, FR-008, SC-004 | Push a dummy repeatedly into the left and right screen edges; spawn one near the lane top and bottom and hit it | It stops at the edge, fully on screen and inside the lane |
| 8 | US3, FR-014, SC-007 | F2, then swing at the dummy from several spots, including misses | Hurtboxes outlined at all times; the reach shows only while a blow can land; every hit shows them overlapping at a depth within tolerance, every miss shows why. F2 again hides them; F2 works with the panel closed |
| 9 | US3 sc. 2, FR-015 | Press F3 five times, facing right, then turn and press it twice | Each new dummy appears in front of the hero at its depth, half a dummy's width beyond the last, never stacked; facing left, they appear to the left; all can be hit |
| 10 | US2 sc. 1, FR-009, SC-006 | Game running: change each of the 14 values in turn (for example double `attack.reach`, `impact.knockback_distance`, `impact.shake_strength`), save, F5, swing | Each change shows on the next swing; "Tuning reloaded" appears |
| 11 | US2 sc. 2, FR-010, SC-006 | Set `impact.hit_stop_time` to 0, F5, swing; restore. Repeat for `flash_time`, `shake_strength` (and `shake_time`), `knockback_distance` | Each time only that effect is gone; the others remain |
| 12 | US2 sc. 3, FR-010 | Set `impact.knockback_time` to 0 (distance 60); then `attack.active_time` to 0; then `attack.reach` to -5. F5 after each | Each rejected with a message naming the field and why; the previous values stay in use |
| 13 | US4, SC-011 | With the final values, land 30 hits | Owner's judgement: weighty, not floaty. If not, retune (scenarios 10–11) and repeat |
| 14 | US3 sc. 4, FR-017, SC-008 | Release export (M2a quickstart, scenario 12 steps); run it; swing; press F2 and F3 | The swing and all four effects work; F2 and F3 do nothing |
| 15 | US5 sc. 1, SC-009 | F3 four times (five dummies), F2 on, panel open; walk and swing for 60 s by the clock, window focused | FPS 60 throughout, except the first second and the impact freezes |
| 16 | FR-011–013 | Read `rust/src/combat.rs`, `hero.rs`, `training_dummy.rs`, `hit_stop.rs` and the new `.gd` files | Timing, hits, depth, facing and knockback in Rust; modes are enums matched exhaustively; flash and shake only react to `was_hit`; `p1_attack` built from `player_index` |
| 17 | US5 sc. 2, SC-010 | `cd rust && cargo fmt --check && cargo build && cargo clippy --all-targets && cargo test` | All pass, 0 warnings, including the new combat and tuning tests |
| 18 | US5 sc. 2, SC-010 | Open every `.gd` file in the editor | 0 GDScript errors or warnings |
| 19 | US5 sc. 3, SC-010 | After committing: `git status` | Clean |
| 20 | FR-018 | Repeat M2a quickstart scenarios 1, 5 and 7 | F1 overlay, F5 reload and rejections work as before |

## If something goes wrong

- **"Hero: the HitStop autoload is missing"**: Project Settings → Autoload must list `HitStop` →
  `res://autoload/hit_stop.tscn`, after `Tuning`.
- **The flash shows on every dummy at once**: the flash `ShaderMaterial` is shared; it must be
  local to scene ([research R6](research.md)).
- **The HUD shakes**: a HUD node is not under a `CanvasLayer`.
- **A spawned dummy is drawn in front of the hero at an equal depth**: it was added after the hero
  in `World`; spawned dummies must be moved before the hero ([research R7](research.md)).
