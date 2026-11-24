//! Loading and checking the project-wide tuning data file.
//!
//! Every tunable value lives in [`TUNING_PATH`], never in code (constitution, Principle IV).
//! A missing or invalid value is an error, never a silent fallback. Reading the file
//! ([`read_raw_tuning`]) is kept apart from checking it ([`validate`]), so the rules are plain
//! functions tested without the engine. The current values are held by
//! [`TuningStore`](crate::tuning_store::TuningStore). The file format is described in
//! `specs/006-m4a-waves-and-brute/contracts/tuning-data.md`.
//!
//! The hero and every enemy kind use the same key names, each in its own section with a nested
//! `attack` object, and one reader handles them all, so their values cannot drift apart. The wave
//! list is the one value that is not a fixed set of numbers, so it is checked into its own
//! [`WaveTuning`]; everything else stays in the `Copy` [`TuningValues`] that fighters read every
//! tick.

use std::fmt;

use godot::classes::Json;
use godot::classes::file_access::ModeFlags;
use godot::global::Error as GodotError;
use godot::prelude::*;
use godot::tools::GFile;

use crate::enemy::EnemyKind;

/// Godot path of the tuning data file. This is a file location, not a tuning value.
pub const TUNING_PATH: &str = "res://data/tuning.json";

/// Section holding the hero: its movement, health, hurtbox, stun, invulnerability and attack.
const HERO_SECTION: &str = "hero";
/// Section holding one section per enemy kind.
const ENEMIES_SECTION: &str = "enemies";
/// Section holding the training dummy's hurtbox.
const DUMMY_SECTION: &str = "dummy";
/// Section holding the lane's limits and the depth tolerance shared by every attacker.
const LANE_SECTION: &str = "lane";
/// Section holding the hit effects shared by every blow: freeze, flash and shake.
const EFFECTS_SECTION: &str = "effects";
/// Section holding the run's pacing: the death fade.
const RUN_SECTION: &str = "run";
/// Section holding the wave list and how waves grow, enter and pause.
const WAVES_SECTION: &str = "waves";
/// Section holding how a crowd of enemies takes turns and keeps apart.
const CROWD_SECTION: &str = "crowd";

/// Key of the attack object nested in each fighter's section.
const ATTACK_KEY: &str = "attack";

/// Keys inside a fighter's section (the hero's or an enemy kind's).
const HORIZONTAL_SPEED_KEY: &str = "horizontal_speed";
const VERTICAL_SPEED_KEY: &str = "vertical_speed";
const MAX_HEALTH_KEY: &str = "max_health";
const HURTBOX_WIDTH_KEY: &str = "hurtbox_width";
const HURTBOX_HEIGHT_KEY: &str = "hurtbox_height";
const HIT_STUN_TIME_KEY: &str = "hit_stun_time";
const INVULNERABILITY_TIME_KEY: &str = "invulnerability_time";

/// Keys only an enemy kind's section has.
const KNOCKBACK_RESISTANCE_KEY: &str = "knockback_resistance";
const INTERRUPTIBLE_KEY: &str = "interruptible";
const ATTACK_TOKEN_COST_KEY: &str = "attack_token_cost";
const POINTS_KEY: &str = "points";

/// Keys inside a fighter's `attack` object.
const WIND_UP_TIME_KEY: &str = "wind_up_time";
const ACTIVE_TIME_KEY: &str = "active_time";
const RECOVERY_TIME_KEY: &str = "recovery_time";
const REACH_KEY: &str = "reach";
const ATTACK_HEIGHT_KEY: &str = "height";
const DAMAGE_KEY: &str = "damage";
const KNOCKBACK_DISTANCE_KEY: &str = "knockback_distance";
const KNOCKBACK_TIME_KEY: &str = "knockback_time";

/// Keys inside [`LANE_SECTION`].
const LANE_TOP_KEY: &str = "top";
const LANE_BOTTOM_KEY: &str = "bottom";
const DEPTH_TOLERANCE_KEY: &str = "depth_tolerance";

/// Keys inside [`EFFECTS_SECTION`].
const HIT_STOP_TIME_KEY: &str = "hit_stop_time";
const FLASH_TIME_KEY: &str = "flash_time";
const SHAKE_STRENGTH_KEY: &str = "shake_strength";
const SHAKE_TIME_KEY: &str = "shake_time";

/// Keys inside [`RUN_SECTION`].
const DEATH_FADE_TIME_KEY: &str = "death_fade_time";

/// Keys inside [`WAVES_SECTION`] and its list entries.
const LIST_KEY: &str = "list";
const GRUNTS_KEY: &str = "grunts";
const BRUTES_KEY: &str = "brutes";
const EXTRA_GRUNTS_PER_WAVE_KEY: &str = "extra_grunts_per_wave";
const BRUTE_EVERY_WAVES_KEY: &str = "brute_every_waves";
const SPAWN_INTERVAL_KEY: &str = "spawn_interval";
const BREATHER_TIME_KEY: &str = "breather_time";
const BANNER_TIME_KEY: &str = "banner_time";
const MAX_ON_SCREEN_KEY: &str = "max_on_screen";

/// Keys inside [`CROWD_SECTION`].
const MAX_ATTACK_TOKENS_KEY: &str = "max_attack_tokens";
const WAITING_DISTANCE_KEY: &str = "waiting_distance";
const DRIFT_SPEED_KEY: &str = "drift_speed";
const SEPARATION_RADIUS_KEY: &str = "separation_radius";
const MAX_WAIT_TIME_KEY: &str = "max_wait_time";

/// Why the tuning data could not be used. Every message names the file and the problem, and
/// the field path where there is one, so the owner can fix the file without reading code.
#[derive(Debug, Clone, PartialEq)]
pub enum TuningError {
    /// The file is missing or cannot be read.
    Unreadable { path: String, reason: String },
    /// The file is not valid JSON.
    InvalidJson {
        path: String,
        line: i32,
        message: String,
    },
    /// A required section or field is absent.
    MissingField { path: String, field: String },
    /// A section or field is present but has an unusable value.
    InvalidValue {
        path: String,
        field: String,
        value: String,
        reason: String,
    },
}

impl fmt::Display for TuningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, reason } => write!(f, "cannot read {path}: {reason}"),
            Self::InvalidJson {
                path,
                line,
                message,
            } => {
                write!(f, "{path} is not valid JSON (line {line}): {message}")
            }
            Self::MissingField { path, field } => write!(f, "{path}: missing field `{field}`"),
            Self::InvalidValue {
                path,
                field,
                value,
                reason,
            } => {
                write!(
                    f,
                    "{path}: field `{field}` has invalid value {value} ({reason})"
                )
            }
        }
    }
}

impl TuningError {
    /// Where the problem is: the dotted field path (`lane.top`), or the file path when the whole
    /// file is unusable. Shown to the owner next to [`reason`](Self::reason).
    pub fn field(&self) -> &str {
        match self {
            Self::Unreadable { path, .. } | Self::InvalidJson { path, .. } => path,
            Self::MissingField { field, .. } | Self::InvalidValue { field, .. } => field,
        }
    }

    /// Why the value or file was rejected, including the bad value where there is one, without
    /// repeating the file or field.
    pub fn reason(&self) -> String {
        match self {
            Self::Unreadable { reason, .. } => format!("cannot be read: {reason}"),
            Self::InvalidJson { line, message, .. } => {
                format!("not valid JSON (line {line}): {message}")
            }
            Self::MissingField { .. } => "missing".to_string(),
            Self::InvalidValue { value, reason, .. } => format!("{reason}, got {value}"),
        }
    }
}

impl std::error::Error for TuningError {}

/// One fighter's `attack` object as read from the file, before any rule is applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawAttack {
    /// `wind_up_time`, in seconds.
    pub wind_up_time: f64,
    /// `active_time`, in seconds.
    pub active_time: f64,
    /// `recovery_time`, in seconds.
    pub recovery_time: f64,
    /// `reach`, in pixels in front of the body.
    pub reach: f64,
    /// `height`, in pixels above the feet.
    pub height: f64,
    /// `damage`, in health points.
    pub damage: f64,
    /// `knockback_distance`, in pixels.
    pub knockback_distance: f64,
    /// `knockback_time`, in seconds.
    pub knockback_time: f64,
}

/// The keys the hero and every enemy kind share, as read from the file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawFighter {
    /// `horizontal_speed`, in pixels per second.
    pub horizontal_speed: f64,
    /// `vertical_speed`, in pixels per second.
    pub vertical_speed: f64,
    /// `max_health`, in health points.
    pub max_health: f64,
    /// `hurtbox_width`, in pixels.
    pub hurtbox_width: f64,
    /// `hurtbox_height`, in pixels above the feet.
    pub hurtbox_height: f64,
    /// `hit_stun_time`, in seconds.
    pub hit_stun_time: f64,
    /// The nested `attack` object.
    pub attack: RawAttack,
}

/// A value that must be `true` or `false`, as read from the file.
#[derive(Debug, Clone, PartialEq)]
pub enum RawFlag {
    /// A JSON boolean.
    Bool(bool),
    /// Anything else, as it reads in the file, so the rejection can show it.
    NotBool(String),
}

/// One enemy kind's section (`enemies.grunt` or `enemies.brute`), as read from the file.
#[derive(Debug, Clone, PartialEq)]
pub struct RawEnemy {
    /// The keys shared with the hero.
    pub fighter: RawFighter,
    /// `knockback_resistance`, a fraction.
    pub knockback_resistance: f64,
    /// `interruptible`.
    pub interruptible: RawFlag,
    /// `attack_token_cost`, in tokens.
    pub attack_token_cost: f64,
    /// `points`, added to the score per kill.
    pub points: f64,
}

/// One entry of `waves.list`, as read from the file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawWave {
    /// `grunts`.
    pub grunts: f64,
    /// `brutes`.
    pub brutes: f64,
}

/// `waves.list` as read from the file.
#[derive(Debug, Clone, PartialEq)]
pub enum RawList {
    /// A JSON array of wave objects.
    Entries(Vec<RawWave>),
    /// Anything else, as it reads in the file, so the rejection can show it.
    NotArray(String),
}

/// The `waves` section, as read from the file.
#[derive(Debug, Clone, PartialEq)]
pub struct RawWaves {
    /// `list`.
    pub list: RawList,
    /// `extra_grunts_per_wave`.
    pub extra_grunts_per_wave: f64,
    /// `brute_every_waves`.
    pub brute_every_waves: f64,
    /// `spawn_interval`, in seconds.
    pub spawn_interval: f64,
    /// `breather_time`, in seconds.
    pub breather_time: f64,
    /// `banner_time`, in seconds.
    pub banner_time: f64,
    /// `max_on_screen`.
    pub max_on_screen: f64,
}

/// The `crowd` section, as read from the file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawCrowd {
    /// `max_attack_tokens`.
    pub max_attack_tokens: f64,
    /// `waiting_distance`, in pixels.
    pub waiting_distance: f64,
    /// `drift_speed`, in pixels per second.
    pub drift_speed: f64,
    /// `separation_radius`, in pixels.
    pub separation_radius: f64,
    /// `max_wait_time`, in seconds.
    pub max_wait_time: f64,
}

/// Every tuning value as read from the file, before any rule is applied. Only [`validate`]
/// turns them into values the game may use.
#[derive(Debug, Clone, PartialEq)]
pub struct RawTuning {
    /// The `hero` section, without `invulnerability_time`.
    pub hero: RawFighter,
    /// `hero.invulnerability_time`, in seconds.
    pub invulnerability_time: f64,
    /// The `enemies.grunt` section.
    pub grunt: RawEnemy,
    /// The `enemies.brute` section.
    pub brute: RawEnemy,
    /// `dummy.hurtbox_width`, in pixels.
    pub dummy_hurtbox_width: f64,
    /// `dummy.hurtbox_height`, in pixels above the feet.
    pub dummy_hurtbox_height: f64,
    /// `lane.top`, in pixels from the top of the screen.
    pub lane_top: f64,
    /// `lane.bottom`, in pixels from the top of the screen.
    pub lane_bottom: f64,
    /// `lane.depth_tolerance`, in pixels of feet height.
    pub depth_tolerance: f64,
    /// `effects.hit_stop_time`, in seconds.
    pub hit_stop_time: f64,
    /// `effects.flash_time`, in seconds.
    pub flash_time: f64,
    /// `effects.shake_strength`, in pixels.
    pub shake_strength: f64,
    /// `effects.shake_time`, in seconds.
    pub shake_time: f64,
    /// `run.death_fade_time`, in seconds.
    pub death_fade_time: f64,
    /// The `waves` section.
    pub waves: RawWaves,
    /// The `crowd` section.
    pub crowd: RawCrowd,
}

/// Declares a small `Copy` group of checked `f32` tuning values with one accessor per value.
/// Instances are only built by [`validate`].
macro_rules! value_group {
    ($(#[$doc:meta])* $name:ident { $($(#[$field_doc:meta])* $field:ident,)* }) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name {
            $($field: f32,)*
        }

        impl $name {
            $(
                $(#[$field_doc])*
                pub fn $field(&self) -> f32 {
                    self.$field
                }
            )*
        }
    };
}

value_group! {
    /// One fighter's attack (the `attack` object of `hero` or an enemy kind).
    AttackTuning {
        /// Seconds from the start of the attack until the blow can land.
        wind_up_time,
        /// Seconds during which the blow can land; always above 0.
        active_time,
        /// Seconds after the active part before the attacker is free again.
        recovery_time,
        /// How far the attack reaches in front of the body, in pixels; always above 0.
        reach,
        /// How tall the attack box is above the feet, in pixels.
        height,
        /// Health points a landed blow takes; always above 0.
        damage,
        /// How far a hit target slides, in pixels; 0 turns knockback off.
        knockback_distance,
        /// How long the slide takes, in seconds; above 0 whenever there is a distance.
        knockback_time,
    }
}

value_group! {
    /// The hit effects shared by every blow (`effects` section). A value of 0 turns its effect off.
    EffectsTuning {
        /// How long play freezes on impact, in seconds.
        hit_stop_time,
        /// How long the one hit flashes white, in seconds.
        flash_time,
        /// How far the screen shakes at most, in pixels.
        shake_strength,
        /// How long the screen shakes, in seconds.
        shake_time,
    }
}

value_group! {
    /// The training dummy's hurtbox (`dummy` section).
    DummyTuning {
        /// Width of the hurtbox, centred on the feet, in pixels; always above 0.
        hurtbox_width,
        /// Height of the hurtbox above the feet, in pixels; always above 0.
        hurtbox_height,
    }
}

value_group! {
    /// The run's pacing (`run` section).
    RunTuning {
        /// Seconds a dying enemy takes to fade out before it is removed.
        death_fade_time,
    }
}

/// The hero's checked values (the `hero` section, without invulnerability).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FighterTuning {
    horizontal_speed: f32,
    vertical_speed: f32,
    max_health: f32,
    hurtbox_width: f32,
    hurtbox_height: f32,
    hit_stun_time: f32,
    attack: AttackTuning,
}

impl FighterTuning {
    /// Speed along the lane, in pixels per second; always above 0.
    pub fn horizontal_speed(&self) -> f32 {
        self.horizontal_speed
    }

    /// Speed into and out of the lane, in pixels per second; always above 0.
    pub fn vertical_speed(&self) -> f32 {
        self.vertical_speed
    }

    /// Health when full, in health points; always above 0.
    pub fn max_health(&self) -> f32 {
        self.max_health
    }

    /// Width of the hurtbox, centred on the feet, in pixels; always above 0.
    pub fn hurtbox_width(&self) -> f32 {
        self.hurtbox_width
    }

    /// Height of the hurtbox above the feet, in pixels; always above 0.
    pub fn hurtbox_height(&self) -> f32 {
        self.hurtbox_height
    }

    /// Seconds a landed blow stops this fighter from moving or attacking.
    pub fn hit_stun_time(&self) -> f32 {
        self.hit_stun_time
    }

    /// This fighter's attack.
    pub fn attack(&self) -> AttackTuning {
        self.attack
    }
}

/// One enemy kind's checked values (`enemies.grunt` or `enemies.brute`). Both kinds have exactly
/// the same fields, so behaviour code never needs to know which kind it is reading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnemyTuning {
    fighter: FighterTuning,
    knockback_resistance: f32,
    interruptible: bool,
    attack_token_cost: u32,
    points: i64,
}

impl EnemyTuning {
    /// Speed along the lane, in pixels per second; always above 0.
    pub fn horizontal_speed(&self) -> f32 {
        self.fighter.horizontal_speed
    }

    /// Speed into and out of the lane, in pixels per second; always above 0.
    pub fn vertical_speed(&self) -> f32 {
        self.fighter.vertical_speed
    }

    /// Health when full, in health points; always above 0.
    pub fn max_health(&self) -> f32 {
        self.fighter.max_health
    }

    /// Width of the hurtbox, centred on the feet, in pixels; always above 0.
    pub fn hurtbox_width(&self) -> f32 {
        self.fighter.hurtbox_width
    }

    /// Height of the hurtbox above the feet, in pixels; always above 0.
    pub fn hurtbox_height(&self) -> f32 {
        self.fighter.hurtbox_height
    }

    /// Seconds a landed blow stuns this enemy, before its resistance is applied.
    pub fn hit_stun_time(&self) -> f32 {
        self.fighter.hit_stun_time
    }

    /// How much of a blow's push and stun it shrugs off, from 0 (none) to 1 (all).
    pub fn knockback_resistance(&self) -> f32 {
        self.knockback_resistance
    }

    /// Whether a blow during its attack stops the attack; false means its attacks are armored.
    pub fn interruptible(&self) -> bool {
        self.interruptible
    }

    /// Attack tokens it must hold to close in and attack; between 1 and the pool size.
    pub fn attack_token_cost(&self) -> u32 {
        self.attack_token_cost
    }

    /// Points its kill adds to the score; always above 0.
    pub fn points(&self) -> i64 {
        self.points
    }

    /// Its attack.
    pub fn attack(&self) -> AttackTuning {
        self.fighter.attack
    }
}

/// How a crowd takes turns and keeps apart (`crowd` section).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrowdTuning {
    max_attack_tokens: u32,
    waiting_distance: f32,
    drift_speed: f32,
    separation_radius: f32,
    max_wait_time: f32,
}

impl CrowdTuning {
    /// Size of the attack-token pool; at least 1.
    pub fn max_attack_tokens(&self) -> u32 {
        self.max_attack_tokens
    }

    /// How far from its target a waiting enemy's spot is, in pixels; always above 0.
    pub fn waiting_distance(&self) -> f32 {
        self.waiting_distance
    }

    /// How fast a waiting enemy drifts to its spot, in pixels per second; 0 holds it still.
    pub fn drift_speed(&self) -> f32 {
        self.drift_speed
    }

    /// How close two enemies may come before they are pushed apart, in pixels; 0 turns it off.
    pub fn separation_radius(&self) -> f32 {
        self.separation_radius
    }

    /// Seconds a waiting enemy may be passed over for nearer ones before it goes first; 0 means
    /// turns go to the longest waiting.
    pub fn max_wait_time(&self) -> f32 {
        self.max_wait_time
    }
}

/// Every checked tuning value but the wave list. It can only be built by [`validate`], so an
/// instance always holds values that passed every rule. It is `Copy`, so fighters read it every
/// tick without cloning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TuningValues {
    hero: FighterTuning,
    invulnerability_time: f32,
    grunt: EnemyTuning,
    brute: EnemyTuning,
    dummy: DummyTuning,
    lane_top: f32,
    lane_bottom: f32,
    depth_tolerance: f32,
    effects: EffectsTuning,
    run: RunTuning,
    crowd: CrowdTuning,
}

impl TuningValues {
    /// The hero's values.
    pub fn hero(&self) -> FighterTuning {
        self.hero
    }

    /// Seconds the hero cannot be hit after a blow lands on it.
    pub fn invulnerability_time(&self) -> f32 {
        self.invulnerability_time
    }

    /// The values of one enemy kind. This is the one place that chooses values by kind.
    pub fn enemy(&self, kind: EnemyKind) -> EnemyTuning {
        match kind {
            EnemyKind::Grunt => self.grunt,
            EnemyKind::Brute => self.brute,
        }
    }

    /// The training dummy's hurtbox.
    pub fn dummy(&self) -> DummyTuning {
        self.dummy
    }

    /// The highest y (furthest into the lane) a character's feet may reach.
    pub fn lane_top(&self) -> f32 {
        self.lane_top
    }

    /// The lowest y (nearest the player) a character's feet may reach.
    pub fn lane_bottom(&self) -> f32 {
        self.lane_bottom
    }

    /// The largest difference in feet height at which a blow still lands, for every attacker.
    pub fn depth_tolerance(&self) -> f32 {
        self.depth_tolerance
    }

    /// The hit effects shared by every blow.
    pub fn effects(&self) -> EffectsTuning {
        self.effects
    }

    /// The run's pacing.
    pub fn run(&self) -> RunTuning {
        self.run
    }

    /// How a crowd takes turns and keeps apart.
    pub fn crowd(&self) -> CrowdTuning {
        self.crowd
    }
}

/// How many grunts and brutes one wave brings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaveSpec {
    /// Grunts in the wave.
    pub grunts: u32,
    /// Brutes in the wave.
    pub brutes: u32,
}

impl WaveSpec {
    /// Enemies in the wave.
    pub fn total(&self) -> u32 {
        self.grunts + self.brutes
    }
}

/// The checked `waves` section. Built only by [`validate`]; the list always holds at least one
/// wave and every wave at least one enemy.
#[derive(Debug, Clone, PartialEq)]
pub struct WaveTuning {
    list: Vec<WaveSpec>,
    extra_grunts_per_wave: u32,
    brute_every_waves: u32,
    spawn_interval: f32,
    breather_time: f32,
    banner_time: f32,
    max_on_screen: u32,
}

impl WaveTuning {
    /// Builds wave values directly, for unit tests elsewhere in the crate; the game only gets
    /// them from [`validate`].
    #[cfg(test)]
    pub fn for_tests(
        list: Vec<WaveSpec>,
        extra_grunts_per_wave: u32,
        brute_every_waves: u32,
        timing: (f32, f32, f32),
        max_on_screen: u32,
    ) -> Self {
        let (spawn_interval, breather_time, banner_time) = timing;
        Self {
            list,
            extra_grunts_per_wave,
            brute_every_waves,
            spawn_interval,
            breather_time,
            banner_time,
            max_on_screen,
        }
    }

    /// The written waves, wave 1 first; never empty.
    pub fn list(&self) -> &[WaveSpec] {
        &self.list
    }

    /// Grunts each wave past the list adds.
    pub fn extra_grunts_per_wave(&self) -> u32 {
        self.extra_grunts_per_wave
    }

    /// Every how many waves past the list one more brute comes; 0 means no more brutes.
    pub fn brute_every_waves(&self) -> u32 {
        self.brute_every_waves
    }

    /// Seconds between two enemies of a wave entering.
    pub fn spawn_interval(&self) -> f32 {
        self.spawn_interval
    }

    /// Seconds of quiet between a wave's last death and the next banner.
    pub fn breather_time(&self) -> f32 {
        self.breather_time
    }

    /// Seconds a wave's banner shows before its enemies start entering.
    pub fn banner_time(&self) -> f32 {
        self.banner_time
    }

    /// The most enemies in play at once; at least 1.
    pub fn max_on_screen(&self) -> u32 {
        self.max_on_screen
    }
}

/// A combination of values that is allowed but probably not meant. The values are still used;
/// the warning is shown so a slip is not silent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TuningWarning {
    /// An enemy kind's wind-up is not longer than the hero's, so its attack may not be seen
    /// coming.
    EnemyWindUpNotLonger {
        kind: EnemyKind,
        hero: f32,
        enemy: f32,
    },
    /// The brute's wind-up is not longer than the grunt's, so it does not telegraph its heavier
    /// blow any better.
    BruteWindUpNotLonger { grunt: f32, brute: f32 },
}

impl fmt::Display for TuningWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnemyWindUpNotLonger { kind, hero, enemy } => write!(
                f,
                "{}: not longer than {} ({hero}), got {enemy}; the {}'s attack may not be seen \
                 coming",
                attack_path(&enemy_section(*kind), WIND_UP_TIME_KEY),
                attack_path(HERO_SECTION, WIND_UP_TIME_KEY),
                kind_key(*kind),
            ),
            Self::BruteWindUpNotLonger { grunt, brute } => write!(
                f,
                "{}: not longer than {} ({grunt}), got {brute}; the brute should telegraph longer \
                 than the grunt",
                attack_path(&enemy_section(EnemyKind::Brute), WIND_UP_TIME_KEY),
                attack_path(&enemy_section(EnemyKind::Grunt), WIND_UP_TIME_KEY),
            ),
        }
    }
}

/// The warnings that apply to values that already passed [`validate`], in a fixed order: each
/// enemy kind against the hero, then the brute against the grunt.
pub fn warnings(values: &TuningValues) -> Vec<TuningWarning> {
    let hero = values.hero.attack.wind_up_time;
    let grunt = values.grunt.fighter.attack.wind_up_time;
    let brute = values.brute.fighter.attack.wind_up_time;
    let mut found = Vec::new();
    for (kind, enemy) in [(EnemyKind::Grunt, grunt), (EnemyKind::Brute, brute)] {
        if enemy <= hero {
            found.push(TuningWarning::EnemyWindUpNotLonger { kind, hero, enemy });
        }
    }
    if brute <= grunt {
        found.push(TuningWarning::BruteWindUpNotLonger { grunt, brute });
    }
    found
}

/// Applies every tuning rule to `raw` and returns the values only if all of them pass, so a
/// caller can never use a mix of good and bad values.
///
/// Start-up and reload both call this, so the same rules apply to both. Each value is converted
/// to `f32` first, so a huge finite number that overflows is rejected too. The rules are listed in
/// `specs/006-m4a-waves-and-brute/contracts/tuning-data.md`; they are applied in file order, except
/// that each enemy's token cost is compared with the pool once the `crowd` section has passed.
pub fn validate(
    raw: &RawTuning,
    screen_height: f32,
) -> Result<(TuningValues, WaveTuning), TuningError> {
    let hero = fighter(&raw.hero, HERO_SECTION)?;
    let invulnerability_time = non_negative(
        raw.invulnerability_time,
        HERO_SECTION,
        INVULNERABILITY_TIME_KEY,
    )?;
    let grunt = enemy(&raw.grunt, EnemyKind::Grunt)?;
    let brute = enemy(&raw.brute, EnemyKind::Brute)?;
    let dummy = DummyTuning {
        hurtbox_width: positive(raw.dummy_hurtbox_width, DUMMY_SECTION, HURTBOX_WIDTH_KEY)?,
        hurtbox_height: positive(raw.dummy_hurtbox_height, DUMMY_SECTION, HURTBOX_HEIGHT_KEY)?,
    };
    let bottom = checked(
        raw.lane_bottom,
        LANE_SECTION,
        LANE_BOTTOM_KEY,
        "must be a finite number",
        |_| true,
    )?;
    let top = checked(
        raw.lane_top,
        LANE_SECTION,
        LANE_TOP_KEY,
        "must be a finite number >= 0",
        |v| v >= 0.0,
    )?;
    if top >= bottom {
        return Err(invalid_value(
            &field_path(LANE_SECTION, LANE_TOP_KEY),
            raw.lane_top.to_string(),
            format!(
                "must be less than {} ({})",
                field_path(LANE_SECTION, LANE_BOTTOM_KEY),
                raw.lane_bottom
            ),
        ));
    }
    if bottom > screen_height {
        return Err(invalid_value(
            &field_path(LANE_SECTION, LANE_BOTTOM_KEY),
            raw.lane_bottom.to_string(),
            format!("must be at most the screen height ({screen_height})"),
        ));
    }
    let depth_tolerance = non_negative(raw.depth_tolerance, LANE_SECTION, DEPTH_TOLERANCE_KEY)?;
    let effects = EffectsTuning {
        hit_stop_time: non_negative(raw.hit_stop_time, EFFECTS_SECTION, HIT_STOP_TIME_KEY)?,
        flash_time: non_negative(raw.flash_time, EFFECTS_SECTION, FLASH_TIME_KEY)?,
        shake_strength: non_negative(raw.shake_strength, EFFECTS_SECTION, SHAKE_STRENGTH_KEY)?,
        shake_time: non_negative(raw.shake_time, EFFECTS_SECTION, SHAKE_TIME_KEY)?,
    };
    let run = RunTuning {
        death_fade_time: non_negative(raw.death_fade_time, RUN_SECTION, DEATH_FADE_TIME_KEY)?,
    };
    let waves = waves(&raw.waves)?;
    let crowd = crowd(&raw.crowd)?;
    for (kind, values, raw_enemy) in [
        (EnemyKind::Grunt, &grunt, &raw.grunt),
        (EnemyKind::Brute, &brute, &raw.brute),
    ] {
        if values.attack_token_cost > crowd.max_attack_tokens {
            return Err(invalid_value(
                &field_path(&enemy_section(kind), ATTACK_TOKEN_COST_KEY),
                raw_enemy.attack_token_cost.to_string(),
                format!(
                    "must be at most {} ({})",
                    field_path(CROWD_SECTION, MAX_ATTACK_TOKENS_KEY),
                    crowd.max_attack_tokens
                ),
            ));
        }
    }
    let values = TuningValues {
        hero,
        invulnerability_time,
        grunt,
        brute,
        dummy,
        lane_top: top,
        lane_bottom: bottom,
        depth_tolerance,
        effects,
        run,
        crowd,
    };
    Ok((values, waves))
}

/// Checks the keys every fighter has; `section` is its dotted path, used in field paths.
fn fighter(raw: &RawFighter, section: &str) -> Result<FighterTuning, TuningError> {
    Ok(FighterTuning {
        horizontal_speed: positive(raw.horizontal_speed, section, HORIZONTAL_SPEED_KEY)?,
        vertical_speed: positive(raw.vertical_speed, section, VERTICAL_SPEED_KEY)?,
        max_health: positive(raw.max_health, section, MAX_HEALTH_KEY)?,
        hurtbox_width: positive(raw.hurtbox_width, section, HURTBOX_WIDTH_KEY)?,
        hurtbox_height: positive(raw.hurtbox_height, section, HURTBOX_HEIGHT_KEY)?,
        hit_stun_time: non_negative(raw.hit_stun_time, section, HIT_STUN_TIME_KEY)?,
        attack: attack(&raw.attack, &field_path(section, ATTACK_KEY))?,
    })
}

/// Checks one enemy kind's section, except its token cost against the pool (see [`validate`]).
fn enemy(raw: &RawEnemy, kind: EnemyKind) -> Result<EnemyTuning, TuningError> {
    let section = enemy_section(kind);
    let fighter = fighter(&raw.fighter, &section)?;
    let knockback_resistance = checked(
        raw.knockback_resistance,
        &section,
        KNOCKBACK_RESISTANCE_KEY,
        "must be a finite number from 0 to 1",
        |v| (0.0..=1.0).contains(&v),
    )?;
    let interruptible = match &raw.interruptible {
        RawFlag::Bool(value) => *value,
        RawFlag::NotBool(shown) => {
            return Err(invalid_value(
                &field_path(&section, INTERRUPTIBLE_KEY),
                shown.clone(),
                "must be true or false".to_string(),
            ));
        }
    };
    let attack_token_cost = whole(raw.attack_token_cost, &section, ATTACK_TOKEN_COST_KEY, 1)?;
    let points = whole(raw.points, &section, POINTS_KEY, 1)?;
    Ok(EnemyTuning {
        fighter,
        knockback_resistance,
        interruptible,
        attack_token_cost,
        points: i64::from(points),
    })
}

/// Checks one `attack` object; `path` is its dotted path, such as `enemies.grunt.attack`.
fn attack(raw: &RawAttack, path: &str) -> Result<AttackTuning, TuningError> {
    let wind_up_time = non_negative(raw.wind_up_time, path, WIND_UP_TIME_KEY)?;
    let active_time = positive(raw.active_time, path, ACTIVE_TIME_KEY)?;
    let recovery_time = non_negative(raw.recovery_time, path, RECOVERY_TIME_KEY)?;
    let reach = positive(raw.reach, path, REACH_KEY)?;
    let height = non_negative(raw.height, path, ATTACK_HEIGHT_KEY)?;
    let damage = positive(raw.damage, path, DAMAGE_KEY)?;
    let knockback_distance = non_negative(raw.knockback_distance, path, KNOCKBACK_DISTANCE_KEY)?;
    let knockback_time = non_negative(raw.knockback_time, path, KNOCKBACK_TIME_KEY)?;
    if knockback_distance > 0.0 && knockback_time == 0.0 {
        return Err(invalid_value(
            &field_path(path, KNOCKBACK_TIME_KEY),
            raw.knockback_time.to_string(),
            format!(
                "must be greater than 0 while {} is {}",
                field_path(path, KNOCKBACK_DISTANCE_KEY),
                raw.knockback_distance
            ),
        ));
    }
    Ok(AttackTuning {
        wind_up_time,
        active_time,
        recovery_time,
        reach,
        height,
        damage,
        knockback_distance,
        knockback_time,
    })
}

/// Checks the `waves` section.
fn waves(raw: &RawWaves) -> Result<WaveTuning, TuningError> {
    let list_path = field_path(WAVES_SECTION, LIST_KEY);
    let entries = match &raw.list {
        RawList::Entries(entries) => entries,
        RawList::NotArray(shown) => {
            return Err(invalid_value(
                &list_path,
                shown.clone(),
                "must be an array of waves".to_string(),
            ));
        }
    };
    if entries.is_empty() {
        return Err(invalid_value(
            &list_path,
            "[]".to_string(),
            "must hold at least one wave".to_string(),
        ));
    }
    let mut list = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let path = entry_path(index);
        let spec = WaveSpec {
            grunts: whole(entry.grunts, &path, GRUNTS_KEY, 0)?,
            brutes: whole(entry.brutes, &path, BRUTES_KEY, 0)?,
        };
        if spec.total() == 0 {
            return Err(invalid_value(
                &path,
                "0 grunts and 0 brutes".to_string(),
                "must hold at least one enemy".to_string(),
            ));
        }
        list.push(spec);
    }
    let extra_grunts_per_wave = whole(
        raw.extra_grunts_per_wave,
        WAVES_SECTION,
        EXTRA_GRUNTS_PER_WAVE_KEY,
        0,
    )?;
    let brute_every_waves = whole(
        raw.brute_every_waves,
        WAVES_SECTION,
        BRUTE_EVERY_WAVES_KEY,
        0,
    )?;
    if extra_grunts_per_wave == 0 && brute_every_waves == 0 {
        return Err(invalid_value(
            &field_path(WAVES_SECTION, EXTRA_GRUNTS_PER_WAVE_KEY),
            "0".to_string(),
            format!(
                "must be above 0 while {} is 0, so waves keep growing",
                field_path(WAVES_SECTION, BRUTE_EVERY_WAVES_KEY)
            ),
        ));
    }
    Ok(WaveTuning {
        list,
        extra_grunts_per_wave,
        brute_every_waves,
        spawn_interval: non_negative(raw.spawn_interval, WAVES_SECTION, SPAWN_INTERVAL_KEY)?,
        breather_time: non_negative(raw.breather_time, WAVES_SECTION, BREATHER_TIME_KEY)?,
        banner_time: non_negative(raw.banner_time, WAVES_SECTION, BANNER_TIME_KEY)?,
        max_on_screen: whole(raw.max_on_screen, WAVES_SECTION, MAX_ON_SCREEN_KEY, 1)?,
    })
}

/// Checks the `crowd` section.
fn crowd(raw: &RawCrowd) -> Result<CrowdTuning, TuningError> {
    Ok(CrowdTuning {
        max_attack_tokens: whole(
            raw.max_attack_tokens,
            CROWD_SECTION,
            MAX_ATTACK_TOKENS_KEY,
            1,
        )?,
        waiting_distance: positive(raw.waiting_distance, CROWD_SECTION, WAITING_DISTANCE_KEY)?,
        drift_speed: non_negative(raw.drift_speed, CROWD_SECTION, DRIFT_SPEED_KEY)?,
        separation_radius: non_negative(
            raw.separation_radius,
            CROWD_SECTION,
            SEPARATION_RADIUS_KEY,
        )?,
        max_wait_time: non_negative(raw.max_wait_time, CROWD_SECTION, MAX_WAIT_TIME_KEY)?,
    })
}

/// Accepts a finite value `>= 0`, naming `section.key` otherwise.
fn non_negative(raw: f64, section: &str, key: &str) -> Result<f32, TuningError> {
    checked(raw, section, key, "must be a finite number >= 0", |v| {
        v >= 0.0
    })
}

/// Accepts a finite value `> 0`, naming `section.key` otherwise.
fn positive(raw: f64, section: &str, key: &str) -> Result<f32, TuningError> {
    checked(raw, section, key, "must be a finite number > 0", |v| {
        v > 0.0
    })
}

/// Accepts a whole number from `min` up to `u32::MAX`, naming `section.key` otherwise.
fn whole(raw: f64, section: &str, key: &str, min: u32) -> Result<u32, TuningError> {
    let fits = raw.is_finite() && raw.fract() == 0.0 && raw >= f64::from(min);
    if fits && raw <= f64::from(u32::MAX) {
        Ok(raw as u32)
    } else {
        Err(invalid_value(
            &field_path(section, key),
            raw.to_string(),
            format!("must be a whole number >= {min}"),
        ))
    }
}

/// The key of an enemy kind's section inside `enemies`.
fn kind_key(kind: EnemyKind) -> &'static str {
    match kind {
        EnemyKind::Grunt => "grunt",
        EnemyKind::Brute => "brute",
    }
}

/// The dotted path of an enemy kind's section, such as `enemies.brute`.
fn enemy_section(kind: EnemyKind) -> String {
    field_path(ENEMIES_SECTION, kind_key(kind))
}

/// The path of one entry of `waves.list`, counted from 0 as in the file: `waves.list[2]`.
fn entry_path(index: usize) -> String {
    format!("{}[{index}]", field_path(WAVES_SECTION, LIST_KEY))
}

/// The dotted path of a key in a fighter's `attack` object, such as `enemies.grunt.attack.reach`.
fn attack_path(section: &str, key: &str) -> String {
    field_path(&field_path(section, ATTACK_KEY), key)
}

/// Reads [`TUNING_PATH`] and returns its values unchecked; [`validate`] checks them.
///
/// Every call rereads the file from disk, so edits made while the game runs are picked up.
/// Unknown keys are ignored, so sections can be added before the code that reads them.
pub fn read_raw_tuning() -> Result<RawTuning, TuningError> {
    let root = read_json_object()?;
    let hero = object_field(&root, HERO_SECTION, HERO_SECTION)?;
    let enemies = object_field(&root, ENEMIES_SECTION, ENEMIES_SECTION)?;
    let dummy = object_field(&root, DUMMY_SECTION, DUMMY_SECTION)?;
    let lane = object_field(&root, LANE_SECTION, LANE_SECTION)?;
    let effects = object_field(&root, EFFECTS_SECTION, EFFECTS_SECTION)?;
    let run = object_field(&root, RUN_SECTION, RUN_SECTION)?;
    let waves = object_field(&root, WAVES_SECTION, WAVES_SECTION)?;
    let crowd = object_field(&root, CROWD_SECTION, CROWD_SECTION)?;
    Ok(RawTuning {
        hero: read_fighter(&hero, HERO_SECTION)?,
        invulnerability_time: number_in(&hero, HERO_SECTION, INVULNERABILITY_TIME_KEY)?,
        grunt: read_enemy(&enemies, EnemyKind::Grunt)?,
        brute: read_enemy(&enemies, EnemyKind::Brute)?,
        dummy_hurtbox_width: number_in(&dummy, DUMMY_SECTION, HURTBOX_WIDTH_KEY)?,
        dummy_hurtbox_height: number_in(&dummy, DUMMY_SECTION, HURTBOX_HEIGHT_KEY)?,
        lane_top: number_in(&lane, LANE_SECTION, LANE_TOP_KEY)?,
        lane_bottom: number_in(&lane, LANE_SECTION, LANE_BOTTOM_KEY)?,
        depth_tolerance: number_in(&lane, LANE_SECTION, DEPTH_TOLERANCE_KEY)?,
        hit_stop_time: number_in(&effects, EFFECTS_SECTION, HIT_STOP_TIME_KEY)?,
        flash_time: number_in(&effects, EFFECTS_SECTION, FLASH_TIME_KEY)?,
        shake_strength: number_in(&effects, EFFECTS_SECTION, SHAKE_STRENGTH_KEY)?,
        shake_time: number_in(&effects, EFFECTS_SECTION, SHAKE_TIME_KEY)?,
        death_fade_time: number_in(&run, RUN_SECTION, DEATH_FADE_TIME_KEY)?,
        waves: read_waves(&waves)?,
        crowd: RawCrowd {
            max_attack_tokens: number_in(&crowd, CROWD_SECTION, MAX_ATTACK_TOKENS_KEY)?,
            waiting_distance: number_in(&crowd, CROWD_SECTION, WAITING_DISTANCE_KEY)?,
            drift_speed: number_in(&crowd, CROWD_SECTION, DRIFT_SPEED_KEY)?,
            separation_radius: number_in(&crowd, CROWD_SECTION, SEPARATION_RADIUS_KEY)?,
            max_wait_time: number_in(&crowd, CROWD_SECTION, MAX_WAIT_TIME_KEY)?,
        },
    })
}

/// Reads the keys every fighter has, including its nested `attack` object.
fn read_fighter(object: &VarDictionary, section: &str) -> Result<RawFighter, TuningError> {
    let attack_section = field_path(section, ATTACK_KEY);
    let attack = object_field(object, ATTACK_KEY, &attack_section)?;
    Ok(RawFighter {
        horizontal_speed: number_in(object, section, HORIZONTAL_SPEED_KEY)?,
        vertical_speed: number_in(object, section, VERTICAL_SPEED_KEY)?,
        max_health: number_in(object, section, MAX_HEALTH_KEY)?,
        hurtbox_width: number_in(object, section, HURTBOX_WIDTH_KEY)?,
        hurtbox_height: number_in(object, section, HURTBOX_HEIGHT_KEY)?,
        hit_stun_time: number_in(object, section, HIT_STUN_TIME_KEY)?,
        attack: RawAttack {
            wind_up_time: number_in(&attack, &attack_section, WIND_UP_TIME_KEY)?,
            active_time: number_in(&attack, &attack_section, ACTIVE_TIME_KEY)?,
            recovery_time: number_in(&attack, &attack_section, RECOVERY_TIME_KEY)?,
            reach: number_in(&attack, &attack_section, REACH_KEY)?,
            height: number_in(&attack, &attack_section, ATTACK_HEIGHT_KEY)?,
            damage: number_in(&attack, &attack_section, DAMAGE_KEY)?,
            knockback_distance: number_in(&attack, &attack_section, KNOCKBACK_DISTANCE_KEY)?,
            knockback_time: number_in(&attack, &attack_section, KNOCKBACK_TIME_KEY)?,
        },
    })
}

/// Reads one enemy kind's section from the `enemies` object. One reader serves every kind.
fn read_enemy(enemies: &VarDictionary, kind: EnemyKind) -> Result<RawEnemy, TuningError> {
    let section = enemy_section(kind);
    let object = object_field(enemies, kind_key(kind), &section)?;
    Ok(RawEnemy {
        fighter: read_fighter(&object, &section)?,
        knockback_resistance: number_in(&object, &section, KNOCKBACK_RESISTANCE_KEY)?,
        interruptible: bool_in(&object, &section, INTERRUPTIBLE_KEY)?,
        attack_token_cost: number_in(&object, &section, ATTACK_TOKEN_COST_KEY)?,
        points: number_in(&object, &section, POINTS_KEY)?,
    })
}

/// Reads the `waves` section, including its list.
fn read_waves(object: &VarDictionary) -> Result<RawWaves, TuningError> {
    Ok(RawWaves {
        list: list_in(object)?,
        extra_grunts_per_wave: number_in(object, WAVES_SECTION, EXTRA_GRUNTS_PER_WAVE_KEY)?,
        brute_every_waves: number_in(object, WAVES_SECTION, BRUTE_EVERY_WAVES_KEY)?,
        spawn_interval: number_in(object, WAVES_SECTION, SPAWN_INTERVAL_KEY)?,
        breather_time: number_in(object, WAVES_SECTION, BREATHER_TIME_KEY)?,
        banner_time: number_in(object, WAVES_SECTION, BANNER_TIME_KEY)?,
        max_on_screen: number_in(object, WAVES_SECTION, MAX_ON_SCREEN_KEY)?,
    })
}

/// Reads `waves.list`: an array of `{ grunts, brutes }` objects. Anything that is not an array is
/// kept as text for [`validate`] to reject; an entry that is not an object is rejected here.
fn list_in(waves: &VarDictionary) -> Result<RawList, TuningError> {
    let path = field_path(WAVES_SECTION, LIST_KEY);
    let value = required(waves, LIST_KEY, &path)?;
    let Ok(array) = value.try_to::<VarArray>() else {
        return Ok(RawList::NotArray(value.to_string()));
    };
    let mut entries = Vec::with_capacity(array.len());
    for (index, item) in array.iter_shared().enumerate() {
        let entry_path = entry_path(index);
        let object = item.try_to::<VarDictionary>().map_err(|_| {
            invalid_value(
                &entry_path,
                item.to_string(),
                "must be a JSON object".into(),
            )
        })?;
        entries.push(RawWave {
            grunts: number_in(&object, &entry_path, GRUNTS_KEY)?,
            brutes: number_in(&object, &entry_path, BRUTES_KEY)?,
        });
    }
    Ok(RawList::Entries(entries))
}

/// Reads the number at `section.key` from the already-read `section` object.
fn number_in(object: &VarDictionary, section: &str, key: &str) -> Result<f64, TuningError> {
    number_field(object, key, &field_path(section, key))
}

/// Reads the value at `section.key` as a flag; anything but a boolean is kept as text for
/// [`validate`] to reject.
fn bool_in(object: &VarDictionary, section: &str, key: &str) -> Result<RawFlag, TuningError> {
    let value = required(object, key, &field_path(section, key))?;
    Ok(match value.get_type() {
        VariantType::BOOL => RawFlag::Bool(value.to::<bool>()),
        _ => RawFlag::NotBool(value.to_string()),
    })
}

/// Converts `raw` to `f32` and accepts it only if it is finite and passes `rule`; otherwise
/// returns an error naming `section.key`, the raw value and `requirement`.
fn checked(
    raw: f64,
    section: &str,
    key: &str,
    requirement: &str,
    rule: impl Fn(f32) -> bool,
) -> Result<f32, TuningError> {
    let value = raw as f32;
    if value.is_finite() && rule(value) {
        Ok(value)
    } else {
        Err(invalid_value(
            &field_path(section, key),
            raw.to_string(),
            requirement.to_string(),
        ))
    }
}

/// Reads and parses the tuning file, requiring a JSON object at the top level.
fn read_json_object() -> Result<VarDictionary, TuningError> {
    let unreadable = |e: std::io::Error| TuningError::Unreadable {
        path: TUNING_PATH.to_string(),
        reason: e.to_string(),
    };
    let mut file = GFile::open(TUNING_PATH, ModeFlags::READ).map_err(unreadable)?;
    let text = file.read_as_gstring_entire().map_err(unreadable)?;

    let mut json = Json::new_gd();
    if json.parse(&text) != GodotError::OK {
        return Err(TuningError::InvalidJson {
            path: TUNING_PATH.to_string(),
            // Godot counts lines from 0; editors, and the owner, count from 1.
            line: json.get_error_line() + 1,
            message: json.get_error_message().to_string(),
        });
    }
    let data = json.get_data();
    data.try_to::<VarDictionary>().map_err(|_| {
        invalid_value(
            "(top level)",
            data.to_string(),
            "must be a JSON object".into(),
        )
    })
}

/// Returns the JSON object stored under `key`; `field` is its full path, used in errors.
fn object_field(
    parent: &VarDictionary,
    key: &str,
    field: &str,
) -> Result<VarDictionary, TuningError> {
    let value = required(parent, key, field)?;
    value
        .try_to::<VarDictionary>()
        .map_err(|_| invalid_value(field, value.to_string(), "must be a JSON object".into()))
}

/// Returns the JSON number stored under `key`; `field` is its full path, used in errors.
fn number_field(parent: &VarDictionary, key: &str, field: &str) -> Result<f64, TuningError> {
    let value = required(parent, key, field)?;
    // Godot's JSON parser yields FLOAT for every number; INT is accepted in case that changes.
    match value.get_type() {
        VariantType::FLOAT => value.try_to::<f64>(),
        VariantType::INT => value.try_to::<i64>().map(|n| n as f64),
        other => {
            return Err(invalid_value(
                field,
                format!("{value:?}"),
                format!(
                    "must be a number, not {}",
                    format!("{other:?}").to_lowercase()
                ),
            ));
        }
    }
    .map_err(|e| invalid_value(field, value.to_string(), e.to_string()))
}

/// Looks up `key`, turning absence into a [`TuningError::MissingField`] naming `field`.
fn required(parent: &VarDictionary, key: &str, field: &str) -> Result<Variant, TuningError> {
    parent
        .get(&key.to_variant())
        .ok_or_else(|| TuningError::MissingField {
            path: TUNING_PATH.to_string(),
            field: field.to_string(),
        })
}

fn invalid_value(field: &str, value: String, reason: String) -> TuningError {
    TuningError::InvalidValue {
        path: TUNING_PATH.to_string(),
        field: field.to_string(),
        value,
        reason,
    }
}

/// The dotted path of a field, as written in error messages (for example `hero.vertical_speed`).
fn field_path(section: &str, key: &str) -> String {
    format!("{section}.{key}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Valid values, deliberately different from the starting values in tuning.json.
    const SCREEN: f32 = 720.0;
    const TOP: f64 = 400.0;
    const BOTTOM: f64 = 650.0;
    const POOL: f64 = 3.0;
    const NOT_FINITE: [f64; 3] = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];

    fn raw_attack(wind_up_time: f64) -> RawAttack {
        RawAttack {
            wind_up_time,
            active_time: 0.12,
            recovery_time: 0.2,
            reach: 80.0,
            height: 50.0,
            damage: 7.0,
            knockback_distance: 50.0,
            knockback_time: 0.25,
        }
    }

    fn raw_enemy(health: f64, wind_up: f64, resistance: f64, cost: f64, points: f64) -> RawEnemy {
        RawEnemy {
            fighter: RawFighter {
                horizontal_speed: 120.0,
                vertical_speed: 70.0,
                max_health: health,
                hurtbox_width: 44.0,
                hurtbox_height: 110.0,
                hit_stun_time: 0.35,
                attack: RawAttack {
                    damage: 12.0,
                    knockback_distance: 70.0,
                    knockback_time: 0.3,
                    ..raw_attack(wind_up)
                },
            },
            knockback_resistance: resistance,
            interruptible: RawFlag::Bool(resistance == 0.0),
            attack_token_cost: cost,
            points,
        }
    }

    fn raw() -> RawTuning {
        RawTuning {
            hero: RawFighter {
                horizontal_speed: 250.0,
                vertical_speed: 150.0,
                max_health: 80.0,
                hurtbox_width: 40.0,
                hurtbox_height: 90.0,
                hit_stun_time: 0.2,
                attack: raw_attack(0.05),
            },
            invulnerability_time: 0.8,
            grunt: raw_enemy(25.0, 0.5, 0.0, 1.0, 15.0),
            brute: raw_enemy(90.0, 0.8, 0.6, 3.0, 40.0),
            dummy_hurtbox_width: 38.0,
            dummy_hurtbox_height: 100.0,
            lane_top: TOP,
            lane_bottom: BOTTOM,
            depth_tolerance: 15.0,
            hit_stop_time: 0.05,
            flash_time: 0.1,
            shake_strength: 4.0,
            shake_time: 0.1,
            death_fade_time: 0.4,
            waves: RawWaves {
                list: RawList::Entries(vec![
                    RawWave {
                        grunts: 1.0,
                        brutes: 0.0,
                    },
                    RawWave {
                        grunts: 2.0,
                        brutes: 1.0,
                    },
                    RawWave {
                        grunts: 0.0,
                        brutes: 2.0,
                    },
                ]),
                extra_grunts_per_wave: 2.0,
                brute_every_waves: 4.0,
                spawn_interval: 0.5,
                breather_time: 1.5,
                banner_time: 1.0,
                max_on_screen: 8.0,
            },
            crowd: RawCrowd {
                max_attack_tokens: POOL,
                waiting_distance: 150.0,
                drift_speed: 30.0,
                separation_radius: 48.0,
                max_wait_time: 3.5,
            },
        }
    }

    fn rejected(raw: RawTuning) -> TuningError {
        validate(&raw, SCREEN).expect_err("should be rejected")
    }

    fn accepted(raw: &RawTuning) -> (TuningValues, WaveTuning) {
        validate(raw, SCREEN).expect("should be accepted")
    }

    /// Sets one number of a [`RawTuning`].
    type Setter = fn(&mut RawTuning, f64);

    /// Every field that must be finite and `> 0`, with its path and a setter.
    fn positive_fields() -> Vec<(&'static str, Setter)> {
        vec![
            ("hero.horizontal_speed", |r, v| r.hero.horizontal_speed = v),
            ("hero.vertical_speed", |r, v| r.hero.vertical_speed = v),
            ("hero.max_health", |r, v| r.hero.max_health = v),
            ("hero.hurtbox_width", |r, v| r.hero.hurtbox_width = v),
            ("hero.hurtbox_height", |r, v| r.hero.hurtbox_height = v),
            ("hero.attack.active_time", |r, v| {
                r.hero.attack.active_time = v
            }),
            ("hero.attack.reach", |r, v| r.hero.attack.reach = v),
            ("hero.attack.damage", |r, v| r.hero.attack.damage = v),
            ("enemies.grunt.horizontal_speed", |r, v| {
                r.grunt.fighter.horizontal_speed = v
            }),
            ("enemies.grunt.vertical_speed", |r, v| {
                r.grunt.fighter.vertical_speed = v
            }),
            ("enemies.grunt.max_health", |r, v| {
                r.grunt.fighter.max_health = v
            }),
            ("enemies.grunt.hurtbox_width", |r, v| {
                r.grunt.fighter.hurtbox_width = v
            }),
            ("enemies.grunt.hurtbox_height", |r, v| {
                r.grunt.fighter.hurtbox_height = v
            }),
            ("enemies.grunt.attack.active_time", |r, v| {
                r.grunt.fighter.attack.active_time = v
            }),
            ("enemies.grunt.attack.reach", |r, v| {
                r.grunt.fighter.attack.reach = v
            }),
            ("enemies.grunt.attack.damage", |r, v| {
                r.grunt.fighter.attack.damage = v
            }),
            ("enemies.brute.horizontal_speed", |r, v| {
                r.brute.fighter.horizontal_speed = v
            }),
            ("enemies.brute.max_health", |r, v| {
                r.brute.fighter.max_health = v
            }),
            ("enemies.brute.attack.damage", |r, v| {
                r.brute.fighter.attack.damage = v
            }),
            ("dummy.hurtbox_width", |r, v| r.dummy_hurtbox_width = v),
            ("dummy.hurtbox_height", |r, v| r.dummy_hurtbox_height = v),
            ("crowd.waiting_distance", |r, v| {
                r.crowd.waiting_distance = v
            }),
        ]
    }

    /// Every field that only has to be finite and `>= 0`, with its path and a setter.
    fn non_negative_fields() -> Vec<(&'static str, Setter)> {
        vec![
            ("hero.hit_stun_time", |r, v| r.hero.hit_stun_time = v),
            ("hero.invulnerability_time", |r, v| {
                r.invulnerability_time = v
            }),
            ("hero.attack.wind_up_time", |r, v| {
                r.hero.attack.wind_up_time = v
            }),
            ("hero.attack.recovery_time", |r, v| {
                r.hero.attack.recovery_time = v
            }),
            ("hero.attack.height", |r, v| r.hero.attack.height = v),
            ("hero.attack.knockback_distance", |r, v| {
                r.hero.attack.knockback_distance = v
            }),
            ("enemies.grunt.hit_stun_time", |r, v| {
                r.grunt.fighter.hit_stun_time = v
            }),
            ("enemies.grunt.attack.wind_up_time", |r, v| {
                r.grunt.fighter.attack.wind_up_time = v
            }),
            ("enemies.grunt.attack.recovery_time", |r, v| {
                r.grunt.fighter.attack.recovery_time = v
            }),
            ("enemies.grunt.attack.knockback_distance", |r, v| {
                r.grunt.fighter.attack.knockback_distance = v
            }),
            ("enemies.brute.hit_stun_time", |r, v| {
                r.brute.fighter.hit_stun_time = v
            }),
            ("enemies.brute.attack.wind_up_time", |r, v| {
                r.brute.fighter.attack.wind_up_time = v
            }),
            ("lane.depth_tolerance", |r, v| r.depth_tolerance = v),
            ("effects.hit_stop_time", |r, v| r.hit_stop_time = v),
            ("effects.flash_time", |r, v| r.flash_time = v),
            ("effects.shake_strength", |r, v| r.shake_strength = v),
            ("effects.shake_time", |r, v| r.shake_time = v),
            ("run.death_fade_time", |r, v| r.death_fade_time = v),
            ("waves.spawn_interval", |r, v| r.waves.spawn_interval = v),
            ("waves.breather_time", |r, v| r.waves.breather_time = v),
            ("waves.banner_time", |r, v| r.waves.banner_time = v),
            ("crowd.drift_speed", |r, v| r.crowd.drift_speed = v),
            ("crowd.separation_radius", |r, v| {
                r.crowd.separation_radius = v
            }),
            ("crowd.max_wait_time", |r, v| r.crowd.max_wait_time = v),
        ]
    }

    /// Sets `waves.list` to `entries` as (grunts, brutes) pairs.
    fn with_list(entries: &[(f64, f64)]) -> RawTuning {
        let mut r = raw();
        r.waves.list = RawList::Entries(
            entries
                .iter()
                .map(|&(grunts, brutes)| RawWave { grunts, brutes })
                .collect(),
        );
        r
    }

    // --- Values are read into the right places. ---

    #[test]
    fn validate_accepts_valid_values_and_keeps_them_apart() {
        let (values, waves) = accepted(&raw());
        let hero = values.hero();
        assert_eq!(hero.horizontal_speed(), 250.0);
        assert_eq!(hero.max_health(), 80.0);
        assert_eq!(hero.attack().wind_up_time(), 0.05);
        assert_eq!(values.invulnerability_time(), 0.8);
        let grunt = values.enemy(EnemyKind::Grunt);
        assert_eq!(grunt.max_health(), 25.0);
        assert_eq!(grunt.hit_stun_time(), 0.35);
        assert_eq!(grunt.attack().wind_up_time(), 0.5);
        assert_eq!(grunt.attack().damage(), 12.0);
        assert_eq!(grunt.knockback_resistance(), 0.0);
        assert!(grunt.interruptible());
        assert_eq!(grunt.attack_token_cost(), 1);
        assert_eq!(grunt.points(), 15);
        let brute = values.enemy(EnemyKind::Brute);
        assert_eq!(brute.max_health(), 90.0);
        assert_eq!(brute.attack().wind_up_time(), 0.8);
        assert_eq!(brute.knockback_resistance(), 0.6);
        assert!(!brute.interruptible());
        assert_eq!(brute.attack_token_cost(), 3);
        assert_eq!(brute.points(), 40);
        assert_eq!(values.dummy().hurtbox_width(), 38.0);
        assert_eq!(values.lane_top(), 400.0);
        assert_eq!(values.depth_tolerance(), 15.0);
        assert_eq!(values.effects().shake_strength(), 4.0);
        assert_eq!(values.run().death_fade_time(), 0.4);
        let crowd = values.crowd();
        assert_eq!(crowd.max_attack_tokens(), 3);
        assert_eq!(crowd.waiting_distance(), 150.0);
        assert_eq!(crowd.drift_speed(), 30.0);
        assert_eq!(crowd.separation_radius(), 48.0);
        assert_eq!(crowd.max_wait_time(), 3.5);
        assert_eq!(
            waves.list(),
            &[
                WaveSpec {
                    grunts: 1,
                    brutes: 0
                },
                WaveSpec {
                    grunts: 2,
                    brutes: 1
                },
                WaveSpec {
                    grunts: 0,
                    brutes: 2
                },
            ]
        );
        assert_eq!(waves.extra_grunts_per_wave(), 2);
        assert_eq!(waves.brute_every_waves(), 4);
        assert_eq!(waves.spawn_interval(), 0.5);
        assert_eq!(waves.breather_time(), 1.5);
        assert_eq!(waves.banner_time(), 1.0);
        assert_eq!(waves.max_on_screen(), 8);
    }

    // --- One passing and one failing case per rule. ---

    #[test]
    fn validate_positive_fields_must_be_above_zero_and_finite() {
        for (path, set) in positive_fields() {
            let mut ok = raw();
            set(&mut ok, 1.0);
            assert!(validate(&ok, SCREEN).is_ok(), "{path} = 1 should pass");
            for bad in [0.0, -5.0, 1e300].into_iter().chain(NOT_FINITE) {
                let mut wrong = raw();
                set(&mut wrong, bad);
                assert_eq!(rejected(wrong).field(), path, "{path} = {bad}");
            }
        }
    }

    #[test]
    fn validate_other_fields_may_be_zero_but_not_negative_or_not_finite() {
        for (path, set) in non_negative_fields() {
            let mut ok = raw();
            set(&mut ok, 0.0);
            // A zero knockback distance allows a zero time; the time stays valid here anyway.
            assert!(validate(&ok, SCREEN).is_ok(), "{path} = 0 should pass");
            for bad in [-1.0, f64::NAN] {
                let mut wrong = raw();
                set(&mut wrong, bad);
                assert_eq!(rejected(wrong).field(), path, "{path} = {bad}");
            }
        }
    }

    #[test]
    fn validate_knockback_time_must_be_above_zero_while_there_is_a_distance() {
        let mut off = raw();
        off.hero.attack.knockback_distance = 0.0;
        off.hero.attack.knockback_time = 0.0;
        off.brute.fighter.attack.knockback_distance = 0.0;
        off.brute.fighter.attack.knockback_time = 0.0;
        assert!(validate(&off, SCREEN).is_ok());
        for section in ["hero", "enemies.grunt", "enemies.brute"] {
            let mut wrong = raw();
            match section {
                "hero" => wrong.hero.attack.knockback_time = 0.0,
                "enemies.grunt" => wrong.grunt.fighter.attack.knockback_time = 0.0,
                _ => wrong.brute.fighter.attack.knockback_time = 0.0,
            }
            let error = rejected(wrong);
            assert_eq!(error.field(), format!("{section}.attack.knockback_time"));
            let reason = error.reason();
            assert!(
                reason.contains(&format!("{section}.attack.knockback_distance")),
                "{reason}"
            );
        }
    }

    #[test]
    fn validate_lane_top_must_be_finite_not_negative_and_above_the_bottom() {
        let ok = RawTuning {
            lane_top: 0.0,
            ..raw()
        };
        assert!(validate(&ok, SCREEN).is_ok());
        for bad in [-1.0, f64::NAN, BOTTOM, BOTTOM + 1.0] {
            let error = rejected(RawTuning {
                lane_top: bad,
                ..raw()
            });
            assert_eq!(error.field(), "lane.top", "{bad}");
        }
        let above = rejected(RawTuning {
            lane_top: BOTTOM,
            ..raw()
        });
        assert!(above.reason().contains("lane.bottom"), "{}", above.reason());
    }

    #[test]
    fn validate_lane_bottom_must_be_finite_and_on_the_screen() {
        let ok = RawTuning {
            lane_bottom: 720.0,
            ..raw()
        };
        assert!(validate(&ok, SCREEN).is_ok());
        for bad in [721.0, f64::INFINITY] {
            let error = rejected(RawTuning {
                lane_bottom: bad,
                ..raw()
            });
            assert_eq!(error.field(), "lane.bottom", "{bad}");
        }
    }

    #[test]
    fn validate_knockback_resistance_must_be_from_zero_to_one() {
        for ok_value in [0.0, 1.0] {
            let mut ok = raw();
            ok.brute.knockback_resistance = ok_value;
            assert!(validate(&ok, SCREEN).is_ok(), "{ok_value}");
        }
        for bad in [-0.1, 1.1, f64::NAN] {
            let mut wrong = raw();
            wrong.grunt.knockback_resistance = bad;
            assert_eq!(
                rejected(wrong).field(),
                "enemies.grunt.knockback_resistance",
                "{bad}"
            );
        }
    }

    #[test]
    fn validate_interruptible_must_be_true_or_false() {
        for ok_value in [true, false] {
            let mut ok = raw();
            ok.brute.interruptible = RawFlag::Bool(ok_value);
            assert!(validate(&ok, SCREEN).is_ok());
        }
        for shown in ["1", "no"] {
            let mut wrong = raw();
            wrong.brute.interruptible = RawFlag::NotBool(shown.to_string());
            let error = rejected(wrong);
            assert_eq!(error.field(), "enemies.brute.interruptible");
            assert!(
                error.reason().contains("true or false"),
                "{}",
                error.reason()
            );
            assert!(error.reason().contains(shown), "{}", error.reason());
        }
    }

    #[test]
    fn validate_token_cost_must_be_whole_and_fit_the_pool() {
        for ok_value in [1.0, POOL] {
            let mut ok = raw();
            ok.grunt.attack_token_cost = ok_value;
            assert!(validate(&ok, SCREEN).is_ok(), "{ok_value}");
        }
        for bad in [0.0, 1.5, -1.0, POOL + 1.0] {
            let mut wrong = raw();
            wrong.brute.attack_token_cost = bad;
            assert_eq!(
                rejected(wrong).field(),
                "enemies.brute.attack_token_cost",
                "{bad}"
            );
        }
        let mut too_big = raw();
        too_big.brute.attack_token_cost = POOL + 1.0;
        let reason = rejected(too_big).reason();
        assert!(reason.contains("crowd.max_attack_tokens (3)"), "{reason}");
    }

    #[test]
    fn validate_points_must_be_whole_and_above_zero() {
        let mut ok = raw();
        ok.grunt.points = 1.0;
        assert!(validate(&ok, SCREEN).is_ok());
        for bad in [0.0, 2.5, -10.0] {
            let mut wrong = raw();
            wrong.grunt.points = bad;
            assert_eq!(rejected(wrong).field(), "enemies.grunt.points", "{bad}");
        }
    }

    #[test]
    fn validate_wave_list_must_be_a_non_empty_array() {
        let empty = with_list(&[]);
        let error = rejected(empty);
        assert_eq!(error.field(), "waves.list");
        let mut not_array = raw();
        not_array.waves.list = RawList::NotArray("5".to_string());
        assert_eq!(rejected(not_array).field(), "waves.list");
        assert!(validate(&with_list(&[(1.0, 0.0)]), SCREEN).is_ok());
    }

    #[test]
    fn validate_wave_entries_must_be_whole_and_hold_an_enemy() {
        assert!(validate(&with_list(&[(1.0, 0.0), (0.0, 1.0)]), SCREEN).is_ok());
        for bad in [-1.0, 0.5] {
            let error = rejected(with_list(&[(1.0, 0.0), (bad, 1.0)]));
            assert_eq!(error.field(), "waves.list[1].grunts", "{bad}");
        }
        let error = rejected(with_list(&[(1.0, 0.0), (2.0, 0.25)]));
        assert_eq!(error.field(), "waves.list[1].brutes");
        let error = rejected(with_list(&[(1.0, 0.0), (0.0, 0.0)]));
        assert_eq!(error.field(), "waves.list[1]");
    }

    #[test]
    fn validate_wave_growth_must_be_whole_and_not_both_zero() {
        let mut only_grunts = raw();
        only_grunts.waves.brute_every_waves = 0.0;
        assert!(validate(&only_grunts, SCREEN).is_ok());
        let mut only_brutes = raw();
        only_brutes.waves.extra_grunts_per_wave = 0.0;
        assert!(validate(&only_brutes, SCREEN).is_ok());
        let mut none = raw();
        none.waves.extra_grunts_per_wave = 0.0;
        none.waves.brute_every_waves = 0.0;
        let error = rejected(none);
        assert_eq!(error.field(), "waves.extra_grunts_per_wave");
        assert!(
            error.reason().contains("keep growing"),
            "{}",
            error.reason()
        );
        for bad in [-1.0, 1.5] {
            let mut wrong = raw();
            wrong.waves.brute_every_waves = bad;
            assert_eq!(rejected(wrong).field(), "waves.brute_every_waves", "{bad}");
        }
    }

    #[test]
    fn validate_counts_of_one_or_more_must_be_whole() {
        type CountSetter = (&'static str, Setter);
        let counts: [CountSetter; 2] = [
            ("waves.max_on_screen", |r, v| r.waves.max_on_screen = v),
            ("crowd.max_attack_tokens", |r, v| {
                r.crowd.max_attack_tokens = v;
                // Keep every cost inside the pool, so only the pool itself is judged.
                r.brute.attack_token_cost = 1.0;
            }),
        ];
        for (path, set) in counts {
            let mut ok = raw();
            set(&mut ok, 1.0);
            assert!(validate(&ok, SCREEN).is_ok(), "{path} = 1");
            for bad in [0.0, 2.5, f64::NAN] {
                let mut wrong = raw();
                set(&mut wrong, bad);
                assert_eq!(rejected(wrong).field(), path, "{path} = {bad}");
            }
        }
    }

    #[test]
    fn validate_rejection_gives_field_reason_and_full_message() {
        let mut wrong = raw();
        wrong.grunt.fighter.max_health = -3.0;
        let error = rejected(wrong);
        assert_eq!(error.field(), "enemies.grunt.max_health");
        let reason = error.reason();
        assert!(reason.contains("> 0"), "{reason}");
        assert!(reason.contains("-3"), "{reason}");
        assert!(!reason.contains("res://"), "{reason}");
        let message = error.to_string();
        assert!(message.contains("res://data/tuning.json"), "{message}");
        assert!(message.contains("enemies.grunt.max_health"), "{message}");
    }

    // --- The wind-up warnings: accepted, but flagged. ---

    #[test]
    fn warnings_flag_an_enemy_wind_up_not_longer_than_the_heros() {
        let (values, _) = accepted(&raw());
        assert!(warnings(&values).is_empty());
        for grunt_wind_up in [0.05, 0.03, 0.0] {
            let mut short = raw();
            short.grunt.fighter.attack.wind_up_time = grunt_wind_up;
            let (values, _) = accepted(&short);
            assert_eq!(
                warnings(&values),
                vec![TuningWarning::EnemyWindUpNotLonger {
                    kind: EnemyKind::Grunt,
                    hero: 0.05,
                    enemy: grunt_wind_up as f32,
                }],
                "{grunt_wind_up}"
            );
        }
    }

    #[test]
    fn warnings_flag_a_brute_wind_up_not_longer_than_the_grunts() {
        for brute_wind_up in [0.5, 0.3] {
            let mut short = raw();
            short.brute.fighter.attack.wind_up_time = brute_wind_up;
            let (values, _) = accepted(&short);
            assert_eq!(
                warnings(&values),
                vec![TuningWarning::BruteWindUpNotLonger {
                    grunt: 0.5,
                    brute: brute_wind_up as f32,
                }],
                "{brute_wind_up}"
            );
        }
        let mut both = raw();
        both.brute.fighter.attack.wind_up_time = 0.04;
        let (values, _) = accepted(&both);
        assert_eq!(warnings(&values).len(), 2, "brute below hero and grunt");
    }

    #[test]
    fn warning_messages_name_both_fields_and_values() {
        let message = TuningWarning::EnemyWindUpNotLonger {
            kind: EnemyKind::Brute,
            hero: 0.05,
            enemy: 0.03,
        }
        .to_string();
        assert!(
            message.starts_with("enemies.brute.attack.wind_up_time"),
            "{message}"
        );
        assert!(
            message.contains("hero.attack.wind_up_time (0.05)"),
            "{message}"
        );
        assert!(message.contains("got 0.03"), "{message}");
        assert!(message.contains("the brute's attack"), "{message}");
        let message = TuningWarning::BruteWindUpNotLonger {
            grunt: 0.5,
            brute: 0.4,
        }
        .to_string();
        assert!(
            message.starts_with("enemies.brute.attack.wind_up_time"),
            "{message}"
        );
        assert!(
            message.contains("enemies.grunt.attack.wind_up_time (0.5)"),
            "{message}"
        );
    }

    #[test]
    fn file_level_errors_use_the_file_as_their_field() {
        let unreadable = TuningError::Unreadable {
            path: TUNING_PATH.to_string(),
            reason: "not found".to_string(),
        };
        assert_eq!(unreadable.field(), TUNING_PATH);
        assert!(unreadable.reason().contains("not found"));
        let missing = TuningError::MissingField {
            path: TUNING_PATH.to_string(),
            field: "enemies.brute.attack".to_string(),
        };
        assert_eq!(missing.field(), "enemies.brute.attack");
        assert_eq!(missing.reason(), "missing");
    }
}
