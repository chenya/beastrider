//! The enemies: every kind that fights back, the grunt and the brute.
//!
//! An enemy walks in from a screen edge, lines up at the nearest hero's depth, closes in to its
//! reach, winds up visibly and strikes. A blow stuns it and cancels its attack; at zero health it
//! fades out and is removed, and the run counts a kill and rolls a potion drop, which the wave
//! director places. The hero's spell damages it and, if it survives, knocks it down: it lies
//! still, untouchable by blows and holding no turn, then gets up and retreats. One class serves every kind: its exported
//! [`EnemyKind`] only chooses which tuning section it reads, so behaviour code never branches on
//! the kind (research R2). Rust owns all of it; the scene only gives it a body, a health bar and
//! their scripts, which read from it and react to its signals.

use godot::classes::{INode2D, Node2D};
use godot::prelude::*;

use crate::combat::{self, Facing};
use crate::crowd::{Body, Give};
use crate::fighter::{
    Approach, AttackPhase, AttackSpec, AttackStep, AttackTimer, Attacker, Countdown, Damage,
    Health, Hit, Knockback, KnockbackSpec, Target, Team, approach_step, resolve_attack, towards,
};
use crate::fighters::FighterRef;
use crate::hit_stop::HitStopTimer;
use crate::movement::Speeds;
use crate::potion::PotionKind;
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

/// Which kind an enemy is. It only chooses the enemy's tuning section (`enemies.grunt` or
/// `enemies.brute`) and, in the wave director, its scene; set in `grunt.tscn` and `brute.tscn`.
/// Exported as its name, so the editor shows a drop-down.
#[derive(GodotConvert, Var, Export, Debug, Clone, Copy, PartialEq, Eq)]
#[godot(via = GString)]
pub enum EnemyKind {
    /// Walks to the hero and strikes; the M3 enemy.
    Grunt,
    /// Slower, hits harder, takes more blows and barely flinches.
    Brute,
}

/// An enemy that hunts the nearest hero, origin at its feet so Y-sort draws it by depth.
///
/// Placed only by the `WaveDirector`, which calls [`place`](Self::place) before adding it and
/// hands out its turns. It joins the enemies' group once it has walked fully on screen and leaves it the moment it starts dying,
/// so it can be neither hit nor counted as a target before or after. If an autoload is missing, it
/// reports one error and never moves. Never give its root node a script: a GDScript `_ready` would
/// replace this class's `ready()`.
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct Enemy {
    /// Which kind it is; every value it uses comes from this kind's tuning section.
    #[export]
    kind: EnemyKind,
    /// The autoloads it reads every tick; `None` until `ready()` finds them, and for good if not.
    links: Option<Links>,
    /// What it is doing.
    mode: EnemyMode,
    /// Health points; `None` until the first tick with tuning, which starts it full.
    health: Option<Health>,
    /// The way it faces: towards its target.
    facing: Facing,
    /// The slide from the last blow taken, if it is still under way.
    knockback: Option<Knockback>,
    /// The screen edge it enters from: `Left` walks in rightwards, `Right` leftwards.
    edge: Facing,
    /// Its side of the hero while it waits: −1 left, 1 right (its entry edge), fixed at `place()`.
    side: f32,
    /// The depth it waits at: its spawn depth, fixed at `place()`.
    waiting_y: f32,
    /// The potion its death left, until the wave director places it (research R10).
    drop: Option<PotionKind>,
    base: Base<Node2D>,
}

/// The autoloads an enemy reads every tick, found once in `ready()`.
struct Links {
    tuning: Gd<TuningStore>,
    hit_stop: Gd<HitStopTimer>,
    run: Gd<RunTracker>,
}

/// What an enemy is doing. Every use is an exhaustive `match`, so a new mode cannot be forgotten.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyMode {
    /// Walking in from its edge; cannot be hit and does not attack.
    Entering,
    /// Without a turn: at or going to its waiting spot on its own side of its target, facing it,
    /// never attacking. The wave director grants it a turn with `grant_turn`. Holds the seconds it
    /// has been waiting, so one passed over for too long goes first.
    Waiting(f32),
    /// Its turn ended (end of recovery) or it was stunned: going back to its waiting spot like a
    /// waiting enemy, but it cannot get a turn until it is back, so turns rotate (planning answer).
    Retreating,
    /// Holding a turn: lining up at its target's depth, then closing in to its reach.
    Approaching,
    /// Winding up: visibly marked, and neither moving nor turning.
    WindUp(Attack),
    /// The blow can land on its target, once per attack.
    Active(Attack),
    /// After the blow, before it hunts again.
    Recovery(Attack),
    /// Stunned by a blow: no moving or attacking until the countdown runs out.
    HitStun(Countdown),
    /// Knocked flat by the spell: no attack, no turn, and blows cannot land, so it cannot be
    /// juggled; it slides away from the caster, then gets up and retreats to its spot (research
    /// R7).
    KnockedDown(Countdown),
    /// Fading out after its health reached zero; removed when the countdown runs out.
    Dying(Countdown),
}

impl EnemyMode {
    /// Whether an enemy in this mode holds a turn: from approaching to the end of its recovery.
    /// The director frees its tokens as soon as this turns false, so a turn ends at the end of the
    /// recovery, on a stun and on death without a separate flag (research R4).
    pub fn holds_turn(self) -> bool {
        match self {
            Self::Approaching | Self::WindUp(_) | Self::Active(_) | Self::Recovery(_) => true,
            Self::Entering
            | Self::Waiting(_)
            | Self::Retreating
            | Self::HitStun(_)
            | Self::KnockedDown(_)
            | Self::Dying(_) => false,
        }
    }
}

/// How readily an enemy in `mode` gives way in the separation pass: attacking holds, approaching
/// presses, the rest yield; `None` while it enters or dies, when it is left out.
pub fn crowd_give(mode: EnemyMode) -> Option<Give> {
    match mode {
        EnemyMode::Entering | EnemyMode::Dying(_) => None,
        EnemyMode::Waiting(_)
        | EnemyMode::Retreating
        | EnemyMode::HitStun(_)
        | EnemyMode::KnockedDown(_) => Some(Give::Yields),
        EnemyMode::Approaching => Some(Give::Presses),
        EnemyMode::WindUp(_) | EnemyMode::Active(_) | EnemyMode::Recovery(_) => Some(Give::Holds),
    }
}

/// One tick of lying knocked down: still down until the countdown runs out, then it retreats to
/// its spot, so it cannot take a turn the moment it rises.
pub fn get_up(down: &mut Countdown, delta: f32) -> EnemyMode {
    if down.tick(delta) {
        EnemyMode::Retreating
    } else {
        EnemyMode::KnockedDown(*down)
    }
}

/// What the spell's impact does to an enemy in `mode`: a killing hit starts the fade; any other
/// knocks it down for `knockdown_time`, whatever it was doing, an armored attack included. Its
/// resistance plays no part: the spell floors brutes too (spec FR-009a). A dying enemy is left
/// alone.
pub fn enemy_after_spell(
    mode: EnemyMode,
    damage: Damage,
    knockdown_time: f32,
    death_fade_time: f32,
) -> EnemyMode {
    match mode {
        EnemyMode::Dying(_) => mode,
        EnemyMode::Entering
        | EnemyMode::Waiting(_)
        | EnemyMode::Retreating
        | EnemyMode::Approaching
        | EnemyMode::WindUp(_)
        | EnemyMode::Active(_)
        | EnemyMode::Recovery(_)
        | EnemyMode::HitStun(_)
        | EnemyMode::KnockedDown(_) => match damage {
            Damage::Killed => EnemyMode::Dying(Countdown::new(death_fade_time)),
            Damage::Survived => EnemyMode::KnockedDown(Countdown::new(knockdown_time)),
            Damage::AlreadyDead => mode,
        },
    }
}

/// `position` moved towards `spot` by at most `speed · delta` on each axis, never overshooting; a
/// speed of 0 holds it still. A waiting enemy drifts to its spot with it.
pub fn drift_step(position: (f32, f32), spot: (f32, f32), speed: f32, delta: f32) -> (f32, f32) {
    let step = speed * delta;
    (
        towards(position.0, spot.0, step),
        towards(position.1, spot.1, step),
    )
}

/// Whether `position` is within `radius` of `spot` (the separation distance, as other enemies may
/// keep it from the exact spot).
pub fn near_spot(position: (f32, f32), spot: (f32, f32), radius: f32) -> bool {
    (spot.0 - position.0).hypot(spot.1 - position.1) <= radius
}

/// Whether a retreating enemy at `position` is back: near its `spot`, or at least `waiting_distance
/// − radius` from its target horizontally, on the waiting ring wherever the crowd lets it stand.
/// The second rule keeps a crowded side from holding enemies in retreat for good.
pub fn back_at_spot(
    position: (f32, f32),
    spot: (f32, f32),
    target_x: f32,
    waiting_distance: f32,
    radius: f32,
) -> bool {
    near_spot(position, spot, radius) || (position.0 - target_x).abs() >= waiting_distance - radius
}

/// One tick of moving towards a waiting spot (planning answer): farther than `radius` from it, the
/// enemy walks there at its own `walk` speeds; within it, it drifts at `drift_speed` to follow
/// the spot as the hero moves (0: it stops once there). Never overshoots.
pub fn spot_step(
    position: (f32, f32),
    spot: (f32, f32),
    walk: Speeds,
    drift_speed: f32,
    radius: f32,
    delta: f32,
) -> (f32, f32) {
    if near_spot(position, spot, radius) {
        drift_step(position, spot, drift_speed, delta)
    } else {
        (
            towards(position.0, spot.0, walk.horizontal * delta),
            towards(position.1, spot.1, walk.vertical * delta),
        )
    }
}

/// Where one tick of moving to the waiting spot left an enemy.
struct SpotMove {
    /// Where it stands now.
    position: (f32, f32),
    /// Its waiting spot, kept inside the lane and the screen.
    spot: (f32, f32),
    /// Its target's x.
    target_x: f32,
}

/// One enemy attack under way.
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
    fn mode(self) -> EnemyMode {
        match self.timer.phase() {
            AttackPhase::WindUp => EnemyMode::WindUp(self),
            AttackPhase::Active => EnemyMode::Active(self),
            AttackPhase::Recovery => EnemyMode::Recovery(self),
        }
    }
}

/// How a blow affects one enemy: its own stun and the push it takes, scaled by its knockback
/// resistance, and whether a blow may stop its attack. Built from the hit and the enemy's tuning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reaction {
    /// Seconds it is stunned: its hit-stun time × (1 − resistance).
    pub stun_time: f32,
    /// The push it takes: the hit's distance × (1 − resistance), over the hit's own time.
    pub knockback: KnockbackSpec,
    /// Whether a blow during its wind-up, active part or recovery stuns it and drops the attack.
    pub interruptible: bool,
}

impl Reaction {
    /// The reaction of an enemy with `hit_stun_time`, `resistance` (0 to 1) and `interruptible`
    /// to a blow pushing it by `hit_knockback`. Resistance shortens the push, never slows it.
    pub fn new(
        hit_knockback: KnockbackSpec,
        hit_stun_time: f32,
        resistance: f32,
        interruptible: bool,
    ) -> Self {
        let kept = 1.0 - resistance;
        Self {
            stun_time: hit_stun_time * kept,
            knockback: KnockbackSpec {
                distance: hit_knockback.distance * kept,
                time: hit_knockback.time,
            },
            interruptible,
        }
    }
}

/// The mode after a blow lands, and whether the blow pushes the enemy. Killed → dying for
/// `death_fade_time`, pushed. A non-interruptible enemy in its wind-up, active part or recovery
/// keeps its attack and is not pushed (an armored attack). Otherwise stunned for the reaction's
/// time, dropping any attack, and pushed. Entering and dying enemies cannot be hit, so a blow leaves
/// them as they are.
pub fn enemy_after_hit(
    mode: EnemyMode,
    damage: Damage,
    reaction: &Reaction,
    death_fade_time: f32,
) -> (EnemyMode, bool) {
    let attacking = match mode {
        // Blows cannot land on these (an entering or downed enemy is not a target).
        EnemyMode::Entering | EnemyMode::Dying(_) | EnemyMode::KnockedDown(_) => {
            return (mode, false);
        }
        EnemyMode::WindUp(_) | EnemyMode::Active(_) | EnemyMode::Recovery(_) => true,
        EnemyMode::Waiting(_)
        | EnemyMode::Retreating
        | EnemyMode::Approaching
        | EnemyMode::HitStun(_) => false,
    };
    match damage {
        Damage::Killed => (EnemyMode::Dying(Countdown::new(death_fade_time)), true),
        Damage::AlreadyDead => (mode, false),
        Damage::Survived if attacking && !reaction.interruptible => (mode, false),
        Damage::Survived => (EnemyMode::HitStun(Countdown::new(reaction.stun_time)), true),
    }
}

#[godot_api]
impl INode2D for Enemy {
    fn init(base: Base<Node2D>) -> Self {
        // An enemy enters from the left unless the spawner places it; health is filled on the first
        // tick with tuning, and `links` by `ready()`.
        Self {
            kind: EnemyKind::Grunt,
            links: None,
            mode: EnemyMode::Entering,
            health: None,
            facing: Facing::Right,
            knockback: None,
            edge: Facing::Left,
            side: -1.0,
            waiting_y: 0.0,
            drop: None,
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
                    "Enemy: the Tuning, HitStop or Run autoload is missing or has the wrong type"
                );
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let Some(links) = &self.links else {
            return;
        };
        if links.hit_stop.bind().is_frozen() || links.run.bind().is_cast_frozen() {
            return;
        }
        let Some(values) = links.tuning.bind().values() else {
            return;
        };
        let playing = links.run.bind().is_playing();
        let delta = delta as f32;
        let max_health = values.enemy(self.kind).max_health();
        match &mut self.health {
            Some(health) => health.set_max(max_health),
            None => self.health = Some(Health::new(max_health)),
        }
        // Once the hero has fallen every enemy stops; only a dying one finishes fading.
        if !playing && self.is_alive() {
            return;
        }
        self.mode = match self.mode {
            EnemyMode::Dying(mut fade) => {
                if fade.tick(delta) {
                    self.base_mut().queue_free();
                }
                EnemyMode::Dying(fade)
            }
            EnemyMode::Entering => self.enter(&values, delta),
            EnemyMode::Waiting(waited) => {
                self.slide(&values, delta);
                self.wait(waited, &values, delta)
            }
            EnemyMode::Retreating => {
                self.slide(&values, delta);
                self.retreat(&values, delta)
            }
            EnemyMode::Approaching => {
                self.slide(&values, delta);
                self.approach(&values, delta)
            }
            EnemyMode::WindUp(attack) | EnemyMode::Active(attack) | EnemyMode::Recovery(attack) => {
                self.slide(&values, delta);
                self.attack(attack, &values, delta)
            }
            EnemyMode::HitStun(mut stun) => {
                self.slide(&values, delta);
                if stun.tick(delta) {
                    EnemyMode::Retreating
                } else {
                    EnemyMode::HitStun(stun)
                }
            }
            EnemyMode::KnockedDown(mut down) => {
                self.slide(&values, delta);
                get_up(&mut down, delta)
            }
        };
    }
}

#[godot_api]
impl Enemy {
    /// Emitted when a blow lands on it. `direction` is the push (−1 left, 1 right); `strength` is
    /// the knockback distance in pixels. Handlers must not call back into the enemy.
    #[signal]
    fn was_hit(direction: f32, strength: f32);

    /// Emitted once, when its health reaches zero; it frees itself `run.death_fade_time` later.
    /// Handlers must not call back into the enemy.
    #[signal]
    fn died();

    /// Whether it is winding up; drives the wind-up colour.
    #[func]
    pub fn is_winding_up(&self) -> bool {
        match self.mode {
            EnemyMode::WindUp(_) => true,
            EnemyMode::Entering
            | EnemyMode::Waiting(_)
            | EnemyMode::Retreating
            | EnemyMode::Approaching
            | EnemyMode::Active(_)
            | EnemyMode::Recovery(_)
            | EnemyMode::HitStun(_)
            | EnemyMode::KnockedDown(_)
            | EnemyMode::Dying(_) => false,
        }
    }

    /// Whether it is alive: false once dying. The spawner counts living enemies with it; an
    /// entering enemy is alive.
    #[func]
    pub fn is_alive(&self) -> bool {
        match self.mode {
            EnemyMode::Dying(_) => false,
            EnemyMode::Entering
            | EnemyMode::Waiting(_)
            | EnemyMode::Retreating
            | EnemyMode::Approaching
            | EnemyMode::WindUp(_)
            | EnemyMode::Active(_)
            | EnemyMode::Recovery(_)
            | EnemyMode::HitStun(_)
            | EnemyMode::KnockedDown(_) => true,
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
            EnemyMode::Entering | EnemyMode::Dying(_) => Rect2::default(),
            EnemyMode::Waiting(_)
            | EnemyMode::Retreating
            | EnemyMode::Approaching
            | EnemyMode::WindUp(_)
            | EnemyMode::Active(_)
            | EnemyMode::Recovery(_)
            | EnemyMode::HitStun(_)
            | EnemyMode::KnockedDown(_) => self
                .hurtbox_rect()
                .map_or_else(Rect2::default, |hurt| to_rect2(&hurt)),
        }
    }

    /// The area its blow can land in, in screen pixels, while its attack is active; empty
    /// otherwise. For the debug hitbox drawing.
    #[func]
    pub fn attack_box(&self) -> Rect2 {
        match self.mode {
            EnemyMode::Active(attack) => self
                .attacker(&attack.timer.spec())
                .map_or_else(Rect2::default, |a| to_rect2(&a.attack_box())),
            EnemyMode::Entering
            | EnemyMode::Waiting(_)
            | EnemyMode::Retreating
            | EnemyMode::Approaching
            | EnemyMode::WindUp(_)
            | EnemyMode::Recovery(_)
            | EnemyMode::HitStun(_)
            | EnemyMode::KnockedDown(_)
            | EnemyMode::Dying(_) => Rect2::default(),
        }
    }

    /// Whether it holds a turn: from approaching to the end of its recovery. For debugging and
    /// checks; the director decides turns.
    #[func]
    pub fn has_turn(&self) -> bool {
        self.mode.holds_turn()
    }

    /// Whether it is knocked down by the spell; drives the knocked-down look.
    #[func]
    pub fn is_knocked_down(&self) -> bool {
        matches!(self.mode, EnemyMode::KnockedDown(_))
    }

    /// Whether it is retreating to its spot after a turn or a stun, unable to get a turn until it
    /// is back. For debugging and windowed checks.
    #[func]
    pub fn is_retreating(&self) -> bool {
        self.mode == EnemyMode::Retreating
    }

    /// Whether it is without a turn, waiting or retreating to its spot. For the debug hitbox
    /// drawing.
    #[func]
    pub fn is_waiting(&self) -> bool {
        match self.mode {
            EnemyMode::Waiting(_) | EnemyMode::Retreating => true,
            EnemyMode::Entering
            | EnemyMode::Approaching
            | EnemyMode::WindUp(_)
            | EnemyMode::Active(_)
            | EnemyMode::Recovery(_)
            | EnemyMode::HitStun(_)
            | EnemyMode::KnockedDown(_)
            | EnemyMode::Dying(_) => false,
        }
    }
}

impl Enemy {
    /// Sets where it starts, just off screen at `edge`, before the spawner adds it to the scene.
    pub fn place(&mut self, edge: Facing, position: Vector2) {
        self.edge = edge;
        self.facing = match edge {
            Facing::Left => Facing::Right,
            Facing::Right => Facing::Left,
        };
        self.side = match edge {
            Facing::Left => -1.0,
            Facing::Right => 1.0,
        };
        self.waiting_y = position.y;
        self.base_mut().set_position(position);
    }

    /// Its distance to its target and the seconds it has waited, while it waits for a turn; `None`
    /// when it is not waiting or has no target. The director orders turns by them.
    pub fn turn_request(&self) -> Option<(f32, f32)> {
        let waited = match self.mode {
            EnemyMode::Waiting(waited) => waited,
            EnemyMode::Entering
            | EnemyMode::Retreating
            | EnemyMode::Approaching
            | EnemyMode::WindUp(_)
            | EnemyMode::Active(_)
            | EnemyMode::Recovery(_)
            | EnemyMode::HitStun(_)
            | EnemyMode::KnockedDown(_)
            | EnemyMode::Dying(_) => return None,
        };
        let target = self.nearest_target()?;
        let position = self.base().get_position();
        let target_x = (target.hurtbox.left + target.hurtbox.right) / 2.0;
        Some((
            (target_x - position.x).hypot(target.feet_y - position.y),
            waited,
        ))
    }

    /// The attack tokens its kind costs, or `None` while there is no tuning.
    pub fn turn_cost(&self) -> Option<u32> {
        let values = self.links.as_ref()?.tuning.bind().values()?;
        Some(values.enemy(self.kind).attack_token_cost())
    }

    /// The director granted it a turn: a waiting enemy starts approaching. Does nothing in any
    /// other mode (the director then frees the token on its next pass).
    pub fn grant_turn(&mut self) {
        match self.mode {
            EnemyMode::Waiting(_) => self.mode = EnemyMode::Approaching,
            EnemyMode::Entering
            | EnemyMode::Retreating
            | EnemyMode::Approaching
            | EnemyMode::WindUp(_)
            | EnemyMode::Active(_)
            | EnemyMode::Recovery(_)
            | EnemyMode::HitStun(_)
            | EnemyMode::KnockedDown(_)
            | EnemyMode::Dying(_) => {}
        }
    }

    /// How the separation pass sees it: its feet and how readily it gives way (attacking holds,
    /// approaching presses, waiting, retreating and stunned yield), or `None` while it enters or
    /// dies, when it is left out.
    pub fn crowd_body(&self) -> Option<Body> {
        let give = crowd_give(self.mode)?;
        let position = self.base().get_position();
        Some(Body {
            x: position.x,
            y: position.y,
            give,
        })
    }

    /// Moves it by the separation pass's offset, kept inside the lane and the screen.
    pub fn nudge(&mut self, dx: f32, dy: f32) {
        let Some(values) = self.links.as_ref().and_then(|l| l.tuning.bind().values()) else {
            return;
        };
        let position = self.base().get_position() + Vector2::new(dx, dy);
        self.set_clamped(position, &values);
    }

    /// How the hit rule sees it, or `None` while there is no tuning.
    pub fn target(&self) -> Option<Target<InstanceId>> {
        Some(Target {
            id: self.base().instance_id(),
            team: Team::Enemies,
            hurtbox: self.hurtbox_rect()?,
            feet_y: self.base().get_position().y,
            alive: self.is_alive(),
            // Downed by the spell: blows cannot land, so it cannot be juggled.
            invulnerable: self.is_knocked_down(),
        })
    }

    /// Takes the spell's impact from a caster at `from_x` (research R6, R7): health drops by
    /// `damage`; a kill counts like a blow's (points, a kill, a potion roll); a survivor is knocked
    /// down for `magic.knockdown_time` and slides `magic.knockdown_push_distance` away from the
    /// caster, its resistance and armor ignored. `was_hit` is emitted with that slide, so the
    /// flash and shake play as for a blow.
    pub fn take_spell(&mut self, damage: f32, from_x: f32) {
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
        let damage = health.damage(damage);
        let magic = values.magic();
        let was_entering = self.mode == EnemyMode::Entering;
        self.mode = enemy_after_spell(
            self.mode,
            damage,
            magic.knockdown_time(),
            values.run().death_fade_time(),
        );
        let x = self.base().get_position().x;
        let direction = if x >= from_x { 1.0 } else { -1.0 };
        let strength = match self.mode {
            EnemyMode::KnockedDown(_) => {
                let push = KnockbackSpec {
                    distance: magic.knockdown_push_distance(),
                    time: magic.knockdown_push_time(),
                };
                self.knockback = Knockback::start(x, direction, push);
                // An enemy floored while still walking in joins the enemies now, so blows can
                // reach it once it rises.
                if was_entering {
                    self.base_mut().add_to_group(Team::Enemies.group());
                }
                push.distance
            }
            EnemyMode::Entering
            | EnemyMode::Waiting(_)
            | EnemyMode::Retreating
            | EnemyMode::Approaching
            | EnemyMode::WindUp(_)
            | EnemyMode::Active(_)
            | EnemyMode::Recovery(_)
            | EnemyMode::HitStun(_)
            | EnemyMode::Dying(_) => 0.0,
        };
        self.signals().was_hit().emit(direction, strength);
        match damage {
            Damage::Killed => self.count_death(&values, &mut run),
            Damage::Survived | Damage::AlreadyDead => {}
        }
    }

    /// Takes a blow from a hero: health drops and `was_hit` is emitted with the push it takes (0
    /// for an armored attack); per [`enemy_after_hit`] it is pushed and stunned with any attack
    /// cancelled, keeps attacking, or, at zero health, starts dying: it leaves the enemies'
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
        let own = values.enemy(self.kind);
        let reaction = Reaction::new(
            hit.knockback,
            own.hit_stun_time(),
            own.knockback_resistance(),
            own.interruptible(),
        );
        let (mode, push) =
            enemy_after_hit(self.mode, damage, &reaction, values.run().death_fade_time());
        self.mode = mode;
        let strength = if push {
            let start_x = self.base().get_position().x;
            self.knockback = Knockback::start(start_x, hit.direction, reaction.knockback);
            reaction.knockback.distance
        } else {
            0.0
        };
        self.signals().was_hit().emit(hit.direction, strength);
        match damage {
            Damage::Killed => self.count_death(&values, &mut run),
            Damage::Survived | Damage::AlreadyDead => {}
        }
    }

    /// What every death does, by a blow or the spell: it leaves the enemies, says it died, adds
    /// its points and a kill, and rolls its potion.
    fn count_death(&mut self, values: &TuningValues, run: &mut Gd<RunTracker>) {
        self.base_mut().remove_from_group(Team::Enemies.group());
        self.signals().died().emit();
        let own = values.enemy(self.kind);
        let mut run = run.bind_mut();
        run.add_kill(own.points());
        self.drop = run.roll_drop(own.drop_chance(), values.potions().health_odds());
    }

    /// The potion its death left and where it fell, once; the wave director places it.
    pub fn take_drop(&mut self) -> Option<(PotionKind, Vector2)> {
        let kind = self.drop.take()?;
        Some((kind, self.base().get_position()))
    }

    /// One tick of walking in from its edge; joins the enemies once fully on screen.
    fn enter(&mut self, values: &TuningValues, delta: f32) -> EnemyMode {
        let half_width = values.enemy(self.kind).hurtbox_width() / 2.0;
        let screen_width = self.base().get_viewport_rect().size.x;
        let mut position = self.base().get_position();
        let inward = match self.edge {
            Facing::Left => 1.0,
            Facing::Right => -1.0,
        };
        position.x += inward * values.enemy(self.kind).horizontal_speed() * delta;
        let on_screen = match self.edge {
            Facing::Left => position.x >= half_width,
            Facing::Right => position.x <= screen_width - half_width,
        };
        self.base_mut().set_position(position);
        if on_screen {
            self.base_mut().add_to_group(Team::Enemies.group());
            EnemyMode::Waiting(0.0)
        } else {
            EnemyMode::Entering
        }
    }

    /// One tick of hunting: faces the nearest living hero, starts an attack if its blow would land
    /// now, and otherwise walks towards it, depth first.
    fn approach(&mut self, values: &TuningValues, delta: f32) -> EnemyMode {
        let Some(target) = self.nearest_target() else {
            return EnemyMode::Approaching;
        };
        let position = self.base().get_position();
        let target_x = (target.hurtbox.left + target.hurtbox.right) / 2.0;
        self.facing = self.facing.after_input(target_x - position.x);
        let spec = AttackSpec::from_tuning(&values.enemy(self.kind).attack());
        let Some(attacker) = self.attacker(&spec) else {
            return EnemyMode::Approaching;
        };
        if !resolve_attack(&attacker, &[target], &[]).is_empty() {
            return EnemyMode::WindUp(Attack::new(spec));
        }
        let own = values.enemy(self.kind);
        let approach = Approach {
            reach: spec.reach,
            own_half_width: own.hurtbox_width() / 2.0,
            target_half_width: (target.hurtbox.right - target.hurtbox.left) / 2.0,
            speeds: Speeds {
                horizontal: own.horizontal_speed(),
                vertical: own.vertical_speed(),
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
        EnemyMode::Approaching
    }

    /// One tick of waiting for a turn: faces the nearest living hero and keeps to its spot. It
    /// never attacks.
    fn wait(&mut self, waited: f32, values: &TuningValues, delta: f32) -> EnemyMode {
        self.drift_to_spot(values, delta);
        EnemyMode::Waiting(waited + delta)
    }

    /// One tick of retreating after a turn or a stun: goes to its spot like a waiting enemy, and
    /// waits again once it is back there (see [`back_at_spot`]).
    fn retreat(&mut self, values: &TuningValues, delta: f32) -> EnemyMode {
        let Some(SpotMove {
            position,
            spot,
            target_x,
        }) = self.drift_to_spot(values, delta)
        else {
            return EnemyMode::Retreating;
        };
        let crowd = values.crowd();
        if back_at_spot(
            position,
            spot,
            target_x,
            crowd.waiting_distance(),
            crowd.separation_radius(),
        ) {
            EnemyMode::Waiting(0.0)
        } else {
            EnemyMode::Retreating
        }
    }

    /// Faces the nearest living hero and moves towards its waiting spot (see [`spot_step`]): on its
    /// own side of the hero, the waiting distance away, at its own depth, kept inside the lane and
    /// the screen.
    /// Returns where it now stands, the spot and the hero's x, or `None` with no hero to wait for.
    fn drift_to_spot(&mut self, values: &TuningValues, delta: f32) -> Option<SpotMove> {
        let target = self.nearest_target()?;
        let position = self.base().get_position();
        let target_x = (target.hurtbox.left + target.hurtbox.right) / 2.0;
        self.facing = self.facing.after_input(target_x - position.x);
        let crowd = values.crowd();
        let screen_width = self.base().get_viewport_rect().size.x;
        let spot = combat::clamp_to_area(
            (
                target_x + self.side * crowd.waiting_distance(),
                self.waiting_y,
            ),
            values.enemy(self.kind).hurtbox_width() / 2.0,
            screen_width,
            values.lane_top(),
            values.lane_bottom(),
        );
        let own = values.enemy(self.kind);
        let walk = Speeds {
            horizontal: own.horizontal_speed(),
            vertical: own.vertical_speed(),
        };
        let (x, y) = spot_step(
            (position.x, position.y),
            spot,
            walk,
            crowd.drift_speed(),
            crowd.separation_radius(),
            delta,
        );
        self.set_clamped(Vector2::new(x, y), values);
        Some(SpotMove {
            position: (x, y),
            spot,
            target_x,
        })
    }

    /// One tick of an attack: lands the blow on the nearest hero at most once while active, and
    /// retreats to its spot when the recovery ends, freeing its turn.
    fn attack(&mut self, mut attack: Attack, values: &TuningValues, delta: f32) -> EnemyMode {
        let advance = attack.timer.advance(delta);
        if advance.was_active && !attack.landed {
            attack.landed = self.land_blow(&attack.timer.spec(), values);
        }
        match advance.step {
            AttackStep::Continue => attack.mode(),
            AttackStep::Finished => EnemyMode::Retreating,
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
        let half_width = values.enemy(self.kind).hurtbox_width() / 2.0;
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

    /// The enemy as an attacker with `spec`, from its current position and facing.
    fn attacker(&self, spec: &AttackSpec) -> Option<Attacker> {
        let values = self.links.as_ref()?.tuning.bind().values()?;
        let position = self.base().get_position();
        Some(Attacker {
            team: Team::Enemies,
            origin: (position.x, position.y),
            facing: self.facing,
            half_width: values.enemy(self.kind).hurtbox_width() / 2.0,
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
            values.enemy(self.kind).hurtbox_width(),
            values.enemy(self.kind).hurtbox_height(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values deliberately different from the starting values in tuning.json.
    const STUN: f32 = 0.35;
    const FADE: f32 = 0.4;
    const PUSH: KnockbackSpec = KnockbackSpec {
        distance: 70.0,
        time: 0.3,
    };

    fn attack() -> Attack {
        Attack::new(AttackSpec {
            wind_up: 0.5,
            active: 0.15,
            recovery: 0.3,
            reach: 55.0,
            height: 45.0,
            damage: 12.0,
            knockback: PUSH,
        })
    }

    fn attacking_modes() -> [EnemyMode; 3] {
        [
            EnemyMode::WindUp(attack()),
            EnemyMode::Active(attack()),
            EnemyMode::Recovery(attack()),
        ]
    }

    // --- The spell's knockdown (M4b). ---

    const KNOCKDOWN: f32 = 0.9;

    /// Every mode the spell can find an enemy in, but dying.
    fn spell_targets() -> Vec<EnemyMode> {
        let mut modes = vec![
            EnemyMode::Entering,
            EnemyMode::Waiting(1.5),
            EnemyMode::Retreating,
            EnemyMode::Approaching,
            EnemyMode::HitStun(Countdown::new(STUN)),
            EnemyMode::KnockedDown(Countdown::new(0.2)),
        ];
        modes.extend(attacking_modes());
        modes
    }

    #[test]
    fn the_spell_knocks_down_every_survivor_whatever_it_was_doing() {
        // No resistance or armor goes into the rule: brutes go down like grunts.
        for mode in spell_targets() {
            assert_eq!(
                enemy_after_spell(mode, Damage::Survived, KNOCKDOWN, FADE),
                EnemyMode::KnockedDown(Countdown::new(KNOCKDOWN)),
                "{mode:?}"
            );
        }
    }

    #[test]
    fn the_spell_kills_like_a_blow_and_leaves_the_dying_alone() {
        for mode in spell_targets() {
            assert_eq!(
                enemy_after_spell(mode, Damage::Killed, KNOCKDOWN, FADE),
                EnemyMode::Dying(Countdown::new(FADE)),
                "{mode:?}"
            );
        }
        let dying = EnemyMode::Dying(Countdown::new(0.1));
        for damage in [Damage::Survived, Damage::Killed, Damage::AlreadyDead] {
            assert_eq!(enemy_after_spell(dying, damage, KNOCKDOWN, FADE), dying);
        }
        let waiting = EnemyMode::Waiting(0.5);
        assert_eq!(
            enemy_after_spell(waiting, Damage::AlreadyDead, KNOCKDOWN, FADE),
            waiting
        );
    }

    #[test]
    fn a_knocked_down_enemy_holds_no_turn_and_yields() {
        let down = EnemyMode::KnockedDown(Countdown::new(KNOCKDOWN));
        assert!(!down.holds_turn());
        assert_eq!(crowd_give(down), Some(Give::Yields));
    }

    #[test]
    fn a_knocked_down_enemy_gets_up_and_retreats() {
        let mut down = Countdown::new(0.25);
        assert_eq!(
            get_up(&mut down, 0.125),
            EnemyMode::KnockedDown(Countdown::new(0.125))
        );
        assert_eq!(get_up(&mut down, 0.125), EnemyMode::Retreating);
    }

    #[test]
    fn resistance_scales_the_push_and_stun_but_not_the_push_time() {
        let none = Reaction::new(PUSH, STUN, 0.0, true);
        assert_eq!(none.knockback, PUSH);
        assert_eq!(none.stun_time, STUN);
        let most = Reaction::new(PUSH, STUN, 0.75, true);
        assert!((most.knockback.distance - 17.5).abs() < 1e-4);
        assert_eq!(most.knockback.time, PUSH.time);
        assert!((most.stun_time - STUN * 0.25).abs() < 1e-6);
        let all = Reaction::new(PUSH, STUN, 1.0, true);
        assert_eq!(all.knockback.distance, 0.0);
        assert_eq!(all.stun_time, 0.0);
    }

    #[test]
    fn an_interruptible_enemy_is_stunned_and_pushed_in_any_mode() {
        let reaction = Reaction::new(PUSH, STUN, 0.0, true);
        let mut modes = attacking_modes().to_vec();
        modes.push(EnemyMode::Waiting(0.5));
        modes.push(EnemyMode::Retreating);
        modes.push(EnemyMode::Approaching);
        modes.push(EnemyMode::HitStun(Countdown::new(0.1)));
        for mode in modes {
            assert_eq!(
                enemy_after_hit(mode, Damage::Survived, &reaction, FADE),
                (EnemyMode::HitStun(Countdown::new(STUN)), true),
                "{mode:?}"
            );
        }
    }

    #[test]
    fn a_non_interruptible_enemy_keeps_its_attack_unpushed() {
        let reaction = Reaction::new(PUSH, STUN, 0.75, false);
        for mode in attacking_modes() {
            assert_eq!(
                enemy_after_hit(mode, Damage::Survived, &reaction, FADE),
                (mode, false),
                "{mode:?}"
            );
        }
        // Outside an attack it is still stunned (briefly) and pushed.
        assert_eq!(
            enemy_after_hit(EnemyMode::Approaching, Damage::Survived, &reaction, FADE),
            (EnemyMode::HitStun(Countdown::new(reaction.stun_time)), true)
        );
    }

    #[test]
    fn the_killing_blow_starts_the_fade_even_mid_armored_attack() {
        let reaction = Reaction::new(PUSH, STUN, 0.75, false);
        assert_eq!(
            enemy_after_hit(EnemyMode::WindUp(attack()), Damage::Killed, &reaction, FADE),
            (EnemyMode::Dying(Countdown::new(FADE)), true)
        );
    }

    #[test]
    fn entering_and_dying_enemies_are_not_changed_by_a_blow() {
        let reaction = Reaction::new(PUSH, STUN, 0.0, true);
        for mode in [EnemyMode::Entering, EnemyMode::Dying(Countdown::new(0.2))] {
            for damage in [Damage::Survived, Damage::Killed] {
                assert_eq!(
                    enemy_after_hit(mode, damage, &reaction, FADE),
                    (mode, false),
                    "{mode:?}"
                );
            }
        }
    }

    #[test]
    fn a_turn_is_held_from_approaching_to_the_end_of_recovery() {
        for mode in [EnemyMode::Approaching]
            .into_iter()
            .chain(attacking_modes())
        {
            assert!(mode.holds_turn(), "{mode:?}");
        }
        for mode in [
            EnemyMode::Entering,
            EnemyMode::Waiting(0.5),
            EnemyMode::Retreating,
            EnemyMode::HitStun(Countdown::new(0.1)),
            EnemyMode::Dying(Countdown::new(0.1)),
        ] {
            assert!(!mode.holds_turn(), "{mode:?}");
        }
    }

    #[test]
    fn drift_reaches_the_spot_without_overshooting() {
        let spot = (300.0, 520.0);
        let mut position = (200.0, 560.0);
        for _ in 0..200 {
            let next = drift_step(position, spot, 45.0, 0.1);
            assert!(next.0 <= spot.0 && next.1 >= spot.1, "{next:?}");
            assert!((next.0 - position.0).abs() <= 4.5 + 1e-4);
            position = next;
        }
        assert_eq!(position, spot);
    }

    #[test]
    fn a_zero_drift_speed_holds_still() {
        assert_eq!(
            drift_step((10.0, 20.0), (90.0, 80.0), 0.0, 0.5),
            (10.0, 20.0)
        );
    }

    #[test]
    fn a_retreating_enemy_is_back_within_the_separation_distance() {
        let spot = (400.0, 600.0);
        // Target at x 220, waiting distance 180: the spot is at x 400.
        assert!(back_at_spot((430.0, 600.0), spot, 220.0, 180.0, 40.0));
        assert!(back_at_spot(spot, spot, 220.0, 180.0, 0.0), "exactly there");
        // Still close to the hero and away from the spot: not back yet.
        assert!(!back_at_spot((300.0, 600.0), spot, 220.0, 180.0, 40.0));
    }

    #[test]
    fn a_crowded_retreat_ends_on_the_waiting_ring() {
        // Pushed off its spot by the crowd, but 150 px from the hero (>= 180 - 40): back.
        let spot = (400.0, 600.0);
        assert!(back_at_spot((370.0, 680.0), spot, 220.0, 180.0, 40.0));
        assert!(!back_at_spot((330.0, 680.0), spot, 220.0, 180.0, 40.0));
    }

    const WALK: Speeds = Speeds {
        horizontal: 110.0,
        vertical: 65.0,
    };

    #[test]
    fn far_from_its_spot_an_enemy_walks_at_its_own_speeds() {
        let next = spot_step((1200.0, 500.0), (800.0, 600.0), WALK, 30.0, 40.0, 0.1);
        assert!((next.0 - 1189.0).abs() < 1e-3, "{next:?}");
        assert!((next.1 - 506.5).abs() < 1e-3, "{next:?}");
    }

    #[test]
    fn near_its_spot_an_enemy_drifts_and_drift_zero_stops_it() {
        let near = spot_step((820.0, 600.0), (800.0, 600.0), WALK, 30.0, 40.0, 0.1);
        assert!((near.0 - 817.0).abs() < 1e-3, "{near:?}");
        let held = spot_step((820.0, 600.0), (800.0, 600.0), WALK, 0.0, 40.0, 0.1);
        assert_eq!(held, (820.0, 600.0));
        // With drift 0 a far enemy still walks until it is near.
        let far = spot_step((1000.0, 600.0), (800.0, 600.0), WALK, 0.0, 40.0, 0.1);
        assert!(far.0 < 1000.0);
    }

    #[test]
    fn walking_to_a_spot_with_no_separation_arrives_exactly() {
        let mut position = (1100.0, 450.0);
        for _ in 0..100 {
            position = spot_step(position, (800.0, 600.0), WALK, 0.0, 0.0, 0.1);
        }
        assert_eq!(position, (800.0, 600.0));
    }
}
