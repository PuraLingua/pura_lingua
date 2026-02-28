use proc_macro_utils::crate_name_resolution::PredefinedCrateName;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};

use shared::define_core_struct::*;

pub fn _impl(ast: DefineCoreStructAst) -> syn::Result<TokenStream> {
    let global_crate = PredefinedCrateName::Global.as_ident(Span::call_site());
    let stdlib_header_crate = PredefinedCrateName::RuntimeStdlib.as_ident(Span::call_site());
    let runtime_crate = PredefinedCrateName::Runtime.as_ident(Span::call_site());

    let field_id_enum_ident = ast.field_id_enum_ident();
    let method_id_enum_ident = ast.method_id_enum_ident();
    let static_method_id_enum_ident = ast.static_method_id_enum_ident();

    let assembly_name = &ast.assembly_name;
    let attr = &ast.attr.inner;
    let id = &ast.id;
    let name = &ast.name;
    let method_generator = &ast.method_generator;

    let field_definition = ast.define_fields(&[]);

    let field_ids = ast.fields.iter().map(|x| &x.id).collect::<Vec<_>>();
    let field_attrs = ast.fields.iter().map(|x| &x.attr.inner).collect::<Vec<_>>();
    let field_names = ast.fields.iter().map(|x| &x.name).collect::<Vec<_>>();
    let field_types = ast.fields.iter().map(|x| &x.ty).collect::<Vec<_>>();

    let method_definition = ast.define_methods(&[]);

    let method_ids = ast.method_ids.iter().map(|x| &x.id).collect::<Vec<_>>();
    let method_names = ast.method_ids.iter().map(|x| &x.name).collect::<Vec<_>>();
    let method_attrs = ast
        .method_ids
        .iter()
        .map(|x| &x.attr.inner)
        .collect::<Vec<_>>();
    let method_parameters = ast
        .method_ids
        .iter()
        .map(|x| {
            x.parameters
                .iter()
                .map(crate::parameter2token_stream)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let method_return_types = ast
        .method_ids
        .iter()
        .map(|x| &x.return_type)
        .collect::<Vec<_>>();

    let static_method_definition = ast.define_static_methods(&[]);

    let static_method_ids = ast
        .static_method_ids
        .iter()
        .map(|x| &x.id)
        .collect::<Vec<_>>();
    let static_method_names = ast
        .static_method_ids
        .iter()
        .map(|x| &x.name)
        .collect::<Vec<_>>();
    let static_method_attrs = ast
        .static_method_ids
        .iter()
        .map(|x| &x.attr.inner)
        .collect::<Vec<_>>();
    let static_method_parameters = ast
        .static_method_ids
        .iter()
        .map(|x| {
            x.parameters
                .iter()
                .map(crate::parameter2token_stream)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let static_method_return_types = ast
        .static_method_ids
        .iter()
        .map(|x| &x.return_type)
        .collect::<Vec<_>>();

    let generic_bounds = match &ast.generic_bounds {
        Some(e) => e.to_token_stream(),
        None => quote!(None),
    };

    Ok(quote! {
        #field_definition

        #method_definition

        #static_method_definition

        impl #field_id_enum_ident {
            pub fn get_attr(&self) -> #global_crate::attrs::FieldAttr {
                match self {
                    #(
                        Self::#field_ids => #global_crate::attr!(
                            field #field_attrs
                        ),
                    )*
                    Self::__END => unreachable!(),
                }
            }
        }

        impl #method_id_enum_ident {
            pub fn get_attr(&self) -> #global_crate::attrs::MethodAttr<#runtime_crate::type_system::type_handle::MaybeUnloadedTypeHandle> {
                match self {
                    #(
                        Self::#method_ids => {
                            let x: #global_crate::attrs::MethodAttr<
                                #stdlib_header_crate::CoreTypeRef,
                            > = #global_crate::attr!(
                                method #method_attrs
                            );
                            x.map_types(#runtime_crate::type_system::type_handle::MaybeUnloadedTypeHandle::from)
                        }
                    )*
                    Self::__END => unreachable!(),
                }
            }
            pub fn get_parameters(&self) -> Vec<#runtime_crate::type_system::method::Parameter> {
                match self {
                    #(
                        Self::#method_ids => vec![
                            #(
                                #method_parameters,
                            )*
                        ],
                    )*
                    Self::__END => unreachable!(),
                }
            }
            pub fn get_return_type(&self) -> #runtime_crate::type_system::type_handle::MaybeUnloadedTypeHandle {
                match self {
                    #(
                        Self::#method_ids =>
                            <#runtime_crate::type_system::type_handle::MaybeUnloadedTypeHandle
                            as From<#stdlib_header_crate::CoreTypeRef>>::from(#method_return_types),
                    )*
                    Self::__END => unreachable!(),
                }
            }
            pub const fn get_name(&self) -> &'static str {
                match self {
                    #(
                        Self::#method_ids => #method_names,
                    )*
                    Self::__END => unreachable!(),
                }
            }
        }

        impl #static_method_id_enum_ident {
            pub fn get_attr(&self) -> #global_crate::attrs::MethodAttr<#runtime_crate::type_system::type_handle::MaybeUnloadedTypeHandle> {
                match self {
                    Self::StaticConstructor => #global_crate::attr!(
                        method Public {Static}
                    ),
                    #(
                        Self::#static_method_ids => {
                            let x: #global_crate::attrs::MethodAttr<
                                #stdlib_header_crate::CoreTypeRef,
                            > = #global_crate::attr!(
                                method #static_method_attrs
                            );
                            x.map_types(#runtime_crate::type_system::type_handle::MaybeUnloadedTypeHandle::from)
                        }
                    )*
                }
            }
            pub fn get_parameters(&self) -> Vec<#runtime_crate::type_system::method::Parameter> {
                match self {
                    Self::StaticConstructor => vec![],
                    #(
                        Self::#static_method_ids => vec![
                            #(
                                #static_method_parameters,
                            )*
                        ],
                    )*
                }
            }
            pub fn get_return_type(&self) -> #runtime_crate::type_system::type_handle::MaybeUnloadedTypeHandle {
                match self {
                    Self::StaticConstructor =>
                    #runtime_crate::type_system::type_handle::MaybeUnloadedTypeHandle::from(
                        #stdlib_header_crate::CoreTypeRef::Core(
                            #stdlib_header_crate::CoreTypeId::System_Void,
                        ),
                    ),
                    #(
                        Self::#static_method_ids =>
                            <#runtime_crate::type_system::type_handle::MaybeUnloadedTypeHandle
                            as From<#stdlib_header_crate::CoreTypeRef>>::from(#static_method_return_types),
                    )*
                }
            }
            pub const fn get_name(&self) -> &'static str {
                match self {
                    Self::StaticConstructor => ".sctor",
                    #(
                        Self::#static_method_ids => #static_method_names,
                    )*
                }
            }
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
                type_handle::{NonGenericTypeHandle, MaybeUnloadedTypeHandle},
                r#struct::Struct,
                method_table::MethodTable,
                field::Field,
            };

            type TFieldId = #field_id_enum_ident;
            type TMethodId = #method_id_enum_ident;
            type TStaticMethodId = #static_method_id_enum_ident;

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
                        <MaybeUnloadedTypeHandle as From<#stdlib_header_crate::CoreTypeRef>>::from(#field_types),
                    ),
                )*],
                Some(#static_method_id_enum_ident::StaticConstructor as _),
                #generic_bounds,
            ).as_non_null_ptr())
        }
    })
}
