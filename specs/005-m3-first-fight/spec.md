# Feature Specification: M3 First Fight

**Feature Branch**: `feat/m3-first-fight`

**Created**: 2026-11-21

**Status**: Draft

**Input**: User description: "m3-first-fight: A grunt that hunts the hero, so the game has a full
run from start to game over and back." (full description, acceptance checks and out-of-scope list
as given by the owner in the `/speckit-specify` request)

> **Note on technology names.** A few requirements below name parts of the stack: enemy behaviour,
> health, damage, death and the run state live on the rules side (Rust); health bars, the Game Over
> screen and the fade are presentation (GDScript) reading from the rules side and reacting to its
> signals; modes are closed sets checked exhaustively; damage and health refer to a player index;
> and the restart key is an input action named `game_restart`. These are constraints from the
> constitution (the Rust/GDScript split, "types over strings", Principles VII and VIII) and from the
> owner's acceptance checks 11 and 12, not choices made by this spec. Every *how* (node types, class
> names, data layout, how the restart is done) is left to the plan.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A grunt hunts the hero and strikes (Priority: P1)

The owner presses Play. The hero stands in the lane at full health, its health bar in the top-left
corner. A grunt, a placeholder shape in its own colour with a small health bar above its head,
walks in from a screen edge. It first moves up or down to the hero's depth in the lane, then walks
towards the hero. Once in range and at the hero's depth, it stops and winds up, visibly changing
colour, for clearly longer than the hero's own wind-up, then strikes. If the hero is still in front
of it, in reach and at its depth, the blow lands: the M2b hit effects play on the hero (freeze,
white flash, shake, knockback away from the grunt), the hero's health bar drops, and for a short
time the hero blinks and cannot be hit again. A watchful player can avoid the blow by stepping out
of the grunt's depth during the wind-up.

**Why this priority**: Something that fights back is what turns hitting a dummy into a game. Every
later milestone (waves, the brute, the boss) builds on one enemy that hunts, telegraphs and hits
fairly.

**Independent Test**: Stand still and let the grunt come: it lines up, approaches, winds up and
hits; the health bar drops and the hero blinks. Repeat, and during the wind-up step up or down out
of its depth: the strike misses.

**Acceptance Scenarios**:

1. **Given** a grunt enters at a different depth from the hero, **When** it approaches, **Then** it
   first moves to the hero's depth and only then closes in horizontally.
2. **Given** the grunt is at the hero's depth but out of reach, **When** time passes, **Then** it
   walks closer and does not attack until it is in range.
3. **Given** the grunt is in range and at the hero's depth, **When** it attacks, **Then** its
   wind-up is visible (a colour change) and lasts clearly longer than the hero's wind-up.
4. **Given** the grunt is winding up, **When** the hero steps out of its depth (beyond the depth
   tolerance) or out of its reach before the strike, **Then** the strike misses: no effects, no
   health lost.
5. **Given** the grunt's strike lands, **Then** the hero's health bar drops by the grunt's damage,
   the hit effects play on the hero, and the hero is knocked back away from the grunt.
6. **Given** the hero has just been hit, **When** another strike lands during the invulnerability
   time, **Then** nothing happens: no damage, no effects; the hero blinks for that whole time.
7. **Given** the hero is hit during its own swing, **When** the blow lands, **Then** the hero's
   swing is cancelled and the hero is stunned for the hero hit-stun time, unable to move or attack.

---

### User Story 2 - Beat the grunt, and another comes (Priority: P1)

The owner hits the grunt. Each blow plays the M2b hit effects, lowers the grunt's bar and stuns it
briefly; a blow during its wind-up cancels its attack. When its health reaches zero, the grunt
flashes, fades out and is removed, the kill counter rises by one, and after a short pause a new
grunt enters from a random screen edge.

**Why this priority**: Winning exchanges is the other half of the fight; killing grunts is the
run's score and pace.

**Independent Test**: Hit a grunt until it dies: it fades out, the kill counter shows 1, and a new
grunt enters after the respawn delay. Hit another grunt during its wind-up: its strike never comes.

**Acceptance Scenarios**:

1. **Given** a grunt is in front of the hero, in reach and at its depth, **When** the hero's swing
   lands, **Then** the grunt flashes, the freeze and shake play, it is knocked back, its bar drops
   by the hero's damage, and it is stunned for the grunt hit-stun time.
2. **Given** a grunt is winding up, **When** the hero's blow lands, **Then** its attack is
   cancelled; after the stun it starts over (lines up, approaches, winds up again).
3. **Given** a grunt's health reaches zero, **Then** it flashes, fades out over the death fade time
   and is removed; while fading it cannot attack, move or be hit, and it never counts twice.
4. **Given** a grunt has died, **Then** the kill counter rises by exactly one, and after the respawn
   delay a new grunt enters from a randomly chosen screen edge.

---

### User Story 3 - Game over and a fast restart (Priority: P1)

When the hero's health reaches zero, the hero falls, the action stops, and a Game Over screen shows
the kill count and "Press R to restart". R starts a fresh run at once, at Game Over or at any time
during play.

**Why this priority**: A whole run, lost and started again, is the loop every later milestone
builds on, and Principle VIII requires a restart in under two seconds from M3.

**Independent Test**: Let the grunts win: the hero falls and Game Over shows the kills. Press R and
time it: a fresh run within two seconds. Press R in the middle of a run too.

**Acceptance Scenarios**:

1. **Given** the hero's health reaches zero, **Then** the hero falls, every grunt stops, and the
   Game Over screen shows the kill count and "Press R to restart".
2. **Given** the Game Over screen, **When** the owner presses R, **Then** within two seconds a fresh
   run is playing: hero at its start position with full health, kill count zero, every grunt and
   dummy gone, and one grunt entering.
3. **Given** a run in progress, **When** the owner presses R, **Then** the same fresh run starts.
4. **Given** the Game Over screen, **When** the owner presses movement or attack keys, **Then**
   nothing happens.

---

### User Story 4 - Tune the fight live, and cheat (Priority: P2)

The owner keeps the game running, changes any of the fight's values in the tuning data file (hero
and grunt health, damage, the grunt's walk speed, wind-up, active and recovery times and reach,
hit-stun times, the hero's invulnerability time, the respawn delay, the death fade time), saves and
presses F5; the change applies live. With the debug overlay, F3 spawns a grunt, F6 spawns a
training dummy and F4 refills the hero's health.

**Why this priority**: Fairness is found by trying numbers (Principle IV), and the cheats make
each situation easy to set up (Principle VIII: refill health arrives in M3). P2 because the fight
itself (US1–US3) comes first.

**Independent Test**: Halve the grunt's wind-up and press F5: the next attack winds up visibly
faster. Press F3 twice: two more grunts enter. Get hit, press F4: the bar is full again. Press F6:
a dummy appears in front of the hero.

**Acceptance Scenarios**:

1. **Given** the game is running, **When** the owner changes any fight value, saves and presses F5,
   **Then** it applies with no restart (see FR-016 for which moment each value takes effect).
2. **Given** a bad fight value (missing, not a number, negative, or zero where zero is not allowed),
   **When** the owner presses F5, **Then** the reload is rejected with a message naming the value
   and why, as in M2a, and every previous value stays in use.
3. **Given** the game is running, **When** the owner presses F3, **Then** one more grunt enters
   from a random screen edge, exactly like a normal grunt.
4. **Given** the hero has lost health, **When** the owner presses F4, **Then** its bar is full.
5. **Given** the game is running, **When** the owner presses F6, **Then** a training dummy appears
   where M2b's F3 put one, and behaves as in M2b.
6. **Given** a release (exported) build, **When** the owner presses F3, F4 or F6, **Then** nothing
   happens; R still restarts.

---

### User Story 5 - The fight is fair (Priority: P1)

The owner plays five runs to game over and judges whether every hit the hero took could have been
seen coming. If not, the owner retunes (US4) and plays again.

**Why this priority**: This is the milestone's real goal (Principle I). It is not done until the
owner says the fight is fair, whatever the other checks say.

**Independent Test**: Five runs to game over with the final values, ending in the owner's yes or
no.

**Acceptance Scenarios**:

1. **Given** the final tuning values, **When** the owner plays five runs to game over, **Then** the
   owner judges that every hit taken could have been seen coming. If not, the values are retuned
   and the playtest is repeated before the milestone can end.

---

### User Story 6 - Still 60 fps, clean and committed (Priority: P2)

The owner spawns grunts with F3 until five are in play, plays for a minute on the target machine
with V-Sync on, then runs the Quality Gate and commits.

**Why this priority**: Principle V and the Quality Gate apply to every milestone; several grunts
hunting, striking and fading add work each frame.

**Independent Test**: With five grunts, fight for 60 seconds by the clock (using F4 to stay alive),
watching the FPS. Then run the Quality Gate and `git status`.

**Acceptance Scenarios**:

1. **Given** five grunts in play on the target machine, **When** the owner fights for one minute,
   **Then** the FPS stays at 60, apart from the first second and the intended freezes on impact.
2. **Given** the M3 work is finished, **When** the owner runs `cargo fmt --check`, `cargo build`
   and `cargo clippy` and opens the project in the editor, **Then** there are no warnings and no
   GDScript errors or warnings.
3. **Given** the project has been built and played, **When** the owner runs `git status` after
   committing, **Then** the working tree is clean.

---

### Edge Cases

- **Several grunts at once** (F3): each hunts and attacks on its own; there is no turn-taking (out
  of scope). Grunts do not block each other, the hero, dummies or pillars; they overlap by depth.
- **Grunt killed while others are in play**: every killed grunt is replaced by one new grunt after
  the respawn delay, so the number in play stays the same; five grunts from F3 stay five. Each kill
  has its own delay, so several kills close together bring several grunts.
- **Grunt hit during its active part or recovery**: it is stunned like any hit; an attack whose
  blow has already landed is not undone.
- **Grunt and hero hit each other in the same tick**: both hits count; each takes its damage and
  effects; one freeze.
- **Hero killed by a blow**: the hit effects play, then the hero falls; no invulnerability blink.
- **Grunt fading out when the hero dies**: it finishes fading; the kill still counts.
- **Respawn pending when the hero dies**: no new grunt enters during Game Over.
- **R pressed during the freeze, a fade, a pending respawn or the hero's fall**: the fresh run
  starts anyway; nothing from the old run carries over.
- **R held down**: one restart per press.
- **F4 at Game Over**: does nothing; only R starts again. F4 at full health does nothing visible.
- **F5 lowers the maximum health below a fighter's current health**: current health drops to the
  new maximum. Raising the maximum does not heal; the bar shows the new proportion.
- **Knockback towards a screen edge or lane limit**: the hero or grunt stops at the limit, as the
  dummy does in M2b.
- **The grunt is entering from off screen**: it can be neither hit nor attack until it is fully on
  screen; after that, screen and lane limits hold it as they hold the dummy.
- **The hero walks behind a grunt**: the grunt turns to face the hero; it never attacks away from
  the hero.
- **Training dummies** (F6): the hero hits them as in M2b; grunts ignore them; they never die and
  never count as kills. Grunt strikes only hit heroes.
- **Hit counter** (M2b panel line): still counts every blow the hero lands, on grunts and dummies;
  it resets with R.
- **Bad startup tuning** (as M2a): nobody moves or attacks and no grunt enters until a reload
  succeeds.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A run MUST start with the hero at its scene start position, at full health, facing
  right, with the kill count at zero and one grunt entering from a randomly chosen screen edge
  (left or right) at a random depth inside the lane. The training dummy no longer stands in the
  scene at start; dummies come only from F6.
- **FR-002**: A grunt MUST be a placeholder shape in a colour unlike the hero's and the dummy's,
  drawn in depth order with the hero, dummies and pillars, with a small health bar above its head.
- **FR-003**: A grunt MUST move to within the depth tolerance of the hero's depth before moving
  horizontally towards the hero, and MUST face the hero. If the hero leaves that depth while the
  grunt approaches, the grunt MUST line up again before closing in further.
- **FR-004**: A grunt MUST start an attack only when the hero is in front of it, within its reach,
  and within the depth tolerance. Its attack MUST have a wind-up, an active part and a recovery,
  timed from the tuning data; during the wind-up the grunt MUST be visibly marked (a colour change),
  and it MUST NOT move or turn from the start of the wind-up to the end of the recovery.
- **FR-005**: The grunt's strike MUST land on the hero using the M2b rule (attack box overlapping
  the hero's body during the active part, depth difference within the depth tolerance), at most
  once per attack.
- **FR-006**: Tuning validation MUST reject a grunt wind-up that is not longer than the hero's
  wind-up, so the grunt's attack can always be seen coming.
- **FR-007**: When a blow lands on the hero or a grunt, the M2b hit effects MUST play with the M2b
  impact values: one freeze per blow, a white flash on the one hit, the shake, and a knockback away
  from the attacker that stays inside the lane and screen.
- **FR-008**: A blow on a grunt MUST lower its health by the hero's damage and stun it for the
  grunt hit-stun time: during the stun it cannot move or attack, and any attack it had started is
  cancelled. After the stun it MUST resume hunting from its new position.
- **FR-009**: A blow on the hero MUST lower its health by the grunt's damage, cancel any swing in
  progress, stun the hero for the hero hit-stun time (no moving or attacking), and make the hero
  blink and immune to further hits for the hero invulnerability time, counted from the hit.
- **FR-010**: A grunt at zero health MUST flash, fade out over the death fade time and then be
  removed; while dying it cannot move, attack, be hit or block anything. Its death MUST add exactly
  one to the kill count, and after the respawn delay a new grunt MUST enter from a random edge, one new grunt
  for every grunt killed, so the number in play stays the same.
- **FR-011**: At zero health the hero MUST fall (a visible change of the placeholder, such as
  tipping over), all grunts MUST stop, player input other than restart MUST be ignored, and a Game
  Over screen MUST show the kill count and "Press R to restart".
- **FR-012**: The `game_restart` action, bound to R, MUST start a fresh run (FR-001) from play or
  Game Over, removing every grunt and dummy and resetting health, kill count and the hit counter,
  within two seconds of the press. It MUST work in release builds.
- **FR-013**: The hero's health bar MUST sit in the top-left corner of the screen, and the kill
  count MUST be visible during play. Both MUST stay still during the shake and work in release
  builds. The M2a debug panel MUST NOT cover the health bar.
- **FR-014**: Health and damage MUST belong to the hero identified by its player index; grunts MUST
  target and damage a hero through that index, never a single hard-coded "the player".
- **FR-015**: The hero's and the grunt's maximum health, the hero's and the grunt's damage, the
  grunt's walk speeds, wind-up, active and recovery times, reach and attack-box size, the grunt's
  body size, the hero's body size, the hero and grunt hit-stun times, the hero's invulnerability
  time, the respawn delay and the death fade time MUST be read from the tuning data file, checked
  like the earlier values, and none may appear as a literal value in code.
- **FR-016**: F5 MUST apply the fight values with the M2a reload rules (all or nothing). Speeds,
  damage and times apply from the next moment they are used (the next step, blow, attack, stun or
  death); an attack already under way finishes with the values it started with; maximum health
  applies at once, with current health lowered to it if needed.
- **FR-017**: Health values MUST be greater than 0, damage MUST be greater than 0, the grunt's walk
  speeds, active time and reach MUST be greater than 0, the grunt's wind-up MUST be longer than the
  hero's (FR-006); every other new time MAY be 0 and no value may be negative.
- **FR-018**: F3 MUST spawn one more grunt, entering from a random screen edge exactly like a
  normal grunt (FR-001), F6 MUST spawn
  a training dummy exactly as M2b's F3 did, and F4 MUST refill the hero's health to its maximum
  during play. Each MUST use a named input action and work whether the panel is open or not.
- **FR-019**: F3, F4, F6 and every other debug tool MUST be absent from release builds, like the
  rest of the M2a overlay.
- **FR-020**: The hero's facing, swing, hit rules, the dummy, F1, F2, F5 and the M1 movement, limits
  and depth sorting MUST keep working; F2 MUST also outline the hero's body, each grunt's body, and
  each grunt's attack box while its blow can land.
- **FR-021**: Enemy behaviour, health, damage, death, the kill count and the run state (playing,
  game over) MUST be owned by the rules side (Rust), with the grunt's and the run's modes as closed
  sets checked exhaustively. Health bars, the kill display, the Game Over screen, the blink, the
  wind-up colour, the fall and the fade MUST be presentation (GDScript) reading from the rules side
  and reacting to its signals.
- **FR-022**: Nothing listed as out of scope (waves and difficulty ramps, the brute, potions, magic,
  jumping, score beyond the kill count, enemies taking turns to attack, sound, animation and art)
  MAY be added in M3.

### Key Entities

- **Run**: one play from start to game over. Has a state (playing or game over), the kill count,
  and the pending respawn, if any. Restart replaces it with a fresh one.
- **Hero** (extended): now has health and a maximum, a hurtbox, a player index that health and
  damage refer to, and two more modes beside free and swinging: stunned (after a hit) and fallen
  (dead). Invulnerability runs alongside the modes for a set time after a hit.
- **Grunt**: an enemy in the lane. Has health and a maximum, a body (hurtbox), a facing, and a mode:
  entering, lining up, closing in, attacking (wind-up, active, recovery), stunned, or dying. Its
  attack hits the hero at most once per attack.
- **Hit** (extended): a blow from the hero on a grunt or dummy, or from a grunt on the hero. Deals
  damage (except to dummies), plays the M2b effects, and stuns the one hit.
- **Fight tuning**: the new values in FR-015, in the same tuning data file as before.
- **Actions** (extended): `game_restart` (R); `debug_spawn_dummy` moves to F6; new
  `debug_spawn_grunt` (F3) and `debug_refill_health` (F4).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 10 approaches from different depths, the grunt reaches the hero's depth before it
  closes the horizontal distance, and it starts no attack while out of range or out of depth.
- **SC-002**: The grunt's wind-up, measured frame by frame, is longer than the hero's every time;
  in 10 attacks where the hero steps out of depth during the wind-up, 0 strikes land.
- **SC-003**: In 10 blows landed on a grunt during its wind-up, 0 of its strikes follow.
- **SC-004**: Every strike that lands on the hero lowers its health by exactly the grunt's damage
  and plays all the hit effects; every strike during the invulnerability time does nothing.
- **SC-005**: Every grunt killed raises the kill count by exactly one, fades out over the death fade
  time, and is followed by a new grunt after the respawn delay; with five grunts in play, five
  remain after any number of kills, once the delays have passed.
- **SC-006**: At zero health the hero falls and the Game Over screen shows the same kill count the
  HUD showed.
- **SC-007**: In 10 restarts (5 from Game Over, 5 during play), a fresh run (full health, kills 0,
  no leftover grunts or dummies, one grunt entering) is playing within 2 seconds of pressing R.
- **SC-008**: Each fight value, changed and applied with F5, visibly changes the fight from the
  moment FR-016 gives; each bad value is rejected with a message naming it.
- **SC-009**: F3, F4 and F6 work in a debug build and have no visible effect in a release build,
  where R, the health bar, the kill count and Game Over still work.
- **SC-010**: With five grunts in play on the target machine, the FPS stays at 60 for 60 seconds of
  fighting, apart from the first second and the intended impact freezes.
- **SC-011**: `cargo fmt --check`, `cargo build` and `cargo clippy` produce 0 warnings, the editor
  shows 0 GDScript errors or warnings, and `git status` is clean after committing.
- **SC-012**: After five runs to game over with the final values, the owner judges that every hit
  the hero took could have been seen coming. Without that judgement the milestone is not done.

## Assumptions

- **Target hardware is the Intel UHD 630**, as named in check 13 and constitution v1.3.2.
- **One milestone, not two.** The owner chose a single M3 (branch `feat/m3-first-fight`) rather
  than splitting it like M2; the plan sizes tasks so the game stays playable after each phase.
- **Starting values** (data, not requirements; the owner retunes them): hero health 100, grunt
  health 30, hero damage 10 (three blows kill a grunt), grunt damage 15 (seven strikes kill the
  hero); grunt walk speeds about half the hero's; grunt wind-up about 0.45 s, active about 0.1 s,
  recovery about 0.4 s, reach about 60 px; hero hit-stun about 0.25 s, grunt hit-stun about 0.3 s;
  hero invulnerability about 1.0 s; respawn delay about 1.5 s; death fade about 0.5 s; the grunt's
  body about the hero's size.
- **The hit effects are shared.** Freeze, flash, shake and knockback use M2b's single set of impact
  values for every blow, whoever is hit; the owner can split them later if the playtest asks.
- **The depth tolerance is shared**: one value for every attacker, as in M2b.
- **Grunt walk speeds** are horizontal and vertical, like the hero's, so lining up can be slower
  than closing in.
- **Entering**: a grunt starts just off screen at the chosen edge and walks in; screen and lane
  limits apply once it is fully on screen.
- **The fall and the Game Over screen**: the fallen hero stays visible behind the Game Over screen,
  which appears right after the fall; the exact look is a presentation choice.
- **The debug panel moves** below the health bar (or the bar is drawn above it), so both are
  readable at once; the FPS counter stays top-right.
- **Kill count display**: a short label near the health bar during play; the Game Over screen
  repeats it. There is no high score (scores beyond the kill count are out of scope).
- **The M2b hit counter stays** a debug panel line and resets with R.
- **Random choices** (entry edge, entry depth) need no fixed seed; the fight does not need to be
  repeatable.
- **The Project Brief** ([docs/project-brief.md](../../docs/project-brief.md)) lists M3 as "A
  chasing grunt, health bars, death, game over, restart, refill-health cheat", checked by "Play from
  start to game over and restart in under two seconds". The owner's description matches and adds
  the kill count, the visible wind-up, invulnerability and the F6 move.
- **Acceptance check mapping**: 1 → US1 sc. 1–2 / FR-003, FR-004 / SC-001; 2 → US1 sc. 3–4 /
  FR-004–FR-006 / SC-002; 3 → US2 sc. 2 / FR-008 / SC-003; 4 → US1 sc. 5–6 / FR-007, FR-009 /
  SC-004; 5 → US2 sc. 3–4 / FR-010 / SC-005; 6 → US3 sc. 1 / FR-011 / SC-006; 7 → US3 sc. 2–3 /
  FR-012 / SC-007; 8 → US4 sc. 3–6 / FR-018, FR-019 / SC-009; 9 → US4 sc. 1–2 / FR-015–FR-017 /
  SC-008; 10 → US5 / SC-012; 11 → FR-021; 12 → FR-012, FR-014; 13 → US6 sc. 1 / SC-010; 14 → US6
  sc. 2–3 / SC-011.
