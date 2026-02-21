use proc_macro_utils::crate_name_resolution::PredefinedCrateName;
use proc_macro2::{Literal, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Expr, Lit, Path};

pub fn derive_str_enum_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let Data::Enum(data) = &input.data else {
        return Err(Error::new(Span::call_site(), "only enums are supported"));
    };

    let crate_data = proc_macro_utils::crate_name_resolution::parse_attributes(&input.attrs);

    let mut global_crate = crate_data
        .get(&PredefinedCrateName::Global)
        .cloned()
        .unwrap_or(PredefinedCrateName::Global.as_path(Span::call_site()));

    let variants = &data.variants;
    let mut ts = TokenStream::new();
    let mut from_str_ts = TokenStream::new();
    let mut variants_ts = TokenStream::new();
    let mut variants_ordered_ts = TokenStream::new();
    let mut variants_ordered_rev_ts = TokenStream::new();

    for metadata in input
        .attrs
        .iter()
        .filter(|x| x.path().get_ident().is_some_and(|x| x == "global_crate"))
        .filter_map(|x| x.meta.require_name_value().ok())
    {
        match &metadata.value {
            Expr::Lit(l) => match &l.lit {
                Lit::Str(s) => {
                    global_crate = syn::parse_str(&s.value())?;
                }
                _ => return Err(Error::new(metadata.span(), "Unsupported expr")),
            },
            Expr::Path(p) => {
                global_crate = p.path.clone();
            }
            _ => return Err(Error::new(metadata.span(), "Unsupported expr")),
        };
    }
    let variant_len = variants.len();
    let mut val_str_dict = Vec::new();

    for v in variants {
        let lit = v
            .attrs
            .iter()
            .filter(|x| x.path().get_ident().is_some_and(|x| x == "str_val"))
            .map(|x| {
                x.meta
                    .require_list()
                    .and_then(|x| x.parse_args::<Literal>())
            })
            .next_back()
            .ok_or(Error::new(v.span(), "expected `str_val`"))??;

        let v_name = &v.ident;
        val_str_dict.push((v_name.clone(), lit.clone()));

        ts.extend(quote!(Self::#v_name => #lit,));
        from_str_ts.extend(quote!(x if x == Self::#v_name.as_str() => Ok(Self::#v_name),));
        variants_ts.extend(quote!(Self:: #v_name,));
    }

    val_str_dict.sort_by_key(|(_, a)| a.to_string().len());

    for (v_name, _) in &val_str_dict {
        variants_ordered_ts.extend(quote!(Self:: #v_name,));
    }

    for (v_name, _) in val_str_dict.iter().rev() {
        variants_ordered_rev_ts.extend(quote!(Self:: #v_name,));
    }

    let name = &input.ident;
    let (i_generic, t_generic, w_clauses) = input.generics.split_for_impl();

    Ok(quote! {
        impl #i_generic #name #t_generic #w_clauses {
            pub const fn as_str(&self) -> &'static str {
                match self {
                    #ts
                }
            }

            pub const VARIANTS: [Self; #variant_len] = [#variants_ts];

            #[doc = "Ordered by length, short to long"]
            pub const VARIANTS_ORDERED: [Self; #variant_len] = [#variants_ordered_ts];
            #[doc = "Ordered by length, long to short"]
            pub const VARIANTS_ORDERED_REV: [Self; #variant_len] = [#variants_ordered_rev_ts];
        }

        impl #i_generic const TryFrom<&str> for #name #t_generic #w_clauses {
            type Error = #global_crate::errors::ConstFromStrError;

            fn try_from(x: &str) -> Result<Self, <Self as TryFrom::<&str>>::Error> {
                match x {
                    #from_str_ts
                    _ => Err(<Self as TryFrom::<&str>>::Error::new()),
                }
            }
        }
    })
}

pub fn derive_char_enum_impl(input: DeriveInput) -> syn::Result<TokenStream> {
    let Data::Enum(data) = &input.data else {
        return Err(Error::new(Span::call_site(), "only enums are supported"));
    };

    let mut global_crate = Path::from(PredefinedCrateName::Global.as_ident(Span::call_site()));

    let variants = &data.variants;
    let mut ts = TokenStream::new();
    let mut from_char_ts = TokenStream::new();

    for metadata in input
        .attrs
        .iter()
        .filter(|x| x.path().get_ident().is_some_and(|x| x == "global_crate"))
        .filter_map(|x| x.meta.require_name_value().ok())
    {
        match &metadata.value {
            Expr::Lit(l) => match &l.lit {
                Lit::Str(s) => {
                    global_crate = syn::parse_str(&s.value())?;
                }
                _ => return Err(Error::new(metadata.span(), "Unsupported expr")),
            },
            Expr::Path(p) => {
                global_crate = p.path.clone();
            }
            _ => return Err(Error::new(metadata.span(), "Unsupported expr")),
        };
    }

    for v in variants {
        let lit = v
            .attrs
            .iter()
            .filter(|x| x.path().get_ident().is_some_and(|x| x == "char_val"))
            .map(|x| {
                x.meta
                    .require_list()
                    .and_then(|x| x.parse_args::<Literal>())
            })
            .next_back()
            .ok_or(Error::new(v.span(), "expected `char_val`"))??;

        let v_name = &v.ident;

        ts.extend(quote!(Self::#v_name => #lit,));
        from_char_ts.extend(quote!(x if x == Self::#v_name.as_char() => Ok(Self::#v_name),));
    }

    let name = &input.ident;
    let (i_generic, t_generic, w_clauses) = input.generics.split_for_impl();

    Ok(quote! {
        impl #i_generic #name #t_generic #w_clauses {
            pub const fn as_char(&self) -> char {
                match self {
                    #ts
                }
            }
        }

        impl #i_generic const TryFrom<char> for #name #t_generic #w_clauses {
            type Error = #global_crate::errors::ConstFromStrError;

            fn try_from(x: char) -> Result<Self, <Self as TryFrom<char>>::Error> {
                match x {
                    #from_char_ts
                    _ => Err(<Self as TryFrom<char>>::Error::new()),
                }
            }
        }
    })
}
