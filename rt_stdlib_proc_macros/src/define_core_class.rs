use std::sync::OnceLock;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

use shared::define_core_class::*;

pub fn _impl(ast: DefineCoreClassAst) -> syn::Result<TokenStream> {
    let field_id_enum_ident = format_ident!("{}_FieldId", ast.id);
    let method_id_enum_ident = format_ident!("{}_MethodId", ast.id);
    let static_method_id_enum_ident = format_ident!("{}_StaticMethodId", ast.id);

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

    let overriding_method_ids = &ast.overriding_method_ids;
    let overriding_method_id_contents = {
        let once_parent = OnceLock::new();
        overriding_method_ids
            .iter()
            .try_fold(Vec::new(), |mut out, id| {
                let parent = once_parent.get_or_try_init(|| {
                    ast.method_parent.clone().ok_or(syn::Error::new(
                        Span::call_site(),
                        "Override methods require method parents",
                    ))
                })?;
                out.push(quote! {
                    (#parent::#id as u32)
                });
                Ok::<_, syn::Error>(out)
            })?
    };

    let method_ids = ast.method_ids.iter().collect::<Vec<_>>();
    let method_id_contents = ast.method_ids.iter().try_fold(Vec::new(), |mut out, _| {
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
                #overriding_method_ids = #overriding_method_id_contents,
            )*
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
    })
}
