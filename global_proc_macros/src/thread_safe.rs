use proc_macro2::TokenStream;
use syn::DeriveInput;

pub fn derive_thread_safe_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (i_generics, t_generics, where_clause) = &input.generics.split_for_impl();
    Ok(quote::quote! {
        unsafe impl #i_generics Send for #name #t_generics #where_clause {}
        unsafe impl #i_generics Sync for #name #t_generics #where_clause {}
    })
}
