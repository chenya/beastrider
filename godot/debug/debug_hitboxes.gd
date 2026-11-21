extends Node2D
## Debug drawing of the hit areas: the hero's attack box while a blow can land, and every training
## dummy's hurtbox.
##
## Shown and hidden with F2 by the debug overlay. Holds no game state: each frame it is shown, it
## reads the boxes from the Rust classes, which decide every hit. Sits outside the Y-sorted
## [code]World[/code] with a higher [member CanvasItem.z_index], so it is drawn over everything.

## Outline colour of the hero's attack box.
const ATTACK_COLOR := Color(1.0, 0.3, 0.2, 1.0)
## Outline colour of each dummy's hurtbox.
const HURTBOX_COLOR := Color(0.3, 1.0, 0.4, 1.0)
## Outline width, in pixels.
const LINE_WIDTH := 2.0

## The hero whose attack box is drawn. Set in the editor.
@export var hero: Hero
## The node holding the training dummies. Set in the editor.
@export var world: Node2D


func _process(_delta: float) -> void:
	if visible:
		queue_redraw()


func _draw() -> void:
	for child: Node in world.get_children():
		if child is TrainingDummy:
			_outline((child as TrainingDummy).hurtbox(), HURTBOX_COLOR)
	var attack := hero.attack_box()
	if attack.has_area():
		_outline(attack, ATTACK_COLOR)


func _outline(box: Rect2, color: Color) -> void:
	draw_rect(Rect2(to_local(box.position), box.size), color, false, LINE_WIDTH)
