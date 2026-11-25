# Contract: classes, signals and scenes for M4b

What GDScript and the editor can see. Rules behind each member are in
[../data-model.md](../data-model.md).

## Autoloads (in order, unchanged)

`Tuning` (`TuningStore`), `HitStop` (`HitStopTimer`), `Run` (`RunTracker`).

## `Hero` (extended)

| Member | Kind | Contract |
| --- | --- | --- |
| `magic_changed(charges: int, max: int)` | signal | A charge was added, spent or refilled. |
| `cast_started(charges: int)` | signal | A cast began, spending `charges`. |
| `spell_impact()` | signal | The cast's impact resolved; the freeze has ended. |
| `cast_failed_empty()` | signal | Magic pressed with no charges; nothing else happened. |
| `potion_collected(kind: String)` | signal | `"Health"` or `"Magic"`. |
| `charges() -> int`, `max_charges() -> int` | func | For the meter; 0 before the first tick with tuning. |
| `is_casting() -> bool`, `cast_progress() -> float` | func | Progress 0 to 1 while casting, else 0. |
| `refill_magic()` | func | F9: the meter to full. Nothing at Game Over or before tuning. |

Handlers use the signal's arguments and call nothing back on the hero during the emit (M3 R8).

## `Enemy` (extended)

| Member | Kind | Contract |
| --- | --- | --- |
| `is_knocked_down() -> bool` | func | For the knocked-down look. |

`was_hit(direction, strength)` is also emitted by the spell's impact, with the knockdown push as
the strength.

## `Potion` (new, extends `Node2D`)

| Member | Kind | Contract |
| --- | --- | --- |
| `kind` | export (`PotionKind`: `"Health"`, `"Magic"`) | Set by the director before it enters the tree. |
| `kind() -> String` | func | For the colour. |
| `is_blinking() -> bool` | func | True in the last `potions.blink_time` of its life. |
| `pickup_box() -> Rect2` | func | For F2's outline; empty before tuning. |

Group: `potions`. No script on the root (M2b R6); scripts go on `Body`.

## `RunTracker` (extended, autoload `Run`)

| Member | Kind | Contract |
| --- | --- | --- |
| `is_cast_frozen() -> bool` | func | True while any cast is in progress. |

`begin_cast()`, `end_cast()` and `roll_drop(chance, odds)` are Rust-side only.

## `WaveDirector` (extended)

| Member | Kind | Contract |
| --- | --- | --- |
| `potion_scene` | export | `res://world/potion.tscn`. Set in the editor. |

## Groups

`team_players`, `team_enemies` (unchanged); new `potions`.

## Input Map

| Action | Keys (physical, any device, deadzone 0.2) | Builds |
| --- | --- | --- |
| `p1_magic` | K, X | all |
| `debug_refill_magic` | F9 | debug only (read by the debug overlay) |

## Scenes

- `godot/world/potion.tscn` (new): root `Potion`; child `Body` (`Polygon2D`, a placeholder flask
  `potions.width × potions.height` drawn by its script) with `potion_body.gd`.
- `godot/main.tscn`: `WaveDirector.potion_scene`; `Hud/MagicMeter` (`Control`, `magic_meter.gd`,
  `hero` path) beside the health bar; new `SpellFlash` (`CanvasLayer` above `Hud` and below
  `GameOver`, a full-screen `ColorRect`, `spell_flash.gd`, `hero` path).

## Presentation scripts

| Script | Reads | Reacts to |
| --- | --- | --- |
| `godot/ui/magic_meter.gd` | `hero.charges()`, `hero.max_charges()` | `cast_failed_empty` (shake) |
| `godot/ui/spell_flash.gd` | `hero.is_casting()`, `hero.cast_progress()` | `spell_impact` (peak, then fade) |
| `godot/world/potion_body.gd` | `potion.kind()`, `potion.is_blinking()` | — |
| `godot/world/enemy_body.gd` (extended) | `enemy.is_knocked_down()` | — |
| `godot/debug/debug_overlay.gd` (extended) | — | F9 → `hero.refill_magic()` |
| `godot/debug/debug_hitboxes.gd` (extended) | `potion.pickup_box()` | — |
