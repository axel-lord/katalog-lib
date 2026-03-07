//! [IntoDelimSpan] impl.

use ::proc_macro2::{Delimiter, Group, Span, TokenStream, extra::DelimSpan};
use ::syn::token;

/// Convert values into [DelimSpan].
pub trait IntoDelimiterSpan {
    /// Convert self into a [DelimSpan].
    fn into_delim_span(self) -> DelimSpan;
}

impl IntoDelimiterSpan for Span {
    fn into_delim_span(self) -> DelimSpan {
        let mut group = Group::new(Delimiter::None, TokenStream::default());
        group.set_span(self);
        group.delim_span()
    }
}

impl IntoDelimiterSpan for DelimSpan {
    fn into_delim_span(self) -> DelimSpan {
        self
    }
}

impl IntoDelimiterSpan for &Span {
    fn into_delim_span(self) -> DelimSpan {
        Span::into_delim_span(*self)
    }
}

impl IntoDelimiterSpan for &mut Span {
    fn into_delim_span(self) -> DelimSpan {
        Span::into_delim_span(*self)
    }
}

impl IntoDelimiterSpan for &DelimSpan {
    fn into_delim_span(self) -> DelimSpan {
        *self
    }
}

impl IntoDelimiterSpan for &mut DelimSpan {
    fn into_delim_span(self) -> DelimSpan {
        *self
    }
}

impl IntoDelimiterSpan for Group {
    fn into_delim_span(self) -> DelimSpan {
        self.delim_span()
    }
}

impl IntoDelimiterSpan for &Group {
    fn into_delim_span(self) -> DelimSpan {
        self.delim_span()
    }
}

/// Implement SpanConvert for delims.
macro_rules! delim_impl {
    ($($delim:path),+) => {
        $(
        impl IntoDelimiterSpan for $delim {
            fn into_delim_span(self) -> DelimSpan {
                self.span
            }
        }
        impl IntoDelimiterSpan for &$delim {
            fn into_delim_span(self) -> DelimSpan {
                self.span
            }
        }
        )*
    };
}

delim_impl!(token::Brace, token::Bracket, token::Paren);
