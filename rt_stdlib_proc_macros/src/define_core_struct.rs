use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use shared::define_core_struct::*;

pub fn _impl(ast: DefineCoreStructAst) -> syn::Result<TokenStream> {
    let field_id_enum_ident = format_ident!("{}_FieldId", ast.id);
    let method_id_enum_ident = format_ident!("{}_MethodId", ast.id);
    let static_method_id_enum_ident = format_ident!("{}_StaticMethodId", ast.id);

    let field_ids = ast.fields.iter().map(|x| &x.id).collect::<Vec<_>>();

    let method_ids = &ast.method_ids;
    let static_method_ids = &ast.static_method_ids;

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
    })
}
