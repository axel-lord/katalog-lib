//! Parse wrapper for types which need to have no tokens following them.

use ::quote::ToTokens;
use ::syn::parse::{Parse, ParseStream};

/// Parse a syntax node expecting it to be the last node.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Last<T>(pub T);

impl<T> Last<T> {
    /// Parse value using provided parser.
    ///
    /// # Errors
    /// If parser errors.
    /// Or if any tokens remain in input after parse.
    pub fn parse_with(
        input: ParseStream,
        parser: fn(ParseStream) -> ::syn::Result<T>,
    ) -> ::syn::Result<Self> {
        let value = input.call(parser)?;

        if !input.is_empty() {
            return Err(input.error("no tokens expected"));
        }

        Ok(Self(value))
    }

    /// Parse then convert into inner value.
    ///
    /// # Errors
    /// If parser errors.
    /// Or if any tokens remain in input after parse.
    pub fn parse_value(input: ParseStream) -> ::syn::Result<T>
    where
        T: Parse,
    {
        Ok(Self::parse(input)?.into_inner())
    }

    /// Parse then convert into inner value using provided parser.
    ///
    /// # Errors
    /// If parser errors.
    /// Or if any tokens remain in input after parse.
    pub fn parse_value_with(
        input: ParseStream,
        parser: fn(ParseStream) -> ::syn::Result<T>,
    ) -> ::syn::Result<T> {
        Ok(Self::parse_with(input, parser)?.into_inner())
    }

    /// Convert into inner value.
    pub fn into_inner(self) -> T {
        let Self(value) = self;
        value
    }
}

impl<T> Parse for Last<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Self::parse_with(input, T::parse)
    }
}

impl<T> ToTokens for Last<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self(value) = self;
        value.to_tokens(tokens);
    }
}
