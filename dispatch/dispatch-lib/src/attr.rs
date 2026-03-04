//! Attributes

use crate::{dispatch_fn::DispatchFn, kw, mono_closure::MonoClosure};

use ::katalog_lib_proc_macro_common::{attr_writer, delimited::MacroDelimited, lazy::Lazy};
use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Attribute, Generics, Ident, Token, braced,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
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

/// A single parameter mapping.
#[derive(Clone)]
pub struct ParameterMapping {
    /// Name of parameter.
    pub param: Ident,
    /// ':' token.
    pub colon_token: Token![:],
    /// Name to bind parameter to.
    pub binding: Ident,
}

impl ParameterMapping {
    /// Convert into a param, (colon, ,binding)) pair.
    pub fn into_pair(self) -> (Ident, (Token![:], Ident)) {
        let Self {
            param,
            binding,
            colon_token,
        } = self;
        (param, (colon_token, binding))
    }
}

impl Parse for ParameterMapping {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            param: input.parse()?,
            colon_token: input.parse()?,
            binding: input.parse()?,
        })
    }
}

impl ToTokens for ParameterMapping {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            param,
            colon_token,
            binding,
        } = self;
        param.to_tokens(tokens);
        colon_token.to_tokens(tokens);
        binding.to_tokens(tokens);
    }
}

/// Attribute used to remap parameters.
#[derive(Clone)]
pub struct AttrRemap {
    /// Remap ident.
    pub remap: kw::remap,
    /// Brace token arround remap block.
    pub brace_token: token::Brace,
    /// Mappings.
    pub mappings: Punctuated<ParameterMapping, Token![,]>,
}

impl Parse for AttrRemap {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            remap: input.parse()?,
            brace_token: braced!(content in input),
            mappings: content.call(Punctuated::parse_terminated)?,
        })
    }
}

impl ToTokens for AttrRemap {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            remap,
            brace_token,
            mappings,
        } = self;
        remap.to_tokens(tokens);
        brace_token.surround(tokens, |tokens| mappings.to_tokens(tokens));
    }
}

/// Attribute to map result of dispatch.
#[derive(Clone)]
pub enum AttrMap {
    /// Map using a mono closure.
    Closure {
        /// Map keyword.
        map_kw: kw::map,
        /// Closure to map with.
        closure: MonoClosure,
    },
}

/// Attribute on dispatch function.
#[derive(Clone)]
pub enum DispatchFnAttr {
    /// Remap parameters in function.
    Remap(AttrRemap),
}

impl Parse for DispatchFnAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::remap) {
            Ok(Self::Remap(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for DispatchFnAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DispatchFnAttr::Remap(attr_remap) => attr_remap.to_tokens(tokens),
        }
    }
}
