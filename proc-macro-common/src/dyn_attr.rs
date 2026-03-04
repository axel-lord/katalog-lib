//! More dynamic attribute parsing.

use ::quote::ToTokens;
use ::syn::{
    Token,
    parse::{Lookahead1, Parse, ParseStream},
    token,
};

use crate::delimited::{self, MacroDelimited};

/// Attribute content which may be either delimited, or follow an '=' equals.
#[derive(Clone)]
pub enum DynAttrContent<T> {
    /// Content is of '=' kind.
    Equals {
        /// '=' token.
        eq_token: token::Eq,
        /// Value of content.
        value: T,
    },
    /// Content is delimited.
    Delimited(MacroDelimited<T>),
}

impl<T> Parse for DynAttrContent<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Self::parse_with(input, T::parse)
    }
}

impl<T> ToTokens for DynAttrContent<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            DynAttrContent::Equals { eq_token, value } => {
                eq_token.to_tokens(tokens);
                value.to_tokens(tokens);
            }
            DynAttrContent::Delimited(macro_delimited) => macro_delimited.to_tokens(tokens),
        }
    }
}

impl DynAttrContent<()> {
    /// Peek valid initial tokens from lookahead.
    pub fn peek_lookahead(lookahead: &Lookahead1) -> bool {
        lookahead.peek(Token![=]) | delimited::peek_lookahead(lookahead)
    }
}

impl<T> DynAttrContent<T> {
    /// Get value of attribute.
    pub const fn value(&self) -> &T {
        match self {
            DynAttrContent::Equals { value, .. } => value,
            DynAttrContent::Delimited(delimited) => &delimited.content,
        }
    }

    /// Get value of attribute as mutable.
    pub const fn value_mut(&mut self) -> &mut T {
        match self {
            DynAttrContent::Equals { value, .. } => value,
            DynAttrContent::Delimited(delimited) => &mut delimited.content,
        }
    }

    /// Parse using the provided parser for value.
    ///
    /// # Errors
    /// If parser errors.
    /// Or if delimiter/eq token cannot be parsed.
    pub fn parse_with(
        input: ParseStream,
        parser: fn(ParseStream) -> ::syn::Result<T>,
    ) -> ::syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Token![=]) {
            Ok(Self::Equals {
                eq_token: input.parse()?,
                value: input.call(parser)?,
            })
        } else if delimited::peek_lookahead(&lookahead) {
            Ok(Self::Delimited(MacroDelimited::parse_with(input, parser)?))
        } else {
            Err(lookahead.error())
        }
    }
}
