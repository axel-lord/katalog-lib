//! Ast for fucntion templates.

use ::proc_macro2::{Span, TokenStream};
use ::quote::ToTokens;
use ::syn::{
    Arm, Block, Expr, ExprBlock, ExprCall, ExprReference, ExprTuple, Generics, Ident, Pat, PatPath,
    Path, QSelf, ReturnType, Token, Type, TypePath, TypeTuple, Variant, braced,
    ext::IdentExt as _,
    parse::{Parse, ParseStream},
    punctuated::{Pair, Punctuated},
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

impl DispatchFn {
    /// Get a call path.
    fn call_path(&self, ty: &Type) -> PatPath {
        const fn empty_path() -> Path {
            Path {
                leading_colon: None,
                segments: Punctuated::new(),
            }
        }
        fn ident_to_boxed_type(ident: impl Into<Ident>) -> Box<Type> {
            Box::new(
                TypePath {
                    qself: None,
                    path: Path::from(ident),
                }
                .into(),
            )
        }
        let (qself, mut path) = self
            .prefix
            .as_ref()
            .map(|prefix| match prefix {
                PathPrefix::SelfTy {
                    lt_token,
                    self_ty: _,
                    gt_token,
                } => (
                    Some(QSelf {
                        lt_token: *lt_token,
                        ty: Box::new(ty.clone()),
                        position: 0,
                        as_token: None,
                        gt_token: *gt_token,
                    }),
                    empty_path(),
                ),
                PathPrefix::QualifiedSelf(Qualified {
                    lt_token,
                    value: _,
                    as_token,
                    path,
                    gt_token,
                }) => (
                    Some(QSelf {
                        lt_token: *lt_token,
                        ty: Box::new(ty.clone()),
                        position: path.segments.len(),
                        as_token: Some(*as_token),
                        gt_token: *gt_token,
                    }),
                    path.clone(),
                ),
                PathPrefix::Qualified(Qualified {
                    lt_token,
                    value,
                    as_token,
                    path,
                    gt_token,
                }) => (
                    Some(QSelf {
                        lt_token: *lt_token,
                        ty: value.clone(),
                        position: path.segments.len(),
                        as_token: Some(*as_token),
                        gt_token: *gt_token,
                    }),
                    path.clone(),
                ),
            })
            .unwrap_or_else(|| (None, empty_path()));

        if path.segments.is_empty() {
            path = self.path.clone();
        } else {
            let Path {
                leading_colon,
                segments,
            } = &self.path;
            if let Some(leading_colon) = &leading_colon {
                path.segments.push_punct(*leading_colon);
            }
            path.segments
                .extend(segments.pairs().map(|pair| pair.cloned()));
        }

        PatPath {
            attrs: Vec::new(),
            qself,
            path,
        }
    }

    /// Generate a cell for receiver with type ty.
    fn call(&self, expr: Option<Expr>, ty: &Type) -> ExprCall {
        fn ident_to_expr(ident: Ident) -> Expr {
            Expr::Path(PatPath {
                attrs: Vec::new(),
                qself: None,
                path: Path::from(ident.clone()),
            })
        }
        let DispatchParameters {
            paren_token,
            receiver,
            infix_comma,
            parameters,
        } = &self.parameters;
        ExprCall {
            attrs: Vec::new(),
            func: Box::new(self.call_path(ty).into()),
            paren_token: *paren_token,
            args: {
                let mut args = Punctuated::new();
                args.push(expr.unwrap_or_else(|| ident_to_expr(receiver.self_token.into())));

                if let Some(infix_comma) = infix_comma {
                    args.push_punct(*infix_comma);
                    args.extend(parameters.pairs().map(|pair| match pair {
                        Pair::Punctuated(value, punct) => {
                            Pair::Punctuated(ident_to_expr(value.ident.clone()), *punct)
                        }
                        Pair::End(value) => Pair::End(ident_to_expr(value.ident.clone())),
                    }));
                }

                args
            },
        }
    }

    /// Generate a match arm for given variant.
    fn match_arm(&self, ident: &Ident, variant: &Variant) -> Arm {
        let Variant {
            ident: variant_ident,
            fields,
            ..
        } = variant;

        let path = Path {
            leading_colon: None,
            segments: {
                let mut segments = Punctuated::new();
                segments.push(ident.clone().into());
                segments.push_punct(Token![::](ident.span()));
                segments.push(variant_ident.clone().into());
                segments
            },
        };
        let (pat, body) = match fields {
            ::syn::Fields::Named(fields_named) => todo!(),
            ::syn::Fields::Unnamed(fields_unnamed) => todo!(),
            ::syn::Fields::Unit => (
                Pat::from(PatPath {
                    attrs: Vec::new(),
                    qself: None,
                    path,
                }),
                self.block
                    .as_ref()
                    .map(|block| {
                        Box::new(Expr::from(ExprBlock {
                            attrs: Vec::new(),
                            label: None,
                            block: block.clone(),
                        }))
                    })
                    .unwrap_or_else(|| {
                        Box::new(Expr::from(self.call(
                            Some({
                                let expr = Expr::from(ExprTuple {
                                    attrs: Vec::new(),
                                    paren_token: token::Paren(variant_ident.span()),
                                    elems: Punctuated::new(),
                                });

                                if let Some((reference, ..)) = &self.parameters.receiver.reference {
                                    Expr::from(ExprReference {
                                        attrs: Vec::new(),
                                        and_token: *reference,
                                        mutability: self.parameters.receiver.mutability,
                                        expr: Box::new(expr),
                                    })
                                } else {
                                    expr
                                }
                            }),
                            &Type::from(TypeTuple {
                                paren_token: token::Paren(variant_ident.span()),
                                elems: Punctuated::new(),
                            }),
                        )))
                    }),
            ),
        };

        Arm {
            attrs: Vec::new(),
            pat,
            guard: None,
            fat_arrow_token: Token![=>](Span::call_site()),
            body,
            comma: Some(Token![,](Span::call_site())),
        }
    }
}

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
