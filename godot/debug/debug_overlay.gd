extends CanvasLayer
## The debug overlay: a panel of live values shown and hidden with F1, and tuning reload with F5.
##
## Holds no game state. Every frame the panel is open, it reads the values it shows from the hero
## and the [code]Tuning[/code] autoload; it never parses or checks the tuning file itself. Keys
## come from the Input Map actions [code]debug_toggle_overlay[/code] and
## [code]debug_reload_tuning[/code], never from key codes. The result of each reload appears in
## the message spot, which is visible whether the panel is open or not, with any tuning warning
## after it. F2 shows the hitboxes, F3 sends a grunt and F7 a brute into the current wave, F4
## refills the hero's health, F6 adds a training dummy, F8 ends the wave and starts the next and F9
## fills the hero's magic meter (actions [code]debug_show_hitboxes[/code],
## [code]debug_spawn_grunt[/code], [code]debug_spawn_brute[/code],
## [code]debug_refill_health[/code], [code]debug_spawn_dummy[/code],
## [code]debug_next_wave[/code] and [code]debug_refill_magic[/code]); the panel also counts hits
## landed.

## How long "Tuning reloaded" stays on screen, in seconds. Debug presentation, not tuning.
const RELOADED_MESSAGE_SECONDS := 2.0
## Shown after a successful reload.
const RELOADED_MESSAGE := "Tuning reloaded"

## The hero whose values the panel shows. Set in the editor.
@export var hero: Hero
## The world markers shown and hidden with the panel. Set in the editor.
@export var markers: Node2D
## The hitbox drawing shown and hidden with F2. Set in the editor.
@export var hitboxes: Node2D
## The node F6 adds new training dummies to. Set in the editor.
@export var world: Node2D
## The training dummy scene F6 adds. Set in the editor.
@export var dummy_scene: PackedScene
## The director F3, F7 and F8 act on. Set in the editor.
@export var director: WaveDirector

# Clears the "Tuning reloaded" message after RELOADED_MESSAGE_SECONDS.
var _message_timer := Timer.new()

@onready var _panel: PanelContainer = $Panel
@onready var _fps_line: Label = $Panel/Lines/FpsLine
@onready var _position_line: Label = $Panel/Lines/PositionLine
@onready var _player_line: Label = $Panel/Lines/PlayerLine
@onready var _horizontal_speed_line: Label = $Panel/Lines/HorizontalSpeedLine
@onready var _vertical_speed_line: Label = $Panel/Lines/VerticalSpeedLine
@onready var _tuning_line: Label = $Panel/Lines/TuningLine
@onready var _hits_line: Label = $Panel/Lines/HitsLine
@onready var _message: Label = $Message


func _ready() -> void:
	# Debug tools exist only in debug builds; a release export plays without them (FR-013).
	if not OS.is_debug_build():
		markers.queue_free()
		hitboxes.queue_free()
		queue_free()
		return
	_set_shown(false)
	hitboxes.visible = false
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
	elif not Tuning.warning().is_empty():
		_show_warning()


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
	_hits_line.text = "Hits: %d" % hero.hits_landed()


func _unhandled_input(event: InputEvent) -> void:
	# is_action_pressed ignores key repeat by default, so holding F5 reloads once.
	if event.is_action_pressed("debug_toggle_overlay"):
		_set_shown(not _panel.visible)
		get_viewport().set_input_as_handled()
	elif event.is_action_pressed("debug_reload_tuning"):
		Tuning.reload()
		get_viewport().set_input_as_handled()
	elif event.is_action_pressed("debug_show_hitboxes"):
		hitboxes.visible = not hitboxes.visible
		get_viewport().set_input_as_handled()
	elif event.is_action_pressed("debug_spawn_grunt"):
		director.spawn_grunt()
		get_viewport().set_input_as_handled()
	elif event.is_action_pressed("debug_spawn_brute"):
		director.spawn_brute()
		get_viewport().set_input_as_handled()
	elif event.is_action_pressed("debug_next_wave"):
		director.skip_wave()
		get_viewport().set_input_as_handled()
	elif event.is_action_pressed("debug_refill_magic"):
		hero.refill_magic()
		get_viewport().set_input_as_handled()
	elif event.is_action_pressed("debug_refill_health"):
		hero.refill_health()
		get_viewport().set_input_as_handled()
	elif event.is_action_pressed("debug_spawn_dummy"):
		_spawn_dummy()
		get_viewport().set_input_as_handled()


func _spawn_dummy() -> void:
	var dummy: Node2D = dummy_scene.instantiate()
	# The hero (in Rust) decides where; this only places it.
	dummy.position = hero.dummy_spawn_point()
	world.add_child(dummy)
	# Keep the hero the last child of the Y-sorted world, so it still wins depth ties.
	world.move_child(dummy, hero.get_index())


func _set_shown(shown: bool) -> void:
	_panel.visible = shown
	markers.visible = shown


func _on_tuning_reloaded() -> void:
	if not Tuning.warning().is_empty():
		_show_warning()
		return
	_message.text = RELOADED_MESSAGE
	_message_timer.start(RELOADED_MESSAGE_SECONDS)


func _show_warning() -> void:
	# A warning stays until the next reload, like a rejection, so it cannot be missed.
	_message_timer.stop()
	_message.text = "%s (warning: %s)" % [RELOADED_MESSAGE, Tuning.warning()]


func _on_tuning_rejected(field: String, reason: String) -> void:
	# A rejection stays until the next reload, so it cannot be missed (spec FR-008).
	_message_timer.stop()
	_message.text = "Tuning rejected: %s: %s" % [field, reason]


func _clear_message() -> void:
	_message.text = ""
