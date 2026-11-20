# Quickstart: M2a Debug Overlay

How to build and play M2a, then check every acceptance item. Run it on the target machine (Ubuntu
24.04 LTS, Intel UHD 630). Prerequisites and versions are unchanged from
[M0's quickstart](../001-m0-setup/quickstart.md); scenario 12 also needs the one-time export setup
below.

## Build and run

```sh
cd rust && cargo build && cd ..                 # rebuilds rust/target/debug/libbeastrider.so
godot --editor --path godot                     # opens the project in the editor
```

In the editor, press **Play** (F5 in the editor, before the game window has focus). Click the game
window once so it has keyboard focus; from then on F1 and F5 go to the game.

## One-time export setup (for scenario 12)

1. In the editor: **Editor → Manage Export Templates → Download and Install** (4.7.2).
2. **Project → Export → Add → Linux**; keep the defaults; export path
   `godot/export/beastrider.x86_64`. This writes `godot/export_presets.cfg` (committed).
3. `cd rust && cargo build --release` (the release library the export packs).

## Validation scenarios

| # | Spec | Do | Expect |
| --- | --- | --- | --- |
| 1 | US1 sc. 1–2, FR-001, SC-001 | Press Play. Then press F1 ten times, looking after each | At start: no panel, no lines, no marker; FPS counter top-right as before. Odd presses show a translucent panel top-left plus lines and marker; even presses hide them all |
| 2 | US1 sc. 3, 6, FR-003, SC-002 | Open the panel; walk the hero around, then stand still | Position changes while walking and stops changing when still; FPS updates; `Player: 1`; speeds match `tuning.json` (300 and 180) |
| 3 | US1 sc. 4, FR-004, SC-003 | Panel open: hold up into the lane top for 2 s, then down into the lane bottom | The hero's bottom edge meets the middle of the top bar, then of the bottom bar; no gap, no overlap |
| 4 | US1 sc. 5, FR-005, SC-003 | Panel open: walk the hero around, including behind each pillar | The cross stays at the hero's feet and stays visible behind pillars |
| 5 | US2 sc. 1, 3, FR-006–007, SC-004 | Game running: in `godot/data/tuning.json` double `hero.horizontal_speed`, save, click the game, press F5. Walk right | "Tuning reloaded" appears top centre for about 2 s; the hero crosses the screen in about half the time; the panel shows the new speed. No restart |
| 6 | US2 sc. 2, FR-006 | Game running, panel open: halve `hero.vertical_speed`; then add 50 to `lane.top`; then subtract 50 from `lane.bottom`. F5 after each | Vertical walking halves; the band, the top bar and the hero's top limit move down 50 px together; then the bottom ones move up 50 px. A hero standing outside the new lane is moved inside on the next frame |
| 7 | US2 sc. 4, FR-008, SC-005 | Set `lane.top` to 800 (above `lane.bottom`), save, F5 | Message: `Tuning rejected: lane.top: …` naming 700 and 800; it stays. Output panel shows a `Tuning:` error. The hero walks exactly as before, band and bars unchanged |
| 8 | Edge cases, FR-008–009, SC-005 | One at a time, F5 after each: a speed `0`, a speed `-5`, a speed `"fast"`, delete `lane.top`, `lane.bottom` 800, a stray comma (broken JSON) | Each time: a message naming the field (or the file and line) and the problem; nothing in play changes |
| 9 | US2 sc. 5 | After scenario 8, restore the file, F5 | "Tuning reloaded"; the error message is gone; panel shows `Tuning: ok` |
| 10 | Edge cases, FR-010 | Close the game. Set `hero.vertical_speed` to 0. Play. Restore the file, F5. Then: rename the `p1_left` action, Play, press F5; restore it | First: the message shows the start-up error and the hero stands still; after F5 it walks. Second: `Hero:` error, F5 does not make it walk. **Restore everything afterwards** |
| 11 | Edge cases | Hold F5 for 3 s; press F5 with no change; press F1 quickly ten times while walking | One reload per press (one message); a no-change reload shows "Tuning reloaded" and nothing moves; F1 never leaves a half-shown state; walking never stutters or pauses |
| 12 | US3, FR-013, SC-006 | **Project → Export → Linux → Export Project**, "Export With Debug" **off**. Run `godot/export/beastrider.x86_64`. Walk, then press F1 and F5 several times | No panel, no bars, no cross, no message; the game plays as in M1 |
| 13 | US4, SC-007 | Open the panel and walk continuously for 60 s by the clock, window focused | FPS shows 60 throughout (the first second excepted) |
| 14 | FR-011, FR-012, FR-014 | Read `rust/src/tuning.rs`, `rust/src/hero.rs` and the `godot/debug/*.gd` scripts; open Project Settings | Parsing, rules and the current values are in Rust; scripts only read and display; `debug_toggle_overlay` and `debug_reload_tuning` are in the Input Map; no key checks in code; FPS counter unchanged |
| 15 | US4, SC-008 | `cd rust && cargo fmt --check && cargo build && cargo clippy --all-targets && cargo test` | All pass with 0 warnings, including the new `validate` tests |
| 16 | US4, SC-008 | Open every `.gd` file in the editor | 0 GDScript errors or warnings |
| 17 | US4, SC-008 | After building, playing and committing: `git status` | Clean (the export folder is ignored) |

## If something goes wrong

- **"Hero: the Tuning autoload is missing"**: Project Settings → Autoload must list `Tuning` →
  `res://autoload/tuning.tscn`, enabled.
- **F1/F5 do nothing in the editor's game**: the game window does not have focus (click it), or
  the actions are missing from the Input Map.
- **F5 restarts the game instead of reloading**: the editor window had focus, and F5 is the
  editor's Play shortcut. Click the game window first.
- **Script error "Opcode: 28" mentioning `Tuning`**: a class and an autoload share a name
  ([research R2](research.md)).
- **FPS shows exactly 1**: the window lost focus; see the M0 quickstart.
