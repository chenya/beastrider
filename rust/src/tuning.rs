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

/// Section of the tuning file read by `DriftingSquare`.
const SQUARE_SECTION: &str = "drifting_square";

/// Key of the square's speed inside [`SQUARE_SECTION`].
const SPEED_KEY: &str = "speed";

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

/// Checked tuning for the drifting square. It can only be built through
/// [`SquareTuning::try_new`], so an instance always holds valid values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SquareTuning {
    speed: f32,
}

impl SquareTuning {
    /// Builds the tuning from a raw speed in pixels per second.
    ///
    /// Accepts only values that are finite and `>= 0` once converted to `f32`. `0` is valid and
    /// means the square stays still.
    pub fn try_new(speed: f64) -> Result<Self, TuningError> {
        // Convert first: a huge but finite f64 becomes infinite as f32 and must be rejected too.
        let speed_f32 = speed as f32;
        if speed_f32.is_finite() && speed_f32 >= 0.0 {
            Ok(Self { speed: speed_f32 })
        } else {
            Err(invalid_value(
                &speed_field(),
                speed.to_string(),
                "must be a finite number >= 0".to_string(),
            ))
        }
    }

    /// Horizontal speed in pixels per second.
    pub fn speed(&self) -> f32 {
        self.speed
    }
}

/// Reads [`TUNING_PATH`] and returns the checked drifting-square tuning.
///
/// Unknown keys are ignored, so sections can be added before the code that reads them.
pub fn load_square_tuning() -> Result<SquareTuning, TuningError> {
    let root = read_json_object()?;
    let section = object_field(&root, SQUARE_SECTION, SQUARE_SECTION)?;
    let speed = number_field(&section, SPEED_KEY, &speed_field())?;
    SquareTuning::try_new(speed)
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

fn speed_field() -> String {
    format!("{SQUARE_SECTION}.{SPEED_KEY}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_zero_and_positive_speeds() {
        assert_eq!(SquareTuning::try_new(0.0).map(|t| t.speed()), Ok(0.0));
        assert_eq!(SquareTuning::try_new(42.5).map(|t| t.speed()), Ok(42.5));
    }

    #[test]
    fn rejects_negative_nan_and_infinite_speeds() {
        for bad in [-5.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1e300] {
            assert!(SquareTuning::try_new(bad).is_err(), "accepted {bad}");
        }
    }

    #[test]
    fn rejection_message_names_file_field_and_value() {
        let message = SquareTuning::try_new(-5.0)
            .expect_err("-5 is negative")
            .to_string();
        assert!(message.contains("res://data/tuning.json"), "{message}");
        assert!(message.contains("drifting_square.speed"), "{message}");
        assert!(message.contains("-5"), "{message}");
    }
}
