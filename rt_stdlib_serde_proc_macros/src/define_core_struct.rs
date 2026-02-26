use proc_macro_utils::crate_name_resolution::PredefinedCrateName;
use proc_macro2::{Span, TokenStream};
use quote::quote;

use shared::define_core_struct::*;

pub fn _impl(ast: DefineCoreStructAst) -> syn::Result<TokenStream> {
    let global_crate = PredefinedCrateName::Global.as_ident(Span::call_site());
    let stdlib_header_crate = PredefinedCrateName::RuntimeStdlib.as_ident(Span::call_site());
    let stdlib_header_serde_crate =
        PredefinedCrateName::RuntimeStdlibSerde.as_ident(Span::call_site());

    let id = &ast.id;
    let name = &ast.name;
    let generic_count = ast
        .generic_count
        .as_ref()
        .map(crate::make_generic_count)
        .map(|x| quote!(Some(#x)))
        .unwrap_or(quote!(None));
    let field_id_enum_ident = ast.field_id_enum_ident();
    let method_id_enum_ident = ast.method_id_enum_ident();
    let static_method_id_enum_ident = ast.static_method_id_enum_ident();

    let attr = &ast.attr.inner;

    let field_definition = ast.define_fields(&[
        quote!(derive(serde::Serialize, serde::Deserialize)),
        quote!(derive(#global_crate::AllVariants)),
        quote!(serde(deny_unknown_fields)),
    ]);

    let field_ids = ast.fields.iter().map(|x| &x.id).collect::<Vec<_>>();
    let field_names = ast.fields.iter().map(|x| &x.name).collect::<Vec<_>>();
    let field_types = ast.fields.iter().map(|x| &x.ty).collect::<Vec<_>>();
    let field_attrs = ast.fields.iter().map(|x| &x.attr.inner).collect::<Vec<_>>();

    let method_definition = ast.define_methods(&[
        quote!(derive(serde::Serialize, serde::Deserialize)),
        quote!(derive(#global_crate::AllVariants)),
        quote!(serde(deny_unknown_fields)),
    ]);

    let method_ids = ast.method_ids.iter().map(|x| &x.id).collect::<Vec<_>>();
    let method_generic_counts = ast
        .method_ids
        .iter()
        .map(|x| {
            x.generic_count
                .as_ref()
                .map(crate::make_generic_count)
                .map(|x| quote!(Some(#x)))
                .unwrap_or(quote!(None))
        })
        .collect::<Vec<_>>();
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

    let static_method_definition = ast.define_static_methods(&[
        quote!(derive(serde::Serialize, serde::Deserialize)),
        quote!(derive(#global_crate::AllVariants)),
        quote!(serde(deny_unknown_fields)),
    ]);

    let static_method_ids = ast
        .static_method_ids
        .iter()
        .map(|x| &x.id)
        .collect::<Vec<_>>();
    let static_method_generic_counts = ast
        .static_method_ids
        .iter()
        .map(|x| {
            x.generic_count
                .as_ref()
                .map(crate::make_generic_count)
                .map(|x| quote!(Some(#x)))
                .unwrap_or(quote!(None))
        })
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
            pub fn get_ty(&self) -> #stdlib_header_crate::CoreTypeRef {
                match self {
                    #(
                        Self::#field_ids => #field_types,
                    )*
                    Self::__END => unreachable!(),
                }
            }
            pub const fn get_name(&self) -> &'static str {
                match self {
                    #(
                        Self::#field_ids => #field_names,
                    )*
                    Self::__END => unreachable!(),
                }
            }
        }

        impl #method_id_enum_ident {
            pub fn get_attr(&self) -> #global_crate::attrs::MethodAttr<#stdlib_header_crate::CoreTypeRef> {
                match self {
                    #(
                        Self::#method_ids => #global_crate::attr!(
                            method #method_attrs
                        ),
                    )*
                    Self::__END => unreachable!(),
                }
            }
            pub fn get_generic_count(&self) -> Option<#stdlib_header_serde_crate::GenericCount> {
                match self {
                    #(
                        Self::#method_ids => #method_generic_counts,
                    )*
                    Self::__END => unreachable!(),
                }
            }
            pub fn get_parameters(&self) -> Vec<(
                #global_crate::attrs::ParameterAttr,
                #stdlib_header_crate::CoreTypeRef,
            )> {
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
            pub fn get_return_type(&self) -> #stdlib_header_crate::CoreTypeRef {
                match self {
                    #(
                        Self::#method_ids => #method_return_types,
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
            pub fn get_attr(&self) -> #global_crate::attrs::MethodAttr<#stdlib_header_crate::CoreTypeRef> {
                match self {
                    Self::StaticConstructor => #global_crate::attr!(
                        method Public {Static}
                    ),
                    #(
                        Self::#static_method_ids => #global_crate::attr!(
                            method #static_method_attrs
                        ),
                    )*
                }
            }
            pub fn get_generic_count(&self) -> Option<#stdlib_header_serde_crate::GenericCount> {
                match self {
                    Self::StaticConstructor => None,
                    #(
                        Self::#static_method_ids => #static_method_generic_counts,
                    )*
                }
            }
            pub fn get_parameters(&self) -> Vec<(
                #global_crate::attrs::ParameterAttr,
                #stdlib_header_crate::CoreTypeRef,
            )> {
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
            pub fn get_return_type(&self) -> #stdlib_header_crate::CoreTypeRef {
                match self {
                    Self::StaticConstructor => #stdlib_header_crate::CoreTypeRef::Core(#stdlib_header_crate::CoreTypeId::System_Void),
                    #(
                        Self::#static_method_ids => #static_method_return_types,
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

        pub fn #id() -> CoreTypeInfo {
            CoreTypeInfo {
                id: CoreTypeId::#id,
                kind: #stdlib_header_serde_crate::CoreTypeKind::Struct,
                attr: #global_crate::attr!(struct #attr),
                name: #name.to_owned(),
                generic_count: #generic_count,
                parent: None,
                methods: #method_id_enum_ident::ALL_VARIANTS
                    .into_iter()
                    .filter(|x| !matches!(x, #method_id_enum_ident::__END))
                    .map(|x| MethodInfo {
                        id: x as u32,
                        name: x.get_name().to_owned(),
                        generic_count: x.get_generic_count(),
                        attr: x.get_attr().into(),
                        args: x
                            .get_parameters()
                            .into_iter()
                            .map(MethodArg::from)
                            .collect(),
                        return_type: x.get_return_type(),
                    })
                    .collect(),
                static_methods: #static_method_id_enum_ident::ALL_VARIANTS
                    .into_iter()
                    .map(|x| MethodInfo {
                        id: x as u32,
                        name: x.get_name().to_owned(),
                        generic_count: x.get_generic_count(),
                        attr: x.get_attr().into(),
                        args: x
                            .get_parameters()
                            .into_iter()
                            .map(MethodArg::from)
                            .collect(),
                        return_type: x.get_return_type(),
                    })
                    .collect(),
                fields: #field_id_enum_ident::ALL_VARIANTS
                    .into_iter()
                    .filter(|x| !matches!(x, #field_id_enum_ident::__END))
                    .map(|x| FieldInfo {
                        id: x as u32,
                        name: x.get_name().to_owned(),
                        attr: x.get_attr(),
                        ty: x.get_ty(),
                    })
                    .collect(),
            }
        }
    })
}
