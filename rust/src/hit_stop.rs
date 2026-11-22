//! The impact freeze ("hit-stop"), shared by every fighter and loaded as the `HitStop` autoload.
//!
//! A landed blow freezes play for a moment so it reads as an impact. Every gameplay object asks
//! this one node whether play is frozen, so they all stop and start together, and two freezes close
//! together never add up. It is not done with `Engine.time_scale`, which would also stop the flash
//! and the shake that must keep running through the freeze.

use godot::classes::{INode, Node};
use godot::prelude::*;

use crate::combat;

/// Counts down the impact freeze in real time; registered as the autoload `HitStop`.
///
/// The autoload name differs from this class name on purpose: in GDScript an autoload with the
/// same name as a class hides the class and breaks typed code that uses it.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct HitStopTimer {
    /// Seconds of freeze left; 0 when play runs.
    remaining: f32,
    base: Base<Node>,
}

#[godot_api]
impl INode for HitStopTimer {
    fn init(base: Base<Node>) -> Self {
        // Play starts unfrozen.
        Self {
            remaining: 0.0,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        // As an autoload this runs before the scene's nodes each tick, so a freeze ends for every
        // fighter on the same tick.
        self.remaining = (self.remaining - delta as f32).max(0.0);
    }
}

#[godot_api]
impl HitStopTimer {
    /// Freezes play for at least `seconds`: the longer of the time left and `seconds`, never the
    /// sum. `0` does nothing, which is how a zero `effects.hit_stop_time` turns the freeze off.
    #[func]
    pub fn freeze(&mut self, seconds: f32) {
        self.remaining = combat::extend_freeze(self.remaining, seconds);
    }

    /// Ends any freeze at once. A restart calls it, so a fresh run never starts frozen.
    #[func]
    pub fn clear(&mut self) {
        self.remaining = 0.0;
    }

    /// Whether play is frozen; fighters skip their gameplay step while it is.
    #[func]
    pub fn is_frozen(&self) -> bool {
        self.remaining > 0.0
    }
}
