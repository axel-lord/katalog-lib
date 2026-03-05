//! Attributes

use ::katalog_lib_proc_macro_common::{delimited::MacroDelimited, lazy::Lazy};
use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Ident,
    parse::{Parse, ParseStream},
};

pub use self::{dispatch_attr::*, dispatch_fn_attr::*, field_attr::*};

mod dispatch_attr;
mod dispatch_fn_attr;
mod field_attr;

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
