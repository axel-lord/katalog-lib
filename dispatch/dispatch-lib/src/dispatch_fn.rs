//! Ast for fucntion templates.

use ::std::collections::BTreeMap;

use ::katalog_lib_proc_macro_common::{attr_writer, lookahead_chain::LookaheadChain};
use ::proc_macro2::{Span, TokenStream, extra::DelimSpan};
use ::quote::ToTokens;
use ::syn::{
    Arm, Attribute, Block, Expr, ExprAwait, ExprBlock, ExprCall, ExprMatch, ExprReference,
    ExprTuple, Field, FieldPat, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, ImplItem,
    ImplItemFn, ItemEnum, Local, LocalInit, Member, Pat, PatPath, PatRest, PatStruct,
    PatTupleStruct, PatWild, Path, QSelf, ReturnType, Signature, Stmt, Token, Type, TypeTuple,
    Variant, Visibility, WhereClause, braced,
    ext::IdentExt as _,
    parse::{Lookahead1, Parse, ParseStream},
    punctuated::{Pair, Punctuated},
    spanned::Spanned,
    token,
};

use crate::{
    attr::{DispatchFnAttr, FieldAttr, FieldAttrInner, ParameterMapping},
    dispatch_parameter::DispatchParameters,
    path_prefix::{PathPrefix, Qualified},
    util::ident_to_expr,
};

/// Get ident by mapping.
#[derive(Debug, Clone, Copy)]
struct ParamMap<'a>(&'a BTreeMap<Ident, (Token![:], Ident)>);

impl<'a> ParamMap<'a> {
    /// Get variable name from parameter name.
    fn get(&'a self, key: &'a Ident) -> &'a Ident {
        self.0.get(key).map(|(_, value)| value).unwrap_or(key)
    }
}

/// Dispatch function template.
#[derive(Clone)]
pub struct DispatchFn {
    /// Function attributes.
    pub attrs: Vec<Attribute>,

    /// 'for' token signaling generics.
    pub for_token: Option<Token![for]>,
    /// Generic arguments.
    pub generics: Generics,

    /// 'as' token.
    pub as_token: Option<Token![as]>,
    /// Name of function.
    pub ident: Ident,

    /// Visibility.
    pub vis: Visibility,
    /// Is the function const.
    pub constness: Option<Token![const]>,
    /// Is the function async.
    pub asyncness: Option<Token![async]>,

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
    /// Check if a dispatch function may be at the lookahead position.
    ///
    /// May not register all possible tokens with lookahead if any is peeked.
    ///
    /// # Tokens
    /// `for, as, pub, const, async, fn`
    pub fn peek_prefix(lookahead: &Lookahead1) -> bool {
        lookahead.peek(Token![for])
            || lookahead.peek(Token![as])
            || lookahead.peek(Token![pub])
            || lookahead.peek(Token![const])
            || lookahead.peek(Token![async])
            || lookahead.peek(Token![fn])
    }

    /// Get a call path.
    fn call_path(&self, ty: &Type) -> PatPath {
        const fn empty_path() -> Path {
            Path {
                leading_colon: None,
                segments: Punctuated::new(),
            }
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
    fn call(&self, expr: Expr, ty: &Type, param_map: ParamMap) -> Box<Expr> {
        let DispatchParameters {
            paren_token,
            infix_comma,
            parameters,
            ..
        } = &self.parameters;
        let call = Box::new(Expr::from(ExprCall {
            attrs: Vec::new(),
            func: Box::new(self.call_path(ty).into()),
            paren_token: *paren_token,
            args: {
                let mut args = Punctuated::new();
                args.push(expr);

                if let Some(infix_comma) = infix_comma {
                    args.push_punct(*infix_comma);
                    args.extend(parameters.pairs().map(|pair| {
                        let (value, punct) = pair.into_tuple();
                        let value = param_map.get(&value.ident);
                        Pair::new(
                            Expr::Path(PatPath {
                                attrs: Vec::new(),
                                qself: None,
                                path: value.clone().into(),
                            }),
                            punct.copied(),
                        )
                    }));
                }

                args
            },
        }));

        if let Some(asyncness) = &self.asyncness {
            Box::new(Expr::from(ExprAwait {
                attrs: Vec::new(),
                base: call,
                dot_token: Token![.](asyncness.span),
                await_token: Token![await](asyncness.span),
            }))
        } else {
            call
        }
    }

    /// Pattern and body of match arm for a regular field.
    #[expect(clippy::too_many_arguments, reason = "helper function")]
    fn field_arm(
        &self,
        field: &Field,
        path: Path,
        this_ident: Ident,
        idx: usize,
        delim_span: DelimSpan,
        is_last: bool,
        param_map: ParamMap,
    ) -> (Pat, Box<Expr>) {
        let is_single = is_last && idx == 0;
        let pat = if let Some(ident) = &field.ident {
            Pat::from(PatStruct {
                attrs: Vec::new(),
                qself: None,
                path,
                brace_token: token::Brace { span: delim_span },
                fields: {
                    let mut punctuated = Punctuated::new();

                    punctuated.push(FieldPat {
                        attrs: Vec::new(),
                        member: Member::Named(ident.clone()),
                        colon_token: Some(Token![:](this_ident.span())),
                        pat: Box::new(Pat::from(PatPath {
                            attrs: Vec::new(),
                            qself: None,
                            path: Path::from(this_ident.clone()),
                        })),
                    });

                    if !is_single {
                        punctuated.push_punct(Token![,](delim_span.close()));
                    }

                    punctuated
                },
                rest: if is_single {
                    None
                } else {
                    Some(PatRest {
                        attrs: Vec::new(),
                        dot2_token: Token![..](delim_span.close()),
                    })
                },
            })
        } else {
            Pat::from(PatTupleStruct {
                attrs: Vec::new(),
                qself: None,
                path,
                paren_token: token::Paren { span: delim_span },
                elems: {
                    let mut punctuated = Punctuated::new();
                    let span = delim_span.open();
                    for _ in 0..idx {
                        punctuated.push(Pat::from(PatWild {
                            attrs: Vec::new(),
                            underscore_token: Token![_](span),
                        }));
                        punctuated.push_punct(Token![,](span));
                    }

                    punctuated.push(Pat::from(PatPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: Path::from(this_ident.clone()),
                    }));

                    if !is_last {
                        let span = delim_span.close();
                        punctuated.push_punct(Token![,](span));
                        punctuated.push(Pat::from(PatRest {
                            attrs: Vec::new(),
                            dot2_token: Token![..](span),
                        }));
                    }

                    punctuated
                },
            })
        };
        let expr = self.call(ident_to_expr(this_ident), &field.ty, param_map);
        (pat, expr)
    }

    /// Pattern and body of a unit arm.
    fn unit_arm(
        &self,
        fields: &Fields,
        path: Path,
        variant_ident: &Ident,
        param_map: ParamMap,
    ) -> (Pat, Box<Expr>) {
        let pat = match fields {
            Fields::Named(FieldsNamed { brace_token, .. }) => Pat::from(PatStruct {
                attrs: Vec::new(),
                qself: None,
                path,
                brace_token: *brace_token,
                fields: Punctuated::new(),
                rest: Some(PatRest {
                    attrs: Vec::new(),
                    dot2_token: Token![..](brace_token.span.open()),
                }),
            }),
            Fields::Unnamed(FieldsUnnamed { paren_token, .. }) => Pat::from(PatTupleStruct {
                attrs: Vec::new(),
                qself: None,
                path,
                paren_token: *paren_token,
                elems: [Pat::from(PatRest {
                    attrs: Vec::new(),
                    dot2_token: Token![..](paren_token.span.open()),
                })]
                .into_iter()
                .collect(),
            }),
            Fields::Unit => Pat::from(PatPath {
                attrs: Vec::new(),
                qself: None,
                path,
            }),
        };
        let expr = self
            .block
            .as_ref()
            .map(|block| {
                Box::new(Expr::from(ExprBlock {
                    attrs: Vec::new(),
                    label: None,
                    block: block.clone(),
                }))
            })
            .unwrap_or_else(|| {
                self.call(
                    {
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
                    },
                    &Type::from(TypeTuple {
                        paren_token: token::Paren(variant_ident.span()),
                        elems: Punctuated::new(),
                    }),
                    param_map,
                )
            });
        (pat, expr)
    }

    /// Determine if a field should be used or ignored based on attributes.
    fn read_field_attrs(&self, attrs: &[Attribute]) -> ::syn::Result<Option<FieldAttrInner>> {
        let mut acc = None;
        let attrs = attrs
            .iter()
            .filter(|attr| attr.path().is_ident("dispatch"))
            .map(|attr| attr.parse_args_with(Punctuated::<FieldAttr, Token![,]>::parse_terminated));

        for attrs in attrs {
            for attr in attrs? {
                match attr {
                    // If regular ignore_use, replace acc.
                    FieldAttr::Inner(ignore_use) => acc = Some(ignore_use),
                    // If named short-circuit on last nested attribute.
                    FieldAttr::Named(named) if named.name == self.ident => {
                        if let ignore_use @ Some(..) = named
                            .content
                            .content
                            .try_into_parsed_with(
                                Punctuated::<FieldAttrInner, Token![,]>::parse_terminated,
                            )?
                            .into_iter()
                            .last()
                        {
                            return Ok(ignore_use);
                        }
                    }
                    // If unknown named continue on.
                    FieldAttr::Named(..) => {}
                }
            }
        }
        Ok(acc)
    }

    /// Generate a match arm for given variant.
    fn match_arm(
        &self,
        ident: &Ident,
        variant: &Variant,
        this_ident: Ident,
        param_map: ParamMap,
    ) -> ::syn::Result<Arm> {
        let Variant {
            ident: variant_ident,
            fields: variant_fields,
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
        let (pat, body) = match variant_fields {
            Fields::Unit => self.unit_arm(variant_fields, path, variant_ident, param_map),
            Fields::Named(FieldsNamed {
                named: fields,
                brace_token: token::Brace { span: delim_span },
            })
            | Fields::Unnamed(FieldsUnnamed {
                unnamed: fields,
                paren_token: token::Paren { span: delim_span },
            }) => {
                let mut dispatch_field = Vec::new();
                let mut idx = 0;
                for (i, field) in fields.iter().enumerate() {
                    match self.read_field_attrs(&field.attrs)? {
                        // On ignore do nothing.
                        Some(FieldAttrInner::Ignore(..)) => {}

                        // On use ensure this field is dispatch field.
                        Some(FieldAttrInner::Use(..)) => {
                            dispatch_field.clear();
                            dispatch_field.push(field);
                            idx = i;
                            break;
                        }

                        // If no attribute matched assume this field is dispatch field.
                        None => {
                            dispatch_field.push(field);
                            idx = i;
                        }
                    }
                }

                // If no dispatch fields without ignore attribute are found, treat as unit variant.
                // If multiple found throw an error.
                match dispatch_field.as_slice() {
                    [] => self.unit_arm(variant_fields, path, variant_ident, param_map),
                    [dispatch_field] => self.field_arm(
                        dispatch_field,
                        path,
                        this_ident,
                        idx,
                        *delim_span,
                        idx + 1 == fields.len(),
                        param_map,
                    ),
                    [..] => {
                        return Err(::syn::Error::new(
                            delim_span.span(),
                            "expected only one field without the #[dispatch(ignore)] attribute \
                            or only one field with the #[dispatch(use)] attribute",
                        ));
                    }
                }
            }
        };

        Ok(Arm {
            attrs: Vec::new(),
            pat,
            guard: None,
            fat_arrow_token: Token![=>](Span::call_site()),
            body,
            comma: Some(Token![,](Span::call_site())),
        })
    }

    /// Create a function implementation for given enum.
    ///
    /// # Errors
    /// If enum variants cannot be converted to arms.
    pub fn to_item(&self, item_enum: &ItemEnum) -> ::syn::Result<ImplItem> {
        let sig = Signature {
            constness: self.constness,
            asyncness: self.asyncness,
            unsafety: None,
            abi: None,
            fn_token: self.fn_token,
            ident: self.ident.clone(),
            generics: self.generics.clone(),
            paren_token: self.parameters.paren_token,
            inputs: self.parameters.to_inputs(),
            variadic: None,
            output: self.output.clone(),
        };

        let mut mappings = BTreeMap::new();
        let mut attrs = Vec::new();

        for attr in &self.attrs {
            if !attr.path().is_ident("dispatch") {
                attrs.push(attr.clone());
            }

            match attr.parse_args::<DispatchFnAttr>()? {
                DispatchFnAttr::Remap(attr_remap) => mappings.extend(
                    attr_remap
                        .mappings
                        .into_iter()
                        .map(ParameterMapping::into_pair),
                ),
            }
        }

        let vis = self.vis.clone();
        let ident = &item_enum.ident;
        let this_ident = self.parameters.this_ident(Span::call_site());
        let expr = Expr::from(ExprMatch {
            attrs: Vec::new(),
            match_token: Token![match](Span::call_site()),
            expr: Box::new(ident_to_expr(Ident::from(
                self.parameters.receiver.self_token,
            ))),
            brace_token: token::Brace::default(),
            arms: item_enum
                .variants
                .iter()
                .map(|variant| {
                    self.match_arm(ident, variant, this_ident.clone(), ParamMap(&mappings))
                })
                .collect::<Result<Vec<_>, _>>()?,
        });

        let mut stmts = Vec::with_capacity(mappings.len() + 1);
        stmts.push(Stmt::Expr(expr, None));

        for (param, (colon_token, binding)) in &mappings {
            let span = colon_token.span;
            stmts.push(Stmt::Local(Local {
                attrs: Vec::new(),
                let_token: Token![let](span),
                pat: Pat::Path(PatPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: binding.clone().into(),
                }),
                init: Some(LocalInit {
                    eq_token: Token![=](span),
                    expr: Box::new(Expr::Path(PatPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: param.clone().into(),
                    })),
                    diverge: None,
                }),
                semi_token: Token![;](span),
            }));
        }

        let block = Block {
            brace_token: token::Brace::default(),
            stmts,
        };

        Ok(ImplItem::Fn(ImplItemFn {
            attrs,
            vis,
            defaultness: None,
            sig,
            block,
        }))
    }
}

impl ToTokens for DispatchFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            attrs,
            for_token,
            generics,
            as_token,
            ident,
            vis,
            constness,
            asyncness,
            fn_token,
            prefix,
            path,
            parameters,
            output,
            block,
            trailing_semi,
        } = self;

        attrs.iter().for_each(attr_writer::outer(tokens));
        for_token.to_tokens(tokens);
        generics.to_tokens(tokens);
        if let Some(as_token) = as_token {
            as_token.to_tokens(tokens);
            ident.to_tokens(tokens);
        }
        vis.to_tokens(tokens);
        constness.to_tokens(tokens);
        asyncness.to_tokens(tokens);
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
        let attrs = input.call(Attribute::parse_outer)?;
        let ((for_token, mut generics), (as_token, ident), vis, constness, asyncness, fn_token) =
            input
                .lookahead1()
                .chain_with_or_default(input, Token![for], |input| {
                    let for_token = input.parse::<Option<Token![for]>>()?;

                    if !input.peek(Token![<]) {
                        return Err(input.error("expected '<' after for"));
                    }

                    let generics = input.parse::<Generics>()?;

                    Ok((for_token, generics))
                })?
                .chain_with_or_default(input, Token![as], |input| {
                    let as_token = input.parse::<Option<Token![as]>>()?;
                    let ident = Some(input.parse::<Ident>()?);
                    Ok((as_token, ident))
                })?
                .chain_with_or(input, Token![pub], Visibility::parse, || {
                    Visibility::Inherited
                })?
                .chain::<Token![const]>(input, Token![const])?
                .chain::<Token![async]>(input, Token![async])?
                .finish::<Token![fn]>(input, Token![fn])?;

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

        let (lookahead, (output, where_clause)) = input
            .lookahead1()
            .chain_with_or(input, Token![->], ReturnType::parse, || ReturnType::Default)?
            .chain::<WhereClause>(input, Token![where])?;

        generics.where_clause = where_clause;

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
            attrs,
            for_token,
            generics,
            as_token,
            ident,
            vis,
            constness,
            asyncness,
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
