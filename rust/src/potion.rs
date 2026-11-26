//! Potions: a health or magic potion an enemy leaves where it falls, collected by a hero who
//! walks over it at about its depth, blinking near the end of its life and then gone.
//!
//! The rules (the drop roll, the pickup, when a potion is useful, its ageing and its blink) are
//! pure functions tested without the engine; the `Potion` node only holds one potion's kind and
//! lifetime. Heroes find potions through [`POTIONS_GROUP`], never a fixed path, so any player's
//! hero can collect them (constitution, Principle VII).

use godot::classes::{INode2D, Node2D};
use godot::prelude::*;

use crate::combat::{self, Rect};
use crate::fighter::Countdown;
use crate::hit_stop::HitStopTimer;
use crate::run::RunTracker;
use crate::training_dummy::to_rect2;
use crate::tuning_store::TuningStore;

/// Godot group of every potion on the ground; heroes look for potions in it.
pub const POTIONS_GROUP: &str = "potions";

/// Where the `Tuning` autoload lives in the scene tree.
const TUNING_PATH: &str = "/root/Tuning";

/// Where the `HitStop` autoload lives in the scene tree.
const HIT_STOP_PATH: &str = "/root/HitStop";

/// Where the `Run` autoload lives in the scene tree.
const RUN_PATH: &str = "/root/Run";

/// Which potion it is. Exported as its name, so the editor shows a drop-down.
#[derive(GodotConvert, Var, Export, Debug, Clone, Copy, PartialEq, Eq)]
#[godot(via = GString)]
pub enum PotionKind {
    /// Red: restores `potions.heal_amount` health.
    Health,
    /// Blue: adds one magic charge.
    Magic,
}

/// The potion a death leaves, from two rolls in [0, 1): `roll_a` below `chance` means a drop, and
/// then `roll_b` below `health_odds` makes it a health potion, else a magic one.
pub fn drop_from_rolls(
    chance: f32,
    health_odds: f32,
    roll_a: f32,
    roll_b: f32,
) -> Option<PotionKind> {
    if roll_a >= chance {
        return None;
    }
    if roll_b < health_odds {
        Some(PotionKind::Health)
    } else {
        Some(PotionKind::Magic)
    }
}

/// Whether a hero collects a potion: their boxes overlap horizontally and their feet differ in
/// depth by at most `depth_tolerance`, the same rule a blow follows.
pub fn can_collect(
    hero_box: &Rect,
    hero_feet_y: f32,
    potion_box: &Rect,
    potion_feet_y: f32,
    depth_tolerance: f32,
) -> bool {
    combat::lands(
        potion_box,
        hero_box,
        potion_feet_y,
        hero_feet_y,
        depth_tolerance,
    )
}

/// Whether a potion would do something: a health potion only when the hero is hurt, a magic one
/// only when the meter has room. A potion that would do nothing stays on the ground.
pub fn potion_useful(kind: PotionKind, health_full: bool, meter_full: bool) -> bool {
    match kind {
        PotionKind::Health => !health_full,
        PotionKind::Magic => !meter_full,
    }
}

/// Ages a lifetime by `delta` unless play is frozen; true once it has run out.
pub fn age(life: &mut Countdown, frozen: bool, delta: f32) -> bool {
    if frozen {
        return !life.is_running();
    }
    life.tick(delta)
}

/// Whether a potion with `life` left is in its last `blink_time`.
pub fn blinking(life: &Countdown, blink_time: f32) -> bool {
    life.remaining() <= blink_time
}

/// A potion on the ground, origin at its feet so Y-sort draws it by depth. The wave director adds
/// it where an enemy fell; a hero collects it; otherwise it frees itself when its life runs out.
/// If the autoloads are missing it reports one error and never ages.
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct Potion {
    /// Which potion it is (`"Health"` or `"Magic"` to scripts, which read it for the colour); set
    /// by the director before it enters the tree.
    #[export]
    kind: PotionKind,
    /// The autoloads it reads every tick; `None` until `ready()` finds them, and for good if not.
    links: Option<Links>,
    /// Seconds left on the ground; `None` until the first tick with tuning starts it.
    life: Option<Countdown>,
    base: Base<Node2D>,
}

/// The autoloads a potion reads every tick, found once in `ready()`.
struct Links {
    tuning: Gd<TuningStore>,
    hit_stop: Gd<HitStopTimer>,
    run: Gd<RunTracker>,
}

#[godot_api]
impl INode2D for Potion {
    fn init(base: Base<Node2D>) -> Self {
        // A health potion unless placed otherwise; its life starts on the first tick with tuning.
        Self {
            kind: PotionKind::Health,
            links: None,
            life: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.base_mut().add_to_group(POTIONS_GROUP);
        let tuning = self.base().try_get_node_as::<TuningStore>(TUNING_PATH);
        let hit_stop = self.base().try_get_node_as::<HitStopTimer>(HIT_STOP_PATH);
        let run = self.base().try_get_node_as::<RunTracker>(RUN_PATH);
        match (tuning, hit_stop, run) {
            (Some(tuning), Some(hit_stop), Some(run)) => {
                self.links = Some(Links {
                    tuning,
                    hit_stop,
                    run,
                });
            }
            _ => {
                godot_error!(
                    "Potion: the Tuning, HitStop or Run autoload is missing or has the wrong type"
                );
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let Some(links) = &self.links else {
            return;
        };
        let Some(values) = links.tuning.bind().values() else {
            return;
        };
        let frozen = links.hit_stop.bind().is_frozen() || links.run.bind().is_cast_frozen();
        let life = self
            .life
            .get_or_insert_with(|| Countdown::new(values.potions().lifetime()));
        if age(life, frozen, delta as f32) {
            self.base_mut().queue_free();
        }
    }
}

#[godot_api]
impl Potion {
    /// Whether it is in its last `potions.blink_time`, so the body blinks.
    #[func]
    pub fn is_blinking(&self) -> bool {
        let blink_time = self
            .links
            .as_ref()
            .and_then(|links| links.tuning.bind().values())
            .map(|values| values.potions().blink_time());
        match (&self.life, blink_time) {
            (Some(life), Some(blink_time)) => blinking(life, blink_time),
            _ => false,
        }
    }

    /// Its pickup box (`potions.width × potions.height` centred over its feet), for its body and
    /// F2's outline; empty before tuning.
    #[func]
    pub fn pickup_box(&self) -> Rect2 {
        self.pickup_rect()
            .map_or_else(Rect2::default, |rect| to_rect2(&rect))
    }
}

impl Potion {
    /// Sets its kind and where it lies, before it enters the tree.
    pub fn place(&mut self, kind: PotionKind, position: Vector2) {
        self.kind = kind;
        self.base_mut().set_position(position);
    }

    /// Which potion it is.
    pub fn potion_kind(&self) -> PotionKind {
        self.kind
    }

    /// The height of its feet, the depth a hero must match to collect it.
    pub fn feet_y(&self) -> f32 {
        self.base().get_position().y
    }

    /// Its pickup box in the hit rule's terms, or `None` while there is no tuning.
    pub fn pickup_rect(&self) -> Option<Rect> {
        let values = self.links.as_ref()?.tuning.bind().values()?;
        let position = self.base().get_position();
        let potions = values.potions();
        Some(combat::hurtbox(
            (position.x, position.y),
            potions.width(),
            potions.height(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::Rect;

    // Values deliberately different from the starting values in tuning.json.
    const TOLERANCE: f32 = 15.0;
    const TICK: f32 = 0.125;

    fn hero_box() -> Rect {
        Rect {
            left: 100.0,
            right: 140.0,
            top: 410.0,
            bottom: 500.0,
        }
    }

    fn potion_box(x: f32) -> Rect {
        Rect {
            left: x - 10.0,
            right: x + 10.0,
            top: 472.0,
            bottom: 500.0,
        }
    }

    #[test]
    fn a_drop_happens_below_the_chance_and_its_kind_follows_the_odds() {
        assert_eq!(
            drop_from_rolls(0.4, 0.7, 0.39, 0.69),
            Some(PotionKind::Health)
        );
        assert_eq!(
            drop_from_rolls(0.4, 0.7, 0.39, 0.7),
            Some(PotionKind::Magic)
        );
        assert_eq!(
            drop_from_rolls(0.4, 0.7, 0.4, 0.0),
            None,
            "the roll at the chance misses"
        );
        assert_eq!(
            drop_from_rolls(0.0, 0.7, 0.0, 0.0),
            None,
            "chance 0 never drops"
        );
        assert!(
            drop_from_rolls(1.0, 0.7, 0.999, 0.0).is_some(),
            "chance 1 always drops"
        );
        assert_eq!(drop_from_rolls(1.0, 0.0, 0.5, 0.0), Some(PotionKind::Magic));
        assert_eq!(
            drop_from_rolls(1.0, 1.0, 0.5, 0.999),
            Some(PotionKind::Health)
        );
    }

    #[test]
    fn a_potion_is_collected_only_when_the_boxes_overlap_at_about_the_same_depth() {
        let hero = hero_box();
        assert!(can_collect(
            &hero,
            500.0,
            &potion_box(120.0),
            500.0,
            TOLERANCE
        ));
        assert!(can_collect(
            &hero,
            500.0,
            &potion_box(150.0),
            500.0 + TOLERANCE,
            TOLERANCE
        ));
        assert!(
            !can_collect(
                &hero,
                500.0,
                &potion_box(120.0),
                500.0 + 2.0 * TOLERANCE,
                TOLERANCE
            ),
            "twice the tolerance away in depth"
        );
        assert!(
            !can_collect(&hero, 500.0, &potion_box(200.0), 500.0, TOLERANCE),
            "the boxes do not overlap"
        );
    }

    #[test]
    fn a_potion_that_would_do_nothing_is_not_useful() {
        assert!(potion_useful(PotionKind::Health, false, true));
        assert!(!potion_useful(PotionKind::Health, true, false));
        assert!(potion_useful(PotionKind::Magic, true, false));
        assert!(!potion_useful(PotionKind::Magic, false, true));
    }

    #[test]
    fn a_lifetime_does_not_age_while_frozen() {
        let mut life = Countdown::new(1.0);
        assert!(!age(&mut life, true, TICK), "frozen");
        assert_eq!(life.remaining(), 1.0);
        assert!(!age(&mut life, false, TICK));
        assert_eq!(life.remaining(), 1.0 - TICK);
        for _ in 0..7 {
            age(&mut life, false, TICK);
        }
        assert!(age(&mut life, false, TICK), "ran out");
    }

    #[test]
    fn a_potion_blinks_in_its_last_blink_time() {
        assert!(!blinking(&Countdown::new(2.5), 2.0));
        assert!(blinking(&Countdown::new(2.0), 2.0));
        assert!(blinking(&Countdown::new(0.5), 2.0));
        assert!(
            !blinking(&Countdown::new(0.5), 0.0),
            "blink time 0 never blinks"
        );
    }
}
