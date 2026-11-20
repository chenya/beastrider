extends CanvasLayer
## The debug overlay: a panel of live values shown and hidden with F1, and tuning reload with F5.
##
## Holds no game state. Every frame the panel is open, it reads the values it shows from the hero
## and the [code]Tuning[/code] autoload; it never parses or checks the tuning file itself. Keys
## come from the Input Map actions [code]debug_toggle_overlay[/code] and
## [code]debug_reload_tuning[/code], never from key codes. The result of each reload appears in
## the message spot, which is visible whether the panel is open or not.

## How long "Tuning reloaded" stays on screen, in seconds. Debug presentation, not tuning.
const RELOADED_MESSAGE_SECONDS := 2.0
## Shown after a successful reload.
const RELOADED_MESSAGE := "Tuning reloaded"

## The hero whose values the panel shows. Set in the editor.
@export var hero: Hero
## The world markers shown and hidden with the panel. Set in the editor.
@export var markers: Node2D

# Clears the "Tuning reloaded" message after RELOADED_MESSAGE_SECONDS.
var _message_timer := Timer.new()

@onready var _panel: PanelContainer = $Panel
@onready var _fps_line: Label = $Panel/Lines/FpsLine
@onready var _position_line: Label = $Panel/Lines/PositionLine
@onready var _player_line: Label = $Panel/Lines/PlayerLine
@onready var _horizontal_speed_line: Label = $Panel/Lines/HorizontalSpeedLine
@onready var _vertical_speed_line: Label = $Panel/Lines/VerticalSpeedLine
@onready var _tuning_line: Label = $Panel/Lines/TuningLine
@onready var _message: Label = $Message


func _ready() -> void:
	# Debug tools exist only in debug builds; a release export plays without them (FR-013).
	if not OS.is_debug_build():
		markers.queue_free()
		queue_free()
		return
	_set_shown(false)
	_message.text = ""
	_message_timer.one_shot = true
	_message_timer.timeout.connect(_clear_message)
	add_child(_message_timer)
	Tuning.tuning_reloaded.connect(_on_tuning_reloaded)
	Tuning.tuning_rejected.connect(_on_tuning_rejected)
	# The start-up load happened before this node existed, so its result is read, not heard.
	var rejected_field: String = Tuning.rejected_field()
	if not rejected_field.is_empty():
		_on_tuning_rejected(rejected_field, Tuning.rejected_reason())


func _process(_delta: float) -> void:
	if not _panel.visible:
		return
	_fps_line.text = "FPS: %d" % Engine.get_frames_per_second()
	_position_line.text = "Position: (%d, %d)" % [roundi(hero.position.x), roundi(hero.position.y)]
	_player_line.text = "Player: %d" % hero.player_index
	_horizontal_speed_line.text = "Horizontal speed: %d px/s" % roundi(Tuning.horizontal_speed())
	_vertical_speed_line.text = "Vertical speed: %d px/s" % roundi(Tuning.vertical_speed())
	var rejected_field: String = Tuning.rejected_field()
	if rejected_field.is_empty():
		_tuning_line.text = "Tuning: ok"
	else:
		_tuning_line.text = "Tuning: rejected (%s)" % rejected_field


func _unhandled_input(event: InputEvent) -> void:
	# is_action_pressed ignores key repeat by default, so holding F5 reloads once.
	if event.is_action_pressed("debug_toggle_overlay"):
		_set_shown(not _panel.visible)
		get_viewport().set_input_as_handled()
	elif event.is_action_pressed("debug_reload_tuning"):
		Tuning.reload()
		get_viewport().set_input_as_handled()


func _set_shown(shown: bool) -> void:
	_panel.visible = shown
	markers.visible = shown


func _on_tuning_reloaded() -> void:
	_message.text = RELOADED_MESSAGE
	_message_timer.start(RELOADED_MESSAGE_SECONDS)


func _on_tuning_rejected(field: String, reason: String) -> void:
	# A rejection stays until the next reload, so it cannot be missed (spec FR-008).
	_message_timer.stop()
	_message.text = "Tuning rejected: %s: %s" % [field, reason]


func _clear_message() -> void:
	_message.text = ""
