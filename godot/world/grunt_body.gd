extends HitFlash
## A grunt's body: flashes when hit, turns the wind-up colour while the grunt winds up, and fades
## out when the grunt dies.
##
## Attached to the grunt's [code]Body[/code], never to the grunt itself (a script there would
## replace the Rust class's own [code]ready[/code]). Presentation only: it reads
## [method Grunt.is_winding_up] each frame and reacts to [signal Grunt.died], reading the fade
## length from the [code]Tuning[/code] autoload.

## The body's colour while the grunt winds up, so its attack can be seen coming.
const WIND_UP_COLOR := Color(1.0, 0.82, 0.25, 1.0)

var _grunt: Grunt
var _normal_color: Color


func _ready() -> void:
	super._ready()
	_grunt = get_parent() as Grunt
	_normal_color = color
	_grunt.died.connect(_on_died)


func _process(_delta: float) -> void:
	color = WIND_UP_COLOR if _grunt.is_winding_up() else _normal_color


func _on_died() -> void:
	# Fades the whole grunt (body and bar) on its own node; nothing is asked of the grunt.
	var fade := create_tween()
	fade.tween_property(_grunt, "modulate:a", 0.0, Tuning.death_fade_time())
