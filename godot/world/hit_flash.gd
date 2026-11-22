class_name HitFlash
extends Polygon2D
## Flashes a fighter's body white when the fighter is hit.
##
## Attached to the fighter's [code]Body[/code], not to the fighter itself: a script on a Rust
## fighter would replace the Rust class's own [code]ready[/code]. Works for every fighter class
## (hero, grunt, training dummy), since each has a [code]was_hit[/code] signal. Presentation only:
## it reacts to the parent's [code]was_hit[/code] and reads the flash length from the
## [code]Tuning[/code] autoload each time. A flash length of 0 turns the flash off. The hero's and
## the grunt's body scripts extend this one.

## Name of the shader parameter in [code]flash.gdshader[/code] that blends towards white.
const FLASH_PARAMETER := &"flash_amount"
## Name of the signal every fighter emits when a blow lands on it.
const WAS_HIT_SIGNAL := &"was_hit"

var _flash_tween: Tween

@onready var _material: ShaderMaterial = material


func _ready() -> void:
	Signal(get_parent(), WAS_HIT_SIGNAL).connect(_on_was_hit)


func _on_was_hit(_direction: float, _strength: float) -> void:
	var flash_time: float = Tuning.flash_time()
	if flash_time <= 0.0:
		return
	if _flash_tween:
		_flash_tween.kill()
	_material.set_shader_parameter(FLASH_PARAMETER, 1.0)
	_flash_tween = create_tween()
	_flash_tween.tween_method(_set_flash, 1.0, 0.0, flash_time)


func _set_flash(amount: float) -> void:
	_material.set_shader_parameter(FLASH_PARAMETER, amount)
