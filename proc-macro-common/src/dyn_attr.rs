//! More dynamic attribute parsing.

use ::syn::{MacroDelimiter, Token, parse::ParseStream, token};

use crate::delimited::MacroDelimited;

/// Attribute content which may be either delimited, or follow an '=' equals.
#[derive(Clone)]
pub enum DynAttr<T> {
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

impl<T> DynAttr<T> {
    /// Get value of attribute.
    pub const fn value(&self) -> &T {
        match self {
            DynAttr::Equals { value, .. } => value,
            DynAttr::Delimited(delimited) => &delimited.content,
        }
    }

    /// Get value of attribute as mutable.
    pub const fn value_mut(&mut self) -> &mut T {
        match self {
            DynAttr::Equals { value, .. } => value,
            DynAttr::Delimited(delimited) => &mut delimited.content,
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
        } else if lookahead.peek(token::Brace)
            || lookahead.peek(token::Bracket)
            || lookahead.peek(token::Paren)
        {
            Ok(Self::Delimited(MacroDelimited::parse_with(input, parser)?))
        } else {
            Err(lookahead.error())
        }
    }
}
