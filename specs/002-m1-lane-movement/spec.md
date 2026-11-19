# Feature Specification: M1 Lane Movement

**Feature Branch**: `feat/m1-lane-movement`

**Created**: 2026-11-19

**Status**: Draft

**Input**: User description: "m1-lane-movement: The player can walk the hero around a Golden Axe
style lane and it feels responsive." (full description, acceptance checks and out-of-scope list as
given by the owner in the `/speckit-specify` request)

> **Note on technology names.** Two requirements below name parts of the stack: movement rules
> live on the rules side (Rust), and input is read through actions named per player. Both are
> constraints from the constitution (the Rust/GDScript split, and Principle VII, Co-op Ready), not
> choices made by this spec. Every *how* (node types, class names, data layout, sorting method) is
> left to the plan. "Golden Axe style" describes the genre's movement model only; no character,
> asset or look from that game is used (Principle VI).

## Clarifications

### Session 2026-11-19

- Q: Should the pillars be solid, so the hero has to walk around them, or should the hero be able
  to walk through a pillar's base? → A: Not solid. The hero can walk through a pillar's base; the
  pillars only test depth sorting. Collision waits for a milestone that needs it.
- Q: Should the pillar positions and the hero's starting spot be set in the scene in the Godot
  editor, or in the data file next to the tuning values? → A: In the scene. Layout is level design
  placed in the editor; the data file holds only values tuned by trying numbers (speeds, lane
  limits).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Walk the hero around the lane (Priority: P1)

The owner presses Play. A 1280x720 window opens with a dark background and a lighter horizontal
band across the lower part of the screen: the lane. A tall coloured rectangle, the hero, stands in
the lane. Using the arrow keys or WASD, the owner walks the hero left and right along the lane and
up and down to step deeper into it or nearer to the screen. The hero starts and stops the moment a
key is pressed or released, moving diagonally is no faster than moving straight, stepping up or
down is slower than walking sideways, and the hero cannot leave the lane or the screen.

**Why this priority**: Movement is the base of every later milestone. If walking feels slow to
respond or slippery now, hitting and dodging enemies will feel the same.

**Independent Test**: Press Play and walk the hero for one minute: tap each direction, hold each
diagonal, and push into all four limits. The hero responds at once, stops at once, never speeds up
on diagonals, moves visibly slower up and down, and never leaves the lane or the screen.

**Acceptance Scenarios**:

1. **Given** the hero is standing still, **When** the owner presses a direction key, **Then** the
   hero starts moving on that same frame, at full speed, with no build-up.
2. **Given** the hero is moving, **When** the owner releases all direction keys, **Then** the hero
   stops on that same frame, with no slide.
3. **Given** the hero is in open space in the lane, **When** the owner holds a diagonal (for
   example right and up), **Then** the hero covers no more distance per second than when holding
   right alone.
4. **Given** the hero is in open space in the lane, **When** the owner holds up (or down) and then
   left (or right), each for the same time, **Then** the vertical distance covered is visibly
   shorter than the horizontal distance.
5. **Given** the hero is at any limit (lane top, lane bottom, left screen edge, right screen
   edge), **When** the owner keeps holding the direction into that limit, **Then** the hero stays
   exactly at the limit, fully inside, with no shaking.
6. **Given** the hero is pressed into a limit, **When** the owner adds a second direction along
   the limit (for example holding up against the lane top while also holding right), **Then** the
   hero slides along the limit at its normal speed for that direction.

---

### User Story 2 - Walk in front of and behind pillars (Priority: P1)

Two or three tall placeholder pillars stand in the lane, with their bases at different depths. As
the owner walks the hero around each pillar, the hero is hidden behind the pillar when the hero's
feet are higher on the screen than the pillar's base, and drawn over the pillar when its feet are
lower.

**Why this priority**: Overlap by depth is what makes the lane read as a floor rather than a flat
picture. Every character added later (enemies, a second player) depends on the same rule, so it
must be right now.

**Independent Test**: Walk the hero up and down past the base of each pillar several times while
overlapping it. The drawing order switches exactly when the hero's feet cross the pillar's base,
and never flickers.

**Acceptance Scenarios**:

1. **Given** the hero overlaps a pillar and the hero's feet are higher on the screen than the
   pillar's base, **When** the owner looks at the overlap, **Then** the pillar covers the hero.
2. **Given** the hero overlaps a pillar and the hero's feet are lower on the screen than the
   pillar's base, **When** the owner looks at the overlap, **Then** the hero covers the pillar.
3. **Given** the hero is walking up or down across a pillar's base line while overlapping it,
   **When** the feet cross the line, **Then** the drawing order switches once, with no flicker,
   including when the owner stops exactly on the line.
4. **Given** the pillars stand at different depths, **When** the owner walks between them,
   **Then** each pillar's overlap with the hero follows its own base, independently of the others.

---

### User Story 3 - Tune movement and the lane without recompiling (Priority: P2)

The owner closes the game, changes the horizontal speed, the vertical speed or the lane's top and
bottom in the tuning data file, and presses Play again. The game reflects the new values. No Rust
rebuild is needed.

**Why this priority**: Movement feel is found by trying numbers, not by reading code (Principle
IV). It is P2 because the hero can be walked with the starting values, but M1 is not done until
this passes.

**Independent Test**: Double the horizontal speed, halve the vertical speed, and move the lane top
down by 50 pixels in the tuning file. Press Play without building Rust. The hero walks sideways
twice as fast, steps up and down half as fast, and both the visible band and the hero's upper
limit have moved down.

**Acceptance Scenarios**:

1. **Given** the horizontal speed in the tuning file is H, **When** the owner changes it to 2×H and
   presses Play without rebuilding Rust, **Then** the hero crosses the screen in about half the
   previous time.
2. **Given** the vertical speed is V, **When** the owner changes it to V/2 and presses Play,
   **Then** walking from the lane bottom to the lane top takes about twice as long.
3. **Given** the lane top and bottom are T and B, **When** the owner changes them and presses Play,
   **Then** the lighter band on screen and the hero's up and down limits both match the new
   values.

---

### User Story 4 - Still 60 fps, clean and committed (Priority: P2)

The owner plays on the target machine with V-Sync on, watching the FPS counter (kept from M0) while
walking the hero for one minute, then runs the Quality Gate and commits.

**Why this priority**: Principle V and the Quality Gate apply to every milestone. M1 adds input and
depth sorting, so the frame rate is checked again.

**Independent Test**: On the target machine, walk the hero continuously for 60 seconds by the
clock with the window in front, watching the FPS counter. Then run the Quality Gate and
`git status`.

**Acceptance Scenarios**:

1. **Given** the game runs on the target machine with V-Sync on, **When** the owner walks the hero
   for one minute, **Then** the FPS counter shows 60 the whole time (the first second excepted).
2. **Given** the M1 work is finished, **When** the owner runs `cargo fmt --check`, `cargo build`
   and `cargo clippy` and opens the project in the editor, **Then** there are no warnings and no
   GDScript errors or warnings.
3. **Given** the project has been built and played, **When** the owner runs `git status` after
   committing, **Then** the working tree is clean.

---

### Edge Cases

- **Opposite keys held together** (left and right, or up and down): the hero does not move along
  that axis. If a key on the other axis is also held, the hero moves along that axis only, at its
  straight-line speed.
- **The same direction on both key sets** (for example the right arrow and D): counts as one
  press; the hero does not move faster.
- **Window loses focus while a key is held**: the hero stops and does not keep walking when the
  window regains focus with no key held.
- **Hero starts outside the lane** (for example after the lane is retuned): on the first frame the
  hero is placed at the nearest point inside the lane and the screen, and stays there until moved.
- **Very high speed in the tuning file** (the hero would cross the lane in one or two frames): the
  hero still never ends a frame outside the lane or the screen.
- **Speed of 0**: invalid, like a negative speed. Both speeds must be greater than 0 (owner's
  plan input for M1); a 0 is reported as a bad value and the hero stays still.
- **Invalid tuning data** (a value missing, not a number, or negative; lane top not above lane
  bottom; lane outside the screen): the game reports a clear error in the output panel naming the
  file, field and problem, as in M0, and the hero stays still. The game does not crash and does not
  use values hidden in code.
- **Pillar base outside the lane after retuning**: the pillar is scenery and stays where it is; its
  drawing order against the hero still follows its base.
- **Hero's feet exactly on a pillar's base line**: one fixed rule decides who is in front (see
  Assumptions), so the result never alternates between frames.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Pressing Play MUST open a 1280x720 window with a dark background and a lighter
  horizontal band (the lane) across the lower part of the screen.
- **FR-002**: The scene MUST contain one hero, a coloured rectangle taller than it is wide, standing
  in the lane, and two or three tall placeholder pillars whose bases stand in the lane at different
  depths. The pillars MUST NOT block the hero: it can walk through a pillar's base. Where the
  pillars stand and where the hero starts MUST be set in the scene in the editor, not in the
  tuning data file; the hero starts inside the lane.
- **FR-003**: The player MUST be able to move the hero left, right, up (deeper into the lane) and
  down (nearer) with the arrow keys and with W, A, S and D, both sets working at the same time.
- **FR-004**: A direction press MUST move the hero at full speed on the frame the press is
  detected, and releasing all directions MUST stop it on the frame the release is detected. There
  is no acceleration, deceleration or slide.
- **FR-005**: The hero's horizontal and vertical speeds MUST be separate values, and the vertical
  speed's starting value MUST be lower than the horizontal one.
- **FR-006**: Moving diagonally MUST NOT cover more distance per second than moving horizontally
  alone.
- **FR-007**: The hero's feet MUST always stay between the lane's top and bottom, and the whole
  hero MUST always stay within the left and right screen edges. Pushing into a limit MUST leave the
  hero exactly at it, with no movement back and forth between frames.
- **FR-008**: The visible lane band MUST match the lane's top and bottom limits, so the hero's feet
  can reach every part of the band and nothing outside it.
- **FR-009**: The hero and each pillar MUST be drawn in order of depth: whichever has its base
  (feet) lower on the screen is drawn in front. The order MUST be the same on every frame for the
  same positions, so it never flickers.
- **FR-010**: The hero's position, movement and lane limits MUST be owned and updated by the rules
  side (Rust). The presentation side MUST NOT calculate or store them.
- **FR-011**: Input MUST be read through actions named for player 1 (for example `p1_left`,
  `p1_right`, `p1_up`, `p1_down`), and the hero MUST know which player index controls it, so a
  second player can be added later without rewriting the hero (Principle VII).
- **FR-012**: The horizontal speed, the vertical speed, and the lane's top and bottom MUST be read
  from the tuning data file at the start of each Play. Changing them MUST take effect on the next
  Play without rebuilding Rust, and none of them may appear as a literal value in code (Principle
  IV).
- **FR-013**: If any of those tuning values is missing or invalid, the game MUST report a clear
  error naming the file, field and problem, keep the hero still, and not crash.
- **FR-014**: The FPS counter from M0 MUST remain visible. The M0 drifting square, its speed label
  and its tuning value MUST be removed.
- **FR-015**: The game MUST still use the Compatibility renderer with V-Sync on.
- **FR-016**: Nothing listed as out of scope (attacking, jumping, enemies, a scrolling camera,
  gamepad input, animation and art, sound, the debug overlay) MAY be added in M1.

### Key Entities

- **Hero**: the one character the player controls. Has a position (measured at its feet), a size
  (taller than wide), a colour, and the index of the player who controls it (1 in M1). Its
  position belongs to the rules side.
- **Lane**: the strip of ground the hero walks on. Has a top and a bottom (the range the feet may
  occupy); its left and right limits are the screen edges. Its top and bottom are tuning values.
- **Pillar**: a tall placeholder object standing in the lane. Has a base position that sets its
  depth. It does not move and does not block the hero (FR-002).
- **Movement tuning**: the tuning values for M1: horizontal speed, vertical speed, lane top and
  lane bottom. They live in the project-wide tuning data file chosen in M0.
- **Player 1 controls**: the four direction actions for player 1, each bound to an arrow key and a
  WASD key.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a frame-by-frame check, the hero's position changes on the first frame after a
  direction press and stops changing on the first frame after release, every time, over 20
  presses.
- **SC-002**: Holding a diagonal for 2 seconds moves the hero no further than holding right alone
  for 2 seconds (within 1 %).
- **SC-003**: With the starting values, the vertical speed is at most two thirds of the horizontal
  speed, and the owner agrees, after walking for a minute, that the lane "reads as depth".
- **SC-004**: Holding into each of the four limits for 5 seconds, the hero's position does not
  change at all between frames, and no part of the hero leaves the screen at any time.
- **SC-005**: Crossing each pillar's base line 10 times while overlapping it, and stopping on it
  once, the drawing order switches exactly once per crossing and never flickers.
- **SC-006**: After changing each of the four movement tuning values and pressing Play without
  rebuilding Rust, the change is visible in the game in all four cases.
- **SC-007**: On the target machine, the FPS counter shows 60 for 60 seconds straight (after the
  first second) while the hero is walked continuously.
- **SC-008**: `cargo fmt --check`, `cargo build` and `cargo clippy` produce 0 warnings, the editor
  shows 0 GDScript errors or warnings, and `git status` is clean after committing.
- **SC-009**: After walking around for a few minutes, the owner can say "moving feels immediate
  and the depth looks right", with nothing about movement carried into M2.

## Assumptions

- **Starting values** (data, not requirements): horizontal speed about 300 px/s, vertical speed
  about 180 px/s (60 % of horizontal), lane top about y = 430 and lane bottom about y = 700 on the
  720-pixel-high screen. The hero is about 48x96 pixels; pillars are about 64 pixels wide and
  200-300 pixels tall.
- **Diagonal speed**: the direction is normalised before the per-axis speeds are applied, so a
  diagonal is never faster than a straight line. The plan picks the exact formula.
- **Feet, not head, are limited by the lane**: the hero's top may extend above the lane band, as a
  standing figure would.
- **Tie at equal depth**: when the hero's feet and a pillar's base are at exactly the same height,
  the hero is drawn in front. Any fixed rule meets FR-009; this one was chosen so the player is
  never hidden by a tie.
- **Target hardware is the Intel UHD 630.** The owner's acceptance check 8 says UHD 620, but
  constitution v1.3.2 corrected the target to the UHD 630, the GPU in the owner's machine, so this
  spec uses the 630.
- **Removing M0 content**: the drifting square, its speed label and its `drifting_square` tuning
  section are deleted; the tuning loader and its error reporting from M0 are reused for the new
  values.
- **Input set**: keyboard only (arrow keys and WASD) for player 1. Gamepads are out of scope.
- **The Project Brief**, which defines M1's must-have list, is not in the repository. This spec
  treats the owner's description as that list, as M0 did.
