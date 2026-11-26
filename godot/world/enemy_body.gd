extends HitFlash
## An enemy's body, for every kind: flashes when hit, turns the wind-up colour while the enemy winds
## up, lies flattened while the spell has knocked it down, and fades out when the enemy dies.
##
## Attached to the enemy's [code]Body[/code], never to the enemy itself (a script there would
## replace the Rust class's own [code]ready[/code]). Presentation only: it reads
## [method Enemy.is_winding_up] and [method Enemy.is_knocked_down] each frame and reacts to [signal Enemy.died], reading the fade
## length from the [code]Tuning[/code] autoload.

## The body's colour while the enemy winds up, so its attack can be seen coming.
const WIND_UP_COLOR := Color(1.0, 0.82, 0.25, 1.0)
## The body's scale while the enemy is knocked down: wide and flat, lying on the ground.
const KNOCKED_DOWN_SCALE := Vector2(1.4, 0.35)

var _enemy: Enemy
var _normal_color: Color


func _ready() -> void:
	super._ready()
	_enemy = get_parent() as Enemy
	_normal_color = color
	_enemy.died.connect(_on_died)


func _process(_delta: float) -> void:
	color = WIND_UP_COLOR if _enemy.is_winding_up() else _normal_color
	scale = KNOCKED_DOWN_SCALE if _enemy.is_knocked_down() else Vector2.ONE


func _on_died() -> void:
	# Fades the whole enemy (body and bar) on its own node; nothing is asked of the enemy.
	var fade := create_tween()
	fade.tween_property(_enemy, "modulate:a", 0.0, Tuning.death_fade_time())
