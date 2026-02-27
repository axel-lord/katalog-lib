//! Closure like type.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Expr, Ident, Token,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

/// Type alias for head and tail params.
type Params = Punctuated<Ident, Token![,]>;

/// Closure like with only one parameter.
#[derive(Clone)]
pub struct PsuedoClosure {
    /// left '|' token.
    pub left_pipe: Token![|],

    /// Initial parameters.
    pub head_params: Params,
    /// '..' token.
    pub rest_token: Option<Token![..]>,
    /// Comma separating rest and tail params.
    pub rest_comma: Option<Token![,]>,
    /// Final parameters.
    pub tail_params: Params,

    /// right '|' token.
    pub right_pipe: Token![|],
    /// Closure expression.
    pub expr: Expr,
}

impl PsuedoClosure {
    /// Get head and tail parameters.
    pub const fn params(&self) -> [&Punctuated<Ident, Token![,]>; 2] {
        [&self.head_params, &self.tail_params]
    }

    /// Returns true if closure has a rest parameter.
    pub const fn captures_rest(&self) -> bool {
        self.rest_token.is_some()
    }
}

/// Parse tail params.
fn parse_tail_inner(input: ParseStream) -> ::syn::Result<Params> {
    let mut tail = Params::new();
    loop {
        let mut lookahead = input.lookahead1();
        if lookahead.peek(Token![|]) {
            break;
        }

        if lookahead.peek(Ident::peek_any) {
            tail.push(input.parse()?);

            lookahead = input.lookahead1();
            if lookahead.peek(Token![|]) {
                break;
            }

            if lookahead.peek(Token![,]) {
                tail.push_punct(input.parse()?);
                continue;
            }
        }
        return Err(lookahead.error());
    }

    Ok(tail)
}

/// Parse remainder after '..'.
fn parse_tail(input: ParseStream) -> ::syn::Result<(Option<Token![,]>, Params)> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![|]) {
        Ok(Default::default())
    } else if lookahead.peek(Token![,]) {
        let rest_comma = Some(input.parse()?);
        let params = input.call(parse_tail_inner)?;

        Ok((rest_comma, params))
    } else {
        Err(lookahead.error())
    }
}

/// Parse remainder including '..'.
fn parse_rest(
    input: ParseStream,
) -> ::syn::Result<(Option<Token![..]>, Option<Token![,]>, Params)> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![|]) {
        Ok(Default::default())
    } else if lookahead.peek(Token![..]) {
        let rest_token = Some(input.parse()?);
        let (rest_comma, tail) = input.call(parse_tail)?;
        Ok((rest_token, rest_comma, tail))
    } else {
        Err(lookahead.error())
    }
}

impl Parse for PsuedoClosure {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let left_pipe = input.parse()?;

        let mut head_params = Punctuated::new();

        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![|]) || lookahead.peek(Token![..]) {
                break;
            } else if lookahead.peek(Ident::peek_any) {
                head_params.push(input.parse()?);
                let lookahead = input.lookahead1();
                if lookahead.peek(Token![,]) {
                    head_params.push_punct(input.parse()?);
                } else if lookahead.peek(Token![|]) {
                    break;
                } else {
                    return Err(lookahead.error());
                }
            } else {
                return Err(lookahead.error());
            }
        }

        let (rest_token, rest_comma, tail_params) = input.call(parse_rest)?;

        let right_pipe = input.parse()?;
        let expr = input.parse()?;

        Ok(Self {
            left_pipe,
            head_params,
            rest_token,
            rest_comma,
            tail_params,
            right_pipe,
            expr,
        })
    }
}

impl ToTokens for PsuedoClosure {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            left_pipe,
            head_params,
            rest_token: rest,
            rest_comma,
            tail_params,
            right_pipe,
            expr,
        } = self;
        left_pipe.to_tokens(tokens);
        head_params.to_tokens(tokens);
        rest.to_tokens(tokens);
        rest_comma.to_tokens(tokens);
        tail_params.to_tokens(tokens);
        right_pipe.to_tokens(tokens);
        expr.to_tokens(tokens);
    }
}
