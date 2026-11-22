extends HitFlash
## The hero's body: flashes when hit, blinks while the hero cannot be hit, and tips over when the
## hero falls.
##
## Attached to the hero's [code]Body[/code], never to the hero itself (a script there would replace
## the Rust class's own [code]ready[/code]). Presentation only: it reads
## [method Hero.is_invulnerable] each frame and reacts to [signal Hero.died].

## Seconds between blinks while the hero cannot be hit.
const BLINK_SECONDS := 0.08
## How far the body turns when the hero falls, in radians (lying down, feet towards the left).
const FALL_ANGLE := -PI / 2.0
## Seconds the fall takes.
const FALL_SECONDS := 0.3

var _hero: Hero
var _blink_time := 0.0
var _fallen := false


func _ready() -> void:
	super._ready()
	_hero = get_parent() as Hero
	_hero.died.connect(_on_died)


func _process(delta: float) -> void:
	if _fallen or not _hero.is_invulnerable():
		visible = true
		_blink_time = 0.0
		return
	_blink_time += delta
	visible = int(_blink_time / BLINK_SECONDS) % 2 == 0


func _on_died() -> void:
	_fallen = true
	visible = true
	var fall := create_tween()
	fall.tween_property(self, "rotation", FALL_ANGLE, FALL_SECONDS)
