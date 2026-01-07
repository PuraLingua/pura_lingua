use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Data, DeriveInput};
use syn::{Ident, Meta};

pub(crate) fn derive_with_type_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let Data::Enum(ref data) = input.data else {
        return Err(syn::Error::new(
            Span::call_site(),
            "WithType can only be derived on enums",
        ));
    };
    let owner_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let name = Ident::new(&format!("{}Type", &input.ident), Span::call_site());
    let vis = &input.vis;
    let variants = data
        .variants
        .iter()
        .map(|x| x.ident.clone())
        .collect::<Vec<_>>();
    let mut to_type_fn_match_arms = TokenStream::new();
    for v in &variants {
        to_type_fn_match_arms.extend(quote!(Self::#v { .. } => #name::#v,));
    }
    let attrs = input.attrs.iter().filter_map(|x| {
        if let Some(n) = x.meta.path().get_ident()
            && n == "with_type"
        {
            Some(x.meta.clone())
        } else {
            None
        }
    });
    let mut repr = TokenStream::new();
    let mut derives = TokenStream::new();
    let mut derives_const = TokenStream::new();
    if let Some(repr_attr) = input
        .attrs
        .iter()
        .find(|x| x.path().get_ident().is_some_and(|x| x.eq("repr")))
        && repr_attr
            .parse_args::<Ident>()
            .is_ok_and(|x| x.to_string().starts_with(['i', 'u']))
    {
        repr_attr.to_tokens(&mut repr);
    }
    for meta in attrs {
        let list = meta.require_list()?;
        let inner_meta = list.parse_args::<Meta>()?;
        let name_value = inner_meta.require_name_value()?;
        let i = name_value.path.require_ident();
        if i.is_err() {
            continue;
        }
        let i = i.unwrap();
        if i == "repr" && repr.is_empty() {
            let v = name_value.value.to_token_stream();
            repr = quote!(#[repr(#v)]);
        } else if i == "derive" {
            let v = name_value.value.to_token_stream();
            derives = quote!(#[derive #v]);
        } else if i == "derive_const" {
            let v = name_value.value.to_token_stream();
            derives_const = quote!(#[derive_const #v]);
        }
    }
    let v = if variants.is_empty() {
        quote!()
    } else {
        let v1 = unsafe { variants.first().unwrap_unchecked() };
        let rest = &variants[1..];
        quote! {
            #v1,
            #(#rest),*
        }
    };
    Ok(quote! {
        #repr
        #derives
        #derives_const
        #vis enum #name {
            #v
        }
        impl #impl_generics #owner_name #ty_generics #where_clause {
            #vis fn to_type(&self) -> #name {
                match self {
                    #to_type_fn_match_arms
                }
            }
        }
    })
}
