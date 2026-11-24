//! The player-controlled hero: walks the lane, swings at enemies, and can be hurt and fall.
//!
//! Rust owns the hero's position, facing, swing, health, stun, invulnerability and knockback; the
//! scene only sets where it starts and what it looks like. Tuning values are read from the `Tuning`
//! autoload when used, never kept beyond one swing, so a reload reaches the hero on its next tick
//! or next swing. Nothing here assumes a single player: every input action is built from
//! `player_index`, and enemies reach heroes through the players' group (constitution, Principle
//! VII).

use godot::classes::{INode2D, Input, InputMap, Node2D, Polygon2D};
use godot::prelude::*;

use crate::combat::{self, Facing};
use crate::fighter::{
    AttackAdvance, AttackPhase, AttackSpec, AttackStep, AttackTimer, Attacker, Countdown, Damage,
    Health, Hit, Knockback, Target, Team, resolve_attack,
};
use crate::fighters::FighterRef;
use crate::hit_stop::HitStopTimer;
use crate::movement::{self, Limits, Speeds};
use crate::run::RunTracker;
use crate::training_dummy::{TrainingDummy, to_rect2};
use crate::tuning::TuningValues;
use crate::tuning_store::TuningStore;

/// Name of the child node whose polygon defines the hero's width (built in the editor).
const BODY_PATH: &str = "Body";

/// Where the `Tuning` autoload lives in the scene tree.
const TUNING_PATH: &str = "/root/Tuning";

/// Where the `HitStop` autoload lives in the scene tree.
const HIT_STOP_PATH: &str = "/root/HitStop";

/// Where the `Run` autoload lives in the scene tree.
const RUN_PATH: &str = "/root/Run";

/// A character walked around the lane by one player, who can swing at enemies in front of it.
///
/// The origin is at the hero's feet, so Godot's Y-sort draws it in front of anything whose base
/// is higher on the screen. If its input actions, `Body` child or the autoloads are unusable, it
/// reports one error in `ready()` and never moves. While `Tuning` has no valid values it stands
/// still, and it starts moving on the first tick after a successful reload. While `HitStop` is
/// frozen it does nothing at all.
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct Hero {
    /// Which player controls this hero; picks the input actions `p{index}_left` and so on.
    #[export]
    player_index: i32,
    /// Everything needed to act; `None` until `ready()` succeeds, and for good if it fails.
    setup: Option<Setup>,
    /// The way the hero faces: the way it last moved horizontally.
    facing: Facing,
    /// What the hero is doing.
    mode: HeroMode,
    /// Health points; `None` until the first tick with tuning, which starts it full.
    health: Option<Health>,
    /// Runs after a blow lands; blows cannot land while it does. Beside the mode, because the
    /// hero walks and swings while blinking.
    invulnerable: Countdown,
    /// The slide from the last blow taken, if it is still under way; runs in every mode.
    knockback: Option<Knockback>,
    /// Targets hit since the game started, one per target per swing.
    hits_landed: i64,
    base: Base<Node2D>,
}

/// What the hero is doing. Every use is an exhaustive `match`, so a new mode cannot be forgotten.
enum HeroMode {
    /// Walking (or standing); a press of attack starts a swing.
    Free,
    /// Swinging; the hero neither moves nor turns.
    Attacking(Swing),
    /// Stunned by a blow: no moving or attacking until the countdown runs out.
    HitStun(Countdown),
    /// Fallen. Stays so until the restart reloads the scene.
    Dead,
}

/// One swing under way.
struct Swing {
    /// The attack's part, time and values (copied at the start, so a reload applies next swing).
    timer: AttackTimer,
    /// Whether attack was pressed during recovery, to swing again the moment this one ends.
    queued: bool,
    /// Targets this swing has hit, so each is hit at most once per swing.
    hit_this_swing: Vec<InstanceId>,
}

impl Swing {
    /// A new swing with `spec`.
    fn new(spec: AttackSpec) -> Self {
        Self {
            timer: AttackTimer::new(spec),
            queued: false,
            hit_this_swing: Vec::new(),
        }
    }
}

/// What a hero checks once in `ready()`: the parts that a tuning reload cannot fix.
struct Setup {
    actions: PlayerActions,
    /// Half the `Body` width: keeps the hero on screen and is where its swing starts.
    half_width: f32,
    /// The single source of every tuning value, read when needed.
    tuning: Gd<TuningStore>,
    /// The shared impact freeze.
    hit_stop: Gd<HitStopTimer>,
    /// The run, told when the hero falls.
    run: Gd<RunTracker>,
}

/// One player's input actions, built once so no action name is typed elsewhere.
struct PlayerActions {
    left: StringName,
    right: StringName,
    up: StringName,
    down: StringName,
    attack: StringName,
}

impl PlayerActions {
    /// Builds the action names for `player_index` and checks each exists in the Input Map.
    fn try_new(player_index: i32) -> Result<Self, String> {
        let name = |action_kind: &str| -> Result<StringName, String> {
            let action = format!("p{player_index}_{action_kind}");
            if InputMap::singleton().has_action(&action) {
                Ok(StringName::from(&action))
            } else {
                Err(format!(
                    "input action `{action}` for player {player_index} is missing from the \
                     Input Map"
                ))
            }
        };
        Ok(Self {
            left: name("left")?,
            right: name("right")?,
            up: name("up")?,
            down: name("down")?,
            attack: name("attack")?,
        })
    }
}

#[godot_api]
impl INode2D for Hero {
    fn init(base: Base<Node2D>) -> Self {
        // Player 1 is the default controller for a hero placed in the editor; the scene can
        // change it. A hero starts free, facing right, with no hits; health is filled on the
        // first tick with tuning, and `setup` by `ready()`.
        Self {
            player_index: 1,
            setup: None,
            facing: Facing::Right,
            mode: HeroMode::Free,
            health: None,
            invulnerable: Countdown::default(),
            knockback: None,
            hits_landed: 0,
            base,
        }
    }

    fn ready(&mut self) {
        match self.set_up() {
            Ok(setup) => {
                self.setup = Some(setup);
                self.base_mut().add_to_group(Team::Players.group());
            }
            Err(message) => {
                godot_error!("Hero: {message}");
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let Some(setup) = &self.setup else {
            return;
        };
        if setup.hit_stop.bind().is_frozen() {
            return;
        }
        let Some(values) = setup.tuning.bind().values() else {
            return;
        };
        let playing = setup.run.bind().is_playing();
        let attack_pressed =
            playing && Input::singleton().is_action_just_pressed(&setup.actions.attack);
        let delta = delta as f32;
        let max_health = values.hero().max_health();
        match &mut self.health {
            Some(health) => health.set_max(max_health),
            None => self.health = Some(Health::new(max_health)),
        }
        self.invulnerable.tick(delta);
        self.slide(&values, delta);
        let mode = std::mem::replace(&mut self.mode, HeroMode::Free);
        self.mode = match mode {
            HeroMode::Free if attack_pressed => HeroMode::Attacking(self.new_swing(&values)),
            HeroMode::Free => {
                if playing {
                    self.walk(&values, delta);
                }
                HeroMode::Free
            }
            HeroMode::Attacking(swing) => self.swing(swing, attack_pressed, &values, delta),
            HeroMode::HitStun(mut stun) => {
                if stun.tick(delta) {
                    HeroMode::Free
                } else {
                    HeroMode::HitStun(stun)
                }
            }
            HeroMode::Dead => HeroMode::Dead,
        };
    }
}

#[godot_api]
impl Hero {
    /// Emitted when a blow lands on the hero. `direction` is the push (−1 left, 1 right);
    /// `strength` is the knockback distance in pixels. Handlers must not call back into the hero.
    #[signal]
    fn was_hit(direction: f32, strength: f32);

    /// Emitted once, when the hero's health reaches zero and it falls. Handlers must not call back
    /// into the hero.
    #[signal]
    fn died();

    /// Targets hit since the game started, one per target per swing. Shown by the debug overlay.
    #[func]
    pub fn hits_landed(&self) -> i64 {
        self.hits_landed
    }

    /// Health points left, for the health bar; `0.0` before the first tick with tuning.
    #[func]
    pub fn health(&self) -> f32 {
        self.health.map_or(0.0, |h| h.current())
    }

    /// Health points when full, for the health bar; `0.0` before the first tick with tuning.
    #[func]
    pub fn max_health(&self) -> f32 {
        self.health.map_or(0.0, |h| h.max())
    }

    /// Whether blows cannot land on the hero right now (just after being hit); drives the blink.
    #[func]
    pub fn is_invulnerable(&self) -> bool {
        self.invulnerable.is_running()
    }

    /// Whether the hero has fallen.
    #[func]
    pub fn is_dead(&self) -> bool {
        match self.mode {
            HeroMode::Dead => true,
            HeroMode::Free | HeroMode::Attacking(_) | HeroMode::HitStun(_) => false,
        }
    }

    /// Back to full health (the F4 cheat). Does nothing once the hero has fallen.
    #[func]
    pub fn refill_health(&mut self) {
        if self.is_dead() {
            return;
        }
        if let Some(health) = &mut self.health {
            health.refill();
        }
    }

    /// The hero's hurtbox in screen pixels, from `hero.hurtbox_width` and `hurtbox_height`; an empty
    /// rectangle while there is no tuning. For the debug hitbox drawing.
    #[func]
    pub fn hurtbox(&self) -> Rect2 {
        self.hurtbox_rect()
            .map_or_else(Rect2::default, |hurt| to_rect2(&hurt))
    }

    /// The area the swing can land in, in screen pixels, while it is in its active part; an empty
    /// rectangle otherwise. For the debug hitbox drawing only.
    #[func]
    pub fn attack_box(&self) -> Rect2 {
        match &self.mode {
            HeroMode::Attacking(swing) => match swing.timer.phase() {
                AttackPhase::Active => self
                    .attacker(&swing.timer.spec(), 0.0)
                    .map_or_else(Rect2::default, |a| to_rect2(&a.attack_box())),
                AttackPhase::WindUp | AttackPhase::Recovery => Rect2::default(),
            },
            HeroMode::Free | HeroMode::HitStun(_) | HeroMode::Dead => Rect2::default(),
        }
    }

    /// Where the debug overlay should put a new training dummy: in front of the hero at its depth,
    /// just inside reach, stepping half a dummy's width further along the facing for each spot
    /// already taken, and kept on screen. The hero's own position if there is no tuning.
    #[func]
    pub fn dummy_spawn_point(&self) -> Vector2 {
        let position = self.base().get_position();
        let Some(setup) = &self.setup else {
            return position;
        };
        let Some(values) = setup.tuning.bind().values() else {
            return position;
        };
        let others_x: Vec<f32> = self
            .dummies()
            .iter()
            .map(|dummy| dummy.get_position().x)
            .collect();
        let (x, y) = combat::spawn_point(
            (position.x, position.y),
            self.facing,
            setup.half_width,
            values.hero().attack().reach(),
            values.dummy().hurtbox_width(),
            &others_x,
            self.base().get_viewport_rect().size.x,
        );
        Vector2::new(x, y)
    }
}

impl Hero {
    /// How the hit rule sees the hero, or `None` while there is no tuning.
    pub fn target(&self) -> Option<Target<InstanceId>> {
        Some(Target {
            id: self.base().instance_id(),
            team: Team::Players,
            hurtbox: self.hurtbox_rect()?,
            feet_y: self.base().get_position().y,
            alive: !self.is_dead(),
            invulnerable: self.invulnerable.is_running(),
        })
    }

    /// Takes a blow from an enemy, unless fallen or invulnerable: health drops, the knockback
    /// starts and `was_hit` is emitted; a surviving hero is stunned (any swing is cancelled) and
    /// becomes invulnerable for a while; a killed hero falls and the run ends.
    pub fn take_hit(&mut self, hit: &Hit<InstanceId>) {
        if self.is_dead() || self.invulnerable.is_running() {
            return;
        }
        let Some(setup) = &self.setup else {
            return;
        };
        let Some(values) = setup.tuning.bind().values() else {
            return;
        };
        let mut run = setup.run.clone();
        let Some(health) = &mut self.health else {
            return;
        };
        let damage = health.damage(hit.damage);
        let start_x = self.base().get_position().x;
        self.knockback = Knockback::start(start_x, hit.direction, hit.knockback);
        self.signals()
            .was_hit()
            .emit(hit.direction, hit.knockback.distance);
        match damage {
            Damage::Survived => {
                self.mode = HeroMode::HitStun(Countdown::new(values.hero().hit_stun_time()));
                self.invulnerable = Countdown::new(values.invulnerability_time());
            }
            Damage::Killed => {
                self.mode = HeroMode::Dead;
                self.signals().died().emit();
                run.bind_mut().end_run();
            }
            Damage::AlreadyDead => {}
        }
    }

    /// A new swing with the current attack values.
    fn new_swing(&self, values: &TuningValues) -> Swing {
        Swing::new(AttackSpec::from_tuning(&values.hero().attack()))
    }

    /// Walks one tick, as in M1, and turns towards any horizontal input.
    fn walk(&mut self, values: &TuningValues, delta: f32) {
        let Some(setup) = &self.setup else {
            return;
        };
        let actions = &setup.actions;
        let input = Input::singleton().get_vector(
            &actions.left,
            &actions.right,
            &actions.up,
            &actions.down,
        );
        let hero = values.hero();
        let speeds = Speeds {
            horizontal: hero.horizontal_speed(),
            vertical: hero.vertical_speed(),
        };
        let screen_width = self.base().get_viewport_rect().size.x;
        let limits = Limits {
            min_x: setup.half_width,
            max_x: screen_width - setup.half_width,
            min_y: values.lane_top(),
            max_y: values.lane_bottom(),
        };
        let position = self.base().get_position();
        let (x, y) = movement::step(
            (position.x, position.y),
            (input.x, input.y),
            speeds,
            delta,
            limits,
        );
        self.facing = self.facing.after_input(input.x);
        self.base_mut().set_position(Vector2::new(x, y));
    }

    /// Moves any knockback slide on by one tick, keeping the hero inside the lane and the screen.
    fn slide(&mut self, values: &TuningValues, delta: f32) {
        let Some(half_width) = self.setup.as_ref().map(|s| s.half_width) else {
            return;
        };
        let Some(mut knockback) = self.knockback else {
            return;
        };
        let mut position = self.base().get_position();
        position.x = knockback.advance(delta);
        let screen_width = self.base().get_viewport_rect().size.x;
        let (x, y) = combat::clamp_to_area(
            (position.x, position.y),
            half_width,
            screen_width,
            values.lane_top(),
            values.lane_bottom(),
        );
        let stopped_at_limit = x != position.x;
        self.knockback = (!knockback.is_done() && !stopped_at_limit).then_some(knockback);
        self.base_mut().set_position(Vector2::new(x, y));
    }

    /// Moves a swing on by one tick: remembers a press made in recovery, lands the blow on any
    /// new target while the swing is active, and says what the hero does next.
    fn swing(
        &mut self,
        mut swing: Swing,
        attack_pressed: bool,
        values: &TuningValues,
        delta: f32,
    ) -> HeroMode {
        if attack_pressed {
            match swing.timer.phase() {
                AttackPhase::Recovery => swing.queued = true,
                AttackPhase::WindUp | AttackPhase::Active => {}
            }
        }
        let AttackAdvance { step, was_active } = swing.timer.advance(delta);
        if was_active {
            self.land_blow(&swing.timer.spec(), values, &mut swing.hit_this_swing);
        }
        match step {
            AttackStep::Continue => HeroMode::Attacking(swing),
            AttackStep::Finished if swing.queued => HeroMode::Attacking(self.new_swing(values)),
            AttackStep::Finished => HeroMode::Free,
        }
    }

    /// Hits every enemy the active swing reaches that it has not hit yet: each takes the hit,
    /// the counter goes up, and play freezes once for the swing, however many targets it hits.
    fn land_blow(
        &mut self,
        spec: &AttackSpec,
        values: &TuningValues,
        hit_this_swing: &mut Vec<InstanceId>,
    ) {
        let Some(attacker) = self.attacker(spec, values.depth_tolerance()) else {
            return;
        };
        let mut enemies = FighterRef::in_team(&self.base().get_tree(), Team::Enemies);
        let targets: Vec<Target<InstanceId>> =
            enemies.iter().filter_map(FighterRef::target).collect();
        let hits = resolve_attack(&attacker, &targets, hit_this_swing);
        if hits.is_empty() {
            return;
        }
        for hit in &hits {
            if let Some(enemy) = enemies.iter_mut().find(|e| e.id() == hit.target) {
                enemy.take_hit(hit);
            }
        }
        hit_this_swing.extend(hits.iter().map(|hit| hit.target));
        self.hits_landed += hits.len() as i64;
        let freeze = values.effects().hit_stop_time();
        if freeze > 0.0
            && let Some(setup) = &self.setup
        {
            setup.hit_stop.clone().bind_mut().freeze(freeze);
        }
    }

    /// The hero as an attacker with `spec`, from its current position and facing.
    fn attacker(&self, spec: &AttackSpec, depth_tolerance: f32) -> Option<Attacker> {
        let position = self.base().get_position();
        Some(Attacker {
            team: Team::Players,
            origin: (position.x, position.y),
            facing: self.facing,
            half_width: self.setup.as_ref()?.half_width,
            spec: *spec,
            depth_tolerance,
        })
    }

    /// The hurtbox in screen pixels, or `None` while there is no tuning.
    fn hurtbox_rect(&self) -> Option<combat::Rect> {
        let values = self.setup.as_ref()?.tuning.bind().values()?;
        let position = self.base().get_position();
        Some(combat::hurtbox(
            (position.x, position.y),
            values.hero().hurtbox_width(),
            values.hero().hurtbox_height(),
        ))
    }

    /// Every training dummy among the enemies.
    fn dummies(&self) -> Vec<Gd<TrainingDummy>> {
        FighterRef::in_team(&self.base().get_tree(), Team::Enemies)
            .into_iter()
            .filter_map(|fighter| match fighter {
                FighterRef::Dummy(dummy) => Some(dummy),
                FighterRef::Hero(_) | FighterRef::Enemy(_) => None,
            })
            .collect()
    }

    /// Checks the player index, input actions, `Body` and the three autoloads, in that order, and
    /// returns a ready-to-print message for the first problem. Tuning values are not checked
    /// here: they are read when needed, so a later reload can still fix them.
    fn set_up(&self) -> Result<Setup, String> {
        if self.player_index < 1 {
            return Err(format!(
                "player_index must be 1 or more, but is {}",
                self.player_index
            ));
        }
        let actions = PlayerActions::try_new(self.player_index)?;
        let half_width = self.body_half_width()?;
        let tuning = self
            .base()
            .try_get_node_as::<TuningStore>(TUNING_PATH)
            .ok_or_else(|| "the Tuning autoload is missing or is not a TuningStore".to_string())?;
        let hit_stop = self
            .base()
            .try_get_node_as::<HitStopTimer>(HIT_STOP_PATH)
            .ok_or_else(|| {
                "the HitStop autoload is missing or is not a HitStopTimer".to_string()
            })?;
        let run = self
            .base()
            .try_get_node_as::<RunTracker>(RUN_PATH)
            .ok_or_else(|| "the Run autoload is missing or is not a RunTracker".to_string())?;
        Ok(Setup {
            actions,
            half_width,
            tuning,
            hit_stop,
            run,
        })
    }

    /// Half the horizontal extent of the `Body` polygon, measured from the hero's origin.
    fn body_half_width(&self) -> Result<f32, String> {
        let body = self
            .base()
            .try_get_node_as::<Polygon2D>(BODY_PATH)
            .ok_or_else(|| format!("child `{BODY_PATH}` is missing or is not a Polygon2D"))?;
        let offset = body.get_position().x;
        let polygon = body.get_polygon();
        let (min, max) = polygon
            .as_slice()
            .iter()
            .map(|point| point.x + offset)
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), x| {
                (lo.min(x), hi.max(x))
            });
        let width = max - min;
        if width > 0.0 {
            Ok(width / 2.0)
        } else {
            Err(format!(
                "child `{BODY_PATH}` must have a polygon wider than 0, but its width is {width}"
            ))
        }
    }
}
