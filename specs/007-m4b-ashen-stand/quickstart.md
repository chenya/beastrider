# Quickstart: M4b Ashen Stand

How to build and play M4b, then check every acceptance item, on the target machine (Ubuntu 24.04
LTS, Intel UHD 630). Prerequisites are unchanged from [M0](../001-m0-setup/quickstart.md); the
export setup is the one from [M2a](../003-m2a-debug-overlay/quickstart.md).

## Build and run

```sh
cd rust && cargo build && cd ..
godot --editor --path godot          # then Play, and click the game window
```

Keys: arrows/WASD walk, **J** or **Z** swing, **K** or **X** cast, **R** restart. Debug builds:
**F1** panel, **F2** hitboxes, **F3** grunt, **F4** refill health, **F5** reload tuning, **F6**
dummy, **F7** brute, **F8** next wave, **F9** fill the magic meter. Before editing
`godot/data/tuning.json`, note its starting values ([contracts/tuning-data.md](contracts/tuning-data.md))
and restore them afterwards.

## Validation scenarios

| # | Spec | Do | Expect |
| --- | --- | --- | --- |
| 1 | US1 sc. 1, FR-001, FR-002, SC-001 | F3 and F7 for 100 grunt and 50 brute kills (F4 to stay alive), counting the potions | About 15% of grunts and 50% of brutes leave a potion where they fell; about 60% red, 40% blue |
| 2 | US1 sc. 2, FR-003, FR-004, SC-002 | Get hurt, then walk over a red potion at its depth | It vanishes; health rises by 30, never above 100 |
| 3 | US1 sc. 3, SC-002 | Walk along a potion's position about 40 px above or below it | It stays |
| 4a | Edge Cases, FR-004, SC-002 | At full health, walk over a red potion | It stays on the ground; collect it after taking a hit |
| 4b | Edge Cases, FR-004, SC-002 | With a full meter (F9), walk over a blue potion | It stays on the ground; collect it after a cast |
| 5 | US1 sc. 4, FR-005, SC-003 | Leave a potion alone, timing it | It blinks for its last 2 s and is gone 8 s after it fell |
| 6 | US2 sc. 1, FR-006 | Collect blue potions one by one | One pip fills per potion, up to 3 pips |
| 7 | US2 sc. 2, FR-007, SC-004 | F9, gather five or more enemies, press K | The action freezes about 0.6 s, the screen flashes, every enemy on screen loses health and is knocked flat for about 1.2 s; the meter is empty |
| 8 | US2 sc. 3, FR-008, SC-004 | With one brute alone on screen, cast with 1 charge; once it rises, cast with 3 (F9 and collect or set the meter between) | Its bar loses a sixth (20 of 120), then a further half (60) |
| 9 | Planning answer, FR-007 | Cast while a grunt is still walking in with its feet beyond the edge | That grunt is untouched; one already on screen is hit |
| 10 | US2 sc. 4, FR-009, SC-004 | Cast just as a brute's strike would land | The hero loses no health during the cast |
| 11 | Clarification, FR-007 | Cast mid-swing, then while hit-stunned | Each cast starts at once |
| 12 | FR-009a, FR-010 | Cast at a brute mid-attack, then swing at it while it is down | It goes flat for the full time, its attack is cancelled, and blows do nothing while it is down; when it rises it goes back to its spot before attacking again |
| 13 | US2 sc. 5, FR-011, SC-005 | Cast with 3 charges at grunts on 30 health; cast to kill a wave's last enemies | Each dies, adds 10 points and a kill, may drop a potion; the wave clears and the breather starts |
| 14 | US2 sc. 6, FR-012, SC-006 | With an empty meter, press K ten times | The meter shakes; nothing else changes |
| 15 | US3 sc. 1, FR-015, SC-008 | Game running: change `magic.damage_mode` to `"flat"`, `potions.heal_amount`, `magic.max_charges` to 5, `enemies.grunt.drop_chance` to 1; save and F5 after each | "Tuning reloaded"; each change shows from the moment the contract gives (any cast does 40; the next potion heals the new amount; 5 pips; every grunt drops) |
| 16 | US3 sc. 2, FR-016 | Set `magic.max_charges` to 0; `potions.blink_time` above `lifetime`; `magic.damage_mode` to `"big"`; a `drop_chance` of 1.5. F5 after each | Each rejected with a message naming the field; the previous values stay in use |
| 17 | Contract warning | Set `enemies.brute.drop_chance` to 0.1, F5 | Accepted, with the warning; restore and F5 clears it |
| 18 | US3 sc. 3–5, FR-013, FR-014, SC-007, SC-008 | F9 during play and at Game Over; R with potions down and charges held | F9 fills the meter in play, does nothing at Game Over; after R, no potions and an empty meter |
| 19 | US3 sc. 4, SC-007 | Release export (M2a quickstart scenario 12 steps); play, cast, press F1–F9 | Potions, the meter, the spell, waves, score and R work; F1–F9 do nothing |
| 20 | US6 sc. 1, SC-009 | F3/F7 to ten enemies, F2 on, panel open; fight 60 s with a cast, F4 to stay alive | FPS 60 throughout, apart from the first second, the impact freezes and the cast |
| 21 | US4, SC-011 | With the final values, play five runs to Game Over | Owner's judgement: the spell turns losing fights, and every run still ends. If not, retune and repeat |
| 22 | US5, SC-012 | Play the whole slice for five minutes | Go if the owner then replays three times in a row unprompted; the outcome and one sentence on why go in the brief |
| 23 | FR-017, FR-018 | Read `magic.rs`, `potion.rs`, `hero.rs`, `enemy.rs`, `run.rs` and the new `.gd` files | Modes as enums matched exhaustively; GDScript only reads and reacts; potions found through the `potions` group; magic through `p{index}_magic` |
| 24 | US6 sc. 2, SC-010 | `cd rust && cargo fmt --check && cargo build && cargo clippy --all-targets && cargo test` | All pass, 0 warnings, including the tests in [research R15](research.md) |
| 25 | US6 sc. 2, SC-010 | Check every `.gd` file through the editor's language server | 0 GDScript errors or warnings |
| 26 | FR-019 | Repeat M4a quickstart scenarios 1, 5, 8 and 14 | Waves, the brute, turns and score work as before |
| 27 | US6 sc. 3, SC-010 | After committing: `git status` | Clean |

## If something goes wrong

- **Nothing drops**: the `WaveDirector`'s `potion_scene` is not set in `main.tscn` (it adds the
  potions), or every `drop_chance` is 0.
- **The cast freezes forever**: an object ended its step without the caster; R still works and
  clears the freeze. Report it: the cast freeze count must always end.
- **A potion or enemy never moves**: a script was put on the `Potion` or `Enemy` root; it replaces
  the Rust `ready()` (M2b R6). Scripts belong on `Body`.
- **A panic "already bound" when a signal fires**: a handler called back into the hero that
  emitted it; handlers use their arguments ([M3 research R8](../005-m3-first-fight/research.md)).
