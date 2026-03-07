//! [IntoPath] impls.

use ::syn::{Expr, Pat, PatPath, Path};

use crate::extension::{IntoExpression, IntoPattern};

/// Convert a path into a pat path.
const fn into_pat_path(path: Path) -> PatPath {
    PatPath {
        attrs: Vec::new(),
        qself: None,
        path,
    }
}

/// Trait for items convertible into paths.
pub trait IntoPath
where
    Self: Into<Path>,
{
    /// Convert into a path.
    fn into_path(self) -> Path {
        <Self as Into<Path>>::into(self)
    }

    /// Convert directly into a pattern.
    fn into_pat(self) -> Pat {
        into_pat_path(self.into_path()).into_pat()
    }

    /// Convert directly into an expression.
    fn into_expr(self) -> Expr {
        into_pat_path(self.into_path()).into_expr()
    }
}

impl<T: Into<Path>> IntoPath for T {}
