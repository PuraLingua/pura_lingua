use proc_macro2::TokenStream;
use quote::quote;

use shared::define_core_struct::*;

pub fn _impl(ast: DefineCoreStructAst) -> syn::Result<TokenStream> {
    let field_definition = ast.define_fields(&[]);

    let method_definition = ast.define_methods(&[]);

    let static_method_definition = ast.define_static_methods(&[]);

    Ok(quote! {
        #field_definition

        #method_definition

        #static_method_definition
    })
}
