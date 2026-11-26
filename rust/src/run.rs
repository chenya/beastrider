//! The run: from wave 1 to the hero's fall, and the restart that begins the next one.
//!
//! It is the `Run` autoload, so it outlives the scene: the restart reloads the scene, which
//! rebuilds the hero, the wave director, the HUD and the Game Over screen, and leaves nothing of
//! the old run behind. The tuning autoload is not reloaded, so values changed with F5 survive, and
//! neither is this one, so the session best survives until the game closes. Nothing is written to
//! disk.
//!
//! It also holds the cast freeze, counted apart from the impact freeze, which stops everyone but
//! the caster while a spell is cast, and the random generator that rolls potion drops. A restart
//! clears the freeze and reseeds the generator.

use godot::classes::{INode, Input, InputMap, Node, RandomNumberGenerator};
use godot::prelude::*;

use crate::hit_stop::HitStopTimer;
use crate::potion::{PotionKind, drop_from_rolls};

/// The Input Map action that restarts the run, in every build.
const RESTART_ACTION: &str = "game_restart";

/// Where the `HitStop` autoload lives in the scene tree.
const HIT_STOP_PATH: &str = "/root/HitStop";

/// Whether the run is still being played.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    /// The hero is alive; enemies hunt, the waves come.
    Playing,
    /// The hero has fallen; only the restart does anything.
    GameOver,
}

/// The run's state, kills, score and wave, and the session best, with no Godot types, so its
/// rules are unit-tested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunRecord {
    state: RunState,
    kills: i64,
    score: i64,
    /// The wave whose banner last started; 0 before the first.
    wave: i64,
    /// The highest score of any run this session, this one counted once it ends.
    best: i64,
}

impl RunRecord {
    /// A fresh session: playing, nothing counted, no best yet.
    pub fn new() -> Self {
        Self {
            state: RunState::Playing,
            kills: 0,
            score: 0,
            wave: 0,
            best: 0,
        }
    }

    /// The run's state.
    pub fn state(&self) -> RunState {
        self.state
    }

    /// Enemies killed this run.
    pub fn kills(&self) -> i64 {
        self.kills
    }

    /// Points scored this run.
    pub fn score(&self) -> i64 {
        self.score
    }

    /// The wave reached: the one whose banner last started.
    pub fn wave(&self) -> i64 {
        self.wave
    }

    /// The highest score of any finished run this session.
    pub fn best(&self) -> i64 {
        self.best
    }

    /// Counts one kill worth `points`. An enemy adds its kill when it starts dying, so only while
    /// playing; true if it was counted.
    pub fn add_kill(&mut self, points: i64) -> bool {
        match self.state {
            RunState::Playing => {
                self.kills += 1;
                self.score += points;
                true
            }
            RunState::GameOver => false,
        }
    }

    /// Records the wave whose banner started, while playing.
    pub fn set_wave(&mut self, wave: i64) {
        match self.state {
            RunState::Playing => self.wave = wave,
            RunState::GameOver => {}
        }
    }

    /// Ends the run, folding its score into the best; true only the first time, so `game_over` is
    /// announced once.
    pub fn end_run(&mut self) -> bool {
        match self.state {
            RunState::Playing => {
                self.state = RunState::GameOver;
                self.best = self.best.max(self.score);
                true
            }
            RunState::GameOver => false,
        }
    }

    /// Back to a fresh run, keeping the best, with this run's score folded in first (a run ended
    /// with R counts too).
    pub fn reset(&mut self) {
        let best = self.best.max(self.score);
        *self = Self {
            best,
            ..Self::new()
        };
    }
}

impl Default for RunRecord {
    fn default() -> Self {
        Self::new()
    }
}

/// The spell's freeze: while any hero is casting, everything but the casters skips its step.
/// A count, not a flag, so two heroes casting at once end it only when both are done.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CastFreeze {
    casts: u32,
}

impl CastFreeze {
    /// A hero started casting.
    pub fn begin(&mut self) {
        self.casts += 1;
    }

    /// A hero's cast ended; never below none.
    pub fn end(&mut self) {
        self.casts = self.casts.saturating_sub(1);
    }

    /// Whether any cast is in progress.
    pub fn is_frozen(&self) -> bool {
        self.casts > 0
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
    /// The spell's freeze, in its own count apart from `HitStop`, because the caster keeps running.
    cast_freeze: CastFreeze,
    /// Rolls every death's potion; reseeded on each restart (research R10).
    rng: Gd<RandomNumberGenerator>,
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
            cast_freeze: CastFreeze::default(),
            rng: RandomNumberGenerator::new_gd(),
            restart_action: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.rng.randomize();
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
    /// Emitted once per run, when the hero falls, with everything Game Over shows: the run's
    /// kills, score and wave reached, and the session best with this run counted. Handlers use the
    /// arguments and call nothing on `Run` during the emit.
    #[signal]
    fn game_over(kills: i64, score: i64, wave: i64, best: i64);

    /// Emitted when a kill adds points, with the new score. Handlers use the argument.
    #[signal]
    fn score_changed(score: i64);

    /// Enemies killed this run.
    #[func]
    pub fn kills(&self) -> i64 {
        self.record.kills()
    }

    /// Points scored this run.
    #[func]
    pub fn score(&self) -> i64 {
        self.record.score()
    }

    /// The wave reached this run; 0 before the first banner.
    #[func]
    pub fn wave(&self) -> i64 {
        self.record.wave()
    }

    /// The highest score of any finished run since the game was launched.
    #[func]
    pub fn best(&self) -> i64 {
        self.record.best()
    }

    /// Whether the run is being played; false after the hero has fallen, until the restart.
    #[func]
    pub fn is_playing(&self) -> bool {
        match self.record.state() {
            RunState::Playing => true,
            RunState::GameOver => false,
        }
    }

    /// Whether a spell's cast is freezing play; every object but the caster skips its step.
    #[func]
    pub fn is_cast_frozen(&self) -> bool {
        self.cast_freeze.is_frozen()
    }

    /// Starts a fresh run: state, kills, score and wave reset (the best kept, with this run's score
    /// folded in), any freeze (an impact's or a cast's) cleared, and the current scene reloaded, so every fighter of the old
    /// run is gone and the director starts wave 1.
    #[func]
    pub fn restart(&mut self) {
        self.record.reset();
        self.cast_freeze = CastFreeze::default();
        self.rng.randomize();
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
    /// A hero started casting: everything but the casters stops until [`end_cast`](Self::end_cast).
    pub fn begin_cast(&mut self) {
        self.cast_freeze.begin();
    }

    /// A hero's cast reached its impact.
    pub fn end_cast(&mut self) {
        self.cast_freeze.end();
    }

    /// The potion a death leaves, if any: a drop at `chance`, then a health potion at
    /// `health_odds`, else a magic one (see [`drop_from_rolls`]).
    pub fn roll_drop(&mut self, chance: f32, health_odds: f32) -> Option<PotionKind> {
        let roll_a = self.rng.randf();
        let roll_b = self.rng.randf();
        drop_from_rolls(chance, health_odds, roll_a, roll_b)
    }

    /// Counts one kill worth `points` (an enemy calls it when it starts dying), announcing the
    /// new score.
    pub fn add_kill(&mut self, points: i64) {
        if self.record.add_kill(points) {
            let score = self.record.score();
            self.signals().score_changed().emit(score);
        }
    }

    /// Records the wave whose banner started (the director calls it).
    pub fn set_wave(&mut self, wave: i64) {
        self.record.set_wave(wave);
    }

    /// Ends the run when the hero falls, announcing `game_over` the first time only.
    pub fn end_run(&mut self) {
        if self.record.end_run() {
            let r = self.record;
            self.signals()
                .game_over()
                .emit(r.kills(), r.score(), r.wave(), r.best());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kills_and_points_count_only_while_playing() {
        let mut run = RunRecord::new();
        assert!(run.add_kill(10));
        assert!(run.add_kill(30));
        assert_eq!((run.kills(), run.score()), (2, 40));
        run.end_run();
        assert!(!run.add_kill(10));
        assert_eq!((run.kills(), run.score()), (2, 40));
    }

    #[test]
    fn the_wave_is_set_only_while_playing() {
        let mut run = RunRecord::new();
        assert_eq!(run.wave(), 0);
        run.set_wave(3);
        assert_eq!(run.wave(), 3);
        run.end_run();
        run.set_wave(4);
        assert_eq!(run.wave(), 3);
    }

    #[test]
    fn the_run_ends_once_and_sets_the_best() {
        let mut run = RunRecord::new();
        run.add_kill(50);
        assert!(run.end_run());
        assert_eq!(run.state(), RunState::GameOver);
        assert_eq!(run.best(), 50);
        assert!(!run.end_run());
        assert_eq!(run.best(), 50);
    }

    #[test]
    fn the_cast_freeze_ends_only_when_every_cast_has_ended() {
        let mut freeze = CastFreeze::default();
        assert!(!freeze.is_frozen());
        freeze.begin();
        freeze.begin();
        freeze.end();
        assert!(freeze.is_frozen(), "one cast still in progress");
        freeze.end();
        assert!(!freeze.is_frozen());
        freeze.end();
        assert!(
            !freeze.is_frozen(),
            "an end with none in progress stays at none"
        );
        freeze.begin();
        assert!(freeze.is_frozen());
    }

    #[test]
    fn restart_resets_the_run_and_keeps_the_best() {
        let mut run = RunRecord::new();
        let mut bests = Vec::new();
        for score in [50, 20, 80] {
            run.set_wave(2);
            run.add_kill(score);
            run.end_run();
            bests.push(run.best());
            run.reset();
            assert_eq!(
                (run.state(), run.kills(), run.score(), run.wave()),
                (RunState::Playing, 0, 0, 0)
            );
        }
        assert_eq!(bests, [50, 50, 80]);
        assert_eq!(run.best(), 80);
    }

    #[test]
    fn a_run_ended_with_restart_counts_for_the_best() {
        let mut run = RunRecord::new();
        run.add_kill(40);
        run.end_run();
        run.reset();
        run.add_kill(70);
        run.reset();
        assert_eq!(run.best(), 70);
        run.add_kill(10);
        run.reset();
        assert_eq!(run.best(), 70);
    }

    #[test]
    fn only_add_kill_changes_kills_and_score() {
        // skip_wave frees enemies directly and never calls add_kill, so it awards nothing.
        let mut run = RunRecord::new();
        run.add_kill(10);
        let before = (run.kills(), run.score());
        run.set_wave(5);
        assert_eq!((run.kills(), run.score()), before);
    }
}
