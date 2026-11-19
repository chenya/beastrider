//! Loading and checking the project-wide tuning data file.
//!
//! Every tunable value lives in [`TUNING_PATH`], never in code (constitution, Principle IV).
//! A missing or invalid value is an error, never a silent fallback. The file format is
//! described in `specs/001-m0-setup/contracts/tuning-data.md`.

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

impl std::error::Error for TuningError {}

/// Checked movement tuning: the hero's speeds and the lane's limits. It can only be built through
/// [`MovementTuning::try_new`], so an instance always holds valid values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementTuning {
    horizontal_speed: f32,
    vertical_speed: f32,
    lane_top: f32,
    lane_bottom: f32,
}

impl MovementTuning {
    /// Builds the tuning from raw values: speeds in pixels per second, lane limits in pixels from
    /// the top of the screen.
    ///
    /// Each value is converted to `f32` first, so a huge but finite number that overflows is
    /// rejected too. Speeds must be finite and `> 0`; the lane top must be finite, `>= 0` and
    /// `<` the lane bottom; the lane bottom must be finite. Whether the lane fits the screen is
    /// checked separately by [`check_lane_fits`], because only the caller knows the screen height.
    pub fn try_new(
        horizontal_speed: f64,
        vertical_speed: f64,
        lane_top: f64,
        lane_bottom: f64,
    ) -> Result<Self, TuningError> {
        let positive = "must be a finite number > 0";
        let horizontal = checked(
            horizontal_speed,
            HERO_SECTION,
            HORIZONTAL_SPEED_KEY,
            positive,
            |v| v > 0.0,
        )?;
        let vertical = checked(
            vertical_speed,
            HERO_SECTION,
            VERTICAL_SPEED_KEY,
            positive,
            |v| v > 0.0,
        )?;
        let bottom = checked(
            lane_bottom,
            LANE_SECTION,
            LANE_BOTTOM_KEY,
            "must be a finite number",
            |_| true,
        )?;
        let top = checked(
            lane_top,
            LANE_SECTION,
            LANE_TOP_KEY,
            "must be a finite number >= 0",
            |v| v >= 0.0,
        )?;
        if top >= bottom {
            return Err(invalid_value(
                &field_path(LANE_SECTION, LANE_TOP_KEY),
                lane_top.to_string(),
                format!(
                    "must be less than {} ({lane_bottom})",
                    field_path(LANE_SECTION, LANE_BOTTOM_KEY)
                ),
            ));
        }
        Ok(Self {
            horizontal_speed: horizontal,
            vertical_speed: vertical,
            lane_top: top,
            lane_bottom: bottom,
        })
    }

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

/// Checks that the lane ends on the screen: `lane.bottom` must not be below `screen_height`.
pub fn check_lane_fits(tuning: &MovementTuning, screen_height: f32) -> Result<(), TuningError> {
    if tuning.lane_bottom <= screen_height {
        Ok(())
    } else {
        Err(invalid_value(
            &field_path(LANE_SECTION, LANE_BOTTOM_KEY),
            tuning.lane_bottom.to_string(),
            format!("must be at most the screen height ({screen_height})"),
        ))
    }
}

/// Reads [`TUNING_PATH`] and returns the checked movement tuning.
///
/// Unknown keys are ignored, so sections can be added before the code that reads them.
pub fn load_movement_tuning() -> Result<MovementTuning, TuningError> {
    let root = read_json_object()?;
    let hero = object_field(&root, HERO_SECTION, HERO_SECTION)?;
    let lane = object_field(&root, LANE_SECTION, LANE_SECTION)?;
    MovementTuning::try_new(
        number_field(
            &hero,
            HORIZONTAL_SPEED_KEY,
            &field_path(HERO_SECTION, HORIZONTAL_SPEED_KEY),
        )?,
        number_field(
            &hero,
            VERTICAL_SPEED_KEY,
            &field_path(HERO_SECTION, VERTICAL_SPEED_KEY),
        )?,
        number_field(&lane, LANE_TOP_KEY, &field_path(LANE_SECTION, LANE_TOP_KEY))?,
        number_field(
            &lane,
            LANE_BOTTOM_KEY,
            &field_path(LANE_SECTION, LANE_BOTTOM_KEY),
        )?,
    )
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
            line: json.get_error_line(),
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
                format!("must be a number, got {other:?}"),
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

    fn message(result: Result<MovementTuning, TuningError>) -> String {
        result.expect_err("should be rejected").to_string()
    }

    #[test]
    fn accepts_valid_values() {
        let tuning = MovementTuning::try_new(HORIZONTAL, VERTICAL, TOP, BOTTOM)
            .expect("all values are valid");
        assert_eq!(tuning.horizontal_speed(), 250.0);
        assert_eq!(tuning.vertical_speed(), 150.0);
        assert_eq!(tuning.lane_top(), 400.0);
        assert_eq!(tuning.lane_bottom(), 650.0);
    }

    #[test]
    fn rejects_speeds_that_are_not_finite_and_positive() {
        for bad in [0.0, -5.0, 1e300].into_iter().chain(NOT_FINITE) {
            assert!(
                MovementTuning::try_new(bad, VERTICAL, TOP, BOTTOM).is_err(),
                "{bad}"
            );
            assert!(
                MovementTuning::try_new(HORIZONTAL, bad, TOP, BOTTOM).is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn rejects_a_lane_top_above_the_screen_or_not_above_the_bottom() {
        for bad_top in [-1.0, BOTTOM, BOTTOM + 1.0].into_iter().chain(NOT_FINITE) {
            assert!(
                MovementTuning::try_new(HORIZONTAL, VERTICAL, bad_top, BOTTOM).is_err(),
                "{bad_top}"
            );
        }
    }

    #[test]
    fn rejects_a_lane_bottom_that_is_not_finite() {
        for bad_bottom in NOT_FINITE {
            assert!(
                MovementTuning::try_new(HORIZONTAL, VERTICAL, TOP, bad_bottom).is_err(),
                "{bad_bottom}"
            );
        }
    }

    #[test]
    fn rejection_message_names_file_field_and_value() {
        let speed = message(MovementTuning::try_new(HORIZONTAL, -5.0, TOP, BOTTOM));
        assert!(speed.contains("res://data/tuning.json"), "{speed}");
        assert!(speed.contains("hero.vertical_speed"), "{speed}");
        assert!(speed.contains("-5"), "{speed}");

        let lane = message(MovementTuning::try_new(HORIZONTAL, VERTICAL, 666.0, BOTTOM));
        assert!(lane.contains("res://data/tuning.json"), "{lane}");
        assert!(lane.contains("lane.top"), "{lane}");
        assert!(lane.contains("666"), "{lane}");
    }

    #[test]
    fn lane_fits_a_screen_exactly_as_tall_as_its_bottom() {
        let tuning = MovementTuning::try_new(HORIZONTAL, VERTICAL, TOP, BOTTOM).expect("valid");
        assert_eq!(check_lane_fits(&tuning, 650.0), Ok(()));
    }

    #[test]
    fn lane_below_the_screen_is_rejected_naming_lane_bottom() {
        let tuning = MovementTuning::try_new(HORIZONTAL, VERTICAL, TOP, BOTTOM).expect("valid");
        let error = check_lane_fits(&tuning, 600.0)
            .expect_err("650 > 600")
            .to_string();
        assert!(error.contains("lane.bottom"), "{error}");
        assert!(error.contains("650"), "{error}");
    }
}
