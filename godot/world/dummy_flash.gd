extends Polygon2D
## Flashes a training dummy's body white when the dummy is hit.
##
## Attached to the dummy's [code]Body[/code], not to the dummy itself: a script on the dummy would
## replace the Rust class's own [code]ready[/code]. Presentation only: it reacts to the parent's
## [signal TrainingDummy.was_hit] and reads the flash length from the [code]Tuning[/code] autoload
## each time. A flash length of 0 turns the flash off.

## Name of the shader parameter in [code]flash.gdshader[/code] that blends towards white.
const FLASH_PARAMETER := &"flash_amount"

var _flash_tween: Tween

@onready var _material: ShaderMaterial = material


func _ready() -> void:
	(get_parent() as TrainingDummy).was_hit.connect(_on_was_hit)


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
