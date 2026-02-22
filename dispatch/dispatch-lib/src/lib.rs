//! Implementation for dispatch derive macro.
#![allow(missing_debug_implementations)]

use ::katalog_lib_proc_macro_common::err_collector::ErrCollector;
use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    ItemEnum, ItemImpl, Path, Token, Type, TypePath,
    parse::{Parse, Parser},
    punctuated::Punctuated,
};

use crate::attr::{DispatchAttr, ImplAttr};

pub mod attr;
pub mod dispatch_fn;
pub mod dispatch_parameter;
pub mod path_prefix;

mod util;
mod kw {
    //! Custom keywords.

    use ::syn::custom_keyword;

    custom_keyword!(ignore);
}

/// Macro to implement dispatch.
pub fn derive_dispatch(item: TokenStream) -> TokenStream {
    dispatch(item).unwrap_or_else(::syn::Error::into_compile_error)
}

/// Dispatch impl.
fn dispatch(item: TokenStream) -> ::syn::Result<TokenStream> {
    let item_enum = ItemEnum::parse.parse2(item)?;

    let mut errors = ErrCollector::<Vec<::syn::Error>>::default();
    let mut impl_blocks = Vec::<ItemImpl>::new();
    for attr in &item_enum.attrs {
        if !attr.path().is_ident("dispatch") {
            continue;
        }

        let Some(attrs) = errors.scope(|| {
            attr.parse_args_with(Punctuated::<DispatchAttr, Token![,]>::parse_terminated)
        }) else {
            continue;
        };

        for attr in attrs {
            match attr {
                DispatchAttr::Impl(ImplAttr {
                    impl_token,
                    brace_token,
                    functions,
                }) => {
                    let impl_block = ItemImpl {
                        attrs: Vec::new(),
                        defaultness: None,
                        unsafety: None,
                        impl_token,
                        generics: item_enum.generics.clone(),
                        trait_: None,
                        self_ty: Box::new(Type::from(TypePath {
                            qself: None,
                            path: Path::from(item_enum.ident.clone()),
                        })),
                        brace_token,
                        items: errors.collect(
                            functions
                                .into_iter()
                                .map(|function| function.to_item(&item_enum)),
                        ),
                    };
                    impl_blocks.push(impl_block);
                }
            }
        }
    }

    let mut tokens = TokenStream::default();

    for impl_block in impl_blocks {
        impl_block.to_tokens(&mut tokens);
    }

    errors.with(tokens).into_result()
}
