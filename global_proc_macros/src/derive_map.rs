//! AIGC

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, GenericParam, Type, WhereClause};

pub fn to_snake_case_with_t_removal(s: &str) -> String {
    // Remove leading 'T' if present
    let without_t = if s.starts_with('T') { &s[1..] } else { s };

    if without_t.is_empty() {
        return s.to_owned();
    }

    let mut result = String::with_capacity(without_t.len());
    let mut chars = without_t.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            // Insert underscore before the uppercase letter unless it's the first character
            if !result.is_empty() {
                // Look at the previous character (last in result) and the next character
                let prev_is_lowercase = result.chars().last().map_or(false, |p| p.is_lowercase());
                let next_is_lowercase = chars.peek().map_or(false, |n| n.is_lowercase());

                if prev_is_lowercase || next_is_lowercase {
                    result.push('_');
                }
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }

    result
}

pub fn map_derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = input.ident;
    let generics = input.generics;

    // 1. Collect the original type parameters (e.g., T1, T2)
    let mut type_params = Vec::new();
    for param in generics.params.iter() {
        if let GenericParam::Type(tp) = param {
            type_params.push(tp.ident.clone());
        }
    }

    // 2. Prepare the impl block's generic parts (original generics + where clause)
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // 3. Generate fresh names for the output types and function types
    let output_types: Vec<_> = type_params
        .iter()
        .map(|t| syn::Ident::new(&format!("__{}", t), t.span()))
        .collect();

    let function_types: Vec<_> = type_params
        .iter()
        .map(|t| syn::Ident::new(&format!("__F_{}", t), t.span()))
        .collect();

    // 4. Build the method's generic parameters (<U1, U2, F1, F2, ...>)
    let mut method_generics = Vec::new();
    for out_tp in &output_types {
        method_generics.push(syn::GenericParam::Type(syn::TypeParam::from(
            out_tp.clone(),
        )));
    }
    for f_tp in &function_types {
        method_generics.push(syn::GenericParam::Type(syn::TypeParam::from(f_tp.clone())));
    }

    // 5. Build the method's where clause: where __F_T1: FnMut(T1) -> __T1, ...
    let mut method_where_clause = WhereClause {
        where_token: Default::default(),
        predicates: if let Some(wheres) = where_clause {
            wheres
                .predicates
                .iter()
                .map(|predicate| match predicate {
                    syn::WherePredicate::Type(predicate_type) => {
                        let mut predicate = predicate_type.clone();
                        if let Type::Path(bounded) = &mut predicate.bounded_ty {
                            if let Some(id) = bounded.path.segments.first_mut()
                                && type_params.contains(&id.ident)
                            {
                                *id = format_ident!("__{}", &id.ident).into();
                            }
                        }
                        syn::WherePredicate::Type(predicate)
                    }
                    _ => predicate.clone(),
                })
                .collect()
        } else {
            Default::default()
        },
    };
    for ((t, out_t), f_t) in type_params
        .iter()
        .zip(output_types.iter())
        .zip(function_types.iter())
    {
        method_where_clause
            .predicates
            .push(syn::parse_quote! { #f_t: ::core::ops::FnMut(#t) -> #out_t });
    }

    // 6. Build the function parameters: mut f_t1: __F_T1, mut f_t2: __F_T2, ...
    let f_params: Vec<_> = type_params
        .iter()
        .zip(function_types.iter())
        .map(|(t, f_t)| {
            let param_name = syn::Ident::new(&format!("f_{}", t), t.span());
            quote! { mut #param_name: #f_t }
        })
        .collect();

    // 7. Generate match arms for each variant
    let match_arms = match input.data {
        Data::Enum(data_enum) => {
            let mut arms = Vec::new();
            for variant in data_enum.variants {
                let variant_ident = variant.ident;
                match variant.fields {
                    Fields::Unit => {
                        arms.push(quote! {
                            #name::#variant_ident => #name::#variant_ident
                        });
                    }
                    Fields::Unnamed(fields_unnamed) => {
                        // Tuple variant
                        let field_count = fields_unnamed.unnamed.len();
                        let field_idents: Vec<_> = (0..field_count)
                            .map(|i| {
                                syn::Ident::new(&format!("x{}", i), proc_macro2::Span::call_site())
                            })
                            .collect();

                        // Determine which fields need mapping
                        let field_transforms: Vec<_> = fields_unnamed
                            .unnamed
                            .iter()
                            .zip(&field_idents)
                            .map(|(field, ident)| {
                                if is_generic_type(&field.ty, &type_params) {
                                    // Find which type parameter this field corresponds to
                                    let pos =
                                        find_generic_position(&field.ty, &type_params).unwrap();
                                    let f_name = syn::Ident::new(
                                        &format!("f_{}", type_params[pos]),
                                        type_params[pos].span(),
                                    );
                                    quote! { #f_name(#ident) }
                                } else {
                                    quote! { #ident }
                                }
                            })
                            .collect();

                        let pattern = quote! { ( #(#field_idents),* ) };
                        let expr = quote! { ( #(#field_transforms),* ) };
                        arms.push(quote! {
                            #name::#variant_ident #pattern => #name::#variant_ident #expr
                        });
                    }
                    Fields::Named(fields_named) => {
                        // Struct variant
                        let field_names: Vec<_> = fields_named
                            .named
                            .iter()
                            .map(|f| f.ident.clone().unwrap())
                            .collect();

                        let field_transforms: Vec<_> = fields_named
                            .named
                            .iter()
                            .zip(&field_names)
                            .map(|(field, name)| {
                                if is_generic_type(&field.ty, &type_params) {
                                    let pos =
                                        find_generic_position(&field.ty, &type_params).unwrap();
                                    let f_name = syn::Ident::new(
                                        &format!("f_{}", type_params[pos]),
                                        type_params[pos].span(),
                                    );
                                    quote! { #name: #f_name(#name) }
                                } else {
                                    quote! { #name: #name }
                                }
                            })
                            .collect();

                        let pattern = quote! { { #(#field_names),* } };
                        let expr = quote! { { #(#field_transforms),* } };
                        arms.push(quote! {
                            #name::#variant_ident #pattern => #name::#variant_ident #expr
                        });
                    }
                }
            }
            arms
        }
        _ => panic!("Map can only be derived for enums"),
    };

    // ---------- Generate splitted map methods ----------
    let mut splitted_methods = Vec::new();
    for (idx, param_ident) in type_params.iter().enumerate() {
        // Method name: map_t1, map_t2, ...
        let method_name = syn::Ident::new(
            &format!(
                "map_{}",
                to_snake_case_with_t_removal(&param_ident.to_string())
            ),
            param_ident.span(),
        );
        let new_type_ident = syn::Ident::new(&format!("__{}", param_ident), param_ident.span());
        let f_type_ident = syn::Ident::new(&format!("__F_{}", param_ident), param_ident.span());

        // Build the method's generic parameters: <__T, __F>
        let method_generics = quote! { <#new_type_ident, #f_type_ident> };
        // Build the where clause: where __F: FnMut(T) -> __T
        let mut method_where_clause: syn::WhereClause = syn::parse_quote! {
            where #f_type_ident: ::core::ops::FnMut(#param_ident) -> #new_type_ident
        };
        if let Some(wheres) = where_clause {
            method_where_clause
                .predicates
                .extend(wheres.predicates.iter().map(|predicate| match predicate {
                    syn::WherePredicate::Type(predicate_type) => {
                        let mut predicate = predicate_type.clone();
                        if let Type::Path(bounded) = &mut predicate.bounded_ty {
                            if let Some(id) = bounded.path.segments.first_mut()
                                && id.ident == *param_ident
                            {
                                *id = format_ident!("__{}", &id.ident).into();
                            }
                        }
                        syn::WherePredicate::Type(predicate)
                    }
                    _ => predicate.clone(),
                }));
        }
        method_where_clause
            .predicates
            .iter_mut()
            .for_each(|x| match x {
                syn::WherePredicate::Type(predicate) => {
                    if let Type::Path(p) = &mut predicate.bounded_ty
                        && let Some(id) = p.path.segments.first_mut()
                        && *param_ident == id.ident
                    {
                        id.ident = new_type_ident.clone();
                    }
                }
                _ => (),
            });

        // Build the return type: A<...> with the new type at the mapped position
        let mut output_types = Vec::new();
        for (j, p) in type_params.iter().enumerate() {
            if j == idx {
                output_types.push(quote! { #new_type_ident });
            } else {
                output_types.push(quote! { #p });
            }
        }
        let return_type = quote! { #name<#(#output_types),*> };

        // Build the arguments to self.map: identity closures for unchanged parameters
        let mut map_args = Vec::new();
        for (j, _) in type_params.iter().enumerate() {
            if j == idx {
                map_args.push(quote! { f });
            } else {
                map_args.push(quote! { noop });
            }
        }

        let method = quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                pub fn #method_name #method_generics(self, f: #f_type_ident) -> #return_type
                #method_where_clause
                {
                    #[inline(always)]
                    const fn noop<T>(val: T) -> T { val }
                    self.map(#(#map_args),*)
                }
            }
        };
        splitted_methods.push(method);
    }

    // 8. Generate the final impl block
    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #[allow(nonstandard_style)]
            pub fn map<#(#method_generics),*>(
                self,
                #(#f_params),*
            ) -> #name<#(#output_types),*>
            #method_where_clause
            {
                match self {
                    #(#match_arms),*
                }
            }
        }
        #(#splitted_methods)*
    };

    Ok(expanded.into())
}

// Helper: check if a type is exactly one of the generic parameters (no nesting)
fn is_generic_type(ty: &Type, generics: &[syn::Ident]) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = &segment.ident;
            return generics.iter().any(|g| g == ident);
        }
    }
    false
}

// Helper: find the index of the generic parameter that exactly matches the type
fn find_generic_position(ty: &Type, generics: &[syn::Ident]) -> Option<usize> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident = &segment.ident;
            return generics.iter().position(|g| g == ident);
        }
    }
    None
}
