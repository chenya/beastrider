extends Label
## Shows the engine's frames per second, read every frame.


func _process(_delta: float) -> void:
	text = "FPS: %d" % Engine.get_frames_per_second()
