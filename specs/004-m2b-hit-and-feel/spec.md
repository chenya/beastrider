# Feature Specification: M2b Hit and Feel

**Feature Branch**: `feat/m2b-hit-and-feel`

**Created**: 2026-11-20

**Status**: Draft

**Input**: User description: "m2b-hit-and-feel: The hero can strike a training dummy, and every hit
feels like it connected." (full description, acceptance checks and out-of-scope list as given by
the owner in the `/speckit-specify` request)

> **Note on technology names.** A few requirements below name parts of the stack: attack timing,
> hit detection, the depth rule and knockback live on the rules side (Rust); flash and shake are
> presentation (GDScript) reacting to a signal from the rules side; the hero's modes are a closed
> set checked exhaustively; and the attack key is an input action named for player 1. These are
> constraints from the constitution (the Rust/GDScript split, "types over strings", Principle VII)
> and from the owner's acceptance checks 10 and 11, not choices made by this spec. Every *how*
> (node types, class names, data layout, how the freeze is done) is left to the plan.

## Clarifications

### Session 2026-11-20

- Q: How should the length of the dummy's knockback slide be set: by its own tuning value, or
  worked out from the knockback distance alone? → A: Its own value, **knockback time**: the dummy
  covers the knockback distance in that time, slowing evenly to a stop. Setting either knockback
  distance or knockback time to 0 turns knockback off.
- Q: When a blow lands, should the screen shake start right away, during the freeze, or only after
  the freeze ends? → A: Flash and shake start at impact and run through the freeze; the knockback
  slide starts when the freeze ends.
- Q: If attack is pressed during a swing's recovery, should the press be remembered so the next
  swing starts as soon as the hero is free, or ignored? → A: Remember one press made during
  recovery; the next swing starts on the first frame the hero is free. Presses during wind-up, the
  active part or a freeze are still ignored.
- Q: Where should F3 put a new dummy? → A: In front of the hero at its depth, starting just inside
  reach; if a dummy already stands there, the new one steps half a dummy's width further along the
  hero's facing, repeating until the spot is free, and is moved inside the lane and
  screen if needed.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Swing at the dummy and land a hit (Priority: P1)

The owner presses Play. A training dummy, a tall placeholder shape in a different colour from the
hero, stands in the lane. The owner walks the hero next to it and presses the attack key (J or Z).
The hero swings: a short wind-up, a brief moment when the blow can land, then a short recovery.
The hero cannot walk while swinging. If the dummy is in front of the hero, within reach and at
about the same depth in the lane, the blow lands: the game freezes for a split second, the dummy
flashes white, the screen gives a small shake, and the dummy slides back away from the hero,
slowing to a stop. The dummy never dies, so the owner can hit it again and again.

**Why this priority**: Hitting is the heart of the game. Every later milestone (enemies, waves,
bosses) depends on a blow that lands reliably and feels like it connected.

**Independent Test**: Walk up to the dummy, face it, and swing ten times from different spots:
every swing that reaches it at the same depth lands with all four effects, and every swing that
does not reach, faces away or is at a clearly different depth misses with no effects.

**Acceptance Scenarios**:

1. **Given** the hero faces the dummy, within reach and at the same depth, **When** the owner
   presses the attack key, **Then** after the wind-up the blow lands, and the owner sees a short
   freeze, a white flash on the dummy, a small screen shake, and the dummy sliding away from the
   hero and slowing to a stop.
2. **Given** the dummy is within reach horizontally but clearly deeper or nearer in the lane than
   the hero, **When** the owner swings, **Then** nothing happens to the dummy: no freeze, flash,
   shake or knockback.
3. **Given** the dummy is behind the hero, or beyond reach, **When** the owner swings, **Then** the
   blow misses with no effects.
4. **Given** the hero is swinging, **When** the owner holds a direction key, **Then** the hero does
   not move until the swing has finished; it then moves normally if the key is still held.
5. **Given** the hero last walked left (or right), **When** the owner stops and swings, **Then** the
   swing goes to the left (or right).
6. **Given** a swing reaches the dummy for several frames, **When** the swing ends, **Then** the
   dummy was hit exactly once.
7. **Given** two dummies are both within reach and at the hero's depth, **When** the owner swings
   once, **Then** both are hit, each once, with one shared freeze.
8. **Given** the dummy stands near a screen edge or the lane's top or bottom, **When** it is hit
   towards that edge, **Then** it stops at the edge and never leaves the lane or the screen.

---

### User Story 2 - Tune the impact live, one effect at a time (Priority: P1)

The owner keeps the game running, changes the freeze length, flash length, shake strength or
length, knockback distance or time, the swing's wind-up, active and recovery times, its reach, or
the depth tolerance in the tuning data file, saves, and presses F5. The next swing uses the new
values. Setting any effect's value to zero turns that effect off, so the owner can judge each
effect on its own and then together.

**Why this priority**: The owner tunes by feel. Weight comes from the balance of the four effects
and the swing's timing, which can only be found by trying numbers quickly (Principle IV, M2a's live
reload).

**Independent Test**: With the game running, set the freeze to 0 and press F5: hits land with no
freeze but with flash, shake and knockback. Restore it and repeat for each effect. Double the reach
and the knockback distance and press F5: the hero hits from further away and the dummy slides about
twice as far.

**Acceptance Scenarios**:

1. **Given** the game is running, **When** the owner changes any of the attack or impact values,
   saves and presses F5, **Then** the next swing uses the new value, with no restart.
2. **Given** an effect's value is set to 0 and reloaded, **When** a blow lands, **Then** that effect
   does not happen and the other effects still do.
3. **Given** a bad attack or impact value (missing, not a number, negative, or zero where zero is
   not allowed), **When** the owner presses F5, **Then** the reload is rejected with a message
   naming the value and why, as in M2a, and every previous value stays in use.

---

### User Story 3 - See the hitboxes, spawn dummies, count hits (Priority: P2)

With the debug overlay available, the owner presses F2 to show the hitboxes: the swing's reach and
each dummy's body. F2 again hides them. F3 spawns another dummy in the lane. The overlay panel
counts the hits landed.

**Why this priority**: Seeing the reach and the bodies explains why a swing missed, and extra
dummies test multi-hits and the frame rate. Principle VIII requires these two cheats in M2b
(constitution v1.3.3). It is P2 because the hits themselves (US1) come first.

**Independent Test**: Press F2, swing at the dummy from several spots and check the drawn reach
overlaps the drawn body exactly when the blow lands. Press F3 four times and swing through two
dummies at once; the hit counter goes up by two.

**Acceptance Scenarios**:

1. **Given** the hitboxes are hidden, **When** the owner presses F2, **Then** each dummy's body is
   outlined and, while a swing can land, its reach is drawn in front of the hero; **When** F2 is
   pressed again, **Then** they are hidden.
2. **Given** the game is running, **When** the owner presses F3, **Then** a new dummy appears in
   the lane, inside the screen, and can be hit like the first.
3. **Given** the overlay panel is open, **When** blows land, **Then** the hit counter goes up by
   exactly one per dummy hit, and by two when one swing hits two dummies.
4. **Given** a release (exported) build, **When** the owner presses F2 or F3, **Then** nothing
   happens, as with F1 and F5 in M2a.

---

### User Story 4 - It feels weighty (Priority: P1)

The owner plays with the tuned values, lands 30 hits on the dummy, and judges whether the impact
feels weighty rather than floaty. If it does not, the owner retunes (US2) and plays again.

**Why this priority**: This is the milestone's real goal (Principle I). The milestone is not done
until the owner says the hits feel right, whatever the other checks say.

**Independent Test**: A playtest of 30 hits with the final tuning values, ending in the owner's
yes or no.

**Acceptance Scenarios**:

1. **Given** the final tuning values, **When** the owner lands 30 hits, **Then** the owner judges
   that the impact feels weighty, not floaty. If not, the values are retuned and the playtest is
   repeated before the milestone can end.

---

### User Story 5 - Still 60 fps, clean and committed (Priority: P2)

The owner spawns dummies until there are five, turns on the hitboxes, and plays for a minute on
the target machine with V-Sync on, then runs the Quality Gate and commits.

**Why this priority**: Principle V and the Quality Gate apply to every milestone. More dummies,
the freeze, the shake and the hitbox drawing all add work each frame.

**Independent Test**: With five dummies and hitboxes shown, swing and walk for 60 seconds by the
clock, watching the FPS. Then run the Quality Gate and `git status`.

**Acceptance Scenarios**:

1. **Given** five dummies with hitboxes shown on the target machine, **When** the owner walks and
   swings for one minute, **Then** the FPS stays at 60, apart from the intended freezes on impact
   and the first second.
2. **Given** the M2b work is finished, **When** the owner runs `cargo fmt --check`, `cargo build`
   and `cargo clippy` and opens the project in the editor, **Then** there are no warnings and no
   GDScript errors or warnings.
3. **Given** the project has been built and played, **When** the owner runs `git status` after
   committing, **Then** the working tree is clean.

---

### Edge Cases

- **Attack pressed during wind-up or the active part**: ignored; there are no combos.
- **Attack pressed during recovery**: remembered (one press only); a new, identical swing starts on
  the first frame the hero is free. Several presses during recovery still give one swing.
- **Attack pressed during the impact freeze**: ignored, like during wind-up and the active part.
- **Attack held down**: one swing per press; holding the key does not swing repeatedly.
- **Attack with a direction held**: the swing faces the way the hero last moved before the swing;
  the hero does not turn or move until the swing ends.
- **The hero has not moved yet**: it faces right at the start.
- **Dummy hit while still sliding from an earlier hit**: the new knockback starts from where the
  dummy is, away from the hero again; the earlier slide is replaced, not added to.
- **Hero and dummy overlapping, or the dummy at the exact edge of reach**: whatever the drawn reach
  and body show decides it; touching edges count as a hit, so F2 always explains the result.
- **Depth exactly at the tolerance**: counts as the same depth (a hit); just beyond it is a miss.
- **Knockback towards a screen edge or lane limit**: the dummy stops at the limit, fully inside.
- **Several dummies hit by one swing**: one freeze for the swing, not one per dummy; each dummy
  flashes, slides and counts once.
- **Effect set to 0**: that effect is skipped entirely (a zero freeze is no pause at all, not a
  one-frame pause).
- **F5 during a swing or a freeze**: the new values apply from the next swing; the current swing
  finishes with the values it started with.
- **The hero walks into a dummy**: dummies do not block the hero, as pillars do not (M1); hero,
  dummies and pillars overlap by depth.
- **F3 pressed many times**: each press adds one dummy; there is no hard limit, but the 60 fps check
  is made with five.
- **F2 with the overlay panel closed**: works on its own, like F5; the hitboxes do not need the
  panel open.
- **Bad startup tuning** (as M2a): the hero cannot swing or move until a reload succeeds; the dummy
  stands still.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The scene MUST start with one training dummy standing in the lane: a tall placeholder
  shape in a colour different from the hero's, drawn in depth order with the hero and pillars.
  Where it stands is set in the scene, like the pillars (M1 clarification Q2).
- **FR-002**: The hero MUST face the direction it last moved horizontally, and face right before it
  has moved.
- **FR-003**: Pressing the attack action MUST start a swing made of three timed parts: wind-up,
  active and recovery. A blow can land only during the active part. One attack press made during
  recovery MUST be remembered and start the next swing on the first frame the hero is free; presses
  during wind-up, the active part or a freeze MUST be ignored. Holding attack MUST NOT repeat the
  swing.
- **FR-004**: The hero MUST NOT move or turn during a swing, whatever keys are held.
- **FR-005**: During the active part, a dummy MUST be hit when its body overlaps the swing's reach
  in front of the hero and the difference between the hero's depth and the dummy's depth (the
  height of their feet on screen) is at most the depth tolerance. Otherwise it MUST NOT be hit.
- **FR-006**: One swing MUST hit each dummy at most once, and MUST hit every dummy that meets
  FR-005 during its active part.
- **FR-007**: When a swing lands on one or more dummies, the game MUST: freeze play for the freeze
  length (once per swing, not per dummy); flash each hit dummy white for the flash length; shake
  the play area by the shake strength for the shake length; and push each hit dummy horizontally
  away from the hero, covering the knockback distance in the knockback time and slowing evenly to
  a stop. The flash and the shake MUST start at the moment of impact and keep running through the
  freeze; the knockback slide MUST start when the freeze ends.
- **FR-008**: Knockback MUST NOT move a dummy outside the lane or off the screen; it stops at the
  limit.
- **FR-009**: The freeze length, flash length, shake strength, shake length, knockback distance,
  knockback time, wind-up time, active time, recovery time, reach and depth tolerance MUST be read
  from the tuning data file, checked like the M1 values, and applied by F5 with the M2a reload
  rules (all or nothing, the next swing uses them). None of them may appear as a literal value in
  code.
- **FR-010**: A value of 0 for the freeze length, flash length, shake strength, shake length,
  knockback distance or knockback time MUST turn that effect off without affecting the others. The
  active time and the reach MUST be greater than 0; wind-up, recovery and depth tolerance MAY be 0.
- **FR-011**: Attack timing, hit detection, the depth rule, the facing rule and knockback MUST be
  owned by the rules side (Rust). The flash and the shake MUST be presentation (GDScript) reacting
  to a signal from the rules side.
- **FR-012**: The hero's modes (free to move, swinging) MUST be a closed set checked exhaustively
  on the rules side, so adding a mode later cannot be forgotten anywhere.
- **FR-013**: The attack MUST be read through an input action named for player 1, `p1_attack`,
  bound to J and Z.
- **FR-014**: F2 MUST show and hide the hitboxes: each dummy's body at all times while shown, and
  the swing's reach while a blow can land. F2 MUST work whether the overlay panel is open or not,
  through a named input action.
- **FR-015**: F3 MUST spawn one more dummy in the lane, inside the screen, through a named input
  action: in front of the hero at the hero's depth, starting just inside reach, and stepping half a
  dummy's width further along the hero's facing for as long as that spot is already taken by a
  dummy. Spawned dummies behave exactly like the first.
- **FR-016**: The overlay panel MUST show a counter of hits landed since the game started, one per
  dummy hit.
- **FR-017**: F2, F3, the hitbox drawing and the hit counter MUST be absent from release (exported)
  builds, like the rest of the M2a overlay. The dummy in the scene and the attack MUST work in
  release builds.
- **FR-018**: Everything from M1 and M2a (movement, limits, depth sorting, F1 overlay, F5 reload,
  lane markers) MUST keep working.
- **FR-019**: Nothing listed as out of scope (dummy health or death, enemies that move or fight
  back, combos, jump attacks, the magic spell, hero health, sound, animation and art) MAY be added
  in M2b.

### Key Entities

- **Hero** (extended): now has a facing (left or right) and a mode: free to move, or swinging
  (with which part of the swing it is in and which dummies this swing has already hit).
- **Swing**: one use of the attack. Has a wind-up, active and recovery time, a reach in front of
  the hero, and the set of dummies it has hit.
- **Training dummy**: a tall target standing in the lane. Has a body (the area a swing must
  overlap), a position at its feet, and its current knockback slide. Never dies, never moves on
  its own.
- **Hit**: one swing landing on one dummy. Starts the freeze (once per swing), the flash, the shake
  and the knockback, and adds one to the hit counter.
- **Attack and impact tuning**: the eleven new tuning values (freeze, flash, shake strength and
  length, knockback distance and time, wind-up, active, recovery, reach, depth tolerance), in the
  same tuning data file as M1's values.
- **Debug actions** (extended): `debug_toggle_hitboxes` (F2) and `debug_spawn_dummy` (F3), beside
  M2a's two.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Over 20 swings that reach the dummy at the same depth, all 20 land with all four
  effects; over 20 swings at the right distance but at a depth difference clearly beyond the
  tolerance, none land.
- **SC-002**: Over 20 swings whose reach overlaps a dummy for several frames, the hit counter rises
  by exactly 20.
- **SC-003**: A swing reaching two dummies raises the hit counter by exactly 2 and freezes the game
  once.
- **SC-004**: In 20 hits that push a dummy towards each screen edge and lane limit, the dummy never
  ends a frame outside the lane or the screen.
- **SC-005**: In a frame-by-frame check, the hero's position does not change from the frame the
  swing starts to the frame it ends, with direction keys held throughout.
- **SC-006**: Each of the eleven tuning values, changed and applied with F5, visibly changes the
  next swing; each of the four effects, with one of its values set to 0, disappears while the other
  effects remain.
- **SC-007**: With F2 on, every landed blow shows the reach overlapping the dummy's body, and every
  miss shows them apart or at a depth difference beyond the tolerance.
- **SC-008**: In a release build, F2 and F3 have no visible effect, and the attack and the dummy
  work as in a debug build.
- **SC-009**: With five dummies and hitboxes shown on the target machine, the FPS stays at 60 for
  60 seconds of walking and swinging, apart from the first second and the intended impact freezes.
- **SC-010**: `cargo fmt --check`, `cargo build` and `cargo clippy` produce 0 warnings, the editor
  shows 0 GDScript errors or warnings, and `git status` is clean after committing.
- **SC-011**: After landing 30 hits with the final values, the owner judges that the impact feels
  weighty, not floaty. Without that judgement the milestone is not done.

## Assumptions

- **Target hardware is the Intel UHD 630.** The owner's check 12 says UHD 620; constitution v1.3.2
  corrected the target to the UHD 630, as in M1 and M2a.
- **Starting values** (data, not requirements; the owner retunes them): wind-up about 0.08 s,
  active about 0.10 s, recovery about 0.18 s; reach about 70 px in front of the hero's body; depth
  tolerance about 20 px; freeze about 0.06 s; flash about 0.08 s; shake about 6 px for about
  0.15 s; knockback about 60 px over about 0.2 s. The dummy is about 56×120 px.
- **The freeze stops play only**: the hero, the dummies' slides and the swing timers stop; the
  flash and shake (FR-007), the FPS counter, the overlay panel and debug keys keep running.
- **The shake moves the play area only**: the HUD, overlay panel and messages stay still.
- **Knockback is horizontal only**: the dummy keeps its depth; it slides along the lane.
- **Spawn position** (F3, clarified above): the start point and the half-width step come from the
  reach and the dummy's own shape, so no new tuning value is needed; spawning is a debug cheat, not
  level design. If every step up to the screen edge is taken, the dummy is placed at the edge.
- **Hitbox drawing**: outlines in placeholder colours; the reach is drawn only while a blow can
  land (the active part), so the owner sees exactly when and where it counts.
- **Hit counter**: counts since the game started; F5 does not reset it. There is no restart in M2b
  (the restart key arrives in M3).
- **Facing** is shown by the swing's side and by the reach in F2; the placeholder hero gets no
  extra facing marker unless the plan finds it needed for playing.
- **Field names**: the new values go into the existing tuning file next to `hero` and `lane`; the
  plan chooses the section and key names, and the error messages name them as in M2a.
- **The Project Brief** ([docs/project-brief.md](../../docs/project-brief.md)) lists M2b as "A still
  dummy enemy you can hit; show-hitboxes and spawn-dummy cheats", with the check "A hit gives
  knockback, flash, impact freeze and shake, all tunable from a data file". The owner's
  description matches and adds the hit counter, facing and the depth rule.
