//! Utilities to chain lookahead calls.

use ::syn::parse::{Lookahead1, Parse, ParseStream, Peek};

/// Chain lookahead peek and parse actions.
pub trait LookaheadChain<'i>
where
    Self: Sized,
{
    /// Output of chain.
    type Output<T>;

    /// Parse T if P is peeked.
    ///
    /// If T is parsed a new lookahead is created. Otherwise the same
    /// one is returned.
    ///
    /// # Errors
    /// If peek returns true but T cannot be parsed.
    fn chain<T, P>(
        self,
        input: ParseStream<'i>,
        peek: P,
    ) -> ::syn::Result<(Self::Output<Option<T>>, Lookahead1<'i>)>
    where
        T: Parse,
        P: Peek,
    {
        self.chain_with(input, peek, T::parse)
    }

    /// Parse T using with if P is peeked.
    ///
    /// If T is parsed a new lookahead is created. Otherwise the same
    /// one is returned.
    ///
    /// # Errors
    /// If peek returns true but T cannot be parsed.
    fn chain_with<T, P>(
        self,
        input: ParseStream<'i>,
        peek: P,
        with: fn(ParseStream<'i>) -> ::syn::Result<T>,
    ) -> ::syn::Result<(Self::Output<Option<T>>, Lookahead1<'i>)>
    where
        P: Peek;

    /// Parse T if P is peeked, error if T cannot be parsed.
    ///
    /// # Errors
    /// If T cannot be parsed or peeked.
    fn finish<T, P>(self, input: ParseStream<'i>, peek: P) -> ::syn::Result<Self::Output<T>>
    where
        T: Parse,
        P: Peek,
    {
        self.finish_with(input, peek, T::parse)
    }

    /// Parse T using with if P is peeked, error if T cannot be parsed.
    ///
    /// # Errors
    /// If T cannot be parsed or peeked.
    fn finish_with<T, P>(
        self,
        input: ParseStream<'i>,
        peek: P,
        with: fn(ParseStream<'i>) -> ::syn::Result<T>,
    ) -> ::syn::Result<Self::Output<T>>
    where
        P: Peek;
}

impl<'i> LookaheadChain<'i> for Lookahead1<'i> {
    type Output<T> = T;

    fn chain_with<T, P>(
        self,
        input: ParseStream<'i>,
        peek: P,
        with: fn(ParseStream<'i>) -> syn::Result<T>,
    ) -> syn::Result<(Self::Output<Option<T>>, Lookahead1<'i>)>
    where
        P: Peek,
    {
        if self.peek(peek) {
            let value = input.call(with)?;

            Ok((Some(value), input.lookahead1()))
        } else {
            Ok((None, self))
        }
    }

    fn finish_with<T, P>(
        self,
        input: ParseStream<'i>,
        peek: P,
        with: fn(ParseStream<'i>) -> syn::Result<T>,
    ) -> syn::Result<Self::Output<T>>
    where
        P: Peek,
    {
        if self.peek(peek) {
            input.call(with)
        } else {
            Err(self.error())
        }
    }
}
