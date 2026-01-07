use proc_macro_utils::crate_name_resolution::PredefinedCrateName;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Data, DeriveInput, spanned::Spanned};

pub fn derive_write_to_file_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let binary_traits = PredefinedCrateName::BinaryTraits.as_ident(Span::call_site());
    let binary_types = PredefinedCrateName::BinaryTypes.as_ident(Span::call_site());
    let global_crate = PredefinedCrateName::Global.as_ident(Span::call_site());
    let data = &input.data;
    let name = &input.ident;
    let (impl_g, ty_g, wh) = input.generics.split_for_impl();
    let wh = match wh {
        Some(wh) => {
            let mut wh = wh.clone();
            for generic in input.generics.type_params() {
                wh.predicates.push(syn::WherePredicate::Type({
                    let mut x = syn::PredicateType {
                        lifetimes: None,
                        bounded_ty: syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: syn::Path::from(generic.ident.clone()),
                        }),
                        colon_token: syn::token::Colon::default(),
                        bounds: syn::punctuated::Punctuated::new(),
                    };

                    x.bounds.push(syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: syn::parse2(quote!(#binary_traits::WriteToFile))?,
                    }));

                    x
                }));
            }

            wh
        }
        None => {
            let mut wh = syn::WhereClause {
                where_token: syn::Token![where](Span::call_site()),
                predicates: syn::punctuated::Punctuated::new(),
            };
            for generic in input.generics.type_params() {
                wh.predicates.push(syn::WherePredicate::Type({
                    let mut x = syn::PredicateType {
                        lifetimes: None,
                        bounded_ty: syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: syn::Path::from(generic.ident.clone()),
                        }),
                        colon_token: syn::token::Colon::default(),
                        bounds: syn::punctuated::Punctuated::new(),
                    };

                    x.bounds.push(syn::TypeParamBound::Trait(syn::TraitBound {
                        paren_token: None,
                        modifier: syn::TraitBoundModifier::None,
                        lifetimes: None,
                        path: syn::parse2(quote!(#binary_traits::WriteToFile))?,
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
            Ok(quote! {
                impl #impl_g #binary_traits::WriteToFile for #name #ty_g #wh {
                    fn write_to_file(
                        &self,
                        file: &mut #binary_types::File,
                    ) -> #global_crate::Result<()> {
                        #(
                            #binary_traits::WriteToFile::write_to_file(&self.#idents, file)?;
                        )*
                        Ok(())
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
                return Ok(quote! {
                    impl #impl_g #binary_traits::WriteToFile for #name #ty_g #wh {
                        fn write_to_file(
                            &self,
                            file: &mut #binary_types::File,
                        ) -> #global_crate::Result<()> {
                            let __i = *self as #repr;
                            __i.write_to_file(file)
                        }
                    }
                });
            }
            let mut ts = TokenStream::new();
            for v in variants {
                let mut is_unnamed = false;
                let f_idents = v
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        if let Some(ref id) = f.ident {
                            id.to_token_stream()
                        } else {
                            is_unnamed = true;
                            let mut i = syn::Index::from(i);
                            i.span = f.span();
                            i.to_token_stream()
                        }
                    })
                    .collect::<Vec<_>>();
                let out_idents = if is_unnamed {
                    f_idents
                        .iter()
                        .map(|x| {
                            let s = Ident::new(&format!("_{x}"), x.span());
                            quote!(#s)
                        })
                        .collect::<Vec<_>>()
                } else {
                    f_idents.clone()
                };
                let v_name = &v.ident;
                let matcher = if is_unnamed {
                    quote!(#name::#v_name(
                        #(#out_idents),*
                    ))
                } else {
                    quote!(#name::#v_name {
                        #(#f_idents: #out_idents),*
                    })
                };

                ts.extend(quote! {
                    #matcher => {
                        #(
                            #binary_traits::WriteToFile::write_to_file(#out_idents, file)?;
                        )*
                    }
                });
            }
            Ok(quote! {
                #[allow(non_shorthand_field_patterns)]
                impl #impl_g #binary_traits::WriteToFile for #name #ty_g #wh {
                    fn write_to_file(
                        &self,
                        file: &mut #binary_types::File,
                    ) -> #global_crate::Result<()> {
                        self.to_type().write_to_file(file)?;
                        match self {
                            #ts
                        }
                        Ok(())
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
