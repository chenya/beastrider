# Feature Specification: M4b Ashen Stand

**Feature Branch**: `feat/m4b-ashen-stand`

**Created**: 2026-11-25

**Status**: Draft

**Input**: User description: "m4b-ashen-stand: Potions and a screen-clearing spell complete the
first playable slice, ending in the go or no-go playtest." (full description, acceptance checks and
out-of-scope list as given by the owner in the `/speckit-specify` request)

> **Note on technology names.** A few requirements below name parts of the stack: drops, pickups,
> the magic meter, the spell and its damage live on the rules side (Rust); the meter's pips, the
> potion blink, the screen flash and the empty-meter shake are presentation (GDScript) reacting to
> the rules side's signals; potions are found through a scene-tree group, like fighters; the magic
> input is the `p1_magic` action and charges belong to the hero's player index. These are
> constraints from the constitution (the Rust/GDScript split, Principles IV, VII and VIII) and from
> the owner's acceptance check 11, not choices made by this spec. Every *how* (node types, class
> names, data layout, how the freeze is timed) is left to the plan.

## Clarifications

### Session 2026-11-25

- Q: When the hero walks over a potion that would have no effect (a red one at full health, a blue
  one with a full meter), what happens? → A: It stays on the ground for later, until it is
  collected when it can help or its lifetime ends.
- Q: The owner's check 8 names a tuned choice between spell damage per charge and flat damage.
  What should a flat cast do with the charges spent? → A: There is no flat mode: the spell's
  damage always grows with the charges spent (damage per charge × charges). That part of check 8
  is dropped.
- Q: When the hero is hit-stunned or mid-swing and the magic key is pressed, does the spell cast
  at once? → A: Yes. It casts at once from any state while the hero is alive: it cancels a swing
  and breaks a hit-stun, so the spell is an emergency escape.
- Q: Does the spell knock a brute flat like a grunt, despite its armor and knockback resistance?
  → A: Yes. Every enemy, brutes included, is knocked flat for the full knockdown time.
- Q: Do enemies killed by the spell drop potions at the usual chance? → A: Yes, exactly like a
  kill by a blow, blue potions included; a spell that refills itself too often is held in check
  by tuning the drop chances and odds (US4).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Potions drop and heal (Priority: P1)

Some enemies leave a potion where they fall. A red potion is a health potion, a blue one a magic
potion. The hero picks one up by walking over it at about the same lane depth: a health potion
restores a set amount of health, never above the maximum. A potion nobody collects blinks near the
end of its life and then disappears.

**Why this priority**: Health has only gone down since M3; potions are the slice's way to recover,
and magic potions are how the spell is fed. Everything else in this milestone builds on drops.

**Independent Test**: Kill grunts and brutes with F4 off, watching what they leave. Walk over a red
potion while hurt and watch the health bar; walk past one at a clearly different depth; leave one
alone and watch it blink and vanish.

**Acceptance Scenarios**:

1. **Given** the starting values, **When** the hero kills many grunts and many brutes, **Then**
   some of each leave a potion where they fell, brutes clearly more often than grunts, and each
   potion is red or blue by the tuned odds.
2. **Given** a red potion on the ground and the hero below full health, **When** the hero walks
   over it at about its depth, **Then** the potion disappears and the hero's health rises by the
   heal amount, capped at the maximum.
3. **Given** a potion on the ground, **When** the hero walks over its position at a clearly
   different depth, **Then** it is not collected.
4. **Given** a potion nobody collects, **When** its lifetime nears its end, **Then** it blinks, and
   when the lifetime has passed it disappears.

---

### User Story 2 - The magic meter and the spell (Priority: P1)

A blue potion adds one charge of magic, up to a maximum. A meter next to the health bar shows the
charges as pips: filled for each charge held, empty for the rest. Pressing the magic key (K, or X)
with at least one charge casts the spell and uses every charge. The action freezes, the screen
floods with a bright flash, and every enemy on screen takes damage and is knocked flat for a short
time; more charges spent means more damage. The hero cannot be hit while casting. Enemies the spell
kills score, free their turns and count toward clearing the wave, like any kill. Pressing the key
with no charges does nothing but give the empty meter a brief shake, so the player knows why.

**Why this priority**: The spell is the slice's comeback move, and the brief names it as the last
piece of the Ashen Stand. Without it the go or no-go playtest has nothing new to judge.

**Independent Test**: Press F9 to fill the meter and cast in a crowd: everything freezes, the
screen flashes, every enemy is knocked flat and loses health, and the meter is empty. Cast again
with fewer charges and compare the damage. Press the key with an empty meter: the meter shakes and
nothing else happens.

**Acceptance Scenarios**:

1. **Given** the hero holds fewer than the maximum charges, **When** the hero collects a blue
   potion, **Then** one more pip fills; at the maximum, the count stays at the maximum.
2. **Given** the hero holds at least one charge, **When** the magic key is pressed, **Then** every
   charge is used, the action freezes for the cast time, the screen flashes, and every enemy on
   screen takes the spell's damage and is knocked flat for the knockdown time.
3. **Given** two casts with different numbers of charges, **Then** the cast with more charges does
   more damage to each enemy: the damage per charge times the charges spent.
4. **Given** an enemy's attack would land during the cast, **Then** the hero takes no damage from
   it.
5. **Given** the spell kills enemies, **Then** each adds its points and a kill, frees any turn it
   held, may drop a potion, and the wave clears when its last enemy dies this way.
6. **Given** the hero holds no charges, **When** the magic key is pressed, **Then** nothing happens
   except a brief shake of the empty meter.

---

### User Story 3 - Tune it live, and cheat (Priority: P2)

The owner changes drop chances and odds, potion lifetime, heal amount, maximum charges, spell
damage, cast time and knockdown time in the tuning data, presses F5, and sees each change in play.
F9 fills the magic meter in debug builds. R starts a fresh run with an empty meter and no potions
on the ground.

**Why this priority**: Balancing the spell against the waves (US4) needs live tuning and a quick
way to get charges; restart must not leak potions or charges between runs.

**Independent Test**: With the game running, change each new value and press F5; press F9 and
cast; press R with potions on the ground and charges in the meter.

**Acceptance Scenarios**:

1. **Given** the game running, **When** the owner changes a new value in the tuning data and
   presses F5, **Then** "Tuning reloaded" shows and the change takes effect from the next drop,
   pickup or cast as the plan's contract gives.
2. **Given** a bad new value (for example a drop chance of 1.5 or maximum charges of 0), **When**
   the owner presses F5, **Then** the reload is rejected with a message naming the field and the
   previous values stay in use.
3. **Given** a debug build, **When** F9 is pressed during play, **Then** the meter fills to the
   maximum; at Game Over it does nothing.
4. **Given** a release build, **When** F9 is pressed, **Then** nothing happens.
5. **Given** potions on the ground and charges in the meter, **When** R is pressed, **Then** the
   fresh run starts with no potions on the ground and an empty meter.

---

### User Story 4 - The spell is a comeback, not a win (Priority: P1)

The owner plays five runs and retunes the waves, drops and spell where needed until the spell
feels like a way to turn a losing fight, not an easy win: every run is still lost eventually.

**Why this priority**: A spell that clears every wave would end the curve M4a built. The balance
has to hold before the go or no-go playtest can judge the slice.

**Independent Test**: Five runs to Game Over with the final values, no cheats.

**Acceptance Scenarios**:

1. **Given** the final values, **When** the owner plays five runs, **Then** the spell turns at
   least one losing fight per run, and every run still ends in Game Over.

---

### User Story 5 - Go or no-go (Priority: P1)

The owner plays the whole slice for five minutes. The slice passes when the owner then chooses to
play it again three times in a row without being asked. The outcome, go or no-go, is recorded in
the project brief with one sentence on why.

**Why this priority**: This is the decision the slice exists for: whether the project goes on to
art, stages, co-op and beasts (brief, M4b row).

**Independent Test**: The owner's own play session, then the brief.

**Acceptance Scenarios**:

1. **Given** the finished slice, **When** the owner has played it for five minutes, **Then** the
   owner either chooses to replay three times in a row unprompted (go) or does not (no-go).
2. **Given** the decision, **Then** the project brief records "go" or "no-go" with one sentence on
   why.

---

### User Story 6 - Still 60 fps, clean and committed (Priority: P2)

The owner gets ten enemies on screen, casts the spell, plays for a minute on the target machine
with V-Sync on, then runs the Quality Gate and commits.

**Why this priority**: Principle V and the Quality Gate apply to every milestone; the spell's flash
and a full screen of knocked-down enemies are the heaviest moment yet.

**Independent Test**: With F3, F7 and F9, bring ten enemies into play and cast; fight for 60
seconds by the clock (F4 to stay alive), watching the FPS. Then run the Quality Gate and
`git status`.

**Acceptance Scenarios**:

1. **Given** ten enemies on screen on the target machine, **When** the owner fights for one minute
   and casts the spell, **Then** the FPS stays at 60, apart from the first second and the intended
   freezes (impacts and the cast).
2. **Given** the M4b work is finished, **When** the owner runs `cargo fmt --check`, `cargo build`
   and `cargo clippy` and opens the project in the editor, **Then** there are no warnings and no
   GDScript errors or warnings.
3. **Given** the project has been built and played, **When** the owner runs `git status` after
   committing, **Then** the working tree is clean.

---

### Edge Cases

- **A potion the hero cannot use** (a red one at full health, a blue one at maximum charges)
  (clarified): it is not collected; it stays on the ground for later, until the hero can use it or
  its lifetime ends.
- **Several potions under the hero at once**: each is collected, or left if it would have no
  effect, on its own; collecting one never blocks another. Two red potions at once while hurt are
  both collected if the first does not fill the bar; the second stays if it does.
- **Potions across waves**: a potion stays on the ground through breathers and banners until it is
  collected or its lifetime ends.
- **Enemies killed by F8**: they leave no potions, as they score nothing.
- **Training dummies** (F6): they never drop potions. The spell hits them like a blow, without
  knocking them flat, so its damage can be read on them.
- **Casting with no enemies in play** (a breather or a banner): the charges are used and the flash
  plays; nothing is hit.
- **Enemies still entering from the edge**: they are in play, so the spell hits them.
- **A brute caught mid-attack by the spell** (clarified): it is knocked flat for the full knockdown
  time like any enemy; its attack is cancelled. Neither its armor nor its knockback resistance
  shortens or prevents the knockdown.
- **The hero is hit-stunned or mid-swing when the magic key is pressed** (clarified): the cast starts
  at once; a swing in progress is cancelled and a hit-stun ends. A knockback slide in progress stops
  with it.
- **The hero dies**: charges and potions stop mattering; the magic key and F9 do nothing at Game
  Over.
- **R pressed during the cast freeze**: the fresh run starts anyway, with an empty meter and no
  potions.
- **Maximum charges lowered with F5 below the charges held**: the count is cut to the new maximum.
- **An enemy knocked flat**: it cannot attack or be given a turn while down; when it gets up, it
  goes back to its waiting spot like an enemy whose turn ended (M4a).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When an enemy starts dying from a blow or the spell, it MUST drop a potion where it
  fell with the drop chance of its kind (from the data). The starting brute chance MUST be higher
  than the grunt chance.
- **FR-002**: Each dropped potion MUST be a health potion or a magic potion by the tuned odds (from
  the data). The two MUST look clearly different (red for health, blue for magic).
- **FR-003**: The hero MUST collect a potion when the hero's body overlaps it horizontally and
  their depths differ by at most the pickup depth tolerance (from the data), while the hero is
  alive. A potion at a clearly greater depth difference MUST NOT be collected.
- **FR-004**: A health potion MUST restore the heal amount (from the data), never above the hero's
  maximum health. A magic potion MUST add one charge, never above the maximum charges (from the
  data). A potion that would have no effect (a red one at full health, a blue one at maximum
  charges) MUST NOT be collected: it stays on the ground until it can help or its lifetime ends.
- **FR-005**: A potion MUST disappear when its lifetime (from the data) ends, and MUST blink during
  the last part of it (the blink time, from the data).
- **FR-006**: The HUD MUST show a magic meter next to the health bar: one pip per possible charge,
  filled for each charge held. It MUST stay still during the screen shake and work in release
  builds.
- **FR-007**: Pressing the magic action (K, or X) with at least one charge MUST cast the spell at
  once from any state while the hero is alive (free, mid-swing or hit-stunned; a swing is cancelled
  and a stun ended):
  every charge is used; play freezes for the cast time (from the data); the screen flashes; then
  every enemy in play takes the spell's damage and is knocked flat for the knockdown time (from the
  data).
- **FR-008**: The spell's damage to each enemy MUST be the damage per charge (from the data) times
  the charges spent, so more charges always means more damage. There is no flat-damage mode.
- **FR-009**: The hero MUST take no damage from the moment a cast starts until it ends.
- **FR-009a**: The knockdown MUST apply to every kind alike: a brute's armor and knockback
  resistance MUST NOT shorten or prevent it, and an attack in progress (armored or not) MUST be
  cancelled.
- **FR-010**: An enemy knocked flat MUST NOT attack or hold a turn while down; it MUST free any
  turn it held when knocked down, and when it gets up it MUST return to its waiting spot before it
  can be given a turn again (as an enemy whose turn ended, M4a).
- **FR-011**: An enemy killed by the spell MUST count exactly as a kill by a blow: its points, a
  kill, its turn freed, a potion at its kind's usual drop chance and odds (blue included), and
  toward clearing the wave.
- **FR-012**: Pressing the magic action with no charges MUST do nothing but a brief shake of the
  empty meter.
- **FR-013**: In debug builds, F9 MUST fill the meter to the maximum charges during play and do
  nothing at Game Over. In release builds F9 MUST do nothing.
- **FR-014**: R MUST start the fresh run with an empty meter and no potions on the ground, keeping
  M4a's other restart rules (session best kept).
- **FR-015**: The drop chance per kind, the potion odds, the potion lifetime and blink time, the
  pickup depth tolerance, the heal amount, the maximum charges, the spell damage per charge, the
  cast time and the knockdown time MUST be read from the tuning data file, checked like the earlier
  values, and none may appear as a literal value in code. F5 MUST apply them with the M2a reload
  rules (all or nothing). The plan's contract gives when each takes effect.
- **FR-016**: A bad new value MUST be rejected with a message naming the field and why (for
  example a chance outside 0–1, maximum charges below 1, a negative time); the previous values
  stay in use.
- **FR-017**: Drops, pickups, the meter's charges, the spell, its damage and the knockdown MUST be
  decided on the rules side (Rust). The meter's pips, the potion blink, the screen flash and the
  empty-meter shake MUST be presentation (GDScript) reacting to rules-side signals and reading
  rules-side state; it MUST NOT decide any of them.
- **FR-018**: Potions MUST be found through a scene-tree group, like fighters, not a fixed path.
  The magic input MUST be the `p1_magic` action, and charges MUST belong to the hero's player
  index, so a second hero can have its own.
- **FR-019**: M4a's waves, the brute, turns, separation, score and best, M3's fight and M2's
  debug tools MUST keep working as before, apart from the spell's knockdown being a new way an
  enemy stops attacking.

### Key Entities

- **Potion**: a pickup on the ground with a kind (health or magic), a position and depth, and the
  time left before it disappears. It belongs to no fighter.
- **Magic charges**: the number of charges a hero holds, from 0 to the maximum; one meter per hero.
- **Spell cast**: a short frozen moment that uses all of a hero's charges and hits every enemy in
  play once with damage from those charges, then knocks them flat.
- **Knockdown**: a time an enemy spends flat on the ground: no attacks, no turn, then back to its
  waiting spot.
- **Potion and spell tuning**: the new values in FR-015, in the same tuning data file as before.
- **Actions** (extended): new `p1_magic` (K, X) and `debug_refill_magic` (F9).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Over 100 grunt kills and 50 brute kills (F3, F7), the share that drop a potion is
  within 10 points of each kind's tuned chance, brutes drop more often than grunts, and the split
  of red and blue is within 10 points of the tuned odds.
- **SC-002**: In 10 pickups of each kind, health never exceeds the maximum and charges never
  exceed the maximum; in 10 walks past a potion at a depth difference twice the pickup tolerance,
  none is collected; in 10 walks over a potion that would have no effect, it stays on the ground.
- **SC-003**: Every uncollected potion blinks during its last blink time and is gone when its
  lifetime ends, within one frame.
- **SC-004**: In 10 casts with a full meter among at least 5 enemies, every enemy in play is hit
  once and knocked flat, the meter is empty afterwards, and the hero's health does not drop during
  any cast. A 3-charge cast does 3 times the damage of a 1-charge cast.
- **SC-005**: Every spell kill raises the score by the enemy's points, adds a kill and frees its
  turn; a wave whose last enemy dies to the spell clears and the breather starts.
- **SC-006**: In 10 presses with an empty meter, the meter shakes and nothing else changes.
- **SC-007**: F9 fills the meter in a debug build and has no visible effect in a release build,
  where potions, the meter, the spell, waves, score and R still work.
- **SC-008**: Each new value, changed and applied with F5, visibly changes play from the moment the
  contract gives; each bad value is rejected with a message naming it. After R, there are no
  potions on the ground and the meter is empty, every time out of 10.
- **SC-009**: With ten enemies on screen on the target machine, the FPS stays at 60 for 60 seconds
  of fighting that includes a cast, apart from the first second and the intended freezes.
- **SC-010**: `cargo fmt --check`, `cargo build` and `cargo clippy` produce 0 warnings, the editor
  shows 0 GDScript errors or warnings, and `git status` is clean after committing.
- **SC-011**: After five runs with the final values, the owner judges that the spell turned losing
  fights and that every run still ended in Game Over. Without that judgement the milestone is not
  done.
- **SC-012**: The go or no-go outcome is recorded in the project brief with one sentence on why.

## Assumptions

- **Target hardware is the Intel UHD 630.** The owner's check 12 says "UHD 620", but constitution
  1.3.2 corrected the target to the UHD 630, M3 and M4a were verified on it, and the brief says
  UHD 630. Check 12 is read as the UHD 630.
- **"Every enemy on screen"** means every enemy in play, including those still entering from an
  edge; enemies waiting in a wave's queue off screen are not yet in play and are not hit.
- **The cast freeze** stops everything the impact freeze stops (fighters, waves, potion lifetimes);
  R still works during it.
- **The magic meter starts empty** at the start of every run (check 8).
- **Drop odds**: one red/blue split for all kinds; only the drop chance differs by kind.
- **Check 8's flat-damage choice** is dropped (clarified): the damage always scales with the
  charges spent, as check 4 says.
- **No-go still finishes the milestone.** Check 10 makes the replaying the test of "go"; if the
  owner does not replay, the outcome "no-go" is recorded with its reason and the milestone is still
  complete, since the decision is its deliverable.
- **Starting values** (data, not requirements; the owner retunes them): grunt drop chance about
  0.15, brute about 0.5; red/blue about 60/40; potion lifetime about 8 s, blinking for the last
  2 s; pickup depth tolerance as the hit depth tolerance (about 20 px); heal 30 of 100; maximum
  charges 3; spell damage per charge about 20 (a 1-charge cast nearly kills a grunt, 3 charges kill
  one and badly hurt a brute); cast time about 0.6 s; knockdown about 1.2 s.
- **The brute-attack feel** the owner noticed in M4a (being locked in a swing when an armored attack
  lands) is not changed here; the spell and potions are the planned answers to losing fights.
- **The Project Brief** ([docs/project-brief.md](../../docs/project-brief.md)) lists M4b as
  "Potion drops, the magic meter and spell, the refill-magic cheat", checked by "A five-minute
  playtest of the whole slice is fun enough to replay three times; go or no-go decision". The
  owner's description matches.
- **Acceptance check mapping**: 1 → US1 sc. 1 / FR-001, FR-002 / SC-001; 2 → US1 sc. 2–3, US2
  sc. 1 / FR-003, FR-004 / SC-002; 3 → US1 sc. 4 / FR-005 / SC-003; 4 → US2 sc. 2–3 / FR-007,
  FR-008 / SC-004; 5 → US2 sc. 4, 6 / FR-009, FR-012 / SC-004, SC-006; 6 → US2 sc. 5 / FR-010,
  FR-011 / SC-005; 7 → US3 sc. 3–4 / FR-013 / SC-007; 8 → US3 sc. 1–2, 5 / FR-014–FR-016 /
  SC-008; 9 → US4 / SC-011; 10 → US5 / SC-012; 11 → FR-017, FR-018; 12 → US6 sc. 1 / SC-009;
  13 → US6 sc. 2–3 / SC-010.
