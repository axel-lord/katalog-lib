//! Proc Macro crate for [Dispatch].

use ::proc_macro::TokenStream;

/// Dispatch functions to enum variants.
///
/// Functions are implemented by passing prototypes to the dispatch attribute.
/// If functions have a body it will be used for unit variants.
///
/// Variant fields may use `#[dispatch(ignore, use)]`
/// to either use specified field as input or ignore specified field.
///
/// Said attribute may be nested in the dispatch function name to have it only
/// apply for said dispatch function `#[dispatch(fn_name(ignore, use))]`
///
/// The dispatch functions are defined using a `#[dispatch(impl { dispatch_fn... })]` attribute.
/// on the enum.
///
/// The syntax for a dispatch function is:
///
/// `[for<GENERICS>] [as NAME] [VIS] [const] [async] fn [<PREFIX>::]PATH([&][mut] RECV, IDENT: TY...) [BLOCK]`
///
/// PREFIX is either Self, Self as PATH or TY as PATH.
/// A trailing semicolon is required if BLOCK is not present.
///
/// If async is present awaits will be added to non unit calls.
#[proc_macro_derive(Dispatch, attributes(dispatch))]
pub fn derive_dispatch(item: TokenStream) -> TokenStream {
    ::katalog_lib_dispatch_lib::derive_dispatch(item.into()).into()
}
