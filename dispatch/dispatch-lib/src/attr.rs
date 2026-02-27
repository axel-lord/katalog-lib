//! Attributes

use crate::{dispatch_fn::DispatchFn, kw};

use ::katalog_lib_proc_macro_common::{delimited::MacroDelimited, lazy::Lazy};
use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Expr, Generics, Ident, Token, braced,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{self},
};

/// A named MetaList like attribute.
#[derive(Clone)]
pub struct NamedAttr<T> {
    /// Attribute name.
    pub name: Ident,
    /// Content of attribute.
    pub content: MacroDelimited<Lazy<T>>,
}

impl<T> Parse for NamedAttr<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            content: input.parse()?,
        })
    }
}

impl<T> ToTokens for NamedAttr<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, content } = self;
        name.to_tokens(tokens);
        content.to_tokens(tokens);
    }
}

/// Dispatch impl attribute.
#[derive(Clone)]
pub struct ImplAttr {
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
            impl_token,
            generics,
            self_token,
            brace_token,
            functions,
        } = self;
        impl_token.to_tokens(tokens);
        generics.to_tokens(tokens);
        self_token.to_tokens(tokens);
        if let Some(generics) = generics {
            generics.where_clause.to_tokens(tokens);
        }
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

        while !content.is_empty() {
            functions.push(content.parse()?);
        }

        Ok(Self {
            impl_token,
            generics,
            self_token,
            brace_token,
            functions,
        })
    }
}

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

/// Closure like with only one parameter.
#[derive(Clone)]
pub struct PsuedoClosure {
    /// left '|' token.
    pub left_pipe: Token![|],

    /// Initial parameters.
    pub head_params: Punctuated<Ident, Token![,]>,
    /// '..' token.
    pub rest_token: Option<Token![..]>,
    /// Comma separating rest and tail params.
    pub rest_comma: Option<Token![,]>,
    /// Final parameters.
    pub tail_params: Punctuated<Ident, Token![,]>,

    /// right '|' token.
    pub right_pipe: Token![|],
    /// Closure expression.
    pub expr: Expr,
}

impl PsuedoClosure {
    /// Get head and tail parameters.
    pub const fn params(&self) -> [&Punctuated<Ident, Token![,]>; 2] {
        [&self.head_params, &self.tail_params]
    }

    /// Returns true if closure has a rest parameter.
    pub const fn captures_rest(&self) -> bool {
        self.rest_token.is_some()
    }
}

impl Parse for PsuedoClosure {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let left_pipe = input.parse()?;

        let mut head_params = Punctuated::new();
        let mut tail_params = Punctuated::new();
        let mut rest_token = None;
        let mut rest_comma = None;

        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![|]) || lookahead.peek(Token![..]) {
                break;
            } else if lookahead.peek(Ident::peek_any) {
                head_params.push(input.parse()?);
                let lookahead = input.lookahead1();
                if lookahead.peek(Token![,]) {
                    head_params.push_punct(input.parse()?);
                } else if lookahead.peek(Token![|]) {
                    break;
                } else {
                    return Err(lookahead.error());
                }
            } else {
                return Err(lookahead.error());
            }
        }

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![..]) {
            rest_token = Some(input.parse()?);

            let lookahead = input.lookahead1();
            if lookahead.peek(Token![,]) {
                rest_comma = Some(input.parse()?);

                loop {
                    let lookahead = input.lookahead1();
                    if lookahead.peek(Token![|]) {
                        break;
                    } else if lookahead.peek(Ident::peek_any) {
                        tail_params.push(input.parse()?);

                        let lookahead = input.lookahead1();
                        if lookahead.peek(Token![,]) {
                            tail_params.push_punct(input.parse()?);
                        } else if lookahead.peek(Token![|]) {
                            break;
                        } else {
                            return Err(lookahead.error());
                        }
                    } else {
                        return Err(lookahead.error());
                    }
                }
            } else if lookahead.peek(Token![|]) {
            } else {
                return Err(lookahead.error());
            }
        } else if lookahead.peek(Token![|]) {
        } else {
            return Err(lookahead.error());
        }

        let right_pipe = input.parse()?;
        let expr = input.parse()?;

        Ok(Self {
            left_pipe,
            head_params,
            rest_token,
            rest_comma,
            tail_params,
            right_pipe,
            expr,
        })
    }
}

impl ToTokens for PsuedoClosure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            left_pipe,
            head_params,
            rest_token: rest,
            rest_comma,
            tail_params,
            right_pipe,
            expr,
        } = self;
        left_pipe.to_tokens(tokens);
        head_params.to_tokens(tokens);
        rest.to_tokens(tokens);
        rest_comma.to_tokens(tokens);
        tail_params.to_tokens(tokens);
        right_pipe.to_tokens(tokens);
        expr.to_tokens(tokens);
    }
}
