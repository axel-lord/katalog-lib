//! Lazy parsing of syntax.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::parse::{Parse, ParseStream, Parser};

/// A node which may be either parsed or not, Parse implementation
/// consumes all remaining tokens of buffer. As such initial parse never fails.
#[derive(Debug, Clone)]
pub enum Lazy<T> {
    /// Node is not parsed yet.
    Unparsed(TokenStream),
    /// Node has been parsed.
    Parsed(T),
}

impl<T> Lazy<T> {
    /// Get parsed node if available.
    pub const fn parsed(&self) -> Option<&T> {
        if let Self::Parsed(parsed) = self {
            Some(parsed)
        } else {
            None
        }
    }

    /// Get parsed node if available.
    pub const fn parsed_mut(&mut self) -> Option<&mut T> {
        if let Self::Parsed(parsed) = self {
            Some(parsed)
        } else {
            None
        }
    }

    /// Get unparsed tokens if available.
    pub const fn unparsed(&self) -> Option<&TokenStream> {
        if let Self::Unparsed(tokens) = self {
            Some(tokens)
        } else {
            None
        }
    }

    /// Get unparsed tokens if available.
    pub const fn unparsed_mut(&mut self) -> Option<&mut TokenStream> {
        if let Self::Unparsed(tokens) = self {
            Some(tokens)
        } else {
            None
        }
    }

    /// Get parsed inner value, attempting to parse it if needed.
    /// Token stream will not be changed if parsing fails.
    ///
    /// # Errors
    /// If inner value needs to be parsed and parsing fails.
    pub fn try_as_parsed(&mut self) -> ::syn::Result<&mut T>
    where
        T: Parse,
    {
        self.try_as_parsed_with(T::parse)
    }

    /// Get parsed inner value, attempting to parse it using parse function if needed.
    /// Token stream will not be changed if parsing fails.
    ///
    /// # Errors
    /// If inner value needs to be parsed and parsing fails.
    pub fn try_as_parsed_with(
        &mut self,
        parser: fn(ParseStream) -> ::syn::Result<T>,
    ) -> ::syn::Result<&mut T> {
        match self {
            Lazy::Unparsed(token_stream) => {
                let value = parser.parse2(token_stream.clone())?;
                *self = Lazy::Parsed(value);

                // Will alwas go along parsed arm.
                self.try_as_parsed_with(parser)
            }
            Lazy::Parsed(value) => Ok(value),
        }
    }

    /// Unwrap into inner value if parsed otherwise parse it using parse implementation.
    ///
    /// # Errors
    /// If parse is needed an fails.
    pub fn try_into_parsed(self) -> ::syn::Result<T>
    where
        T: Parse,
    {
        self.try_into_parsed_with(T::parse)
    }

    /// Unwrap into inner value if parsed otherwise parse it using parser.
    ///
    /// # Errors
    /// If parse is needed an fails.
    pub fn try_into_parsed_with(
        self,
        parser: fn(ParseStream) -> ::syn::Result<T>,
    ) -> ::syn::Result<T> {
        match self {
            Lazy::Unparsed(token_stream) => parser.parse2(token_stream),
            Lazy::Parsed(value) => Ok(value),
        }
    }
}

impl<T> ToTokens for Lazy<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Lazy::Unparsed(token_stream) => token_stream.to_tokens(tokens),
            Lazy::Parsed(node) => node.to_tokens(tokens),
        }
    }
}

impl<T> Parse for Lazy<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self::Unparsed(input.parse()?))
    }
}

impl<T> Default for Lazy<T> {
    /// Default is an empty unparsed stream.
    fn default() -> Self {
        Self::Unparsed(TokenStream::default())
    }
}
