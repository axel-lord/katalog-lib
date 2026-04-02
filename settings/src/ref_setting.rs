//! [RefSetting] impl.
use ::core::{any::type_name, borrow::Borrow, fmt::Debug, hash::Hash, ops::Deref};

use crate::Setting;

/// Key to resolve a setting.
pub struct RefSetting<T: 'static, R: ?Sized> {
    /// Parent setting.
    pub setting: Setting<T>,

    /// Get cheap reference like type.
    ///
    /// Allows for situations such as
    /// copying/cloning where cheap.
    pub to_ref: for<'a> fn(&'a T) -> &'a R,
}

impl<T: 'static, R: ?Sized> RefSetting<T, R> {
    /// Convert a reference to value
    /// to an instance of `R`.
    pub fn to_ref(self, value: &T) -> &R {
        let Self { to_ref, .. } = self;
        to_ref(value)
    }
}

impl<T: 'static, R: ?Sized> AsRef<Setting<T>> for RefSetting<T, R> {
    fn as_ref(&self) -> &Setting<T> {
        self
    }
}

impl<T: 'static, R: ?Sized> From<RefSetting<T, R>> for Setting<T> {
    fn from(value: RefSetting<T, R>) -> Self {
        value.setting
    }
}

impl<T: 'static, R: ?Sized> Deref for RefSetting<T, R> {
    type Target = Setting<T>;

    fn deref(&self) -> &Self::Target {
        &self.setting
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
            .field("R", &type_name::<R>())
            .field("setting", &self.setting)
            .finish()
    }
}
