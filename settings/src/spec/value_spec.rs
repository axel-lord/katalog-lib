//! [ValueSpec] impls.

use ::core::{borrow::Borrow, fmt::Debug, marker::PhantomData};

/// Spec for values which settings use.
pub trait Value {
    /// Setting value type.
    type Value;

    /// Reference to value, should be cheap.
    type Ref<'any>: Copy;

    /// Get a value ref from a reference to a value.
    fn to_ref<'any>(value: &'any Self::Value) -> Self::Ref<'any>;
}

/// [Value] implementor using a [ToOwned] implementation for
/// the types.
pub struct UseRefToOwned<R: ?Sized> {
    /// Make instances impossible and store type information.
    _p: (::core::convert::Infallible, PhantomData<fn() -> R>),
}

impl<R: ?Sized> Debug for UseRefToOwned<R> {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("UseRefToOwned")
            .field("_p", &self._p)
            .finish()
    }
}

impl<R: 'static + ?Sized + ToOwned> Value for UseRefToOwned<R> {
    type Value = R::Owned;

    type Ref<'any> = &'any R;

    fn to_ref<'any>(value: &'any Self::Value) -> Self::Ref<'any> {
        Borrow::borrow(value)
    }
}
