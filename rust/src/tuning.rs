//! Loading and checking the project-wide tuning data file.
//!
//! Every tunable value lives in [`TUNING_PATH`], never in code (constitution, Principle IV).
//! A missing or invalid value is an error, never a silent fallback. Reading the file
//! ([`read_raw_tuning`]) is kept apart from checking it ([`validate`]), so the rules are plain
//! functions tested without the engine. The current values are held by
//! [`TuningStore`](crate::tuning_store::TuningStore). The file format is described in
//! `specs/005-m3-first-fight/contracts/tuning-data.md`.
//!
//! The hero and the grunt use the same key names, each in its own section with a nested `attack`
//! object, and one reader handles both, so the two fighters' values cannot drift apart.

use std::fmt;

use godot::classes::Json;
use godot::classes::file_access::ModeFlags;
use godot::global::Error as GodotError;
use godot::prelude::*;
use godot::tools::GFile;

/// Godot path of the tuning data file. This is a file location, not a tuning value.
pub const TUNING_PATH: &str = "res://data/tuning.json";

/// Section holding the hero: its movement, health, hurtbox, stun, invulnerability and attack.
const HERO_SECTION: &str = "hero";
/// Section holding the grunt, with the same keys as the hero except invulnerability.
const GRUNT_SECTION: &str = "grunt";
/// Section holding the training dummy's hurtbox.
const DUMMY_SECTION: &str = "dummy";
/// Section holding the lane's limits and the depth tolerance shared by every attacker.
const LANE_SECTION: &str = "lane";
/// Section holding the hit effects shared by every blow: freeze, flash and shake.
const EFFECTS_SECTION: &str = "effects";
/// Section holding the run's pacing: respawn delay and death fade.
const RUN_SECTION: &str = "run";

/// Key of the attack object nested in each fighter's section.
const ATTACK_KEY: &str = "attack";

/// Keys inside a fighter's section.
const HORIZONTAL_SPEED_KEY: &str = "horizontal_speed";
const VERTICAL_SPEED_KEY: &str = "vertical_speed";
const MAX_HEALTH_KEY: &str = "max_health";
const HURTBOX_WIDTH_KEY: &str = "hurtbox_width";
const HURTBOX_HEIGHT_KEY: &str = "hurtbox_height";
const HIT_STUN_TIME_KEY: &str = "hit_stun_time";
const INVULNERABILITY_TIME_KEY: &str = "invulnerability_time";

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
const RESPAWN_DELAY_KEY: &str = "respawn_delay";
const DEATH_FADE_TIME_KEY: &str = "death_fade_time";

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

/// One fighter's section as read from the file, before any rule is applied.
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

/// Every tuning number as read from the file, before any rule is applied. Only [`validate`]
/// turns them into values the game may use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawTuning {
    /// The `hero` section, without `invulnerability_time`.
    pub hero: RawFighter,
    /// `hero.invulnerability_time`, in seconds.
    pub invulnerability_time: f64,
    /// The `grunt` section.
    pub grunt: RawFighter,
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
    /// `run.respawn_delay`, in seconds.
    pub respawn_delay: f64,
    /// `run.death_fade_time`, in seconds.
    pub death_fade_time: f64,
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
    /// One fighter's attack (the `attack` object of `hero` or `grunt`).
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
        /// Seconds between the last grunt dying and a new one entering.
        respawn_delay,
        /// Seconds a dying grunt takes to fade out before it is removed.
        death_fade_time,
    }
}

/// One fighter's checked values (the `hero` or `grunt` section).
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

/// Every checked tuning value. It can only be built by [`validate`], so an instance always holds
/// values that passed every rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TuningValues {
    hero: FighterTuning,
    invulnerability_time: f32,
    grunt: FighterTuning,
    dummy: DummyTuning,
    lane_top: f32,
    lane_bottom: f32,
    depth_tolerance: f32,
    effects: EffectsTuning,
    run: RunTuning,
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

    /// The grunt's values.
    pub fn grunt(&self) -> FighterTuning {
        self.grunt
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
}

/// A combination of values that is allowed but probably not meant. The values are still used;
/// the warning is shown so a slip is not silent.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TuningWarning {
    /// The grunt's wind-up is not longer than the hero's, so its attack may not be seen coming.
    GruntWindUpNotLonger { hero: f32, grunt: f32 },
}

impl fmt::Display for TuningWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GruntWindUpNotLonger { hero, grunt } => write!(
                f,
                "{}: not longer than {} ({hero}), got {grunt}; the grunt's attack may not be seen \
                 coming",
                attack_path(GRUNT_SECTION, WIND_UP_TIME_KEY),
                attack_path(HERO_SECTION, WIND_UP_TIME_KEY),
            ),
        }
    }
}

/// The warnings that apply to values that already passed [`validate`].
pub fn warnings(values: &TuningValues) -> Vec<TuningWarning> {
    let hero = values.hero.attack.wind_up_time;
    let grunt = values.grunt.attack.wind_up_time;
    if grunt <= hero {
        vec![TuningWarning::GruntWindUpNotLonger { hero, grunt }]
    } else {
        Vec::new()
    }
}

/// Applies every tuning rule to `raw` and returns the values only if all of them pass, so a
/// caller can never use a mix of good and bad values.
///
/// Start-up and reload both call this, so the same rules apply to both. Each value is converted
/// to `f32` first, so a huge finite number that overflows is rejected too. The rules are listed in
/// `specs/005-m3-first-fight/contracts/tuning-data.md`; they are applied in file order.
pub fn validate(raw: &RawTuning, screen_height: f32) -> Result<TuningValues, TuningError> {
    let hero = fighter(&raw.hero, HERO_SECTION)?;
    let invulnerability_time = non_negative(
        raw.invulnerability_time,
        HERO_SECTION,
        INVULNERABILITY_TIME_KEY,
    )?;
    let grunt = fighter(&raw.grunt, GRUNT_SECTION)?;
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
        respawn_delay: non_negative(raw.respawn_delay, RUN_SECTION, RESPAWN_DELAY_KEY)?,
        death_fade_time: non_negative(raw.death_fade_time, RUN_SECTION, DEATH_FADE_TIME_KEY)?,
    };
    Ok(TuningValues {
        hero,
        invulnerability_time,
        grunt,
        dummy,
        lane_top: top,
        lane_bottom: bottom,
        depth_tolerance,
        effects,
        run,
    })
}

/// Checks one fighter's section; `section` is `hero` or `grunt`, used in field paths.
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

/// Checks one `attack` object; `path` is its dotted path, such as `grunt.attack`.
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

/// The dotted path of a key in a fighter's `attack` object, such as `grunt.attack.reach`.
fn attack_path(section: &str, key: &str) -> String {
    field_path(&field_path(section, ATTACK_KEY), key)
}

/// Reads [`TUNING_PATH`] and returns its numbers unchecked; [`validate`] checks them.
///
/// Every call rereads the file from disk, so edits made while the game runs are picked up.
/// Unknown keys are ignored, so sections can be added before the code that reads them.
pub fn read_raw_tuning() -> Result<RawTuning, TuningError> {
    let root = read_json_object()?;
    let hero = object_field(&root, HERO_SECTION, HERO_SECTION)?;
    let grunt = object_field(&root, GRUNT_SECTION, GRUNT_SECTION)?;
    let dummy = object_field(&root, DUMMY_SECTION, DUMMY_SECTION)?;
    let lane = object_field(&root, LANE_SECTION, LANE_SECTION)?;
    let effects = object_field(&root, EFFECTS_SECTION, EFFECTS_SECTION)?;
    let run = object_field(&root, RUN_SECTION, RUN_SECTION)?;
    Ok(RawTuning {
        hero: read_fighter(&hero, HERO_SECTION)?,
        invulnerability_time: number_in(&hero, HERO_SECTION, INVULNERABILITY_TIME_KEY)?,
        grunt: read_fighter(&grunt, GRUNT_SECTION)?,
        dummy_hurtbox_width: number_in(&dummy, DUMMY_SECTION, HURTBOX_WIDTH_KEY)?,
        dummy_hurtbox_height: number_in(&dummy, DUMMY_SECTION, HURTBOX_HEIGHT_KEY)?,
        lane_top: number_in(&lane, LANE_SECTION, LANE_TOP_KEY)?,
        lane_bottom: number_in(&lane, LANE_SECTION, LANE_BOTTOM_KEY)?,
        depth_tolerance: number_in(&lane, LANE_SECTION, DEPTH_TOLERANCE_KEY)?,
        hit_stop_time: number_in(&effects, EFFECTS_SECTION, HIT_STOP_TIME_KEY)?,
        flash_time: number_in(&effects, EFFECTS_SECTION, FLASH_TIME_KEY)?,
        shake_strength: number_in(&effects, EFFECTS_SECTION, SHAKE_STRENGTH_KEY)?,
        shake_time: number_in(&effects, EFFECTS_SECTION, SHAKE_TIME_KEY)?,
        respawn_delay: number_in(&run, RUN_SECTION, RESPAWN_DELAY_KEY)?,
        death_fade_time: number_in(&run, RUN_SECTION, DEATH_FADE_TIME_KEY)?,
    })
}

/// Reads one fighter's section, including its nested `attack` object.
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

/// Reads the number at `section.key` from the already-read `section` object.
fn number_in(object: &VarDictionary, section: &str, key: &str) -> Result<f64, TuningError> {
    number_field(object, key, &field_path(section, key))
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
            grunt: RawFighter {
                horizontal_speed: 120.0,
                vertical_speed: 70.0,
                max_health: 25.0,
                hurtbox_width: 44.0,
                hurtbox_height: 110.0,
                hit_stun_time: 0.35,
                attack: RawAttack {
                    damage: 12.0,
                    knockback_distance: 70.0,
                    knockback_time: 0.3,
                    ..raw_attack(0.5)
                },
            },
            dummy_hurtbox_width: 38.0,
            dummy_hurtbox_height: 100.0,
            lane_top: TOP,
            lane_bottom: BOTTOM,
            depth_tolerance: 15.0,
            hit_stop_time: 0.05,
            flash_time: 0.1,
            shake_strength: 4.0,
            shake_time: 0.1,
            respawn_delay: 1.2,
            death_fade_time: 0.4,
        }
    }

    fn rejected(raw: RawTuning) -> TuningError {
        validate(&raw, SCREEN).expect_err("should be rejected")
    }

    /// Sets one field of a [`RawTuning`].
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
            ("grunt.horizontal_speed", |r, v| {
                r.grunt.horizontal_speed = v
            }),
            ("grunt.vertical_speed", |r, v| r.grunt.vertical_speed = v),
            ("grunt.max_health", |r, v| r.grunt.max_health = v),
            ("grunt.hurtbox_width", |r, v| r.grunt.hurtbox_width = v),
            ("grunt.hurtbox_height", |r, v| r.grunt.hurtbox_height = v),
            ("grunt.attack.active_time", |r, v| {
                r.grunt.attack.active_time = v
            }),
            ("grunt.attack.reach", |r, v| r.grunt.attack.reach = v),
            ("grunt.attack.damage", |r, v| r.grunt.attack.damage = v),
            ("dummy.hurtbox_width", |r, v| r.dummy_hurtbox_width = v),
            ("dummy.hurtbox_height", |r, v| r.dummy_hurtbox_height = v),
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
            ("grunt.hit_stun_time", |r, v| r.grunt.hit_stun_time = v),
            ("grunt.attack.wind_up_time", |r, v| {
                r.grunt.attack.wind_up_time = v
            }),
            ("grunt.attack.recovery_time", |r, v| {
                r.grunt.attack.recovery_time = v
            }),
            ("grunt.attack.height", |r, v| r.grunt.attack.height = v),
            ("grunt.attack.knockback_distance", |r, v| {
                r.grunt.attack.knockback_distance = v
            }),
            ("lane.depth_tolerance", |r, v| r.depth_tolerance = v),
            ("effects.hit_stop_time", |r, v| r.hit_stop_time = v),
            ("effects.flash_time", |r, v| r.flash_time = v),
            ("effects.shake_strength", |r, v| r.shake_strength = v),
            ("effects.shake_time", |r, v| r.shake_time = v),
            ("run.respawn_delay", |r, v| r.respawn_delay = v),
            ("run.death_fade_time", |r, v| r.death_fade_time = v),
        ]
    }

    // --- Values are read into the right places. ---

    #[test]
    fn validate_accepts_valid_values_and_keeps_them_apart() {
        let values = validate(&raw(), SCREEN).expect("all values are valid");
        let hero = values.hero();
        assert_eq!(hero.horizontal_speed(), 250.0);
        assert_eq!(hero.vertical_speed(), 150.0);
        assert_eq!(hero.max_health(), 80.0);
        assert_eq!(hero.hurtbox_width(), 40.0);
        assert_eq!(hero.hurtbox_height(), 90.0);
        assert_eq!(hero.hit_stun_time(), 0.2);
        assert_eq!(hero.attack().wind_up_time(), 0.05);
        assert_eq!(hero.attack().damage(), 7.0);
        assert_eq!(hero.attack().knockback_distance(), 50.0);
        assert_eq!(values.invulnerability_time(), 0.8);
        let grunt = values.grunt();
        assert_eq!(grunt.horizontal_speed(), 120.0);
        assert_eq!(grunt.max_health(), 25.0);
        assert_eq!(grunt.hit_stun_time(), 0.35);
        assert_eq!(grunt.attack().wind_up_time(), 0.5);
        assert_eq!(grunt.attack().damage(), 12.0);
        assert_eq!(grunt.attack().knockback_distance(), 70.0);
        assert_eq!(grunt.attack().knockback_time(), 0.3);
        assert_eq!(values.dummy().hurtbox_width(), 38.0);
        assert_eq!(values.lane_top(), 400.0);
        assert_eq!(values.lane_bottom(), 650.0);
        assert_eq!(values.depth_tolerance(), 15.0);
        assert_eq!(values.effects().hit_stop_time(), 0.05);
        assert_eq!(values.effects().shake_strength(), 4.0);
        assert_eq!(values.run().respawn_delay(), 1.2);
        assert_eq!(values.run().death_fade_time(), 0.4);
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
        off.grunt.attack.knockback_distance = 0.0;
        off.grunt.attack.knockback_time = 0.0;
        assert!(validate(&off, SCREEN).is_ok());
        for section in ["hero", "grunt"] {
            let mut wrong = raw();
            match section {
                "hero" => wrong.hero.attack.knockback_time = 0.0,
                _ => wrong.grunt.attack.knockback_time = 0.0,
            }
            let error = rejected(wrong);
            assert_eq!(error.field(), format!("{section}.attack.knockback_time"));
            let reason = error.reason();
            assert!(
                reason.contains(&format!("{section}.attack.knockback_distance")),
                "{reason}"
            );
        }
        let mut negative = raw();
        negative.grunt.attack.knockback_distance = 0.0;
        negative.grunt.attack.knockback_time = -1.0;
        assert_eq!(rejected(negative).field(), "grunt.attack.knockback_time");
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
    fn validate_rejection_gives_field_reason_and_full_message() {
        let mut wrong = raw();
        wrong.grunt.max_health = -3.0;
        let error = rejected(wrong);
        assert_eq!(error.field(), "grunt.max_health");
        let reason = error.reason();
        assert!(reason.contains("> 0"), "{reason}");
        assert!(reason.contains("-3"), "{reason}");
        assert!(!reason.contains("res://"), "{reason}");
        let message = error.to_string();
        assert!(message.contains("res://data/tuning.json"), "{message}");
        assert!(message.contains("grunt.max_health"), "{message}");
    }

    // --- The wind-up warning: accepted, but flagged. ---

    #[test]
    fn warnings_flag_a_grunt_wind_up_not_longer_than_the_heros() {
        let values = validate(&raw(), SCREEN).expect("valid");
        assert!(warnings(&values).is_empty());
        for grunt_wind_up in [0.05, 0.03, 0.0] {
            let mut short = raw();
            short.grunt.attack.wind_up_time = grunt_wind_up;
            let values = validate(&short, SCREEN).expect("still accepted");
            assert_eq!(
                warnings(&values),
                vec![TuningWarning::GruntWindUpNotLonger {
                    hero: 0.05,
                    grunt: grunt_wind_up as f32,
                }],
                "{grunt_wind_up}"
            );
        }
    }

    #[test]
    fn warning_message_names_both_fields_and_values() {
        let message = TuningWarning::GruntWindUpNotLonger {
            hero: 0.05,
            grunt: 0.03,
        }
        .to_string();
        assert!(
            message.starts_with("grunt.attack.wind_up_time"),
            "{message}"
        );
        assert!(
            message.contains("hero.attack.wind_up_time (0.05)"),
            "{message}"
        );
        assert!(message.contains("got 0.03"), "{message}");
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
            field: "hero.attack".to_string(),
        };
        assert_eq!(missing.field(), "hero.attack");
        assert_eq!(missing.reason(), "missing");
    }
}
