extends Node2D
## Debug drawing of the hit areas: every fighter's hurtbox (hero, enemies, training dummies; an
## enemy waiting for a turn in grey) and each attack box while its blow can land.
##
## Shown and hidden with F2 by the debug overlay. Holds no game state: each frame it is shown, it
## reads the boxes from the Rust classes, which decide every hit. Sits outside the Y-sorted
## [code]World[/code] with a higher [member CanvasItem.z_index], so it is drawn over everything.

## Outline colour of the hero's attack box.
const ATTACK_COLOR := Color(1.0, 0.3, 0.2, 1.0)
## Outline colour of each dummy's hurtbox.
const HURTBOX_COLOR := Color(0.3, 1.0, 0.4, 1.0)
## Outline colour of the hero's hurtbox.
const HERO_HURTBOX_COLOR := Color(0.3, 0.7, 1.0, 1.0)
## Outline colour of each enemy's hurtbox.
const ENEMY_HURTBOX_COLOR := Color(0.9, 0.4, 1.0, 1.0)
## Outline colour of each enemy's hurtbox while it waits for a turn.
const WAITING_HURTBOX_COLOR := Color(0.55, 0.55, 0.6, 1.0)
## Outline colour of each enemy's attack box.
const ENEMY_ATTACK_COLOR := Color(1.0, 0.85, 0.2, 1.0)
## Outline width, in pixels.
const LINE_WIDTH := 2.0

## The hero whose boxes are drawn. Set in the editor.
@export var hero: Hero
## The node holding the enemies and training dummies. Set in the editor.
@export var world: Node2D


func _process(_delta: float) -> void:
	if visible:
		queue_redraw()


func _draw() -> void:
	for child: Node in world.get_children():
		if child is TrainingDummy:
			_outline((child as TrainingDummy).hurtbox(), HURTBOX_COLOR)
		elif child is Enemy:
			var enemy := child as Enemy
			var color := WAITING_HURTBOX_COLOR if enemy.is_waiting() else ENEMY_HURTBOX_COLOR
			_outline(enemy.hurtbox(), color)
			_outline(enemy.attack_box(), ENEMY_ATTACK_COLOR)
	_outline(hero.hurtbox(), HERO_HURTBOX_COLOR)
	var attack := hero.attack_box()
	if attack.has_area():
		_outline(attack, ATTACK_COLOR)


func _outline(box: Rect2, color: Color) -> void:
	if not box.has_area():
		return
	draw_rect(Rect2(to_local(box.position), box.size), color, false, LINE_WIDTH)
