//! Closure like with a single parameter.

use ::katalog_lib_proc_macro_common::lookahead_chain::LookaheadChain;
use ::quote::ToTokens;
use ::syn::{Expr, Ident, Token, parse::Parse};

/// Closure like syntax node with only one parameter.
#[derive(Clone)]
pub struct MonoClosure {
    /// param delim '|'.
    pub left_pipe: Token![|],
    /// Parameter.
    pub param: Ident,
    /// Optional trailing comma
    pub trailing_comma: Option<Token![,]>,
    /// param delim '|'.
    pub right_pipe: Token![|],
    /// Expression of closure.
    pub expr: Expr,
}

impl Parse for MonoClosure {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let left_pipe = input.parse()?;
        let param = input.parse()?;

        let (trailing_comma, right_pipe) = input
            .lookahead1()
            .chain(input, Token![,])?
            .finish(input, Token![|])?;

        let expr = input.parse()?;

        Ok(Self {
            left_pipe,
            param,
            trailing_comma,
            right_pipe,
            expr,
        })
    }
}

impl ToTokens for MonoClosure {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            left_pipe,
            param,
            trailing_comma,
            right_pipe,
            expr,
        } = self;
        left_pipe.to_tokens(tokens);
        param.to_tokens(tokens);
        trailing_comma.to_tokens(tokens);
        right_pipe.to_tokens(tokens);
        expr.to_tokens(tokens);
    }
}
