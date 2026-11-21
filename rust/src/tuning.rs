//! Loading and checking the project-wide tuning data file.
//!
//! Every tunable value lives in [`TUNING_PATH`], never in code (constitution, Principle IV).
//! A missing or invalid value is an error, never a silent fallback. Reading the file
//! ([`read_raw_tuning`]) is kept apart from checking it ([`validate`]), so the rules are plain
//! functions tested without the engine. The current values are held by
//! [`TuningStore`](crate::tuning_store::TuningStore). The file format is described in
//! `specs/002-m1-lane-movement/contracts/tuning-data.md`.

use std::fmt;

use godot::classes::Json;
use godot::classes::file_access::ModeFlags;
use godot::global::Error as GodotError;
use godot::prelude::*;
use godot::tools::GFile;

/// Godot path of the tuning data file. This is a file location, not a tuning value.
pub const TUNING_PATH: &str = "res://data/tuning.json";

/// Section of the tuning file holding the hero's movement speeds.
const HERO_SECTION: &str = "hero";

/// Section of the tuning file holding the lane's limits.
const LANE_SECTION: &str = "lane";

/// Keys inside [`HERO_SECTION`].
const HORIZONTAL_SPEED_KEY: &str = "horizontal_speed";
const VERTICAL_SPEED_KEY: &str = "vertical_speed";

/// Keys inside [`LANE_SECTION`].
const LANE_TOP_KEY: &str = "top";
const LANE_BOTTOM_KEY: &str = "bottom";

/// Section holding the hero's swing: its timing, reach and depth tolerance.
const ATTACK_SECTION: &str = "attack";
/// Section holding what a landed blow does: freeze, flash, shake and knockback.
const IMPACT_SECTION: &str = "impact";
/// Section holding the training dummy's hurtbox.
const DUMMY_SECTION: &str = "dummy";

/// Keys inside [`ATTACK_SECTION`].
const WIND_UP_TIME_KEY: &str = "wind_up_time";
const ACTIVE_TIME_KEY: &str = "active_time";
const RECOVERY_TIME_KEY: &str = "recovery_time";
const REACH_KEY: &str = "reach";
const ATTACK_HEIGHT_KEY: &str = "height";
const DEPTH_TOLERANCE_KEY: &str = "depth_tolerance";

/// Keys inside [`IMPACT_SECTION`].
const HIT_STOP_TIME_KEY: &str = "hit_stop_time";
const FLASH_TIME_KEY: &str = "flash_time";
const SHAKE_STRENGTH_KEY: &str = "shake_strength";
const SHAKE_TIME_KEY: &str = "shake_time";
const KNOCKBACK_DISTANCE_KEY: &str = "knockback_distance";
const KNOCKBACK_TIME_KEY: &str = "knockback_time";

/// Keys inside [`DUMMY_SECTION`].
const HURTBOX_WIDTH_KEY: &str = "hurtbox_width";
const HURTBOX_HEIGHT_KEY: &str = "hurtbox_height";

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

/// The movement tuning numbers as read from the file, before any rule is applied. Only
/// [`validate`] turns them into values the game may use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawTuning {
    /// `hero.horizontal_speed`, in pixels per second.
    pub horizontal_speed: f64,
    /// `hero.vertical_speed`, in pixels per second.
    pub vertical_speed: f64,
    /// `lane.top`, in pixels from the top of the screen.
    pub lane_top: f64,
    /// `lane.bottom`, in pixels from the top of the screen.
    pub lane_bottom: f64,
    /// `attack.wind_up_time`, in seconds.
    pub wind_up_time: f64,
    /// `attack.active_time`, in seconds.
    pub active_time: f64,
    /// `attack.recovery_time`, in seconds.
    pub recovery_time: f64,
    /// `attack.reach`, in pixels in front of the hero's body.
    pub reach: f64,
    /// `attack.height`, in pixels above the feet.
    pub attack_height: f64,
    /// `attack.depth_tolerance`, in pixels of feet height.
    pub depth_tolerance: f64,
    /// `impact.hit_stop_time`, in seconds.
    pub hit_stop_time: f64,
    /// `impact.flash_time`, in seconds.
    pub flash_time: f64,
    /// `impact.shake_strength`, in pixels.
    pub shake_strength: f64,
    /// `impact.shake_time`, in seconds.
    pub shake_time: f64,
    /// `impact.knockback_distance`, in pixels.
    pub knockback_distance: f64,
    /// `impact.knockback_time`, in seconds.
    pub knockback_time: f64,
    /// `dummy.hurtbox_width`, in pixels.
    pub hurtbox_width: f64,
    /// `dummy.hurtbox_height`, in pixels above the feet.
    pub hurtbox_height: f64,
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
    /// The hero's swing (`attack` section).
    AttackTuning {
        /// Seconds from the press until the blow can land.
        wind_up_time,
        /// Seconds during which the blow can land; always above 0.
        active_time,
        /// Seconds after the active part before the hero is free again.
        recovery_time,
        /// How far the swing reaches in front of the hero's body, in pixels; always above 0.
        reach,
        /// How tall the drawn attack box is above the feet, in pixels.
        height,
        /// The largest difference in feet height at which a blow still lands, in pixels.
        depth_tolerance,
    }
}

value_group! {
    /// What a landed blow does (`impact` section). A value of 0 turns its effect off.
    ImpactTuning {
        /// How long play freezes on impact, in seconds.
        hit_stop_time,
        /// How long the hit dummy flashes white, in seconds.
        flash_time,
        /// How far the screen shakes at most, in pixels.
        shake_strength,
        /// How long the screen shakes, in seconds.
        shake_time,
        /// How far a hit dummy slides, in pixels.
        knockback_distance,
        /// How long the slide takes, in seconds; above 0 whenever there is a distance.
        knockback_time,
    }
}

value_group! {
    /// The training dummy's hurtbox (`dummy` section).
    DummyTuning {
        /// Width of the hurtbox, centred on the feet, in pixels.
        hurtbox_width,
        /// Height of the hurtbox above the feet, in pixels.
        hurtbox_height,
    }
}

/// Checked movement tuning: the hero's speeds and the lane's limits. It can only be built by
/// [`validate`], so an instance always holds values that passed every rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TuningValues {
    horizontal_speed: f32,
    vertical_speed: f32,
    lane_top: f32,
    lane_bottom: f32,
    attack: AttackTuning,
    impact: ImpactTuning,
    dummy: DummyTuning,
}

impl TuningValues {
    /// Speed along the lane, in pixels per second.
    pub fn horizontal_speed(&self) -> f32 {
        self.horizontal_speed
    }

    /// Speed into and out of the lane, in pixels per second.
    pub fn vertical_speed(&self) -> f32 {
        self.vertical_speed
    }

    /// The highest y (furthest into the lane) a character's feet may reach.
    pub fn lane_top(&self) -> f32 {
        self.lane_top
    }

    /// The lowest y (nearest the player) a character's feet may reach.
    pub fn lane_bottom(&self) -> f32 {
        self.lane_bottom
    }

    /// The hero's swing values.
    pub fn attack(&self) -> AttackTuning {
        self.attack
    }

    /// What a landed blow does.
    pub fn impact(&self) -> ImpactTuning {
        self.impact
    }

    /// The training dummy's hurtbox.
    pub fn dummy(&self) -> DummyTuning {
        self.dummy
    }
}

/// Applies every tuning rule to `raw` and returns the values only if all of them pass, so a
/// caller can never use a mix of good and bad values.
///
/// Start-up and reload both call this, so the same rules apply to both. Each value is converted
/// to `f32` first, so a huge finite number that overflows is rejected too. Rules: both speeds
/// finite and `> 0`; the lane bottom finite and at most `screen_height`; the lane top finite,
/// `>= 0` and `<` the lane bottom.
pub fn validate(raw: &RawTuning, screen_height: f32) -> Result<TuningValues, TuningError> {
    let horizontal = positive(raw.horizontal_speed, HERO_SECTION, HORIZONTAL_SPEED_KEY)?;
    let vertical = positive(raw.vertical_speed, HERO_SECTION, VERTICAL_SPEED_KEY)?;
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
    let attack = AttackTuning {
        wind_up_time: non_negative(raw.wind_up_time, ATTACK_SECTION, WIND_UP_TIME_KEY)?,
        active_time: positive(raw.active_time, ATTACK_SECTION, ACTIVE_TIME_KEY)?,
        recovery_time: non_negative(raw.recovery_time, ATTACK_SECTION, RECOVERY_TIME_KEY)?,
        reach: positive(raw.reach, ATTACK_SECTION, REACH_KEY)?,
        height: non_negative(raw.attack_height, ATTACK_SECTION, ATTACK_HEIGHT_KEY)?,
        depth_tolerance: non_negative(raw.depth_tolerance, ATTACK_SECTION, DEPTH_TOLERANCE_KEY)?,
    };
    let knockback_distance = non_negative(
        raw.knockback_distance,
        IMPACT_SECTION,
        KNOCKBACK_DISTANCE_KEY,
    )?;
    let knockback_time = non_negative(raw.knockback_time, IMPACT_SECTION, KNOCKBACK_TIME_KEY)?;
    if knockback_distance > 0.0 && knockback_time == 0.0 {
        return Err(invalid_value(
            &field_path(IMPACT_SECTION, KNOCKBACK_TIME_KEY),
            raw.knockback_time.to_string(),
            format!(
                "must be greater than 0 while {} is {}",
                field_path(IMPACT_SECTION, KNOCKBACK_DISTANCE_KEY),
                raw.knockback_distance
            ),
        ));
    }
    let impact = ImpactTuning {
        hit_stop_time: non_negative(raw.hit_stop_time, IMPACT_SECTION, HIT_STOP_TIME_KEY)?,
        flash_time: non_negative(raw.flash_time, IMPACT_SECTION, FLASH_TIME_KEY)?,
        shake_strength: non_negative(raw.shake_strength, IMPACT_SECTION, SHAKE_STRENGTH_KEY)?,
        shake_time: non_negative(raw.shake_time, IMPACT_SECTION, SHAKE_TIME_KEY)?,
        knockback_distance,
        knockback_time,
    };
    let dummy = DummyTuning {
        hurtbox_width: non_negative(raw.hurtbox_width, DUMMY_SECTION, HURTBOX_WIDTH_KEY)?,
        hurtbox_height: non_negative(raw.hurtbox_height, DUMMY_SECTION, HURTBOX_HEIGHT_KEY)?,
    };
    Ok(TuningValues {
        horizontal_speed: horizontal,
        vertical_speed: vertical,
        lane_top: top,
        lane_bottom: bottom,
        attack,
        impact,
        dummy,
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

/// Reads [`TUNING_PATH`] and returns its movement numbers unchecked; [`validate`] checks them.
///
/// Every call rereads the file from disk, so edits made while the game runs are picked up.
/// Unknown keys are ignored, so sections can be added before the code that reads them.
pub fn read_raw_tuning() -> Result<RawTuning, TuningError> {
    let root = read_json_object()?;
    let hero = object_field(&root, HERO_SECTION, HERO_SECTION)?;
    let lane = object_field(&root, LANE_SECTION, LANE_SECTION)?;
    let attack = object_field(&root, ATTACK_SECTION, ATTACK_SECTION)?;
    let impact = object_field(&root, IMPACT_SECTION, IMPACT_SECTION)?;
    let dummy = object_field(&root, DUMMY_SECTION, DUMMY_SECTION)?;
    Ok(RawTuning {
        horizontal_speed: number_field(
            &hero,
            HORIZONTAL_SPEED_KEY,
            &field_path(HERO_SECTION, HORIZONTAL_SPEED_KEY),
        )?,
        vertical_speed: number_field(
            &hero,
            VERTICAL_SPEED_KEY,
            &field_path(HERO_SECTION, VERTICAL_SPEED_KEY),
        )?,
        lane_top: number_field(&lane, LANE_TOP_KEY, &field_path(LANE_SECTION, LANE_TOP_KEY))?,
        lane_bottom: number_field(
            &lane,
            LANE_BOTTOM_KEY,
            &field_path(LANE_SECTION, LANE_BOTTOM_KEY),
        )?,
        wind_up_time: number_in(&attack, ATTACK_SECTION, WIND_UP_TIME_KEY)?,
        active_time: number_in(&attack, ATTACK_SECTION, ACTIVE_TIME_KEY)?,
        recovery_time: number_in(&attack, ATTACK_SECTION, RECOVERY_TIME_KEY)?,
        reach: number_in(&attack, ATTACK_SECTION, REACH_KEY)?,
        attack_height: number_in(&attack, ATTACK_SECTION, ATTACK_HEIGHT_KEY)?,
        depth_tolerance: number_in(&attack, ATTACK_SECTION, DEPTH_TOLERANCE_KEY)?,
        hit_stop_time: number_in(&impact, IMPACT_SECTION, HIT_STOP_TIME_KEY)?,
        flash_time: number_in(&impact, IMPACT_SECTION, FLASH_TIME_KEY)?,
        shake_strength: number_in(&impact, IMPACT_SECTION, SHAKE_STRENGTH_KEY)?,
        shake_time: number_in(&impact, IMPACT_SECTION, SHAKE_TIME_KEY)?,
        knockback_distance: number_in(&impact, IMPACT_SECTION, KNOCKBACK_DISTANCE_KEY)?,
        knockback_time: number_in(&impact, IMPACT_SECTION, KNOCKBACK_TIME_KEY)?,
        hurtbox_width: number_in(&dummy, DUMMY_SECTION, HURTBOX_WIDTH_KEY)?,
        hurtbox_height: number_in(&dummy, DUMMY_SECTION, HURTBOX_HEIGHT_KEY)?,
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
    const HORIZONTAL: f64 = 250.0;
    const VERTICAL: f64 = 150.0;
    const TOP: f64 = 400.0;
    const BOTTOM: f64 = 650.0;
    const NOT_FINITE: [f64; 3] = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];

    // --- validate(): one passing and one failing case per rule (M2a). ---

    const SCREEN: f32 = 720.0;

    fn raw() -> RawTuning {
        RawTuning {
            horizontal_speed: HORIZONTAL,
            vertical_speed: VERTICAL,
            lane_top: TOP,
            lane_bottom: BOTTOM,
            wind_up_time: 0.05,
            active_time: 0.12,
            recovery_time: 0.2,
            reach: 80.0,
            attack_height: 50.0,
            depth_tolerance: 15.0,
            hit_stop_time: 0.05,
            flash_time: 0.1,
            shake_strength: 4.0,
            shake_time: 0.1,
            knockback_distance: 50.0,
            knockback_time: 0.25,
            hurtbox_width: 40.0,
            hurtbox_height: 100.0,
        }
    }

    fn rejected(raw: RawTuning) -> TuningError {
        validate(&raw, SCREEN).expect_err("should be rejected")
    }

    #[test]
    fn validate_accepts_valid_values() {
        let values = validate(&raw(), SCREEN).expect("all values are valid");
        assert_eq!(values.horizontal_speed(), 250.0);
        assert_eq!(values.vertical_speed(), 150.0);
        assert_eq!(values.lane_top(), 400.0);
        assert_eq!(values.lane_bottom(), 650.0);
    }

    #[test]
    fn validate_horizontal_speed_must_be_finite_and_positive() {
        let ok = RawTuning {
            horizontal_speed: 1.0,
            ..raw()
        };
        assert!(validate(&ok, SCREEN).is_ok());
        for bad in [0.0, -5.0, 1e300].into_iter().chain(NOT_FINITE) {
            let error = rejected(RawTuning {
                horizontal_speed: bad,
                ..raw()
            });
            assert_eq!(error.field(), "hero.horizontal_speed", "{bad}");
        }
    }

    #[test]
    fn validate_vertical_speed_must_be_finite_and_positive() {
        let ok = RawTuning {
            vertical_speed: 1.0,
            ..raw()
        };
        assert!(validate(&ok, SCREEN).is_ok());
        for bad in [0.0, -5.0, 1e300].into_iter().chain(NOT_FINITE) {
            let error = rejected(RawTuning {
                vertical_speed: bad,
                ..raw()
            });
            assert_eq!(error.field(), "hero.vertical_speed", "{bad}");
        }
    }

    #[test]
    fn validate_lane_top_must_be_finite_and_not_negative() {
        let ok = RawTuning {
            lane_top: 0.0,
            ..raw()
        };
        assert!(validate(&ok, SCREEN).is_ok());
        for bad in [-1.0, f64::NAN] {
            let error = rejected(RawTuning {
                lane_top: bad,
                ..raw()
            });
            assert_eq!(error.field(), "lane.top", "{bad}");
        }
    }

    #[test]
    fn validate_lane_bottom_must_be_finite() {
        let ok = RawTuning {
            lane_bottom: 690.0,
            ..raw()
        };
        assert!(validate(&ok, SCREEN).is_ok());
        let error = rejected(RawTuning {
            lane_bottom: f64::INFINITY,
            ..raw()
        });
        assert_eq!(error.field(), "lane.bottom");
    }

    #[test]
    fn validate_lane_top_must_be_above_lane_bottom() {
        let ok = RawTuning {
            lane_top: BOTTOM - 1.0,
            ..raw()
        };
        assert!(validate(&ok, SCREEN).is_ok());
        for bad in [BOTTOM, BOTTOM + 1.0] {
            let error = rejected(RawTuning {
                lane_top: bad,
                ..raw()
            });
            assert_eq!(error.field(), "lane.top", "{bad}");
            assert!(error.reason().contains("lane.bottom"), "{}", error.reason());
        }
    }

    #[test]
    fn validate_lane_bottom_must_be_on_the_screen() {
        let ok = RawTuning {
            lane_bottom: 720.0,
            ..raw()
        };
        assert!(validate(&ok, SCREEN).is_ok());
        let error = rejected(RawTuning {
            lane_bottom: 721.0,
            ..raw()
        });
        assert_eq!(error.field(), "lane.bottom");
        assert!(error.reason().contains("720"), "{}", error.reason());
    }

    #[test]
    fn validate_rejection_gives_field_reason_and_full_message() {
        let error = rejected(RawTuning {
            lane_top: 666.0,
            ..raw()
        });
        assert_eq!(error.field(), "lane.top");
        let reason = error.reason();
        assert!(reason.contains("must be less than lane.bottom"), "{reason}");
        assert!(reason.contains("666"), "{reason}");
        assert!(!reason.contains("res://"), "{reason}");
        let message = error.to_string();
        assert!(message.contains("res://data/tuning.json"), "{message}");
        assert!(message.contains("lane.top"), "{message}");
        assert!(message.contains("666"), "{message}");
    }

    // --- M2b: attack, impact and dummy values. ---

    /// Sets one field of a [`RawTuning`].
    type Setter = fn(&mut RawTuning, f64);

    /// Every new field that only has to be finite and `>= 0`, with its path and a setter.
    fn non_negative_fields() -> Vec<(&'static str, Setter)> {
        vec![
            ("attack.wind_up_time", |r, v| r.wind_up_time = v),
            ("attack.recovery_time", |r, v| r.recovery_time = v),
            ("attack.height", |r, v| r.attack_height = v),
            ("attack.depth_tolerance", |r, v| r.depth_tolerance = v),
            ("impact.hit_stop_time", |r, v| r.hit_stop_time = v),
            ("impact.flash_time", |r, v| r.flash_time = v),
            ("impact.shake_strength", |r, v| r.shake_strength = v),
            ("impact.shake_time", |r, v| r.shake_time = v),
            ("impact.knockback_distance", |r, v| r.knockback_distance = v),
            ("dummy.hurtbox_width", |r, v| r.hurtbox_width = v),
            ("dummy.hurtbox_height", |r, v| r.hurtbox_height = v),
        ]
    }

    #[test]
    fn validate_reads_the_combat_values() {
        let values = validate(&raw(), SCREEN).expect("all values are valid");
        let attack = values.attack();
        assert_eq!(attack.wind_up_time(), 0.05);
        assert_eq!(attack.active_time(), 0.12);
        assert_eq!(attack.recovery_time(), 0.2);
        assert_eq!(attack.reach(), 80.0);
        assert_eq!(attack.height(), 50.0);
        assert_eq!(attack.depth_tolerance(), 15.0);
        let impact = values.impact();
        assert_eq!(impact.hit_stop_time(), 0.05);
        assert_eq!(impact.flash_time(), 0.1);
        assert_eq!(impact.shake_strength(), 4.0);
        assert_eq!(impact.shake_time(), 0.1);
        assert_eq!(impact.knockback_distance(), 50.0);
        assert_eq!(impact.knockback_time(), 0.25);
        let dummy = values.dummy();
        assert_eq!(dummy.hurtbox_width(), 40.0);
        assert_eq!(dummy.hurtbox_height(), 100.0);
    }

    #[test]
    fn validate_combat_values_may_be_zero_but_not_negative_or_not_finite() {
        for (path, set) in non_negative_fields() {
            let mut ok = raw();
            set(&mut ok, 0.0);
            if path == "impact.knockback_distance" {
                ok.knockback_time = 0.0;
            }
            assert!(validate(&ok, SCREEN).is_ok(), "{path} = 0 should pass");
            for bad in [-1.0, f64::NAN] {
                let mut wrong = raw();
                set(&mut wrong, bad);
                assert_eq!(rejected(wrong).field(), path, "{path} = {bad}");
            }
        }
    }

    #[test]
    fn validate_active_time_must_be_above_zero() {
        let ok = RawTuning {
            active_time: 0.01,
            ..raw()
        };
        assert!(validate(&ok, SCREEN).is_ok());
        let error = rejected(RawTuning {
            active_time: 0.0,
            ..raw()
        });
        assert_eq!(error.field(), "attack.active_time");
    }

    #[test]
    fn validate_reach_must_be_above_zero() {
        let ok = RawTuning {
            reach: 1.0,
            ..raw()
        };
        assert!(validate(&ok, SCREEN).is_ok());
        let error = rejected(RawTuning {
            reach: 0.0,
            ..raw()
        });
        assert_eq!(error.field(), "attack.reach");
    }

    #[test]
    fn validate_knockback_time_must_be_above_zero_while_there_is_a_distance() {
        let off = RawTuning {
            knockback_distance: 0.0,
            knockback_time: 0.0,
            ..raw()
        };
        assert!(validate(&off, SCREEN).is_ok());
        let error = rejected(RawTuning {
            knockback_time: 0.0,
            ..raw()
        });
        assert_eq!(error.field(), "impact.knockback_time");
        assert!(
            error.reason().contains("impact.knockback_distance"),
            "{}",
            error.reason()
        );
        let negative = rejected(RawTuning {
            knockback_distance: 0.0,
            knockback_time: -1.0,
            ..raw()
        });
        assert_eq!(negative.field(), "impact.knockback_time");
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
            field: "lane.top".to_string(),
        };
        assert_eq!(missing.field(), "lane.top");
        assert_eq!(missing.reason(), "missing");
    }
}
