//! Extension traits for easier writing of macro implementations.

pub use self::{into_block::IntoBlock, into_statement::IntoStatement};

mod into_block;
mod into_statement;
