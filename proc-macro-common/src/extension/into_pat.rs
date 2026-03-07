//! [IntoPat] impls.

use ::syn::Pat;

/// Trait for items convertible into patterns.
pub trait IntoPattern
where
    Self: Into<Pat>,
{
    /// Convert into a pattern.
    fn into_pat(self) -> Pat {
        <Self as Into<Pat>>::into(self)
    }
}

impl<T: Into<Pat>> IntoPattern for T {}
