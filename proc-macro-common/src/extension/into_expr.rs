//! [IntoExpr] impls.

use ::syn::Expr;

/// Trait for items convertible into Expressions.
pub trait IntoExpression
where
    Self: Into<Expr>,
{
    /// Convert into an expression.
    fn into_expr(self) -> Expr {
        <Self as Into<Expr>>::into(self)
    }
}

impl<T: Into<Expr>> IntoExpression for T {}
