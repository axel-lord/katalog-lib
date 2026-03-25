//! Settings handling and creation.

use ::std::borrow::Cow;

pub use crate::setting_value::SettingValue;
use crate::spec::{RefOf, default_of};

mod setting_value;
pub mod spec;

/// Spec for a Setting.
pub trait Setting {
    /// Setting value type.
    type ValueSpec: spec::Value;

    /// Default value provider.
    type DefaultSpec: for<'any> spec::DefaultValue<RefOf<'any, Self::ValueSpec>>;

    /// Provide the name of this setting.
    fn name<'any>() -> Cow<'any, str>;

    /// Get a reference to default value of setting.
    fn default_value<'any>() -> RefOf<'any, Self::ValueSpec> {
        default_of::<RefOf<Self::ValueSpec>, Self::DefaultSpec>()
    }
}
