//! Attributes

use crate::{dispatch_fn::DispatchFn, kw};

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{Token, braced, parse::Parse, token};

/// Dispatch impl attribute.
pub struct ImplAttr {
    /// Impl token.
    pub impl_token: Token![impl],
    /// braces '{}'.
    pub brace_token: token::Brace,
    /// Dispatch functions.
    pub functions: Vec<DispatchFn>,
}

impl ToTokens for ImplAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            impl_token,
            brace_token,
            functions,
        } = self;
        impl_token.to_tokens(tokens);
        brace_token.surround(tokens, |tokens| {
            for function in functions {
                function.to_tokens(tokens);
            }
        });
    }
}

impl Parse for ImplAttr {
    fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
        let impl_token = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let mut functions = Vec::new();

        while !content.is_empty() {
            functions.push(content.parse()?);
        }

        Ok(Self {
            impl_token,
            brace_token,
            functions,
        })
    }
}

/// Dispatch attributes.
pub enum DispatchAttr {
    /// Impl block attribute.
    Impl(ImplAttr),
}

impl Parse for DispatchAttr {
    fn parse(input: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![impl]) {
            Ok(DispatchAttr::Impl(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

/// Enum variant field dispatch attribute content.
pub enum FieldAttr {
    /// Ignore this field.
    Ignore(kw::ignore),
    /// Use only this field.
    Use(Token![use]),
}

impl Parse for FieldAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::ignore) {
            input.parse().map(FieldAttr::Ignore)
        } else if lookahead.peek(Token![use]) {
            input.parse().map(FieldAttr::Use)
        } else {
            Err(lookahead.error())
        }
    }
}
