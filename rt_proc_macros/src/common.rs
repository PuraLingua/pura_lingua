use proc_macro2::TokenStream;
use syn::{Token, parse::Parse};

pub(crate) mod keywords {
    use syn::custom_keyword;

    custom_keyword!(fields);
    custom_keyword!(methods);
    custom_keyword!(of);
    custom_keyword!(with);
}

#[allow(unused)]
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
