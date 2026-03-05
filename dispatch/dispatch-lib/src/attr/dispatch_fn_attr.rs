//! [DispatchFnAttr] impl.

use crate::{kw, mono_closure::MonoClosure};

use ::katalog_lib_proc_macro_common::dyn_attr::DynAttrContent;
use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Block, Expr, ExprBlock, ExprCall, ExprLet, Ident, Pat, PatPath, Stmt, Token, braced,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token,
};

/// Attribute on dispatch function.
#[derive(Clone)]
pub enum DispatchFnAttr {
    /// Remap parameters in function.
    Remap(AttrRemap),
    /// Map result of calls.
    Map(AttrMap),
}

impl Parse for DispatchFnAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::remap) {
            Ok(Self::Remap(input.parse()?))
        } else if lookahead.peek(kw::map) {
            Ok(Self::Map(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for DispatchFnAttr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DispatchFnAttr::Remap(attr_remap) => attr_remap.to_tokens(tokens),
            DispatchFnAttr::Map(attr_map) => attr_map.to_tokens(tokens),
        }
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
    /// Map using a path to a function.
    Path {
        /// Map keyword.
        map_kw: kw::map,
        /// Path to map using.
        path: DynAttrContent<PatPath>,
    },
}

impl AttrMap {
    /// Wrap an expression with either closure or path.
    pub fn wrap_expr(&self, expr: Expr) -> Expr {
        match self {
            AttrMap::Closure { map_kw, closure } => Expr::Block(ExprBlock {
                attrs: Vec::new(),
                label: None,
                block: Block {
                    brace_token: token::Brace(map_kw.span),
                    stmts: Vec::from_iter([
                        Stmt::Expr(
                            Expr::Let(ExprLet {
                                attrs: Vec::new(),
                                let_token: Token![let](map_kw.span),
                                pat: Box::new(Pat::Path(PatPath {
                                    attrs: Vec::new(),
                                    qself: None,
                                    path: closure.param.clone().into(),
                                })),
                                eq_token: Token![=](map_kw.span),
                                expr: Box::new(expr),
                            }),
                            Some(Token![;](map_kw.span)),
                        ),
                        Stmt::Expr(closure.expr.clone(), None),
                    ]),
                },
            }),
            AttrMap::Path { map_kw, path } => Expr::Call(ExprCall {
                attrs: Vec::new(),
                func: Box::new(Expr::Path(path.value().clone())),
                paren_token: token::Paren(map_kw.span),
                args: {
                    let mut args = Punctuated::new();
                    args.push(expr);
                    args
                },
            }),
        }
    }
}

impl Parse for AttrMap {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let map_kw = input.parse()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![|]) {
            let closure = input.parse()?;
            Ok(Self::Closure { map_kw, closure })
        } else if DynAttrContent::peek_lookahead(&lookahead) {
            let path = input.parse()?;
            Ok(Self::Path { map_kw, path })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for AttrMap {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            AttrMap::Closure { map_kw, closure } => {
                map_kw.to_tokens(tokens);
                closure.to_tokens(tokens);
            }
            AttrMap::Path { map_kw, path } => {
                map_kw.to_tokens(tokens);
                path.to_tokens(tokens);
            }
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
