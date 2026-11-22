extends CanvasLayer
## The Game Over screen: shown when the hero falls, with the run's kill count and the restart hint.
##
## Presentation only: it reacts to [signal RunTracker.game_over] and uses the kill count passed
## with it; it calls nothing on [code]Run[/code] while the signal is being emitted. The restart
## itself is the [code]Run[/code] autoload's job, so R works whether this screen is shown or not.

## The screen's title.
const TITLE := "Game Over"
## The restart hint.
const HINT := "Press R to restart"

@onready var _title: Label = $Box/Title
@onready var _kills: Label = $Box/Kills
@onready var _hint: Label = $Box/Hint


func _ready() -> void:
	visible = false
	Run.game_over.connect(_on_game_over)


func _on_game_over(kills: int) -> void:
	_title.text = TITLE
	_kills.text = "Kills: %d" % kills
	_hint.text = HINT
	visible = true
