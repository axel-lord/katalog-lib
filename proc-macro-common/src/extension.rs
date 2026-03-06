//! Extension traits for easier writing of macro implementations.

pub use self::{into_block::IntoBlock, into_statement::IntoStatement};

mod into_statement {
    //! [IntoStatement] impls

    use ::syn::{Expr, Item, Local, Macro, Stmt, StmtMacro};

    /// Convert valid items into statements.
    pub trait IntoStatement {
        /// Convert value into a statement.
        fn into_statement(self) -> Stmt;
    }

    impl IntoStatement for StmtMacro {
        fn into_statement(self) -> Stmt {
            Stmt::Macro(self)
        }
    }

    impl IntoStatement for Macro {
        fn into_statement(self) -> Stmt {
            Stmt::Macro(StmtMacro {
                attrs: Vec::new(),
                mac: self,
                semi_token: None,
            })
        }
    }

    impl IntoStatement for Local {
        fn into_statement(self) -> Stmt {
            Stmt::Local(self)
        }
    }

    impl IntoStatement for Item {
        fn into_statement(self) -> Stmt {
            Stmt::Item(self)
        }
    }

    impl IntoStatement for Expr {
        fn into_statement(self) -> Stmt {
            Stmt::Expr(self, None)
        }
    }

    impl<T> IntoStatement for Box<T>
    where
        T: IntoStatement,
    {
        fn into_statement(self) -> Stmt {
            (*self).into_statement()
        }
    }
}

mod into_block {
    //! [IntoBlock] impls.

    use ::proc_macro2::{Span, extra::DelimSpan};
    use ::syn::{Block, token};

    /// Trait to convert interators of statements into blocks.
    pub trait IntoBlock {
        /// Create a block using the given spans for delimiters and punctuation.
        fn into_block(self, delim_span: DelimSpan, punct_span: impl FnMut() -> Span) -> Block;

        /// Create a block with call site spans for delimiters and punctuation.
        fn into_block_call_site(self) -> Block
        where
            Self: Sized,
        {
            self.into_block(token::Brace(Span::call_site()).span, Span::call_site)
        }
    }
}
