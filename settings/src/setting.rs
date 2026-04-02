//! [Setting] impl.
use ::core::{any::type_name, borrow::Borrow, fmt::Debug, hash::Hash};

use crate::{Primitive, SettingsError};

/// Key to resolve a setting.
pub struct RefSetting<T: 'static, R: ?Sized> {
    /// Default value of setting, used when not set.
    pub default: fn() -> T,

    /// Get cheap reference like type.
    ///
    /// Allows for situations such as
    /// copying/cloning where cheap.
    pub to_ref: for<'a> fn(&'a T) -> &'a R,

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

    /// Convert from settings primitives to this setting.
    pub try_from_primitive: fn(Primitive) -> Result<T, SettingsError>,

    /// Convert to settings primitive.
    pub to_primitive: fn(&R) -> Primitive,
}

impl<T: 'static, R: ?Sized> RefSetting<T, R> {
    /// Get default value of setting.
    pub fn default(self) -> T {
        let Self { default, .. } = self;
        default()
    }

    /// Convert a reference to value
    /// to an instance of `R`.
    pub fn to_ref(self, value: &T) -> &R {
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

    /// Convert a primitive settings value to this setting.
    ///
    /// # Errors
    /// If given primitive cannot be converted to this setting.
    pub fn try_from_primitive(self, primitive: Primitive) -> Result<T, SettingsError> {
        let Self {
            try_from_primitive, ..
        } = self;
        try_from_primitive(primitive)
    }

    /// Convert to a primitive setting.
    pub fn to_primitive(self, value: &R) -> Primitive {
        let Self { to_primitive, .. } = self;
        to_primitive(value)
    }
}

impl<T: 'static, R: ?Sized> Borrow<str> for RefSetting<T, R> {
    fn borrow(&self) -> &str {
        self.path()
    }
}

impl<T: 'static, R: ?Sized> AsRef<str> for RefSetting<T, R> {
    fn as_ref(&self) -> &str {
        self.path()
    }
}

impl<T: 'static, R: ?Sized> Eq for RefSetting<T, R> {}

impl<T: 'static, R: ?Sized, S: AsRef<str>> PartialEq<S> for RefSetting<T, R> {
    fn eq(&self, other: &S) -> bool {
        self.path().eq(other.as_ref())
    }
}

impl<T: 'static, R: ?Sized, S: AsRef<str>> PartialOrd<S> for RefSetting<T, R> {
    fn partial_cmp(&self, other: &S) -> Option<::core::cmp::Ordering> {
        Some(self.path().cmp(other.as_ref()))
    }
}

impl<T: 'static, R: ?Sized> Ord for RefSetting<T, R> {
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        self.path().cmp(other.path())
    }
}

impl<T: 'static, R: ?Sized> Hash for RefSetting<T, R> {
    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
        self.path().hash(state);
    }
}

impl<T: 'static, R: ?Sized> Clone for RefSetting<T, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static, R: ?Sized> Copy for RefSetting<T, R> {}

impl<T: 'static, R: ?Sized> Debug for RefSetting<T, R> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Setting")
            .field("T", &type_name::<T>())
            .field("R", &type_name::<R>())
            .field("path", &(self.path)())
            .finish()
    }
}
