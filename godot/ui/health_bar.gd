extends Control
## The hero's health bar in the top-left corner of the screen.
##
## Presentation only: each frame it reads the hero's [method Hero.health] and
## [method Hero.max_health] and draws two rectangles. It sits on the HUD's canvas layer, so the
## screen shake does not move it, and it is part of release builds.

## Bar size, in pixels.
const SIZE := Vector2(240.0, 14.0)
## Colour of the empty part.
const BACK_COLOR := Color(0.1, 0.1, 0.12, 0.85)
## Colour of the health left.
const FILL_COLOR := Color(0.85, 0.2, 0.2, 1.0)

## The hero whose health is shown. Set in the editor.
@export var hero: Hero


func _process(_delta: float) -> void:
	queue_redraw()


func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, SIZE), BACK_COLOR)
	var max_health := hero.max_health()
	if max_health > 0.0:
		var filled := SIZE.x * clampf(hero.health() / max_health, 0.0, 1.0)
		draw_rect(Rect2(Vector2.ZERO, Vector2(filled, SIZE.y)), FILL_COLOR)
