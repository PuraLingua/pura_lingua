use proc_macro_utils::crate_name_resolution::PredefinedCrateName;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, Ident, Token, parse::Parse, punctuated::Punctuated};

mod keywords {
    use syn::custom_keyword;

    custom_keyword!(method);
    custom_keyword!(parameter);
    custom_keyword!(field);
    custom_keyword!(class);
}

pub enum AttrKind {
    Method,
    Parameter,
    Field,
    Class,
    Struct,
}

impl From<keywords::method> for AttrKind {
    fn from(_: keywords::method) -> Self {
        Self::Method
    }
}

impl From<keywords::parameter> for AttrKind {
    fn from(_: keywords::parameter) -> Self {
        Self::Parameter
    }
}

impl From<keywords::field> for AttrKind {
    fn from(_: keywords::field) -> Self {
        Self::Field
    }
}

impl From<keywords::class> for AttrKind {
    fn from(_: keywords::class) -> Self {
        Self::Class
    }
}

impl From<Token![struct]> for AttrKind {
    fn from(_: Token![struct]) -> Self {
        Self::Struct
    }
}

impl Parse for AttrKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input
            .parse::<keywords::method>()
            .map(Self::from)
            .or_else(|_| input.parse::<keywords::parameter>().map(Self::from))
            .or_else(|_| input.parse::<keywords::field>().map(Self::from))
            .or_else(|_| input.parse::<keywords::class>().map(Self::from))
            .or_else(|_| input.parse::<Token![struct]>().map(Self::from))
            .map_err(|e| {
                syn::Error::new(
                    e.span(),
                    "expected one of `method`, `parameter`, `field`, `class` or `struct`",
                )
            })
    }
}

pub enum CreateAttrAst {
    Method {
        ov: Option<Expr>,
        vis: Ident,
        flags: Vec<Ident>,
        types: Vec<Expr>,
    },
    Parameter {
        flags: Vec<Ident>,
    },
    Field {
        vis: Ident,
        flags: Vec<Ident>,
    },
    Class {
        vis: Ident,
        flags: Vec<Ident>,
    },
    Struct {
        vis: Ident,
        flags: Vec<Ident>,
    },
}

impl Parse for CreateAttrAst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attr_kind: AttrKind = input.parse()?;
        match attr_kind {
            AttrKind::Method => {
                let ov = if input.parse::<Token![override]>().is_ok() {
                    Some(input.parse::<Expr>()?)
                } else {
                    None
                };

                let vis = input.parse()?;

                let flag_tokens;
                syn::braced!(flag_tokens in input);
                let mut flags = Vec::new();
                while let Ok(f) = flag_tokens.parse::<Ident>() {
                    flags.push(f);
                }
                let punctuated = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;

                Ok(Self::Method {
                    ov,
                    vis,
                    flags,
                    types: punctuated.iter().cloned().collect(),
                })
            }
            AttrKind::Parameter => {
                let flag_tokens;
                syn::braced!(flag_tokens in input);
                let mut flags = Vec::new();
                while let Ok(f) = flag_tokens.parse::<Ident>() {
                    flags.push(f);
                }

                Ok(Self::Parameter { flags })
            }
            AttrKind::Field => {
                let vis = input.parse()?;

                let flag_tokens;
                syn::braced!(flag_tokens in input);
                let mut flags = Vec::new();
                while let Ok(f) = flag_tokens.parse::<Ident>() {
                    flags.push(f);
                }

                Ok(Self::Field { vis, flags })
            }
            AttrKind::Class => {
                let vis = input.parse()?;

                let flag_tokens;
                syn::braced!(flag_tokens in input);
                let mut flags = Vec::new();
                while let Ok(f) = flag_tokens.parse::<Ident>() {
                    flags.push(f);
                }

                Ok(Self::Class { vis, flags })
            }
            AttrKind::Struct => {
                let vis = input.parse()?;

                let flag_tokens;
                syn::braced!(flag_tokens in input);
                let mut flags = Vec::new();
                while let Ok(f) = flag_tokens.parse::<Ident>() {
                    flags.push(f);
                }

                Ok(Self::Struct { vis, flags })
            }
        }
    }
}

fn make_flags(_crate: &Ident, name: Ident, flags: Vec<Ident>) -> TokenStream {
    quote! {
        {
            let mut n = 0;
            #(
                n |= #_crate::attrs::#name::#flags as
                    <
                        #_crate::attrs::#name as #_crate::__internal::enumflags2::_internal::RawBitFlags
                    >::Numeric;
            )*
            // SAFETY: The value has been created from numeric values of the underlying
            // enum, so only valid bits are set.
            unsafe {
                #_crate::__internal::enumflags2::BitFlags::<#_crate::attrs::#name>::from_bits_unchecked_c(
                    n,
                    #_crate::__internal::enumflags2::BitFlags::CONST_TOKEN,
                )
            }
        }
    }
}

pub fn create_attr_impl(ast: CreateAttrAst) -> syn::Result<TokenStream> {
    let _crate = PredefinedCrateName::Global.as_ident(Span::call_site());

    match ast {
        CreateAttrAst::Method {
            ov,
            vis,
            flags,
            types,
        } => {
            let ov = ov.unwrap_or_else(|| syn::parse_str("None").unwrap());
            let flags = make_flags(
                &_crate,
                Ident::new("MethodImplementationFlags", Span::call_site()),
                flags,
            );

            Ok(quote! {
                #_crate::attrs::MethodAttr::new(
                    #_crate::attrs::Visibility::#vis,
                    #flags,
                    #ov,
                    vec![#(#types),*],
                )
            })
        }
        CreateAttrAst::Parameter { flags } => {
            let flags = make_flags(
                &_crate,
                Ident::new("ParameterImplementationFlags", Span::call_site()),
                flags,
            );
            Ok(quote! {
                #_crate::attrs::ParameterAttr::new(#flags)
            })
        }
        CreateAttrAst::Field { vis, flags } => {
            let flags = make_flags(
                &_crate,
                Ident::new("FieldImplementationFlags", Span::call_site()),
                flags,
            );
            Ok(quote! {
                #_crate::attrs::FieldAttr::new(
                    #_crate::attrs::Visibility::#vis,
                    #flags,
                )
            })
        }
        CreateAttrAst::Class { vis, flags } => {
            let flags = make_flags(
                &_crate,
                Ident::new("ClassImplementationFlags", Span::call_site()),
                flags,
            );
            Ok(quote! {
                #_crate::attrs::TypeAttr::new(
                    #_crate::attrs::Visibility::#vis,
                    #_crate::attrs::TypeSpecificAttr::Class(#flags),
                )
            })
        }
        CreateAttrAst::Struct { vis, flags } => {
            let flags = make_flags(
                &_crate,
                Ident::new("StructImplementationFlags", Span::call_site()),
                flags,
            );
            Ok(quote! {
                #_crate::attrs::TypeAttr::new(
                    #_crate::attrs::Visibility::#vis,
                    #_crate::attrs::TypeSpecificAttr::Struct(#flags),
                )
            })
        }
    }
}
