//! Settings provider library with the goal of allowing creation of settings across
//! a workspace without cross dependencies.

use ::core::{any::type_name, borrow::Borrow, fmt::Debug, hash::Hash};

pub mod factory {
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
}

/// Type alias for settings which get their backing type
/// usinng [ToOwned].
pub type StdSetting<'lt, R> = Setting<'lt, <R as ToOwned>::Owned, &'lt R>;

/// Key to resolve a setting.
pub struct Setting<'lt, T: 'static, R: 'lt = T> {
    /// Default value of setting, used when not set.
    pub default: fn() -> T,

    /// Get cheap reference like type.
    ///
    /// Allows for situations such as
    /// copying/cloning where cheap.
    pub to_ref: fn(&'lt T) -> R,

    /// Path of setting in store.
    ///
    /// Different stores are allowed to structure
    /// content in different ways, as such no
    /// guarantees can be made of how the name
    /// relates to the setting name in the store.
    ///
    /// Any equality, ordering and/or hasing is performed on the path
    /// as such two settings may have different types, but as long as
    /// they have the same path they are considered equal.
    pub path: fn() -> &'static str,

    /// Aquire possible values for setting, if empty values are not
    /// restricted by set.
    pub possible_values: fn() -> &'static [T],
}

impl<'lt, T: 'static, R> Setting<'lt, T, R> {
    /// Get default value of setting.
    pub fn default(self) -> T {
        let Self { default, .. } = self;
        default()
    }

    /// Convert a reference to value
    /// to an instance of `R`.
    pub fn to_ref(self, value: &'lt T) -> R {
        let Self { to_ref, .. } = self;
        to_ref(value)
    }

    /// Get setting path.
    pub fn path(self) -> &'static str {
        let Self { path, .. } = self;
        path()
    }

    /// Get possible values of setting.
    pub fn possible_values(self) -> &'static [T] {
        let Self {
            possible_values, ..
        } = self;
        possible_values()
    }
}

impl<T: 'static, R> Borrow<str> for Setting<'_, T, R> {
    fn borrow(&self) -> &str {
        self.path()
    }
}

impl<T: 'static, R> AsRef<str> for Setting<'_, T, R> {
    fn as_ref(&self) -> &str {
        self.path()
    }
}

impl<T: 'static, R> Eq for Setting<'_, T, R> {}

impl<T: 'static, R, S: AsRef<str>> PartialEq<S> for Setting<'_, T, R> {
    fn eq(&self, other: &S) -> bool {
        self.path().eq(other.as_ref())
    }
}

impl<T: 'static, R, S: AsRef<str>> PartialOrd<S> for Setting<'_, T, R> {
    fn partial_cmp(&self, other: &S) -> Option<::core::cmp::Ordering> {
        Some(self.path().cmp(other.as_ref()))
    }
}

impl<T: 'static, R> Ord for Setting<'_, T, R> {
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        self.path().cmp(other.path())
    }
}

impl<T: 'static, R> Hash for Setting<'_, T, R> {
    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
        self.path().hash(state);
    }
}

impl<T: 'static, R> Clone for Setting<'_, T, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static, R> Copy for Setting<'_, T, R> {}

impl<T: 'static, R> Debug for Setting<'_, T, R> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Setting")
            .field("T", &type_name::<T>())
            .field("R", &type_name::<R>())
            .field("path", &(self.path)())
            .finish()
    }
}
