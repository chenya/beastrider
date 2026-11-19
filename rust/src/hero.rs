//! The player-controlled hero: reads its own player's input and walks the lane.
//!
//! Rust owns the hero's position, speeds and limits; the scene only sets where it starts and what
//! it looks like. Nothing here assumes a single player: the input actions are built from
//! `player_index` (constitution, Principle VII).

use godot::classes::{INode2D, Input, InputMap, Node2D, Polygon2D};
use godot::prelude::*;

use crate::movement::{self, Limits, Speeds};
use crate::tuning::{self, MovementTuning};

/// Name of the child node whose polygon defines the hero's width (built in the editor).
const BODY_PATH: &str = "Body";

/// A character walked around the lane by one player.
///
/// The origin is at the hero's feet, so Godot's Y-sort draws it in front of anything whose base
/// is higher on the screen. If its input actions, tuning or `Body` child are unusable, it reports
/// one error in `ready()` and stays still.
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct Hero {
    /// Which player controls this hero; picks the input actions `p{index}_left` and so on.
    #[export]
    player_index: i32,
    /// Everything needed to move; `None` until `ready()` succeeds, and for good if it fails.
    active: Option<Active>,
    base: Base<Node2D>,
}

/// The state a hero needs to move, set up once in `ready()`.
struct Active {
    actions: PlayerActions,
    tuning: MovementTuning,
    limits: Limits,
}

/// One player's four direction actions, built once so no action name is typed elsewhere.
struct PlayerActions {
    left: StringName,
    right: StringName,
    up: StringName,
    down: StringName,
}

impl PlayerActions {
    /// Builds the action names for `player_index` and checks each exists in the Input Map.
    fn try_new(player_index: i32) -> Result<Self, String> {
        let name = |direction: &str| -> Result<StringName, String> {
            let action = format!("p{player_index}_{direction}");
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
        })
    }
}

#[godot_api]
impl INode2D for Hero {
    fn init(base: Base<Node2D>) -> Self {
        // Player 1 is the default controller for a hero placed in the editor; the scene can
        // change it. `active` is a "not set up yet" marker, filled by `ready()`.
        Self {
            player_index: 1,
            active: None,
            base,
        }
    }

    fn ready(&mut self) {
        match self.set_up() {
            Ok(active) => self.active = Some(active),
            Err(message) => {
                godot_error!("Hero: {message}");
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let Some(active) = &self.active else {
            return;
        };
        let actions = &active.actions;
        let input = Input::singleton().get_vector(
            &actions.left,
            &actions.right,
            &actions.up,
            &actions.down,
        );
        let speeds = Speeds {
            horizontal: active.tuning.horizontal_speed(),
            vertical: active.tuning.vertical_speed(),
        };
        let limits = active.limits;
        let position = self.base().get_position();
        let (x, y) = movement::step(
            (position.x, position.y),
            (input.x, input.y),
            speeds,
            delta as f32,
            limits,
        );
        self.base_mut().set_position(Vector2::new(x, y));
    }
}

#[godot_api]
impl Hero {
    /// The highest y (furthest into the lane) the hero's feet may reach, in pixels, or `0.0`
    /// while the hero is stopped. Read-only: only the tuning file sets it.
    #[func]
    pub fn lane_top(&self) -> f32 {
        self.active.as_ref().map_or(0.0, |a| a.tuning.lane_top())
    }

    /// The lowest y (nearest the player) the hero's feet may reach, in pixels, or `0.0` while
    /// the hero is stopped. Read-only: only the tuning file sets it.
    #[func]
    pub fn lane_bottom(&self) -> f32 {
        self.active.as_ref().map_or(0.0, |a| a.tuning.lane_bottom())
    }
}

impl Hero {
    /// Checks the player index, input actions, tuning, lane and `Body`, in that order, and
    /// returns a ready-to-print message for the first problem.
    fn set_up(&self) -> Result<Active, String> {
        if self.player_index < 1 {
            return Err(format!(
                "player_index must be 1 or more, but is {}",
                self.player_index
            ));
        }
        let actions = PlayerActions::try_new(self.player_index)?;
        let tuning = tuning::load_movement_tuning().map_err(|e| e.to_string())?;
        let screen = self.base().get_viewport_rect().size;
        tuning::check_lane_fits(&tuning, screen.y).map_err(|e| e.to_string())?;
        let half_width = self.body_half_width()?;
        let limits = Limits {
            min_x: half_width,
            max_x: screen.x - half_width,
            min_y: tuning.lane_top(),
            max_y: tuning.lane_bottom(),
        };
        Ok(Active {
            actions,
            tuning,
            limits,
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
