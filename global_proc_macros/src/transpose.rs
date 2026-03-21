//! AIGC

use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericParam, Ident, Result, Token, Type, TypePath,
    WhereClause,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

// Parser for `#[transpose(T1, T2)]`
struct TransposeAttr {
    params: Punctuated<Ident, Token![,]>,
}

impl Parse for TransposeAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let params = Punctuated::parse_terminated(&input)?;
        Ok(TransposeAttr { params })
    }
}

pub fn derive_transpose(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;

    let (_, _, ty_where_clauses) = generics.split_for_impl();

    // Collect all type parameter identifiers (T1, T2, …)
    let all_type_params: Vec<&Ident> = generics
        .params
        .iter()
        .filter_map(|p| {
            if let GenericParam::Type(t) = p {
                Some(&t.ident)
            } else {
                None
            }
        })
        .collect();

    // Determine which type parameters should be transposed
    let transpose_set = get_transpose_set(&input.attrs, "transpose", &all_type_params);

    // Build the target type (self) with Option wrapped around transposed parameters
    let target_generics = all_type_params.iter().map(|tp| {
        if transpose_set.contains(tp) {
            quote! { ::core::option::Option<#tp> }
        } else {
            quote! { #tp }
        }
    });
    let target_type = quote! { #name::<#(#target_generics),*> };

    // The result type uses the original parameters (no Option)
    let result_type = quote! { #name::<#(#all_type_params),*> };

    // Generate match arms for each variant
    let match_arms: Vec<_> = match &input.data {
        Data::Enum(data_enum) => {
            data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Unit => {
                        quote! {
                            #target_type::#variant_name => ::core::option::Option::Some(#result_type::#variant_name)
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let field_idents: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| format_ident!("x{}", i))
                            .collect();
                        let field_types = &fields.unnamed;
                        // For each field, check if its type is one of the transposed parameters
                        let is_transposed: Vec<bool> = field_types.iter().map(|f| {
                            if let Type::Path(p) = &f.ty {
                                if p.path.segments.len() == 1 {
                                    let ident = &p.path.segments[0].ident;
                                    transpose_set.contains(ident)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }).collect();

                        // Build the pattern: for transposed fields we require `Some(ident)`,
                        // for concrete or ignored generics we just bind `ident`.
                        let pattern: Vec<proc_macro2::TokenStream> = field_idents.iter().zip(is_transposed.iter()).map(|(ident, is_tp)| {
                            if *is_tp {
                                quote! { ::core::option::Option::Some(#ident) }
                            } else {
                                quote! { #ident }
                            }
                        }).collect();

                        // Constructor arguments: all field identifiers
                        let constructor_args: Vec<_> = field_idents.iter().map(|ident| quote! { #ident }).collect();
                        let constructor = if field_idents.len() == 1 {
                            quote! { #result_type::#variant_name(#(#constructor_args)*) }
                        } else {
                            quote! { #result_type::#variant_name( #(#constructor_args),* ) }
                        };

                        // If any field is transposed, we need to check all transposed fields for `Some`
                        let has_transposed = is_transposed.iter().any(|&b| b);
                        if has_transposed {
                            if field_idents.len() == 1 && is_transposed[0] {
                                // Single transposed field: just map
                                quote! {
                                    Self::#variant_name(x) => x.map(#result_type::#variant_name)
                                }
                            } else {
                                // Multiple fields: match on the pattern, returning None if any transposed field is None
                                quote! {
                                    Self::#variant_name(#(#field_idents),*) => match (#(#pattern),*) {
                                        (#(#pattern),*) => ::core::option::Option::Some(#constructor),
                                        _ => ::core::option::Option::None,
                                    }
                                }
                            }
                        } else {
                            // No transposed fields: just wrap in Some
                            quote! {
                                Self::#variant_name(#(#field_idents),*) => ::core::option::Option::Some(#constructor)
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let field_idents: Vec<_> = fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();
                        let field_types = &fields.named;
                        let is_transposed: Vec<bool> = field_types.iter().map(|f| {
                            if let Type::Path(p) = &f.ty {
                                if p.path.segments.len() == 1 {
                                    let ident = &p.path.segments[0].ident;
                                    transpose_set.contains(ident)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }).collect();

                        let pattern: Vec<proc_macro2::TokenStream> = field_idents.iter().zip(is_transposed.iter()).map(|(ident, is_tp)| {
                            if *is_tp {
                                quote! { ::core::option::Option::Some(#ident) }
                            } else {
                                quote! { #ident }
                            }
                        }).collect();

                        let constructor_fields: Vec<_> = field_idents.iter().map(|ident| quote! { #ident }).collect();
                        let constructor = quote! { #result_type::#variant_name { #(#constructor_fields),* } };

                        let has_transposed = is_transposed.iter().any(|&b| b);
                        if has_transposed {
                            if field_idents.len() == 1 && is_transposed[0] {
                                // Single transposed field: map
                                quote! {
                                    Self::#variant_name { #(#field_idents),* } => #(#field_idents)*.map(|#(#field_idents)*| #constructor)
                                }
                            } else {
                                quote! {
                                    Self::#variant_name { #(#field_idents),* } => match (#(#field_idents),*) {
                                        (#(#pattern,)*) => ::core::option::Option::Some(#constructor),
                                        _ => ::core::option::Option::None,
                                    }
                                }
                            }
                        } else {
                            quote! {
                                Self::#variant_name { #(#field_idents),* } => ::core::option::Option::Some(#constructor)
                            }
                        }
                    }
                }
            }).collect()
        }
        _ => panic!("Transpose can only be derived for enums"),
    };

    let result_impl = generate_transpose_result_impl(&input);

    let expanded = quote! {
        impl #generics #target_type #ty_where_clauses {
            pub fn transpose(self) -> ::core::option::Option<#result_type> {
                match self {
                    #(#match_arms),*
                }
            }
        }
        #result_impl
    };

    Ok(TokenStream::from(expanded))
}

// -----------------------------------------------------------------------------
// Result transpose implementation
// -----------------------------------------------------------------------------

fn generate_transpose_result_impl(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;

    let all_type_params: Vec<&Ident> = generics
        .params
        .iter()
        .filter_map(|p| {
            if let GenericParam::Type(t) = p {
                Some(&t.ident)
            } else {
                None
            }
        })
        .collect();

    let transpose_set = get_transpose_set(&input.attrs, "transpose", &all_type_params);

    let (_, _, ty_where_clauses) = generics.split_for_impl();
    let ty_where_clauses = match ty_where_clauses {
        None => None,
        Some(clauses) => Some(WhereClause {
            where_token: clauses.where_token,
            predicates: clauses
                .predicates
                .iter()
                .map(|predicate| match predicate {
                    syn::WherePredicate::Type(predicate_type) => {
                        let mut predicate_type = predicate_type.clone();
                        if let Type::Path(p) = &mut predicate_type.bounded_ty {
                            if let Some(orig) = p.path.get_ident()
                                && transpose_set.contains(orig)
                            {
                                *p.path.segments.first_mut().unwrap() =
                                    format_ident!("{}_Success", orig).into();
                            }
                        }
                        syn::WherePredicate::Type(predicate_type)
                    }
                    _ => predicate.clone(),
                })
                .collect(),
        }),
    };

    // Build new generic parameters for the impl:
    // - For each transposed parameter T, generate Success_<T> and Error_<T>
    // - For non-transposed, keep as is
    let mut impl_generics = Vec::new();
    let mut target_generics = Vec::new();
    let mut result_generics = Vec::new();
    let mut error_bounds = Vec::new();

    // Store mapping from original generic to its success/error identifiers
    let mut success_ident_map = std::collections::HashMap::new();
    let mut error_ident_map = std::collections::HashMap::new();

    for orig in &all_type_params {
        if transpose_set.contains(orig) {
            let success_ident = format_ident!("{}_Success", orig);
            let error_ident = format_ident!("{}_Error", orig);
            success_ident_map.insert(*orig, success_ident.clone());
            error_ident_map.insert(*orig, error_ident.clone());

            impl_generics.push(quote! { #success_ident });
            impl_generics.push(quote! { #error_ident });
            target_generics.push(quote! { ::core::result::Result<#success_ident, #error_ident> });
            result_generics.push(quote! { #success_ident });
            // We'll add error bounds later after scanning fields
            error_bounds.push(quote! { ::core::convert::From<#error_ident> });
        } else {
            impl_generics.push(quote! { #orig });
            target_generics.push(quote! { #orig });
            result_generics.push(quote! { #orig });
        }
    }

    // Build the impl generics as a list of type parameters
    let impl_generics_tokens = if impl_generics.is_empty() {
        quote! {}
    } else {
        quote! { <#(#impl_generics),*> }
    };

    let target_type = quote! { #name::<#(#target_generics),*> };
    let result_type = quote! { #name::<#(#result_generics),*> };

    // Now we need to generate the match arms. We'll also collect which error types actually appear in fields.
    let mut used_error_types = HashSet::new();

    let match_arms: Vec<_> = match &input.data {
        Data::Enum(data_enum) => {
            data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Unit => {
                        quote! {
                            #target_type::#variant_name => ::core::result::Result::Ok(#result_type::#variant_name)
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let field_idents: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| format_ident!("x{}", i))
                            .collect();
                        let field_types = &fields.unnamed;
                        // For each field, check if its type is one of the transposed generics.
                        let is_transposed: Vec<bool> = field_types.iter().map(|f| {
                            if let Some(orig) = get_generic_ident(&f.ty) {
                                transpose_set.contains(orig)
                            } else {
                                false
                            }
                        }).collect();

                        // Also collect which error types are used for later bounds
                        for (field, &is_tp) in field_types.iter().zip(&is_transposed) {
                            if is_tp {
                                if let Some(orig) = get_generic_ident(&field.ty) {
                                    used_error_types.insert(orig.clone());
                                }
                            }
                        }

                        // Generate early‑return extraction for transposed fields.
                        // For each transposed field, we will generate a match that returns on error.
                        let extraction: Vec<proc_macro2::TokenStream> = field_idents.iter().zip(is_transposed.iter()).map(|(ident, &is_tp)| {
                            if is_tp {
                                quote! {
                                    let #ident = match #ident {
                                        ::core::result::Result::Ok(v) => v,
                                        ::core::result::Result::Err(e) => return ::core::result::Result::Err(e.into()),
                                    };
                                }
                            } else {
                                // For non-transposed, just bind as is (no extraction)
                                quote! { let #ident = #ident; }
                            }
                        }).collect();

                        // Constructor uses the extracted values
                        let constructor_args: Vec<_> = field_idents.iter().map(|ident| quote! { #ident }).collect();
                        let constructor = if field_idents.len() == 1 {
                            quote! { #result_type::#variant_name(#(#constructor_args)*) }
                        } else {
                            quote! { #result_type::#variant_name( #(#constructor_args),* ) }
                        };

                        if is_transposed.iter().any(|&b| b) {
                            quote! {
                                #target_type::#variant_name(#(#field_idents),*) => {
                                    #(#extraction)*
                                    ::core::result::Result::Ok(#constructor)
                                }
                            }
                        } else {
                            quote! {
                                #target_type::#variant_name(#(#field_idents),*) => ::core::result::Result::Ok(#constructor)
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let field_idents: Vec<_> = fields.named.iter().map(|f| f.ident.as_ref().unwrap()).collect();
                        let field_types = &fields.named;
                        let is_transposed: Vec<bool> = field_types.iter().map(|f| {
                            if let Some(orig) = get_generic_ident(&f.ty) {
                                transpose_set.contains(orig)
                            } else {
                                false
                            }
                        }).collect();

                        for (field, &is_tp) in field_types.iter().zip(&is_transposed) {
                            if is_tp {
                                if let Some(orig) = get_generic_ident(&field.ty) {
                                    used_error_types.insert(orig.clone());
                                }
                            }
                        }

                        let extraction: Vec<proc_macro2::TokenStream> = field_idents.iter().zip(is_transposed.iter()).map(|(ident, &is_tp)| {
                            if is_tp {
                                quote! {
                                    let #ident = match #ident {
                                        ::core::result::Result::Ok(v) => v,
                                        ::core::result::Result::Err(e) => return ::core::result::Result::Err(e.into()),
                                    };
                                }
                            } else {
                                quote! { let #ident = #ident; }
                            }
                        }).collect();

                        let constructor_fields: Vec<_> = field_idents.iter().map(|ident| quote! { #ident }).collect();
                        let constructor = quote! { #result_type::#variant_name { #(#constructor_fields),* } };

                        if is_transposed.iter().any(|&b| b) {
                            quote! {
                                #target_type::#variant_name { #(#field_idents),* } => {
                                    #(#extraction)*
                                    ::core::result::Result::Ok(#constructor)
                                }
                            }
                        } else {
                            quote! {
                                #target_type::#variant_name { #(#field_idents),* } => ::core::result::Result::Ok(#constructor)
                            }
                        }
                    }
                }
            }).collect()
        }
        _ => panic!("TransposeResult can only be derived for enums"),
    };

    // Build the where clause with bounds for each used error type
    let where_clause = if used_error_types.is_empty() {
        quote! {}
    } else {
        let bounds: Vec<_> = used_error_types
            .into_iter()
            .map(|orig| {
                let error_ident = error_ident_map.get(&orig).unwrap();
                quote! { UniE: ::core::convert::From<#error_ident> }
            })
            .collect();
        quote! { where #(#bounds),* }
    };

    let expanded = quote! {
        impl #impl_generics_tokens #target_type
        #ty_where_clauses {
            pub fn transpose<UniE>(self) -> ::core::result::Result<#result_type, UniE>
            #where_clause
            {
                match self {
                    #(#match_arms),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}

// -----------------------------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------------------------

/// Parse the `#[transpose(...)]` attribute and return the set of identifiers to transpose.
/// If no attribute is present, all type parameters are transposed.
fn get_transpose_set(
    attrs: &[Attribute],
    name: &str,
    all_params: &[&Ident],
) -> std::collections::HashSet<Ident> {
    for attr in attrs {
        if attr.path().is_ident(name) {
            // Parse the attribute content
            if let Ok(transpose_attr) = attr.parse_args::<TransposeAttr>() {
                let mut set = std::collections::HashSet::new();
                for ident in transpose_attr.params {
                    set.insert(ident.clone());
                }
                // Ensure all named parameters exist in the enum's generics
                for ident in &set {
                    if !all_params.iter().any(|p| **p == *ident) {
                        panic!(
                            "Type parameter `{}` specified in #[transpose] does not exist",
                            ident
                        );
                    }
                }
                return set;
            } else {
                panic!("Invalid #[transpose] attribute. Expected format: #[transpose(T1, T2)]");
            }
        }
    }
    // No attribute -> transpose all
    all_params.iter().map(|&ident| ident.clone()).collect()
}

/// Returns the generic identifier if the type is exactly a generic parameter.
fn get_generic_ident(ty: &Type) -> Option<&Ident> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if path.segments.len() == 1 {
            return Some(&path.segments[0].ident);
        }
    }
    None
}
