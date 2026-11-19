//! The M0 toolchain-proof square: Rust owns its position, speed and wrap-around.

use godot::classes::{Control, INode2D, Node2D};
use godot::prelude::*;

use crate::tuning::{self, SquareTuning};

/// Name of the child node whose size defines the square's width (built in the editor).
const BODY_PATH: &str = "Body";

/// A square that drifts left to right at its tuned speed and wraps around the screen edge.
///
/// The speed comes from the tuning file (`drifting_square.speed`). If the tuning or the `Body`
/// child is unusable, the square reports one error in `ready()` and stays still.
#[derive(GodotClass)]
#[class(base = Node2D)]
pub struct DriftingSquare {
    /// Loaded tuning; `None` until `ready()` succeeds, and for good if it fails (stopped).
    tuning: Option<SquareTuning>,
    /// Width of the `Body` child, cached in `ready()` for the wrap-around.
    width: f32,
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for DriftingSquare {
    fn init(base: Base<Node2D>) -> Self {
        // "Not loaded yet" markers, not tuning values: `ready()` fills both or leaves the
        // square stopped.
        Self {
            tuning: None,
            width: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        match self.load() {
            Ok((tuning, width)) => {
                self.tuning = Some(tuning);
                self.width = width;
            }
            Err(message) => {
                godot_error!("DriftingSquare: {message}");
            }
        }
    }

    fn process(&mut self, delta: f64) {
        let Some(tuning) = self.tuning else {
            return;
        };
        let viewport_width = self.base().get_viewport_rect().size.x;
        let mut position = self.base().get_position();
        position.x = wrap_x(
            position.x + tuning.speed() * delta as f32,
            self.width,
            viewport_width,
        );
        self.base_mut().set_position(position);
    }
}

#[godot_api]
impl DriftingSquare {
    /// Current horizontal speed in pixels per second, or `0.0` while stopped. Read-only: only
    /// the tuning file sets the speed.
    #[func]
    pub fn speed(&self) -> f32 {
        self.tuning.map_or(0.0, |tuning| tuning.speed())
    }
}

impl DriftingSquare {
    /// Loads the tuning and measures `Body`, returning a ready-to-print message on failure.
    fn load(&self) -> Result<(SquareTuning, f32), String> {
        let tuning = tuning::load_square_tuning().map_err(|e| e.to_string())?;
        let body = self
            .base()
            .try_get_node_as::<Control>(BODY_PATH)
            .ok_or_else(|| format!("child `{BODY_PATH}` is missing or is not a Control"))?;
        let width = body.get_size().x;
        if width > 0.0 {
            Ok((tuning, width))
        } else {
            Err(format!(
                "child `{BODY_PATH}` must be wider than 0, but is {width}"
            ))
        }
    }
}

/// Wraps a horizontal position so the square fully leaves the right edge before re-entering
/// from the left. The result is always in `[-width, viewport_width)`, however far `x` jumped
/// in one frame.
fn wrap_x(x: f32, width: f32, viewport_width: f32) -> f32 {
    (x + width).rem_euclid(viewport_width + width) - width
}

#[cfg(test)]
mod tests {
    use super::wrap_x;

    const WIDTH: f32 = 64.0;
    const VIEWPORT: f32 = 1280.0;

    #[test]
    fn position_in_range_is_unchanged() {
        assert_eq!(wrap_x(100.0, WIDTH, VIEWPORT), 100.0);
        assert_eq!(wrap_x(-WIDTH, WIDTH, VIEWPORT), -WIDTH);
    }

    #[test]
    fn right_edge_maps_to_just_off_left_edge() {
        assert_eq!(wrap_x(VIEWPORT, WIDTH, VIEWPORT), -WIDTH);
    }

    #[test]
    fn just_past_right_edge_lands_just_after_left_start() {
        assert_eq!(wrap_x(VIEWPORT + 5.0, WIDTH, VIEWPORT), -WIDTH + 5.0);
    }

    #[test]
    fn several_screens_past_edge_stays_in_range() {
        let wrapped = wrap_x(VIEWPORT * 7.5, WIDTH, VIEWPORT);
        assert!((-WIDTH..VIEWPORT).contains(&wrapped), "{wrapped}");
    }
}
