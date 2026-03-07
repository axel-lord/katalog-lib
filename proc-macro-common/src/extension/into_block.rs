//! [IntoBlock] impls.

use ::proc_macro2::Span;
use ::syn::{Block, Stmt, StmtMacro, Token, token};

use crate::extension::{IntoDelimiterSpan, IntoStatement};

/// Trait to convert interators of statements into blocks.
pub trait IntoBlock {
    /// Create a block using the given spans for delimiters and punctuation.
    fn into_block(
        self,
        delim_span: impl IntoDelimiterSpan,
        punct_span: impl FnMut() -> Span,
    ) -> Block;

    /// Create a block with call site spans for delimiters and punctuation.
    fn into_block_call_site(self) -> Block
    where
        Self: Sized,
    {
        self.into_block(Span::call_site(), Span::call_site)
    }
}

impl<I> IntoBlock for I
where
    I: IntoIterator,
    I::Item: IntoStatement,
{
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

            stmts.push(stmt.into_statement());
        }

        Block {
            brace_token: token::Brace {
                span: delim_span.into_delim_span(),
            },
            stmts,
        }
    }
}
