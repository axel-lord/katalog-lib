//! [IntoPath] impls.

use ::syn::Path;

/// Trait for items convertible into paths.
pub trait IntoPath
where
    Self: Into<Path>,
{
    /// Convert into a path.
    fn into_path(self) -> Path {
        <Self as Into<Path>>::into(self)
    }
}

impl<T: Into<Path>> IntoPath for T {}
