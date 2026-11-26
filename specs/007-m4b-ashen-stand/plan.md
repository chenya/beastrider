# Implementation Plan: M4b Ashen Stand

**Branch**: `feat/m4b-ashen-stand` | **Date**: 2026-11-25 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/007-m4b-ashen-stand/spec.md`

## Summary

Potions and a screen-clearing spell complete the Ashen Stand slice, which ends in the go or no-go
playtest. The approach:

- **A magic meter in each hero**: a pure `MagicMeter { charges, max }` with `add_charge`, `refill`
  and `spend_all`, starting empty each run ([research R2](research.md)).
- **Casting as a hero mode**: `HeroMode::Casting` entered at once from `Free`, `Attacking` or
  `HitStun`, invulnerable for its whole length; an empty press only shakes the meter
  ([R4](research.md)).
- **A cast freeze in `Run`**, apart from `HitStop`: everything but the caster skips its step until
  the impact ([R5](research.md)).
- **The impact**: every enemy whose feet are on screen takes `damage_per_charge × charges` (or the
  flat amount, by a tuned `SpellDamageMode`) and is knocked down; brutes too
  ([R3, R6](research.md)).
- **`KnockedDown`**: a new enemy mode that frees the turn, cannot be hit by blows, and ends in
  `Retreating` ([R7](research.md)).
- **Potions**: a Rust `Potion` node in a `potions` group with its own lifetime; the hero collects
  useful ones by the shared depth rule; drops are rolled by `Run` on each death and placed by the
  director ([R8–R10](research.md)).
- **Tuning**: `drop_chance` per kind and new `potions` and `magic` sections
  ([R13](research.md), [contracts/tuning-data.md](contracts/tuning-data.md)).
- **Presentation in GDScript**: meter pips and their shake, the screen flash, the potion's colour
  and blink, the knocked-down look ([R11](research.md)).

## Technical Context

**Language/Version**: Rust 1.100.0 for rules and state; GDScript (Godot 4.7.2) for presentation.
Unchanged.

**Primary Dependencies**: Godot **4.7.2-stable**; gdext `=0.5.5` with `api-4-6`. No new crates or
addons ([R1](research.md)).

**Storage**: `godot/data/tuning.json`, extended, read at start-up and on F5. Nothing is saved.

**Testing**: `cargo test` for every pure rule ([R15](research.md)); everything else by windowed
check scripts and by playing ([quickstart.md](quickstart.md) scenarios 1–27).

**Target Platform**: Ubuntu 24.04 LTS, Intel UHD 630, stock Mesa; release export as in M2a.

**Project Type**: Desktop game: a Godot project plus a Rust GDExtension library.

**Performance Goals**: A steady 60 fps at 1280×720 with V-Sync on, with ten enemies on screen and a
cast (apart from the intended freezes); restart in under 2 s.

**Constraints**: Compatibility renderer only; no gameplay value in code; GDScript holds no game
state; signal handlers never call back into the emitter; no behaviour branch on the enemy kind;
keyboard only; nothing from the spec's out-of-scope list.

**Scale/Scope**: 2 new Rust modules (`magic`, `potion`); `hero`, `enemy`, `run`, `wave_director`,
`training_dummy`, `fighter`, `tuning`, `tuning_store`, `lib` changed; 1 new scene
(`potion.tscn`); 3 new scripts (`magic_meter.gd`, `spell_flash.gd`, `potion_body.gd`); 4 scripts
and `main.tscn` changed; 2 new input actions.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution v1.3.3. ✅ = passes.

| Rule | Pre-research | Post-design | How M4b meets it |
| --- | --- | --- | --- |
| **I. Fun First, Playable Always** | ✅ | ✅ | Nine of the thirteen checks are player-visible; the milestone ends in the owner's balance playtest (US4) and the go or no-go playtest (US5). The game stays playable after each step: potions before the spell, the spell before the cheat. |
| **II. One Milestone at a Time** | ✅ | ✅ | Only the brief's M4b row: potion drops, the magic meter and spell, the refill-magic cheat and the go or no-go decision. Jumping, more spells, other items, art, sound and saving stay out. Sized at about three sessions. |
| **III. Architecture Serves the Game** | ✅ | ✅ | Each piece names its problem: a cast mode because the hero must be invulnerable and still while casting; a cast freeze apart from `HitStop` because the caster must keep running while everything else stops; a `KnockedDown` mode because downed enemies must free their turns and not be juggled; a `Potion` node because each potion is drawn, sorted by depth and times out. Not added: an item system, an inventory, an effects framework, `Area2D` pickups. |
| **IV. Data-Driven Tuning** | ✅ | ✅ | Every new value (drop chances and odds, potion lifetime, blink time, heal, size, maximum charges, damage mode and amounts, cast time, knockdown time and push) in `tuning.json`, checked on load, reloaded live. Flash and blink timings are looks, named constants in GDScript. |
| **V. Runs Well on Weak Hardware** | ✅ | ✅ | One full-screen `ColorRect`, a few potion polygons, one pass over the enemies at the impact; no lights, particles or shaders beyond M2b's flash. Checked with ten enemies and a cast on the UHD 630 (quickstart 20). |
| **VI. Original Art, Cheap Animation** | ✅ | ✅ | Placeholder polygons: a red and a blue flask; a flattened enemy shape when down. No outside assets. |
| **VII. Co-op Ready, Local Only** | ✅ | ✅ | Charges live in each hero, built from its player index (`p{index}_magic`); potions are found through the `potions` group and collected by any `team_players` hero; the cast freeze is a count, so two casters can overlap. No networking. |
| **VIII. Always Debuggable** | ✅ | ✅ | F9 fills the meter, arriving with the meter; F2 outlines potions' pickup boxes; debug builds only. R restarts in under 2 s with no potions and an empty meter. |
| **Tech: engine and API level** | ✅ | ✅ | 4.7.2, API level 4.6, unchanged. |
| **Tech: Rust/GDScript split** | ✅ | ✅ | Rust: meter, casting, freeze, impact, knockdown, drops, potions, pickups. GDScript: pips, shake, flash, potion colour and blink, knocked-down look, F9 key. |
| **Tech: types over strings** | ✅ | ✅ | `HeroMode::Casting`, `EnemyMode::KnockedDown`, `PotionKind`, `SpellDamageMode` enums, matched exhaustively; the group and action names are Rust constants or built from the player index. |
| **Tech: dependencies** | ✅ | ✅ | gdext only. |
| **Tech: one tuning-data format** | ✅ | ✅ | The same JSON file and loader; one new enum reader for `damage_mode`. |
| **Code Style: Rust** | ✅ | ✅ | rustfmt at 100; `try_new`/accessor naming; `Result` for data; no new `unsafe`; `///` docs on all public items and exposed members; pure rules in their own modules with tests. |
| **Code Style: GDScript** | ✅ | ✅ | Typed, tabs, `##` docs, code order per the constitution; constants for looks. |
| **Code Style: shared naming** | ✅ | ✅ | `Potion`, `PotionKind`, `MagicMeter`, `magic_changed`, `cast_started`, `spell_impact` the same in Rust, the editor and GDScript. |
| **Quality Gate** | ✅ | ✅ | fmt, build and clippy with 0 warnings, `cargo test`, 0 GDScript warnings, every check passed by playing (quickstart 1–27), including the owner's two playtests. |

**Result**: both passes are green. The expected departure (scenes and settings changed by the
agent, if the owner hands those tasks over, as in M0–M4a) is recorded below.

## Spec and input alignment

Decided by the owner during planning; [spec.md](spec.md) was updated to match:

- **Flat damage mode**: kept as a tuned choice (`magic.damage_mode`), revising the specify-step
  answer that dropped it. A flat cast still uses every charge; the starting mode is per charge, so
  check 4 holds ([R3](research.md)).
- **Who the spell hits**: enemies whose feet are on screen at the impact (the owner's input),
  replacing the spec's assumption that every enemy in play is hit ([R6](research.md)).

Taken from the owner's input, with the spec updated to match:

- **No juggling**: blows cannot land on a knocked-down enemy; the spell still can (FR-010,
  [R7](research.md)).

Resolved in the plan without a spec change:

- **Getting up**: to `Retreating`, M4a's path back to the waiting spot, which meets the input's
  "returns to Waiting" without letting a risen enemy take a turn at once ([R7](research.md)).
- **Training dummies**: they have no health, so the spell only flashes them and shakes the
  screen, without a knockdown (spec, Edge Cases; [R6](research.md)).
- **The push**: `magic.knockdown_push_time` added beside the input's push distance, because the
  shared `Knockback` needs both ([R13](research.md)).
- **Pickup depth**: `lane.depth_tolerance`, the value hits use; the input's "pickup box size" is
  the potion's `width` and `height` ([R9, R13](research.md)).
- **Blink**: `Potion.is_blinking()` rather than a remaining fraction, so `blink_time` stays out of
  GDScript ([R8](research.md)).
- **Who places potions**: `Run` rolls (the input); the director instantiates the scene, as it does
  for enemies ([R10](research.md)).
- **UHD 620 → UHD 630** ([R1](research.md)).

## Project Structure

### Documentation (this feature)

```text
specs/007-m4b-ashen-stand/
├── spec.md              # Feature spec (/speckit-specify, /speckit-clarify, planning answers)
├── plan.md              # This file
├── research.md          # Phase 0: meter, damage, casting, freeze, impact, knockdown, potions,
│                        #          pickup, drops, presentation, input, tuning, restart, tests
├── data-model.md        # Phase 1: structs, modes and transitions, formulas, rules
├── quickstart.md        # Phase 1: build, play, 27 validation scenarios
├── contracts/
│   ├── magic-classes.md # Hero, Enemy, Potion, RunTracker, WaveDirector, actions, scenes, scripts
│   └── tuning-data.md   # The extended file, rules, warning, when values apply
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 (/speckit-tasks — not created here)
```

### Source Code (repository root)

```text
rust/src/
├── lib.rs                 # + mod magic, potion
├── magic.rs               # NEW (pure): MagicMeter, SpellDamageMode, spell_damage(), can_cast(),
│                          #   on_screen(); unit tests
├── potion.rs              # NEW: Potion node, PotionKind, POTIONS_GROUP; pure drop_from_rolls(),
│                          #   can_collect(), potion_useful(); unit tests
├── hero.rs                # CHANGED: MagicMeter, HeroMode::Casting, the impact, pickups, magic
│                          #   action, new signals and funcs
├── enemy.rs               # CHANGED: EnemyMode::KnockedDown, take_spell(), enemy_after_spell(),
│                          #   pending drop, cast-freeze skip; tests
├── run.rs                 # CHANGED: cast freeze count, drop generator, roll_drop(); tests
├── wave_director.rs       # CHANGED: potion_scene, places drops, cast-freeze skip
├── training_dummy.rs      # CHANGED: cast-freeze skip; takes spell damage
├── fighter.rs             # CHANGED: Health::heal(), is_full(); tests
├── tuning.rs              # CHANGED: drop_chance, potions, magic, enum reader; rules, warning; tests
├── tuning_store.rs        # unchanged unless a new accessor is needed
├── crowd.rs, combat.rs, movement.rs, fighters.rs, hit_stop.rs   # unchanged

godot/
├── project.godot          # CHANGED: actions p1_magic (K, X), debug_refill_magic (F9)
├── main.tscn              # CHANGED: WaveDirector.potion_scene; Hud/MagicMeter; SpellFlash layer
├── world/
│   ├── potion.tscn        # NEW: Potion root, Body with potion_body.gd
│   ├── potion_body.gd     # NEW: colour by kind, blink while is_blinking()
│   └── enemy_body.gd      # CHANGED: knocked-down look
├── ui/
│   ├── magic_meter.gd     # NEW: pips; shake on cast_failed_empty
│   └── spell_flash.gd     # NEW: full-screen flash following the cast, peak at spell_impact
├── debug/
│   ├── debug_overlay.gd   # CHANGED: F9 → hero.refill_magic()
│   └── debug_hitboxes.gd  # CHANGED: potions' pickup boxes
└── data/tuning.json       # CHANGED: extended (contracts/tuning-data.md)
```

**Structure Decision**: Same layout as M4a. The meter and the spell's pure rules get their own
module, as `crowd.rs` did; the potion's rules live beside the node that uses them, as the wave
rules live beside the director.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
| --- | --- | --- |
| `main.tscn`, `potion.tscn` and the Input Map may be changed by the agent rather than by hand in the editor (Tech: "Scenes, nodes and animation are built in the Godot editor"), as in M0–M4a, if the owner hands those tasks over at the start of `/speckit-implement`. | The agent cannot use the editor GUI; the M3 and M4a methods (`ProjectSettings` for actions, `PackedScene.pack` + `ResourceSaver.save` or small text edits for scenes) produce ordinary files that open in the editor. | Building by hand in the editor stays the default for the owner's own tasks. Recorded at T045: the owner handed these tasks over, and the agent built `potion.tscn`, the `main.tscn` additions (the potion scene link, `Hud/MagicMeter`, the `SpellFlash` layer) and the `p1_magic` and `debug_refill_magic` actions this way. The owner's playtests (T017, T029, T033) passed with them. No other departure was found. |

## Follow-ups (outside M4b)

- **After the go or no-go decision**: on go, the brief's next milestones (art, stages, co-op,
  beasts); on no-go, the brief records why and the owner decides what to change.
- **A second hero (M7)**: the meter, the magic action and the cast freeze count already work per
  hero; the HUD meter will need one per player.
- **The brute-attack feel** the owner noticed in M4a (locked in a swing as an armored strike lands)
  is unchanged; the instant cast is now an escape from it.
