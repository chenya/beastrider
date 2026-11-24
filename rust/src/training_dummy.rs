//! A training dummy: a target that can be hit over and over and never dies.
//!
//! Rust owns its knockback slide and keeps it inside the lane and the screen; the scene only sets
//! where it starts and what it looks like. Presentation (the white flash, the screen shake) reacts
//! to its `was_hit` signal without the dummy knowing about it. It is on the enemies' team with no
//! health: the hero can hit it, enemies ignore it, and it never dies or counts as a kill.

use godot::classes::{INode2D, Node2D};
use godot::prelude::*;

use crate::combat;
use crate::fighter::{Hit, Knockback, Target, Team};
use crate::hit_stop::HitStopTimer;
use crate::tuning_store::TuningStore;

/// Where the `Tuning` autoload lives in the scene tree.
const TUNING_PATH: &str = "/root/Tuning";

/// Where the `HitStop` autoload lives in the scene tree.
const HIT_STOP_PATH: &str = "/root/HitStop";

/// A target standing in the lane, origin at its feet so Y-sort draws it by depth.
///
/// It joins the enemies' group in `ready()`, so a swing finds it however it was added. If the
/// `Tuning` or `HitStop` autoload is missing, it reports one error and never moves.
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct TrainingDummy {
    /// The autoloads it reads every tick; `None` until `ready()` finds them, and for good if not.
    links: Option<Links>,
    /// The knockback slide under way, if any.
    slide: Option<Knockback>,
    base: Base<Node2D>,
}

/// The autoloads a dummy reads every tick, found once in `ready()`.
struct Links {
    tuning: Gd<TuningStore>,
    hit_stop: Gd<HitStopTimer>,
}

#[godot_api]
impl INode2D for TrainingDummy {
    fn init(base: Base<Node2D>) -> Self {
        // A dummy stands still until hit; `links` is filled by `ready()`.
        Self {
            links: None,
            slide: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group(Team::Enemies.group());
        let tuning = self.base().try_get_node_as::<TuningStore>(TUNING_PATH);
        let hit_stop = self.base().try_get_node_as::<HitStopTimer>(HIT_STOP_PATH);
        match (tuning, hit_stop) {
            (Some(tuning), Some(hit_stop)) => self.links = Some(Links { tuning, hit_stop }),
            _ => {
                godot_error!(
                    "TrainingDummy: the Tuning or HitStop autoload is missing or has the wrong type"
                );
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let Some(links) = &self.links else {
            return;
        };
        if links.hit_stop.bind().is_frozen() {
            return;
        }
        let Some(values) = links.tuning.bind().values() else {
            return;
        };
        let mut position = self.base().get_position();
        let mut slide = self.slide;
        if let Some(active) = &mut slide {
            position.x = active.advance(delta as f32);
        }
        // Clamp every tick, sliding or not, so a lane changed by a reload still holds the dummy.
        let half_width = values.dummy().hurtbox_width() / 2.0;
        let screen_width = self.base().get_viewport_rect().size.x;
        let (x, y) = combat::clamp_to_area(
            (position.x, position.y),
            half_width,
            screen_width,
            values.lane_top(),
            values.lane_bottom(),
        );
        let stopped_at_limit = x != position.x;
        self.slide = slide.filter(|s| !s.is_done() && !stopped_at_limit);
        self.base_mut().set_position(Vector2::new(x, y));
    }
}

#[godot_api]
impl TrainingDummy {
    /// Emitted at the moment of each hit. `direction` is the push direction (−1 left, 1 right);
    /// `strength` is the knockback distance in pixels (the attacker's).
    #[signal]
    fn was_hit(direction: f32, strength: f32);

    /// The hurtbox in screen pixels, from `dummy.hurtbox_width` and `hurtbox_height`, for the
    /// debug hitbox drawing; an empty rectangle while there is no tuning.
    #[func]
    pub fn hurtbox(&self) -> Rect2 {
        self.hurtbox_rect()
            .map_or_else(Rect2::default, |hurt| to_rect2(&hurt))
    }
}

impl TrainingDummy {
    /// Takes a blow: starts the attacker's knockback from where it stands, replacing any slide
    /// under way, and emits `was_hit`. It has no health, so the damage is ignored. The slide waits
    /// while play is frozen, so it starts when the impact freeze ends.
    pub fn take_hit<T>(&mut self, hit: &Hit<T>) {
        let start_x = self.base().get_position().x;
        self.slide = Knockback::start(start_x, hit.direction, hit.knockback);
        self.signals()
            .was_hit()
            .emit(hit.direction, hit.knockback.distance);
    }

    /// How the hit rule sees it: always alive and never invulnerable, since it cannot die. `None`
    /// while there is no tuning.
    pub fn target(&self) -> Option<Target<InstanceId>> {
        Some(Target {
            id: self.base().instance_id(),
            team: Team::Enemies,
            hurtbox: self.hurtbox_rect()?,
            feet_y: self.base().get_position().y,
            alive: true,
            invulnerable: false,
        })
    }

    /// The hurtbox in screen pixels, for hit checks, or `None` while there is no tuning.
    pub fn hurtbox_rect(&self) -> Option<combat::Rect> {
        let values = self.links.as_ref()?.tuning.bind().values()?;
        let position = self.base().get_position();
        Some(combat::hurtbox(
            (position.x, position.y),
            values.dummy().hurtbox_width(),
            values.dummy().hurtbox_height(),
        ))
    }
}

/// Converts a combat box into Godot's `Rect2`, for drawing.
pub fn to_rect2(rect: &combat::Rect) -> Rect2 {
    Rect2::new(
        Vector2::new(rect.left, rect.top),
        Vector2::new(rect.right - rect.left, rect.bottom - rect.top),
    )
}
