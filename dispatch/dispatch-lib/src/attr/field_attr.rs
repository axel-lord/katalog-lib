//! [FieldAttr] impl.

use crate::{attr::NamedAttr, kw};

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Ident, Token,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

/// Enum variant field dispatch attribute.
#[derive(Clone)]
pub enum FieldAttr {
    /// Ignore or use the field.
    Inner(FieldAttrInner),
    /// Named attribute with lazy parsing.
    Named(NamedAttr<Punctuated<FieldAttrInner, Token![,]>>),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::ignore) || lookahead.peek(Token![use]) {
            input.parse().map(Self::Inner)
        } else if lookahead.peek(Ident::peek_any) {
            input.parse().map(Self::Named)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for FieldAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldAttr::Inner(ignore_use) => ignore_use.to_tokens(tokens),
            FieldAttr::Named(meta_list) => meta_list.to_tokens(tokens),
        }
    }
}

/// Enum variant field dispatch attribute content.
#[derive(Clone)]
pub enum FieldAttrInner {
    /// Ignore this field.
    Ignore(kw::ignore),
    /// Use only this field.
    Use(Token![use]),
}

impl ToTokens for FieldAttrInner {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FieldAttrInner::Ignore(ignore) => ignore.to_tokens(tokens),
            FieldAttrInner::Use(r#use) => r#use.to_tokens(tokens),
        }
    }
}

impl Parse for FieldAttrInner {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::ignore) {
            input.parse().map(FieldAttrInner::Ignore)
        } else if lookahead.peek(Token![use]) {
            input.parse().map(FieldAttrInner::Use)
        } else {
            Err(lookahead.error())
        }
    }
}
