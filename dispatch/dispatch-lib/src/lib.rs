//! Implementation for dispatch derive macro.
#![allow(missing_debug_implementations)]

use ::katalog_lib_proc_macro_common::err_collector::ErrCollector;
use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    ItemEnum, Token, braced,
    parse::{Parse, Parser},
    punctuated::Punctuated,
    token,
};

use crate::dispatch_fn::DispatchFn;

pub mod dispatch_fn;
pub mod dispatch_parameter;
pub mod path_prefix;

pub(crate) mod util;

/// Macro to implement dispatch.
pub fn derive_dispatch(item: TokenStream) -> TokenStream {
    dispatch(item).unwrap_or_else(::syn::Error::into_compile_error)
}

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

/// Dispatch impl.
fn dispatch(item: TokenStream) -> ::syn::Result<TokenStream> {
    let item_enum = ItemEnum::parse.parse2(item)?;

    let mut errors = ErrCollector::<Vec<::syn::Error>>::default();
    let mut dispatch_functions = Vec::new();
    for attr in &item_enum.attrs {
        if !attr.path().is_ident("dispatch") {
            continue;
        }

        let attrs =
            match attr.parse_args_with(Punctuated::<DispatchAttr, Token![,]>::parse_terminated) {
                Ok(attrs) => attrs,
                Err(err) => {
                    errors.push_err(err);
                    continue;
                }
            };

        for attr in attrs {
            match attr {
                DispatchAttr::Impl(impl_attr) => {
                    dispatch_functions.extend(impl_attr.functions);
                }
            }
        }
    }

    errors.with(TokenStream::default()).into_result()
}
