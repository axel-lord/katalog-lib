//! Path prefix ast.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Path, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

/// A Qualified type '<Ty as ::path>'
pub struct Qualified<T> {
    /// '<' token.
    pub lt_token: Token![<],
    /// Qualified value.
    pub value: T,
    /// 'as' token.
    pub as_token: Token![as],
    /// Path to qualify as.
    pub path: Path,
    /// '>' token.
    pub gt_token: Token![>],
}

impl<T: ToTokens> ToTokens for Qualified<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            lt_token,
            value,
            as_token,
            path,
            gt_token,
        } = self;
        lt_token.to_tokens(tokens);
        value.to_tokens(tokens);
        as_token.to_tokens(tokens);
        path.to_tokens(tokens);
        gt_token.to_tokens(tokens);
    }
}

impl<T> Parse for Qualified<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            lt_token: input.parse()?,
            value: input.parse()?,
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
    }
}

/// Prefix to path of function.
pub enum PathPrefix {
    /// Self type replaced by variant type.
    SelfTy {
        /// '<' token.
        lt_token: Token![<],
        /// 'Self' type.
        self_ty: Token![Self],
        /// '>' token.
        gt_token: Token![>],
    },
    /// Qulified self type.
    QualifiedSelf(Qualified<Token![Self]>),
    /// Qualified type.
    Qualified(Qualified<Box<Type>>),
}

impl ToTokens for PathPrefix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PathPrefix::SelfTy {
                lt_token,
                self_ty,
                gt_token,
            } => {
                lt_token.to_tokens(tokens);
                self_ty.to_tokens(tokens);
                gt_token.to_tokens(tokens);
            }
            PathPrefix::QualifiedSelf(qualified) => qualified.to_tokens(tokens),
            PathPrefix::Qualified(qualified) => qualified.to_tokens(tokens),
        }
    }
}
