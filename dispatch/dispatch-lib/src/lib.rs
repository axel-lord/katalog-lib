//! Implementation for dispatch derive macro.
#![allow(missing_debug_implementations)]

use ::katalog_lib_proc_macro_common::err_collector::ErrCollector;
use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    ItemEnum, ItemImpl, Path, Type, TypePath,
    parse::{Parse, Parser},
};

use crate::attr::{DispatchAttr, ImplAttr};

pub mod attr;
pub mod dispatch_fn;
pub mod dispatch_parameter;
pub mod mono_closure;
pub mod path_prefix;
pub mod psuedo_closure;

mod kw;
mod util;

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

        let Some(attr) = errors.scope(|| {
            // attr.parse_args_with(Punctuated::<DispatchAttr, Token![,]>::parse_terminated)
            attr.parse_args::<DispatchAttr>()
        }) else {
            continue;
        };

        let self_ty = Type::from(TypePath {
            qself: None,
            path: Path::from(item_enum.ident.clone()),
        });

        match attr {
            DispatchAttr::Dispatches(dispatches) => {
                impl_blocks.push(ItemImpl {
                    attrs: Vec::new(),
                    defaultness: None,
                    unsafety: None,
                    impl_token: Default::default(),
                    generics: Default::default(),
                    trait_: None,
                    self_ty: Box::new(self_ty.clone()),
                    brace_token: Default::default(),
                    items: errors
                        .collect(dispatches.into_iter().map(|item| item.to_item(&item_enum))),
                });
            }
            DispatchAttr::Impls(impls) => {
                for ImplAttr {
                    attrs,
                    impl_token,
                    generics,
                    self_token: _,
                    brace_token,
                    items,
                } in impls
                {
                    impl_blocks.push(ItemImpl {
                        attrs,
                        defaultness: None,
                        unsafety: None,
                        impl_token,
                        generics: generics.unwrap_or_else(|| item_enum.generics.clone()),
                        trait_: None,
                        self_ty: Box::new(self_ty.clone()),
                        brace_token,
                        items: errors.collect(items.into_iter().map(|item| match item {
                            attr::ImplAttrItem::ImplItem(impl_item) => Ok(impl_item),
                            attr::ImplAttrItem::DispatchFn(dispatch_fn) => {
                                dispatch_fn.to_item(&item_enum)
                            }
                        })),
                    });
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
