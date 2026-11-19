# Contract: `DriftingSquare` (Rust → Godot)

The interface the Rust library registers with Godot. GDScript and scenes may rely only on what is
listed here.

## Class

| Item | Value |
|---|---|
| Godot class name | `DriftingSquare` (same name in Rust, the editor and GDScript) |
| Base class | `Node2D` |
| Registered by | the `beastrider` GDExtension library (`godot/beastrider.gdextension`) |
| Runs in the editor | No (not a `tool` class; it only runs while the game is playing) |

## Required scene structure

```text
DriftingSquare        (Node2D, Rust)  position set in the editor
└── Body              (ColorRect)     size set in the editor, e.g. 64×64; position (0, 0)
```

- The child MUST be named `Body` and be a `Control`, so that it has a size.
- If `Body` is missing or its width is not greater than 0, the square reports an error and stops
  (see Errors).

## Methods exposed to Godot

| Method | Returns | Description |
|---|---|---|
| `speed()` | `float` (px/s) | The current horizontal speed. `0.0` when tuning failed to load. Read-only: there is no setter. |

No properties, signals or other methods are exposed in M0.

## Behaviour

| When | What happens |
|---|---|
| `ready()` | Loads the `drifting_square` section of the tuning data ([tuning-data.md](tuning-data.md)), reads `Body`'s width, and goes to *drifting* or *stopped* ([data-model.md](../data-model.md)) |
| `process(delta)` while drifting | `x += speed × delta`, then wraps so the square fully leaves the right edge before entering from the left (`x` stays in `[-width, viewport_width)`) |
| `process(delta)` while stopped | Does nothing |

## Errors

All errors are reported once, in `ready()`, through Godot's error output (the editor's Output and
Debugger panels). Each message starts with `DriftingSquare:` and names the cause:

| Cause | Message names |
|---|---|
| Tuning file missing or unreadable | the file path and the read error |
| Tuning file is not valid JSON | the file path, line number and parser message |
| `drifting_square` section or `speed` field missing | the file path and the field path |
| `speed` not a number, not finite, or negative | the file path, the field path and the bad value |
| `Body` child missing or width ≤ 0 | the node path and what is wrong |

The game never crashes because of these errors.

## Used from GDScript

```gdscript
@export var square: DriftingSquare   # typed: the class is visible to GDScript

func _process(_delta: float) -> void:
	text = "Speed: %d px/s" % roundi(square.speed())
```

(The example shows the call only. The full script follows the constitution's code order.)
