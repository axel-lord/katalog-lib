//! [IntoExpr] impls.

use ::syn::{Block, Expr, Stmt};

use crate::extension::{IntoDelimiterSpan, IntoStatement};

/// Trait for items convertible into Expressions.
pub trait IntoExpression
where
    Self: Into<Expr>,
{
    /// Convert into an expression.
    fn into_expr(self) -> Expr {
        <Self as Into<Expr>>::into(self)
    }

    /// Convert directly into a statement.
    fn into_stmt(self) -> Stmt {
        IntoStatement::into_stmt(self.into_expr())
    }

    /// Convert directly into a block.
    fn into_block(self, delim_span: impl IntoDelimiterSpan) -> Block {
        IntoStatement::into_block(self.into_expr(), delim_span)
    }

    /// Convert directly into a block, using call site for delim span.
    fn into_block_call_site(self) -> Block {
        IntoStatement::into_block_call_site(self.into_expr())
    }
}

impl<T: Into<Expr>> IntoExpression for T {}
