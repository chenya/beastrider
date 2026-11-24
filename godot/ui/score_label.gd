extends Label
## The run's score, shown under the kill count during play.
##
## Presentation only: it reads [method RunTracker.score] once at start-up, then shows the score
## passed with each [signal RunTracker.score_changed]; it calls nothing on [code]Run[/code] while
## the signal is being emitted. Part of release builds.


func _ready() -> void:
	text = "Score: %d" % Run.score()
	Run.score_changed.connect(_on_score_changed)


func _on_score_changed(score: int) -> void:
	text = "Score: %d" % score
