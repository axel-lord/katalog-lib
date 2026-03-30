//! Settings provider library with the goal of allowing creation of settings across
//! a workspace without cross dependencies.

pub use self::{error::SettingsError, primitive::Primitive, setting::Setting};

mod error;
pub mod factory;
mod primitive;
mod setting;

/// Type alias for settings which get their backing type
/// usinng [ToOwned].
pub type StdSetting<'lt, R> = Setting<'lt, <R as ToOwned>::Owned, &'lt R>;

/// Settings reader, used to auqire settings values.
///
/// Type alias to a dynamic dispatch `Fn()` closure.
pub type SettingsReader<'lt> = &'lt dyn for<'a> Fn(&'a str) -> Result<Primitive, SettingsError>;

/// Settings writer, used when writing settings primitives.
///
/// Type alias to dynamic dispatch `FnMut()` closure which returnsA
/// true on successful writes and false otherwise.
pub type SettingsWriter<'lt> =
    &'lt mut dyn for<'a> FnMut(&'a str, Primitive) -> Result<(), SettingsError>;
