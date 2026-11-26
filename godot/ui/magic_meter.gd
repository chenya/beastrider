extends Control
## The hero's magic meter beside the health bar: one pip per charge it can hold, filled for each
## charge held.
##
## Presentation only: each frame it reads [method Hero.charges] and [method Hero.max_charges] and
## draws the pips. On [signal Hero.cast_failed_empty] it shakes for a moment, so a press with an
## empty meter is seen; the handler calls nothing back on the hero. It sits on the HUD's canvas
## layer, so the screen shake does not move it, and it is part of release builds.

## Size of one pip, in pixels.
const PIP_SIZE := Vector2(14.0, 14.0)
## Gap between two pips, in pixels.
const PIP_GAP := 6.0
## Colour of a filled pip.
const FILLED_COLOR := Color(0.25, 0.5, 1.0, 1.0)
## Colour of an empty pip.
const EMPTY_COLOR := Color(0.1, 0.1, 0.12, 0.85)
## How long the empty-meter shake lasts, in seconds.
const SHAKE_TIME := 0.25
## How far the meter swings to each side while it shakes, in pixels.
const SHAKE_DISTANCE := 4.0
## How many times a second the shake swings.
const SHAKE_RATE := 30.0

## The hero whose charges are shown. Set in the editor.
@export var hero: Hero

# Seconds of shake left; presentation state only.
var _shake_left := 0.0


func _ready() -> void:
	hero.cast_failed_empty.connect(_on_cast_failed_empty)


func _process(delta: float) -> void:
	_shake_left = maxf(_shake_left - delta, 0.0)
	queue_redraw()


func _draw() -> void:
	var offset := 0.0
	if _shake_left > 0.0:
		offset = sin(_shake_left * SHAKE_RATE * TAU) * SHAKE_DISTANCE
	var filled := hero.charges()
	for i: int in hero.max_charges():
		var at := Vector2(offset + i * (PIP_SIZE.x + PIP_GAP), 0.0)
		draw_rect(Rect2(at, PIP_SIZE), FILLED_COLOR if i < filled else EMPTY_COLOR)


func _on_cast_failed_empty() -> void:
	_shake_left = SHAKE_TIME
