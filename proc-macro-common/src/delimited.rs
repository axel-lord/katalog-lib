//! Extensions to macro delimiters.

use ::core::ops::{Deref, DerefMut};

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    MacroDelimiter, braced, bracketed, parenthesized,
    parse::{Lookahead1, Parse, ParseStream},
    token,
};

use crate::last::Last;

/// Peek a lookahead for any macro delimiter.
pub fn peek_lookahead(lookahead: &Lookahead1) -> bool {
    lookahead.peek(token::Brace) || lookahead.peek(token::Bracket) || lookahead.peek(token::Paren)
}

/// Node for macro delimited content.
#[derive(Clone)]
pub struct MacroDelimited<T> {
    /// Delimiter of content.
    pub delim: MacroDelimiter,
    /// Content to delimit.
    pub content: T,
}

impl<T> MacroDelimited<T> {
    /// Parse delimited content using provided parser.
    ///
    /// # Errors
    /// If no delimiter is parsed.
    /// Or if content cannot be parsed.
    /// Or if any tokens are left within delimiters after parse.
    pub fn parse_with(
        input: ParseStream,
        parser: fn(ParseStream) -> ::syn::Result<T>,
    ) -> ::syn::Result<Self> {
        let lookahead = input.lookahead1();
        let content;
        let delim = if lookahead.peek(token::Paren) {
            MacroDelimiter::Paren(parenthesized!(content in input))
        } else if lookahead.peek(token::Brace) {
            MacroDelimiter::Brace(braced!(content in input))
        } else if lookahead.peek(token::Bracket) {
            MacroDelimiter::Bracket(bracketed!(content in input))
        } else {
            return Err(lookahead.error());
        };
        let content = Last::<T>::parse_value_with(&content, parser)?;

        Ok(Self { delim, content })
    }
}

impl<T> Deref for MacroDelimited<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<T> DerefMut for MacroDelimited<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<T> Parse for MacroDelimited<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> ::syn::Result<Self> {
        Self::parse_with(input, T::parse)
    }
}

impl<T> ToTokens for MacroDelimited<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { delim, content } = self;
        let surround = |tokens: &mut TokenStream| content.to_tokens(tokens);
        match delim {
            MacroDelimiter::Paren(paren) => paren.surround(tokens, surround),
            MacroDelimiter::Brace(brace) => brace.surround(tokens, surround),
            MacroDelimiter::Bracket(bracket) => bracket.surround(tokens, surround),
        }
    }
}
