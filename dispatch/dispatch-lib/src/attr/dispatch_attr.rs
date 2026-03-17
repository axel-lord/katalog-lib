//! [DispatchAttr] impl.

use crate::{dispatch_fn::DispatchFn, kw};

use ::katalog_lib_proc_macro_common::attr_writer;
use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Attribute, Generics, ImplItem, Token, braced,
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
    pub items: Vec<ImplAttrItem>,
}

impl ToTokens for ImplAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            attrs,
            impl_token,
            generics,
            self_token,
            brace_token,
            items,
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
            for item in items {
                item.to_tokens(tokens);
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
        let mut items = Vec::new();

        attrs.extend(content.call(Attribute::parse_inner)?);

        while !content.is_empty() {
            items.push(content.parse()?);
        }

        Ok(Self {
            attrs,
            impl_token,
            generics,
            self_token,
            brace_token,
            items,
        })
    }
}

/// Items of impl attr block.
#[derive(Clone)]
pub enum ImplAttrItem {
    /// Regular impl item.
    ImplItem(ImplItem),
    /// Dispatch function.
    DispatchFn(DispatchFn),
}

impl ToTokens for ImplAttrItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ImplAttrItem::ImplItem(impl_item) => impl_item.to_tokens(tokens),
            ImplAttrItem::DispatchFn(dispatch_fn) => dispatch_fn.to_tokens(tokens),
        }
    }
}

impl Parse for ImplAttrItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::dispatch) {
            let mut dispatch_fn = input.parse::<DispatchFn>()?;
            dispatch_fn.attrs = attrs;
            Ok(Self::DispatchFn(dispatch_fn))
        } else {
            let mut lookahead_error = lookahead.error();
            let mut item = input.parse::<ImplItem>().map_err(|err| {
                lookahead_error.combine(err);
                lookahead_error
            })?;

            match &mut item {
                ImplItem::Const(impl_item_const) => impl_item_const.attrs = attrs,
                ImplItem::Fn(impl_item_fn) => impl_item_fn.attrs = attrs,
                ImplItem::Type(impl_item_type) => impl_item_type.attrs = attrs,
                ImplItem::Macro(impl_item_macro) => impl_item_macro.attrs = attrs,
                _ => {}
            }

            Ok(Self::ImplItem(item))
        }
    }
}
