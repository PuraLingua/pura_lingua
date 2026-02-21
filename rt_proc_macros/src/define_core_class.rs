use proc_macro_utils::crate_name_resolution::PredefinedCrateName;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};

use shared::define_core_class::*;

pub fn _impl(ast: DefineCoreClassAst) -> syn::Result<TokenStream> {
    let runtime_crate = PredefinedCrateName::Runtime.as_ident(Span::call_site());

    let field_id_enum_ident = format_ident!("{}_FieldId", ast.id);
    let method_id_enum_ident = format_ident!("{}_MethodId", ast.id);
    let static_method_id_enum_ident = format_ident!("{}_StaticMethodId", ast.id);
    let assembly_name = &ast.assembly_name;
    let attr = &ast.attr.inner;
    let id = &ast.id;
    let name = &ast.name;
    let method_generator = &ast.method_generator;

    let field_ids = ast.fields.iter().map(|x| &x.id).collect::<Vec<_>>();
    let field_id_contents = ast.fields.iter().fold(Vec::new(), |mut out, _| {
        if let Some(last) = out.last().cloned() {
            out.push(quote!(1u32 + #last));
        } else {
            out.push(match ast.field_parent.as_ref() {
                Some(parent) => {
                    quote!(#parent::__END as u32)
                }
                None => {
                    quote!(0u32)
                }
            });
        }
        out
    });
    let field_attrs = ast.fields.iter().map(|x| &x.attr.inner).collect::<Vec<_>>();
    let field_names = ast.fields.iter().map(|x| &x.name).collect::<Vec<_>>();
    let field_types = ast.fields.iter().map(|x| &x.ty).collect::<Vec<_>>();

    let method_ids = ast.method_ids.iter().map(|x| &x.1).collect::<Vec<_>>();
    let method_id_contents = ast
        .method_ids
        .iter()
        .try_fold(Vec::new(), |mut out, (o, id)| {
            if o.is_some() {
                let parent = ast.method_parent.as_ref().ok_or(syn::Error::new(
                    Span::call_site(),
                    "Override field require field parents",
                ))?;
                out.push(quote! {
                    (#parent::#id as u32)
                });

                Ok(out)
            } else {
                if let Some(last) = out.last().cloned() {
                    out.push(quote!(1u32 + #last));
                } else {
                    out.push(match ast.method_parent.as_ref() {
                        Some(parent) => {
                            quote!(#parent::__END as u32)
                        }
                        None => {
                            quote!(0u32)
                        }
                    });
                }
                Ok::<_, syn::Error>(out)
            }
        })?;

    let method_end_content = if method_id_contents.is_empty() {
        match ast.method_parent.as_ref() {
            Some(parent) => {
                quote!(= #parent::__END as u32)
            }
            None => {
                quote!(= 0u32)
            }
        }
    } else {
        quote!()
    };

    let static_method_ids = &ast.static_method_ids;

    let generic_bounds = match &ast.generic_bounds {
        Some(e) => e.to_token_stream(),
        None => quote!(None),
    };

    let parent = match &ast.parent {
        Some(p) => p.to_token_stream(),
        None => quote!(None),
    };

    Ok(quote! {
        #[repr(u32)]
        pub enum #field_id_enum_ident {
            #(
                #field_ids = #field_id_contents,
            )*

            #[doc(hidden)]
            __END,
        }

        #[repr(u32)]
        pub enum #method_id_enum_ident {
            #(
                #method_ids = #method_id_contents,
            )*

            #[doc(hidden)]
            __END #method_end_content,
        }

        #[repr(u32)]
        pub enum #static_method_id_enum_ident {
            StaticConstructor = #method_id_enum_ident::__END as u32,
            #(
                #static_method_ids,
            )*

            #[doc(hidden)]
            __END,
        }

        impl const From<#method_id_enum_ident> for #runtime_crate::type_system::method::MethodRef {
            fn from(val: #method_id_enum_ident) -> Self {
                Self::Index(val as _)
            }
        }

        impl const From<#static_method_id_enum_ident> for #runtime_crate::type_system::method::MethodRef {
            fn from(val: #static_method_id_enum_ident) -> Self {
                Self::Index(val as _)
            }
        }

        #[allow(unused)]
        pub fn #id(
            #assembly_name:
            &#runtime_crate::type_system::assembly::Assembly,
        ) ->
        #runtime_crate::type_system::type_handle::NonGenericTypeHandle {
            use ::std::ptr::NonNull;

            use #runtime_crate::type_system::{
                type_handle::NonGenericTypeHandle,
                class::Class,
                method_table::MethodTable,
                field::Field,
            };

            NonGenericTypeHandle::Class(Class::new(
                NonNull::from_ref(#assembly_name),
                #name.to_owned(),
                ::global::attr!(class #attr),
                #parent,
                |class| {
                    MethodTable::new(class, #method_generator).as_non_null_ptr()
                },
                vec![#(
                    Field::new(
                        #field_names.to_owned(),
                        ::global::attr!(field #field_attrs),
                        #field_types,
                    ),
                )*],
                Some(#static_method_id_enum_ident::StaticConstructor as _),
                #generic_bounds,
            ).as_non_null_ptr())
        }
    })
}
