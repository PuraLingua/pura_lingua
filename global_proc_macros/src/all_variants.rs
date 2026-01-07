use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, spanned::Spanned};

pub fn derive_all_variants_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let syn::Data::Enum(data) = &input.data else {
        return Err(syn::Error::new(
            input.span(),
            "AllVariants could only be derived for enums",
        ));
    };
    let name = &input.ident;

    let variants = data.variants.iter().map(|x| &x.ident).collect::<Vec<_>>();
    let v_len = variants.len();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub const ALL_VARIANTS: [Self; #v_len] = [
                #(Self::#variants),*
            ];
        }
    })
}
