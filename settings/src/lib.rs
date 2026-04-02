//! Settings provider library with the goal of allowing creation of settings across
//! a workspace without cross dependencies.

pub use self::{primitive::Primitive, setting::Setting};

#[doc(inline)]
pub use self::error::SettingsError;

pub mod error;
pub mod factory;
pub mod io;

#[cfg(feature = "cached")]
pub mod cached;

mod primitive;
mod setting;
