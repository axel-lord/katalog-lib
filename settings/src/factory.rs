//! Setting factories.

use ::core::{borrow::Borrow, str::FromStr};

use crate::{Setting, StdSetting};

/// Construct a setting using implementations of traits
/// from the standard library.
pub const fn standard<'lt, R>(path: fn() -> &'static str) -> StdSetting<'lt, R>
where
    R: 'static + ToOwned + ?Sized,
    R::Owned: 'static + Default + FromStr + ToString,
{
    Setting {
        default: <R::Owned as Default>::default,
        to_ref: <R::Owned as Borrow<R>>::borrow,
        path,
        possible_values: || &[],
    }
}

/// Construct a setting for string values.
pub const fn string<'lt>(
    path: fn() -> &'static str,
    default: fn() -> String,
) -> StdSetting<'lt, str> {
    Setting {
        default,
        to_ref: String::borrow,
        path,
        possible_values: || &[],
    }
}

/// Construct a boolean setting with given default value.
pub const fn boolean(path: fn() -> &'static str, default: bool) -> Setting<'static, bool> {
    Setting {
        default: if default { || true } else { || false },
        to_ref: |b| *b,
        path,
        possible_values: || &[true, false],
    }
}
