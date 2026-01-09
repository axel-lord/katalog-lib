//! [StaticPath] impl.

use ::core::{fmt::Debug, str::Utf8Error};
use ::std::path::Path;

use ::iceoryx2::prelude::ZeroCopySend;
use ::iceoryx2_bb_container::vector::StaticVec;

/// Error raised when converting a [Path] to a [StaticPath].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ::thiserror::Error)]
pub enum FromPathError {
    /// Error returned when trying to create StaticPath from a
    /// path that is too long.
    #[error("cannot create StaticPath<{at_most}> from a path of length{len}")]
    TooLong {
        /// Longest length that would have been possible.
        at_most: usize,
        /// Length that was attempted.
        len: usize,
    },
    /// `Path` was not utf-8 on a windows platform.
    #[error("path is required to be utf-8 on windows")]
    NotUtf8,
}

/// Error raised when converting a [SaticPath] to a [Path].
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::thiserror::Error)]
pub enum IntoPathError {
    /// `StaticPath` was not utf-8 on a windows platform.
    #[error("path is required to be utf-8 on windows, {err}")]
    NotUtf8 {
        /// Wrapped utf8 error.
        #[from]
        err: Utf8Error,
    },
}

/// A static path with a lenght of at most N.
#[derive(Clone, ZeroCopySend)]
#[repr(C)]
pub struct StaticPath<const N: usize> {
    /// Byte data of path.
    data: StaticVec<u8, N>,
}

impl<const N: usize> StaticPath<N> {
    /// Attempt ot get a path reference from a static path.
    ///
    /// # Errors
    /// On windows if the stored path is not utf-8.
    #[cfg_attr(
        target_family = "unix",
        expect(clippy::unnecessary_fallible_conversions)
    )]
    pub fn try_into_path(&self) -> Result<&Path, <&Path as TryFrom<&Self>>::Error> {
        self.try_into()
    }
}

#[cfg(target_family = "windows")]
impl<'a, const N: usize> TryFrom<&'a StaticPath<N>> for &'a Path {
    type Error = IntoPathError;

    fn try_from(value: &'a StaticPath<N>) -> Result<Self, Self::Error> {
        str::from_utf8(&value.data)
            .map_err(From::from)
            .map(Path::new)
    }
}

#[cfg(target_family = "unix")]
impl<'a, const N: usize> From<&'a StaticPath<N>> for &'a Path {
    fn from(value: &'a StaticPath<N>) -> Self {
        use ::std::{ffi::OsStr, os::unix::ffi::OsStrExt};
        Path::new(OsStr::from_bytes(&value.data))
    }
}

impl<const N: usize> TryFrom<&Path> for StaticPath<N> {
    type Error = FromPathError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        #[cfg(target_family = "windows")]
        let bytes = value.to_str().ok_or(FromPathError::NotUtf8)?.as_bytes();

        #[cfg(target_family = "unix")]
        let bytes = {
            use ::std::os::unix::ffi::OsStrExt;
            value.as_os_str().as_bytes()
        };

        StaticVec::try_from(bytes)
            .map(|data| Self { data })
            .map_err(|_| FromPathError::TooLong {
                at_most: N,
                len: bytes.len(),
            })
    }
}

impl<const N: usize> Debug for StaticPath<N> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.write_str("\"")?;

        for chunk in self.data.utf8_chunks() {
            f.write_str(chunk.valid())?;

            for byte in chunk.invalid() {
                write!(f, "\\x{byte:02X}")?;
            }
        }

        f.write_str("\"")?;
        Ok(())
    }
}
