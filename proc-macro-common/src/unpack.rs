//! Utilities for unpacking tuples of size 1 to content.

use crate::unpack::sealed::Sealed;

mod sealed {
    #![doc(hidden)]

    #[doc(hidden)]
    pub trait Sealed {}
}

/// Unpack into Unpacked type.
pub trait Unpack: Sealed {
    /// Unpacking result.
    type Unpacked;

    /// Unpack value.
    fn unpack(self) -> Self::Unpacked;
}

impl<T> Sealed for (T,) {}
impl<T> Unpack for (T,) {
    type Unpacked = T;

    #[inline]
    fn unpack(self) -> Self::Unpacked {
        let (value,) = self;
        value
    }
}

impl<T> Sealed for Option<(T,)> {}
impl<T> Unpack for Option<(T,)> {
    type Unpacked = Option<T>;

    fn unpack(self) -> Self::Unpacked {
        if let Some((value,)) = self {
            Some(value)
        } else {
            None
        }
    }
}

impl<T, E> Sealed for Result<(T,), E> {}
impl<T, E> Unpack for Result<(T,), E> {
    type Unpacked = Result<T, E>;

    fn unpack(self) -> Self::Unpacked {
        match self {
            Ok((value,)) => Ok(value),
            Err(err) => Err(err),
        }
    }
}

