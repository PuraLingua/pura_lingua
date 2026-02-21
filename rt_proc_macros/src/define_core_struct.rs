use proc_macro_utils::crate_name_resolution::PredefinedCrateName;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};

use shared::define_core_struct::*;

pub fn _impl(ast: DefineCoreStructAst) -> syn::Result<TokenStream> {
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
    let field_attrs = ast.fields.iter().map(|x| &x.attr.inner).collect::<Vec<_>>();
    let field_names = ast.fields.iter().map(|x| &x.name).collect::<Vec<_>>();
    let field_types = ast.fields.iter().map(|x| &x.ty).collect::<Vec<_>>();

    let method_ids = &ast.method_ids;
    let static_method_ids = &ast.static_method_ids;

    let generic_bounds = match &ast.generic_bounds {
        Some(e) => e.to_token_stream(),
        None => quote!(None),
    };

    Ok(quote! {
        #[repr(u32)]
        pub enum #field_id_enum_ident {
            #(
                #field_ids,
            )*

            #[doc(hidden)]
            __END,
        }

        #[repr(u32)]
        pub enum #method_id_enum_ident {
            #(
                #method_ids,
            )*

            #[doc(hidden)]
            __END,
        }

        #[repr(u32)]
        pub enum #static_method_id_enum_ident {
            StaticConstructor = #method_id_enum_ident::__END as u32,
            #(
                #static_method_ids,
            )*
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
                r#struct::Struct,
                method_table::MethodTable,
                field::Field,
            };

            NonGenericTypeHandle::Struct(Struct::new(
                NonNull::from_ref(#assembly_name),
                #name.to_owned(),
                ::global::attr!(struct #attr),
                |s| {
                    MethodTable::new(s, #method_generator).as_non_null_ptr()
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
