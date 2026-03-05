//! Utilities to chain lookahead calls.

use ::syn::parse::{Lookahead1, Parse, ParseStream, Peek};

use crate::lookahead_chain::sealed::Sealed;

#[doc(hidden)]
mod sealed {
    #[doc(hidden)]
    pub trait Sealed {}
}

/// Chain lookahead peek and parse actions.
pub trait LookaheadChain<'i>
where
    Self: Sized + Sealed,
{
    /// Output of chain.
    type Output<T>;

    #[doc(hidden)]
    fn map_output<A, B, M>(output: Self::Output<A>, map: M) -> ::syn::Result<Self::Output<B>>
    where
        M: FnOnce(A) -> ::syn::Result<B>;

    /// Parse T if P is peeked.
    ///
    /// If T is parsed a new lookahead is created. Otherwise the same
    /// one is returned.
    ///
    /// # Errors
    /// If peek returns true but T cannot be parsed.
    fn chain<T>(
        self,
        input: ParseStream<'i>,
        peek: impl Peek,
    ) -> ::syn::Result<(Lookahead1<'i>, Self::Output<Option<T>>)>
    where
        T: Parse,
    {
        self.chain_with(input, peek, T::parse)
    }

    /// Parse T if P is peeked and condition returns true.
    ///
    /// If T is parsed a new lookahead is created. Otherwise the same
    /// one is returned.
    ///
    /// # Errors
    /// If peek returns true but T cannot be parsed.
    fn chain_and<T>(
        self,
        input: ParseStream<'i>,
        peek: impl Peek,
        condition: fn(ParseStream<'i>) -> bool,
    ) -> ::syn::Result<(Lookahead1<'i>, Self::Output<Option<T>>)>
    where
        T: Parse,
    {
        self.chain_with_and(input, peek, T::parse, condition)
    }

    /// Parse T if P is peeked.
    ///
    /// If T is parsed a new lookahead is created. Otherwise the same
    /// one is returned.
    ///
    /// If P is not peeked [Default::default] for T is returned.
    ///
    /// # Errors
    /// If peek returns true but T cannot be parsed.
    fn chain_or_default<T>(
        self,
        input: ParseStream<'i>,
        peek: impl Peek,
    ) -> ::syn::Result<(Lookahead1<'i>, Self::Output<T>)>
    where
        T: Parse + Default,
    {
        self.chain_with_or(input, peek, T::parse, T::default)
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
        P: Peek,
    {
        self.chain_with_and(input, peek, with, |_| true)
    }

    /// Parse T using with if P is peeked amd the condition returns true.
    ///
    /// If T is parsed a new lookahead is created. Otherwise the same
    /// one is returned.
    ///
    /// # Errors
    /// If peek returns true but T cannot be parsed.
    fn chain_with_and<T, P>(
        self,
        input: ParseStream<'i>,
        peek: P,
        with: fn(ParseStream<'i>) -> ::syn::Result<T>,
        condition: fn(ParseStream<'i>) -> bool,
    ) -> ::syn::Result<(Lookahead1<'i>, Self::Output<Option<T>>)>
    where
        P: Peek;

    /// Parse T using with if P is peeked.
    ///
    /// If T is parsed a new lookahead is created. Otherwise the same
    /// one is returned.
    ///
    /// `or` is used to get value of T if not parsed.
    ///
    /// # Errors
    /// If peek returns true but T cannot be parsed.
    fn chain_with_or<T, P>(
        self,
        input: ParseStream<'i>,
        peek: P,
        with: fn(ParseStream<'i>) -> ::syn::Result<T>,
        or: fn() -> T,
    ) -> ::syn::Result<(Lookahead1<'i>, Self::Output<T>)>
    where
        P: Peek,
    {
        let (lookahead, output) = self.chain_with(input, peek, with)?;
        let output = Self::map_output(output, |value| Ok(value.unwrap_or_else(or)))?;
        Ok((lookahead, output))
    }

    /// Parse T using with if P is peeked.
    ///
    /// If T is parsed a new lookahead is created. Otherwise the same
    /// one is returned.
    ///
    /// If P is not peeked [Default::default] for T is returned.
    ///
    /// # Errors
    /// If peek returns true but T cannot be parsed.
    fn chain_with_or_default<T, P>(
        self,
        input: ParseStream<'i>,
        peek: P,
        with: fn(ParseStream<'i>) -> ::syn::Result<T>,
    ) -> ::syn::Result<(Lookahead1<'i>, Self::Output<T>)>
    where
        P: Peek,
        T: Default,
    {
        self.chain_with_or(input, peek, with, T::default)
    }

    /// Parse T if P is peeked, error if T cannot be parsed.
    ///
    /// # Errors
    /// If T cannot be parsed or peeked.
    fn finish<T>(self, input: ParseStream<'i>, peek: impl Peek) -> ::syn::Result<Self::Output<T>>
    where
        T: Parse,
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
        P: Peek,
    {
        let (lookahead, value) = self.chain_with(input, peek, with)?;
        Self::map_output(value, |value| value.ok_or_else(|| lookahead.error()))
    }
}

impl<'i> Sealed for Lookahead1<'i> {}

impl<'i> LookaheadChain<'i> for Lookahead1<'i> {
    type Output<T> = (T,);

    #[doc(hidden)]
    fn map_output<A, B, M>(output: Self::Output<A>, map: M) -> ::syn::Result<Self::Output<B>>
    where
        M: FnOnce(A) -> ::syn::Result<B>,
    {
        let (value,) = output;
        Ok((map(value)?,))
    }

    fn chain_with_and<T, P>(
        self,
        input: ParseStream<'i>,
        peek: P,
        with: fn(ParseStream<'i>) -> syn::Result<T>,
        condition: fn(ParseStream<'i>) -> bool,
    ) -> syn::Result<(Lookahead1<'i>, Self::Output<Option<T>>)>
    where
        P: Peek,
    {
        if self.peek(peek) && condition(input) {
            let value = input.call(with)?;

            Ok((input.lookahead1(), (Some(value),)))
        } else {
            Ok((self, (None,)))
        }
    }
}

/// Macro to generate impls for tuples.
macro_rules! lookahead_chain_tuple_impl {
    (@impl $($v:ident)+ ) => {
        impl<'i, $($v),*> Sealed for (Lookahead1<'i>, ($($v,)*)) {}

        #[expect(non_snake_case)]
        impl<'i, $($v),*> LookaheadChain<'i> for (Lookahead1<'i>, ($($v,)*)) {
            type Output<T> = ($($v,)* T);

            #[doc(hidden)]
            fn map_output<A, B, M>(output: Self::Output<A>, map: M) -> ::syn::Result<Self::Output<B>>
            where
                M: FnOnce(A) -> ::syn::Result<B>
            {
                let ($($v,)* value,) = output;
                Ok(($($v,)* map(value)?,))
            }

            fn chain_with_and<T, P>(
                self,
                input: ParseStream<'i>,
                peek: P,
                with: fn(ParseStream<'i>) -> syn::Result<T>,
                condition: fn(ParseStream<'i>) -> bool,
            ) -> syn::Result<(Lookahead1<'i>, Self::Output<Option<T>>)>
            where
                P: Peek {
                let (lookahead, ( $($v,)*)) = self;
                let (lookahead, (value,)) = lookahead.chain_with_and(input, peek, with, condition)?;
                Ok((lookahead, ($($v,)* value,)))
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
