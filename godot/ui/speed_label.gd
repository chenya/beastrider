extends Label
## Shows the drifting square's current speed, read from Rust every frame.
##
## Holds no game state: the speed lives only in [DriftingSquare].

## The square whose speed is shown. Set in the editor.
@export var square: DriftingSquare


func _process(_delta: float) -> void:
	text = "Speed: %d px/s" % roundi(square.speed())
