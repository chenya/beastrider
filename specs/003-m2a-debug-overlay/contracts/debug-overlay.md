# Contract: debug overlay, markers and keys

What the owner sees and presses, and how the scene is arranged. See
[research R7–R10](../research.md).

## Input Map actions (Project Settings → Input Map)

| Action | Event | Notes |
| --- | --- | --- |
| `debug_toggle_overlay` | Key F1 (physical keycode) | `device -1`, deadzone 0.2, like `p1_*` |
| `debug_reload_tuning` | Key F5 (physical keycode) | same |

No script checks a key directly (FR-012).

## Scene tree of `main.tscn`

```text
Main                 (Node2D)
├── LaneBand         (ColorRect, lane_band.gd)                    reads Tuning; no hero link
├── World            (Node2D, y_sort_enabled = true)              unchanged
│   ├── Pillar1..3   (Polygon2D)
│   └── Hero         (Hero, player_index = 1)
│       └── Body     (Polygon2D)
├── DebugMarkers     (Node2D, debug_markers.gd, z_index = 1)      NEW: after World, drawn over it
├── Hud              (CanvasLayer, layer 1)                        unchanged
│   └── FpsLabel     (Label, fps_label.gd, top-right)
└── DebugOverlay     (CanvasLayer, layer 2, debug_overlay.gd)     NEW
    ├── Panel        (PanelContainer, top-left, translucent, hidden at start)
    │   └── Lines    (VBoxContainer)
    │       ├── FpsLine, PositionLine, PlayerLine,
    │       ├── HorizontalSpeedLine, VerticalSpeedLine, TuningLine   (Label each)
    └── Message      (Label, top centre, empty at start)
```

Exported links, set in the editor: `DebugOverlay.hero → World/Hero`,
`DebugOverlay.markers → DebugMarkers`, `DebugMarkers.hero → World/Hero`. Mouse filter is
`ignore` on the panel and labels so they never take input.

## Behaviour

| When | Then |
| --- | --- |
| Game starts (debug build) | Panel and markers hidden; Message shows the start-up rejection if there was one, else empty |
| Game starts (release build) | `DebugOverlay` frees `DebugMarkers` and itself; F1 and F5 do nothing |
| F1 pressed | Panel and markers flip between shown and hidden together |
| Panel shown, each frame | Lines show `FPS: 60`, `Position: (460, 600)`, `Player: 1`, `Horizontal speed: 300 px/s`, `Vertical speed: 180 px/s`, `Tuning: ok` or `Tuning: rejected (lane.top)` |
| F5 pressed (panel open or not) | `Tuning.reload()` is called once per press (echo ignored) |
| `tuning_reloaded` | Message shows `Tuning reloaded`, cleared after `RELOADED_MESSAGE_SECONDS` (2.0); markers and band redraw |
| `tuning_rejected(field, reason)` | Message shows `Tuning rejected: <field>: <reason>` until the next reload; nothing else changes |
| Markers shown | 2-px bars at `Tuning.lane_top()` and `Tuning.lane_bottom()` across the screen; a cross at the hero's origin; redrawn only when the hero moves or tuning reloads |

Text and layout are placeholders; the owner may restyle them in the editor.
