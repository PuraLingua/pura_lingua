use proc_macro_utils::crate_name_resolution::PredefinedCrateName;
use proc_macro2::{Span, TokenStream};
use quote::quote;

use shared::define_core_interface::*;

pub fn _impl(ast: DefineCoreInterfaceAst) -> syn::Result<TokenStream> {
    let global_crate = PredefinedCrateName::Global.as_ident(Span::call_site());
    let stdlib_header_crate = PredefinedCrateName::RuntimeStdlib.as_ident(Span::call_site());

    let id = &ast.id;
    let name = &ast.name;
    let generic_count = ast
        .generic_count
        .as_ref()
        .map(crate::make_generic_count)
        .map(|x| quote!(Some(#x)))
        .unwrap_or(quote!(None));
    let method_id_enum_ident = ast.method_id_enum_ident();

    let attr = &ast.attr.inner;

    let required_interfaces = &ast.required_interfaces;

    let method_definition = ast.define_methods(&[
        quote!(derive(serde::Serialize, serde::Deserialize)),
        quote!(derive(#global_crate::AllVariants)),
        quote!(serde(deny_unknown_fields)),
    ])?;

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

    Ok(quote! {
        #method_definition

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
            pub fn get_generic_count(&self) -> Option<#stdlib_header_crate::GenericCount> {
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

        pub fn load() -> #stdlib_header_crate::CoreTypeInfo {
            #stdlib_header_crate::CoreTypeInfo {
                id: #stdlib_header_crate::CoreTypeId::#id,
                kind: #stdlib_header_crate::CoreTypeKind::Interface,
                attr: #global_crate::attr!(class #attr),
                name: #name.to_owned(),
                generic_count: #generic_count,
                parent: None,
                parent_generics: Vec::new(),
                implemented_interfaces: vec![#(#required_interfaces),*],
                methods: #method_id_enum_ident::ALL_VARIANTS
                    .into_iter()
                    .filter(|x| !matches!(x, #method_id_enum_ident::__END))
                    .map(|x| #stdlib_header_crate::MethodInfo {
                        id: x as u32,
                        name: x.get_name().to_owned(),
                        generic_count: x.get_generic_count(),
                        attr: x.get_attr(),
                        args: x.get_parameters(),
                        return_type: x.get_return_type(),
                    })
                    .collect(),
                static_methods: Vec::new(),
                fields: Vec::new(),
            }
        }
    })
}
