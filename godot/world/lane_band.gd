extends ColorRect
## The lighter strip of ground the hero walks on, placed from the lane limits in [TuningStore].
##
## Holds no game state: it reads the limits from the [code]Tuning[/code] autoload when placed and
## again after every successful tuning reload. Stays zero-height (not drawn) while
## [code]Tuning[/code] has no valid values.


func _ready() -> void:
	Tuning.tuning_reloaded.connect(_place)
	_place()


func _place() -> void:
	position = Vector2(0.0, Tuning.lane_top())
	size = Vector2(get_viewport_rect().size.x, Tuning.lane_bottom() - Tuning.lane_top())
