extends Node2D
## Debug markers drawn over the world: the lane's top and bottom limits and the hero's origin.
##
## Shown and hidden by the debug overlay. Sits outside the Y-sorted [code]World[/code] with a
## higher [member CanvasItem.z_index], so it is drawn over every pillar and the hero. Reads the
## limits from the [code]Tuning[/code] autoload and the origin from the hero each time it draws;
## it redraws only when the hero moves or tuning is reloaded.

## Height of each lane bar, in pixels. The bar straddles the limit, so the hero's feet, whose
## bottom edge sits on the limit, meet the middle of the bar. Debug presentation, not tuning.
const BAR_THICKNESS := 2.0
## Half the length of each arm of the origin cross, in pixels.
const CROSS_HALF_SIZE := 6.0
## Line width of the origin cross, in pixels.
const CROSS_WIDTH := 2.0
## Colour of the lane bars.
const BAR_COLOR := Color(0.3, 0.9, 1.0, 0.9)
## Colour of the origin cross.
const CROSS_COLOR := Color(1.0, 0.25, 0.85, 1.0)

## The hero whose origin the cross marks. Set in the editor.
@export var hero: Hero

# Where the hero was at the last redraw; only decides whether to repaint, holds no game state.
var _drawn_position := Vector2.INF


func _ready() -> void:
	Tuning.tuning_reloaded.connect(queue_redraw)


func _process(_delta: float) -> void:
	if visible and hero.global_position != _drawn_position:
		queue_redraw()


func _draw() -> void:
	_drawn_position = hero.global_position
	if Tuning.has_values():
		var width := get_viewport_rect().size.x
		for limit: float in [Tuning.lane_top(), Tuning.lane_bottom()]:
			var bar := Rect2(0.0, limit - BAR_THICKNESS / 2.0, width, BAR_THICKNESS)
			draw_rect(_to_local_rect(bar), BAR_COLOR)
	var origin := to_local(_drawn_position)
	draw_line(origin - Vector2(CROSS_HALF_SIZE, 0.0), origin + Vector2(CROSS_HALF_SIZE, 0.0),
			CROSS_COLOR, CROSS_WIDTH)
	draw_line(origin - Vector2(0.0, CROSS_HALF_SIZE), origin + Vector2(0.0, CROSS_HALF_SIZE),
			CROSS_COLOR, CROSS_WIDTH)


# Converts a rectangle in world (canvas) coordinates to this node's local coordinates.
func _to_local_rect(rect: Rect2) -> Rect2:
	return Rect2(to_local(rect.position), rect.size)
