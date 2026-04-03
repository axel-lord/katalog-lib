//! Setting factories.

use crate::{Primitive, Setting};

/// Construct a setting for string values.
pub const fn string(path: fn() -> &'static str, default: fn() -> String) -> Setting<String> {
    Setting {
        default,
        path,
        possible_values: || &[],
        try_from_primitive: |primitive| match primitive {
            Primitive::String(value) => Ok(value),
            Primitive::Int(value) => Ok(value.to_string()),
            Primitive::Float(value) => Ok(value.to_string()),
            Primitive::Bool(value) => Ok(value.to_string()),
            Primitive::Array(_) | Primitive::Map(_) | Primitive::Null => {
                Err("cannot get string setting from null, array or map primitives".into())
            }
        },
        to_primitive: |value| Primitive::String(value.clone()),
    }
}
/// Construct a boolean setting with given default value.
pub const fn boolean(path: fn() -> &'static str, default: bool) -> Setting<bool> {
    Setting {
        default: if default { || true } else { || false },
        path,
        possible_values: || &[true, false],
        try_from_primitive: |primitive| match primitive {
            Primitive::Int(value) => Ok(value != 0),
            Primitive::Float(value) => Ok(!value.is_nan() && value != 0.0),
            Primitive::Bool(value) => Ok(value),
            Primitive::String(value) => {
                if value.eq_ignore_ascii_case("true")
                    || value.eq_ignore_ascii_case("yes")
                    || value.eq_ignore_ascii_case("t")
                    || value.eq_ignore_ascii_case("y")
                {
                    Ok(true)
                } else if value.eq_ignore_ascii_case("false")
                    || value.eq_ignore_ascii_case("no")
                    || value.eq_ignore_ascii_case("f")
                    || value.eq_ignore_ascii_case("n")
                {
                    Ok(false)
                } else {
                    Err(format!("could not parse string primitive {value:?} as a bool").into())
                }
            }
            Primitive::Array(_) | Primitive::Map(_) | Primitive::Null => {
                Err("cannot get boolean setting from null, array, or map primitives".into())
            }
        },
        to_primitive: |value| Primitive::Bool(*value),
    }
}
