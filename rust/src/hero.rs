//! The player-controlled hero: walks the lane and swings at whatever it can hit.
//!
//! Rust owns the hero's position, facing, swing and hits; the scene only sets where it starts and
//! what it looks like. Tuning values are read from the `Tuning` autoload, never kept beyond one
//! swing, so a reload reaches the hero on its next tick or next swing. Nothing here assumes a
//! single player: every input action is built from `player_index` (constitution, Principle VII).

use godot::classes::{INode2D, Input, InputMap, Node2D, Polygon2D};
use godot::prelude::*;

use crate::combat::{
    self, Facing, HITTABLE_GROUP, SwingPhase, SwingStep, SwingTimings, attack_box, new_hits,
};
use crate::hit_stop::HitStopTimer;
use crate::movement::{self, Limits, Speeds};
use crate::training_dummy::{TrainingDummy, to_rect2};
use crate::tuning::{AttackTuning, TuningValues};
use crate::tuning_store::TuningStore;

/// Name of the child node whose polygon defines the hero's width (built in the editor).
const BODY_PATH: &str = "Body";

/// Where the `Tuning` autoload lives in the scene tree.
const TUNING_PATH: &str = "/root/Tuning";

/// Where the `HitStop` autoload lives in the scene tree.
const HIT_STOP_PATH: &str = "/root/HitStop";

/// A character walked around the lane by one player, who can swing at targets in front of it.
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
    /// What the hero is doing: free to move, or swinging.
    mode: HeroMode,
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
}

/// One swing under way.
struct Swing {
    /// Which part of the swing it is in.
    phase: SwingPhase,
    /// Seconds spent in that part.
    elapsed: f32,
    /// The attack values when the swing started, so a reload applies from the next swing.
    attack: AttackTuning,
    /// Targets this swing has hit, so each is hit at most once per swing.
    hit_this_swing: Vec<InstanceId>,
}

impl HeroMode {
    /// A new swing with the current attack values.
    fn new_swing(attack: AttackTuning) -> Self {
        Self::Attacking(Swing {
            phase: SwingPhase::WindUp,
            elapsed: 0.0,
            attack,
            hit_this_swing: Vec::new(),
        })
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
        // change it. A hero starts free, facing right, with no hits. `setup` is filled by
        // `ready()`.
        Self {
            player_index: 1,
            setup: None,
            facing: Facing::Right,
            mode: HeroMode::Free,
            hits_landed: 0,
            base,
        }
    }

    fn ready(&mut self) {
        match self.set_up() {
            Ok(setup) => self.setup = Some(setup),
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
        let attack_pressed = Input::singleton().is_action_just_pressed(&setup.actions.attack);
        let mode = std::mem::replace(&mut self.mode, HeroMode::Free);
        self.mode = match mode {
            HeroMode::Free if attack_pressed => HeroMode::new_swing(values.attack()),
            HeroMode::Free => {
                self.walk(&values, delta as f32);
                HeroMode::Free
            }
            HeroMode::Attacking(swing) => self.swing(swing, attack_pressed, &values, delta as f32),
        };
    }
}

#[godot_api]
impl Hero {
    /// Targets hit since the game started, one per target per swing. Shown by the debug overlay.
    #[func]
    pub fn hits_landed(&self) -> i64 {
        self.hits_landed
    }

    /// The area the swing can land in, in screen pixels, while it is in its active part; an empty
    /// rectangle otherwise. For the debug hitbox drawing only.
    #[func]
    pub fn attack_box(&self) -> Rect2 {
        let Some(setup) = &self.setup else {
            return Rect2::default();
        };
        match &self.mode {
            HeroMode::Attacking(swing) => match swing.phase {
                SwingPhase::Active => to_rect2(&self.reach_box(setup.half_width, &swing.attack)),
                SwingPhase::WindUp | SwingPhase::Recovery { .. } => Rect2::default(),
            },
            HeroMode::Free => Rect2::default(),
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
            values.attack().reach(),
            values.dummy().hurtbox_width(),
            &others_x,
            self.base().get_viewport_rect().size.x,
        );
        Vector2::new(x, y)
    }
}

impl Hero {
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
        let speeds = Speeds {
            horizontal: values.horizontal_speed(),
            vertical: values.vertical_speed(),
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

    /// Moves a swing on by one tick: remembers a press made in recovery, lands the blow on any
    /// new target while the swing is active, and says what the hero does next.
    fn swing(
        &mut self,
        swing: Swing,
        attack_pressed: bool,
        values: &TuningValues,
        delta: f32,
    ) -> HeroMode {
        let Swing {
            phase,
            elapsed,
            attack,
            mut hit_this_swing,
        } = swing;
        let phase = match phase {
            SwingPhase::Recovery { .. } if attack_pressed => SwingPhase::Recovery { queued: true },
            SwingPhase::WindUp | SwingPhase::Active | SwingPhase::Recovery { .. } => phase,
        };
        let timings = SwingTimings {
            wind_up: attack.wind_up_time(),
            active: attack.active_time(),
            recovery: attack.recovery_time(),
        };
        let advance = combat::advance_swing(phase, elapsed, delta, timings);
        if advance.was_active {
            self.land_blow(&attack, values, &mut hit_this_swing);
        }
        match advance.step {
            SwingStep::Continue => HeroMode::Attacking(Swing {
                phase: advance.phase,
                elapsed: advance.elapsed,
                attack,
                hit_this_swing,
            }),
            SwingStep::Finished => HeroMode::Free,
            SwingStep::Restart => HeroMode::new_swing(values.attack()),
        }
    }

    /// Hits every target the active swing reaches that it has not hit yet: each takes the hit,
    /// the counter goes up, and play freezes once for the swing, however many targets it hits.
    fn land_blow(
        &mut self,
        attack: &AttackTuning,
        values: &TuningValues,
        hit_this_swing: &mut Vec<InstanceId>,
    ) {
        let Some(setup) = &self.setup else {
            return;
        };
        let feet_y = self.base().get_position().y;
        let reach = self.reach_box(setup.half_width, attack);
        let dummies = self.dummies();
        let targets: Vec<(InstanceId, combat::Rect, f32)> = dummies
            .iter()
            .filter_map(|dummy| {
                let hurt = dummy.bind().hurtbox_rect()?;
                Some((dummy.instance_id(), hurt, dummy.get_position().y))
            })
            .collect();
        let hits = new_hits(
            &reach,
            feet_y,
            attack.depth_tolerance(),
            &targets,
            hit_this_swing,
        );
        if hits.is_empty() {
            return;
        }
        let direction = self.facing.sign();
        let mut hit_stop = setup.hit_stop.clone();
        for mut dummy in dummies
            .into_iter()
            .filter(|dummy| hits.contains(&dummy.instance_id()))
        {
            dummy.bind_mut().take_hit(direction);
        }
        hit_this_swing.extend(hits.iter().copied());
        self.hits_landed += hits.len() as i64;
        let freeze = values.impact().hit_stop_time();
        if freeze > 0.0 {
            hit_stop.bind_mut().freeze(freeze);
        }
    }

    /// The swing's reach as a box, from the hero's current position and facing.
    fn reach_box(&self, half_width: f32, attack: &AttackTuning) -> combat::Rect {
        let position = self.base().get_position();
        attack_box(
            (position.x, position.y),
            self.facing,
            half_width,
            attack.reach(),
            attack.height(),
        )
    }

    /// Every training dummy in the hittable group.
    fn dummies(&self) -> Vec<Gd<TrainingDummy>> {
        self.base()
            .get_tree()
            .get_nodes_in_group(HITTABLE_GROUP)
            .iter_shared()
            .filter_map(|node| node.try_cast::<TrainingDummy>().ok())
            .collect()
    }

    /// Checks the player index, input actions, `Body` and the two autoloads, in that order, and
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
        Ok(Setup {
            actions,
            half_width,
            tuning,
            hit_stop,
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
