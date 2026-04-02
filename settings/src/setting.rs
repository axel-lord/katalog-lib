//! [Setting] impl.
use ::core::{any::type_name, borrow::Borrow, fmt::Debug, hash::Hash};

use crate::{Primitive, SettingsError};

/// Description of a setting.
pub struct Setting<T: 'static> {
    /// Default value of setting, used when not set.
    pub default: fn() -> T,

    /// Aquire possible values for setting, if empty values are not
    /// restricted by set.
    pub possible_values: fn() -> &'static [T],

    /// Convert from settings primitives to this setting.
    pub try_from_primitive: fn(Primitive) -> Result<T, SettingsError>,

    /// Convert to settings primitive.
    pub to_primitive: fn(&T) -> Primitive,

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
}

impl<T: 'static> Setting<T> {
    /// Get default value of setting.
    pub fn default(self) -> T {
        let Self { default, .. } = self;
        default()
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
    pub fn to_primitive(self, value: &T) -> Primitive {
        let Self { to_primitive, .. } = self;
        to_primitive(value)
    }
}

impl<T: 'static, S: AsRef<str>> PartialEq<S> for Setting<T> {
    fn eq(&self, other: &S) -> bool {
        self.path().eq(other.as_ref())
    }
}

impl<T: 'static, S: AsRef<str>> PartialOrd<S> for Setting<T> {
    fn partial_cmp(&self, other: &S) -> Option<::core::cmp::Ordering> {
        Some(self.path().cmp(other.as_ref()))
    }
}

impl<T: 'static> Eq for Setting<T> {}

impl<T: 'static> Ord for Setting<T> {
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        self.path().cmp(other.path())
    }
}

impl<T: 'static> Hash for Setting<T> {
    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
        self.path().hash(state);
    }
}

impl<T: 'static> Borrow<str> for Setting<T> {
    fn borrow(&self) -> &str {
        self.path()
    }
}

impl<T: 'static> AsRef<str> for Setting<T> {
    fn as_ref(&self) -> &str {
        self.path()
    }
}

impl<T: 'static> AsRef<Setting<T>> for Setting<T> {
    fn as_ref(&self) -> &Setting<T> {
        self
    }
}

impl<T: 'static> Debug for Setting<T> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Setting")
            .field("T", &type_name::<T>())
            .field("path", &(self.path)())
            .finish()
    }
}

impl<T: 'static> Copy for Setting<T> {}

impl<T: 'static> Clone for Setting<T> {
    fn clone(&self) -> Self {
        *self
    }
}
