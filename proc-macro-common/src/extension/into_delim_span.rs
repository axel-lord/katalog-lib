//! [IntoDelimSpan] impl.

use ::proc_macro2::{Delimiter, Group, Span, TokenStream, extra::DelimSpan};
use ::syn::token;

/// Convert values into [DelimSpan].
pub trait IntoDelimSpan {
    /// Convert self into a [DelimSpan].
    fn into_delim_span(self) -> DelimSpan;
}

impl IntoDelimSpan for Span {
    fn into_delim_span(self) -> DelimSpan {
        let mut group = Group::new(Delimiter::None, TokenStream::default());
        group.set_span(self);
        group.delim_span()
    }
}

impl IntoDelimSpan for DelimSpan {
    fn into_delim_span(self) -> DelimSpan {
        self
    }
}

impl IntoDelimSpan for &Span {
    fn into_delim_span(self) -> DelimSpan {
        Span::into_delim_span(*self)
    }
}

impl IntoDelimSpan for &mut Span {
    fn into_delim_span(self) -> DelimSpan {
        Span::into_delim_span(*self)
    }
}

impl IntoDelimSpan for &DelimSpan {
    fn into_delim_span(self) -> DelimSpan {
        *self
    }
}

impl IntoDelimSpan for &mut DelimSpan {
    fn into_delim_span(self) -> DelimSpan {
        *self
    }
}

impl IntoDelimSpan for Group {
    fn into_delim_span(self) -> DelimSpan {
        self.delim_span()
    }
}

impl IntoDelimSpan for &Group {
    fn into_delim_span(self) -> DelimSpan {
        self.delim_span()
    }
}

/// Implement SpanConvert for delims.
macro_rules! delim_impl {
    ($($delim:path),+) => {
        $(
        impl IntoDelimSpan for $delim {
            fn into_delim_span(self) -> DelimSpan {
                self.span
            }
        }
        impl IntoDelimSpan for &$delim {
            fn into_delim_span(self) -> DelimSpan {
                self.span
            }
        }
        )*
    };
}

delim_impl!(token::Brace, token::Bracket, token::Paren);
