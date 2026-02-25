//! Attributes

use crate::{dispatch_fn::DispatchFn, kw};

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Expr, Ident, MetaList, Token, braced,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    token,
};

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
    fn parse(input: ParseStream) -> ::syn::Result<Self> {
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
    fn parse(input: ParseStream) -> ::syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![impl]) {
            Ok(DispatchAttr::Impl(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

/// Enum variant field dispatch attribute content.
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

/// Enum variant field dispatch attribute.
pub enum FieldAttr {
    /// Ignore or use the field.
    Inner(FieldAttrInner),
    /// Named attribute with lazy parsing.
    Named(MetaList),
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::ignore) || lookahead.peek(Token![use]) {
            input.parse().map(Self::Inner)
        } else if lookahead.peek(Token![::]) || lookahead.peek(Ident::peek_any) {
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

/// Closure like with only one parameter.
pub struct MonoClosure {
    /// left '|' token.
    pub left_pipe: Token![|],
    /// Parameter ident.
    pub param: Ident,
    /// right '|' token.
    pub right_pipe: Token![|],
    /// Closure expression.
    pub expr: Expr,
}

impl Parse for MonoClosure {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            left_pipe: input.parse()?,
            param: input.parse()?,
            right_pipe: input.parse()?,
            expr: input.parse()?,
        })
    }
}

impl ToTokens for MonoClosure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            left_pipe,
            param,
            right_pipe,
            expr,
        } = self;
        left_pipe.to_tokens(tokens);
        param.to_tokens(tokens);
        right_pipe.to_tokens(tokens);
        expr.to_tokens(tokens);
    }
}
