extends Node2D
## A small health bar above a grunt's head.
##
## Presentation only: each frame it reads the parent grunt's [method Grunt.health] and
## [method Grunt.max_health] and draws two rectangles. It hides once the grunt is dying.

## Bar size, in pixels.
const SIZE := Vector2(48.0, 6.0)
## Gap between the top of the grunt's body and the bar, in pixels.
const GAP := 8.0
## Height of the grunt's drawn body, in pixels (the polygon in [code]grunt.tscn[/code]).
const BODY_HEIGHT := 100.0
## Colour of the empty part.
const BACK_COLOR := Color(0.1, 0.1, 0.12, 0.85)
## Colour of the health left.
const FILL_COLOR := Color(0.85, 0.2, 0.2, 1.0)

var _grunt: Grunt


func _ready() -> void:
	_grunt = get_parent() as Grunt


func _process(_delta: float) -> void:
	visible = _grunt.is_alive()
	queue_redraw()


func _draw() -> void:
	var top_left := Vector2(-SIZE.x / 2.0, -BODY_HEIGHT - GAP - SIZE.y)
	draw_rect(Rect2(top_left, SIZE), BACK_COLOR)
	var max_health := _grunt.max_health()
	if max_health > 0.0:
		var filled := SIZE.x * clampf(_grunt.health() / max_health, 0.0, 1.0)
		draw_rect(Rect2(top_left, Vector2(filled, SIZE.y)), FILL_COLOR)
