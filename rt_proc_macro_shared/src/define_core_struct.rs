use syn::{Expr, Ident, LitStr, Token, parse::Parse, token::Bracket};

use crate::common::{Attr, keywords};

pub struct FieldAst {
    pub attr: Attr,
    pub id: Ident,
    pub name: LitStr,
    pub ty: Expr,
}

impl Parse for FieldAst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr = input.parse()?;
        let id = input.parse()?;
        let name = input.parse()?;
        input.parse::<Token![=>]>()?;
        let ty = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(Self { attr, id, name, ty })
    }
}

pub struct DefineCoreStructAst {
    pub attr: Attr,
    pub assembly_name: Ident,
    pub id: Ident,
    pub name: LitStr,
    pub generic_bounds: Option<Expr>,
    pub fields: Vec<FieldAst>,
    pub method_ids: Vec<Ident>,
    pub static_method_ids: Vec<Ident>,
    pub method_generator: Expr,
}

impl Parse for DefineCoreStructAst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr = input.parse()?;
        let assembly_name = input.parse()?;
        let id = input.parse()?;
        let name = input.parse()?;
        input.parse::<Token![=>]>()?;
        let generic_bounds = if let Ok(brackets) = syn::__private::parse_brackets(input) {
            Some(brackets.content.parse()?)
        } else {
            None
        };

        input.parse::<Token![#]>()?;
        input.parse::<keywords::fields>()?;
        input.parse::<Token![:]>()?;
        let mut fields = Vec::new();
        while input.peek2(Bracket) {
            fields.push(input.parse()?);
        }

        input.parse::<Token![#]>()?;
        input.parse::<keywords::methods>()?;
        input.parse::<Token![:]>()?;
        let method_ids_buf;
        syn::bracketed!(method_ids_buf in input);
        let mut method_ids = Vec::new();
        while !method_ids_buf.is_empty() {
            let id = method_ids_buf.parse()?;
            method_ids.push(id);
        }
        let static_method_ids_buf;
        syn::bracketed!(static_method_ids_buf in input);
        let mut static_method_ids = Vec::new();
        while !static_method_ids_buf.is_empty() {
            let id = static_method_ids_buf.parse()?;
            static_method_ids.push(id);
        }
        input.parse::<keywords::with>()?;
        let method_generator = input.parse()?;
        Ok(Self {
            attr,
            assembly_name,
            id,
            name,
            generic_bounds,
            fields,
            method_ids,
            static_method_ids,
            method_generator,
        })
    }
}
