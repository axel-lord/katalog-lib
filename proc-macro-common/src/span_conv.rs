//! Convert into/between span types.

use ::std::{rc::Rc, sync::Arc};

use ::proc_macro2::{Delimiter, Group, Span, TokenStream, extra::DelimSpan};
use ::syn::token;

/// Get a call site span of type `S`.
pub fn call_site<S>() -> S
where
    Span: GetSpan<S>,
{
    Span::call_site().span_conv()
}

/// Unit struct implementing [GetSpan] for callsite spans.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct CallSite;

impl GetSpan<Span> for CallSite {
    fn get_span(&self) -> Span {
        call_site()
    }
}

/// Trait for pipeline span conversions.
pub trait SpanConvert {
    /// Convert into a span of type `S`.
    fn span_conv<S>(self) -> S
    where
        Self: Sized + GetSpan<S>,
    {
        self.get_span()
    }
}
impl<T> SpanConvert for T {}

/// Get span of specified type from self.
pub trait GetSpan<S> {
    /// Get a span of type `S`.
    fn get_span(&self) -> S;
}

impl GetSpan<Span> for Span {
    fn get_span(&self) -> Span {
        *self
    }
}

impl GetSpan<DelimSpan> for DelimSpan {
    fn get_span(&self) -> DelimSpan {
        *self
    }
}

impl GetSpan<DelimSpan> for Group {
    fn get_span(&self) -> DelimSpan {
        self.delim_span()
    }
}

impl<const N: usize, T, S> GetSpan<[S; N]> for T
where
    T: GetSpan<S>,
{
    fn get_span(&self) -> [S; N] {
        ::core::array::from_fn(|_| <Self as GetSpan<S>>::get_span(self))
    }
}

impl<T> GetSpan<DelimSpan> for T
where
    T: GetSpan<Span>,
{
    fn get_span(&self) -> DelimSpan {
        let mut group = Group::new(Delimiter::None, TokenStream::default());
        group.set_span(<T as GetSpan<Span>>::get_span(self));
        group.delim_span()
    }
}

/// Implement SpanConvert for delims.
macro_rules! delim_impl {
    ($($delim:path),+) => {
        $(
        impl GetSpan<DelimSpan> for $delim {
            fn get_span(&self) -> DelimSpan {
                self.span
            }
        }
        )*
    };
}

delim_impl!(token::Brace, token::Bracket, token::Paren);

/// Generate GetSpan impls for derefs.
macro_rules! deref_impl {
    ($t:ident: $($ty:ty),+) => {
        $(
        impl<$t> GetSpan<Span> for $ty
        where
            $t: GetSpan<Span>,
        {
            fn get_span(&self) -> Span {
                $t::get_span(self)
            }
        }
        )*
    };
}

deref_impl!(T: &T, &mut T, Box<T>, Rc<T>, Arc<T>);
