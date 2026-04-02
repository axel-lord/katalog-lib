//! Setting factories.

use ::core::{borrow::Borrow, str::FromStr};

use crate::{Primitive, RefSetting, SettingsError, StdSetting};

/// Construct a setting using implementations of traits
/// from the standard library.
pub const fn standard<R>(path: fn() -> &'static str) -> StdSetting<R>
where
    R: ToOwned + ?Sized,
    R::Owned: 'static + Default + FromStr + ToString,
{
    RefSetting {
        default: <R::Owned as Default>::default,
        to_ref: <R::Owned as Borrow<R>>::borrow,
        path,
        possible_values: || &[],
        try_from_primitive: |_| Err(SettingsError::unknown()),
        to_primitive: |_| Primitive::Null,
    }
}

/// Construct a setting for string values.
pub const fn string(
    path: fn() -> &'static str,
    default: fn() -> String,
) -> RefSetting<String, str> {
    RefSetting {
        default,
        to_ref: String::borrow,
        path,
        possible_values: || &[],
        try_from_primitive: |_| Err(SettingsError::unknown()),
        to_primitive: |_| Primitive::Null,
    }
}
/*
/// Construct a boolean setting with given default value.
pub const fn boolean(path: fn() -> &'static str, default: bool) -> RefSetting<bool> {
    RefSetting {
        default: if default { || true } else { || false },
        to_ref: |b| *b,
        path,
        possible_values: || &[true, false],
        try_from_primitive: |_| Err(SettingsError::unknown()),
        to_primitive: |_| Primitive::Null,
    }
}
*/
