//! Settings provider library with the goal of allowing creation of settings across
//! a workspace without cross dependencies.

pub use self::{primitive::Primitive, setting::Setting};

pub mod factory;
mod primitive;
mod setting;

/// Type alias for settings which get their backing type
/// usinng [ToOwned].
pub type StdSetting<'lt, R> = Setting<'lt, <R as ToOwned>::Owned, &'lt R>;
