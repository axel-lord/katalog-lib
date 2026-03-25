//! Specifications used for setting.

pub use self::{
    default_spec::{DefaultValue, UseValueDefault},
    value_spec::{UseRefToOwned, Value},
};

/// Extract value type from a [spec::Value][Value].
pub type ValueOf<V> = <V as Value>::Value;

/// Extract ref type from a [spec::Value][Value].
pub type RefOf<'any, V> = <V as Value>::Ref<'any>;

/// Get default value from a [spec::Default][Default].
pub fn default_of<V, D: crate::spec::DefaultValue<V>>() -> V {
    D::default_value()
}

mod default_spec;
mod value_spec;
