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
}

/// Checked movement tuning: the hero's speeds and the lane's limits. It can only be built by
/// [`validate`], so an instance always holds values that passed every rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TuningValues {
    horizontal_speed: f32,
    vertical_speed: f32,
    lane_top: f32,
    lane_bottom: f32,
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
}

/// Applies every tuning rule to `raw` and returns the values only if all of them pass, so a
/// caller can never use a mix of good and bad values.
///
/// Start-up and reload both call this, so the same rules apply to both. Each value is converted
/// to `f32` first, so a huge finite number that overflows is rejected too. Rules: both speeds
/// finite and `> 0`; the lane bottom finite and at most `screen_height`; the lane top finite,
/// `>= 0` and `<` the lane bottom.
pub fn validate(raw: &RawTuning, screen_height: f32) -> Result<TuningValues, TuningError> {
    let positive = "must be a finite number > 0";
    let horizontal = checked(
        raw.horizontal_speed,
        HERO_SECTION,
        HORIZONTAL_SPEED_KEY,
        positive,
        |v| v > 0.0,
    )?;
    let vertical = checked(
        raw.vertical_speed,
        HERO_SECTION,
        VERTICAL_SPEED_KEY,
        positive,
        |v| v > 0.0,
    )?;
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
    Ok(TuningValues {
        horizontal_speed: horizontal,
        vertical_speed: vertical,
        lane_top: top,
        lane_bottom: bottom,
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
