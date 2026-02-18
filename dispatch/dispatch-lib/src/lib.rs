//! Implementation for dispatch derive macro.

use ::proc_macro2::TokenStream;
use ::syn::{
    Block, Ident, ItemEnum, Path, Signature, Token, braced, parse::{Parse, Parser}, spanned::Spanned, token
};

/// Macro to implement dispatch.
pub fn derive_dispatch(item: TokenStream) -> TokenStream {
    dispatch(item).unwrap_or_else(::syn::Error::into_compile_error)
}

/// Function signature for dispatch function.
struct DispatchSignature {
    /// fn token.
    fn_token: Token![fn],
    /// Self token replaced by type in calls.
    self_ty: Option<Token![Self]>,
    /// Path to function.
    path: Path,
}

/// Parsed dispatch function.
struct DispatchFn {
    /// Function signature.
    signature: Signature,
    /// Optional block.
    block: Option<Block>,
    /// Trailing semicolon if block is None.
    semi: Option<Token![;]>,
}

impl Parse for DispatchFn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let signature = input.parse()?;
        let lookahead = input.lookahead1();
        let (block, semi) = if lookahead.peek(token::Brace) {
            let content;
            let brace_token = braced!(content in input);
            let stmts = content.call(Block::parse_within)?;

            (Some(Block { brace_token, stmts }), None)
        } else if lookahead.peek(Token![;]) {
            (None, Some(input.parse()?))
        } else {
            return Err(lookahead.error());
        };

        Ok(Self {
            signature,
            block,
            semi,
        })
    }
}

/// Template used for creating dispatch function.
struct DispatchTemplateFn {
    /// Signature in impl block.
    signature: Signature,
    /// Self token of receiver in signature.
    self_token: Token![self],
    /// Default block for unit variants.
    block: Option<Block>,
}

impl TryFrom<DispatchFn> for DispatchTemplateFn {
    type Error = ::syn::Error;

    fn try_from(value: DispatchFn) -> Result<Self, Self::Error> {
        let DispatchFn {
            mut signature,
            block,
            ..
        } = value;

        let self_token = if let Some(receiver) = signature.receiver() {



            receiver.self_token
        } else {
            return Err(::syn::Error::new(
                signature.inputs.span(),
                "expected a receiver (self)",
            ));
        };



        Ok(Self {
            signature,
            self_token,
            block,
        })
    }
}

/// Dispatch impl attribute.
struct ImplAttr {
    /// Impl token.
    impl_token: Token![impl],
    /// braces '{}'.
    brace_token: token::Brace,
    /// Dispatch functions.
    functions: Vec<DispatchFn>,
}

impl Parse for ImplAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let impl_token = input.parse()?;
        let content;
        let brace_token = braced!(content in input);
        let mut functions = Vec::new();

        while !input.is_empty() {
            functions.push(input.parse()?);
        }

        Ok(Self {
            impl_token,
            brace_token,
            functions,
        })
    }
}

/// Dispatch impl.
fn dispatch(item: TokenStream) -> ::syn::Result<TokenStream> {
    let _item_enum = ItemEnum::parse.parse2(item)?;
    Ok(Default::default())
}
