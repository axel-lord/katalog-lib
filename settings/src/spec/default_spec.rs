//! [DefaultSpec] impls.

use ::core::{fmt::Debug, marker::PhantomData};

/// Trait used for providing default value to a setting.
pub trait DefaultValue<V> {
    /// Get a reference to default value of setting.
    fn default_value() -> V;
}

/// [DefaultValue] implementor using the [Default] implementation
/// of the value reference.
pub struct UseValueDefault<V> {
    /// Make instances impossible and store type information.
    _p: (::core::convert::Infallible, PhantomData<fn() -> V>),
}

impl<V> Debug for UseValueDefault<V> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("UseValueDefault")
            .field("_p", &self._p)
            .finish()
    }
}

impl<V> DefaultValue<V> for UseValueDefault<V>
where
    V: Default,
{
    fn default_value() -> V {
        V::default()
    }
}

impl<V> DefaultValue<Option<V>> for Option<V> {
    fn default_value() -> Option<V> {
        None
    }
}

impl<V, E> DefaultValue<Result<V, E>> for Result<V, E>
where
    E: Default,
{
    fn default_value() -> Result<V, E> {
        Err(E::default())
    }
}
