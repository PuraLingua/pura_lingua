use proc_macro_utils::crate_name_resolution::PredefinedCrateName;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Colon;
use syn::{
    Data, DeriveInput, Expr, Path, PredicateType, Token, TraitBound, Type, TypePath, WhereClause,
};

pub fn derive_read_from_section_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let binary_core = PredefinedCrateName::BinaryCore.as_ident(Span::call_site());
    let data = &input.data;
    let name = &input.ident;
    let (impl_g, ty_g, wh) = input.generics.split_for_impl();
    let wh = match wh {
        Some(wh) => {
            let mut wh = wh.clone();
            for generic in input.generics.type_params() {
                wh.predicates.push(syn::WherePredicate::Type({
                    let mut x = PredicateType {
                        lifetimes: None,
                        bounded_ty: Type::Path(TypePath {
                            qself: None,
                            path: Path::from(generic.ident.clone()),
                        }),
                        colon_token: Colon::default(),
                        bounds: Punctuated::new(),
                    };

                    x.bounds.push(syn::TypeParamBound::Trait(TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: syn::parse2(quote!(#binary_core::traits::ReadFromSection))?,
                    }));

                    x
                }));
            }

            wh
        }
        None => {
            let mut wh = WhereClause {
                where_token: Token![where](Span::call_site()),
                predicates: Punctuated::new(),
            };
            for generic in input.generics.type_params() {
                wh.predicates.push(syn::WherePredicate::Type({
                    let mut x = PredicateType {
                        lifetimes: None,
                        bounded_ty: Type::Path(TypePath {
                            qself: None,
                            path: Path::from(generic.ident.clone()),
                        }),
                        colon_token: Colon::default(),
                        bounds: Punctuated::new(),
                    };

                    x.bounds.push(syn::TypeParamBound::Trait(TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: syn::parse2(quote!(#binary_core::traits::ReadFromSection))?,
                    }));

                    x
                }));
            }

            wh
        }
    };
    match data {
        Data::Struct(s) => {
            let idents = s
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    if let Some(ref id) = f.ident {
                        id.to_token_stream()
                    } else {
                        let mut i = syn::Index::from(i);
                        i.span = f.span();
                        i.to_token_stream()
                    }
                })
                .collect::<Vec<_>>();
            let per_ident_expr = idents.iter().map(|x| {
                quote_spanned! {
                    x.span() => #x: #binary_core::traits::ReadFromSection::read_from_section(cursor)?,
                }
            });
            Ok(quote! {
                impl #impl_g #binary_core::traits::ReadFromSection for #name #ty_g #wh {
                    fn read_from_section(
                        cursor: &mut std::io::Cursor<&#binary_core::section::Section>,
                    ) -> Result<Self, #binary_core::error::Error> {
                        Ok(Self {
                            #(
                                #per_ident_expr
                            )*
                        })
                    }
                }
            })
        }
        Data::Enum(e) => {
            let variants = &e.variants;
            if let Some(repr) = input.attrs.iter().find_map(|x| {
                if x.path().get_ident()? == "repr" {
                    Some(&x.meta)
                } else {
                    None
                }
            }) && variants
                .iter()
                .all(|x| matches!(&x.fields, syn::Fields::Unit))
            {
                let repr = repr.require_list()?.parse_args::<syn::Ident>()?;
                let idents = variants.iter().map(|x| &x.ident);
                let vars = variants.iter().enumerate().map(|(i, x)| {
                    x.discriminant
                        .clone()
                        .map(|x| x.1)
                        .unwrap_or(Parser::parse_str(Expr::parse, &format!("{i}")).unwrap())
                });
                return Ok(quote! {
                    impl #impl_g #binary_core::traits::ReadFromSection for #name #ty_g #wh {
                        fn read_from_section(
                            cursor: &mut std::io::Cursor<&#binary_core::section::Section>,
                        ) -> Result<Self, #binary_core::error::Error> {
                            let __i = #repr::read_from_section(cursor)?;
                            match __i {
                                #(
                                    #vars => Ok(#name::#idents),
                                )*
                                _ => Err(
                                    #binary_core::error::Error::enum_out_of_bounds::<Self>()
                                        .into(),
                                ),
                            }
                        }
                    }
                });
            }
            let mut ts = TokenStream::new();
            let type_ident = Ident::new(&format!("{name}Type"), Span::call_site());
            for v in variants {
                let f_idents = v
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        if let Some(ref id) = f.ident {
                            id.to_token_stream()
                        } else {
                            let mut i = syn::Index::from(i);
                            i.span = f.span();
                            i.to_token_stream()
                        }
                    })
                    .collect::<Vec<_>>();
                let v_name = &v.ident;
                ts.extend(quote! {
                    #type_ident::#v_name => Ok(#name::#v_name {
                        #(#f_idents: #binary_core::traits::ReadFromSection::read_from_section(cursor)?,)*
                    }),
                });
            }
            Ok(quote! {
                #[allow(non_shorthand_field_patterns)]
                impl #impl_g #binary_core::traits::ReadFromSection for #name #ty_g #wh {
                    fn read_from_section(
                        cursor: &mut std::io::Cursor<&#binary_core::section::Section>,
                    ) -> Result<Self, #binary_core::error::Error> {
                        let x = #type_ident::read_from_section(cursor)?;
                        match x {
                            #ts
                        }
                    }
                }
            })
        }
        Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            "Unions are not supported",
        )),
    }
}

struct ReadFromFileForeignInput {
    t: syn::Type,
    i: syn::LitInt,
}

impl Parse for ReadFromFileForeignInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let t = input.parse()?;
        let i = input.parse()?;
        Ok(Self { t, i })
    }
}

pub fn read_from_section_foreign_with_new_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let parsed = Parser::parse2(ReadFromFileForeignInput::parse, input)?;
    let binary_core = PredefinedCrateName::BinaryCore.as_ident(Span::call_site());
    let t = &parsed.t;
    let mut token_stream = TokenStream::new();
    for _ in 0..parsed.i.base10_parse::<u64>()? {
        token_stream
            .extend(quote! {#binary_core::traits::ReadFromSection::read_from_section(cursor)?,});
    }
    Ok(quote! {
        impl #binary_core::traits::ReadFromSection for #t {
            fn read_from_file(
                cursor: &mut std::io::Cursor<&#binary_core::section::Section>,
            ) -> Result<Self, #binary_core::error::Error> {
                Ok(Self::new(
                    #token_stream
                ))
            }
        }
    })
}
