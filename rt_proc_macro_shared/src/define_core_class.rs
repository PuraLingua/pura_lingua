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

pub struct DefineCoreClassAst {
    pub attr: Attr,
    pub assembly_name: Ident,
    pub id: Ident,
    pub name: LitStr,
    pub parent: Option<Expr>,
    pub generic_bounds: Option<Expr>,
    pub field_parent: Option<Ident>,
    pub fields: Vec<FieldAst>,

    pub method_parent: Option<Ident>,
    pub method_ids: Vec<Ident>,
    pub overriding_method_ids: Vec<Ident>,
    pub static_method_ids: Vec<Ident>,
    pub method_generator: Expr,
}

impl Parse for DefineCoreClassAst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr = input.parse()?;
        let assembly_name = input.parse()?;
        let id = input.parse()?;
        let name = input.parse()?;
        let parent = if !input.peek(Token![=>]) {
            Some(input.parse()?)
        } else {
            None
        };
        input.parse::<Token![=>]>()?;
        let generic_bounds = if let Ok(brackets) = syn::__private::parse_brackets(input) {
            Some(brackets.content.parse()?)
        } else {
            None
        };

        input.parse::<Token![#]>()?;
        input.parse::<keywords::fields>()?;
        let field_parent = if input.peek(keywords::of) {
            input.parse::<keywords::of>()?;
            Some(input.parse()?)
        } else {
            None
        };
        input.parse::<Token![:]>()?;
        let mut fields = Vec::new();
        while input.peek2(Bracket) {
            fields.push(input.parse()?);
        }

        input.parse::<Token![#]>()?;
        input.parse::<keywords::methods>()?;
        let method_parent = if input.peek(keywords::of) {
            input.parse::<keywords::of>()?;
            Some(input.parse()?)
        } else {
            None
        };
        input.parse::<Token![:]>()?;

        let method_ids_buf;
        syn::bracketed!(method_ids_buf in input);
        let mut method_ids: Vec<(Option<Token![override]>, syn::Ident)> = Vec::new();
        while !method_ids_buf.is_empty() {
            let overridable = method_ids_buf.parse()?;
            let id = method_ids_buf.parse()?;
            method_ids.push((overridable, id));
        }
        method_ids.sort_by(|(a, _), (b, _)| match (a, b) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (Some(_), Some(_)) => std::cmp::Ordering::Equal,
        });
        let mid_index = method_ids
            .iter()
            .position(|x| x.0.is_none())
            .unwrap_or(method_ids.len());

        let static_method_ids_buf;
        syn::bracketed!(static_method_ids_buf in input);
        let mut static_method_ids = Vec::new();
        while !static_method_ids_buf.is_empty() {
            static_method_ids.push(static_method_ids_buf.parse()?);
        }
        input.parse::<keywords::with>()?;
        let method_generator = input.parse()?;
        Ok(Self {
            attr,
            assembly_name,
            id,
            name,
            parent,
            generic_bounds,
            field_parent,
            fields,
            method_parent,
            method_ids: method_ids[mid_index..]
                .iter()
                .map(|x| &x.1)
                .cloned()
                .collect(),
            overriding_method_ids: method_ids[..mid_index]
                .iter()
                .map(|x| &x.1)
                .cloned()
                .collect(),
            static_method_ids,
            method_generator,
        })
    }
}
