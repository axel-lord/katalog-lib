//! [IntoPath] impls.

use ::syn::{Block, Expr, Pat, PatPath, Path, Stmt};

use crate::extension::{IntoDelimiterSpan, IntoExpression, IntoPattern};

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

    /// Convert directly into a statement.
    fn into_stmt(self) -> Stmt {
        self.into_expr().into_stmt()
    }

    /// Convert directly into a block.
    fn into_block(self, delim_span: impl IntoDelimiterSpan) -> Block {
        self.into_expr().into_block(delim_span)
    }

    /// Convert directly into a block, using call site for delim span.
    fn into_block_call_site(self) -> Block {
        self.into_expr().into_block_call_site()
    }
}

impl<T: Into<Path>> IntoPath for T {}
