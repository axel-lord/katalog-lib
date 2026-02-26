//! Ast fo parameters of dispatch functions.

use ::std::collections::BTreeSet;

use ::proc_macro2::{Span, TokenStream};
use ::quote::ToTokens;
use ::syn::{
    Attribute, FnArg, Ident, Pat, PatIdent, PatType, Receiver, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::{Pair, Punctuated},
    token,
};

/// A single non receiver dispatch function template parameter.
#[derive(Clone)]
pub struct IdentType {
    /// Parameter attributes.
    pub attrs: Vec<Attribute>,
    /// Parameter ident, unlinke with normal parameters may not be a pattern.
    pub ident: Ident,
    /// ':' token.
    pub colon_token: Token![:],
    /// Type of parameter.
    pub ty: Box<Type>,
}

impl ToTokens for IdentType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            attrs,
            ident,
            colon_token,
            ty,
        } = self;
        for attr in attrs {
            attr.to_tokens(tokens);
        }
        ident.to_tokens(tokens);
        colon_token.to_tokens(tokens);
        ty.to_tokens(tokens);
    }
}

impl Parse for IdentType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

impl From<IdentType> for PatType {
    fn from(value: IdentType) -> Self {
        let IdentType {
            attrs,
            ident,
            colon_token,
            ty,
        } = value;
        PatType {
            attrs,
            pat: Box::new(Pat::from(PatIdent {
                ident,
                attrs: Vec::new(),
                by_ref: None,
                mutability: None,
                subpat: None,
            })),
            colon_token,
            ty,
        }
    }
}

impl From<IdentType> for FnArg {
    fn from(value: IdentType) -> Self {
        FnArg::Typed(value.into())
    }
}
/// Parameters of dispatch function template.
#[derive(Clone)]
pub struct DispatchParameters {
    /// Enclosing perentheses.
    pub paren_token: token::Paren,
    /// Required receiver as first argument, (self, &self, self: Arc<Self>, etc.)
    pub receiver: Receiver,
    /// Comma separating receiver and remaining parameters if any.
    pub infix_comma: Option<Token![,]>,
    /// Additional parameters.
    pub parameters: Punctuated<IdentType, Token![,]>,
}

impl DispatchParameters {
    /// Get inputs for function signature.
    pub fn to_inputs(&self) -> Punctuated<FnArg, Token![,]> {
        let mut inputs = Punctuated::new();
        inputs.push_value(self.receiver.clone().into());

        if let Some(infix_comma) = self.infix_comma {
            fn pair_map(pair: Pair<&IdentType, &Token![,]>) -> Pair<FnArg, Token![,]> {
                let (value, punct) = pair.into_tuple();
                let punct = punct.copied();
                let value = <FnArg as From<IdentType>>::from(value.clone());
                Pair::new(value, punct)
            }

            inputs.push_punct(infix_comma);
            inputs.extend(self.parameters.pairs().map(pair_map));
        }

        inputs
    }

    /// Get an ident for self not without any overlap with parameters.
    pub fn this_ident(&self, span: Span) -> Ident {
        let set = self
            .parameters
            .iter()
            .map(|param| {
                const RAW_PREFIX: &str = "r#";
                let mut ident = param.ident.to_string();
                if ident.starts_with(RAW_PREFIX) {
                    ident.drain(..RAW_PREFIX.len()).for_each(drop);
                }
                ident
            })
            .collect::<BTreeSet<_>>();

        const IDENT_PREFIX: &str = "__this";

        if !set.contains(IDENT_PREFIX) {
            return Ident::new(IDENT_PREFIX, span);
        }

        for i in 1..=self.parameters.len() {
            let ident = format!("{IDENT_PREFIX}{i}");
            if !set.contains(&ident) {
                return Ident::new(&ident, span);
            }
        }

        unreachable!("more idents than parameters equal to some parameter")
    }
}

impl ToTokens for DispatchParameters {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            paren_token,
            receiver,
            infix_comma,
            parameters,
        } = self;
        paren_token.surround(tokens, |tokens| {
            receiver.to_tokens(tokens);
            infix_comma.to_tokens(tokens);
            parameters.to_tokens(tokens);
        });
    }
}

impl Parse for DispatchParameters {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        let receiver = content.parse()?;

        let (infix_comma, parameters) = if !content.is_empty() {
            (
                Some(content.parse()?),
                content.call(Punctuated::parse_terminated)?,
            )
        } else {
            (None, Punctuated::default())
        };

        Ok(Self {
            paren_token,
            receiver,
            infix_comma,
            parameters,
        })
    }
}
