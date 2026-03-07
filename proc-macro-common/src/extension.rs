//! Extension traits for easier writing of macro implementations.

pub use self::{
    into_block::IntoBlock, into_delim_span::IntoDelimiterSpan, into_expr::IntoExpression,
    into_item::IntoItem, into_pat::IntoPattern, into_path::IntoPath,
    into_punctuated::IntoPunctuated, into_stmt::IntoStatement,
};

pub mod prelude {
    //! Prelude containing extension traits as '_'.

    pub use super::{
        IntoBlock as _, IntoDelimiterSpan as _, IntoExpression as _, IntoItem as _, IntoPath as _,
        IntoPattern as _, IntoPunctuated as _, IntoStatement as _,
    };
}

mod into_block;
mod into_delim_span;
mod into_expr;
mod into_item;
mod into_pat;
mod into_path;
mod into_punctuated;
mod into_stmt;
