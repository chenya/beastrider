# Beastrider: Project Brief and Milestone Plan

As of 2026-11-21. Status: M0, M1, M2a (debug overlay) and M2b (hit and feel) done and merged into main (the owner is working through Spec Kit milestone by milestone, one branch each); M3 (first fight) is next. Constitution v1.3.3. The live, editable copy is the doc "Beastrider: Project Brief and Milestone Plan"; this file is the project's shared-memory copy. If the two differ, ask the user which is current. The Spec Kit constitution is in `.specify/memory/constitution.md`.

## Summary

Beastrider is a side-scrolling fantasy brawler in the spirit of Golden Axe, built one playable step at a time, with a dark, painterly mood in the tradition of Frank Frazetta's fantasy art. Characters, creatures and scenes are all original; nothing recreates a specific painting or game asset.

- Genre: 2D side-scrolling beat 'em up on a lane (left/right plus up/down within a strip of ground).
- Target feel: weighty, satisfying hits; grim, mythic atmosphere with heavy shadow, silhouettes, red sky and firelight.
- Platform: Ubuntu 24.04 LTS desktop, single machine, must run smoothly on an Intel UHD 630 integrated GPU (target 60 fps at 1280x720, V-Sync on, stock Mesa driver).
- Engine: Godot 4.7.2, Compatibility renderer (OpenGL), built for older GPUs and integrated graphics and usually good enough for 2D.
- Languages: Rust through gdext (0.5.5, API level 4.6) owns rules and state (hero and enemy state machines, combat and damage, hit detection, wave spawning, loading and checking tuning data). Typed GDScript owns presentation and glue (HUD, menus, debug overlay, flash and shake effects, scene wiring). GDScript holds no game state; it reads Rust objects and reacts to their signals. Rust follows the Google Rust Style Guide; GDScript follows the official GDScript style guide.
- Developer: has built Rust projects before; new to Godot and gdext.
- Players: one player first, then local co-op on one machine. Online play is out of scope. Input actions are named per player (p1_left and so on).
- Art approach: cutout animation (a few separate pieces posed with Godot's animation tools). Art source undecided.
- Incremental rule: nothing from the later list starts until the current milestone is playable and feels good.
- Working method: spec-driven development with Spec Kit, one feature branch per milestone.
- Time budget: about 15 hours a week.

## Core loop and first slice

Advance, fight, collect, survive, repeat until a boss falls or you do.

1. Move into the lane and push right as the screen scrolls.
2. Fight groups of enemies with attack and jump.
3. Pick up health and magic potions that enemies drop.
4. Spend magic on a screen-clearing spell when swamped.
5. Beat the boss, or die and restart.

First slice, "Ashen Stand": one hero on one screen under a red sky, fighting escalating waves of two enemy types, with potion drops and a magic spell, ending in game over and a restart. Done when the user replays it voluntarily three times in a row.

## Must-have for the first slice

- Hero moving in four directions inside lane bounds, with attack, jump and one magic spell.
- Two enemy types: a grunt (walks to you and strikes) and a brute (slower, hits harder, takes more blows).
- Hit feedback: brief freeze on impact, knockback, white flash, small screen shake.
- Escalating waves until the hero dies, with a score at game over.
- Health bar, magic meter, health and magic potion drops.
- Restart key returning to wave one in under two seconds.
- Placeholder art only (coloured shapes) until the loop is fun.
- All tuning values (damage, speed, health, wave sizes) in data files.
- Debug overlay (F1) with live tuning reload (F5); cheats arrive with what they act on: show hitboxes and spawn an enemy (M2b), refill health (M3), refill magic (M4).
- Version control from the first session, and a player index in the code instead of a hard-coded "the player", so co-op stays possible.

## Later list (roughly in order)

1. Painted background, dramatic lighting, vignette.
2. Cutout animation for the hero, replacing coloured shapes.
3. Scrolling stage with encounter zones (screen locks until a group is beaten).
4. More enemy types and one boss.
5. Further playable heroes with different attacks.
6. Local two-player co-op on one machine.
7. Mountable beasts.
8. Multiple stages, title screen, simple story.
9. Sound effects and music.
10. A packaged build to send to friends.

## Not doing

- Online multiplayer of any kind.
- Procedurally generated levels.
- A custom engine, 3D, or heavy shaders beyond a simple vignette and colour grade.
- Frame-by-frame hand-drawn animation.
- Save system, inventory, levelling.
- Cutscenes and voice acting.
- Mod support, console or mobile ports.
- Recreating any specific Frazetta painting, or any Golden Axe character, creature or asset.
- Starting any later-list item while the current milestone is unfinished.

## Working agreements

1. Each milestone is written as a short spec with acceptance checks before any code; it is done only when every check passes by playing.
2. If a session changes the plan, update the spec first, then the code.
3. One milestone at a time, two or three sessions each, always ending with something runnable.
4. Add a pattern only when a concrete problem demands it, and name the problem.
5. Numbers and content in data files from day one.
6. Commit to version control at the end of every passing milestone.
7. If it is unclear whether the loop is fun, stop and playtest first.
8. If a session stalls, shrink the next step; never guilt.

## Milestones (about two or three sessions each; M2a is about one)

| Milestone | End state | Acceptance check |
| --- | --- | --- |
| M0 Setup | Godot project, Rust gdext library and version control on Ubuntu 24.04 | Rust builds and loads in Godot; a GDScript scene calls into a Rust class; a window opens with the Compatibility renderer and a Rust-driven coloured square holds 60 fps (V-Sync on) on the UHD 630 |
| M1 Lane movement | A hero rectangle moves in four directions inside a lane strip, with pillars to sort against | Instant start and stop, diagonals no faster, vertical slower than horizontal, stays in bounds, sorts by depth via Y-sort with origins at the feet |
| M2a Debug overlay | F1 overlay with FPS, hero position, speeds, lane-limit lines and origin marker; F5 reloads tuning data live | Editing a speed and pressing F5 changes the hero with no restart; a bad value is rejected with a clear message; absent from release builds |
| M2b Hit and feel | A still dummy enemy you can hit; show-hitboxes and spawn-dummy cheats | A hit gives knockback, flash, impact freeze and shake, all tunable from a data file |
| M3 First fight | A chasing grunt, health bars, death, game over, restart, refill-health cheat | Play from start to game over and restart in under two seconds |
| M4 Ashen Stand | Waves, the brute, potion drops, the magic spell, refill-magic cheat | Five-minute playtest is fun enough to replay three times; go or no-go decision |
| M5 The look | Red sky, lighting, vignette, first cutout-animated hero | Mood reads in a screenshot; still 60 fps on the UHD 630 |
| M6 The stage | Scrolling level with locked encounter zones and a boss | Finish a full stage from the left edge to the boss |
| M7 Local co-op | A second player on the same machine | Two players complete a full stage without input or camera problems |
| M8 Beasts | A mountable creature with its own moves | Mount, fight and dismount a beast during a stage |

## Open decisions

- Versions (decided in the M0 plan): Godot 4.7.2, gdext 0.5.5 with API level 4.6, Rust 1.100.0 (gdext 0.5 needs Rust 1.94 or newer).
- Data format for tuning data (JSON or Godot Resources): chosen in the M0 plan.
- Ubuntu notes (from memory): the UHD 630 uses Mesa's iris driver, well above the OpenGL 3.3 the Compatibility renderer needs; if anything misbehaves under Wayland, test under X11.
- Art source for character pieces: undecided; not needed until M5.
- Pace: about 15 hours a week, roughly one to two weeks per milestone. Estimates only.

## Next step

Run `/speckit-specify` for M2a on branch `feat/m2a-debug-overlay`, then plan, tasks and implement. After that, M2b brings the first hit.
