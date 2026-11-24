# Quickstart: M4a Waves and the Brute

How to build and play M4a, then check every acceptance item, on the target machine (Ubuntu 24.04
LTS, Intel UHD 630). Prerequisites are unchanged from [M0](../001-m0-setup/quickstart.md); the
export setup is the one from [M2a](../003-m2a-debug-overlay/quickstart.md).

## Build and run

```sh
cd rust && cargo build && cd ..
godot --editor --path godot          # then Play, and click the game window
```

Keys: arrows/WASD walk, **J** or **Z** swing, **R** restart. Debug builds: **F1** panel, **F2**
hitboxes, **F3** spawn a grunt, **F4** refill health, **F5** reload tuning, **F6** spawn a dummy,
**F7** spawn a brute, **F8** next wave. Before editing `godot/data/tuning.json`, note its starting
values ([contracts/tuning-data.md](contracts/tuning-data.md)) and restore them afterwards.

## Validation scenarios

| # | Spec | Do | Expect |
| --- | --- | --- | --- |
| 1 | US1 sc. 1, FR-001–FR-003, SC-001 | Press Play | "Wave 1" top centre for about 1.5 s; then 2 grunts enter one after another from random edges at random depths. Score 0, kills 0 |
| 2 | US1 sc. 2–3, FR-004, SC-001 | Clear waves 1–3, watching the gap after each | No banner while an enemy lives (entering ones too); after the last dies, about 2 s with nothing, then "Wave N+1" and its group (3, then 4 grunts) |
| 3 | US1 sc. 4, FR-005, SC-002 | F8 until wave 10, noting each wave's count with F2 on | Waves 4–5 as listed (3 grunts + 1 brute, 4 + 1); from wave 6 one more grunt each wave and one more brute every 3 waves (wave 8: 7 grunts, 2 brutes); never fewer than the wave before |
| 4 | Edge cases, FR-004a | At wave 10, count enemies on screen | Never more than 10 alive at once; the rest enter as others die; the wave clears only when all are dead |
| 5 | US2 sc. 1–2, FR-006, FR-007, SC-003 | F7; kill the brute, counting blows; let it strike once | A bigger shape in its own colour with its own bar; 12 blows to kill (a grunt takes 3); its strike costs 35 of 100 (a grunt's 15) |
| 6 | US2 sc. 3, SC-003 | Watch 5 brute wind-ups next to grunt wind-ups; step out of depth during each | Its colour change lasts clearly longer than a grunt's (0.7 s vs 0.45 s); every dodged strike misses |
| 7 | US2 sc. 4–5, FR-008, SC-003 | Hit a waiting brute; then hit a brute 10 times during its wind-up, staying in reach | Waiting: effects, a short push (about 15 px) and a very short stun. Winding up: effects and damage, no push, no stun, and the strike still comes, 10 of 10 |
| 8 | US3 sc. 1–3, FR-010, FR-011, SC-004 | F3 until 6 grunts are in play; watch 60 s with F2 on and F4 to stay alive | At most 2 close in or attack at once; the others wait on their own side about 180 px away, facing the hero; an attacker that recovers or is stunned falls back to its spot first; the freed turn goes to the waiting grunt nearest the hero, or to one that has waited over 4 s; every grunt gets turns |
| 9 | US3 sc. 1, 3, SC-004 | With grunts waiting, F7 a brute and let it take a turn | While the brute attacks (cost 2), no grunt closes in; the brute gets turns too (at the latest once it has waited 4 s); while it is first in line with one token free, no grunt jumps ahead |
| 10 | US3 sc. 2, Edge cases | With enemies waiting, walk the hero towards one, then across the lane | It never attacks without a turn; it drifts back to its spot on its own side as the hero moves |
| 11 | US3 sc. 4, FR-012, SC-005 | Same 60 s as scenario 8, and again with 10 enemies | No two enemies drawn fully over one another while approaching or waiting, including at the lane edges |
| 12 | US4 sc. 1, FR-013, SC-006 | Kill a grunt, then a brute | The score rises by 10, then by 30, beside the kills |
| 13 | US4 sc. 2, FR-015, SC-006 | Let the enemies win, once mid-wave and once during a breather | Game Over shows the score (same as the HUD), the wave reached (mid-wave: that wave; breather: the wave just cleared), kills, best and "Press R to restart" |
| 14 | US4 sc. 3–4, FR-014, FR-016, SC-007 | Three runs with different scores, R at Game Over and mid-run, timing each; then quit and relaunch | Each R: within 2 s, wave 1, score 0, kills 0, nothing left over; best shows the highest score so far, including a run ended with R; after relaunch best is 0 |
| 15 | US5 sc. 1, FR-017, SC-009 | Game running: change wave 2's grunts, `enemies.brute.max_health`, `crowd.max_attack_tokens` to 1, `crowd.drift_speed` to 0, `enemies.grunt.points`; save, F5 after each | "Tuning reloaded"; each change shows from the moment the contract gives (wave 2's count when its banner starts; one attacker at a time; waiting enemies stand still; the next kill's points) |
| 16 | US5 sc. 2, FR-018 | Set `waves.list` to `[]`; then a wave of 0 + 0; `enemies.brute.attack_token_cost` to 3; `enemies.brute.knockback_resistance` to 1.5; `interruptible` to `"no"`. F5 after each | Each rejected with a message naming the field and why; the previous values stay in use |
| 17 | Contract warnings | Set `enemies.brute.attack.wind_up_time` to 0.4, F5 | Accepted, with the warning that the brute's wind-up is not longer than the grunt's; restore and F5 clears it |
| 18 | US5 sc. 3–4, FR-019, SC-008 | F3 and F7 during a wave, then during a breather; F8 mid-wave and during a breather; all three at Game Over | F3/F7: one enemy enters and joins the wave (during a breather, the wave must be cleared again). F8: every enemy vanishes, no points or kills, the next banner at once. At Game Over: nothing |
| 19 | US5 sc. 5, FR-020, SC-008 | Release export (M2a quickstart scenario 12 steps); play to Game Over; press F1–F8, then R | Waves, brutes, score, Game Over with best and R all work; F1–F8 do nothing |
| 20 | US7 sc. 1, SC-010 | F3/F7 to ten enemies (mixed), F2 on, panel open; fight 60 s by the clock, F4 to stay alive, window focused | FPS 60 throughout, except the first second and the impact freezes |
| 21 | US6, SC-012 | With the final values, play five runs to Game Over | Owner's judgement: wave 1 easy, each wave noticeably harder, every death felt earned. If not, retune (15–17) and repeat |
| 22 | FR-009, FR-022, FR-023 | Read `enemy.rs`, `crowd.rs`, `wave_director.rs`, `run.rs` and the new or renamed `.gd` files | No behaviour branch on the kind; modes and wave states as enums matched exhaustively; GDScript only reads and reacts; tokens and waiting spots count against the nearest living hero |
| 23 | US7 sc. 2, SC-011 | `cd rust && cargo fmt --check && cargo build && cargo clippy --all-targets && cargo test` | All pass, 0 warnings, including the tests in [research R13](research.md) |
| 24 | US7 sc. 2, SC-011 | Open every `.gd` file in the editor | 0 GDScript errors or warnings |
| 25 | FR-021 | Repeat M3 quickstart scenarios 3, 4, 8 and 13 and M2b scenario 1 against an F6 dummy | Hero hits, stun, blink, Game Over, F4, F6, F2 and the dummy work as before |
| 26 | US7 sc. 3, SC-011 | After committing: `git status` | Clean |

## If something goes wrong

- **An enemy never moves or cannot be hit**: a script was put on the `Enemy` root; it replaces the
  Rust `ready()` (M2b research R6). Scripts belong on `Body` and `HealthBar`.
- **Enemies all stand waiting and none attacks**: `crowd.max_attack_tokens` is smaller than every
  cost, which the rules reject, or the `WaveDirector` is missing from `main.tscn` (it hands out the
  tokens).
- **Enemies jitter against each other**: `crowd.separation_radius` is larger than the waiting
  distance allows on a crowded side; lower it, or raise `waiting_distance`.
- **A panic "already bound" when a signal fires**: a handler called back into the object that
  emitted it; handlers use their arguments and poll in `_process` instead
  ([M3 research R8](../005-m3-first-fight/research.md)).
