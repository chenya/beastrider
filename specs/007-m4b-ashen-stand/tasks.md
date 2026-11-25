---

description: "Task list for M4b Ashen Stand"
---

# Tasks: M4b Ashen Stand

**Input**: Design documents from `specs/007-m4b-ashen-stand/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: Rust unit tests are included, because the owner listed them in the plan input
([research.md R15](research.md)): `spend_all` returns and clears every charge; damage in both
modes; casting allowed from `Free`, `Attacking` and `HitStun` but not `Dead`; an empty cast changes
nothing; knockdown ignores `knockback_resistance`; knocked-down enemies ignore blows and free their
tokens; seeded drop rolls give the expected kinds; pickup respects the depth tolerance; health and
charges clamp at the maximum; lifetimes pause during the cast freeze; off-screen enemies are
untouched; plus every new tuning rule and the warning. They are written before the code they test.
Everything else is verified by windowed check scripts and by playing ([quickstart.md](quickstart.md)).

**Organization**: Tasks are grouped by user story. Phases follow the spec's order, except that the
two playtests come last: US4 (balance) needs US3's live tuning and F9, and US5 (go or no-go) judges
the finished slice, so it runs after US6's frame-rate and gate checks.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on unfinished tasks)
- **[Story]**: Which user story the task belongs to (US1–US6 from spec.md)
- **(owner)**: Needs the owner: the Godot editor GUI, or a judgement made by playing. The owner may
  hand an editor task to the agent, which then uses a headless Godot script or small text edits
  and `ProjectSettings` (`event.device = -1`), as recorded in the plan's Complexity Tracking.

## Path Conventions

`rust/` (the crate) and `godot/` (the project), as before. Run Rust commands from `rust/`. Values
in tests must differ from those in `godot/data/tuning.json` (see
[contracts/tuning-data.md](contracts/tuning-data.md)), so the magic-number check in T041 stays
meaningful. Size- and focus-dependent checks need a windowed 1280×720 run, not `--headless` (a
headless run reports a 64 px screen and rejects `lane.bottom`). Never put a script on a Rust-class
root node (`Potion`, `Enemy`, `Hero`, `TrainingDummy`, `RunTracker`, `WaveDirector`): a GDScript
`_ready` replaces the Rust `ready()` (M2b research R6). Signal handlers use their arguments and
never call back into the emitter during the emit (M3 research R8). Behaviour code never matches on
`EnemyKind` (M4a research R2).

## Session plan (about 15 h/week, Principle II)

- **Session 1**: Phases 1–3 (actions, tuning, `Health::heal`, `MagicMeter`, the cast freeze in
  `Run`, then US1 potions with the drop roll). Ends playable: enemies drop red and blue potions that the hero
  collects.
- **Session 2**: Phases 4–5 (US2 the spell, US3 live tuning, F9 and the release build). Ends with
  the whole slice playable.
- **Session 3**: Phases 6–9 (US4 balance, US6 frame rate and gate, US5 go or no-go, polish and the
  commit).

If a session stalls, shrink the next step, as the constitution's Development Workflow says (for
example, a cast with no freeze before adding the freeze).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm nothing was upgraded and add the two new actions.

- [ ] T001 Confirm nothing was upgraded (research.md R1): `godot --version` prints `4.7.2.stable.official.ed1daf0bf`; `rust/Cargo.toml` still has `godot = { version = "=0.5.5", features = ["api-4-6"] }`; `rust/rust-toolchain.toml` still pins `1.100.0`; `cargo build` and `cargo test` in `rust/` pass (109 tests). If anything differs, stop and ask the owner
- [ ] T002 (owner) In Project Settings → Input Map, bound to **physical** keys, deadzone 0.2, saved to `godot/project.godot`: add `p1_magic` = K and X, and `debug_refill_magic` = F9 ([contracts/magic-classes.md](contracts/magic-classes.md#input-map)). Confirm each event has `"device":-1` and `git diff godot/project.godot` shows nothing else. Neither key does anything until US2 (K, X) and US3 (F9)

**Checkpoint**: The Input Map has the M4b actions.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The extended tuning file, `Health::heal`, the magic meter and the cast freeze in `Run`,
all under tests. Potions and the spell both need them.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete. At the end the game
plays exactly as M4a.

- [ ] T003 [P] Write the tuning tests first in the `#[cfg(test)] mod tests` of `rust/src/tuning.rs` (they will not compile until T004). Extend the test `raw()` helper with `drop_chance` per kind, `potions` and `magic`, with values different from the file's. One passing and one failing case per new rule in [contracts/tuning-data.md](contracts/tuning-data.md#rules): `enemies.<kind>.drop_chance` "`0 <= c <= 1`" (pass 0 and 1, fail −0.1 and 1.1); `potions.health_odds` "`0 <= o <= 1`"; `potions.lifetime` "`> 0`"; `potions.blink_time` "`0 <= b <= potions.lifetime`" (fail lifetime + 0.1); `potions.heal_amount`, `potions.width`, `potions.height` "`> 0`"; `magic.max_charges` "a whole number, `>= 1`" (fail 0 and 1.5); `magic.damage_mode` "`\"per_charge\"` or `\"flat\"`" (fail `"big"` and a number); `magic.damage_per_charge`, `magic.flat_damage` "`> 0` (both always required)"; `magic.cast_time`, `magic.knockdown_time`, `magic.knockdown_push_time` "`> 0`"; `magic.knockdown_push_distance` "`>= 0`". Warning test: `BruteDropsNotMore` when `enemies.brute.drop_chance <= enemies.grunt.drop_chance` (equal and smaller), none when higher; such values still pass `validate`
- [ ] T004 In `rust/src/tuning.rs`, extend per [contracts/tuning-data.md](contracts/tuning-data.md) and [data-model.md](data-model.md#tuning): key constants for `drop_chance`, the `potions` and `magic` sections; `EnemyTuning` gains `drop_chance()`; new `Copy` structs `PotionTuning { health_odds, lifetime, blink_time, heal_amount, width, height }` and `MagicTuning { max_charges: u32, damage_mode: SpellDamageMode, damage_per_charge, flat_damage, cast_time, knockdown_time, knockdown_push_distance, knockdown_push_time }` with accessors, reached by `TuningValues::potions()` and `magic()`; `SpellDamageMode { PerCharge, Flat }` declared in `rust/src/magic.rs` (created here with only the enum and `//!` docs) and read by a new string-choice reader with the message of the contract; rules in file order, all or nothing; `TuningWarning::BruteDropsNotMore { grunt, brute }` with the contract's message. Add the contract's values to `godot/data/tuning.json`, keeping every M4a value. Add `mod magic;` to `rust/src/lib.rs`. Run `cargo test` until T003 passes (depends on T003)
- [ ] T005 [P] In `rust/src/fighter.rs`, write tests first, then add to `Health` ([data-model.md](data-model.md#health-extended-rustsrcfighterrs)): `heal(amount) -> f32` "while alive, `current = min(current + amount, max)`; returns the health gained (0 when dead or full)" and `is_full()` "`current >= max`". `///` docs. `cargo test`
- [ ] T006 [P] In `rust/src/magic.rs`, write tests first, then the pure `MagicMeter { charges: u32, max: u32 }` ([data-model.md](data-model.md#magicmeter-rustsrcmagicrs-pure)): `new(max)` "`charges = 0`"; `add_charge() -> bool` "`charges < max`: `charges + 1`, true; else unchanged, false"; `refill()` "`charges = max`"; `spend_all() -> u32` "returns `charges`, leaves 0"; `set_max(max)` "`charges = min(charges, max)`"; `is_full()`, `is_empty()`. Tests: spending returns every charge and leaves 0; adding stops at the maximum; refill fills; a lower maximum cuts the charges (depends on T004)
- [ ] T007 [P] In `rust/src/run.rs`, write tests first, then add ([data-model.md](data-model.md#run-extended), research R5): a cast-freeze count with Rust-side `begin_cast()`, `end_cast()` (never below 0) and `#[func] is_cast_frozen()` "`> 0`", reset to 0 by `restart()`. Tests on the pure count: two begins need two ends; an end with none in progress stays at 0 (depends on T004)
- [ ] T008 Confirm with windowed Godot script runs (1280×720) that the game plays exactly as M4a: tuning loads with the new file and no warning; a wave comes, a grunt and a brute fight and die, score and Game Over work, R restarts; F5 rejects `magic.max_charges` 0 naming that path. Then `cargo fmt --check`, `cargo clippy --all-targets` (0 warnings except dead code used in later phases) and `cargo test` (depends on T004–T007)

**Checkpoint**: Tuning extended and tested, `Health::heal`, `MagicMeter` and the cast freeze exist;
the game still plays as M4a.

---

## Phase 3: User Story 1 – Potions drop and heal (Priority: P1) 🎯 MVP

**Goal**: Enemies leave red and blue potions; the hero collects useful ones at its depth; the rest
blink and vanish.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 1–5.

- [ ] T009 [P] [US1] Write the potion tests first in a new `rust/src/potion.rs` `#[cfg(test)] mod tests` (they will not compile until T010), per [data-model.md](data-model.md#drop-roll-rustsrcpotionrs-pure): `drop_from_rolls(chance, health_odds, roll_a, roll_b)` "`roll_a < chance` → a drop; then `roll_b < health_odds` → `Health`, else `Magic`" (chance 0 never drops, chance 1 always does; odds 0 gives `Magic`, odds 1 `Health`; rolls just under and at the limits); `can_collect(hero_box, hero_feet_y, potion_box, potion_feet_y, depth_tolerance)` true when the boxes overlap horizontally and `|hero_feet_y − potion_feet_y| <= depth_tolerance`, false at twice the tolerance and when the boxes don't overlap; `potion_useful(kind, health_full, meter_full)` "`Health` → not `health_full`; `Magic` → not `meter_full`"; a pure lifetime step that does not count down while frozen (`HitStop` or the cast) and counts down otherwise; `is_blinking` "`life <= potions.blink_time`"
- [ ] T010 [US1] Implement `rust/src/potion.rs` so T009 passes (research R8, [contracts/magic-classes.md](contracts/magic-classes.md#potion-new-extends-node2d)): `PotionKind { Health, Magic }` with `#[derive(GodotConvert, Var, Export)]` and `#[godot(via = GString)]`; `pub const POTIONS_GROUP: &str = "potions"`; the pure functions of T009; `#[class(base = Node2D)] pub struct Potion` with `#[export] kind`, joining `POTIONS_GROUP` in `ready()`, handles to `Tuning`, `HitStop`, `Run` (missing ⇒ one `godot_error!("Potion: …")`); Rust-side `place(kind, position)`; each physics tick, unless `HitStop.is_frozen()` or `Run.is_cast_frozen()`: start `life` at `potions.lifetime` on the first tick with tuning, count it down, `queue_free()` at 0; `#[func]`s `kind() -> GString`, `is_blinking()`, `pickup_box() -> Rect2` ("`potions.width × potions.height` centred over the feet"). Add `mod potion;` to `rust/src/lib.rs`. `cargo test`, `cargo clippy --all-targets` (depends on T008, T009)
- [ ] T011 [US1] In `rust/src/run.rs` (research R10): a `RandomNumberGenerator` `randomize()`d in `init` and again in `restart()`, and Rust-side `roll_drop(chance, health_odds) -> Option<PotionKind>` drawing two numbers in [0, 1) and returning `drop_from_rolls(...)`. In `rust/src/enemy.rs`: a field `drop: Option<PotionKind>`; on entering `Dying` by any path (today: `take_hit`), `drop = run.bind_mut().roll_drop(own.drop_chance(), values.potions().health_odds())`; Rust-side `take_drop() -> Option<(PotionKind, Vector2)>` that empties it with the feet position. In `rust/src/wave_director.rs`: `#[export] potion_scene: Option<Gd<PackedScene>>` (missing ⇒ the existing ready-time error lists it); after the enemies tick, for each `Enemy` child of `world` with a drop, instance the potion, `place(kind, position clamped to the lane)`, add it to `world` before the hero (as enemies are); `skip_wave` frees enemies without a death, so no drop. `cargo test`, `cargo clippy --all-targets` (depends on T010)
- [ ] T012 [US1] In `rust/src/hero.rs` (research R9): a field `magic: Option<MagicMeter>` created with `magic.max_charges` on the first tick with tuning and `set_max` each tick after; each tick the hero steps (not while `Dead`): for every node in `POTIONS_GROUP` cast to `Potion`, collect it when `can_collect(...)` with the hero's hurtbox, its feet, the potion's `pickup_box()` and feet and `lane.depth_tolerance`, and `potion_useful(kind, health.is_full(), magic.is_full())`: `Health` → `health.heal(potions.heal_amount)`; `Magic` → `add_charge()` and emit `magic_changed(charges, max)`; then emit `potion_collected(kind)` and `queue_free()` the potion. Declare the signals `magic_changed(charges: i64, max: i64)` and `potion_collected(kind: GString)` and `#[func]`s `charges()`, `max_charges()` (0 before tuning). `cargo test`, `cargo clippy --all-targets` (depends on T006, T010)
- [ ] T013 [P] [US1] Create `godot/world/potion_body.gd` (`extends Polygon2D`, `##` docs, typed): reads its parent `Potion`; in `_ready` builds the flask polygon from named constants or the parent's `pickup_box()` once tuning is there; each frame sets `color` to a named `HEALTH_COLOR` (red) or `MAGIC_COLOR` (blue) by `kind()`, and while `is_blinking()` hides itself every other `BLINK_PERIOD` (a named presentation constant). No game state (research R11) (depends on T010)
- [ ] T014 [P] [US1] In `godot/debug/debug_hitboxes.gd`, also outline every node in the `potions` group by its `pickup_box()`, in a named colour, while F2 is on (depends on T010)
- [ ] T015 [US1] (owner) Create `godot/world/potion.tscn` ([contracts/magic-classes.md](contracts/magic-classes.md#scenes)): root `Potion` named `Potion`, **no script**; child `Body` (`Polygon2D`) with `res://world/potion_body.gd`. In `godot/main.tscn`, set `WaveDirector.potion_scene` → `res://world/potion.tscn`. Save (depends on T011, T013)
- [ ] T016 [US1] Confirm by windowed script runs: with `enemies.grunt.drop_chance` set to 1 and `potions.health_odds` 1, then 0, each grunt death leaves a red, then a blue potion at its feet, inside the lane; over 200 deaths at the starting values the shares are within 10 points of 0.15 / 0.5 and 60 / 40; a hurt hero collects a red potion at its depth (health +30, capped at 100) and leaves one 40 px above or below; a full-health hero leaves a red potion; a blue potion adds a charge up to 3 and stays at 3; an untouched potion blinks for its last 2 s and is gone at 8 s; frozen time (`HitStop`) does not age it; F8 leaves no potions. Restore `godot/data/tuning.json`. No errors (depends on T012, T014, T015)
- [ ] T017 [US1] (owner) Press Play and run quickstart scenarios 1–5. Fix anything that fails before going on (depends on T016)

**Checkpoint**: Potions drop, heal and time out. Blue potions fill a meter nobody can see or spend
yet. M4b's MVP is playable.

---

## Phase 4: User Story 2 – The magic meter and the spell (Priority: P1)

**Goal**: A meter of pips, a cast that freezes play, flashes the screen and knocks every enemy on
screen down, and a shake when the meter is empty.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 6–14.

- [ ] T018 [P] [US2] Write tests first in `rust/src/magic.rs` (they will not compile until T020): `spell_damage(mode, per_charge, flat, charges)` "`per_charge × charges` | `flat`" (per charge: 1 and 3 charges give 1× and 3×; flat: the same for 1 and 3); `on_screen(feet, screen)` true inside, false for feet beyond each edge
- [ ] T019 [P] [US2] Write tests first in `rust/src/enemy.rs` `#[cfg(test)] mod tests` (they will not compile until T021), per [data-model.md](data-model.md#enemy-extended): `enemy_after_spell(mode, damage, knockdown_time, fade)`: `Dying` unchanged; `Killed` in any other mode (`Entering` included) → `Dying(fade)`; `Survived` in any other mode → `KnockedDown(knockdown_time)` with the attack dropped, for a resistance of 0.75 and for `interruptible = false` as well as for 0 and `true`; `AlreadyDead` unchanged. `KnockedDown` does not hold a turn; its crowd give is `Yields`; when its countdown ends the next mode is `Retreating`
- [ ] T020 [US2] Implement in `rust/src/magic.rs` so T018 passes: `spell_damage` and `on_screen`, with `///` docs (research R3, R6). `cargo test` (depends on T018)
- [ ] T021 [US2] In `rust/src/enemy.rs` (research R7) so T019 passes: `EnemyMode::KnockedDown(Countdown)` in every exhaustive `match` (`holds_turn` false, `is_alive` true, `is_waiting` false, `crowd_body` `Give::Yields`, `turn_request` none); while down its knockback slides and its countdown runs, then `Retreating`; `target()` reports `invulnerable: true` while down, so no blow lands; `enemy_after_spell`; Rust-side `take_spell(damage, from_x)`: health drops; on `Killed` the same path as a blow kill (leave the group, `died`, `add_kill(points)`, the drop roll of T011); on `Survived` `KnockedDown` and a push away from `from_x` of `magic.knockdown_push_distance` over `magic.knockdown_push_time` through `Knockback::start`, ignoring resistance and armor; emit `was_hit(direction, strength)`; `#[func] is_knocked_down()`. Skip the physics step while `Run.is_cast_frozen()` as well as `HitStop`. `cargo test`, `cargo clippy --all-targets` (depends on T011, T019)
- [ ] T022 [P] [US2] In `rust/src/training_dummy.rs`, skip the step while `Run.is_cast_frozen()` too, and add a Rust-side `take_spell(damage)` that takes the damage as from a blow (health, flash via its existing signal) with no knockdown. In `rust/src/wave_director.rs`, skip the tick while `Run.is_cast_frozen()` too (depends on T007)
- [ ] T023 [US2] In `rust/src/hero.rs` (research R4–R6, [data-model.md](data-model.md#hero-extended)), tests first in its `#[cfg(test)] mod tests`: `can_cast(mode)` "`Free | Attacking | HitStun` → true; `Casting | Dead` → false". Then: `PlayerActions` gains `magic` (`p{index}_magic`); `HeroMode::Casting(Cast)` with `Cast { timer: Countdown, charges: u32 }` in every exhaustive `match`; on `magic` pressed, alive and `can_cast`: with charges, `spend_all`, drop the swing (and its `hit_this_swing`) or the stun, stop any knockback slide, `Run.begin_cast()`, `HitStop.clear()`, emit `cast_started(charges)` and `magic_changed`, enter `Casting(magic.cast_time)`; with none, emit `cast_failed_empty()` and change nothing. While `Casting`: no move or turn, `target()` invulnerable, the post-hit countdown keeps its own time. When the cast ends: damage = `spell_damage(...)`; for each `FighterRef` in `team_enemies` whose feet are `on_screen` in `get_viewport_rect()`, `Enemy::take_spell(damage, hero_x)` or `TrainingDummy::take_spell(damage)`; then `Run.end_cast()`, emit `spell_impact()`, `Free`. The hero skips its step while `HitStop` is frozen (as before) and while `Run.is_cast_frozen()` **unless it is casting**. Signals `cast_started(charges: i64)`, `spell_impact()`, `cast_failed_empty()`; `#[func]`s `is_casting()`, `cast_progress()` (0 to 1), and `refill_magic()` (the meter to full and `magic_changed`; nothing at Game Over or before tuning; F9 calls it from US3). `cargo test`, `cargo clippy --all-targets` (depends on T012, T020, T021, T022)
- [ ] T024 [P] [US2] Create `godot/ui/magic_meter.gd` (`extends Control`, `##` docs): `## The hero whose charges are shown.` `@export var hero: Hero`; each frame `queue_redraw()`; `_draw` draws `hero.max_charges()` pips in a row (named `PIP_SIZE`, `PIP_GAP`, `FILLED_COLOR`, `EMPTY_COLOR`), filled up to `hero.charges()`; on `hero.cast_failed_empty` starts a horizontal shake for a named `SHAKE_TIME` (presentation state only). Handlers call nothing back on the hero (depends on T023)
- [ ] T025 [P] [US2] Create `godot/ui/spell_flash.gd` (`extends ColorRect`, `##` docs): `@export var hero: Hero`; full screen, `mouse_filter` ignore; each frame, while `hero.is_casting()`, alpha follows `hero.cast_progress()` up to a named `PEAK_ALPHA`; on `hero.spell_impact` it holds the peak and fades over a named `FADE_TIME`; otherwise transparent (depends on T023)
- [ ] T026 [P] [US2] In `godot/world/enemy_body.gd`, show a flattened shape (for example scale `(1.4, 0.35)` from named constants) while `enemy.is_knocked_down()`, and the normal shape otherwise (depends on T021)
- [ ] T027 [US2] (owner) In `godot/main.tscn` ([contracts/magic-classes.md](contracts/magic-classes.md#scenes)): add a `Control` `MagicMeter` to `Hud` beside the health bar (for example at (264, 16)) with `res://ui/magic_meter.gd`, `hero` → `World/Hero`; add a `CanvasLayer` `SpellFlash` (layer between `Hud` and `GameOver`) with a full-screen `ColorRect` child using `res://ui/spell_flash.gd`, `hero` → `World/Hero`. Save (depends on T024, T025)
- [ ] T028 [US2] Confirm by windowed script runs, filling the meter with `hero.refill_magic()`: a cast with charges empties the meter, `cast_started(n)` and then `spell_impact()` fire once, `Run.is_cast_frozen()` is true for `magic.cast_time` and enemies, the director and potions don't move or age during it; every enemy with feet on screen loses `damage_per_charge × n` (20 and 60 at the test values, read from health) and goes down for `knockdown_time`, a brute too, mid-attack included; an enemy with feet beyond the edge is untouched; blows don't land on a downed enemy; a downed enemy frees its turn and, after getting up, retreats before taking a turn; a cast from mid-swing and from hit-stun starts at once; the hero's health does not drop during a cast while a strike would land; spell kills add points, kills and may drop potions, and a wave cleared by the spell goes to its breather; an empty press emits `cast_failed_empty()` and changes nothing else. No errors (depends on T023, T026, T027)
- [ ] T029 [US2] (owner) Press Play and run quickstart scenarios 6–14 (depends on T028)

**Checkpoint**: The full Ashen Stand slice plays: potions, the meter, the spell.

---

## Phase 5: User Story 3 – Tune it live, and cheat (Priority: P2)

**Goal**: Every new value reloads live; F9 fills the meter in debug builds; R clears potions and
charges.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 15–19.

- [ ] T030 [US3] In `godot/debug/debug_overlay.gd`, `debug_refill_magic` calls `hero.refill_magic()` and marks the event handled; update the `##` docs listing the keys (F9 fills the magic meter) (depends on T023)
- [ ] T031 [US3] Confirm by windowed script runs: each new value in [contracts/tuning-data.md](contracts/tuning-data.md) changed and reloaded with F5 takes effect at the moment the contract gives (drop chance and odds the next death; lifetime for potions dropped after; blink time and size the next tick; heal the next pickup; maximum charges the next tick, cutting charges above it; damage mode and amounts and knockdown values the next impact; cast time the next cast); the bad values of quickstart scenario 16 are each rejected naming the field; `enemies.brute.drop_chance` 0.1 is accepted with the warning and restoring clears it; F9 fills the meter during play and does nothing at Game Over; R with potions down and charges held starts a run with neither, and R during a cast freeze clears it. Restore `godot/data/tuning.json` afterwards (depends on T030)
- [ ] T032 [US3] Build and export a release (`cd rust && cargo build --release`, then `godot --headless --path godot --export-release Linux export/beastrider.x86_64`); confirm `godot/export/` holds the binary, `.pck` and `libbeastrider.so` and that a 5-second windowed run prints no errors (depends on T031)
- [ ] T033 [US3] (owner) Quickstart scenarios 15–19: retune live, the bad values, the warning, F9 and R, and the release build (potions, the meter, the spell, waves, score and R work; F1–F9 do nothing) (depends on T031, T032)

**Checkpoint**: The slice can be balanced live; cheats exist in debug builds only.

---

## Phase 6: User Story 4 – The spell is a comeback, not a win (Priority: P1)

**Goal**: The owner's judgement that the spell turns losing fights and runs still end.

**Independent Test**: [quickstart.md](quickstart.md) scenario 21.

- [ ] T034 [US4] (owner) Play and retune `godot/data/tuning.json` with F5 (drops, odds, heal, charges, damage, cast and knockdown values, and the waves where needed), using F2, F3, F4, F7, F8 and F9. Then play five runs to Game Over with the final values and judge (SC-011): did the spell turn losing fights, and did every run still end in Game Over? If not, keep retuning (depends on T033)
- [ ] T035 [US4] Record the final values: confirm `godot/data/tuning.json` holds them, passes the rules and gives no warning; update the JSON and the starting-values sentence in `specs/007-m4b-ashen-stand/contracts/tuning-data.md` (and M4a's values in `specs/006-m4a-waves-and-brute/contracts/tuning-data.md` only if the waves were retuned, noting it there) (depends on T034)

**Checkpoint**: The owner says the spell is a comeback, not a win.

---

## Phase 7: User Story 6 – Still 60 fps, clean and committed (Priority: P2)

**Goal**: 60 fps with ten enemies and a cast; the Quality Gate is green.

**Independent Test**: [quickstart.md](quickstart.md) scenarios 20, 24, 25 and 27.

- [ ] T036 [US6] (owner) Quickstart scenario 20: F3 and F7 to ten enemies, F2 on, panel open; fight for 60 s by the clock with at least one cast (F9), F4 to stay alive, window focused. FPS stays 60 apart from the first second, the impact freezes and the cast (depends on T033)
- [ ] T037 [US6] In `rust/`, run `cargo fmt --check`, `cargo build`, `cargo clippy --all-targets` and `cargo test`; fix every warning in `rust/src/*.rs` until all pass with 0 warnings. No `#[allow]` without a written reason
- [ ] T038 [US6] Check every GDScript file (`godot/world/hit_flash.gd`, `hero_body.gd`, `enemy_body.gd`, `enemy_health_bar.gd`, `potion_body.gd`, `screen_shake.gd`, `lane_band.gd`, `godot/ui/fps_label.gd`, `health_bar.gd`, `kills_label.gd`, `score_label.gd`, `wave_banner.gd`, `game_over.gd`, `magic_meter.gd`, `spell_flash.gd`, `godot/debug/debug_overlay.gd`, `debug_markers.gd`, `debug_hitboxes.gd`) through the editor's language server (a headless editor with `--lsp-port` if the editor is closed, with a negative control as in M4a's T044): 0 errors or warnings, `untyped_declaration` still Error

**Checkpoint**: The Quality Gate is green.

---

## Phase 8: User Story 5 – Go or no-go (Priority: P1)

**Goal**: The decision the slice exists for, recorded in the brief.

**Independent Test**: [quickstart.md](quickstart.md) scenario 22.

- [ ] T039 [US5] (owner) Play the whole slice for five minutes with no cheats. Go if you then choose to play it again three times in a row without being asked; otherwise no-go. Tell the agent the outcome and one sentence on why (depends on T035, T036)
- [ ] T040 [US5] Record the outcome (SC-012): in `docs/project-brief.md` and the live brief (the doc "Beastrider: Project Brief and Milestone Plan"), the M4b decision "go" or "no-go" with the owner's sentence on why, worded the same in both (depends on T039)

**Checkpoint**: The go or no-go decision is recorded.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Constitution review, docs and the milestone commit.

- [ ] T041 [P] Check for magic numbers (Principle IV): none of the values in `godot/data/tuning.json` appears as a literal in `rust/src` (outside tests, whose values must differ) or in `godot/world`, `godot/ui` and `godot/debug` scripts; the only numbers in scripts are named presentation constants
- [ ] T042 [P] Check the mode rules (FR-017): `grep -rnE 'is_casting: bool|knocked_down: bool|casting: bool|frozen: bool' rust/src` returns nothing; every `match` on `HeroMode`, `EnemyMode`, `PotionKind`, `SpellDamageMode` has no `_ =>` arm; `grep -rn 'EnemyKind::' rust/src` outside tests shows nothing new beyond M4a's uses
- [ ] T043 [P] Check the split (FR-017) and co-op (FR-018): no `.gd` file decides a drop, a pickup, a charge, a cast, its damage or a knockdown (they only read Rust and react to `magic_changed`, `cast_started`, `spell_impact`, `cast_failed_empty`, `potion_collected`, `was_hit`); potions are found only through `POTIONS_GROUP`; the magic action is built from the player index; `grep -rnE 'KEY_|keycode' godot/world godot/ui godot/debug` returns nothing; `grep -rn '"p1_' rust/src` returns nothing
- [ ] T044 [P] Review docs: `//!` module docs on `rust/src/magic.rs` and `potion.rs`, and updated on `hero.rs`, `enemy.rs`, `run.rs`, `tuning.rs` (pointing at `specs/007-m4b-ashen-stand/contracts/tuning-data.md`); `///` on every public item and everything exposed to Godot, including the new signals; `// SAFETY:` in `rust/src/lib.rs` unchanged; `##` docs on every new script and exported property
- [ ] T045 Run the Constitution compliance review required by Governance: check the finished work against Principles I–VIII (v1.3.3) and the Code Style section, using the table in `specs/007-m4b-ashen-stand/plan.md` as the checklist; record any departure (for example scenes built by the agent) in that file's Complexity Tracking
- [ ] T046 After a build, a Play and the export, run `git status`: `rust/target/`, `godot/.godot/` and `godot/export/` absent; `magic.rs` and `potion.rs` present; the new `.gd` files with their `.uid` files, `godot/world/potion.tscn`, `godot/main.tscn`, `godot/project.godot` and `godot/data/tuning.json` present to be committed
- [ ] T047 (owner) Run the full quickstart.md, scenarios 1–27, in one sitting on the target machine; scenario 26 re-checks M4a
- [ ] T048 Commit the M4b implementation on branch `feat/m4b-ashen-stand` in small logical commits that each pass the gate on their own staged tree (for example: input actions; the game code; the go or no-go record and docs), staging by explicit path and reviewing `git diff --staged` before each, as the owner's git rules require. Ask the owner before committing; then `git status` must be clean (depends on T034–T047)
- [ ] T049 After the merge, update the status line in `docs/project-brief.md` and the live brief (M4b done, with the go or no-go outcome, and what comes next) on the next branch, as was done after M4a

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies. T002 is needed before T023 (K, X) and T030 (F9) can be tried.
- **Foundational (Phase 2)**: Needs T001. Blocks every story. Ends playing as M4a.
- **US1 (Phase 3)**: Needs Phase 2. The MVP: potions.
- **US2 (Phase 4)**: Needs US1 (blue potions fill the meter; spell kills drop potions).
- **US3 (Phase 5)**: Needs US2 (every value and F9 in place).
- **US4 (Phase 6)**: Needs US3 (live tuning and F9 to judge with).
- **US6 (Phase 7)**: T036 needs US3; T037–T038 need all code.
- **US5 (Phase 8)**: Needs US4 and T036: the finished, balanced slice.
- **Polish (Phase 9)**: Needs all stories.

### Within Each Phase

- Tests before the code they test (T003 → T004, T009 → T010, T018 → T020, T019 → T021; T005,
  T006, T007 and T023 write their tests first inside the task).
- Rust classes registered (T010) before scenes use them (T015).
- Scripts before the scenes that use them (T013 → T015; T024, T025 → T027).
- Windowed checks, then the owner's play checks, last in each story.

### Parallel Opportunities

- **Phase 2**: T003 first; then T005, T006 and T007 together (different files).
- **US1**: T009 first; T013 and T014 together once T010 builds.
- **US2**: T018 and T019 together; T022 alongside T020–T021; T024, T025 and T026 together once T023
  builds.
- **Polish**: T041–T044 in parallel.

---

## Parallel Example: User Story 2

```bash
# Tests first, together (different files):
Task: "Write spell_damage and on_screen tests in rust/src/magic.rs"
Task: "Write enemy_after_spell and KnockedDown tests in rust/src/enemy.rs"

# Once the hero casts:
Task: "Create godot/ui/magic_meter.gd"
Task: "Create godot/ui/spell_flash.gd"
Task: "Flatten the knocked-down enemy in godot/world/enemy_body.gd"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: actions. 2. Phase 2: tuning, `Health::heal`, `MagicMeter`, the cast freeze (game still
plays as M4a).
3. Phase 3: US1. Potions drop, heal and time out.
4. **STOP and VALIDATE**: quickstart 1–5. Health potions already change how a run feels.

### Incremental Delivery

1. Setup + Foundational → tests pass, M4a unchanged
2. US1 → potions → MVP (session 1)
3. US2 → the meter and the spell → the whole slice
4. US3 → live tuning, F9, release build (session 2)
5. US4 → the spell is a comeback, by the owner's judgement
6. US6 → 60 fps, gate green
7. US5 → go or no-go, recorded
8. Polish → committed (session 3) → M4b done

### If a Session Stalls

Shrink the next step (constitution, Development Workflow): for example, potions that only heal
before magic potions; a cast with no freeze before adding the freeze; damage with no knockdown
before adding `KnockedDown`. Add each piece behind its tuning value. Never put a tuning value in
code as a shortcut. Every checkpoint is a safe place to stop.

---

## Notes

- [P] = different files, no dependency on unfinished tasks.
- (owner) tasks need the Godot editor GUI or a judgement made by playing.
- Commit only at the end, with the owner's go-ahead (T048). Rust parts that are unused until a
  later phase would fail clippy's dead-code check on their own, so commits group them with their
  first user, as in M4a.
- Godot creates a `*.uid` file next to each new script. Commit it; it is not cache.
