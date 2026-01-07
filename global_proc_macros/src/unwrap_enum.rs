use bitfields::bitfield;
use convert_case::{Case, Casing};
use proc_macro_utils::crate_name_resolution::PredefinedCrateName;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, Ident, Token, punctuated::Punctuated, spanned::Spanned,
    token::Paren,
};

#[bitfield(u8)]
#[derive(Clone, Copy)]
struct EnableOptions {
    owned: bool,
    r#ref: bool,
    ref_mut: bool,
    r#try: bool,
    #[bits(4)]
    _pad: u8,
}

fn parse_enable_options(orig: &mut EnableOptions, attrs: &[Attribute]) -> syn::Result<()> {
    for attr in attrs {
        if attr.path().get_ident().is_none_or(|x| x.ne("unwrap_enum")) {
            continue;
        }
        let list = attr.meta.require_list()?;
        for meta in list.tokens.clone() {
            match meta {
                proc_macro2::TokenTree::Ident(ident) => {
                    if ident.eq("ref") {
                        orig.set_ref(true);
                    } else if ident.eq("ref_mut") {
                        orig.set_ref_mut(true);
                    } else if ident.eq("owned") {
                        orig.set_owned(true);
                    } else if ident.eq("try") {
                        orig.set_try(true);
                    } else {
                        return Err(syn::Error::new(ident.span(), "unknown ident"));
                    }
                }
                proc_macro2::TokenTree::Punct(ref p) => {
                    if p.as_char().ne(&',') {
                        return Err(syn::Error::new(
                            meta.span(),
                            format_args!("abnormal meta: {meta}"),
                        ));
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        meta.span(),
                        format_args!("abnormal meta: {meta}"),
                    ));
                }
            }
        }
    }

    Ok(())
}

pub fn derive_unwrap_enum_impl(ast: DeriveInput) -> syn::Result<TokenStream> {
    let global_ident = PredefinedCrateName::Global.as_ident(Span::call_site());
    let leading_colon2 = if global_ident.eq("crate") {
        None
    } else {
        Some(quote!(::))
    };
    let type_name = &ast.ident;
    let (impl_generics, generics, where_clauses) = ast.generics.split_for_impl();
    let Data::Enum(data) = &ast.data else {
        return Err(syn::Error::new(
            Span::call_site(),
            "Only enums are supported",
        ));
    };
    let mut tokens = Vec::new();
    let mut global_enable_options = EnableOptions::new();
    parse_enable_options(&mut global_enable_options, &ast.attrs)?;

    for variant in &data.variants {
        let name = &variant.ident;
        let mut enable_options = global_enable_options;
        parse_enable_options(&mut enable_options, &variant.attrs)?;

        let (ref_impl, ref_mut_impl, owned_impl) = match &variant.fields {
            Fields::Named(fields_named) => {
                let identifiers: Vec<_> = fields_named
                    .named
                    .iter()
                    .map(|x| x.ident.as_ref().unwrap())
                    .collect();

                let ref_impl = if enable_options.r#ref() {
                    let mut ret_ty = proc_macro2::TokenStream::new();
                    Paren::default().surround(&mut ret_ty, |x| {
                        let mut punctuated: Punctuated<TokenStream, Token![,]> =
                            syn::punctuated::Punctuated::new();

                        for field in &fields_named.named {
                            let f_ty = &field.ty;
                            punctuated.push(quote!(&#f_ty));
                        }
                        x.extend_one(punctuated.to_token_stream());
                    });
                    let success = if enable_options.r#try() {
                        quote!(Ok((#(#identifiers),*)))
                    } else {
                        quote!((#(#identifiers),*))
                    };
                    let fallback = if enable_options.r#try() {
                        quote!(Err(#leading_colon2 #global_ident::errors::UnwrapError.into()))
                    } else {
                        quote!(panic!("call unwrap at incorrect value"))
                    };
                    if enable_options.r#try() {
                        ret_ty = quote!(#leading_colon2 #global_ident::Result<#ret_ty, #global_ident::errors::UnwrapError>);
                    }

                    let fn_name = format_ident!(
                        "unwrap_{ident}_ref",
                        ident = name.to_string().to_case(Case::Snake)
                    );
                    Some(quote! {
                        pub fn #fn_name(&self) -> #ret_ty {
                            match self {
                                Self::#name {#(#identifiers, )*} => #success,
                                _ => #fallback,
                            }
                        }
                    })
                } else {
                    None
                };
                let ref_mut_impl = if enable_options.ref_mut() {
                    let mut ret_ty = proc_macro2::TokenStream::new();
                    Paren::default().surround(&mut ret_ty, |x| {
                        let mut punctuated: Punctuated<TokenStream, Token![,]> =
                            syn::punctuated::Punctuated::new();

                        for field in &fields_named.named {
                            let f_ty = &field.ty;
                            punctuated.push(quote!(&mut #f_ty));
                        }
                        x.extend_one(punctuated.to_token_stream());
                    });
                    let success = if enable_options.r#try() {
                        quote!(Ok((#(#identifiers),*)))
                    } else {
                        quote!((#(#identifiers),*))
                    };
                    let fallback = if enable_options.r#try() {
                        quote!(Err(#leading_colon2 #global_ident::errors::UnwrapError.into()))
                    } else {
                        quote!(panic!("call unwrap at incorrect value"))
                    };
                    if enable_options.r#try() {
                        ret_ty = quote!(#leading_colon2 #global_ident::Result<#ret_ty, #global_ident::errors::UnwrapError>);
                    }
                    let fn_name = format_ident!(
                        "unwrap_{ident}_mut",
                        ident = name.to_string().to_case(Case::Snake)
                    );
                    Some(quote! {
                        pub fn #fn_name(&mut self) -> #ret_ty {
                            match self {
                                Self::#name {#(#identifiers, )*} => #success,
                                _ => #fallback,
                            }
                        }
                    })
                } else {
                    None
                };
                let owned_impl = if enable_options.owned() {
                    let mut ret_ty = proc_macro2::TokenStream::new();
                    Paren::default().surround(&mut ret_ty, |x| {
                        let mut punctuated: Punctuated<TokenStream, Token![,]> =
                            syn::punctuated::Punctuated::new();

                        for field in &fields_named.named {
                            let f_ty = &field.ty;
                            punctuated.push(quote!(#f_ty));
                        }
                        x.extend_one(punctuated.to_token_stream());
                    });
                    let success = if enable_options.r#try() {
                        quote!(Ok((#(#identifiers),*)))
                    } else {
                        quote!((#(#identifiers),*))
                    };
                    let fallback = if enable_options.r#try() {
                        quote!(Err(#leading_colon2 #global_ident::errors::UnwrapError.into()))
                    } else {
                        quote!(panic!("call unwrap at incorrect value"))
                    };
                    if enable_options.r#try() {
                        ret_ty = quote!(#leading_colon2 #global_ident::Result<#ret_ty, #global_ident::errors::UnwrapError>);
                    }
                    let fn_name = format_ident!(
                        "unwrap_{ident}",
                        ident = name.to_string().to_case(Case::Snake)
                    );
                    Some(quote! {
                        pub fn #fn_name(self) -> #ret_ty {
                            match self {
                                Self::#name {#(#identifiers, )*} => #success,
                                _ => #fallback,
                            }
                        }
                    })
                } else {
                    None
                };
                (ref_impl, ref_mut_impl, owned_impl)
            }
            Fields::Unnamed(fields_unnamed) => {
                let identifiers: Vec<_> = fields_unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|x| Ident::new(&format!("a{}", x.0), x.1.span()))
                    .collect();
                let ref_impl = if enable_options.r#ref() {
                    let mut ret_ty = proc_macro2::TokenStream::new();
                    Paren::default().surround(&mut ret_ty, |x| {
                        let mut punctuated: Punctuated<TokenStream, Token![,]> =
                            syn::punctuated::Punctuated::new();

                        for field in &fields_unnamed.unnamed {
                            let f_ty = &field.ty;
                            punctuated.push(quote!(&#f_ty));
                        }
                        x.extend_one(punctuated.to_token_stream());
                    });
                    let success = if enable_options.r#try() {
                        quote!(Ok((#(#identifiers),*)))
                    } else {
                        quote!((#(#identifiers),*))
                    };
                    let fallback = if enable_options.r#try() {
                        quote!(Err(#leading_colon2 #global_ident::errors::UnwrapError.into()))
                    } else {
                        quote!(panic!("call unwrap at incorrect value"))
                    };
                    if enable_options.r#try() {
                        ret_ty = quote!(#leading_colon2 #global_ident::Result<#ret_ty, #global_ident::errors::UnwrapError>);
                    }

                    let fn_name = format_ident!(
                        "unwrap_{ident}_ref",
                        ident = name.to_string().to_case(Case::Snake)
                    );
                    Some(quote! {
                        pub fn #fn_name(&self) -> #ret_ty {
                            match self {
                                Self::#name(#(#identifiers, )*) => #success,
                                _ => #fallback,
                            }
                        }
                    })
                } else {
                    None
                };
                let ref_mut_impl = if enable_options.ref_mut() {
                    let mut ret_ty = proc_macro2::TokenStream::new();
                    Paren::default().surround(&mut ret_ty, |x| {
                        let mut punctuated: Punctuated<TokenStream, Token![,]> =
                            syn::punctuated::Punctuated::new();

                        for field in &fields_unnamed.unnamed {
                            let f_ty = &field.ty;
                            punctuated.push(quote!(&mut #f_ty));
                        }
                        x.extend_one(punctuated.to_token_stream());
                    });
                    let success = if enable_options.r#try() {
                        quote!(Ok((#(#identifiers),*)))
                    } else {
                        quote!((#(#identifiers),*))
                    };
                    let fallback = if enable_options.r#try() {
                        quote!(Err(#leading_colon2 #global_ident::errors::UnwrapError.into()))
                    } else {
                        quote!(panic!("call unwrap at incorrect value"))
                    };
                    if enable_options.r#try() {
                        ret_ty = quote!(#leading_colon2 #global_ident::Result<#ret_ty, #global_ident::errors::UnwrapError>);
                    }
                    let fn_name = format_ident!(
                        "unwrap_{ident}_mut",
                        ident = name.to_string().to_case(Case::Snake)
                    );
                    Some(quote! {
                        pub fn #fn_name(&mut self) -> #ret_ty {
                            match self {
                                Self::#name(#(#identifiers, )*) => #success,
                                _ => #fallback,
                            }
                        }
                    })
                } else {
                    None
                };
                let owned_impl = if enable_options.owned() {
                    let mut ret_ty = proc_macro2::TokenStream::new();
                    Paren::default().surround(&mut ret_ty, |x| {
                        let mut punctuated: Punctuated<TokenStream, Token![,]> =
                            syn::punctuated::Punctuated::new();

                        for field in &fields_unnamed.unnamed {
                            let f_ty = &field.ty;
                            punctuated.push(quote!(#f_ty));
                        }
                        x.extend_one(punctuated.to_token_stream());
                    });
                    let success = if enable_options.r#try() {
                        quote!(Ok((#(#identifiers),*)))
                    } else {
                        quote!((#(#identifiers),*))
                    };
                    let fallback = if enable_options.r#try() {
                        quote!(Err(#leading_colon2 #global_ident::errors::UnwrapError.into()))
                    } else {
                        quote!(panic!("call unwrap at incorrect value"))
                    };
                    if enable_options.r#try() {
                        ret_ty = quote!(#leading_colon2 #global_ident::Result<#ret_ty, #global_ident::errors::UnwrapError>);
                    }
                    let fn_name = format_ident!(
                        "unwrap_{ident}",
                        ident = name.to_string().to_case(Case::Snake)
                    );
                    Some(quote! {
                        pub fn #fn_name(self) -> #ret_ty {
                            match self {
                                Self::#name(#(#identifiers, )*) => #success,
                                _ => #fallback,
                            }
                        }
                    })
                } else {
                    None
                };
                (ref_impl, ref_mut_impl, owned_impl)
            }
            Fields::Unit => {
                let mut ret_ty = quote!(());
                let success = if enable_options.r#try() {
                    quote!(Ok(()))
                } else {
                    quote!(())
                };
                let fallback = if enable_options.r#try() {
                    quote!(Err(#leading_colon2 #global_ident::errors::UnwrapError.into()))
                } else {
                    quote!(panic!("call unwrap at incorrect value"))
                };
                if enable_options.r#try() {
                    ret_ty = quote!(#leading_colon2 #global_ident::Result<#ret_ty, #global_ident::errors::UnwrapError>);
                }
                let ref_impl = if enable_options.r#ref() {
                    let fn_name = format_ident!(
                        "unwrap_{ident}_ref",
                        ident = name.to_string().to_case(Case::Snake)
                    );
                    Some(quote! {
                        pub fn #fn_name(&self) -> #ret_ty {
                            match self {
                                Self::#name => #success,
                                _ => #fallback,
                            }
                        }
                    })
                } else {
                    None
                };
                let ref_mut_impl = if enable_options.ref_mut() {
                    let fn_name = format_ident!(
                        "unwrap_{ident}_mut",
                        ident = name.to_string().to_case(Case::Snake)
                    );
                    Some(quote! {
                        pub fn #fn_name(&mut self) -> #ret_ty {
                            match self {
                                Self::#name => #success,
                                _ => #fallback,
                            }
                        }
                    })
                } else {
                    None
                };
                let owned_impl = if enable_options.owned() {
                    let fn_name = format_ident!(
                        "unwrap_{ident}",
                        ident = name.to_string().to_case(Case::Snake)
                    );
                    Some(quote! {
                        pub fn #fn_name(self) -> #ret_ty {
                            match self {
                                Self::#name => #success,
                                _ => #fallback,
                            }
                        }
                    })
                } else {
                    None
                };
                (ref_impl, ref_mut_impl, owned_impl)
            }
        };
        tokens.push(ref_impl);
        tokens.push(ref_mut_impl);
        tokens.push(owned_impl);
    }
    Ok(quote! {
        #[allow(unreachable_code)] // omit warnings for `!` and other unreachable types
        #[automatically_derived]
        impl #impl_generics #type_name #generics #where_clauses {
            #(#tokens)*
        }
    })
}
