//! Ast for fucntion templates.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Block, Generics, Ident, Path, ReturnType, Token, braced,
    ext::IdentExt as _,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};

use crate::{
    dispatch_parameter::DispatchParameters,
    path_prefix::{PathPrefix, Qualified},
};

/// Dispatch function template.
pub struct DispatchFn {
    /// 'for' token signaling generics.
    pub for_token: Option<Token![for]>,
    /// Generic arguments.
    pub generics: Generics,

    /// 'as' token.
    pub as_token: Option<Token![as]>,
    /// Name of function.
    pub ident: Ident,

    /// 'fn' token.
    pub fn_token: Token![fn],

    /// Optional prefix of path.
    pub prefix: Option<PathPrefix>,
    /// Path of function to call.
    pub path: Path,
    /// Name of the function, same as

    /// Parameters of function.
    pub parameters: DispatchParameters,
    /// Return type of function.
    pub output: ReturnType,

    /// Block used for unit variants.
    pub block: Option<Block>,
    /// Trailing semicolon if no block.
    pub trailing_semi: Option<Token![;]>,
}

impl DispatchFn {}

impl ToTokens for DispatchFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            for_token,
            generics,
            as_token,
            ident,
            fn_token,
            prefix,
            path,
            parameters,
            output,
            block,
            trailing_semi,
        } = self;

        for_token.to_tokens(tokens);
        generics.to_tokens(tokens);
        if let Some(as_token) = as_token {
            as_token.to_tokens(tokens);
            ident.to_tokens(tokens);
        }
        fn_token.to_tokens(tokens);
        prefix.to_tokens(tokens);
        path.to_tokens(tokens);
        parameters.to_tokens(tokens);
        output.to_tokens(tokens);
        generics.where_clause.to_tokens(tokens);
        block.to_tokens(tokens);
        trailing_semi.to_tokens(tokens);
    }
}

impl Parse for DispatchFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let for_token;
        let mut generics;

        // Trick to create a new lookahead if for was matched thus not adding it
        // to the error message if matched whilst still having it be in the set if not matched.
        let lookahead = input.lookahead1();
        let lookahead = if lookahead.peek(Token![for]) {
            for_token = input.parse()?;

            if !input.peek(Token![<]) {
                return Err(input.error("expected '<' after for"));
            }

            generics = input.parse()?;

            input.lookahead1()
        } else {
            for_token = None;
            generics = Generics::default();
            lookahead
        };

        let as_token;
        let ident;

        let lookahead = if lookahead.peek(Token![as]) {
            as_token = input.parse()?;
            ident = Some(input.parse()?);
            input.lookahead1()
        } else {
            as_token = None;
            ident = None;
            lookahead
        };

        let fn_token = if lookahead.peek(Token![fn]) {
            input.parse()?
        } else {
            return Err(lookahead.error());
        };

        let leading_colon;
        let lookahead = input.lookahead1();
        let prefix = if lookahead.peek(Token![::]) || lookahead.peek(Ident::peek_any) {
            leading_colon = input.parse()?;
            None
        } else if lookahead.peek(Token![<]) {
            let prefix = if input.peek2(Token![Self]) {
                let lt_token = input.parse()?;
                let self_ty = input.parse()?;

                let lookahead = input.lookahead1();
                if lookahead.peek(Token![>]) {
                    PathPrefix::SelfTy {
                        lt_token,
                        self_ty,
                        gt_token: input.parse()?,
                    }
                } else if lookahead.peek(Token![as]) {
                    let value = self_ty;

                    PathPrefix::QualifiedSelf(Qualified {
                        lt_token,
                        value,
                        as_token: input.parse()?,
                        path: {
                            let leading_colon = input.parse()?;
                            let segments = input.call(Punctuated::parse_separated_nonempty)?;
                            Path {
                                leading_colon,
                                segments,
                            }
                        },
                        gt_token: input.parse()?,
                    })
                } else {
                    return Err(lookahead.error());
                }
            } else {
                PathPrefix::Qualified(input.parse()?)
            };

            leading_colon = Some(input.parse()?);
            Some(prefix)
        } else {
            return Err(lookahead.error());
        };

        let path = Path {
            leading_colon,
            segments: input.call(Punctuated::parse_separated_nonempty)?,
        };

        let parameters = input.parse()?;

        let lookahead = input.lookahead1();

        let output;
        let lookahead = if lookahead.peek(Token![->]) {
            output = input.parse()?;
            input.lookahead1()
        } else {
            output = ReturnType::Default;
            lookahead
        };
        let lookahead = if lookahead.peek(Token![where]) {
            generics.where_clause = Some(input.parse()?);
            input.lookahead1()
        } else {
            lookahead
        };

        let (block, trailing_semi) = if lookahead.peek(Token![;]) {
            (None, input.parse()?)
        } else if lookahead.peek(token::Brace) {
            let content;
            let brace_token = braced!(content in input);
            let stmts = content.call(Block::parse_within)?;
            (Some(Block { brace_token, stmts }), None)
        } else {
            return Err(lookahead.error());
        };

        let ident = ident.map(Ok).unwrap_or_else(|| {
            path.segments.last().map_or_else(
                || {
                    Err(::syn::Error::new_spanned(
                        &path,
                        "expected at least one ident component in path",
                    ))
                },
                |segment| Ok(segment.ident.clone()),
            )
        })?;

        Ok(Self {
            for_token,
            generics,
            as_token,
            ident,
            fn_token,
            prefix,
            path,
            parameters,
            output,
            block,
            trailing_semi,
        })
    }
}
