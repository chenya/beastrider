# Feature Specification: M2a Debug Overlay

**Feature Branch**: `feat/m2a-debug-overlay`

**Created**: 2026-11-20

**Status**: Draft

**Input**: User description: "m2a-debug-overlay: A debug overlay that shows what the game is doing
and lets the owner retune it without restarting." (full description, acceptance checks and
out-of-scope list as given by the owner in the `/speckit-specify` request)

> **Note on technology names.** A few requirements below name parts of the stack: tuning data is
> read and checked on the rules side (Rust), the overlay is presentation (GDScript) and only
> displays values it reads from the rules side, and the debug keys are named input actions. These
> are constraints from the constitution (the Rust/GDScript split, Principle VIII) and from the
> owner's acceptance checks 6 and 8, not choices made by this spec. Every *how* (node types, class
> names, how release builds leave the overlay out) is left to the plan.

## Clarifications

### Session 2026-11-20

- Q: If the game starts with a bad tuning value (hero stopped) and the owner fixes the file and
  presses F5, should the hero come to life without a restart? → A: Yes, for tuning errors only. A
  successful reload makes the hero move; other startup errors (a missing input action, a bad hero
  shape) still need a restart, because they come from the scene or project settings.
- Q: Should the always-on FPS counter from M0 and M1 stay, or should the FPS appear only in the
  overlay? → A: Keep both. The always-on counter stays where it is; the overlay also lists the FPS
  and sits in a corner that does not cover the counter.
- Q: When F5 is pressed with the overlay hidden, where does the result ("Tuning reloaded" or the
  error) appear? → A: In its own small spot on screen, shown whether the overlay is open or not;
  the overlay also lists the last reload result when it is open.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See what the game is doing (Priority: P1)

The owner presses Play and walks the hero around. Nothing extra is on screen apart from the FPS
counter. The owner presses F1: a translucent panel appears in a corner, listing the FPS, the hero's
position, its player index, and its current horizontal and vertical speeds. The values change live
as the hero moves. While the panel is open, two thin lines cross the screen at the lane's top and
bottom limits, and a small marker sits at the hero's feet, where its position is measured. Pressing
F1 again hides the panel, the lines and the marker.

**Why this priority**: The lane limits and the hero's origin are invisible, so the owner cannot
tell whether a tuning change or a bug is to blame for what they see. Seeing the real values is the
base the reload (User Story 2) and every later cheat build on.

**Independent Test**: Press Play, press F1, walk the hero into the top and bottom of the lane and
around a pillar, then press F1 again. The panel appears and disappears, its values follow the hero,
the hero stops exactly on the lines, and the marker stays at its feet.

**Acceptance Scenarios**:

1. **Given** the game has just started, **When** the owner looks at the screen, **Then** the
   overlay, the lane lines and the origin marker are not shown.
2. **Given** the overlay is hidden, **When** the owner presses F1, **Then** the overlay appears in a
   corner, translucent, over the game; **When** the owner presses F1 again, **Then** it disappears.
3. **Given** the overlay is open, **When** the owner walks the hero, **Then** the shown position
   changes every frame the hero moves and stays the same while the hero stands still, and the FPS
   value updates while playing.
4. **Given** the overlay is open, **When** the owner holds the hero against the top and then the
   bottom of the lane, **Then** the hero's feet stop exactly on the top line and the bottom line.
5. **Given** the overlay is open, **When** the owner walks the hero anywhere, including behind a
   pillar, **Then** the origin marker stays at the hero's feet and stays visible.
6. **Given** the overlay is open, **When** the owner reads it, **Then** it shows player index 1 and
   the horizontal and vertical speeds the hero is using now.

---

### User Story 2 - Retune without restarting (Priority: P1)

The owner keeps the game running, opens the tuning data file, changes the horizontal speed (or the
vertical speed, or the lane's top or bottom), saves, switches back to the game and presses F5. The
hero moves at the new speed on the next frame, and the lane band, lane lines and limits move to the
new values. A short "Tuning reloaded" message confirms it. If the file has a bad value, nothing
changes in the game: the hero keeps moving with the previous values, and a message names the value
that was rejected and why.

**Why this priority**: The owner tunes by feel, and restarting after every change breaks the loop.
This is the main reason for the milestone.

**Independent Test**: With the game running, double `hero.horizontal_speed` in the tuning file,
save, and press F5: the hero crosses the screen in about half the time. Then set `lane.top` above
`lane.bottom`, save, and press F5: an error names `lane.top` and the problem, and the hero still
walks with the previous limits.

**Acceptance Scenarios**:

1. **Given** the game is running, **When** the owner changes `hero.horizontal_speed`, saves and
   presses F5, **Then** the hero moves at the new speed from the next frame, with no restart.
2. **Given** the game is running, **When** the owner changes `hero.vertical_speed`, `lane.top` or
   `lane.bottom`, saves and presses F5, **Then** the new value takes effect from the next frame:
   the speed changes, or the lane band, the lane lines and the hero's limits all move to the new
   lane.
3. **Given** a reload succeeded, **When** the owner looks at the screen, **Then** "Tuning reloaded"
   is shown for about 2 seconds and then disappears.
4. **Given** the game is running with valid values, **When** the owner sets `lane.top` greater than
   `lane.bottom`, saves and presses F5, **Then** a message names `lane.top` and why it was rejected,
   and the hero keeps moving with the previous limits, speeds and band.
5. **Given** a reload was rejected, **When** the owner fixes the file and presses F5 again,
   **Then** the new values are applied, the error goes away and "Tuning reloaded" is shown.
6. **Given** the overlay is hidden, **When** the owner presses F5, **Then** the reload still
   happens and its result ("Tuning reloaded" or the error) is still shown.

---

### User Story 3 - Debug tools stay out of the released game (Priority: P2)

The owner exports a release build and plays it. Pressing F1 or F5 does nothing: there is no
overlay, no lane lines, no origin marker and no reload.

**Why this priority**: Debug tools are for the owner, not for players. It is P2 because the owner
mostly plays from the editor, but the milestone is not done until this holds.

**Independent Test**: Export a release build, run it, and press F1 and F5. Nothing debug-related
appears or changes.

**Acceptance Scenarios**:

1. **Given** a release (exported) build is running, **When** the owner presses F1, **Then** no
   overlay, lane lines or origin marker appear.
2. **Given** a release build is running, **When** the owner edits the tuning file and presses F5,
   **Then** nothing is reloaded and nothing changes.

---

### User Story 4 - Still 60 fps, clean and committed (Priority: P2)

The owner plays on the target machine with V-Sync on and the overlay open, watching the FPS for one
minute while walking the hero, then runs the Quality Gate and commits.

**Why this priority**: Principle V and the Quality Gate apply to every milestone. The overlay draws
extra text and lines every frame, so the frame rate is checked with it open.

**Independent Test**: On the target machine, open the overlay and walk the hero continuously for
60 seconds by the clock with the window in front. Then run the Quality Gate and `git status`.

**Acceptance Scenarios**:

1. **Given** the game runs on the target machine with V-Sync on and the overlay open, **When** the
   owner walks the hero for one minute, **Then** the FPS stays at 60 the whole time (the first
   second excepted).
2. **Given** the M2a work is finished, **When** the owner runs `cargo fmt --check`, `cargo build`
   and `cargo clippy` and opens the project in the editor, **Then** there are no warnings and no
   GDScript errors or warnings.
3. **Given** the project has been built and played, **When** the owner runs `git status` after
   committing, **Then** the working tree is clean.

---

### Edge Cases

- **Part of the file is bad**: a reload is all or nothing. If any value is missing or invalid, no
  value from the file is applied, even the valid ones.
- **Several bad values**: the message names one rejected value and why (the first one found);
  fixing it and pressing F5 again reveals the next.
- **The file cannot be read or is not valid data** (deleted, half-saved, a syntax error): treated
  like a bad value; the message names the file and the problem, and the previous values stay.
- **The game started with bad tuning** (the hero is stopped, as in M1): the overlay shows the
  startup error. Fixing the file and pressing F5 applies the values and the hero can move, with no
  restart.
- **The game started with a non-tuning error** (a missing input action, a hero shape with no
  width): F5 does not bring the hero to life; the error stays and a restart is needed after fixing
  the scene or project settings.
- **The new lane leaves the hero outside it**: on the next frame the hero is placed at the nearest
  point inside the new lane, as M1 does for a hero that starts outside it.
- **F5 held down**: one reload per press; holding the key does not reload repeatedly.
- **F5 pressed while walking**: the hero keeps moving in the held direction, at the new speed from
  the next frame.
- **F1 toggled quickly**: each press flips the overlay once; it never ends up in a half-shown state.
- **Overlay and play**: the overlay never takes keyboard input from the hero and never pauses the
  game.
- **Pressing F5 with no changes**: the same values are applied again and "Tuning reloaded" is
  shown; nothing in play changes.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The game MUST have a debug overlay that is hidden when the game starts. Pressing F1
  MUST show it, and pressing F1 again MUST hide it.
- **FR-002**: The overlay MUST be a translucent panel in a corner of the screen, drawn over the
  game, and MUST NOT take input from the hero or pause the game.
- **FR-003**: The overlay MUST show the FPS, the hero's position, its player index, and its current
  horizontal and vertical speeds, and these values MUST be refreshed every frame while it is open.
- **FR-004**: While the overlay is open, the lane's top and bottom limits MUST be drawn as thin
  lines across the full screen width, at exactly the heights where the hero's feet stop.
- **FR-005**: While the overlay is open, a small marker MUST be drawn at the hero's origin (its
  feet), in front of everything else in the scene so it is visible even behind a pillar.
- **FR-006**: Pressing F5 MUST reread the tuning data file and check every value in it, whether
  the overlay is open or not. If every value is valid, the new values MUST all take effect from the
  next frame, with no restart: the hero's speeds, the hero's lane limits, the lane band, and
  the lane lines.
- **FR-007**: The result of each reload MUST appear in its own small message spot on screen,
  separate from the overlay panel and shown whether the overlay is open or not; the overlay also
  lists the last reload result while it is open. After a successful reload, the message "Tuning
  reloaded" MUST be shown for about 2 seconds.
- **FR-008**: If the reread file is unreadable or any value is missing or invalid, the game MUST
  keep every previous value unchanged and MUST show a message naming the rejected value (file and
  field) and why it was rejected. The same error MUST also be written to the output panel, as in
  M1. The message MUST stay until the next reload.
- **FR-009**: The checks applied on reload MUST be the same checks applied when the game starts
  (M1: both speeds finite and greater than 0, lane top at least 0 and below lane bottom, lane
  bottom inside the screen).
- **FR-010**: If the game started with invalid tuning, a successful reload MUST make the hero
  active with the new values, with no restart. A hero stopped for any other reason (a missing
  input action, a hero shape with no width) MUST stay stopped after a reload.
- **FR-011**: Reading and checking the tuning data MUST stay on the rules side (Rust). The overlay
  MUST be presentation only (GDScript): it displays values it reads from the rules side and MUST
  NOT compute or store game state.
- **FR-012**: The two debug keys MUST be named input actions, `debug_toggle_overlay` (F1) and
  `debug_reload_tuning` (F5), not checks for a specific key in code.
- **FR-013**: In a release (exported) build, the overlay, the lane lines, the origin marker, the
  reload and its message spot MUST NOT be available: F1 and F5 do nothing.
- **FR-014**: The always-on FPS counter from M0 and M1 MUST stay as it is, visible whether the
  overlay is open or not, and the overlay MUST NOT cover it. Everything M1 does
  (movement, limits, depth sorting, tuning at Play) MUST keep working unchanged.
- **FR-015**: Nothing listed as out of scope (hitboxes, enemies, health, magic, the hit dummy,
  attacking, any cheat that acts on something not yet in the game, editing values from inside the
  overlay) MAY be added in M2a.

### Key Entities

- **Debug overlay**: a panel the owner shows and hides. Shows FPS, hero position, player index,
  horizontal and vertical speed, and the result of the last reload. Holds no game state; it reads
  everything it shows from the rules side.
- **Lane limit lines**: two thin lines at the lane's top and bottom, shown with the overlay.
- **Origin marker**: a small mark at the hero's feet, shown with the overlay.
- **Tuning reload**: a request to reread the tuning data file. Its result is either "applied" (all
  new values in use) or "rejected" (previous values kept, with the rejected field and the reason).
- **Debug actions**: the two named input actions, `debug_toggle_overlay` and `debug_reload_tuning`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Over 10 presses of F1, the overlay shows on every odd press and hides on every even
  press, and it is hidden on every fresh start.
- **SC-002**: With the overlay open, the shown position matches the hero's true position on every
  frame it is checked, and changes on every frame the hero moves.
- **SC-003**: Holding the hero against the top and the bottom of the lane, its feet are on the
  limit line to the pixel, and the origin marker is at its feet in every position tried.
- **SC-004**: After changing `hero.horizontal_speed` to 2×, saving and pressing F5, the hero
  crosses the screen in about half the previous time, with no restart. Each of the four tuning
  values can be changed this way.
- **SC-005**: For each bad value tried (lane top above lane bottom, a speed of 0, a negative
  speed, text instead of a number, a missing field, a broken file), F5 shows a message naming that
  value and the problem, and the hero's speeds and limits stay exactly as before.
- **SC-006**: In a release build, F1 and F5 have no visible effect.
- **SC-007**: On the target machine, with the overlay open, the FPS stays at 60 for 60 seconds
  straight (after the first second) while the hero is walked continuously.
- **SC-008**: `cargo fmt --check`, `cargo build` and `cargo clippy` produce 0 warnings, the editor
  shows 0 GDScript errors or warnings, and `git status` is clean after committing.
- **SC-009**: After a tuning session of a few minutes, the owner can say "I can retune without
  restarting and see why the hero stops where it does".

## Assumptions

- **Target hardware is the Intel UHD 630.** The owner's acceptance check 9 says UHD 620, but
  constitution v1.3.2 corrected the target to the UHD 630, the GPU in the owner's machine, so this
  spec uses the 630, as M1 did.
- **Field names**: the owner's checks say `lane_top` and `lane_bottom`; in the tuning file since
  M1 these are `lane.top` and `lane.bottom`. Messages name the field by its path in the file.
- **Message timing**: an error stays in the message spot until the next reload; "Tuning
  reloaded" disappears after about 2 seconds.
- **All-or-nothing reload**: one bad value rejects the whole file, so the game never runs with a
  mix of old and new values.
- **Number format**: position and speeds are shown in pixels and pixels per second, rounded so
  they are easy to read (for example whole pixels); exact formatting is left to the plan.
- **Corner and look**: the overlay sits in a corner that does not cover the FPS counter; colours,
  size and font are placeholder presentation choices left to the plan.
- **"Release build"** means a build exported without debug features. Running from the editor, and
  a debug export, are debug builds where the overlay is available.
- **Only one hero exists** in M2a, so the overlay shows that hero's values. Showing several heroes
  waits for co-op (M7).
- **The Project Brief** ([docs/project-brief.md](../../docs/project-brief.md)) splits M2
  into **M2a Debug overlay** (this spec) and M2b Hit and feel. Its milestone table matches the
  owner's description: F1 overlay with FPS, hero position, speeds, lane-limit lines and origin
  marker; F5 live reload; bad values rejected with a clear message; absent from release builds.
  The brief schedules the cheats with what they act on: show hitboxes and spawn an enemy in M2b,
  refill health in M3, refill magic in M4.
