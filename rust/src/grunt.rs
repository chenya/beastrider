//! The grunt: the first enemy that fights back.
//!
//! It walks in from a screen edge, lines up at the nearest hero's depth, closes in to its reach,
//! winds up visibly and strikes. Any blow stuns it and cancels its attack; at zero health it fades
//! out and is removed, and the run counts a kill. Rust owns all of it; the scene only gives it a
//! body, a health bar and their scripts, which read from it and react to its signals.

use godot::classes::{INode2D, Node2D};
use godot::prelude::*;

use crate::combat::{self, Facing};
use crate::fighter::{
    Approach, AttackPhase, AttackSpec, AttackStep, AttackTimer, Attacker, Countdown, Damage,
    Health, Hit, Knockback, Target, Team, approach_step, resolve_attack,
};
use crate::fighters::FighterRef;
use crate::hit_stop::HitStopTimer;
use crate::movement::Speeds;
use crate::run::RunTracker;
use crate::training_dummy::to_rect2;
use crate::tuning::TuningValues;
use crate::tuning_store::TuningStore;

/// Where the `Tuning` autoload lives in the scene tree.
const TUNING_PATH: &str = "/root/Tuning";

/// Where the `HitStop` autoload lives in the scene tree.
const HIT_STOP_PATH: &str = "/root/HitStop";

/// Where the `Run` autoload lives in the scene tree.
const RUN_PATH: &str = "/root/Run";

/// An enemy that hunts the nearest hero, origin at its feet so Y-sort draws it by depth.
///
/// Placed only by the `GruntSpawner`, which calls [`place`](Self::place) before adding it. It joins
/// the enemies' group once it has walked fully on screen and leaves it the moment it starts dying,
/// so it can be neither hit nor counted as a target before or after. If an autoload is missing, it
/// reports one error and never moves. Never give its root node a script: a GDScript `_ready` would
/// replace this class's `ready()`.
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct Grunt {
    /// The autoloads it reads every tick; `None` until `ready()` finds them, and for good if not.
    links: Option<Links>,
    /// What it is doing.
    mode: GruntMode,
    /// Health points; `None` until the first tick with tuning, which starts it full.
    health: Option<Health>,
    /// The way it faces: towards its target.
    facing: Facing,
    /// The slide from the last blow taken, if it is still under way.
    knockback: Option<Knockback>,
    /// The screen edge it enters from: `Left` walks in rightwards, `Right` leftwards.
    edge: Facing,
    base: Base<Node2D>,
}

/// The autoloads a grunt reads every tick, found once in `ready()`.
struct Links {
    tuning: Gd<TuningStore>,
    hit_stop: Gd<HitStopTimer>,
    run: Gd<RunTracker>,
}

/// What a grunt is doing. Every use is an exhaustive `match`, so a new mode cannot be forgotten.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GruntMode {
    /// Walking in from its edge; cannot be hit and does not attack.
    Entering,
    /// Lining up at its target's depth, then closing in to its reach.
    Approaching,
    /// Winding up: visibly marked, and neither moving nor turning.
    WindUp(Attack),
    /// The blow can land on its target, once per attack.
    Active(Attack),
    /// After the blow, before it hunts again.
    Recovery(Attack),
    /// Stunned by a blow: no moving or attacking until the countdown runs out.
    HitStun(Countdown),
    /// Fading out after its health reached zero; removed when the countdown runs out.
    Dying(Countdown),
}

/// One grunt attack under way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attack {
    /// The attack's part, time and values (copied at the start, so a reload applies next attack).
    timer: AttackTimer,
    /// Whether this attack has already landed, so it lands at most once.
    landed: bool,
}

impl Attack {
    /// A new attack with `spec`, at the start of its wind-up.
    fn new(spec: AttackSpec) -> Self {
        Self {
            timer: AttackTimer::new(spec),
            landed: false,
        }
    }

    /// The mode matching the attack's current part.
    fn mode(self) -> GruntMode {
        match self.timer.phase() {
            AttackPhase::WindUp => GruntMode::WindUp(self),
            AttackPhase::Active => GruntMode::Active(self),
            AttackPhase::Recovery => GruntMode::Recovery(self),
        }
    }
}

/// The mode after a blow lands: killed → dying for `death_fade_time`; otherwise stunned for
/// `hit_stun_time`, dropping any attack in any part. Entering and dying grunts cannot be hit, so a
/// blow leaves them as they are.
pub fn grunt_after_hit(
    mode: GruntMode,
    damage: Damage,
    hit_stun_time: f32,
    death_fade_time: f32,
) -> GruntMode {
    match mode {
        GruntMode::Entering | GruntMode::Dying(_) => mode,
        GruntMode::Approaching
        | GruntMode::WindUp(_)
        | GruntMode::Active(_)
        | GruntMode::Recovery(_)
        | GruntMode::HitStun(_) => match damage {
            Damage::Survived => GruntMode::HitStun(Countdown::new(hit_stun_time)),
            Damage::Killed => GruntMode::Dying(Countdown::new(death_fade_time)),
            Damage::AlreadyDead => mode,
        },
    }
}

#[godot_api]
impl INode2D for Grunt {
    fn init(base: Base<Node2D>) -> Self {
        // A grunt enters from the left unless the spawner places it; health is filled on the first
        // tick with tuning, and `links` by `ready()`.
        Self {
            links: None,
            mode: GruntMode::Entering,
            health: None,
            facing: Facing::Right,
            knockback: None,
            edge: Facing::Left,
            base,
        }
    }

    fn ready(&mut self) {
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
                    "Grunt: the Tuning, HitStop or Run autoload is missing or has the wrong type"
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
        let playing = links.run.bind().is_playing();
        let delta = delta as f32;
        let max_health = values.grunt().max_health();
        match &mut self.health {
            Some(health) => health.set_max(max_health),
            None => self.health = Some(Health::new(max_health)),
        }
        // Once the hero has fallen every grunt stops; only a dying one finishes fading.
        if !playing && self.is_alive() {
            return;
        }
        self.mode = match self.mode {
            GruntMode::Dying(mut fade) => {
                if fade.tick(delta) {
                    self.base_mut().queue_free();
                }
                GruntMode::Dying(fade)
            }
            GruntMode::Entering => self.enter(&values, delta),
            GruntMode::Approaching => {
                self.slide(&values, delta);
                self.approach(&values, delta)
            }
            GruntMode::WindUp(attack) | GruntMode::Active(attack) | GruntMode::Recovery(attack) => {
                self.slide(&values, delta);
                self.attack(attack, &values, delta)
            }
            GruntMode::HitStun(mut stun) => {
                self.slide(&values, delta);
                if stun.tick(delta) {
                    GruntMode::Approaching
                } else {
                    GruntMode::HitStun(stun)
                }
            }
        };
    }
}

#[godot_api]
impl Grunt {
    /// Emitted when a blow lands on it. `direction` is the push (−1 left, 1 right); `strength` is
    /// the knockback distance in pixels. Handlers must not call back into the grunt.
    #[signal]
    fn was_hit(direction: f32, strength: f32);

    /// Emitted once, when its health reaches zero; it frees itself `run.death_fade_time` later.
    /// Handlers must not call back into the grunt.
    #[signal]
    fn died();

    /// Whether it is winding up; drives the wind-up colour.
    #[func]
    pub fn is_winding_up(&self) -> bool {
        match self.mode {
            GruntMode::WindUp(_) => true,
            GruntMode::Entering
            | GruntMode::Approaching
            | GruntMode::Active(_)
            | GruntMode::Recovery(_)
            | GruntMode::HitStun(_)
            | GruntMode::Dying(_) => false,
        }
    }

    /// Whether it is alive: false once dying. The spawner counts living grunts with it; an
    /// entering grunt is alive.
    #[func]
    pub fn is_alive(&self) -> bool {
        match self.mode {
            GruntMode::Dying(_) => false,
            GruntMode::Entering
            | GruntMode::Approaching
            | GruntMode::WindUp(_)
            | GruntMode::Active(_)
            | GruntMode::Recovery(_)
            | GruntMode::HitStun(_) => true,
        }
    }

    /// Health points left, for its bar; `0.0` before the first tick with tuning.
    #[func]
    pub fn health(&self) -> f32 {
        self.health.map_or(0.0, |h| h.current())
    }

    /// Health points when full, for its bar; `0.0` before the first tick with tuning.
    #[func]
    pub fn max_health(&self) -> f32 {
        self.health.map_or(0.0, |h| h.max())
    }

    /// Its hurtbox in screen pixels, for the debug hitbox drawing; empty while it enters or dies,
    /// when it cannot be hit, and while there is no tuning.
    #[func]
    pub fn hurtbox(&self) -> Rect2 {
        match self.mode {
            GruntMode::Entering | GruntMode::Dying(_) => Rect2::default(),
            GruntMode::Approaching
            | GruntMode::WindUp(_)
            | GruntMode::Active(_)
            | GruntMode::Recovery(_)
            | GruntMode::HitStun(_) => self
                .hurtbox_rect()
                .map_or_else(Rect2::default, |hurt| to_rect2(&hurt)),
        }
    }

    /// The area its blow can land in, in screen pixels, while its attack is active; empty
    /// otherwise. For the debug hitbox drawing.
    #[func]
    pub fn attack_box(&self) -> Rect2 {
        match self.mode {
            GruntMode::Active(attack) => self
                .attacker(&attack.timer.spec())
                .map_or_else(Rect2::default, |a| to_rect2(&a.attack_box())),
            GruntMode::Entering
            | GruntMode::Approaching
            | GruntMode::WindUp(_)
            | GruntMode::Recovery(_)
            | GruntMode::HitStun(_)
            | GruntMode::Dying(_) => Rect2::default(),
        }
    }
}

impl Grunt {
    /// Sets where it starts, just off screen at `edge`, before the spawner adds it to the scene.
    pub fn place(&mut self, edge: Facing, position: Vector2) {
        self.edge = edge;
        self.facing = match edge {
            Facing::Left => Facing::Right,
            Facing::Right => Facing::Left,
        };
        self.base_mut().set_position(position);
    }

    /// How the hit rule sees it, or `None` while there is no tuning.
    pub fn target(&self) -> Option<Target<InstanceId>> {
        Some(Target {
            id: self.base().instance_id(),
            team: Team::Enemies,
            hurtbox: self.hurtbox_rect()?,
            feet_y: self.base().get_position().y,
            alive: self.is_alive(),
            invulnerable: false,
        })
    }

    /// Takes a blow from a hero: health drops, the knockback starts and `was_hit` is emitted; it is
    /// stunned with any attack cancelled, or, at zero health, starts dying: it leaves the enemies'
    /// group at once, emits `died` and the run counts a kill.
    pub fn take_hit(&mut self, hit: &Hit<InstanceId>) {
        let Some(links) = &self.links else {
            return;
        };
        let Some(values) = links.tuning.bind().values() else {
            return;
        };
        let mut run = links.run.clone();
        let Some(health) = &mut self.health else {
            return;
        };
        let damage = health.damage(hit.damage);
        self.mode = grunt_after_hit(
            self.mode,
            damage,
            values.grunt().hit_stun_time(),
            values.run().death_fade_time(),
        );
        let start_x = self.base().get_position().x;
        self.knockback = Knockback::start(start_x, hit.direction, hit.knockback);
        self.signals()
            .was_hit()
            .emit(hit.direction, hit.knockback.distance);
        match damage {
            Damage::Killed => {
                self.base_mut().remove_from_group(Team::Enemies.group());
                self.signals().died().emit();
                run.bind_mut().add_kill();
            }
            Damage::Survived | Damage::AlreadyDead => {}
        }
    }

    /// One tick of walking in from its edge; joins the enemies once fully on screen.
    fn enter(&mut self, values: &TuningValues, delta: f32) -> GruntMode {
        let half_width = values.grunt().hurtbox_width() / 2.0;
        let screen_width = self.base().get_viewport_rect().size.x;
        let mut position = self.base().get_position();
        let inward = match self.edge {
            Facing::Left => 1.0,
            Facing::Right => -1.0,
        };
        position.x += inward * values.grunt().horizontal_speed() * delta;
        let on_screen = match self.edge {
            Facing::Left => position.x >= half_width,
            Facing::Right => position.x <= screen_width - half_width,
        };
        self.base_mut().set_position(position);
        if on_screen {
            self.base_mut().add_to_group(Team::Enemies.group());
            GruntMode::Approaching
        } else {
            GruntMode::Entering
        }
    }

    /// One tick of hunting: faces the nearest living hero, starts an attack if its blow would land
    /// now, and otherwise walks towards it, depth first.
    fn approach(&mut self, values: &TuningValues, delta: f32) -> GruntMode {
        let Some(target) = self.nearest_target() else {
            return GruntMode::Approaching;
        };
        let position = self.base().get_position();
        let target_x = (target.hurtbox.left + target.hurtbox.right) / 2.0;
        self.facing = self.facing.after_input(target_x - position.x);
        let spec = AttackSpec::from_tuning(&values.grunt().attack());
        let Some(attacker) = self.attacker(&spec) else {
            return GruntMode::Approaching;
        };
        if !resolve_attack(&attacker, &[target], &[]).is_empty() {
            return GruntMode::WindUp(Attack::new(spec));
        }
        let grunt = values.grunt();
        let approach = Approach {
            reach: spec.reach,
            own_half_width: grunt.hurtbox_width() / 2.0,
            target_half_width: (target.hurtbox.right - target.hurtbox.left) / 2.0,
            speeds: Speeds {
                horizontal: grunt.horizontal_speed(),
                vertical: grunt.vertical_speed(),
            },
            depth_tolerance: values.depth_tolerance(),
        };
        let (x, y) = approach_step(
            (position.x, position.y),
            (target_x, target.feet_y),
            &approach,
            delta,
        );
        self.set_clamped(Vector2::new(x, y), values);
        GruntMode::Approaching
    }

    /// One tick of an attack: lands the blow on the nearest hero at most once while active, and
    /// goes back to hunting when the recovery ends.
    fn attack(&mut self, mut attack: Attack, values: &TuningValues, delta: f32) -> GruntMode {
        let advance = attack.timer.advance(delta);
        if advance.was_active && !attack.landed {
            attack.landed = self.land_blow(&attack.timer.spec(), values);
        }
        match advance.step {
            AttackStep::Continue => attack.mode(),
            AttackStep::Finished => GruntMode::Approaching,
        }
    }

    /// Lands the blow on every hero it reaches; true if it hit anyone. Play freezes once.
    fn land_blow(&mut self, spec: &AttackSpec, values: &TuningValues) -> bool {
        let Some(attacker) = self.attacker(spec) else {
            return false;
        };
        let mut heroes = FighterRef::in_team(&self.base().get_tree(), Team::Players);
        let targets: Vec<Target<InstanceId>> =
            heroes.iter().filter_map(FighterRef::target).collect();
        let hits = resolve_attack(&attacker, &targets, &[]);
        for hit in &hits {
            if let Some(hero) = heroes.iter_mut().find(|h| h.id() == hit.target) {
                hero.take_hit(hit);
            }
        }
        let freeze = values.effects().hit_stop_time();
        if !hits.is_empty()
            && freeze > 0.0
            && let Some(links) = &self.links
        {
            links.hit_stop.clone().bind_mut().freeze(freeze);
        }
        !hits.is_empty()
    }

    /// Moves any knockback slide on by one tick, inside the lane and the screen.
    fn slide(&mut self, values: &TuningValues, delta: f32) {
        let Some(mut knockback) = self.knockback else {
            return;
        };
        let mut position = self.base().get_position();
        position.x = knockback.advance(delta);
        let stopped_at_limit = self.set_clamped(position, values);
        self.knockback = (!knockback.is_done() && !stopped_at_limit).then_some(knockback);
    }

    /// Moves to `position` kept inside the lane and the screen; true if a limit stopped it.
    fn set_clamped(&mut self, position: Vector2, values: &TuningValues) -> bool {
        let half_width = values.grunt().hurtbox_width() / 2.0;
        let screen_width = self.base().get_viewport_rect().size.x;
        let (x, y) = combat::clamp_to_area(
            (position.x, position.y),
            half_width,
            screen_width,
            values.lane_top(),
            values.lane_bottom(),
        );
        self.base_mut().set_position(Vector2::new(x, y));
        x != position.x
    }

    /// The nearest hero that can be hit by the rule's own view (alive), or `None`.
    fn nearest_target(&self) -> Option<Target<InstanceId>> {
        let position = self.base().get_position();
        FighterRef::in_team(&self.base().get_tree(), Team::Players)
            .iter()
            .filter_map(FighterRef::target)
            .filter(|target| target.alive)
            .min_by(|a, b| {
                let distance = |t: &Target<InstanceId>| {
                    let x = (t.hurtbox.left + t.hurtbox.right) / 2.0;
                    (x - position.x).hypot(t.feet_y - position.y)
                };
                distance(a).total_cmp(&distance(b))
            })
    }

    /// The grunt as an attacker with `spec`, from its current position and facing.
    fn attacker(&self, spec: &AttackSpec) -> Option<Attacker> {
        let values = self.links.as_ref()?.tuning.bind().values()?;
        let position = self.base().get_position();
        Some(Attacker {
            team: Team::Enemies,
            origin: (position.x, position.y),
            facing: self.facing,
            half_width: values.grunt().hurtbox_width() / 2.0,
            spec: *spec,
            depth_tolerance: values.depth_tolerance(),
        })
    }

    /// The hurtbox in screen pixels, or `None` while there is no tuning.
    fn hurtbox_rect(&self) -> Option<combat::Rect> {
        let values = self.links.as_ref()?.tuning.bind().values()?;
        let position = self.base().get_position();
        Some(combat::hurtbox(
            (position.x, position.y),
            values.grunt().hurtbox_width(),
            values.grunt().hurtbox_height(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fighter::KnockbackSpec;

    // Values deliberately different from the starting values in tuning.json.
    const STUN: f32 = 0.35;
    const FADE: f32 = 0.4;

    fn attack() -> Attack {
        Attack::new(AttackSpec {
            wind_up: 0.5,
            active: 0.15,
            recovery: 0.3,
            reach: 55.0,
            height: 45.0,
            damage: 12.0,
            knockback: KnockbackSpec {
                distance: 70.0,
                time: 0.3,
            },
        })
    }

    #[test]
    fn a_hit_during_any_attack_part_cancels_the_attack() {
        for mode in [
            GruntMode::Approaching,
            GruntMode::WindUp(attack()),
            GruntMode::Active(attack()),
            GruntMode::Recovery(attack()),
            GruntMode::HitStun(Countdown::new(0.1)),
        ] {
            assert_eq!(
                grunt_after_hit(mode, Damage::Survived, STUN, FADE),
                GruntMode::HitStun(Countdown::new(STUN)),
                "{mode:?}"
            );
        }
    }

    #[test]
    fn the_killing_blow_starts_the_fade() {
        assert_eq!(
            grunt_after_hit(GruntMode::WindUp(attack()), Damage::Killed, STUN, FADE),
            GruntMode::Dying(Countdown::new(FADE))
        );
    }

    #[test]
    fn entering_and_dying_grunts_are_not_changed_by_a_blow() {
        for mode in [GruntMode::Entering, GruntMode::Dying(Countdown::new(0.2))] {
            for damage in [Damage::Survived, Damage::Killed] {
                assert_eq!(grunt_after_hit(mode, damage, STUN, FADE), mode, "{mode:?}");
            }
        }
    }
}
