//! The one place the game's current tuning lives, loaded as the `Tuning` autoload.
//!
//! Everything that needs a tuning value asks this node when it uses it, instead of keeping a
//! copy, so a reload reaches the whole game at once (M2a). A reload is all or nothing: the file
//! is read and checked in full before anything changes.

use godot::classes::{INode, Node};
use godot::prelude::*;

use crate::tuning::{self, TuningError, TuningValues};

/// Loads, checks and holds the current tuning values; registered as the autoload `Tuning`.
///
/// The autoload name differs from this class name on purpose: in GDScript an autoload with the
/// same name as a class hides the class and breaks typed code that uses it.
///
/// Signals report each reload so the debug overlay and other presentation can react without this
/// class knowing they exist.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct TuningStore {
    /// The values in use; `None` only until a load first succeeds.
    values: Option<TuningValues>,
    /// Why the last load or reload was rejected; `None` after a success.
    rejection: Option<TuningError>,
    base: Base<Node>,
}

#[godot_api]
impl INode for TuningStore {
    fn init(base: Base<Node>) -> Self {
        // Nothing is loaded until `ready()`, when the viewport (needed by the lane rule) exists.
        Self {
            values: None,
            rejection: None,
            base,
        }
    }

    fn ready(&mut self) {
        // Nobody can be listening yet, so the start-up result is kept for `rejected_field()` and
        // `rejected_reason()` instead of being signalled.
        let _ = self.load();
    }
}

#[godot_api]
impl TuningStore {
    /// Emitted after a successful [`reload`](Self::reload), once the new values are in place.
    #[signal]
    fn tuning_reloaded();

    /// Emitted after a rejected [`reload`](Self::reload); the previous values are still in use.
    /// `field` is the field path (`lane.top`), or the file path when the whole file is unusable.
    #[signal]
    fn tuning_rejected(field: GString, reason: GString);

    /// Rereads and checks the whole tuning file. If every value is valid, they all replace the
    /// current ones and `tuning_reloaded` is emitted; otherwise nothing changes, the error is
    /// written to the output panel and `tuning_rejected` is emitted.
    #[func]
    pub fn reload(&mut self) {
        match self.load() {
            Ok(()) => self.signals().tuning_reloaded().emit(),
            Err(error) => {
                let field = GString::from(error.field());
                let reason = GString::from(&error.reason());
                self.signals().tuning_rejected().emit(&field, &reason);
            }
        }
    }

    /// Whether any load has succeeded; `false` means the game has no tuning to run with yet.
    #[func]
    pub fn has_values(&self) -> bool {
        self.values.is_some()
    }

    /// The hero's speed along the lane in pixels per second, or `0.0` without values.
    #[func]
    pub fn horizontal_speed(&self) -> f32 {
        self.values.map_or(0.0, |v| v.horizontal_speed())
    }

    /// The hero's speed into and out of the lane in pixels per second, or `0.0` without values.
    #[func]
    pub fn vertical_speed(&self) -> f32 {
        self.values.map_or(0.0, |v| v.vertical_speed())
    }

    /// The highest y a character's feet may reach, in pixels, or `0.0` without values.
    #[func]
    pub fn lane_top(&self) -> f32 {
        self.values.map_or(0.0, |v| v.lane_top())
    }

    /// The lowest y a character's feet may reach, in pixels, or `0.0` without values.
    #[func]
    pub fn lane_bottom(&self) -> f32 {
        self.values.map_or(0.0, |v| v.lane_bottom())
    }

    /// Where the last load or reload failed (a field path or the file path), or `""` if it
    /// succeeded. Lets the overlay show a start-up error it was not around to hear.
    #[func]
    pub fn rejected_field(&self) -> GString {
        self.rejection
            .as_ref()
            .map_or_else(GString::new, |e| GString::from(e.field()))
    }

    /// Why the last load or reload failed, or `""` if it succeeded.
    #[func]
    pub fn rejected_reason(&self) -> GString {
        self.rejection
            .as_ref()
            .map_or_else(GString::new, |e| GString::from(&e.reason()))
    }
}

impl TuningStore {
    /// The values in use, copied, or `None` while no load has succeeded. Rust callers read this
    /// every time they need tuning instead of keeping their own copy.
    pub fn values(&self) -> Option<TuningValues> {
        self.values
    }

    /// Reads and checks the file, and swaps the values in only if all of them are valid.
    fn load(&mut self) -> Result<(), TuningError> {
        let screen_height = self
            .base()
            .get_viewport()
            .map_or(0.0, |viewport| viewport.get_visible_rect().size.y);
        let result =
            tuning::read_raw_tuning().and_then(|raw| tuning::validate(&raw, screen_height));
        match result {
            Ok(values) => {
                self.values = Some(values);
                self.rejection = None;
                Ok(())
            }
            Err(error) => {
                godot_error!("Tuning: {error}");
                self.rejection = Some(error.clone());
                Err(error)
            }
        }
    }
}
