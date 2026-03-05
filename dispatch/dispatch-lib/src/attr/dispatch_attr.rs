//! [DispatchAttr] impl.

use crate::dispatch_fn::DispatchFn;

use ::katalog_lib_proc_macro_common::attr_writer;
use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Attribute, Generics, Token, braced,
    parse::{Parse, ParseStream},
    token,
};

/// Dispatch attributes.
#[derive(Clone)]
pub enum DispatchAttr {
    /// Impl block attribute.
    Impl(ImplAttr),
}

impl Parse for DispatchAttr {
    fn parse(input: ParseStream) -> ::syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![impl]) {
            Ok(DispatchAttr::Impl(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

/// Dispatch impl attribute.
#[derive(Clone)]
pub struct ImplAttr {
    /// Impl block attributes.
    pub attrs: Vec<Attribute>,
    /// Impl token.
    pub impl_token: Token![impl],
    /// Optional generic parameters for impl.
    pub generics: Option<Generics>,
    /// Optional 'Self' token.
    pub self_token: Option<Token![Self]>,
    /// braces '{}'.
    pub brace_token: token::Brace,
    /// Dispatch functions.
    pub functions: Vec<DispatchFn>,
}

impl ToTokens for ImplAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            attrs,
            impl_token,
            generics,
            self_token,
            brace_token,
            functions,
        } = self;
        attrs.iter().for_each(attr_writer::outer(tokens));
        impl_token.to_tokens(tokens);
        generics.to_tokens(tokens);
        self_token.to_tokens(tokens);
        if let Some(generics) = generics {
            generics.where_clause.to_tokens(tokens);
        }
        brace_token.surround(tokens, |tokens| {
            attrs.iter().for_each(attr_writer::inner(tokens));
            for function in functions {
                function.to_tokens(tokens);
            }
        });
    }
}

impl Parse for ImplAttr {
    fn parse(input: ParseStream) -> ::syn::Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let impl_token = input.parse()?;

        let lookahead = input.lookahead1();
        let generics = if lookahead.peek(token::Brace) || lookahead.peek(Token![Self]) {
            None
        } else if lookahead.peek(Token![<]) {
            Some(input.parse()?)
        } else {
            return Err(lookahead.error());
        };

        let lookahead = input.lookahead1();
        let self_token = if lookahead.peek(token::Brace) {
            None
        } else if lookahead.peek(Token![Self]) {
            Some(input.parse()?)
        } else {
            return Err(lookahead.error());
        };

        let content;
        let brace_token = braced!(content in input);
        let mut functions = Vec::new();

        attrs.extend(content.call(Attribute::parse_inner)?);

        while !content.is_empty() {
            let attrs = content.call(Attribute::parse_outer)?;
            let mut function = content.parse::<DispatchFn>()?;
            function.attrs.extend(attrs);
            functions.push(function);
        }

        Ok(Self {
            attrs,
            impl_token,
            generics,
            self_token,
            brace_token,
            functions,
        })
    }
}
