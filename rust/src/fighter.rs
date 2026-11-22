//! The parts every fighter shares, as plain structs and functions with no Godot types.
//!
//! The hero and the grunt both need health, timed attacks, knockback, stun timers and the same hit
//! rule; keeping them here means the two cannot drift apart, and every rule is unit-tested without
//! the engine (constitution, Principle III). Godot classes hold these as fields; nothing here is a
//! node.

use crate::combat::{self, Facing, Rect};
use crate::movement::Speeds;
use crate::tuning::AttackTuning;

/// The result of a blow on a fighter's [`Health`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Damage {
    /// Health went down but is still above 0.
    Survived,
    /// This blow took health to 0. Reported once per fighter, so a death is handled once.
    Killed,
    /// Health was already 0; nothing changed.
    AlreadyDead,
}

/// A fighter's health points, never below 0 or above the maximum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Health {
    current: f32,
    max: f32,
}

impl Health {
    /// Full health with `max` points.
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    /// Points left.
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Points when full.
    pub fn max(&self) -> f32 {
        self.max
    }

    /// Whether no points are left.
    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    /// Takes `amount` points, stopping at 0, and says whether this blow was the killing one.
    pub fn damage(&mut self, amount: f32) -> Damage {
        if self.is_dead() {
            return Damage::AlreadyDead;
        }
        self.current = (self.current - amount).max(0.0);
        if self.is_dead() {
            Damage::Killed
        } else {
            Damage::Survived
        }
    }

    /// Back to full health.
    pub fn refill(&mut self) {
        self.current = self.max;
    }

    /// Takes a new maximum from a tuning reload: current health is lowered to it if needed, and
    /// never raised, so a reload cannot heal.
    pub fn set_max(&mut self, max: f32) {
        self.max = max;
        self.current = self.current.min(max);
    }
}

/// How far and for how long a blow pushes its target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KnockbackSpec {
    /// Pixels the target slides; 0 means no knockback.
    pub distance: f32,
    /// Seconds the slide takes.
    pub time: f32,
}

/// One attack's values, copied when the attack starts so a reload applies from the next attack.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackSpec {
    /// Seconds before the blow can land.
    pub wind_up: f32,
    /// Seconds during which the blow can land.
    pub active: f32,
    /// Seconds after the active part before the attacker is free.
    pub recovery: f32,
    /// How far the attack reaches in front of the body, in pixels.
    pub reach: f32,
    /// How tall the attack box is above the feet, in pixels.
    pub height: f32,
    /// Health points a landed blow takes.
    pub damage: f32,
    /// The push a landed blow gives.
    pub knockback: KnockbackSpec,
}

impl AttackSpec {
    /// The spec for one fighter's checked `attack` values.
    pub fn from_tuning(attack: &AttackTuning) -> Self {
        Self {
            wind_up: attack.wind_up_time(),
            active: attack.active_time(),
            recovery: attack.recovery_time(),
            reach: attack.reach(),
            height: attack.height(),
            damage: attack.damage(),
            knockback: KnockbackSpec {
                distance: attack.knockback_distance(),
                time: attack.knockback_time(),
            },
        }
    }
}

/// The three timed parts of an attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackPhase {
    /// Before the blow can land.
    WindUp,
    /// The blow can land.
    Active,
    /// After the blow.
    Recovery,
}

/// Whether an attack goes on after a tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackStep {
    /// The attack goes on.
    Continue,
    /// The attack ended; the attacker is free.
    Finished,
}

/// The result of [`AttackTimer::advance`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttackAdvance {
    /// Whether the attack goes on or ended.
    pub step: AttackStep,
    /// Whether the attack was in its active part at any moment of the tick, so a blow is checked
    /// even when the active part is shorter than a tick.
    pub was_active: bool,
}

/// One attack under way: its part, the time spent in it, and its values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackTimer {
    phase: AttackPhase,
    elapsed: f32,
    spec: AttackSpec,
}

impl AttackTimer {
    /// A new attack at the start of its wind-up.
    pub fn new(spec: AttackSpec) -> Self {
        Self {
            phase: AttackPhase::WindUp,
            elapsed: 0.0,
            spec,
        }
    }

    /// The part the attack is in.
    pub fn phase(&self) -> AttackPhase {
        self.phase
    }

    /// The values the attack started with.
    pub fn spec(&self) -> AttackSpec {
        self.spec
    }

    /// Moves the attack on by `delta` seconds, through as many parts as that covers. A part of
    /// length 0 is passed through in the same tick.
    pub fn advance(&mut self, delta: f32) -> AttackAdvance {
        self.elapsed += delta;
        let mut was_active = self.phase == AttackPhase::Active;
        loop {
            let length = match self.phase {
                AttackPhase::WindUp => self.spec.wind_up,
                AttackPhase::Active => self.spec.active,
                AttackPhase::Recovery => self.spec.recovery,
            };
            if self.elapsed < length {
                return AttackAdvance {
                    step: AttackStep::Continue,
                    was_active,
                };
            }
            self.elapsed -= length;
            match self.phase {
                AttackPhase::WindUp => {
                    self.phase = AttackPhase::Active;
                    was_active = true;
                }
                AttackPhase::Active => self.phase = AttackPhase::Recovery,
                AttackPhase::Recovery => {
                    self.elapsed = 0.0;
                    return AttackAdvance {
                        step: AttackStep::Finished,
                        was_active,
                    };
                }
            }
        }
    }
}

/// A knockback slide: covers a distance in a time from where the target stood, slowing evenly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Knockback {
    start_x: f32,
    direction: f32,
    spec: KnockbackSpec,
    elapsed: f32,
}

impl Knockback {
    /// A slide from `start_x` in `direction` (−1 or 1), or `None` when the spec has no distance.
    pub fn start(start_x: f32, direction: f32, spec: KnockbackSpec) -> Option<Self> {
        (spec.distance > 0.0).then_some(Self {
            start_x,
            direction,
            spec,
            elapsed: 0.0,
        })
    }

    /// Moves the slide on by `delta` seconds and returns the target's new x.
    pub fn advance(&mut self, delta: f32) -> f32 {
        self.elapsed += delta;
        let offset = combat::knockback_offset(self.spec.distance, self.spec.time, self.elapsed);
        self.start_x + self.direction * offset
    }

    /// Whether the slide has covered its whole distance.
    pub fn is_done(&self) -> bool {
        self.elapsed >= self.spec.time
    }
}

/// A timer counting down to 0: hit-stun, invulnerability, dying and the respawn delay.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Countdown {
    remaining: f32,
}

impl Countdown {
    /// A countdown of `seconds`; 0 has already run out.
    pub fn new(seconds: f32) -> Self {
        Self {
            remaining: seconds.max(0.0),
        }
    }

    /// Whether time is left.
    pub fn is_running(&self) -> bool {
        self.remaining > 0.0
    }

    /// Counts `delta` seconds down and says whether the countdown has run out (true on the tick it
    /// reaches 0, and on every tick after).
    pub fn tick(&mut self, delta: f32) -> bool {
        self.remaining = (self.remaining - delta).max(0.0);
        !self.is_running()
    }
}

/// Which side a fighter is on. Attackers only hit the other team, so grunts never hit grunts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    /// Heroes.
    Players,
    /// Grunts and training dummies.
    Enemies,
}

/// Godot group of every [`Team::Players`] fighter.
pub const PLAYERS_GROUP: &str = "team_players";
/// Godot group of every [`Team::Enemies`] fighter that can be hit.
pub const ENEMIES_GROUP: &str = "team_enemies";

impl Team {
    /// The team this one attacks.
    pub fn other(self) -> Self {
        match self {
            Self::Players => Self::Enemies,
            Self::Enemies => Self::Players,
        }
    }

    /// The Godot group this team's fighters join.
    pub fn group(self) -> &'static str {
        match self {
            Self::Players => PLAYERS_GROUP,
            Self::Enemies => ENEMIES_GROUP,
        }
    }
}

/// A fighter whose attack is in its active part, as the hit rule sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attacker {
    /// The attacker's team.
    pub team: Team,
    /// Its feet position.
    pub origin: (f32, f32),
    /// The way it faces.
    pub facing: Facing,
    /// Half its body width: where its attack box starts.
    pub half_width: f32,
    /// The attack's values.
    pub spec: AttackSpec,
    /// The largest feet-height difference at which a blow still lands.
    pub depth_tolerance: f32,
}

impl Attacker {
    /// The area the attack reaches.
    pub fn attack_box(&self) -> Rect {
        combat::attack_box(
            self.origin,
            self.facing,
            self.half_width,
            self.spec.reach,
            self.spec.height,
        )
    }
}

/// A fighter that might be hit, as the hit rule sees it. `T` identifies it (an instance id).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target<T> {
    /// Who it is.
    pub id: T,
    /// Its team.
    pub team: Team,
    /// Its hurtbox.
    pub hurtbox: Rect,
    /// Its feet height.
    pub feet_y: f32,
    /// False once it is dead or dying.
    pub alive: bool,
    /// True while blows cannot land on it.
    pub invulnerable: bool,
}

/// One blow landing on one target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hit<T> {
    /// Who was hit.
    pub target: T,
    /// Health points it loses.
    pub damage: f32,
    /// The push direction: −1 left, 1 right (the attacker's facing).
    pub direction: f32,
    /// How far and for how long it slides.
    pub knockback: KnockbackSpec,
}

/// The one hit rule for every attacker: the blow lands on each target of the other team that is
/// alive, not invulnerable, not hit by this attack yet, overlapped by the attack box (touching
/// counts) and within the depth tolerance.
pub fn resolve_attack<T: Copy + PartialEq>(
    attacker: &Attacker,
    targets: &[Target<T>],
    already_hit: &[T],
) -> Vec<Hit<T>> {
    let reach = attacker.attack_box();
    let open: Vec<(T, Rect, f32)> = targets
        .iter()
        .filter(|t| t.team == attacker.team.other() && t.alive && !t.invulnerable)
        .map(|t| (t.id, t.hurtbox, t.feet_y))
        .collect();
    combat::new_hits(
        &reach,
        attacker.origin.1,
        attacker.depth_tolerance,
        &open,
        already_hit,
    )
    .into_iter()
    .map(|target| Hit {
        target,
        damage: attacker.spec.damage,
        direction: attacker.facing.sign(),
        knockback: attacker.spec.knockback,
    })
    .collect()
}

/// What an approaching fighter needs to know to walk up to its target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Approach {
    /// The approaching fighter's attack reach.
    pub reach: f32,
    /// Half the approaching fighter's body width.
    pub own_half_width: f32,
    /// Half the target's body width.
    pub target_half_width: f32,
    /// The approaching fighter's walking speeds.
    pub speeds: Speeds,
    /// The largest feet-height difference at which a blow still lands.
    pub depth_tolerance: f32,
}

/// The side of the target a fighter at `x` is on: 1 for its right (also when level), −1 for its
/// left.
pub fn side_of(x: f32, target_x: f32) -> f32 {
    if x < target_x { -1.0 } else { 1.0 }
}

/// One tick of walking towards a target: first only up or down until the depth difference is
/// within the tolerance, then towards the slot where the attack box just touches the target's
/// hurtbox, on the side the fighter is already on. Never overshoots.
pub fn approach_step(
    position: (f32, f32),
    target: (f32, f32),
    approach: &Approach,
    delta: f32,
) -> (f32, f32) {
    let side = side_of(position.0, target.0);
    let slot_x =
        target.0 + side * (approach.target_half_width + approach.reach + approach.own_half_width);
    let y = towards(position.1, target.1, approach.speeds.vertical * delta);
    let lined_up = (position.1 - target.1).abs() <= approach.depth_tolerance;
    let x = if lined_up {
        towards(position.0, slot_x, approach.speeds.horizontal * delta)
    } else {
        position.0
    };
    (x, y)
}

/// `from` moved towards `to` by at most `step`.
fn towards(from: f32, to: f32, step: f32) -> f32 {
    if (to - from).abs() <= step {
        to
    } else {
        from + step * (to - from).signum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Values deliberately different from the starting values in tuning.json.
    const TICK: f32 = 1.0 / 60.0;
    const TOLERANCE: f32 = 12.0;

    fn spec(wind_up: f32, active: f32, recovery: f32) -> AttackSpec {
        AttackSpec {
            wind_up,
            active,
            recovery,
            reach: 50.0,
            height: 40.0,
            damage: 9.0,
            knockback: KnockbackSpec {
                distance: 45.0,
                time: 0.3,
            },
        }
    }

    // --- Health ---

    #[test]
    fn the_killing_blow_is_reported_exactly_once() {
        let mut health = Health::new(25.0);
        assert_eq!(health.damage(10.0), Damage::Survived);
        assert_eq!(health.current(), 15.0);
        assert_eq!(health.damage(15.0), Damage::Killed);
        assert_eq!(health.damage(5.0), Damage::AlreadyDead);
        assert_eq!(health.damage(5.0), Damage::AlreadyDead);
    }

    #[test]
    fn overkill_clamps_at_zero() {
        let mut health = Health::new(25.0);
        assert_eq!(health.damage(100.0), Damage::Killed);
        assert_eq!(health.current(), 0.0);
        assert!(health.is_dead());
    }

    #[test]
    fn refill_and_a_new_maximum() {
        let mut health = Health::new(40.0);
        health.damage(30.0);
        health.refill();
        assert_eq!(health.current(), 40.0);
        health.set_max(20.0);
        assert_eq!((health.current(), health.max()), (20.0, 20.0));
        health.set_max(60.0);
        assert_eq!((health.current(), health.max()), (20.0, 60.0));
    }

    // --- AttackTimer (M2b's swing rules) ---

    /// Runs a whole attack tick by tick, returning the phase seen after each tick.
    fn run(spec: AttackSpec) -> Vec<AttackPhase> {
        let mut timer = AttackTimer::new(spec);
        let mut seen = Vec::new();
        for _ in 0..1000 {
            let advance = timer.advance(TICK);
            if advance.step == AttackStep::Finished {
                return seen;
            }
            seen.push(timer.phase());
        }
        panic!("the attack never ended");
    }

    #[test]
    fn phases_advance_on_time() {
        let seen = run(spec(0.05, 0.1, 0.2));
        let count = |wanted: AttackPhase| seen.iter().filter(|p| **p == wanted).count();
        // 0.05 s, 0.1 s and 0.2 s at 60 ticks per second, give or take a tick at each boundary.
        assert!((2..=3).contains(&count(AttackPhase::WindUp)));
        assert!((5..=7).contains(&count(AttackPhase::Active)));
        assert!((11..=13).contains(&count(AttackPhase::Recovery)));
    }

    #[test]
    fn zero_length_wind_up_and_recovery_are_passed_in_one_tick() {
        let mut timer = AttackTimer::new(spec(0.0, 0.1, 0.0));
        let first = timer.advance(TICK);
        assert_eq!(timer.phase(), AttackPhase::Active);
        assert!(first.was_active);
        assert!(!run(spec(0.0, 0.1, 0.0)).contains(&AttackPhase::WindUp));
    }

    #[test]
    fn a_very_short_active_part_is_still_reported() {
        let mut timer = AttackTimer::new(spec(0.0, 0.001, 0.2));
        assert!(timer.advance(TICK).was_active);
        assert_eq!(timer.phase(), AttackPhase::Recovery);
    }

    // --- Knockback and Countdown ---

    #[test]
    fn knockback_covers_its_distance_in_its_time() {
        let mut slide =
            Knockback::start(100.0, -1.0, spec(0.0, 0.1, 0.0).knockback).expect("has a distance");
        assert_eq!(slide.advance(0.0), 100.0);
        assert_eq!(slide.advance(0.3), 55.0);
        assert!(slide.is_done());
        let none = KnockbackSpec {
            distance: 0.0,
            time: 0.0,
        };
        assert!(Knockback::start(100.0, 1.0, none).is_none());
    }

    #[test]
    fn countdown_runs_out_on_time() {
        let mut countdown = Countdown::new(0.05);
        assert!(countdown.is_running());
        assert!(!countdown.tick(0.03));
        assert!(countdown.tick(0.03));
        assert!(!countdown.is_running());
        assert!(countdown.tick(0.03));
        assert!(Countdown::new(0.0).tick(0.0));
    }

    // --- Teams and the hit rule ---

    fn attacker(team: Team) -> Attacker {
        Attacker {
            team,
            origin: (100.0, 500.0),
            facing: Facing::Right,
            half_width: 20.0,
            spec: spec(0.1, 0.1, 0.1),
            depth_tolerance: TOLERANCE,
        }
    }

    fn target(id: u64, team: Team, x: f32, y: f32) -> Target<u64> {
        Target {
            id,
            team,
            hurtbox: combat::hurtbox((x, y), 30.0, 100.0),
            feet_y: y,
            alive: true,
            invulnerable: false,
        }
    }

    #[test]
    fn a_hit_carries_the_attackers_damage_direction_and_knockback() {
        let hits = resolve_attack(
            &attacker(Team::Players),
            &[target(1, Team::Enemies, 150.0, 500.0)],
            &[],
        );
        assert_eq!(
            hits,
            vec![Hit {
                target: 1,
                damage: 9.0,
                direction: 1.0,
                knockback: KnockbackSpec {
                    distance: 45.0,
                    time: 0.3
                },
            }]
        );
    }

    #[test]
    fn teams_never_hit_themselves() {
        let same = [target(1, Team::Enemies, 150.0, 500.0)];
        assert!(resolve_attack(&attacker(Team::Enemies), &same, &[]).is_empty());
        let heroes = [target(2, Team::Players, 150.0, 500.0)];
        assert!(resolve_attack(&attacker(Team::Players), &heroes, &[]).is_empty());
        assert_eq!(
            resolve_attack(&attacker(Team::Enemies), &heroes, &[]).len(),
            1
        );
    }

    #[test]
    fn invulnerable_dead_and_already_hit_targets_are_ignored() {
        let mut blinking = target(1, Team::Enemies, 150.0, 500.0);
        blinking.invulnerable = true;
        let mut dead = target(2, Team::Enemies, 150.0, 500.0);
        dead.alive = false;
        let fresh = target(3, Team::Enemies, 150.0, 500.0);
        let hit = target(4, Team::Enemies, 150.0, 500.0);
        let hits = resolve_attack(
            &attacker(Team::Players),
            &[blinking, dead, fresh, hit],
            &[4],
        );
        assert_eq!(hits.iter().map(|h| h.target).collect::<Vec<_>>(), vec![3]);
    }

    #[test]
    fn two_targets_in_reach_are_both_hit_and_depth_still_counts() {
        let targets = [
            target(1, Team::Enemies, 140.0, 500.0),
            target(2, Team::Enemies, 160.0, 500.0 + TOLERANCE),
            target(3, Team::Enemies, 150.0, 500.0 + TOLERANCE + 1.0),
        ];
        let hits = resolve_attack(&attacker(Team::Players), &targets, &[]);
        assert_eq!(
            hits.iter().map(|h| h.target).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    // --- Approaching ---

    const SPEEDS: Speeds = Speeds {
        horizontal: 120.0,
        vertical: 60.0,
    };

    fn approach() -> Approach {
        Approach {
            reach: 50.0,
            own_half_width: 22.0,
            target_half_width: 18.0,
            speeds: SPEEDS,
            depth_tolerance: TOLERANCE,
        }
    }

    /// Walks until the position stops changing, returning every position on the way.
    fn walk(from: (f32, f32), target: (f32, f32)) -> Vec<(f32, f32)> {
        let mut path = vec![from];
        for _ in 0..2000 {
            let last = *path.last().expect("not empty");
            let next = approach_step(last, target, &approach(), TICK);
            if next == last {
                return path;
            }
            path.push(next);
        }
        panic!("never arrived");
    }

    #[test]
    fn approach_lines_up_in_depth_before_closing_in() {
        let target = (500.0, 600.0);
        let path = walk((900.0, 450.0), target);
        for (x, y) in &path {
            if (y - target.1).abs() > TOLERANCE {
                assert_eq!(
                    *x, 900.0,
                    "moved sideways while still out of depth at y {y}"
                );
            }
        }
        // Slot on the right: 500 + 18 + 50 + 22.
        assert_eq!(*path.last().expect("not empty"), (590.0, 600.0));
    }

    #[test]
    fn approach_stops_at_the_slot_on_the_left_side_too() {
        let path = walk((100.0, 605.0), (500.0, 600.0));
        assert_eq!(*path.last().expect("not empty"), (410.0, 600.0));
        // Never past the slot.
        assert!(path.iter().all(|(x, _)| *x <= 410.0));
    }

    #[test]
    fn level_with_the_target_counts_as_its_right_side() {
        assert_eq!(side_of(500.0, 500.0), 1.0);
        assert_eq!(side_of(499.0, 500.0), -1.0);
        assert_eq!(side_of(501.0, 500.0), 1.0);
    }
}
