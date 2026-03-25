//! [SettingValue] impl.

use ::core::fmt::Debug;

use crate::{
    Setting,
    spec::{RefOf, default_of},
};

/// A value of a setting.
#[derive(Default)]
pub enum SettingValue<'a, S: Setting> {
    /// Setting has a set value.
    Set(RefOf<'a, S::ValueSpec>),
    /// Setting is default value.
    #[default]
    Default,
}

impl<'a, S: Setting> Debug for SettingValue<'a, S>
where
    for<'b> RefOf<'b, S::ValueSpec>: Debug,
{
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
            Self::Set(arg0) => f.debug_tuple("Set").field(arg0).finish(),
            Self::Default => f
                .debug_tuple("Default")
                .field(&default_of::<RefOf<S::ValueSpec>, S::DefaultSpec>())
                .finish(),
        }
    }
}

impl<'a, S: Setting> Clone for SettingValue<'a, S> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, S: Setting> Copy for SettingValue<'a, S> {}
