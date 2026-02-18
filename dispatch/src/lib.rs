//! Proc Macro crate for [Dispatch].

use ::proc_macro::TokenStream;

/// Dispatch functions to enum variants.
///
/// Functions are implemented by passing prototypes to the dispatch attribute.
/// If functions have a body it will be used for unit variants.
#[proc_macro_derive(Dispatch, attributes(dispatch))]
pub fn derive_dispatch(item: TokenStream) -> TokenStream {
    ::katalog_lib_dispatch_lib::derive_dispatch(item.into()).into()
}
