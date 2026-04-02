//! Settings provider library with the goal of allowing creation of settings across
//! a workspace without cross dependencies.

pub use self::{primitive::Primitive, ref_setting::RefSetting, setting::Setting};

#[doc(inline)]
pub use self::error::SettingsError;

pub mod error;
pub mod factory;
pub mod io;

#[cfg(feature = "cached")]
pub mod cached;

mod primitive;
mod ref_setting;
mod setting;

/// Type alias for settings which get their backing type
/// usinng [ToOwned].
pub type StdSetting<R> = RefSetting<<R as ToOwned>::Owned, R>;
