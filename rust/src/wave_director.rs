//! Sends the run's enemies in waves: a banner, a stream of the wave's enemies, a breather once the
//! wave is cleared, then the next, larger wave.
//!
//! The rules are pure and unit-tested without the engine: [`composition`] says what a wave holds,
//! [`spawn_queue`] the order its enemies enter, and [`WaveProgress::tick`] when the wave advances
//! and when an enemy enters. The [`WaveDirector`] node only counts living enemies, rolls each
//! enemy's edge and depth, adds it to the world and emits the wave signals. It also runs the crowd
//! rules each frame, after every enemy has moved: it hands out attack tokens and keeps enemies
//! apart ([`crate::crowd`]). It lives in the scene, so the restart's scene reload rebuilds it at
//! wave 1.

use std::collections::VecDeque;

use godot::classes::{INode, Node, Node2D, PackedScene, RandomNumberGenerator};
use godot::prelude::*;

use crate::combat::Facing;
use crate::crowd::{AttackTokens, TurnRequest, separate};
use crate::enemy::{Enemy, EnemyKind};
use crate::fighter::Countdown;
use crate::hero::Hero;
use crate::hit_stop::HitStopTimer;
use crate::potion::Potion;
use crate::run::RunTracker;
use crate::tuning::{WaveSpec, WaveTuning};
use crate::tuning_store::TuningStore;

/// Where the `Tuning` autoload lives in the scene tree.
const TUNING_PATH: &str = "/root/Tuning";

/// Where the `HitStop` autoload lives in the scene tree.
const HIT_STOP_PATH: &str = "/root/HitStop";

/// Where the `Run` autoload lives in the scene tree.
const RUN_PATH: &str = "/root/Run";

/// The director's physics priority: higher runs later, so it always acts after every enemy has
/// moved in the same frame (research R7). An ordering, not a tuning value.
const PHYSICS_PRIORITY: i32 = 1;

/// What wave `number` (from 1) holds: the list's entry while there is one; past the last entry,
/// that entry plus `extra_grunts_per_wave` grunts per wave and one more brute every
/// `brute_every_waves` waves (0: no more brutes). No wave past the list is smaller than the one
/// before it.
pub fn composition(number: u32, waves: &WaveTuning) -> WaveSpec {
    let list = waves.list();
    let written = list.len() as u32;
    let index = number.max(1) - 1;
    if index < written {
        return list[index as usize];
    }
    let last = list[list.len() - 1];
    let past = number.max(1) - written;
    let extra_brutes = match waves.brute_every_waves() {
        0 => 0,
        every => past / every,
    };
    WaveSpec {
        grunts: last
            .grunts
            .saturating_add(past.saturating_mul(waves.extra_grunts_per_wave())),
        brutes: last.brutes.saturating_add(extra_brutes),
    }
}

/// The order a wave's enemies enter: its grunts, with its brutes spread evenly through them (brute
/// `i` of `b` at index `round((i + 1) · t / (b + 1)) − 1`), so the brutes neither all come at once
/// nor all come last.
pub fn spawn_queue(spec: WaveSpec) -> Vec<EnemyKind> {
    let total = spec.total() as usize;
    let mut queue = vec![EnemyKind::Grunt; total];
    let brutes = spec.brutes as usize;
    for i in 0..brutes {
        let at = ((i + 1) * total) as f64 / (brutes + 1) as f64;
        let index = (at.round() as usize).saturating_sub(1).min(total - 1);
        // Two brutes can round to one slot when brutes outnumber grunts; take the next free one.
        let free = (index..total)
            .chain(0..index)
            .find(|&slot| queue[slot] == EnemyKind::Grunt)
            .unwrap_or(index);
        queue[free] = EnemyKind::Brute;
    }
    queue
}

/// Where the current wave is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveState {
    /// The wave's banner shows; its enemies enter when the countdown runs out.
    Banner(Countdown),
    /// The wave's enemies are entering or alive.
    Fighting,
    /// The wave is cleared; the next banner starts when the countdown runs out.
    Breather(Countdown),
}

/// What one tick of [`WaveProgress::tick`] asks the director to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveEvent {
    /// Nothing.
    None,
    /// Add one enemy of this kind.
    Spawn(EnemyKind),
    /// The wave with this number was cleared; its breather starts.
    Cleared(u32),
    /// The banner of the wave with this number starts.
    Started(u32),
}

/// The run's progress through its waves.
#[derive(Debug, Clone, PartialEq)]
pub struct WaveProgress {
    number: u32,
    state: WaveState,
    queue: VecDeque<EnemyKind>,
    spawn_timer: Countdown,
    /// Whether the current banner's start is still to be reported (wave 1, on the first ready
    /// tick).
    unannounced: bool,
}

impl WaveProgress {
    /// The start of a run: wave 1's banner, reported on the first tick with tuning and play.
    pub fn first(waves: &WaveTuning) -> Self {
        Self {
            number: 1,
            state: WaveState::Banner(Countdown::new(waves.banner_time())),
            queue: spawn_queue(composition(1, waves)).into(),
            spawn_timer: Countdown::new(0.0),
            unannounced: true,
        }
    }

    /// Starts the next wave's banner. Its queue is filled now, from the values in use as the
    /// banner starts, so a reload during the banner applies from the wave after.
    fn next_banner(&mut self, waves: &WaveTuning) -> WaveEvent {
        self.number += 1;
        self.queue
            .extend(spawn_queue(composition(self.number, waves)));
        self.state = WaveState::Banner(Countdown::new(waves.banner_time()));
        WaveEvent::Started(self.number)
    }

    /// The current wave's number, from 1.
    pub fn number(&self) -> u32 {
        self.number
    }

    /// Where the current wave is.
    pub fn state(&self) -> WaveState {
        self.state
    }

    /// The enemies still to enter in this wave, in order.
    #[cfg(test)]
    pub fn queued(&self) -> Vec<EnemyKind> {
        self.queue.iter().copied().collect()
    }

    /// One tick of the wave rule. `living` counts the director's enemies that are not dying
    /// (entering ones count). Returns at most one event per tick.
    pub fn tick(&mut self, living: u32, waves: &WaveTuning, delta: f32) -> WaveEvent {
        if self.unannounced {
            self.unannounced = false;
            return WaveEvent::Started(self.number);
        }
        match &mut self.state {
            WaveState::Banner(countdown) => {
                if countdown.tick(delta) {
                    self.state = WaveState::Fighting;
                    self.spawn_timer = Countdown::new(0.0);
                }
                WaveEvent::None
            }
            WaveState::Fighting => {
                if self.queue.is_empty() {
                    if living == 0 {
                        self.state = WaveState::Breather(Countdown::new(waves.breather_time()));
                        return WaveEvent::Cleared(self.number);
                    }
                    return WaveEvent::None;
                }
                let ready = self.spawn_timer.tick(delta);
                if ready
                    && living < waves.max_on_screen()
                    && let Some(kind) = self.queue.pop_front()
                {
                    self.spawn_timer = Countdown::new(waves.spawn_interval());
                    return WaveEvent::Spawn(kind);
                }
                WaveEvent::None
            }
            WaveState::Breather(countdown) => {
                if living > 0 {
                    // An enemy from F3 or F7 joined the cleared wave; it must be cleared again.
                    self.state = WaveState::Fighting;
                    WaveEvent::None
                } else if countdown.tick(delta) {
                    self.next_banner(waves)
                } else {
                    WaveEvent::None
                }
            }
        }
    }

    /// F3 or F7: one more enemy of `kind` joins the current wave, first in its queue. During a
    /// breather, the wave is fought again until it is cleared.
    pub fn push_front(&mut self, kind: EnemyKind) {
        self.queue.push_front(kind);
        self.spawn_timer = Countdown::new(0.0);
        match self.state {
            WaveState::Breather(_) => self.state = WaveState::Fighting,
            WaveState::Banner(_) | WaveState::Fighting => {}
        }
    }

    /// F8: the queue is emptied and the next wave's banner starts at once, skipping any breather.
    /// It counts nothing: the director removes the enemies without a hit.
    pub fn skip(&mut self, waves: &WaveTuning) -> WaveEvent {
        self.queue.clear();
        self.unannounced = false;
        self.next_banner(waves)
    }
}

/// Sends the run's enemies in waves by the rule in [`WaveProgress::tick`], each from a random edge
/// at a random lane depth. If its exports or the autoloads are missing, it reports one error and
/// never spawns.
#[derive(GodotClass)]
#[class(base = Node)]
pub struct WaveDirector {
    /// The grunt scene (`res://world/grunt.tscn`). Set in the editor.
    #[export]
    grunt_scene: Option<Gd<PackedScene>>,
    /// The brute scene (`res://world/brute.tscn`). Set in the editor.
    #[export]
    brute_scene: Option<Gd<PackedScene>>,
    /// The potion scene (`res://world/potion.tscn`), added where an enemy's death left one. Set in
    /// the editor.
    #[export]
    potion_scene: Option<Gd<PackedScene>>,
    /// The Y-sorted node enemies and potions are added to, before the hero. Set in the editor.
    #[export]
    world: Option<Gd<Node2D>>,
    /// The run's waves; `None` until the first tick with tuning.
    progress: Option<WaveProgress>,
    /// Who holds a turn; the pool's size follows `crowd.max_attack_tokens` every tick.
    tokens: AttackTokens<InstanceId>,
    rng: Gd<RandomNumberGenerator>,
    /// The autoloads it reads every tick; `None` until `ready()` finds them, and for good if not.
    links: Option<Links>,
    base: Base<Node>,
}

/// The autoloads the director reads every tick, found once in `ready()`.
struct Links {
    tuning: Gd<TuningStore>,
    hit_stop: Gd<HitStopTimer>,
    run: Gd<RunTracker>,
}

#[godot_api]
impl INode for WaveDirector {
    fn init(base: Base<Node>) -> Self {
        Self {
            grunt_scene: None,
            brute_scene: None,
            potion_scene: None,
            world: None,
            progress: None,
            tokens: AttackTokens::new(0),
            rng: RandomNumberGenerator::new_gd(),
            links: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.rng.randomize();
        self.base_mut()
            .set_physics_process_priority(PHYSICS_PRIORITY);
        if self.grunt_scene.is_none()
            || self.brute_scene.is_none()
            || self.potion_scene.is_none()
            || self.world.is_none()
        {
            godot_error!(
                "WaveDirector: `grunt_scene`, `brute_scene`, `potion_scene` and `world` must all be \
                 set in the editor"
            );
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
                    "WaveDirector: the Tuning, HitStop or Run autoload is missing or has the \
                     wrong type"
                );
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if self.links.is_none() {
            return;
        }
        // Before the freeze check: a kill freezes play, and its potion should show at once.
        self.place_drops();
        let Some(links) = &self.links else {
            return;
        };
        if links.hit_stop.bind().is_frozen()
            || links.run.bind().is_cast_frozen()
            || !links.run.bind().is_playing()
        {
            return;
        }
        let Some(waves) = links.tuning.bind().waves() else {
            return;
        };
        let living = self.living_enemies().len() as u32;
        let progress = self
            .progress
            .get_or_insert_with(|| WaveProgress::first(&waves));
        let event = progress.tick(living, &waves, delta as f32);
        self.handle(event);
        self.run_crowd();
    }
}

#[godot_api]
impl WaveDirector {
    /// Emitted when a wave's banner starts, with the wave's number (1 at the start of a run).
    /// Handlers must not call back into the director.
    #[signal]
    fn wave_started(number: i64);

    /// Emitted when the last enemy of a fully spawned wave has died; the breather starts.
    /// Handlers must not call back into the director.
    #[signal]
    fn wave_cleared(number: i64);

    /// The current wave's number, from 1; 0 before the first tick with tuning.
    #[func]
    pub fn wave(&self) -> i64 {
        self.progress.as_ref().map_or(0, |p| i64::from(p.number()))
    }

    /// The wave's number while its banner shows, else 0. The banner polls it.
    #[func]
    pub fn banner_wave(&self) -> i64 {
        self.progress
            .as_ref()
            .map_or(0, |progress| match progress.state() {
                WaveState::Banner(_) => i64::from(progress.number()),
                WaveState::Fighting | WaveState::Breather(_) => 0,
            })
    }

    /// F3: one more grunt joins the current wave. Nothing at Game Over.
    #[func]
    pub fn spawn_grunt(&mut self) {
        self.spawn_now(EnemyKind::Grunt);
    }

    /// F7: one more brute joins the current wave. Nothing at Game Over.
    #[func]
    pub fn spawn_brute(&mut self) {
        self.spawn_now(EnemyKind::Brute);
    }

    /// F8: every enemy is removed with no points or kills (freed directly, never through a hit),
    /// the queue is emptied, and the next wave's banner starts at once. Nothing at Game Over.
    #[func]
    pub fn skip_wave(&mut self) {
        if !self.is_playing() {
            return;
        }
        let Some(waves) = self.links.as_ref().and_then(|l| l.tuning.bind().waves()) else {
            return;
        };
        for mut enemy in self.all_enemies() {
            enemy.queue_free();
        }
        let progress = self
            .progress
            .get_or_insert_with(|| WaveProgress::first(&waves));
        let event = progress.skip(&waves);
        self.handle(event);
    }
}

impl WaveDirector {
    /// One more enemy of `kind` joins the current wave, first in its queue. Nothing at Game Over
    /// or before the first tick with tuning.
    pub fn spawn_now(&mut self, kind: EnemyKind) {
        if !self.is_playing() {
            return;
        }
        if let Some(progress) = &mut self.progress {
            progress.push_front(kind);
        }
    }

    /// One pass of the crowd rules, after every enemy has moved this frame: frees the tokens of
    /// enemies that no longer hold a turn, hands free tokens to waiting enemies (an overdue one
    /// first, else the nearest), and
    /// pushes enemies that came too close apart. Enemies never call back into the director, so no
    /// enemy is bound while this runs.
    fn run_crowd(&mut self) {
        let Some(values) = self.links.as_ref().and_then(|l| l.tuning.bind().values()) else {
            return;
        };
        let crowd = values.crowd();
        let mut enemies = self.living_enemies();
        self.tokens.set_capacity(crowd.max_attack_tokens());
        let holding: Vec<InstanceId> = enemies
            .iter()
            .filter(|enemy| enemy.bind().has_turn())
            .map(|enemy| enemy.instance_id())
            .collect();
        self.tokens.reconcile(&holding);
        let waiting: Vec<TurnRequest<InstanceId>> = enemies
            .iter()
            .filter_map(|enemy| {
                let enemy_ref = enemy.bind();
                let (distance, waited) = enemy_ref.turn_request()?;
                Some(TurnRequest {
                    id: enemy.instance_id(),
                    cost: enemy_ref.turn_cost()?,
                    distance,
                    waited,
                })
            })
            .collect();
        for id in self.tokens.grant(&waiting, crowd.max_wait_time()) {
            if let Some(enemy) = enemies.iter_mut().find(|e| e.instance_id() == id) {
                enemy.bind_mut().grant_turn();
            }
        }
        let hero = values.hero();
        let depth_scale = hero.horizontal_speed() / hero.vertical_speed();
        let mut in_pass = Vec::new();
        let mut bodies = Vec::new();
        for enemy in enemies {
            if let Some(body) = enemy.bind().crowd_body() {
                bodies.push(body);
                in_pass.push(enemy.clone());
            }
        }
        let offsets = separate(&bodies, crowd.separation_radius(), depth_scale);
        for (mut enemy, (dx, dy)) in in_pass.into_iter().zip(offsets) {
            if dx != 0.0 || dy != 0.0 {
                enemy.bind_mut().nudge(dx, dy);
            }
        }
    }

    /// Whether the run is being played.
    fn is_playing(&self) -> bool {
        self.links
            .as_ref()
            .is_some_and(|links| links.run.bind().is_playing())
    }

    /// Acts on one event of the wave rule.
    fn handle(&mut self, event: WaveEvent) {
        match event {
            WaveEvent::None => {}
            WaveEvent::Spawn(kind) => self.spawn(kind),
            WaveEvent::Cleared(number) => {
                self.signals().wave_cleared().emit(i64::from(number));
            }
            WaveEvent::Started(number) => {
                if let Some(links) = self.links.as_mut() {
                    links.run.bind_mut().set_wave(i64::from(number));
                }
                self.signals().wave_started().emit(i64::from(number));
            }
        }
    }

    /// Every enemy in the world, dying ones included.
    fn all_enemies(&self) -> Vec<Gd<Enemy>> {
        self.world.as_ref().map_or_else(Vec::new, |world| {
            world
                .get_children()
                .iter_shared()
                .filter_map(|child| child.try_cast::<Enemy>().ok())
                .collect()
        })
    }

    /// The enemies in the world that are not dying; entering ones count.
    fn living_enemies(&self) -> Vec<Gd<Enemy>> {
        self.all_enemies()
            .into_iter()
            .filter(|enemy| enemy.bind().is_alive())
            .collect()
    }

    /// Adds a potion where each enemy that died since the last tick left one, its feet kept in
    /// the lane, before the hero in the world (as enemies are). F8's removals leave none, because
    /// they are not deaths.
    fn place_drops(&mut self) {
        let Some(values) = self
            .links
            .as_ref()
            .and_then(|links| links.tuning.bind().values())
        else {
            return;
        };
        let (Some(scene), Some(mut world)) = (self.potion_scene.clone(), self.world.clone()) else {
            return;
        };
        for mut enemy in self.all_enemies() {
            let Some((kind, position)) = enemy.bind_mut().take_drop() else {
                continue;
            };
            let Some(mut potion) = scene
                .instantiate()
                .and_then(|node| node.try_cast::<Potion>().ok())
            else {
                godot_error!("WaveDirector: `potion_scene` does not have a Potion at its root");
                return;
            };
            let y = position.y.clamp(values.lane_top(), values.lane_bottom());
            potion.bind_mut().place(kind, Vector2::new(position.x, y));
            world.add_child(&potion);
            Self::put_before_hero(&mut world, &potion.upcast::<Node>());
        }
    }

    /// Moves `node` before the hero among `world`'s children, so the hero wins depth ties.
    fn put_before_hero(world: &mut Gd<Node2D>, node: &Gd<Node>) {
        let hero_index = world
            .get_children()
            .iter_shared()
            .find_map(|child| child.try_cast::<Hero>().ok())
            .map(|hero| hero.get_index());
        if let Some(index) = hero_index {
            world.move_child(node, index);
        }
    }

    /// Adds one enemy of `kind` just off a random edge at a random lane depth, before the hero in
    /// the world, so the hero still wins depth ties.
    fn spawn(&mut self, kind: EnemyKind) {
        let Some(values) = self
            .links
            .as_ref()
            .and_then(|links| links.tuning.bind().values())
        else {
            return;
        };
        // The director's one choice by kind: which scene to add.
        let scene = match kind {
            EnemyKind::Grunt => self.grunt_scene.clone(),
            EnemyKind::Brute => self.brute_scene.clone(),
        };
        let (Some(scene), Some(mut world)) = (scene, self.world.clone()) else {
            return;
        };
        let Some(mut enemy) = scene
            .instantiate()
            .and_then(|node| node.try_cast::<Enemy>().ok())
        else {
            godot_error!("WaveDirector: the {kind:?} scene does not have an Enemy at its root");
            return;
        };
        let half_width = values.enemy(kind).hurtbox_width() / 2.0;
        let screen_width = world.get_viewport_rect().size.x;
        let (edge, x) = if self.rng.randi_range(0, 1) == 0 {
            (Facing::Left, -half_width)
        } else {
            (Facing::Right, screen_width + half_width)
        };
        let depth = self
            .rng
            .randf_range(values.lane_top(), values.lane_bottom());
        enemy.bind_mut().place(edge, Vector2::new(x, depth));
        world.add_child(&enemy);
        Self::put_before_hero(&mut world, &enemy.upcast::<Node>());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values deliberately different from the starting values in tuning.json, and exact in binary
    // so whole numbers of ticks add up to them without rounding.
    const INTERVAL: f32 = 0.5;
    const BREATHER: f32 = 1.25;
    const BANNER: f32 = 0.75;
    const MAX_ON_SCREEN: u32 = 4;
    const TICK: f32 = 0.125;

    fn spec(grunts: u32, brutes: u32) -> WaveSpec {
        WaveSpec { grunts, brutes }
    }

    fn waves_with(extra: u32, every: u32) -> WaveTuning {
        WaveTuning::for_tests(
            vec![spec(1, 0), spec(3, 0), spec(2, 1)],
            extra,
            every,
            (INTERVAL, BREATHER, BANNER),
            MAX_ON_SCREEN,
        )
    }

    fn waves() -> WaveTuning {
        waves_with(2, 3)
    }

    /// Ticks `progress` with `living` enemies until `seconds` have passed, collecting the events.
    fn run_for(
        progress: &mut WaveProgress,
        living: u32,
        waves: &WaveTuning,
        seconds: f32,
    ) -> Vec<WaveEvent> {
        let ticks = (seconds / TICK).round() as usize;
        (0..ticks)
            .map(|_| progress.tick(living, waves, TICK))
            .filter(|event| *event != WaveEvent::None)
            .collect()
    }

    /// A progress past wave 1's banner, in `Fighting` with its queue full.
    fn fighting() -> (WaveProgress, WaveTuning) {
        let waves = waves();
        let mut progress = WaveProgress::first(&waves);
        assert_eq!(progress.tick(0, &waves, TICK), WaveEvent::Started(1));
        run_for(&mut progress, 0, &waves, BANNER);
        assert_eq!(progress.state(), WaveState::Fighting);
        (progress, waves)
    }

    // --- Composition. ---

    #[test]
    fn written_waves_come_from_the_list() {
        let waves = waves();
        assert_eq!(composition(1, &waves), spec(1, 0));
        assert_eq!(composition(2, &waves), spec(3, 0));
        assert_eq!(composition(3, &waves), spec(2, 1));
    }

    #[test]
    fn waves_past_the_list_follow_the_escalation_rule() {
        let waves = waves();
        // Last written wave: 2 grunts, 1 brute. +2 grunts per wave, +1 brute every 3 waves.
        assert_eq!(composition(4, &waves), spec(4, 1));
        assert_eq!(composition(5, &waves), spec(6, 1));
        assert_eq!(composition(6, &waves), spec(8, 2));
        assert_eq!(composition(9, &waves), spec(14, 3));
    }

    #[test]
    fn brute_every_zero_adds_no_brutes() {
        let waves = waves_with(1, 0);
        assert_eq!(composition(10, &waves), spec(9, 1));
    }

    #[test]
    fn no_wave_is_smaller_than_the_one_before_past_the_list() {
        for (extra, every) in [(2, 3), (0, 1), (1, 0)] {
            let waves = waves_with(extra, every);
            let last_written = waves.list().len() as u32;
            for n in last_written..last_written + 20 {
                assert!(
                    composition(n + 1, &waves).total() >= composition(n, &waves).total(),
                    "wave {n} → {} with {extra}/{every}",
                    n + 1
                );
            }
        }
    }

    #[test]
    fn the_spawn_queue_spreads_brutes() {
        use EnemyKind::{Brute as B, Grunt as G};
        assert_eq!(spawn_queue(spec(3, 1)), vec![G, B, G, G]);
        assert_eq!(spawn_queue(spec(4, 0)), vec![G, G, G, G]);
        assert_eq!(spawn_queue(spec(0, 2)), vec![B, B]);
        let queue = spawn_queue(spec(7, 2));
        assert_eq!(queue.len(), 9);
        assert_eq!(queue.iter().filter(|k| **k == B).count(), 2);
        assert_ne!(queue[queue.len() - 1], B, "brutes do not all come last");
        assert_ne!(queue[0], B, "brutes do not all come first");
    }

    // --- The wave tick. ---

    #[test]
    fn a_run_starts_with_wave_ones_banner() {
        let waves = waves();
        let mut progress = WaveProgress::first(&waves);
        assert_eq!(progress.number(), 1);
        assert!(matches!(progress.state(), WaveState::Banner(_)));
        assert_eq!(progress.tick(0, &waves, TICK), WaveEvent::Started(1));
        // The queue is filled as the banner starts; the banner holds for its time.
        assert_eq!(progress.queued(), vec![EnemyKind::Grunt]);
        let events = run_for(&mut progress, 0, &waves, BANNER - TICK);
        assert!(events.is_empty(), "{events:?}");
        assert!(matches!(progress.state(), WaveState::Banner(_)));
        progress.tick(0, &waves, TICK);
        assert_eq!(progress.state(), WaveState::Fighting);
    }

    #[test]
    fn a_reload_during_the_banner_applies_from_the_next_wave() {
        let waves = waves();
        let mut progress = WaveProgress::first(&waves);
        progress.tick(0, &waves, TICK);
        let reloaded = WaveTuning::for_tests(
            vec![spec(4, 0), spec(5, 0)],
            1,
            0,
            (INTERVAL, BREATHER, BANNER),
            MAX_ON_SCREEN,
        );
        run_for(&mut progress, 0, &reloaded, BANNER);
        assert_eq!(
            progress.queued(),
            vec![EnemyKind::Grunt],
            "wave 1 keeps its queue"
        );
        assert_eq!(progress.skip(&reloaded), WaveEvent::Started(2));
        assert_eq!(
            progress.queued().len(),
            5,
            "wave 2 follows the reloaded list"
        );
    }

    #[test]
    fn enemies_enter_one_interval_apart() {
        let waves = WaveTuning::for_tests(
            vec![spec(3, 0)],
            1,
            0,
            (INTERVAL, BREATHER, BANNER),
            MAX_ON_SCREEN,
        );
        let mut progress = WaveProgress::first(&waves);
        progress.tick(0, &waves, TICK);
        run_for(&mut progress, 0, &waves, BANNER);
        // First at once, then one every INTERVAL.
        assert_eq!(
            progress.tick(0, &waves, TICK),
            WaveEvent::Spawn(EnemyKind::Grunt)
        );
        let events = run_for(&mut progress, 1, &waves, INTERVAL - TICK);
        assert!(events.is_empty(), "{events:?}");
        assert_eq!(
            progress.tick(1, &waves, TICK),
            WaveEvent::Spawn(EnemyKind::Grunt)
        );
    }

    #[test]
    fn the_on_screen_maximum_holds_the_queue() {
        let (mut progress, waves) = fighting();
        progress.push_front(EnemyKind::Grunt);
        let events = run_for(&mut progress, MAX_ON_SCREEN, &waves, 3.0);
        assert!(events.is_empty(), "{events:?}");
        assert_eq!(
            progress.tick(MAX_ON_SCREEN - 1, &waves, TICK),
            WaveEvent::Spawn(EnemyKind::Grunt)
        );
    }

    #[test]
    fn no_clear_while_enemies_live_or_wait_to_enter() {
        let (mut progress, waves) = fighting();
        // Queue not empty: no clear even with none alive.
        assert_eq!(
            progress.tick(0, &waves, TICK),
            WaveEvent::Spawn(EnemyKind::Grunt)
        );
        // Queue empty, one alive: still fighting.
        let events = run_for(&mut progress, 1, &waves, 5.0);
        assert!(events.is_empty(), "{events:?}");
        assert_eq!(progress.state(), WaveState::Fighting);
        // The last one dies.
        assert_eq!(progress.tick(0, &waves, TICK), WaveEvent::Cleared(1));
        assert!(matches!(progress.state(), WaveState::Breather(_)));
    }

    #[test]
    fn the_breather_then_the_next_banner() {
        let (mut progress, waves) = fighting();
        progress.tick(0, &waves, TICK);
        assert_eq!(progress.tick(0, &waves, TICK), WaveEvent::Cleared(1));
        let events = run_for(&mut progress, 0, &waves, BREATHER - TICK);
        assert!(events.is_empty(), "{events:?}");
        assert_eq!(progress.tick(0, &waves, TICK), WaveEvent::Started(2));
        assert_eq!(progress.number(), 2);
        assert!(matches!(progress.state(), WaveState::Banner(_)));
        assert_eq!(progress.queued().len(), 3);
    }

    #[test]
    fn an_enemy_during_the_breather_returns_to_fighting() {
        let (mut progress, waves) = fighting();
        progress.tick(0, &waves, TICK);
        assert_eq!(progress.tick(0, &waves, TICK), WaveEvent::Cleared(1));
        progress.tick(0, &waves, TICK);
        assert_eq!(progress.tick(1, &waves, TICK), WaveEvent::None);
        assert_eq!(progress.state(), WaveState::Fighting);
        assert_eq!(progress.number(), 1);
        assert_eq!(progress.tick(0, &waves, TICK), WaveEvent::Cleared(1));
    }

    #[test]
    fn push_front_puts_a_kind_first_and_zeroes_the_timer() {
        let (mut progress, waves) = fighting();
        assert_eq!(
            progress.tick(0, &waves, TICK),
            WaveEvent::Spawn(EnemyKind::Grunt)
        );
        progress.push_front(EnemyKind::Brute);
        assert_eq!(progress.queued(), vec![EnemyKind::Brute]);
        // No interval wait: it enters on the next tick.
        assert_eq!(
            progress.tick(1, &waves, TICK),
            WaveEvent::Spawn(EnemyKind::Brute)
        );
    }

    #[test]
    fn a_kind_pushed_during_the_banner_stays_first() {
        let waves = waves();
        let mut progress = WaveProgress::first(&waves);
        progress.tick(0, &waves, TICK);
        progress.push_front(EnemyKind::Brute);
        assert!(matches!(progress.state(), WaveState::Banner(_)));
        run_for(&mut progress, 0, &waves, BANNER);
        assert_eq!(progress.queued(), vec![EnemyKind::Brute, EnemyKind::Grunt]);
    }

    #[test]
    fn push_front_during_the_breather_returns_to_fighting() {
        let (mut progress, waves) = fighting();
        progress.tick(0, &waves, TICK);
        assert_eq!(progress.tick(0, &waves, TICK), WaveEvent::Cleared(1));
        progress.push_front(EnemyKind::Grunt);
        assert_eq!(progress.state(), WaveState::Fighting);
        assert_eq!(
            progress.tick(0, &waves, TICK),
            WaveEvent::Spawn(EnemyKind::Grunt)
        );
    }

    #[test]
    fn skip_empties_the_queue_and_starts_the_next_banner() {
        let (mut progress, waves) = fighting();
        progress.push_front(EnemyKind::Brute);
        assert_eq!(progress.skip(&waves), WaveEvent::Started(2));
        assert_eq!(progress.number(), 2);
        // Only wave 2's own queue: nothing of wave 1's is left.
        assert_eq!(progress.queued().len(), 3);
        assert!(!progress.queued().contains(&EnemyKind::Brute));
        assert!(matches!(progress.state(), WaveState::Banner(_)));
        // From a breather too, with no wait.
        let (mut progress, waves) = fighting();
        progress.tick(0, &waves, TICK);
        progress.tick(0, &waves, TICK);
        assert!(matches!(progress.state(), WaveState::Breather(_)));
        assert_eq!(progress.skip(&waves), WaveEvent::Started(2));
    }
}
