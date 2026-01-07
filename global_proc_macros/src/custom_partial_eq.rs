use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{Data, DeriveInput, Expr, Fields, Ident, spanned::Spanned};

pub fn derive_custom_partial_eq_impl(ast: DeriveInput) -> syn::Result<TokenStream> {
    let type_name = &ast.ident;
    let (impl_generics, generics, where_clauses) = ast.generics.split_for_impl();
    let is_fully_eq = ast.attrs.iter().any(|x| {
        x.path()
            .get_ident()
            .map(|x| x.eq("fully_eq"))
            .unwrap_or(false)
    });
    match &ast.data {
        Data::Struct(_data_struct) => todo!(),
        Data::Enum(data_enum) => {
            let mut tokens = Vec::new();
            for variant in &data_enum.variants {
                let name = &variant.ident;
                let fields = match &variant.fields {
                    Fields::Named(named) => {
                        let identifiers: Vec<_> = named
                            .named
                            .iter()
                            .map(|x| x.ident.clone().unwrap())
                            .collect();
                        let _identifiers: Vec<_> = named
                            .named
                            .iter()
                            .map(|x| format_ident!("{}_", x.ident.as_ref().unwrap()))
                            .collect();
                        let custom_eqs = &variant
                            .attrs
                            .iter()
                            .find_map(|x| {
                                if !x
                                    .path()
                                    .get_ident()
                                    .map(|a| a.eq("custom_eq"))
                                    .unwrap_or(false)
                                {
                                    return None;
                                }
                                x.parse_args::<Expr>().ok().map(|x| x.to_token_stream())
                            })
                            .unwrap_or(quote!(#(#identifiers.eq(#_identifiers) &&)* true));
                        quote_spanned! {
                            variant.span() =>
                                (Self::#name {
                                    #(#identifiers)*
                                }, Self::#name {
                                    #(#identifiers: #_identifiers)*
                                }) => #custom_eqs,
                        }
                    }
                    Fields::Unnamed(unnamed) => {
                        let identifiers: Vec<_> = unnamed
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|x| Ident::new(&format!("a{}", x.0), x.1.span()))
                            .collect();
                        let _identifiers: Vec<_> = identifiers
                            .iter()
                            .map(|x| format_ident!("{}_", x))
                            .collect();
                        let custom_eqs = &variant
                            .attrs
                            .iter()
                            .find_map(|x| {
                                if !x
                                    .path()
                                    .get_ident()
                                    .map(|a| a.eq("custom_eq"))
                                    .unwrap_or(false)
                                {
                                    return None;
                                }
                                x.parse_args::<Expr>().ok().map(|x| x.to_token_stream())
                            })
                            .unwrap_or(quote!(#(#identifiers.eq(#_identifiers) &&)* true));
                        quote_spanned! {
                            variant.span() =>
                                (Self::#name(
                                    #(#identifiers)*
                                ), Self::#name(
                                    #(#_identifiers)*
                                )) => #custom_eqs,
                        }
                    }
                    Fields::Unit => quote_spanned! {
                        variant.span() =>
                            (Self::#name, Self::#name) => true,
                    },
                };
                tokens.push(fields);
            }
            let eq_ts = if is_fully_eq {
                quote! {
                    impl #impl_generics Eq for #type_name #generics #where_clauses {}
                }
            } else {
                quote!()
            };
            Ok(quote! {
                impl #impl_generics PartialEq for #type_name #generics #where_clauses {
                    fn eq(&self, other: &Self) -> bool {
                        match (self, other) {
                            #(#tokens)*
                            #[allow(unreachable_pattern)]
                            _ => false,
                        }
                    }
                }
                #eq_ts
            })
        }
        Data::Union(_) => Err(syn::Error::new(
            Span::call_site(),
            "Cannot derive for unions",
        )),
    }
}
