# Contract: `Hero` (Rust → Godot)

The interface the Rust library registers with Godot for M1. GDScript and scenes may rely only on
what is listed here. It replaces M0's `DriftingSquare`, which is removed.

## Class

| Item | Value |
| --- | --- |
| Godot class name | `Hero` (same name in Rust, the editor and GDScript) |
| Base class | `Node2D` |
| Registered by | the `beastrider` GDExtension library |
| Runs in the editor | No (not a `tool` class) |

## Required scene structure

```text
World                 (Node2D, y_sort_enabled = true)
├── Pillar1..3        (Polygon2D)  origin at the base, polygon above it
└── Hero              (Hero)       origin at the feet; last child of World, so it wins depth ties
    └── Body          (Polygon2D)  rectangle above the origin, e.g. (-24, -96) to (24, 0)
```

- The child MUST be named `Body` and be a `Polygon2D`. The hero's half width is half the
  horizontal extent of its polygon (plus `Body`'s own x offset, normally 0).
- If `Body` is missing, is not a `Polygon2D`, or has a width of 0, the hero reports an error and
  stays still (see Errors).

## Properties exposed to Godot

| Property | Type | Default | Description |
| --- | --- | --- | --- |
| `player_index` | `int`, exported | `1` | Which player controls this hero. Picks the input actions `p{player_index}_left`, `_right`, `_up` and `_down`. Read once in `ready()`; changing it later has no effect. |

## Methods exposed to Godot

| Method | Returns | Description |
| --- | --- | --- |
| `lane_top()` | `float` (px) | The highest y the hero's feet may reach. `0.0` when the hero is stopped. |
| `lane_bottom()` | `float` (px) | The lowest y the hero's feet may reach. `0.0` when the hero is stopped. |

Both are read-only. No signals or other methods are exposed in M1.

## Input actions (Input Map, project settings)

| Action | Keys (physical) |
| --- | --- |
| `p1_left` | Left arrow, A |
| `p1_right` | Right arrow, D |
| `p1_up` | Up arrow, W |
| `p1_down` | Down arrow, S |

A second player needs only `p2_*` actions and a second `Hero` with `player_index = 2`.

## Behaviour

| When | What happens |
| --- | --- |
| `ready()` | Builds and checks the four action names, loads the movement tuning ([tuning-data.md](tuning-data.md)), checks the lane fits the viewport, measures `Body`, and goes to *active* or *stopped* ([data-model.md](../data-model.md)) |
| `physics_process(delta)` while active | Reads `Input.get_vector` on its four actions, then sets its position to the movement step's result: move by input × speeds × delta, then clamp to the limits |
| `physics_process(delta)` while stopped | Does nothing |

## Errors

Reported once, in `ready()`, through Godot's error output. Each message starts with `Hero:` and
names the cause:

| Cause | Message names |
| --- | --- |
| An input action is missing from the Input Map | the action name and the player index |
| Tuning file missing, unreadable or not valid JSON | as in M0: the file path and the problem |
| `hero` or `lane` section, or one of their fields, missing | the file path and the field path |
| A speed not a number, not finite, or `<= 0` | the file path, the field path and the bad value |
| A lane value not a number or not finite; `lane.top >= lane.bottom`; `lane.top < 0`; `lane.bottom` beyond the screen height | the file path, the field path, the bad value and the rule broken |
| `Body` missing, not a `Polygon2D`, or zero width | the node path and what is wrong |

The game never crashes because of these errors.

## Used from GDScript

```gdscript
@export var hero: Hero   # typed: the class is visible to GDScript

func _ready() -> void:
	if not hero.is_node_ready():
		await hero.ready
	var top := hero.lane_top()
```
