use std::sync::OnceLock;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::format_ident;
use syn::{
    Expr, ExprBinary, ExprCast, Ident, LitInt, LitStr, PathSegment, Token, TypePath, parenthesized,
    parse::Parse, spanned::Spanned, token::Bracket,
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

impl MethodAst {
    fn is_override(&self) -> bool {
        fn contains(ts: proc_macro2::TokenStream) -> bool {
            for x in ts.into_iter() {
                match x {
                    TokenTree::Ident(id) if id == "override" => return true,
                    TokenTree::Group(_) => break,
                    _ => (),
                }
            }
            false
        }
        contains(self.attr.inner.clone())
    }
}

pub struct DefineCoreClassAst {
    pub attr: Attr,
    pub assembly_name: Ident,
    pub id: Ident,
    pub generic_count: Option<GenericCount>,
    pub name: LitStr,
    pub parent: Option<Expr>,
    pub generic_bounds: Option<Expr>,
    pub field_parent: Option<Ident>,
    pub fields: Vec<FieldAst>,

    pub method_parent: Option<Ident>,
    pub method_ids: Vec<MethodAst>,
    pub overriding_method_ids: Vec<MethodAst>,
    pub static_method_ids: Vec<MethodAst>,
    pub method_generator: Expr,
}

impl Parse for DefineCoreClassAst {
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
        let mut method_ids: Vec<(bool, MethodAst)> = Vec::new();
        while !method_ids_buf.is_empty() {
            let id: MethodAst = method_ids_buf.parse()?;
            method_ids.push((id.is_override(), id));
        }
        method_ids.sort_by(|(a, _), (b, _)| match (*a, *b) {
            (false, false) => std::cmp::Ordering::Equal,
            (false, true) => std::cmp::Ordering::Greater,
            (true, false) => std::cmp::Ordering::Less,
            (true, true) => std::cmp::Ordering::Equal,
        });
        let mid_index = method_ids
            .iter()
            .position(|x| !x.0)
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
            generic_count,
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

impl DefineCoreClassAst {
    pub fn field_id_contents(&self) -> Vec<Expr> {
        self.fields.iter().fold(Vec::new(), |mut out, _| {
            if let Some(last) = out.last().cloned() {
                out.push(syn::parse2(quote::quote!(1u32 + #last)).unwrap());
            } else {
                out.push(match self.field_parent.as_ref() {
                    Some(parent) => syn::parse2(quote::quote!(#parent::__END as u32)).unwrap(),
                    None => syn::Expr::Lit(syn::ExprLit {
                        attrs: Vec::new(),
                        lit: syn::Lit::Int(LitInt::new("0u32", Span::call_site())),
                    }),
                });
            }
            out
        })
    }
    pub fn overriding_method_id_contents(&self) -> syn::Result<Vec<Expr>> {
        let once_parent = OnceLock::new();
        self.overriding_method_ids
            .iter()
            .try_fold(Vec::new(), |mut out, id| {
                let parent = once_parent.get_or_try_init(|| {
                    self.method_parent.clone().ok_or(syn::Error::new(
                        Span::call_site(),
                        "Override methods require method parents",
                    ))
                })?;
                out.push(syn::Expr::Cast(ExprCast {
                    attrs: Vec::new(),
                    expr: Box::new(Expr::Path(syn::ExprPath {
                        attrs: Vec::new(),
                        // cSpell:disable-next-line
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: vec![
                                PathSegment {
                                    ident: parent.clone(),
                                    arguments: syn::PathArguments::None,
                                },
                                PathSegment {
                                    ident: id.id.clone(),
                                    arguments: syn::PathArguments::None,
                                },
                            ]
                            .into_iter()
                            .collect(),
                        },
                    })),
                    as_token: Token![as](Span::call_site().resolved_at(parent.span())),
                    ty: Box::new(syn::Type::Path(TypePath {
                        // cSpell:disable-next-line
                        qself: None,
                        path: syn::Path {
                            leading_colon: None,
                            segments: vec![PathSegment {
                                ident: Ident::new("u32", Span::call_site()),
                                arguments: syn::PathArguments::None,
                            }]
                            .into_iter()
                            .collect(),
                        },
                    })),
                }));
                Ok::<_, syn::Error>(out)
            })
    }
    pub fn method_id_contents(&self) -> syn::Result<Vec<Expr>> {
        self.method_ids
            .iter()
            .try_fold(Vec::new(), |mut out: Vec<Expr>, id| {
                if self.overriding_method_ids.contains(id) {
                    return Ok(out);
                }
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
                    out.push(match self.method_parent.as_ref() {
                        Some(parent) => {
                            syn::Expr::Cast(ExprCast {
                                attrs: Vec::new(),
                                expr: Box::new(Expr::Path(syn::ExprPath {
                                    attrs: Vec::new(),
                                    // cSpell:disable-next-line
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: vec![
                                            PathSegment {
                                                ident: parent.clone(),
                                                arguments: syn::PathArguments::None,
                                            },
                                            PathSegment {
                                                ident: Ident::new(
                                                    "__END",
                                                    Span::call_site().resolved_at(parent.span()),
                                                ),
                                                arguments: syn::PathArguments::None,
                                            },
                                        ]
                                        .into_iter()
                                        .collect(),
                                    },
                                })),
                                as_token: Token![as](Span::call_site().resolved_at(parent.span())),
                                ty: Box::new(syn::Type::Path(TypePath {
                                    // cSpell:disable-next-line
                                    qself: None,
                                    path: syn::Path {
                                        leading_colon: None,
                                        segments: vec![PathSegment {
                                            ident: Ident::new("u32", Span::call_site()),
                                            arguments: syn::PathArguments::None,
                                        }]
                                        .into_iter()
                                        .collect(),
                                    },
                                })),
                            })
                        }
                        None => syn::Expr::Lit(syn::ExprLit {
                            attrs: Vec::new(),
                            lit: syn::Lit::Int(LitInt::new("0u32", Span::call_site())),
                        }),
                    });
                }
                Ok::<_, syn::Error>(out)
            })
    }

    pub fn field_id_enum_ident(&self) -> Ident {
        format_ident!("{}_FieldId", self.id)
    }
    pub fn define_fields(&self, attrs: &[TokenStream]) -> TokenStream {
        let field_id_enum_ident = self.field_id_enum_ident();

        let field_ids = self.fields.iter().map(|x| &x.id).collect::<Vec<_>>();
        let field_id_contents = self.field_id_contents();

        quote::quote! {
            #[repr(u32)]
            #[derive(Clone, Copy)]
            #(#[#attrs])*
            pub enum #field_id_enum_ident {
                #(
                    #field_ids = #field_id_contents,
                )*

                #[doc(hidden)]
                __END,
            }
        }
    }

    pub fn method_id_enum_ident(&self) -> Ident {
        format_ident!("{}_MethodId", self.id)
    }
    pub fn define_methods(&self, attrs: &[TokenStream]) -> syn::Result<TokenStream> {
        let method_id_enum_ident = self.method_id_enum_ident();

        let overriding_method_ids = self
            .overriding_method_ids
            .iter()
            .map(|x| &x.id)
            .collect::<Vec<_>>();
        let overriding_method_id_contents = self.overriding_method_id_contents()?;

        let method_ids = self.method_ids.iter().map(|x| &x.id).collect::<Vec<_>>();
        let method_id_contents = self.method_id_contents()?;

        let method_end_content = if method_id_contents.is_empty() {
            match self.method_parent.as_ref() {
                Some(parent) => {
                    quote::quote!(= #parent::__END as u32)
                }
                None => {
                    quote::quote!(= 0u32)
                }
            }
        } else {
            quote::quote!()
        };

        Ok(quote::quote! {
            #[repr(u32)]
            #[derive(Clone, Copy)]
            #(#[#attrs])*
            pub enum #method_id_enum_ident {
                #(
                    #overriding_method_ids = #overriding_method_id_contents,
                )*
                #(
                    #method_ids = #method_id_contents,
                )*

                #[doc(hidden)]
                __END #method_end_content,
            }
        })
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

                #[doc(hidden)]
                __END,
            }
        }
    }
}
