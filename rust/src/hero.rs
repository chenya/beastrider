//! The player-controlled hero: reads its own player's input and walks the lane.
//!
//! Rust owns the hero's position and movement; the scene only sets where it starts and what it
//! looks like. Speeds and lane limits are read from the `Tuning` autoload every physics tick, never
//! copied, so a tuning reload reaches the hero on the next tick. Nothing here assumes a single
//! player: the input actions are built from `player_index` (constitution, Principle VII).

use godot::classes::{INode2D, Input, InputMap, Node2D, Polygon2D};
use godot::prelude::*;

use crate::movement::{self, Limits, Speeds};
use crate::tuning_store::TuningStore;

/// Name of the child node whose polygon defines the hero's width (built in the editor).
const BODY_PATH: &str = "Body";

/// Where the `Tuning` autoload lives in the scene tree.
const TUNING_PATH: &str = "/root/Tuning";

/// A character walked around the lane by one player.
///
/// The origin is at the hero's feet, so Godot's Y-sort draws it in front of anything whose base
/// is higher on the screen. If its input actions, `Body` child or the `Tuning` autoload are
/// unusable, it reports one error in `ready()` and never moves. While `Tuning` has no valid values
/// it stands still, and it starts moving on the first tick after a successful reload.
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct Hero {
    /// Which player controls this hero; picks the input actions `p{index}_left` and so on.
    #[export]
    player_index: i32,
    /// Everything needed to move; `None` until `ready()` succeeds, and for good if it fails.
    setup: Option<Setup>,
    base: Base<Node2D>,
}

/// What a hero checks once in `ready()`: the parts that a tuning reload cannot fix.
struct Setup {
    actions: PlayerActions,
    /// Half the `Body` width, so the whole hero stays on screen.
    half_width: f32,
    /// The single source of speeds and lane limits, read every tick.
    tuning: Gd<TuningStore>,
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
        // change it. `setup` is a "not set up yet" marker, filled by `ready()`.
        Self {
            player_index: 1,
            setup: None,
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
        let Some(values) = setup.tuning.bind().values() else {
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
            delta as f32,
            limits,
        );
        self.base_mut().set_position(Vector2::new(x, y));
    }
}

impl Hero {
    /// Checks the player index, input actions, `Body` and the `Tuning` autoload, in that order,
    /// and returns a ready-to-print message for the first problem. Tuning values are not checked
    /// here: they are read every tick, so a later reload can still fix them.
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
        Ok(Setup {
            actions,
            half_width,
            tuning,
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
