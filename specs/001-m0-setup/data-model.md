# Data Model: M0 Setup – Toolchain Proof

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Research**: [research.md](research.md)

M0 has very little data. This file lists who owns each value, what makes it valid, and how it
changes. Owners follow the constitution's split: **Rust** holds rules and state, **GDScript** only
displays, and the **scene** (built in the editor) holds layout and placeholder looks.

## Entities

### DriftingSquare (Rust class, extends Node2D)

| Field | Type | Owner | Source | Notes |
|---|---|---|---|---|
| `position.x` | `f32` | Rust | starting value from the scene, then updated each frame | Wrapped into `[-width, viewport_width)` every frame |
| `position.y` | `f32` | Scene | set in the editor | Never changed by Rust in M0 |
| `tuning` | `Option<SquareTuning>` | Rust | `godot/data/tuning.json`, loaded once in `ready()` | `None` if loading or checking failed |
| `width` | `f32` | Rust (cached) | width of the child `Body` node, read once in `ready()` | Must be > 0 |

Exposed to Godot: `speed() -> f32` (read-only). See
[contracts/drifting-square.md](contracts/drifting-square.md).

**State transitions**

```text
          ready(): tuning OK and Body found            process(delta)
[created] ───────────────────────────────────▶ [drifting] ─────────┐
    │                                              ▲                │ x += speed × delta
    │ ready(): tuning error or Body missing        └────────────────┘ x = wrap_x(x, width, vw)
    ▼
[stopped]   reports one error; process() does nothing; speed() = 0.0
```

Only two states, so a plain `Option` covers them; no state machine is needed (Principle III).

### SquareTuning (Rust struct, not exposed to Godot)

| Field | Type | Validation | Error message names |
|---|---|---|---|
| `speed` | `f32` (px/s) | present; a number; finite; `>= 0` | file path, `drifting_square.speed`, the bad value or "missing" |

Built only through a checked constructor (for example `SquareTuning::try_new(speed: f64)`), so an
invalid `SquareTuning` cannot exist.

### Tuning data file (JSON)

`godot/data/tuning.json`, the single project-wide tuning file. The schema and error cases are in
[contracts/tuning-data.md](contracts/tuning-data.md).

### Speed label (GDScript, extends Label)

| Field | Type | Owner | Notes |
|---|---|---|---|
| `square` | `DriftingSquare` (`@export`) | Scene | A node reference set in the editor. It is wiring, not game state. |

Each frame: `text = "Speed: %d px/s" % roundi(square.speed())`. It keeps no other variables.

### FPS counter (GDScript, extends Label)

No fields. Each frame: `text = "FPS: %d" % Engine.get_frames_per_second()`.

## Relationships

```text
tuning.json ──read once by──▶ DriftingSquare ◀──speed() polled each frame── SpeedLabel
                                   │
                                   └── child: Body (ColorRect, size set in the scene)

Engine.get_frames_per_second() ◀── polled each frame ── FpsLabel
```

## Co-op readiness (Principle VII)

`DriftingSquare` is not a player and reads no input, so it needs no player index. Nothing in M0
refers to "the player". The first player-controlled node (M1) must take a player index from the
start.
