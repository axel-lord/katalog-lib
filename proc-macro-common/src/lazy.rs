//! Lazy parsing of syntax.

use ::core::{
    cell::{Cell, OnceCell},
    fmt::Debug,
};

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::parse::{Parse, ParseStream, Parser};

/// A node which may be either parsed or not, Parse implementation
/// consumes all remaining tokens of buffer. As such initial parse never fails.
pub struct Lazy<T> {
    /// Unparsed content.
    unparsed: Cell<TokenStream>,
    /// Parsed content.
    parsed: OnceCell<T>,
}

impl<T> Default for Lazy<T> {
    /// Default value is an unparsed stream.
    fn default() -> Self {
        Self {
            unparsed: Default::default(),
            parsed: Default::default(),
        }
    }
}

impl<T> Clone for Lazy<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            unparsed: Cell::new(self.cloned_tokens()),
            parsed: self.parsed.clone(),
        }
    }
}

impl<T> Debug for Lazy<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        f.debug_struct("Lazy")
            .field("parsed", &self.parsed.get())
            .finish_non_exhaustive()
    }
}

impl<T> Lazy<T> {
    /// CLone tokenstream from cell.
    fn cloned_tokens(&self) -> TokenStream {
        let unparsed = self.unparsed.take();
        self.unparsed.set(unparsed.clone());
        unparsed
    }

    /// Check if lazy contains a parsed value.
    pub fn is_parsed(&self) -> bool {
        self.parsed().is_some()
    }

    /// Get parsed node if available.
    pub fn parsed(&self) -> Option<&T> {
        OnceCell::get(&self.parsed)
    }

    /// Get parsed node if available.
    pub fn parsed_mut(&mut self) -> Option<&mut T> {
        OnceCell::get_mut(&mut self.parsed)
    }

    /// Get unparsed tokens if available.
    pub fn unparsed(&mut self) -> Option<&mut TokenStream> {
        if self.parsed().is_none() {
            Some(Cell::get_mut(&mut self.unparsed))
        } else {
            None
        }
    }

    /// Get parsed inner value, attempting to parse it if needed.
    /// TokenStream is cloned before parsing and as such left in place should parse fail.
    ///
    /// # Errors
    /// If inner value needs to be parsed and parsing fails.
    pub fn try_as_parsed(&self) -> ::syn::Result<&T>
    where
        T: Parse,
    {
        self.try_as_parsed_with(T::parse)
    }

    /// Get parsed inner value, attempting to parse it using parse function if needed.
    /// TokenStream is cloned before parsing and as such left in place should parse fail.
    ///
    /// # Errors
    /// If inner value needs to be parsed and parsing fails.
    pub fn try_as_parsed_with(
        &self,
        parser: fn(ParseStream) -> ::syn::Result<T>,
    ) -> ::syn::Result<&T> {
        if let Some(parsed) = self.parsed() {
            Ok(parsed)
        } else {
            let tokens = self.cloned_tokens();
            let value = parser.parse2(tokens)?;
            Ok(self.parsed.get_or_init(move || value))
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
        if let Some(value) = self.parsed.into_inner() {
            Ok(value)
        } else {
            parser.parse2(self.unparsed.take())
        }
    }
}

impl<T> ToTokens for Lazy<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(parsed) = self.parsed() {
            parsed.to_tokens(tokens);
        } else {
            tokens.extend(self.cloned_tokens());
        }
    }
}

impl<T> Parse for Lazy<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            unparsed: Cell::new(input.parse()?),
            parsed: OnceCell::new(),
        })
    }
}
