//! Sends grunts into the run: the first at once, then a new one only when none is left.
//!
//! It lives in the scene, so the restart's scene reload rebuilds it and it sends the first grunt of
//! the new run. The rule of when to spawn is the pure [`SpawnerState::tick`], unit-tested without
//! the engine; the node only counts living grunts, rolls the edge and depth, and adds the grunt.

use godot::classes::{INode, Node, Node2D, PackedScene, RandomNumberGenerator};
use godot::prelude::*;

use crate::combat::Facing;
use crate::fighter::Countdown;
use crate::grunt::Grunt;
use crate::hero::Hero;
use crate::hit_stop::HitStopTimer;
use crate::run::RunTracker;
use crate::tuning_store::TuningStore;

/// Where the `Tuning` autoload lives in the scene tree.
const TUNING_PATH: &str = "/root/Tuning";

/// Where the `HitStop` autoload lives in the scene tree.
const HIT_STOP_PATH: &str = "/root/HitStop";

/// Where the `Run` autoload lives in the scene tree.
const RUN_PATH: &str = "/root/Run";

/// When the spawner sends the next grunt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpawnerState {
    /// The run has just started; the first grunt comes as soon as tuning is valid.
    AwaitingFirst,
    /// Grunts are alive; waiting for none to be left.
    Watching,
    /// None is left; the respawn delay is counting down.
    Waiting(Countdown),
}

impl SpawnerState {
    /// One tick of the spawn rule. `living` counts grunts that are not dying (entering ones count);
    /// returns the next state and whether to spawn a grunt now. Nothing happens without valid
    /// tuning or once the run is over. A grunt that appears during the delay (F3) cancels it,
    /// because a new grunt comes only when none is left.
    pub fn tick(
        self,
        living: usize,
        tuning_ready: bool,
        playing: bool,
        respawn_delay: f32,
        delta: f32,
    ) -> (Self, bool) {
        if !tuning_ready || !playing {
            return (self, false);
        }
        match self {
            Self::AwaitingFirst => (Self::Watching, true),
            Self::Watching if living == 0 => (Self::Waiting(Countdown::new(respawn_delay)), false),
            Self::Watching => (Self::Watching, false),
            Self::Waiting(_) if living > 0 => (Self::Watching, false),
            Self::Waiting(mut delay) => {
                if delay.tick(delta) {
                    (Self::Watching, true)
                } else {
                    (Self::Waiting(delay), false)
                }
            }
        }
    }
}

/// Adds grunts to the world by the rule in [`SpawnerState::tick`], each from a random edge at a
/// random lane depth. If its exports or the autoloads are missing, it reports one error and never
/// spawns.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct GruntSpawner {
    /// The grunt scene to add (`res://world/grunt.tscn`). Set in the editor.
    #[export]
    grunt_scene: Option<Gd<PackedScene>>,
    /// The Y-sorted node grunts are added to, before the hero. Set in the editor.
    #[export]
    world: Option<Gd<Node2D>>,
    state: SpawnerState,
    rng: Gd<RandomNumberGenerator>,
    /// The autoloads it reads every tick; `None` until `ready()` finds them, and for good if not.
    links: Option<Links>,
    base: Base<Node>,
}

/// The autoloads a spawner reads every tick, found once in `ready()`.
struct Links {
    tuning: Gd<TuningStore>,
    hit_stop: Gd<HitStopTimer>,
    run: Gd<RunTracker>,
}

#[godot_api]
impl INode for GruntSpawner {
    fn init(base: Base<Node>) -> Self {
        // The first grunt is sent once tuning is valid; `links` is filled by `ready()`.
        Self {
            grunt_scene: None,
            world: None,
            state: SpawnerState::AwaitingFirst,
            rng: RandomNumberGenerator::new_gd(),
            links: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.rng.randomize();
        if self.grunt_scene.is_none() || self.world.is_none() {
            godot_error!("GruntSpawner: `grunt_scene` and `world` must both be set in the editor");
            return;
        }
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
                    "GruntSpawner: the Tuning, HitStop or Run autoload is missing or has the \
                     wrong type"
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
        let values = links.tuning.bind().values();
        let playing = links.run.bind().is_playing();
        let respawn_delay = values.map_or(0.0, |v| v.run().respawn_delay());
        let living = self.living_grunts();
        let (state, spawn) = self.state.tick(
            living,
            values.is_some(),
            playing,
            respawn_delay,
            delta as f32,
        );
        self.state = state;
        if spawn {
            self.spawn();
        }
    }
}

#[godot_api]
impl GruntSpawner {
    /// Sends one more grunt now, the normal way: from a random edge at a random lane depth. For the
    /// F3 cheat.
    #[func]
    pub fn spawn_now(&mut self) {
        self.spawn();
    }
}

impl GruntSpawner {
    /// Grunts in the world that are not dying; entering ones count.
    fn living_grunts(&self) -> usize {
        self.world.as_ref().map_or(0, |world| {
            world
                .get_children()
                .iter_shared()
                .filter_map(|child| child.try_cast::<Grunt>().ok())
                .filter(|grunt| grunt.bind().is_alive())
                .count()
        })
    }

    /// Adds one grunt just off a random edge at a random lane depth, before the hero in the world,
    /// so the hero still wins depth ties.
    fn spawn(&mut self) {
        let Some(values) = self
            .links
            .as_ref()
            .and_then(|links| links.tuning.bind().values())
        else {
            return;
        };
        let (Some(scene), Some(mut world)) = (self.grunt_scene.clone(), self.world.clone()) else {
            return;
        };
        let Some(mut grunt) = scene
            .instantiate()
            .and_then(|node| node.try_cast::<Grunt>().ok())
        else {
            godot_error!("GruntSpawner: `grunt_scene` does not have a Grunt at its root");
            return;
        };
        let half_width = values.grunt().hurtbox_width() / 2.0;
        let screen_width = world.get_viewport_rect().size.x;
        let (edge, x) = if self.rng.randi_range(0, 1) == 0 {
            (Facing::Left, -half_width)
        } else {
            (Facing::Right, screen_width + half_width)
        };
        let depth = self
            .rng
            .randf_range(values.lane_top(), values.lane_bottom());
        grunt.bind_mut().place(edge, Vector2::new(x, depth));
        world.add_child(&grunt);
        let hero_index = world
            .get_children()
            .iter_shared()
            .find_map(|child| child.try_cast::<Hero>().ok())
            .map(|hero| hero.get_index());
        if let Some(index) = hero_index {
            world.move_child(&grunt, index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values deliberately different from the starting values in tuning.json.
    const DELAY: f32 = 1.2;
    const TICK: f32 = 0.1;

    #[test]
    fn the_first_grunt_comes_at_once_when_tuning_is_valid() {
        let (state, spawn) = SpawnerState::AwaitingFirst.tick(0, true, true, DELAY, TICK);
        assert_eq!((state, spawn), (SpawnerState::Watching, true));
        let waiting = SpawnerState::AwaitingFirst.tick(0, false, true, DELAY, TICK);
        assert_eq!(waiting, (SpawnerState::AwaitingFirst, false));
    }

    #[test]
    fn nothing_happens_while_a_grunt_lives() {
        let (state, spawn) = SpawnerState::Watching.tick(1, true, true, DELAY, TICK);
        assert_eq!((state, spawn), (SpawnerState::Watching, false));
    }

    #[test]
    fn the_spawner_waits_the_delay_once_none_is_left() {
        let (mut state, spawn) = SpawnerState::Watching.tick(0, true, true, DELAY, TICK);
        assert!(!spawn);
        assert_eq!(state, SpawnerState::Waiting(Countdown::new(DELAY)));
        let mut ticks = 0;
        loop {
            let (next, spawn) = state.tick(0, true, true, DELAY, TICK);
            ticks += 1;
            state = next;
            if spawn {
                break;
            }
            assert!(ticks < 100, "never spawned");
        }
        // 1.2 s at 0.1 s per tick, give or take one tick of rounding.
        assert!((12..=13).contains(&ticks), "{ticks}");
        assert_eq!(state, SpawnerState::Watching);
    }

    #[test]
    fn a_grunt_arriving_during_the_delay_cancels_it() {
        let waiting = SpawnerState::Waiting(Countdown::new(DELAY));
        assert_eq!(
            waiting.tick(1, true, true, DELAY, TICK),
            (SpawnerState::Watching, false)
        );
    }

    #[test]
    fn nothing_happens_once_the_run_is_over_or_without_tuning() {
        for state in [
            SpawnerState::AwaitingFirst,
            SpawnerState::Watching,
            SpawnerState::Waiting(Countdown::new(0.0)),
        ] {
            assert_eq!(state.tick(0, true, false, DELAY, TICK), (state, false));
            assert_eq!(state.tick(0, false, true, DELAY, TICK), (state, false));
        }
    }
}
