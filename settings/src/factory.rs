//! Setting factories.

use ::core::{borrow::Borrow, str::FromStr};

use crate::{Primitive, Setting, SettingsError, StdSetting};

/// Construct a setting using implementations of traits
/// from the standard library.
pub const fn standard<R>(path: fn() -> &'static str) -> StdSetting<R>
where
    R: ToOwned + ?Sized,
    R::Owned: 'static + Default + FromStr + ToString,
{
    StdSetting {
        setting: Setting {
            default: <R::Owned as Default>::default,
            path,
            possible_values: || &[],
            try_from_primitive: |_| Err(SettingsError::unknown()),
            to_primitive: |_| Primitive::Null,
        },
        to_ref: <R::Owned as Borrow<R>>::borrow,
    }
}

/// Construct a setting for string values.
pub const fn string(path: fn() -> &'static str, default: fn() -> String) -> StdSetting<str> {
    StdSetting {
        setting: Setting {
            default,
            path,
            possible_values: || &[],
            try_from_primitive: |_| Err(SettingsError::unknown()),
            to_primitive: |_| Primitive::Null,
        },
        to_ref: String::borrow,
    }
}
/// Construct a boolean setting with given default value.
pub const fn boolean(path: fn() -> &'static str, default: bool) -> Setting<bool> {
    Setting {
        default: if default { || true } else { || false },
        path,
        possible_values: || &[true, false],
        try_from_primitive: |_| Err(SettingsError::unknown()),
        to_primitive: |_| Primitive::Null,
    }
}
