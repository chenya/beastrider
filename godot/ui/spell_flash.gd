extends ColorRect
## The spell's flash: the whole screen brightens as the cast builds, peaks at the impact, then
## fades.
##
## Presentation only: while the hero casts, its alpha follows [method Hero.cast_progress]; on
## [signal Hero.spell_impact] it holds the peak and fades over [constant FADE_TIME]. The handler
## calls nothing back on the hero. It never catches the mouse.

## Alpha at the moment of impact.
const PEAK_ALPHA := 0.85
## Seconds the flash takes to fade after the impact.
const FADE_TIME := 0.35

## The hero whose casts are shown. Set in the editor.
@export var hero: Hero

# Seconds of fade left after an impact; presentation state only.
var _fade_left := 0.0


func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	modulate.a = 0.0
	hero.spell_impact.connect(_on_spell_impact)


func _process(delta: float) -> void:
	if hero.is_casting():
		modulate.a = hero.cast_progress() * PEAK_ALPHA
	elif _fade_left > 0.0:
		_fade_left = maxf(_fade_left - delta, 0.0)
		modulate.a = PEAK_ALPHA * _fade_left / FADE_TIME
	else:
		modulate.a = 0.0


func _on_spell_impact() -> void:
	_fade_left = FADE_TIME
	modulate.a = PEAK_ALPHA
