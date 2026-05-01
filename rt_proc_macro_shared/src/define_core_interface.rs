use proc_macro2::{Span, TokenStream};
use syn::{
    Expr, ExprBinary, Ident, LitInt, LitStr, Token, parenthesized, parse::Parse, spanned::Spanned,
};

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
/// <attr> <id> [name] (<parameters>) -> <return_type>;
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
        parenthesized!(parameter_tokens in input);
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

pub struct DefineCoreInterfaceAst {
    pub attr: Attr,
    pub assembly_name: Ident,
    pub id: Ident,
    pub generic_count: Option<GenericCount>,
    pub name: LitStr,
    pub required_interfaces: Vec<Expr>,
    pub generic_bounds: Option<Expr>,

    pub method_ids: Vec<MethodAst>,
}

impl Parse for DefineCoreInterfaceAst {
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
        let mut required_interfaces = Vec::new();
        while !input.peek(Token![=>]) {
            required_interfaces.push(input.parse()?);
            if input.parse::<Token![,]>().is_err() {
                break;
            }
        }
        input.parse::<Token![=>]>()?;
        let generic_bounds = if let Ok(brackets) = syn::__private::parse_brackets(input) {
            Some(brackets.content.parse()?)
        } else {
            None
        };

        input.parse::<Token![#]>()?;
        input.parse::<keywords::methods>()?;
        input.parse::<Token![:]>()?;

        let method_ids_buf;
        syn::bracketed!(method_ids_buf in input);
        let mut method_ids = Vec::new();
        while !method_ids_buf.is_empty() {
            let id: MethodAst = method_ids_buf.parse()?;
            method_ids.push(id);
        }

        Ok(Self {
            attr,
            assembly_name,
            id,
            generic_count,
            name,
            required_interfaces,
            generic_bounds,
            method_ids: method_ids,
        })
    }
}

impl DefineCoreInterfaceAst {
    pub fn method_id_contents(&self) -> syn::Result<Vec<Expr>> {
        self.method_ids
            .iter()
            .try_fold(Vec::new(), |mut out: Vec<Expr>, _| {
                if let Some(last) = out.last() {
                    out.push(Expr::Binary(ExprBinary {
                        attrs: Vec::new(),
                        left: Box::new(Expr::Lit(syn::ExprLit {
                            attrs: Vec::new(),
                            lit: syn::Lit::Int(LitInt::new(
                                "1u32",
                                Span::call_site().resolved_at(last.span()),
                            )),
                        })),
                        op: syn::BinOp::Add(Token![+](Span::call_site())),
                        right: Box::clone_from_ref(last),
                    }));
                } else {
                    out.push(syn::Expr::Lit(syn::ExprLit {
                        attrs: Vec::new(),
                        lit: syn::Lit::Int(LitInt::new("0u32", Span::call_site())),
                    }));
                }
                Ok::<_, syn::Error>(out)
            })
    }

    pub fn method_id_enum_ident(&self) -> Ident {
        Ident::new("MethodId", Span::call_site())
    }
    pub fn define_methods(&self, attrs: &[TokenStream]) -> syn::Result<TokenStream> {
        let method_id_enum_ident = self.method_id_enum_ident();

        let method_ids = self.method_ids.iter().map(|x| &x.id).collect::<Vec<_>>();
        let method_id_contents = self.method_id_contents()?;

        let method_end_content = if method_id_contents.is_empty() {
            quote::quote!(= 0u32)
        } else {
            quote::quote!()
        };

        Ok(quote::quote! {
            #[repr(u32)]
            #[derive(Clone, Copy)]
            #(#[#attrs])*
            pub enum #method_id_enum_ident {
                #(
                    #method_ids = #method_id_contents,
                )*

                #[doc(hidden)]
                __END #method_end_content,
            }
        })
    }
}
