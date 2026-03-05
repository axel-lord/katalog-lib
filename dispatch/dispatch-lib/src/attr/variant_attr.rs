//! [VariantAttr] impl.

use ::quote::ToTokens;
use ::syn::{
    Ident, Token,
    ext::IdentExt as _,
    parse::{Lookahead1, Parse},
    punctuated::Punctuated,
};

use crate::attr::NamedAttr;

/// Variant attribute.
#[derive(Clone)]
pub enum VariantAttr {
    /// Global variant attriubte.
    Inner(VariantAttrInner),
    /// Function local attribute.
    Named(NamedAttr<Punctuated<VariantAttrInner, Token![,]>>),
}

impl Parse for VariantAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if VariantAttrInner::peek_lookahead(&lookahead) {
            input.parse().map(Self::Inner)
        } else if lookahead.peek(Ident::peek_any) {
            input.parse().map(Self::Named)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for VariantAttr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            VariantAttr::Inner(variant_attr_inner) => variant_attr_inner.to_tokens(tokens),
            VariantAttr::Named(named_attr) => named_attr.to_tokens(tokens),
        }
    }
}

/// Variant attribute without run specification.
#[derive(Clone)]
pub enum VariantAttrInner {
    /// Variant should use default block instead of dispatch.
    Default(Token![default]),
}

impl VariantAttrInner {
    /// Check if token of lookahead is valid first token.
    pub fn peek_lookahead(lookahead: &Lookahead1) -> bool {
        lookahead.peek(Token![default])
    }
}

impl Parse for VariantAttrInner {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse().map(Self::Default)
    }
}

impl ToTokens for VariantAttrInner {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            VariantAttrInner::Default(default_token) => default_token.to_tokens(tokens),
        }
    }
}
