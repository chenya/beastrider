extends Label
## The kill count, shown under the health bar during play.
##
## Presentation only: each frame it reads [method RunTracker.kills] from the [code]Run[/code]
## autoload. Part of release builds.


func _process(_delta: float) -> void:
	text = "Kills: %d" % Run.kills()
