# Contract: Tuning data file (M1)

The project-wide tuning format from M0 ([M0 contract](../../001-m0-setup/contracts/tuning-data.md)):
JSON at `godot/data/tuning.json`, read by Rust once per run, edited by the owner, with changes
taking effect on the next Play and no Rust rebuild. M1 replaces the `drifting_square` section with
two new sections. All the M0 rules for every section still apply.

## Shape

```json
{
  "hero": {
    "horizontal_speed": 300,
    "vertical_speed": 180
  },
  "lane": {
    "top": 430,
    "bottom": 700
  }
}
```

## Fields (M1)

| Field path | Type | Unit | Valid values | Default value in code |
| --- | --- | --- | --- | --- |
| `hero.horizontal_speed` | number | pixels per second | finite and `> 0` | **None** |
| `hero.vertical_speed` | number | pixels per second | finite and `> 0` | **None** |
| `lane.top` | number | pixels from the top of the screen | finite, `>= 0` and `< lane.bottom` | **None** |
| `lane.bottom` | number | pixels from the top of the screen | finite, `> lane.top` and `<= screen height` | **None** |

A missing field is an error (Principle IV). The starting values above make the vertical speed 60 %
of the horizontal one (spec SC-003: at most two thirds) and put the lane in the lower part of the
720 px screen.

## Section naming (updated from M0)

M0's rule said a section's key matches the Rust class that reads it. `lane` has no class of its
own: the hero reads it now, and enemies will read it later. The rule becomes: one top-level key
per tuned thing in `snake_case`, matching the Rust class name when one class owns it
(`Hero` → `hero`), otherwise naming the thing (`lane`).

## Removed

`drifting_square.speed`: deleted with the drifting square.
