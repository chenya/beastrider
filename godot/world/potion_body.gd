extends Polygon2D
## A potion's body: a flask that fills the potion's pickup box, red for health and blue for
## magic, blinking near the end of the potion's life.
##
## Attached to the potion's [code]Body[/code], never to the potion itself (a script there would
## replace the Rust class's own [code]ready[/code]). Presentation only: each frame it reads
## [member Potion.kind], [method Potion.pickup_box] and [method Potion.is_blinking]; the flask is
## rebuilt whenever the box's size changes, so retuning [code]potions.width[/code] and
## [code]potions.height[/code] changes what is seen and what can be collected together.

## Colour of a health potion.
const HEALTH_COLOR := Color(0.85, 0.15, 0.15, 1.0)
## Colour of a magic potion.
const MAGIC_COLOR := Color(0.2, 0.4, 0.95, 1.0)
## Seconds the flask is shown, then hidden, while it blinks.
const BLINK_PERIOD := 0.12
## The neck's share of the flask's width.
const NECK_WIDTH := 0.4
## The neck's share of the flask's height.
const NECK_HEIGHT := 0.3

var _potion: Potion
var _drawn_size := Vector2.ZERO


func _ready() -> void:
	_potion = get_parent() as Potion


func _process(_delta: float) -> void:
	var box := _potion.pickup_box()
	if box.size != _drawn_size and box.has_area():
		_build(box.size)
	color = HEALTH_COLOR if str(_potion.kind) == "Health" else MAGIC_COLOR
	if _potion.is_blinking():
		var phase := int(Time.get_ticks_msec() / (BLINK_PERIOD * 1000.0))
		visible = phase % 2 == 0
	else:
		visible = true


## Rebuilds the flask to fill a box of [param size], standing on the potion's feet.
func _build(size: Vector2) -> void:
	_drawn_size = size
	var half := size.x / 2.0
	var neck := half * NECK_WIDTH
	var shoulder := -size.y * (1.0 - NECK_HEIGHT)
	polygon = PackedVector2Array([
		Vector2(-half, 0.0),
		Vector2(half, 0.0),
		Vector2(half, shoulder),
		Vector2(neck, shoulder),
		Vector2(neck, -size.y),
		Vector2(-neck, -size.y),
		Vector2(-neck, shoulder),
		Vector2(-half, shoulder),
	])
