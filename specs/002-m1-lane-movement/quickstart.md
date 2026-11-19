# Quickstart: M1 Lane Movement

How to build and play M1, then check every acceptance item. Run it on the target machine (Ubuntu
24.04 LTS, Intel UHD 630). Prerequisites and versions are unchanged from
[M0's quickstart](../001-m0-setup/quickstart.md).

## Build and run

```sh
cd rust && cargo build && cd ..                 # rebuilds rust/target/debug/libbeastrider.so
godot --editor --path godot                     # opens the project in the editor
```

In the editor, press **Play** (F5). Click the game window once so it has keyboard focus.

## Validation scenarios

| # | Spec | Do | Expect |
| --- | --- | --- | --- |
| 1 | US1, FR-001–003 | Press Play | A 1280×720 window: dark background, a lighter band across the lower part, two or three tall pillars standing in it at different depths, a tall hero rectangle in the band, an FPS counter. No drifting square or speed label |
| 2 | US1, FR-004, SC-001 | Tap each direction, then hold and release each | The hero moves the moment a key goes down and stops the moment it comes up, with no build-up and no slide |
| 3 | US1, FR-006, SC-002 | Hold right for 2 s, then right+up for 2 s, from the same start | The diagonal covers no more ground than straight right |
| 4 | US1, FR-005, SC-003 | Hold up for 1 s, then right for 1 s | Up covers visibly less ground than right |
| 5 | US1, FR-007, SC-004 | Hold into each limit (lane top, lane bottom, left edge, right edge) for 5 s; then, against a limit, add a direction along it | The hero stops exactly at the limit, fully on screen, without shaking; with the second key it slides along the limit at normal speed |
| 6 | Edge cases | Hold left+right; hold up+down; hold right with both the arrow and D; hold a key and click outside the window, then release it there and click back | Opposite keys cancel; the two key sets do not add up; the hero is stopped when the window has focus again |
| 7 | US2, FR-009, SC-005 | Walk the hero up and down across each pillar's base while overlapping it, 10 times, and stop exactly on the line once | Feet above the base: the pillar covers the hero. Below: the hero covers the pillar. The switch happens once per crossing, never flickers, and on the line the hero is in front |
| 8 | US3, FR-012, SC-006 | Close the game. In `godot/data/tuning.json`, double `hero.horizontal_speed`, halve `hero.vertical_speed`, and add 50 to `lane.top`. **Do not run cargo.** Press Play | Sideways is twice as fast, up and down half as fast, and the band's top edge and the hero's upper limit have both moved down 50 px. **Restore the file afterwards** |
| 9 | Edge cases, FR-013 | One at a time: set a speed to `0`, then `-5`, then `"fast"`; delete `lane.top`; set `lane.top` above `lane.bottom`; set `lane.bottom` to `800`. Press Play after each | Each time: one `Hero:` error naming the file, field and problem; the hero stays still and the band is not drawn; no crash. **Restore the file afterwards** |
| 10 | US4, SC-007 | Walk the hero continuously for 60 s by the clock, window in front and focused | The FPS counter shows 60 throughout (the first second excepted) and movement looks even, with no regular hitch |
| 11 | FR-010, FR-011, FR-015 | Read `rust/src/hero.rs`, `rust/src/movement.rs` and `godot/world/lane_band.gd`; open Project Settings | Movement and limits are in Rust; the band script only places itself; the actions are `p1_*`; no literal speed or lane value in code; renderer `gl_compatibility`, V-Sync Enabled |
| 12 | US4, SC-008 | `cd rust && cargo fmt --check && cargo build && cargo clippy --all-targets && cargo test` | All pass with 0 warnings, including the movement tests |
| 13 | US4, SC-008 | Open every `.gd` file in the editor | 0 GDScript errors or warnings |
| 14 | US4, SC-008 | After building, playing and committing: `git status` | Clean |

## If something goes wrong

- **"Hero: input action `p1_left` is missing"**: the Input Map entries are not in
  `godot/project.godot`. Add them in Project Settings → Input Map (see
  [contracts/hero.md](contracts/hero.md#input-actions-input-map-project-settings)).
- **Movement looks uneven with a regular hitch** (scenario 10): the display may not be running at
  exactly 60 Hz. See [research.md R4](research.md); the fix is to turn on physics interpolation.
- **The hero is drawn in front of a pillar it should be behind**: check that the pillar and the
  hero are direct children of `World`, that `World` has Y-sort on, and that each origin is at the
  feet ([research.md R2](research.md)).
- **FPS shows exactly 1**: the window lost focus; see the M0 quickstart.
