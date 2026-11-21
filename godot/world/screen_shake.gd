extends Camera2D
## Shakes the play area when a training dummy is hit.
##
## Presentation only: it listens to [signal TrainingDummy.was_hit] on every dummy that enters the
## tree, including ones spawned later, and reads the shake strength and length from the
## [code]Tuning[/code] autoload when a shake starts. It runs in [method _process], so it keeps
## shaking through the impact freeze. HUD and overlay sit on their own canvas layers and stay still.
## Several hits on one tick restart one shake; they never add up. A strength or length of 0 turns
## the shake off.

var _strength := 0.0
var _duration := 0.0
var _time_left := 0.0


func _enter_tree() -> void:
	# Connected before the scene's other nodes enter, so dummies placed in the scene are caught too.
	get_tree().node_added.connect(_on_node_added)


func _process(delta: float) -> void:
	if _time_left <= 0.0:
		return
	_time_left = maxf(_time_left - delta, 0.0)
	var amplitude := _strength * _time_left / _duration
	offset = Vector2.from_angle(randf() * TAU) * randf() * amplitude


func _on_node_added(node: Node) -> void:
	if node is TrainingDummy:
		(node as TrainingDummy).was_hit.connect(_on_was_hit)


func _on_was_hit(_direction: float, _strength_px: float) -> void:
	var strength: float = Tuning.shake_strength()
	var duration: float = Tuning.shake_time()
	if strength <= 0.0 or duration <= 0.0:
		return
	_strength = strength
	_duration = duration
	_time_left = duration
