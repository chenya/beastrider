//! The run: from the first grunt to the hero's fall, and the restart that begins the next one.
//!
//! It is the `Run` autoload, so it outlives the scene: the restart reloads the scene, which
//! rebuilds the hero, the spawner, the HUD and the Game Over screen, and leaves nothing of the
//! old run behind. The tuning autoload is not reloaded, so values changed with F5 survive.

use godot::classes::{INode, Input, InputMap, Node};
use godot::prelude::*;

use crate::hit_stop::HitStopTimer;

/// The Input Map action that restarts the run, in every build.
const RESTART_ACTION: &str = "game_restart";

/// Where the `HitStop` autoload lives in the scene tree.
const HIT_STOP_PATH: &str = "/root/HitStop";

/// Whether the run is still being played.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    /// The hero is alive; grunts hunt, the spawner spawns.
    Playing,
    /// The hero has fallen; only the restart does anything.
    GameOver,
}

/// The run's state and kill count, with no Godot types, so its rules are unit-tested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunRecord {
    state: RunState,
    kills: i64,
}

impl RunRecord {
    /// A fresh run: playing, no kills.
    pub fn new() -> Self {
        Self {
            state: RunState::Playing,
            kills: 0,
        }
    }

    /// The run's state.
    pub fn state(&self) -> RunState {
        self.state
    }

    /// Grunts killed this run.
    pub fn kills(&self) -> i64 {
        self.kills
    }

    /// Counts one kill. A grunt adds its kill when it starts dying, so only while playing.
    pub fn add_kill(&mut self) {
        match self.state {
            RunState::Playing => self.kills += 1,
            RunState::GameOver => {}
        }
    }

    /// Ends the run; true only the first time, so `game_over` is announced once.
    pub fn end_run(&mut self) -> bool {
        match self.state {
            RunState::Playing => {
                self.state = RunState::GameOver;
                true
            }
            RunState::GameOver => false,
        }
    }

    /// Back to a fresh run.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for RunRecord {
    fn default() -> Self {
        Self::new()
    }
}

/// Keeps the run's state and kills, and restarts the run on `game_restart`; registered as the
/// autoload `Run`.
///
/// The autoload name differs from this class name on purpose: in GDScript an autoload with the
/// same name as a class hides the class and breaks typed code that uses it.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct RunTracker {
    record: RunRecord,
    /// The restart action, once checked to exist; `None` if the Input Map lacks it.
    restart_action: Option<StringName>,
    base: Base<Node>,
}

#[godot_api]
impl INode for RunTracker {
    fn init(base: Base<Node>) -> Self {
        // A run is playing from the moment the game starts.
        Self {
            record: RunRecord::new(),
            restart_action: None,
            base,
        }
    }

    fn ready(&mut self) {
        if InputMap::singleton().has_action(RESTART_ACTION) {
            self.restart_action = Some(StringName::from(RESTART_ACTION));
        } else {
            godot_error!("Run: input action `{RESTART_ACTION}` is missing from the Input Map");
        }
    }

    fn process(&mut self, _delta: f64) {
        // Read every frame, frozen or not, in every build: the restart must always work.
        let pressed = self
            .restart_action
            .as_ref()
            .is_some_and(|action| Input::singleton().is_action_just_pressed(action));
        if pressed {
            self.restart();
        }
    }
}

#[godot_api]
impl RunTracker {
    /// Emitted once per run, when the hero falls; `kills` is the run's final kill count.
    /// Handlers use the argument and call nothing on `Run` during the emit.
    #[signal]
    fn game_over(kills: i64);

    /// Grunts killed this run.
    #[func]
    pub fn kills(&self) -> i64 {
        self.record.kills()
    }

    /// Whether the run is being played; false after the hero has fallen, until the restart.
    #[func]
    pub fn is_playing(&self) -> bool {
        match self.record.state() {
            RunState::Playing => true,
            RunState::GameOver => false,
        }
    }

    /// Starts a fresh run: state and kills reset, any freeze cleared, and the current scene
    /// reloaded, so every fighter of the old run is gone and the spawner sends the first grunt.
    #[func]
    pub fn restart(&mut self) {
        self.record.reset();
        match self.base().try_get_node_as::<HitStopTimer>(HIT_STOP_PATH) {
            Some(mut hit_stop) => hit_stop.bind_mut().clear(),
            None => {
                godot_error!("Run: the HitStop autoload is missing or is not a HitStopTimer");
            }
        }
        let result = self.base().get_tree().reload_current_scene();
        if result != godot::global::Error::OK {
            godot_error!("Run: reloading the scene failed: {result:?}");
        }
    }
}

impl RunTracker {
    /// Counts one kill (a grunt calls it when it starts dying).
    pub fn add_kill(&mut self) {
        self.record.add_kill();
    }

    /// Ends the run when the hero falls, announcing `game_over` the first time only.
    pub fn end_run(&mut self) {
        if self.record.end_run() {
            let kills = self.record.kills();
            self.signals().game_over().emit(kills);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kills_count_only_while_playing() {
        let mut run = RunRecord::new();
        run.add_kill();
        run.add_kill();
        assert_eq!(run.kills(), 2);
        run.end_run();
        run.add_kill();
        assert_eq!(run.kills(), 2);
    }

    #[test]
    fn the_run_ends_once() {
        let mut run = RunRecord::new();
        assert!(run.end_run());
        assert_eq!(run.state(), RunState::GameOver);
        assert!(!run.end_run());
    }

    #[test]
    fn restart_resets_the_run() {
        let mut run = RunRecord::new();
        run.add_kill();
        run.end_run();
        run.reset();
        assert_eq!(run, RunRecord::new());
        assert_eq!((run.state(), run.kills()), (RunState::Playing, 0));
    }
}
