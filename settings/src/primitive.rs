//! [Primitive] impl.

use ::std::collections::BTreeMap;

/// Primitive settings type.
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub enum Primitive {
    /// Null value.
    #[default]
    Null,
    /// Integer value.
    Int(i64),
    /// Floating point value.
    Float(f64),
    /// Boolean value.
    Bool(bool),
    /// String value.
    String(String),
    /// Array of values.
    Array(Vec<Self>),
    /// Map of values.
    /// Avoid using where possible.
    Map(BTreeMap<String, Self>),
}
