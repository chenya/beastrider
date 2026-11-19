# Research: M1 Lane Movement

Decisions for M1, each with its reason and the options rejected. Most choices come from the
owner's technical context; this file records why they hold up and the details they leave open.
Behaviour marked *checked* was confirmed against the installed Godot 4.7.2 on 2026-11-19.

## R1. Versions: carried over from M0 unchanged

- **Decision**: Engine Godot **4.7.2-stable**, gdext `godot = "=0.5.5"` with `api-4-6`, Rust
  1.100.0, the `godot/` + `rust/` layout, JSON tuning data in `godot/data/tuning.json`, the
  Compatibility renderer, 1280×720, V-Sync on. Nothing is upgraded.
- **Rationale**: The owner asked to carry M0's pins over unchanged. The technical context says
  "Godot 4.6.x", but M0 actually pinned engine 4.7.2 with API level 4.6 (M0 research R1, chosen
  by the owner), so "unchanged" means 4.7.2. API level 4.6 stays: every API M1 uses
  (`Input.get_vector`, `InputMap`, `Node2D` Y-sort, `Polygon2D`) exists in 4.6.
- **Alternatives considered**: moving to engine 4.6.x to match the wording (a downgrade, and it
  would contradict "do not upgrade anything"); raising the API level to 4.7 (no 4.7-only API is
  needed, and it is an upgrade).

## R2. Depth sorting: Godot's Y-sort, origins at the feet

- **Decision**: A `Node2D` named `World` with `y_sort_enabled = true` holds the hero and the
  pillars as direct children. Each child's origin is at its feet (bottom centre), and its shape is
  drawn above the origin by giving its polygon negative y coordinates. The hero is the **last**
  child of `World`.
- **Rationale**: Y-sort draws children in order of their origin's y, which is exactly the spec's
  rule (FR-009): lower feet are drawn in front. The order is computed from positions each frame,
  so the same positions always give the same order, with no flicker. When two origins have the
  same y, Godot falls back to tree order, so making the hero the last child makes it win ties, as
  the spec's assumption asks. The hero's own child (`Body`) is drawn with the hero, because Y-sort
  orders direct children as whole groups.
- **Alternatives considered**: setting `z_index` from Rust every frame (more code doing what the
  engine already does); one `CanvasItem` per depth band (coarse, and breaks at band edges);
  sorting in Rust and calling `move_child` (moves nodes in the tree every frame, for no gain).

## R3. Placeholder shapes

- **Decision**: Pillars are `Polygon2D` nodes whose polygon is a tall rectangle above the origin
  (for example from (-32, -260) to (32, 0)). The hero's `Body` child is a `Polygon2D` rectangle
  about 48×96, from (-24, -96) to (24, 0). The lane band is a `ColorRect` that sits **outside**
  `World`, before it in the tree, so it is always drawn behind. The background stays the project's
  clear colour from M0.
- **Rationale**: `Polygon2D` is a `Node2D`, so it joins Y-sort by its origin, and the origin can
  be put at the feet. `ColorRect` is a `Control`: its origin is its top-left corner, so it cannot
  stand at its feet without an extra wrapper (the owner ruled it out inside `World`). Outside
  `World`, the band needs no sorting, so a `ColorRect` is the simplest band.
- **Alternatives considered**: `Sprite2D` with a generated texture (needs an image asset for a
  plain rectangle); drawing the band with `_draw()` (more code than a `ColorRect`).

## R4. Movement step: instant velocity in `_physics_process`

- **Decision**: In `physics_process`, the hero reads its input vector with `Input.get_vector` on
  its four actions, and sets its new position to
  `clamp(position + (input.x × horizontal_speed, input.y × vertical_speed) × delta, limits)`.
  There is no stored velocity, acceleration or friction.
- **Rationale**: Velocity is a pure function of the keys held this tick, so a press moves the
  hero at full speed on the first tick that sees it and a release stops it on the first tick that
  does not (FR-004). *Checked*: `get_vector` returns (0.707, -0.707) for right+up, (0, 0) for
  left+right, and (0, -1) for left+right+up, so diagonals are normalised and opposite keys cancel,
  matching the spec's edge cases with no extra code.
- **Diagonal speed**: with a unit diagonal (0.707, 0.707), the speed is
  0.707 × √(H² + V²). That is never more than the larger of H and V, so a diagonal is never faster
  than the faster straight direction. With V < H, as the starting values have, it is never faster
  than walking sideways (FR-006). The unit tests check both the starting values and V > H.
- **Timing**: physics runs at 60 ticks per second and physics interpolation is off (*checked*).
  With V-Sync at 60 Hz there is one physics tick per drawn frame, so "the same frame" holds. If
  the display runs at a rate that is not a multiple of 60, a frame can occasionally see zero or
  two ticks, which shows as slight unevenness. The quickstart watches for it. If it shows, the fix
  is to turn on Godot's physics interpolation (a project setting, no code change), recorded as a
  plan change at that point.
- **Alternatives considered**: `_process` with the frame delta (smooth at any refresh rate, but
  the owner chose physics so movement steps are fixed and match combat later); an acceleration
  curve (the spec forbids slide).

## R5. Player-indexed input actions

- **Decision**: The Input Map holds `p1_left`, `p1_right`, `p1_up` and `p1_down`, each bound to
  an arrow key and a WASD key (physical keycodes). The hero has an exported `player_index`
  (default 1). In `ready`, it builds the four names (`p{index}_left` and so on) once, stores them
  as `StringName`s in a small `PlayerActions` struct, and checks that each exists in the
  `InputMap`; a missing action is a clear error and the hero stays still.
- **Rationale**: Principle VII: nothing in the hero assumes "the player". A second hero needs
  `player_index = 2` and four `p2_*` actions, and no code change. Caching the names avoids
  building strings every tick, and the struct keeps the four names together rather than as loose
  strings. Physical keycodes keep WASD in the same place on non-QWERTY layouts.
- **Focus loss**: Godot releases held actions when the window loses focus, so the hero stops.
  This is confirmed by playing (quickstart scenario 6), since it cannot be checked headlessly.
- **Alternatives considered**: hard-coded `ui_left` and friends (shared by every player, and used
  by menus); an enum of players with a match per action (more code for the same result, and still
  needs the Input Map entries).

## R6. Limits: clamp after moving

- **Decision**: After the move, clamp the origin's x to
  `[half_width, viewport_width - half_width]` and its y to `[lane_top, lane_bottom]`.
  `half_width` is measured once in `ready` from the `Body` polygon's horizontal extent;
  `viewport_width` comes from the viewport. The clamp runs every tick, with or without input.
- **Rationale**: Clamping the result (not the input) means pushing into a limit puts the hero
  exactly on it every tick, so it cannot move back and forth (FR-007). Sliding along a limit works
  for free, because the other axis is not affected. Running the clamp with no input also places a
  hero that starts outside the lane at the nearest point inside on the first tick (spec edge case).
  Measuring the width from `Body` keeps the hero's size a scene choice (Principle IV: looks are
  not tuning) with no copy of it in code.
- **Alternatives considered**: refusing a move that would leave the area (the hero stops short of
  the edge at high speed); an exported `half_width` on the hero (a second copy of the size that
  can disagree with the shape).

## R7. Tuning data: two new sections, checked on load

- **Decision**: `godot/data/tuning.json` becomes:

  ```json
  {
    "hero": { "horizontal_speed": 300, "vertical_speed": 180 },
    "lane": { "top": 430, "bottom": 700 }
  }
  ```

  The `drifting_square` section is removed. The M0 loader is generalised: one read and parse of
  the file, then a checked `MovementTuning` value. Rules: both speeds finite and **> 0**; lane
  values finite with `0 <= top < bottom <= viewport height`. Each failure names the file, the field
  path (`hero.vertical_speed`, `lane.top`) and the bad value, as in M0.
- **Rationale**: The four values are the ones the owner named (`horizontal_speed`,
  `vertical_speed`, lane top and bottom). Splitting them into `hero` and `lane` sections matches
  how they will be used later: enemies will walk the same lane at their own speeds. The
  lane-inside-screen rule comes from the spec's edge cases; it needs the viewport height, so it is
  checked in the hero's `ready`, where the viewport is known, with the same error style.
- **Speed 0**: the owner's plan input requires both speeds to be greater than 0. The committed
  spec listed 0 on one axis as valid; this plan follows the owner's newer instruction, and the
  spec's edge case is updated to say 0 is invalid.
- **Alternatives considered**: flat keys at the top level (`lane_top` and so on), which would mix
  hero and lane values in one section; a separate file per section (the constitution asks for one
  format and M0 chose one file).

## R8. The lane band follows the tuning

- **Decision**: The `LaneBand` `ColorRect` has a GDScript (`godot/world/lane_band.gd`) with an
  exported `hero: Hero`. In `_ready` it waits for the hero to be ready if needed, then sets its own
  top and height from `hero.lane_top()` and `hero.lane_bottom()` and its width to the viewport
  width. If the hero has no valid tuning, both return 0, the band has zero height, and so it is
  not drawn.
- **Rationale**: FR-008 requires the band and the limits to match. Reading them from Rust once
  means there is one source (the tuning file), and the script holds no game state: it only places
  a shape. The band is a sibling before `World` in the tree, so its `_ready` runs before the
  hero's, which is why it waits for the hero's `ready` signal.
- **Alternatives considered**: placing the band by hand in the editor (it would drift from the
  tuning file, failing FR-008 and acceptance check 6); a Rust `Lane` node that draws the band
  (moves presentation into Rust, against the split in the constitution).

## R9. A pure, tested movement core

- **Decision**: `rust/src/movement.rs` holds plain types (`Vec2`-like pair of `f32`, `Speeds`,
  `Limits`) and one pure function, `step(position, input, speeds, delta, limits) -> position`,
  with no Godot types. Unit tests cover: zero input gives no movement; a diagonal is not faster
  than straight (for V < H and for V > H); vertical moves slower than horizontal with the starting
  values; clamping at each of the four limits; holding into a limit leaves the position unchanged
  over many ticks; a huge delta stays inside.
- **Rationale**: The owner's reason: these are easy to get subtly wrong and tedious to check by
  eye. Plain types let `cargo test` run without the engine, as `wrap_x` did in M0.
- **Alternatives considered**: using gdext's `Vector2` in the core (it works in tests, but ties
  the rules to the binding for no benefit); integration tests through Godot (slow, and the
  quickstart already covers the in-engine checks).

## R10. Removing M0 content

- **Decision**: Delete `rust/src/drifting_square.rs`, `godot/ui/speed_label.gd` and its `.uid`,
  and the `drifting_square` tuning section; rebuild `godot/main.tscn` without them. Keep
  `godot/ui/fps_label.gd` in the HUD.
- **Rationale**: The spec (FR-014) and the owner ask for it. Leaving dead code would keep a
  tuning value nothing reads.
- **Alternatives considered**: keeping the square as a second scene for reference (git history
  already keeps it).
