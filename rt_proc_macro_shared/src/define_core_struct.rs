use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{Expr, Ident, LitStr, Token, parse::Parse, token::Bracket};

use crate::common::{Attr, GenericCount, Parameter, keywords};

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

#[derive(Clone)]
pub struct MethodAst {
    pub attr: Attr,
    pub id: Ident,
    pub generic_count: Option<GenericCount>,
    pub name: LitStr,
    pub parameters: Vec<Parameter>,
    pub return_type: Expr,
}

impl PartialEq for MethodAst {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Parse for MethodAst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr = input.parse()?;
        let id: Ident = input.parse()?;
        let generic_count = if GenericCount::is_possible(&input) {
            Some(input.parse()?)
        } else {
            None
        };
        let name = match input.parse::<LitStr>() {
            Ok(x) => x,
            Err(_) => LitStr::new(&id.to_string(), id.span()),
        };

        let parameter_tokens;
        syn::parenthesized!(parameter_tokens in input);
        let mut parameters = Vec::new();
        while !parameter_tokens.is_empty() {
            parameters.push(parameter_tokens.parse()?);
        }

        input.parse::<Token![->]>()?;

        let return_type = input.parse()?;

        input.parse::<Token![;]>()?;

        Ok(Self {
            attr,
            id,
            generic_count,
            name,
            parameters,
            return_type,
        })
    }
}

pub struct DefineCoreStructAst {
    pub attr: Attr,
    pub assembly_name: Ident,
    pub id: Ident,
    pub generic_count: Option<GenericCount>,
    pub name: LitStr,
    pub generic_bounds: Option<Expr>,
    pub fields: Vec<FieldAst>,
    pub method_ids: Vec<MethodAst>,
    pub static_method_ids: Vec<MethodAst>,
    pub method_generator: Expr,
}

impl Parse for DefineCoreStructAst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr = input.parse()?;
        let assembly_name = input.parse()?;
        let id = input.parse()?;
        let generic_count = if GenericCount::is_possible(&input) {
            Some(input.parse()?)
        } else {
            None
        };
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
            generic_count,
            name,
            generic_bounds,
            fields,
            method_ids,
            static_method_ids,
            method_generator,
        })
    }
}

impl DefineCoreStructAst {
    pub fn field_id_enum_ident(&self) -> Ident {
        format_ident!("{}_FieldId", self.id)
    }
    pub fn define_fields(&self, attrs: &[TokenStream]) -> TokenStream {
        let field_id_enum_ident = self.field_id_enum_ident();
        let field_ids = self.fields.iter().map(|x| &x.id).collect::<Vec<_>>();

        quote::quote! {
            #[repr(u32)]
            #[derive(Clone, Copy)]
            #(#[#attrs])*
            pub enum #field_id_enum_ident {
                #(
                    #field_ids,
                )*

                #[doc(hidden)]
                __END,
            }
        }
    }
    pub fn method_id_enum_ident(&self) -> Ident {
        format_ident!("{}_MethodId", self.id)
    }
    pub fn define_methods(&self, attrs: &[TokenStream]) -> TokenStream {
        let method_id_enum_ident = self.method_id_enum_ident();
        let method_ids = self.method_ids.iter().map(|x| &x.id).collect::<Vec<_>>();

        quote::quote! {
            #[repr(u32)]
            #[derive(Clone, Copy)]
            #(#[#attrs])*
            pub enum #method_id_enum_ident {
                #(
                    #method_ids,
                )*

                #[doc(hidden)]
                __END,
            }
        }
    }
    pub fn static_method_id_enum_ident(&self) -> Ident {
        format_ident!("{}_StaticMethodId", self.id)
    }
    pub fn define_static_methods(&self, attrs: &[TokenStream]) -> TokenStream {
        let static_method_id_enum_ident = self.static_method_id_enum_ident();

        let method_id_enum_ident = self.method_id_enum_ident();

        let static_method_ids = self
            .static_method_ids
            .iter()
            .map(|x| &x.id)
            .collect::<Vec<_>>();

        quote::quote! {
            #[repr(u32)]
            #[derive(Clone, Copy)]
            #(#[#attrs])*
            pub enum #static_method_id_enum_ident {
                StaticConstructor = #method_id_enum_ident::__END as u32,
                #(
                    #static_method_ids,
                )*
            }
        }
    }
}
