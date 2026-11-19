# Feature Specification: M0 Setup – Toolchain Proof

**Feature Branch**: `feat/m0-setup`

**Created**: 2026-11-19

**Status**: Draft

**Input**: User description: "m0-setup: Prove the toolchain works end to end on the target machine
before any gameplay exists." (full description, acceptance checks and out-of-scope list as given by
the owner in the `/speckit-specify` request)

> **Note on technology names.** Specs normally avoid naming technology. M0 is the exception: its
> only purpose is to prove that the stack fixed by the constitution (Godot with the Compatibility
> renderer, Rust through gdext for rules and state, GDScript for presentation) works on the target
> machine. Those names appear here as constraints inherited from the constitution, not as choices
> made by this spec. Every *how* (versions, data format, file layout, class names) is left to the
> plan.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Press Play and see it run (Priority: P1)

The owner opens the Beastrider project in the Godot editor and presses Play. A 1280x720 window
opens with a dark background. One coloured square drifts slowly from left to right; when it
leaves the right edge it comes back in from the left and keeps going. A small label in a corner
shows the square's current speed. An FPS counter shows the frame rate.

**Why this priority**: This is the whole point of M0. If this works, the rules side (Rust) and the
presentation side (GDScript) are talking to each other inside Godot, and every later milestone can
build on it.

**Independent Test**: Open the project, press Play, watch for 30 seconds. The square drifts and
wraps at least once, the speed label shows a number, and the FPS counter is visible.

**Acceptance Scenarios**:

1. **Given** a fresh checkout that has been built once, **When** the owner opens the project and
   presses Play, **Then** a 1280x720 window opens with a dark background, one coloured square
   drifting left to right, a speed label, and an FPS counter.
2. **Given** the game is running, **When** the square reaches the right edge, **Then** it leaves
   the screen and reappears from the left edge, still moving at the same speed, with no pause.
3. **Given** the game is running, **When** the owner reads the speed label, **Then** it shows the
   same speed the square is actually moving at (the value the rules side reports).
4. **Given** the game is running, **When** the owner looks at the editor's output panel, **Then**
   it shows no errors, including none about loading the Rust library.

---

### User Story 2 - Holds 60 fps on the target machine (Priority: P1)

The owner runs the game on their own machine (Ubuntu 24.04 LTS, Intel UHD 630, stock Mesa driver,
V-Sync on) and watches the FPS counter for one minute.

**Why this priority**: Principle V makes 60 fps on this hardware a hard rule. Finding out now,
with nothing on screen but a square, that the setup cannot hold 60 fps is far cheaper than finding
out at M6.

**Independent Test**: On the target machine, press Play and watch the FPS counter for 60 seconds
by the clock.

**Acceptance Scenarios**:

1. **Given** the game is running on the target machine with V-Sync on, **When** the owner watches
   the FPS counter for one minute, **Then** it shows 60 for the whole minute (no reading below 60
   after the first second, which is allowed for startup).

---

### User Story 3 - Tune speed without recompiling (Priority: P2)

The owner closes the game, opens the tuning data file, changes the square's speed, saves it and
presses Play again. The square now moves at the new speed and the label shows the new value. No
Rust rebuild is needed.

**Why this priority**: This proves Principle IV (data-driven tuning) works end to end before there
are dozens of values to tune. It is P2 because the game is still useful as a toolchain proof without
it, but M0 is not done until it passes.

**Independent Test**: Note the speed shown on the label. Double the speed value in the tuning
file, press Play without running any Rust build, and check that the label shows the doubled value
and the square visibly crosses the screen in about half the time.

**Acceptance Scenarios**:

1. **Given** the speed in the tuning file is S, **When** the owner changes it to 2×S and presses
   Play again without rebuilding Rust, **Then** the label shows 2×S and the square crosses the
   screen in about half the previous time.
2. **Given** the speed in the tuning file is set to 0, **When** the owner presses Play, **Then**
   the square stays still and the label shows 0.

---

### User Story 4 - Clean, committed, ready for M1 (Priority: P2)

The owner runs the Quality Gate checks, checks the Godot editor for warnings, and commits
everything to git.

**Why this priority**: The constitution says a milestone is only done when the Quality Gate passes
and the work is committed. Setting this up while the project is tiny means it stays clean.

**Independent Test**: Run `cargo fmt --check`, `cargo build` and `cargo clippy`; open the project
in the editor; then run `git status` after committing.

**Acceptance Scenarios**:

1. **Given** the M0 work is finished, **When** the owner runs `cargo fmt --check`, `cargo build`
   and `cargo clippy`, **Then** all three pass with no warnings.
2. **Given** the project is open in the Godot editor, **When** the owner looks at the script
   errors and warnings, **Then** there are none.
3. **Given** the project has been built and run, **When** the owner runs `git status`, **Then** the
   working tree is clean: everything is committed, and Rust build output and Godot's cache folder
   do not appear as untracked files.

---

### Edge Cases

- **Tuning file missing or unreadable**: the game still opens, the output panel shows a clear error
  naming the file and the problem, and the square does not move. The game does not crash and does
  not silently use a speed hidden in code.
- **Tuning file has a bad speed value** (missing, not a number, or negative): treated the same as
  a missing file. The error names the field and the bad value.
- **Very high speed** (the square would cross the screen in a few frames): the square still wraps
  correctly and never gets stuck off-screen.
- **Rust library not built yet** (fresh checkout): pressing Play shows Godot's own error about the
  missing library. M0 does not need to handle this better, but the plan must document the one
  build command needed before the first Play.
- **Window resized or moved by the owner**: out of scope. The 1280x720 size is the only one
  checked.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Pressing Play in the Godot editor MUST open a 1280x720 window with a dark background.
- **FR-002**: The window MUST show exactly one coloured square that moves left to right at a
  constant speed.
- **FR-003**: When the square fully leaves the right edge, it MUST reappear entering from the left
  edge and continue at the same speed, with no pause or jump in speed.
- **FR-004**: The square's position and speed MUST be owned and updated by the rules side (Rust).
  The presentation side MUST NOT calculate or store the square's position or speed.
- **FR-005**: A label in a corner of the window MUST show the square's current speed, read from
  the rules side every time it is displayed. The label MUST NOT keep its own copy of the value.
- **FR-006**: An FPS counter MUST be visible in the window while the game runs.
- **FR-007**: The square's speed MUST be read from a tuning data file when the game starts.
  Changing the value and pressing Play again MUST change the speed without rebuilding Rust.
- **FR-008**: The game code (Rust and GDScript) MUST NOT contain the square's speed as a literal
  value (Principle IV).
- **FR-009**: If the tuning file is missing or the speed value is invalid, the game MUST report a
  clear error in the output panel naming the file, field and problem, and the square MUST stay
  still. The game MUST NOT crash.
- **FR-010**: The project MUST use the Compatibility renderer and MUST run with V-Sync on.
- **FR-011**: Godot MUST load the Rust library with no errors in the output panel.
- **FR-012**: The repository MUST ignore Rust build output and Godot's cache folder, so that
  building and running the project leaves `git status` clean.
- **FR-013**: The plan MUST document the steps needed to go from a fresh checkout to pressing
  Play (for example, the one build command to run first).
- **FR-014**: Nothing listed as out of scope (player input, the lane, enemies, art, sound, the debug
  overlay) MAY be added in M0.

### Key Entities

- **Drifting square**: the one moving object. Has a position (horizontal only for M0), a speed,
  and a colour. Its position and speed belong to the rules side.
- **Tuning data**: the file holding tunable values. For M0 it holds one value: the square's speed,
  in pixels per second. The format is chosen in the M0 plan and then used for all later tuning
  data.
- **Speed label**: a read-only view of the square's speed. It holds no state.
- **FPS counter**: a read-only view of the engine's frame rate.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: From a built project, the owner goes from pressing Play to seeing the square, label
  and FPS counter in under 5 seconds.
- **SC-002**: On the target machine, the FPS counter shows 60 for 60 seconds straight (after the
  first second).
- **SC-003**: After doubling the speed in the tuning file and pressing Play again, the square
  crosses the screen in half the time (±10%), with no Rust rebuild.
- **SC-004**: The editor's output panel shows 0 errors on load and during 60 seconds of play.
- **SC-005**: `cargo fmt --check`, `cargo build` and `cargo clippy` produce 0 warnings, and the
  editor shows 0 GDScript errors or warnings.
- **SC-006**: After building and running, `git status` shows a clean working tree.
- **SC-007**: The owner can say, after M0, "the toolchain works on my machine and I can start on
  gameplay", with no open setup problems carried into M1.

## Assumptions

- "Slowly" means the default speed in the tuning file lets the square cross the 1280-pixel window
  in about 8 seconds (about 160 pixels per second). This is a starting value in the data file, not
  a requirement.
- The square is a plain filled square about 64x64 pixels in a bright, saturated colour against a
  near-black background, which is placeholder art under Principle VI.
- "Wrapping around" means the square fully leaves the right edge before reappearing from the left,
  so it never appears on both edges at once.
- The tuning file is read once, when the game starts. Changing it while the game runs has no
  effect until Play is pressed again; live reloading is not part of M0.
- A speed of 0 is valid (the square stays still). Negative speeds are invalid for M0 and handled as
  a bad value (see Edge Cases).
- The speed label shows a whole number of pixels per second, for example "Speed: 160 px/s".
- The FPS counter may be a simple label showing the engine's frame rate. The debug overlay from
  Principle VIII is a separate feature that starts at M2.
- The FPS check is done by watching the counter. No automated performance logging is required.
- The Project Brief, which defines M0's must-have list, is not in the repository. This spec treats
  the owner's description as that must-have list.
- The owner's machine is the target machine, so every acceptance check runs on it.
- Pinning the Godot version, the gdext version, the Rust toolchain version and the tuning-data
  format are all M0 plan decisions, as the constitution requires.
