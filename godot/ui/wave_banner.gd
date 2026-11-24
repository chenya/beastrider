extends Label
## The wave banner: "Wave N" at the top centre of the play area while a wave's banner shows.
##
## Presentation only: each frame it reads [method WaveDirector.banner_wave], which is the wave's
## number while its banner shows and 0 otherwise, so the banner's length lives in Rust alone. Part of
## release builds.

## The director whose banner is shown. Set in the editor.
@export var director: WaveDirector


func _process(_delta: float) -> void:
	var wave: int = director.banner_wave()
	visible = wave > 0
	if visible:
		text = "Wave %d" % wave
