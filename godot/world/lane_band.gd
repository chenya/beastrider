extends ColorRect
## The lighter strip of ground the hero walks on, placed from the hero's lane limits.
##
## Holds no game state: the limits live only in [Hero], which reads them from the tuning file.
## Stays zero-height (not drawn) if the hero failed to load its tuning.

## The hero whose lane limits this band shows. Set in the editor.
@export var hero: Hero


func _ready() -> void:
	# The band comes before the hero in the tree, so its _ready runs first.
	if not hero.is_node_ready():
		await hero.ready
	position = Vector2(0.0, hero.lane_top())
	size = Vector2(get_viewport_rect().size.x, hero.lane_bottom() - hero.lane_top())
