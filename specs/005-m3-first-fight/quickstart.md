# Quickstart: M3 First Fight

How to build and play M3, then check every acceptance item, on the target machine (Ubuntu 24.04
LTS, Intel UHD 630). Prerequisites are unchanged from [M0](../001-m0-setup/quickstart.md); the
export setup is the one from [M2a](../003-m2a-debug-overlay/quickstart.md).

## Build and run

```sh
cd rust && cargo build && cd ..
godot --editor --path godot          # then Play, and click the game window
```

Keys: arrows/WASD walk, **J** or **Z** swing, **R** restart. Debug builds: **F1** panel, **F2**
hitboxes, **F3** spawn a grunt, **F4** refill health, **F5** reload tuning, **F6** spawn a dummy.
Before editing `godot/data/tuning.json`, note its starting values
([contracts/tuning-data.md](contracts/tuning-data.md)) and restore them afterwards.

## Validation scenarios

| # | Spec | Do | Expect |
| --- | --- | --- | --- |
| 1 | US1 sc. 1–2, FR-001–FR-003, SC-001 | Press Play and stand still, 10 times (R between) | Health bar full top-left, kills 0, no dummy. One grunt walks in from the left or right edge at some depth; it first moves up or down to the hero's depth, then walks in and stops at reach. F2 on: its attack box appears only while it strikes |
| 2 | US1 sc. 3–4, FR-004–FR-006, SC-002 | Let it wind up 10 times; each time step up or down out of its depth during the wind-up | The grunt changes colour for clearly longer than the hero's wind-up; all 10 strikes miss: no effects, no health lost |
| 3 | US1 sc. 5–6, FR-007, FR-009, SC-004 | Let a strike land; then stay in place during the blink | Freeze, white flash on the hero, shake, hero knocked away from the grunt; bar drops by the grunt's damage (15 of 100). The hero blinks for about 1 s; strikes in that time do nothing |
| 4 | US1 sc. 7, FR-009 | Start a swing just as the grunt's strike lands | The hero's swing is cut off; the hero cannot move or swing for the stun, then can |
| 5 | US2 sc. 1–2, FR-008, SC-003 | Hit a grunt; then hit one 10 times during its wind-up | Each hit: flash, freeze, shake, knockback, bar drops, short stun. A hit during the wind-up cancels its strike (0 of 10 follow); after the stun it lines up and approaches again |
| 6 | US2 sc. 3–4, FR-010, SC-005 | Kill a grunt (3 blows) | It flashes, fades out over about 0.5 s and is removed; kills shows 1; about 1.5 s later a new grunt enters from a random edge. Hitting it while it fades does nothing |
| 7 | Edge cases, FR-010 | F3 twice (three grunts), kill one, then another | No replacement while any grunt lives; after the last dies, one grunt enters after the delay |
| 8 | US3 sc. 1, 4, FR-011, SC-006 | Let the grunts win; then press walk and attack keys | The hero tips over, grunts stop, Game Over shows "Game Over", the same kill count as the HUD, and "Press R to restart". Keys do nothing. F4 does nothing |
| 9 | US3 sc. 2–3, FR-012, SC-007 | Press R 5 times at Game Over and 5 times mid-run (with F3 grunts, F6 dummies, a fading grunt, during a freeze), timing each | Each time, within 2 s: full health, kills 0, hit counter 0, no leftover grunts or dummies, one grunt entering. F5-tuned values are still in use |
| 10 | US4 sc. 1, FR-015, FR-016, SC-008 | Game running: change each new value in turn (for example halve `grunt.attack.wind_up_time`, double `grunt.horizontal_speed`, set `hero.attack.damage` to 30), save, F5 | "Tuning reloaded"; each change shows from the moment the contract gives (next tick, next attack, next hit, at once for `max_health`) |
| 11 | US4 sc. 2, FR-017 | Set `grunt.max_health` to 0; then `grunt.attack.knockback_time` to 0 (distance 80); then `run.respawn_delay` to -1. F5 after each | Each rejected with a message naming the field and why; the previous values stay in use |
| 12 | FR-006 | Set `grunt.attack.wind_up_time` to 0.05, F5 | Accepted, with a warning that the grunt's wind-up is not longer than the hero's; restore and F5 clears it |
| 13 | US4 sc. 3–5, FR-018, FR-020 | F3; F4 after being hit; F6; F2 | F3: a grunt enters from a random edge. F4: bar full. F6: a dummy in front of the hero, as M2b's F3; grunts ignore it; hitting it adds to the hit counter, not kills. F2: hero, grunt and dummy bodies outlined, attack boxes while active |
| 14 | US4 sc. 6, FR-012, FR-019, SC-009 | Release export (M2a quickstart scenario 12 steps); play a run to Game Over; press F1–F6, then R | The fight, bars, kills, Game Over and R all work; F1–F6 do nothing |
| 15 | US6 sc. 1, SC-010 | F3 to five grunts, F2 on, panel open; fight 60 s by the clock, F4 to stay alive, F3 to keep five | FPS 60 throughout, except the first second and the impact freezes |
| 16 | US5, SC-012 | With the final values, play five runs to Game Over | Owner's judgement: every hit taken could have been seen coming. If not, retune (10–12) and repeat |
| 17 | FR-014, FR-021 | Read `fighter.rs`, `grunt.rs`, `grunt_spawner.rs`, `run.rs`, `hero.rs` and the new `.gd` files | Rules and state in Rust, modes as enums matched exhaustively; GDScript only reads and reacts; health belongs to the hero by player index; grunts target the nearest player |
| 18 | US6 sc. 2, SC-011 | `cd rust && cargo fmt --check && cargo build && cargo clippy --all-targets && cargo test` | All pass, 0 warnings, including the owner's test list ([research R11](research.md)) |
| 19 | US6 sc. 2, SC-011 | Open every `.gd` file in the editor | 0 GDScript errors or warnings |
| 20 | FR-020 | Repeat M2b quickstart scenarios 1, 3 and 8 against an F6 dummy, and M2a scenarios 1, 5 and 7 | Swing, facing, effects, F2, F1, F5 and rejections work as before |
| 21 | US6 sc. 3, SC-011 | After committing: `git status` | Clean |

## If something goes wrong

- **A grunt never moves or cannot be hit**: a script was put on the `Grunt` root; it replaces the
  Rust `ready()` (M2b research R6). Scripts belong on `Body` and `HealthBar`.
- **"Run: … autoload is missing"**: Project Settings → Autoload must list `Tuning`, `HitStop`, then
  `Run` → `res://autoload/run.tscn`.
- **A panic "already bound" when a signal fires**: a handler called back into the object that
  emitted it; handlers use their arguments and poll in `_process` instead ([research
  R8](research.md)).
- **R takes more than 2 s**: check the output for errors during the scene reload; the reload itself
  is a small scene.
