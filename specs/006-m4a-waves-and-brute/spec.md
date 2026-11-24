# Feature Specification: M4a Waves and the Brute

**Feature Branch**: `feat/m4a-waves-and-brute`

**Created**: 2026-11-23

**Status**: Draft

**Input**: User description: "m4a-waves-and-brute: Escalating waves and a second enemy type turn
the single endless grunt into a run that gets harder until the hero falls." (full description,
acceptance checks and out-of-scope list as given by the owner in the `/speckit-specify` request)

> **Note on technology names.** A few requirements below name parts of the stack: wave progression,
> the attacker limit, enemy behaviour, the score and the session best live on the rules side
> (Rust); banners, the HUD score and the Game Over screen are presentation (GDScript) reading from
> the rules side and reacting to its signals; the brute reuses the shared fighter parts from M3
> (health, attack timing, knockback, hit resolution) rather than a copy of the grunt. These are
> constraints from the constitution (the Rust/GDScript split, "types over strings", Principles IV,
> VII and VIII) and from the owner's acceptance check 11, not choices made by this spec. Every
> *how* (node types, class names, data layout, how turns are handed out, how enemies keep apart) is
> left to the plan.

## Clarifications

### Session 2026-11-23

- Q: When the hero hits a brute during its attack, does the blow cancel the attack, as it does a
  grunt's? → A: No. A brute keeps attacking through blows: during its wind-up, active part and
  recovery, a blow does damage and plays the hit effects, but neither knocks it back nor stuns it,
  and its strike still comes. Outside an attack, a blow knocks it back a little and stuns it
  briefly. A brute's strike has to be dodged, not out-swung.
- Q: Does the hero's health come back between waves? → A: No. Health carries over for the whole
  run; nothing heals the hero in M4a except the F4 cheat. Healing arrives with potions in M4b.
- Q: Should there be a limit on how many enemies are in play at once, with the rest of a wave
  waiting off screen until others die? → A: Yes. A tuned on-screen maximum (starting at 10): a
  wave's extra enemies enter as others die, and the wave clears only when all of them are dead.
- Q: When an attacker's turn is freed, which waiting enemy should get it? → A: The one that has
  waited longest (first come, first served). *(Revised during planning, by the owner: the waiting
  enemy nearest its target gets it.)*
- Q: While waiting for a turn, should an enemy just hold its spot at the waiting distance, or keep
  moving around the hero? → A: It drifts slowly around the hero at the waiting distance, changing
  depth and side, at a tuned drift speed; a drift speed of 0 makes it hold still. *(Revised during
  planning, by the owner: each waiting enemy has a waiting spot on its own side of the hero, at
  least the waiting distance away and at its own depth, and drifts to it at the drift speed as the
  hero moves; it does not cross to the other side.)*
- Q (planning): How is the attacker limit counted? → A: As attack tokens: a pool of tuned size,
  and each kind costs its own number of tokens to close in (starting at 1 for a grunt and 2 for a
  brute), so a brute can take a whole turn's worth alone.
- Q (planning): Do a wave's enemies enter while its banner shows? → A: No. The banner shows for
  the banner time, then the enemies start entering.
- Q (planning): Does F8 wait out a breather? → A: No. It removes every enemy with no points and
  starts the next wave's banner at once.

### Session 2026-11-23 (implementation)

- Q: An attacker that finishes its recovery is itself the nearest waiting enemy, so it took every
  turn back. How should turns rotate? → A: An enemy whose turn ended, or that was stunned,
  retreats to its waiting spot and can take a turn again only once it is back (within the
  separation distance of its spot, or on the waiting ring if the crowd keeps it off the spot).
- Q: A new enemy drifted from the screen edge to its spot at the drift speed, about 11 s. How
  should enemies reach their spot? → A: Farther than the separation distance from its spot, an
  enemy walks there at its own speeds; within it, it drifts at the drift speed. A drift speed of 0
  stops it once there.
- Q: With "nearest first", a brute among grunts could wait for ever. How does no enemy wait for
  ever? → A: A tuned maximum wait (`crowd.max_wait_time`, starting at 4 s): an enemy that has
  waited longer goes first, longest waiting first, before nearer ones.
- Found while checking (no question): an enemy approaching with a turn could be held off its
  line by a waiting enemy at its spot, so it never attacked and kept its token. Enemies with a
  turn now push waiting ones aside; attacking enemies still never move.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Waves that clear and come again (Priority: P1)

The owner presses Play. A "Wave 1" banner appears, and a few grunts enter from the screen edges.
When the last enemy of the wave has died, nothing happens for a short breather; then a "Wave 2"
banner appears and a larger group enters. Each wave's make-up (how many grunts, how many brutes)
comes from the tuning data. After the last wave written in the data, waves keep coming, each one
bigger by a rule also set in the data, so a run never runs out of waves.

**Why this priority**: Waves are what make a run rise and end. They replace M3's single endless
grunt and are the frame every other part of this milestone fits into.

**Independent Test**: Play with only grunts in the first waves: wave 1's banner and grunts appear;
clear them, wait out the breather, and wave 2's banner and a larger group follow. Skip ahead with F8
past the last written wave: the waves keep growing.

**Acceptance Scenarios**:

1. **Given** a fresh run, **When** play starts, **Then** a "Wave 1" banner shows and wave 1's
   enemies enter from the screen edges, as many grunts and brutes as the data gives for wave 1.
2. **Given** a wave in progress, **When** enemies of that wave are still alive (entering ones
   included), **Then** the next wave does not start.
3. **Given** the last enemy of a wave dies, **When** the breather time has passed, **Then** the
   next wave's banner shows and its enemies enter.
4. **Given** the run reaches the wave after the last one written in the data, **Then** that wave
   and every later one follow the escalation rule in the data, each at least as large as the one
   before, without end.

---

### User Story 2 - The brute (Priority: P1)

From a wave set in the data, brutes join the grunts. A brute is a bigger placeholder shape in its
own colour, with its own health bar. It walks slower, winds up for longer than a grunt, takes many
more blows to kill and hits much harder. It barely flinches: a blow on a brute plays the hit effects
and does damage, but knocks it back only a little and stuns it only briefly, and once it has started
an attack, blows do not stop it: its strike must be dodged.

**Why this priority**: A second enemy that asks for different play is what turns a crowd of grunts
into a fight with choices. The brief names it as one of the slice's two enemy types.

**Independent Test**: Press F7: a brute enters. Count the blows to kill it and compare with a
grunt; let it strike and compare the health lost; watch its wind-up and how far a blow pushes it.

**Acceptance Scenarios**:

1. **Given** a brute and a grunt with the starting values, **When** the hero kills each, **Then**
   the brute takes clearly more blows (at least three times as many).
2. **Given** a brute's strike lands, **Then** the hero loses the brute's damage, clearly more than
   a grunt's (at least twice as much), and the hit effects play with the brute's knockback.
3. **Given** a brute attacks, **Then** its wind-up is visibly marked and clearly longer than a
   grunt's, so its strike can be avoided by stepping out of its depth or reach.
4. **Given** the hero's blow lands on a brute that is not attacking, **Then** the effects play and
   its health drops, and it is knocked back a shorter distance and stunned for a shorter time than
   a grunt would be.
5. **Given** the hero's blow lands on a brute during its wind-up, active part or recovery, **Then**
   the effects play and its health drops, but it is neither knocked back nor stunned, and its
   attack goes on: the strike still comes unless the hero leaves its reach or depth.

---

### User Story 3 - Enemies take turns (Priority: P1)

With several enemies in play, only a limited number (set in the data) close in and attack at one
time. The others hold back on their own side of the hero, at least a waiting distance away,
drifting to stay there, and the one nearest the hero takes a turn when an attacker finishes its
attack, is stunned or dies. Enemies do not stack into one
overlapping shape, whether approaching or waiting.

**Why this priority**: Without turns, a wave of five or more enemies would strike at once and no
hit could be seen coming. The owner's check 10 (deaths that felt earned) depends on it.

**Independent Test**: Press F3 until six grunts are in play, with 2 attack tokens: at most two
close in and strike at any time; the rest hold their distance. Kill an attacker: a waiting one
moves in. Watch the waiting ones: none sits inside another.

**Acceptance Scenarios**:

1. **Given** more enemies in play than the attack tokens allow, **When** they hunt the hero,
   **Then** the tokens held by enemies closing in or attacking never add up to more than the pool.
2. **Given** an enemy without a turn, **Then** it drifts at the drift speed to its waiting spot on
   its own side of the hero (at least the waiting distance away, at its own depth), never starts
   an attack, and keeps facing the hero.
3. **Given** an attacker finishes its attack's recovery, is stunned or dies, **Then** its tokens are
   freed and the waiting enemy nearest the hero takes a turn, once enough tokens are free for it.
4. **Given** several enemies approaching or waiting, **Then** no two of them overlap into one
   shape; they spread out while keeping depth order.

---

### User Story 4 - Score and the session best (Priority: P1)

A score in the HUD rises with each kill: a brute is worth more points than a grunt, both set in the
data. When the hero falls, Game Over shows the score, the wave reached, the kill count and the best
score since the game was launched. R starts again at wave 1 with the score at zero, and the session
best stays.

**Why this priority**: A score and a best to beat give each run an outcome, which the owner's
check 10 and the M4b replay test rely on.

**Independent Test**: Kill a grunt and a brute and watch the score rise by each one's points. Die;
Game Over shows score, wave, kills and best. Press R; score 0, wave 1. Die with a lower score; the
best still shows the higher one.

**Acceptance Scenarios**:

1. **Given** a grunt dies, **Then** the score rises by the grunt's points; **given** a brute dies,
   **Then** it rises by the brute's points.
2. **Given** the hero falls, **Then** Game Over shows the score, the wave reached (the wave in
   progress, or during a breather the wave just cleared), the kill count and the session best, the higher of the old best and
   this run's score.
3. **Given** Game Over or a run in progress, **When** the owner presses R, **Then** within two
   seconds a fresh run plays from wave 1 with score 0 and kills 0, and the session best is kept.
4. **Given** the game is closed and opened again, **Then** the session best starts at 0.

---

### User Story 5 - Tune the waves live, and cheat (Priority: P2)

The owner keeps the game running, changes the wave make-up, the escalation rule, the breather time,
the brute's values, the attack tokens, the waiting distance, the drift speed or the points per
enemy in the tuning data, saves and presses F5; the change applies live. F3 spawns a grunt, F7 a
brute, and F8 ends the current wave and starts the next.

**Why this priority**: The difficulty curve is found by trying numbers (Principle IV), and the
cheats make each wave easy to reach (Principle VIII). P2 because waves, the brute, turns and the
score come first.

**Independent Test**: Change wave 2's grunt count and press F5 during wave 1: wave 2 has the new
count. Press F7: a brute enters. Press F8: the wave's enemies vanish and the next banner shows.

**Acceptance Scenarios**:

1. **Given** the game is running, **When** the owner changes any new value, saves and presses F5,
   **Then** it applies with no restart (see FR-017 for which moment each value takes effect).
2. **Given** a bad new value (missing, not a number, negative, zero where zero is not allowed, or a
   wave with no enemies), **When** the owner presses F5, **Then** the reload is rejected with a
   message naming the value and why, as in M2a, and every previous value stays in use.
3. **Given** the game is running, **When** the owner presses F3 or F7, **Then** one more grunt or
   brute enters from a random screen edge and joins the current wave.
4. **Given** a wave in progress or a breather, **When** the owner presses F8, **Then** every enemy
   still in play is removed with no points or kills, and the next wave's banner shows at once.
5. **Given** a release (exported) build, **When** the owner presses F3, F4, F6, F7 or F8, **Then**
   nothing happens; R still restarts.

---

### User Story 6 - The curve feels right (Priority: P1)

The owner plays five runs to Game Over and judges: wave 1 is easy, difficulty rises noticeably from
wave to wave, and every run ends in a death that felt earned. If not, the owner retunes (US5) and
plays again.

**Why this priority**: This is the milestone's real goal (Principle I). It is not done until the
owner says yes, whatever the other checks say.

**Independent Test**: Five runs to Game Over with the final values, ending in the owner's yes or
no.

**Acceptance Scenarios**:

1. **Given** the final tuning values, **When** the owner plays five runs to Game Over, **Then** the
   owner judges that wave 1 was easy, each wave was noticeably harder than the last, and every death
   felt earned. If not, the values are retuned and the playtest is repeated.

---

### User Story 7 - Still 60 fps, clean and committed (Priority: P2)

The owner gets ten enemies on screen, mixing grunts and brutes, plays for a minute on the target
machine with V-Sync on, then runs the Quality Gate and commits.

**Why this priority**: Principle V and the Quality Gate apply to every milestone; ten enemies that
hunt, wait, keep apart and take turns add work each frame.

**Independent Test**: With F3 and F7, bring ten enemies into play; fight for 60 seconds by the
clock (F4 to stay alive), watching the FPS. Then run the Quality Gate and `git status`.

**Acceptance Scenarios**:

1. **Given** ten enemies on screen on the target machine, **When** the owner fights for one minute,
   **Then** the FPS stays at 60, apart from the first second and the intended impact freezes.
2. **Given** the M4a work is finished, **When** the owner runs `cargo fmt --check`, `cargo build`
   and `cargo clippy` and opens the project in the editor, **Then** there are no warnings and no
   GDScript errors or warnings.
3. **Given** the project has been built and played, **When** the owner runs `git status` after
   committing, **Then** the working tree is clean.

---

### Edge Cases

- **Hero dies mid-wave or during a breather**: Game Over shows the wave in progress (or, during a
  breather, the wave just cleared); no further wave starts.
- **F3 or F7 enemies**: they join the current wave and must die before it counts as cleared; they
  score and count as kills like any enemy. Pressed during a breather, they belong to the wave just
  cleared, which must be cleared again before the breather restarts.
- **F8 during a breather**: ends the breather; the next wave starts at once.
- **F8 at Game Over**: does nothing; nor do F3 and F7.
- **Enemies fading out**: a dying enemy no longer counts as alive, holds no turn, and does not keep
  the wave going; its points are added when it starts dying, once.
- **Token pool or costs changed with F5**: a smaller pool takes no turn away from an enemy already
  holding one; it applies as tokens are freed and handed out again.
- **More tokens than the enemies in play need**: every enemy may close in, as in M3.
- **The nearest waiting enemy costs more than the free tokens** (a brute while one grunt attacks):
  it waits until enough are free; no farther grunt goes first.
- **An enemy waiting while the hero walks right up to it**: it does not attack without a turn; it
  moves back to keep its waiting distance where the lane and screen allow.
- **A waiting enemy is hit**: it takes damage, effects and its stun as usual; being hit gives it no
  turn.
- **Enemies at a screen edge or lane limit**: they keep apart within the limits; when there is no
  room, they may touch but do not sit fully inside one another.
- **Very late waves** (for example after F8 many times): the run keeps going; enemies above the
  on-screen maximum wait off screen and enter as others die (FR-004a).
- **Training dummies** (F6): enemies ignore them; they never take turns, score or count as kills.
- **R pressed during a banner, a breather, a fade or a freeze**: the fresh run starts anyway at wave
  1; nothing from the old run carries over except the session best.
- **Bad startup tuning** (as M2a): nobody moves or attacks and no wave starts until a reload
  succeeds.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A run MUST start at wave 1 with the hero at its start position at full health, score
  0, kill count 0, and wave 1's enemies entering. M3's rule of one grunt respawning after the last
  dies MUST be replaced by waves.
- **FR-002**: Each wave MUST start with a banner naming its number ("Wave 1", "Wave 2", …), shown
  for the banner time; its enemies MUST start entering when the banner ends, from randomly chosen
  screen edges (left or right) at random depths inside the lane, one after another rather than all in the
  same instant.
- **FR-003**: Each wave written in the tuning data MUST bring exactly the number of grunts and
  brutes the data gives for it.
- **FR-004**: The next wave MUST start only when no enemy of the current wave is alive (entering
  ones count as alive, dying ones do not), none is still waiting to enter, and the breather time,
  counted from the last death, has passed.
- **FR-004a**: No more than the on-screen maximum (from the data) of enemies MUST be in play at
  once, dying ones not counted. A wave's enemies beyond it MUST wait off screen and enter, one at a
  time, as others die. F3 and F7 enemies count towards the maximum and wait like any other.
- **FR-005**: Every wave after the last one written in the data MUST be built by an escalation rule
  in the data that adds grunts and brutes as the wave number rises, so that no later wave has fewer
  enemies than the one before and the run never runs out of waves.
- **FR-006**: A brute MUST be a placeholder shape clearly bigger than a grunt, in a colour unlike
  the hero's, the grunt's and the dummy's, with a small health bar above it, drawn in depth order.
- **FR-007**: A brute MUST hunt and attack the hero under the same rules as a grunt (line up at the
  hero's depth, close in, attack in front, within reach and depth tolerance, with a marked wind-up,
  an active part and a recovery, landing at most once per attack), with its own values: health,
  damage, walk speeds, wind-up, active and recovery times, reach, attack-box size, body size,
  knockback dealt, and the knockback and hit-stun it takes.
- **FR-008**: A blow on a brute MUST play the shared hit effects and lower its health. When the
  brute is not attacking, the blow MUST also push it back by its own (shorter) knockback-taken
  distance and stun it for its own (shorter) hit-stun time. During its wind-up, active part or
  recovery, the blow MUST NOT push it, stun it or cancel its attack.
- **FR-009**: The brute MUST reuse the same shared parts as the hero and the grunt for health,
  attack timing, knockback and hit resolution, with only its own values and its look being new; its
  behaviour MUST NOT be a copy of the grunt's.
- **FR-010**: An enemy MUST hold its kind's attack-token cost (from the data) to close in and
  attack, and the tokens held at one time MUST never exceed the token pool (from the data). Every
  other enemy MUST wait at its waiting spot, on its own side of the hero, at least the waiting
  distance (from the data) away and at its own depth, within the lane and screen: farther than the
  separation distance from the spot it walks there at its own speeds, nearer it drifts at the
  drift speed (from the data); it keeps facing the hero and never starts an attack. A drift speed
  of 0 MUST make it stop once at its spot.
- **FR-011**: An enemy's tokens MUST be freed when it finishes its attack's recovery, is stunned,
  or dies. It then retreats to its spot and MUST NOT get a turn until it is back there (within
  the separation distance of the spot, or the waiting distance less the separation distance from
  the hero). Free tokens MUST go first to waiting enemies that have waited at least the maximum
  wait (from the data), longest first, then to the nearest; only when the enemy's whole cost
  fits, and nobody behind it in that order is served first. Grunts and brutes MUST share one
  token pool.
- **FR-012**: Enemies approaching or waiting MUST keep apart so that no two overlap into one shape,
  while staying in the lane and on screen and keeping depth order. An enemy with a turn MUST NOT
  be kept from its way in by waiting ones: they give way to it.
- **FR-013**: Each kill MUST add the dead enemy's points (grunt points or brute points, from the
  data) to the score, once, when it starts dying. The score MUST be shown in the HUD during play,
  stay still during the shake, and work in release builds.
- **FR-014**: The session best MUST be the highest score of any run since the game was launched. It
  MUST be kept across restarts and MUST NOT be saved between launches.
- **FR-015**: At Game Over, the screen MUST show the score, the wave reached, the kill count, the
  session best and "Press R to restart".
- **FR-016**: The `game_restart` action (R) MUST start a fresh run (FR-001) from play or Game Over,
  removing every enemy and dummy, resetting score, kills, wave and the hit counter and keeping the
  session best, within two seconds of the press, in debug and release builds.
- **FR-017**: The wave list (grunts and brutes per wave), the escalation rule, the breather time,
  the banner time, the time between enemies entering, the on-screen maximum, every brute value in
  FR-007, the token pool and each kind's token cost, the waiting distance, the drift speed, the
  separation distance, the maximum wait and the points per grunt and per
  brute MUST be read from the tuning data file, checked like the earlier values, and none may
  appear as a literal value in code. F5
  MUST apply them with the M2a reload rules (all or nothing): a changed wave list or escalation
  rule applies from the next wave to start; enemy values from the next moment they are used, as in
  M3; the token pool and costs as tokens are next handed out; points from the next kill.
- **FR-018**: Health, damage, walk speeds, active time, reach, the token pool, each token cost
  (also at most the pool), the on-screen maximum, the points and the waiting distance MUST be greater than 0; the wave list MUST hold at
  least one wave, and each wave at least one enemy; every other new value (the drift speed
  included) MAY be 0, and no value may be negative.
- **FR-019**: F3 MUST spawn one more grunt and F7 one more brute, each entering from a random edge
  like a wave's enemy and joining the current wave. F8 MUST remove every enemy in play with no
  points or kills and start the next wave at once, skipping any breather. Each MUST use a named
  input action, work whether the panel is open or not, and do nothing at Game Over.
- **FR-020**: F3, F4, F6, F7, F8 and every other debug tool MUST be absent from release builds,
  like the rest of the M2a overlay.
- **FR-021**: Everything from M3 MUST keep working: the hero's moves, health, stun and
  invulnerability, the grunt's attack and its warning, the hit effects, Game Over, R, F1, F2, F4,
  F5 and F6. F2 MUST also outline each brute's body and its attack box while its blow can land.
- **FR-022**: Wave progression, the attack tokens and turns, enemy behaviour, keeping apart, the
  score and the session best MUST be owned by the rules side (Rust), with modes as closed sets
  checked exhaustively. Banners, the HUD score, the brute's look and the Game Over screen MUST be
  presentation (GDScript) reading from the rules side and reacting to its signals.
- **FR-023**: Enemies MUST target and damage a hero through its player index, choosing the nearest
  living hero, as in M3; turns and waiting distance MUST be counted against that hero, never a
  single hard-coded "the player".
- **FR-024**: Nothing listed as out of scope (potions, magic and the spell, jumping, saving the best
  score between launches, new hero moves, sound, animation and art) MAY be added in M4a.

### Key Entities

- **Run** (extended): one play from start to Game Over. Adds the current wave number, the wave
  state (banner, fighting, breather), the score, and the enemies still to enter in this wave.
- **Session**: everything that lasts from launch to quit across restarts; for now, only the best
  score.
- **Wave**: a numbered group of enemies, so many grunts and so many brutes, from the data list or,
  past its end, from the escalation rule.
- **Brute**: an enemy like the grunt, with its own values and look: bigger, slower, tougher,
  harder-hitting, longer wind-up, less knockback and stun taken.
- **Turn**: the right to close in and attack, held as attack tokens (the kind's cost) from one
  pool; free tokens go to an enemy that has waited past the maximum wait, else to the waiting
  enemy nearest the hero. An enemy without a turn waits at its spot on its own side; one whose
  turn just ended retreats there first.
- **Wave tuning**: the new values in FR-017, in the same tuning data file as before.
- **Actions** (extended): new `debug_spawn_brute` (F7) and `debug_next_wave` (F8).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 3 runs, each written wave brings exactly its data's grunts and brutes, and the
  next wave never starts while an enemy of the current wave is alive or before the breather has
  passed.
- **SC-002**: Skipping with F8 to 5 waves past the last written one, each wave has at least as many
  enemies as the one before and matches the escalation rule.
- **SC-003**: With the starting values, a brute takes at least 3 times as many blows to kill as a
  grunt, its strike costs the hero at least 2 times a grunt's, its wind-up is longer than a grunt's
  every time, and every blow on it outside an attack pushes it a shorter distance and stuns it for
  less time than the same blow on a grunt. In 10 blows landed during a brute's wind-up with the
  hero staying in reach, 10 of its strikes still come.
- **SC-004**: Over 60 seconds with six grunts in play and a pool of 2 tokens, no more than 2 are
  ever closing in or attacking at once, and every freed turn is taken by the waiting grunt nearest
  the hero; with a brute (cost 2) attacking, no grunt closes in.
- **SC-005**: Over the same 60 seconds, no two enemies are ever drawn fully over one another while
  approaching or waiting.
- **SC-006**: Every kill raises the score by exactly the enemy's points; Game Over shows the same
  score as the HUD, the wave reached, the kills, and a session best equal to the highest score of
  the session.
- **SC-007**: In 10 restarts (5 from Game Over, 5 during play), a fresh run at wave 1 with score 0
  and kills 0 is playing within 2 seconds of pressing R, and the session best is unchanged.
- **SC-008**: F3, F7 and F8 work as described in a debug build and have no visible effect in a
  release build, where waves, the score, Game Over and R still work.
- **SC-009**: Each new value, changed and applied with F5, visibly changes play from the moment
  FR-017 gives; each bad value is rejected with a message naming it.
- **SC-010**: With ten enemies on screen on the target machine, the FPS stays at 60 for 60 seconds
  of fighting, apart from the first second and the intended impact freezes.
- **SC-011**: `cargo fmt --check`, `cargo build` and `cargo clippy` produce 0 warnings, the editor
  shows 0 GDScript errors or warnings, and `git status` is clean after committing.
- **SC-012**: After five runs to Game Over with the final values, the owner judges that wave 1 was
  easy, difficulty rose noticeably from wave to wave, and every death felt earned. Without that
  judgement the milestone is not done.

## Assumptions

- **Target hardware is the Intel UHD 630.** The owner's check 12 says "UHD 620", but constitution
  1.3.2 corrected the target to the UHD 630 and M3 was verified on it; the brief now says UHD 630
  too. Check 12 is read as the UHD 630.
- **Hero health between waves** (clarified): health carries over for the whole run; no wave or
  breather restores it. The starting values must let a careful player reach the brute waves.
- **The escalation rule** adds a fixed number of grunts per wave past the last written one, and a
  brute every so many waves, both from the data; the plan may pick another rule shape as long as it
  is set in the data and never shrinks a wave.
- **On-screen maximum** (clarified, FR-004a): it starts at 10, the number checked in SC-010. It is
  a performance guard, not a difficulty tool; the attack tokens set how hard a wave feels.
- **Banner** (planning answer): shown for about 1.5 s at the top centre of the play area, not
  blocking play; the wave's enemies start entering when it ends.
- **Starting values** (data, not requirements; the owner retunes them): waves 1–3 of 2, 3 and 4
  grunts, brutes from wave 4 (wave 4: 3 grunts and 1 brute; wave 5: 4 grunts and 1 brute); past
  that, one more grunt per wave and one more brute every 3 waves; breather about 2 s; entry interval
  about 0.6 s; a pool of 2 attack tokens, grunt cost 1, brute cost 2; waiting distance about
  180 px; separation distance about 56 px; drift speed about 40 px/s; maximum wait 4 s; grunt
  10 points, brute 30 points. Brute: health about 120 (12 hero blows), damage 35, walk speeds about
  two thirds of a grunt's, wind-up about 0.7 s, active about 0.12 s, recovery about 0.6 s, reach
  about 75 px, body about 1.4 times a grunt's, knockback taken about a quarter of a grunt's,
  hit-stun taken about 0.1 s.
- **The M3 respawn delay** is retired with the respawn rule; the breather takes its place.
- **Kill count** stays on the HUD beside the score and on Game Over.
- **The M2b hit counter** stays a debug panel line and resets with R.
- **Random choices** (entry edge, entry depth) need no fixed seed.
- **The Project Brief** ([docs/project-brief.md](../../docs/project-brief.md)) lists M4a as
  "Escalating waves, the brute, an attacker limit and a score", checked by "Five runs rise in
  difficulty and end in deaths that felt earned". The owner's description matches and adds the
  session best, the waiting distance, keeping apart, F7 and F8.
- **Acceptance check mapping**: 1 → US1 sc. 1–3 / FR-002–FR-004 / SC-001; 2 → US1 sc. 4 / FR-005
  / SC-002; 3 → US2 / FR-006–FR-008 / SC-003; 4 → US3 sc. 1–3 / FR-010, FR-011 / SC-004; 5 → US3
  sc. 4 / FR-012 / SC-005; 6 → US4 sc. 1–2 / FR-013–FR-015 / SC-006; 7 → US4 sc. 3–4 / FR-016 /
  SC-007; 8 → US5 sc. 3–5 / FR-019, FR-020 / SC-008; 9 → US5 sc. 1–2 / FR-017, FR-018 / SC-009; 10
  → US6 / SC-012; 11 → FR-009, FR-022; 12 → US7 sc. 1 / SC-010; 13 → US7 sc. 2–3 / SC-011.
