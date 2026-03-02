//! Closure like type.

use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    Expr, Ident, Pat, PatPath, PatWild, Token,
    ext::IdentExt,
    parse::{Lookahead1, Parse, ParseStream},
    punctuated::Punctuated,
};

/// Type alias for head and tail params.
type Params = Punctuated<PsuedoClosureParam, Token![,]>;

/// Parameter of psuedo closure.
#[derive(Clone)]
pub enum PsuedoClosureParam {
    /// Parame}ter is a wildcard.
    Wild(Token![_]),
    /// Parameter is named.
    Ident(Ident),
}

impl PsuedoClosureParam {
    /// Get ident if available.
    pub const fn ident(&self) -> Option<&Ident> {
        if let Self::Ident(ident) = self {
            Some(ident)
        } else {
            None
        }
    }

    /// Peek if psuedo closure parameter may be parsed.
    pub fn peek(lookahead: &Lookahead1) -> bool {
        lookahead.peek(Token![_]) || lookahead.peek(Ident::peek_any)
    }
}

impl From<PsuedoClosureParam> for Pat {
    fn from(value: PsuedoClosureParam) -> Self {
        match value {
            PsuedoClosureParam::Wild(underscore_token) => Pat::Wild(PatWild {
                attrs: Vec::new(),
                underscore_token,
            }),
            PsuedoClosureParam::Ident(ident) => Pat::Path(PatPath {
                attrs: Vec::new(),
                qself: None,
                path: ident.into(),
            }),
        }
    }
}

impl Parse for PsuedoClosureParam {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![_]) {
            input.parse().map(Self::Wild)
        } else if lookahead.peek(Ident::peek_any) {
            input.parse().map(Self::Ident)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for PsuedoClosureParam {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            PsuedoClosureParam::Wild(underscore) => underscore.to_tokens(tokens),
            PsuedoClosureParam::Ident(ident) => ident.to_tokens(tokens),
        }
    }
}

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
    pub const fn params(&self) -> [&Params; 2] {
        [&self.head_params, &self.tail_params]
    }

    /// Returns true if closure has a rest parameter.
    pub const fn captures_rest(&self) -> bool {
        self.rest_token.is_some()
    }

    /// Returns lenght of head and tail parameters combined.
    pub fn parameters_len(&self) -> usize {
        self.head_params.len() + self.tail_params.len()
    }

    /// Get all parameters.
    pub fn parameters(&self) -> impl DoubleEndedIterator<Item = &'_ PsuedoClosureParam> {
        let Self {
            head_params,
            tail_params,
            ..
        } = self;
        head_params.iter().chain(tail_params.iter())
    }

    /// Get all parameters as mutable.
    pub fn parameters_mut(
        &mut self,
    ) -> impl DoubleEndedIterator<Item = &'_ mut PsuedoClosureParam> {
        let Self {
            head_params,
            tail_params,
            ..
        } = self;
        head_params.iter_mut().chain(tail_params.iter_mut())
    }

    /// Convert into parameters.
    pub fn into_parameters(self) -> impl DoubleEndedIterator<Item = PsuedoClosureParam> {
        let Self {
            head_params,
            tail_params,
            ..
        } = self;
        head_params.into_iter().chain(tail_params)
    }

    /// Zip parameters against a double ended iterator of values.
    ///
    /// # Errors
    /// If there are not enough items in the iterator, the length of the iterator is returned.
    pub fn zip_with<I>(&self, items: I) -> Result<Vec<(I::Item, &PsuedoClosureParam)>, usize>
    where
        I: IntoIterator,
        I::IntoIter: DoubleEndedIterator,
    {
        let mut outputs = Vec::new();
        let mut items = items.into_iter();

        let [head_params, tail_params] = self.params();

        // Add head parameters.
        for param in head_params {
            let Some(item) = items.next() else {
                return Err(outputs.len());
            };
            outputs.push((item, param));
        }

        // Save point of vec where tail parameters start.
        let rest_point = outputs.len();

        // Add tail parameters read in reverse.
        for param in tail_params.iter().rev() {
            let Some(item) = items.next_back() else {
                return Err(outputs.len());
            };
            outputs.push((item, param));
        }

        // Correct order of tail parameters.
        if let Some(tail) = outputs.get_mut(rest_point..) {
            tail.reverse();
        }

        Ok(outputs)
    }
}

/// Parse tail params.
fn parse_tail_inner(input: ParseStream) -> ::syn::Result<Params> {
    let mut tail = Params::new();
    loop {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![|]) {
            return Ok(tail);
        }

        if !PsuedoClosureParam::peek(&lookahead) {
            return Err(lookahead.error());
        }

        tail.push(input.parse()?);

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![|]) {
            return Ok(tail);
        }

        if !lookahead.peek(Token![,]) {
            return Err(lookahead.error());
        }

        tail.push_punct(input.parse()?);
    }
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

/// Parse initial parameters.
fn parse_head(input: ParseStream) -> ::syn::Result<Params> {
    let mut head = Params::new();
    loop {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![|]) || lookahead.peek(Token![..]) {
            return Ok(head);
        }

        if !PsuedoClosureParam::peek(&lookahead) {
            return Err(lookahead.error());
        }

        head.push(input.parse()?);

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![|]) {
            return Ok(head);
        }

        if !lookahead.peek(Token![,]) {
            return Err(lookahead.error());
        }

        head.push_punct(input.parse()?);
    }
}

impl Parse for PsuedoClosure {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let left_pipe = input.parse()?;

        let head_params = input.call(parse_head)?;
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
            rest_token,
            rest_comma,
            tail_params,
            right_pipe,
            expr,
        } = self;
        left_pipe.to_tokens(tokens);
        head_params.to_tokens(tokens);
        rest_token.to_tokens(tokens);
        rest_comma.to_tokens(tokens);
        tail_params.to_tokens(tokens);
        right_pipe.to_tokens(tokens);
        expr.to_tokens(tokens);
    }
}
