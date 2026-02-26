use proc_macro2::TokenStream;
use syn::{Expr, LitInt, Token, parse::Parse};

pub(crate) mod keywords {
    use syn::custom_keyword;

    custom_keyword!(fields);
    custom_keyword!(methods);
    custom_keyword!(of);
    custom_keyword!(with);
}

#[allow(unused)]
#[derive(Clone)]
pub struct Attr {
    pub pound: Token![#],
    pub bracket: syn::token::Bracket,
    pub inner: TokenStream,
}

impl Parse for Attr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pound = input.parse()?;
        let inner;
        let bracket = syn::bracketed!(inner in input);

        Ok(Attr {
            pound,
            bracket,
            inner: inner.parse()?,
        })
    }
}

#[allow(unused)]
#[derive(Clone)]
pub struct Parameter {
    pub attr: Attr,
    pub ty: Expr,
}

impl Parse for Parameter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr = input.parse()?;
        let ty = input.parse()?;

        Ok(Self { attr, ty })
    }
}

#[derive(Clone)]
pub struct GenericCount {
    pub count: LitInt,
    pub is_infinite: Option<Token![+]>,
}

impl GenericCount {
    pub fn is_possible(input: &syn::parse::ParseStream) -> bool {
        input.peek(LitInt)
    }
}

impl Parse for GenericCount {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let count = input.parse()?;
        let is_infinite = input.parse().ok();
        Ok(Self { count, is_infinite })
    }
}
