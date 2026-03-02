//! Functions to create function objects writing attributes to token streams
//! depending on whether they are inner or outer attributes.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{AttrStyle, Attribute};

/// Create an attribute writer writing outer attributes.
pub fn outer(tokens: &mut TokenStream) -> impl for<'a> FnMut(&'a Attribute) {
    |attr| {
        if matches!(attr.style, AttrStyle::Outer) {
            attr.to_tokens(tokens);
        }
    }
}

/// Create an attribute writer writing inner attributes.
pub fn inner(tokens: &mut TokenStream) -> impl for<'a> FnMut(&'a Attribute) {
    |attr| {
        if matches!(attr.style, AttrStyle::Inner(..)) {
            attr.to_tokens(tokens);
        }
    }
}
