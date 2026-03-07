//! [IntoBlock] impls.

use ::proc_macro2::Span;
use ::syn::{Block, Expr, ExprBlock, Stmt, StmtMacro, Token, token};

use crate::extension::{IntoDelimiterSpan, IntoExpression, IntoStatement};

/// Trait to convert interators of statements into blocks.
pub trait IntoBlock
where
    Self: IntoIterator + Sized,
    Self::Item: IntoStatement,
{
    /// Create a block using the given spans for delimiters and punctuation.
    fn into_block(
        self,
        delim_span: impl IntoDelimiterSpan,
        mut punct_span: impl FnMut() -> Span,
    ) -> Block {
        let mut stmts = Vec::new();

        for stmt in self {
            if let Some(
                Stmt::Expr(_, semi_token @ None)
                | Stmt::Macro(StmtMacro {
                    semi_token: semi_token @ None,
                    ..
                }),
            ) = stmts.last_mut()
            {
                *semi_token = Some(Token![;](punct_span()))
            }

            stmts.push(stmt.into_stmt());
        }

        Block {
            brace_token: token::Brace {
                span: delim_span.into_delim_span(),
            },
            stmts,
        }
    }

    /// Create a block with call site spans for delimiters and punctuation.
    fn into_block_call_site(self) -> Block {
        self.into_block(Span::call_site(), Span::call_site)
    }

    /// Create a block using spans same as [IntoBlock::into_block], then convert into a block
    /// expression.
    fn into_block_expr(
        self,
        delim_span: impl IntoDelimiterSpan,
        punct_span: impl FnMut() -> Span,
    ) -> Expr {
        ExprBlock {
            attrs: Vec::new(),
            label: None,
            block: self.into_block(delim_span, punct_span),
        }
        .into_expr()
    }

    /// Create a block using call site spans same as [IntoBlock::into_block_call_site], then convert into a block
    /// expression.
    fn into_block_expr_call_site(self) -> Expr {
        self.into_block_expr(Span::call_site(), Span::call_site)
    }
}

impl<I> IntoBlock for I
where
    I: IntoIterator,
    I::Item: IntoStatement,
{
}
