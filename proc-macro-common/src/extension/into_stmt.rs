//! [IntoStatement] impls.

use ::proc_macro2::Span;
use ::syn::{Block, Expr, Item, Local, Macro, Stmt, StmtMacro, token};

use crate::extension::IntoDelimiterSpan;

/// Convert valid items into statements.
pub trait IntoStatement {
    /// Convert value into a statement.
    fn into_stmt(self) -> Stmt;

    /// Convert value directly into a block.
    fn into_block(self, delim_span: impl IntoDelimiterSpan) -> Block
    where
        Self: Sized,
    {
        Block {
            brace_token: token::Brace {
                span: delim_span.into_delim_span(),
            },
            stmts: Vec::from_iter([self.into_stmt()]),
        }
    }

    /// Convert value directly into a block, using call site for delim span.
    fn into_block_call_site(self) -> Block
    where
        Self: Sized,
    {
        self.into_block(Span::call_site())
    }
}

impl IntoStatement for StmtMacro {
    fn into_stmt(self) -> Stmt {
        Stmt::Macro(self)
    }
}

impl IntoStatement for Macro {
    fn into_stmt(self) -> Stmt {
        Stmt::Macro(StmtMacro {
            attrs: Vec::new(),
            mac: self,
            semi_token: None,
        })
    }
}

impl IntoStatement for Local {
    fn into_stmt(self) -> Stmt {
        Stmt::Local(self)
    }
}

impl IntoStatement for Item {
    fn into_stmt(self) -> Stmt {
        Stmt::Item(self)
    }
}

impl IntoStatement for Expr {
    fn into_stmt(self) -> Stmt {
        Stmt::Expr(self, None)
    }
}

impl IntoStatement for Stmt {
    fn into_stmt(self) -> Stmt {
        self
    }
}

impl<T> IntoStatement for Box<T>
where
    T: IntoStatement,
{
    fn into_stmt(self) -> Stmt {
        (*self).into_stmt()
    }
}
