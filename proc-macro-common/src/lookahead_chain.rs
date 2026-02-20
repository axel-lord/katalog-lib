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
    ) -> ::syn::Result<(Lookahead1<'i>, Self::Output<Option<T>>)>
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
    ) -> ::syn::Result<(Lookahead1<'i>, Self::Output<Option<T>>)>
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
    type Output<T> = (T,);

    fn chain_with<T, P>(
        self,
        input: ParseStream<'i>,
        peek: P,
        with: fn(ParseStream<'i>) -> syn::Result<T>,
    ) -> syn::Result<(Lookahead1<'i>, Self::Output<Option<T>>)>
    where
        P: Peek,
    {
        if self.peek(peek) {
            let value = input.call(with)?;

            Ok((input.lookahead1(), (Some(value),)))
        } else {
            Ok((self, (None,)))
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
            input.call(with).map(|value| (value,))
        } else {
            Err(self.error())
        }
    }
}

/// Macro to generate impls for tuples.
macro_rules! lookahead_chain_tuple_impl {
    (@impl $($v:ident)+ ) => {
        #[expect(non_snake_case)]
        impl<'i, $($v),*> LookaheadChain<'i> for (Lookahead1<'i>, ($($v,)*)) {
            type Output<T> = ($($v,)* T);

            fn chain_with<T, P>(
                self,
                input: ParseStream<'i>,
                peek: P,
                with: fn(ParseStream<'i>) -> syn::Result<T>,
            ) -> syn::Result<(Lookahead1<'i>, Self::Output<Option<T>>)>
            where
                P: Peek {
                let (lookahead, ( $($v,)*)) = self;
                let (lookahead, (value,)) = lookahead.chain_with(input, peek, with)?;
                Ok((lookahead, ($($v,)* value,)))
            }

            fn finish_with<T, P>(
                self,
                input: ParseStream<'i>,
                peek: P,
                with: fn(ParseStream<'i>) -> syn::Result<T>,
            ) -> syn::Result<Self::Output<T>>
            where
                P: Peek {
                let (lookahead, ( $($v,)*)) = self;
                let (value,) = lookahead.finish_with(input, peek, with)?;
                Ok(($($v,)* value,))
            }
        }
    };
    ($v:ident) => {
        lookahead_chain_tuple_impl!(@impl $v);
    };
    ($f:ident $(, $v:ident)* ) => {
        lookahead_chain_tuple_impl!(@impl $f $($v)*);
        lookahead_chain_tuple_impl!($($v),*);
    };
}

lookahead_chain_tuple_impl!(V1, V2, V3, V4, V5, V6, V7, V8, V9, V10, V11, V12);
