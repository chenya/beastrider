# Contract: Tuning data file

The project-wide tuning format, decided in M0 ([research.md R4](../research.md)). Later milestones
add sections to this file. They do not invent new formats.

## Location and format

| Item | Value |
|---|---|
| Path | `godot/data/tuning.json` (in Godot: `res://data/tuning.json`) |
| Format | JSON, UTF-8, one top-level object |
| Read by | Rust only, once per run, when the node that needs it is ready |
| Edited by | the owner, in any text editor. Changes take effect on the next Play, with no Rust rebuild. |

## Shape

One top-level key per tuned thing, written in `snake_case` and matching the Rust class name
(`DriftingSquare` → `drifting_square`). Units go in this contract, not in the key names.

```json
{
  "drifting_square": {
    "speed": 160
  }
}
```

## Fields (M0)

| Field path | Type | Unit | Valid values | Default value in code |
|---|---|---|---|---|
| `drifting_square.speed` | number | pixels per second | finite and `>= 0` (`0` = stays still) | **None.** A missing field is an error (Principle IV) |

Starting value: `160`, which crosses the 1280 px window in about 8 seconds (spec Assumptions).

## Rules for all sections

1. Every value a reader needs MUST be in the file. Readers MUST NOT fall back to a value written
   in code.
2. Unknown keys are ignored, so a section can be added before the code that reads it.
3. Each reader checks its own section and reports errors naming the file, the field path and the
   problem ([drifting-square.md, Errors](drifting-square.md#errors)).
4. Numbers are JSON numbers, not strings.
