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

    impl IntoStatement for Stmt {
        fn into_statement(self) -> Stmt {
            self
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
    use ::syn::{Block, Stmt, StmtMacro, Token, token};

    use crate::extension::IntoStatement;

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

    impl<I> IntoBlock for I
    where
        I: IntoIterator,
        I::Item: IntoStatement,
    {
        fn into_block(self, delim_span: DelimSpan, mut punct_span: impl FnMut() -> Span) -> Block {
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
                brace_token: token::Brace { span: delim_span },
                stmts,
            }
        }
    }
}
