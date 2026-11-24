extends Node2D
## A small health bar above an enemy's head, for every kind.
##
## Presentation only: each frame it reads the parent enemy's [method Enemy.health] and
## [method Enemy.max_health] and draws two rectangles. It hides once the enemy is dying.

## Bar size, in pixels.
const SIZE := Vector2(48.0, 6.0)
## Gap between the top of the enemy's body and the bar, in pixels.
const GAP := 8.0
## Colour of the empty part.
const BACK_COLOR := Color(0.1, 0.1, 0.12, 0.85)
## Colour of the health left.
const FILL_COLOR := Color(0.85, 0.2, 0.2, 1.0)

## Height of the enemy's drawn body, in pixels: the [code]Body[/code] polygon in its scene. Set in
## [code]grunt.tscn[/code] and [code]brute.tscn[/code].
@export var body_height := 100.0

var _enemy: Enemy


func _ready() -> void:
	_enemy = get_parent() as Enemy


func _process(_delta: float) -> void:
	visible = _enemy.is_alive()
	queue_redraw()


func _draw() -> void:
	var top_left := Vector2(-SIZE.x / 2.0, -body_height - GAP - SIZE.y)
	draw_rect(Rect2(top_left, SIZE), BACK_COLOR)
	var max_health := _enemy.max_health()
	if max_health > 0.0:
		var filled := SIZE.x * clampf(_enemy.health() / max_health, 0.0, 1.0)
		draw_rect(Rect2(top_left, Vector2(filled, SIZE.y)), FILL_COLOR)
