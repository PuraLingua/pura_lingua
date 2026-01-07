use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Token, Type, braced, parse::Parse};

pub struct InstructionFields {
    fields: IndexMap<Ident, Type>,
}

impl Parse for InstructionFields {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        let fields = content.parse_terminated(
            |p| {
                let name: Ident = p.parse()?;
                let _: Token![:] = p.parse()?;
                let ty: Type = p.parse()?;
                Ok((name, ty))
            },
            Token![,],
        )?;
        Ok(Self {
            fields: fields.into_iter().collect(),
        })
    }
}

pub struct DefineInstructionAst {
    name: Ident,
    t_type_ref: Ident,
    t_method_ref: Ident,
    t_field_ref: Ident,
    fields: IndexMap<Ident, InstructionFields>,
}

impl Parse for DefineInstructionAst {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![<]>()?;
        let t_type_ref = input.parse()?;
        input.parse::<Token![,]>()?;
        let t_method_ref = input.parse()?;
        input.parse::<Token![,]>()?;
        let t_field_ref = input.parse()?;
        #[expect(unused_must_use)]
        input.parse::<Token![,]>();
        input.parse::<Token![>]>()?;
        let fields;
        braced!(fields in input);
        let fields = fields.parse_terminated(
            |p| {
                let name: Ident = p.parse()?;
                let fields: InstructionFields = p.parse()?;
                Ok((name, fields))
            },
            Token![,],
        )?;

        Ok(Self {
            name,
            t_type_ref,
            t_method_ref,
            t_field_ref,
            fields: fields.into_iter().collect(),
        })
    }
}

pub fn define_instruction_impl(
    DefineInstructionAst {
        name,
        t_type_ref,
        t_method_ref,
        t_field_ref,
        fields,
    }: DefineInstructionAst,
) -> syn::Result<TokenStream> {
    let (field_names, field_inners) = fields.iter().unzip::<_, _, Vec<_>, Vec<_>>();
    let (field_inner_names, field_inner_types) = field_inners
        .iter()
        .map(|x| x.fields.iter().unzip::<_, _, Vec<_>, Vec<_>>())
        .unzip::<_, _, Vec<_>, Vec<_>>();
    let type_name = Ident::new(&format!("{name}Type"), name.span());
    let union_name = Ident::new(&format!("{name}Union"), name.span());
    let (union_structs, union_idents) = {
        fields
            .iter()
            .map(|(variant_name, fields)| {
                let struct_name = Ident::new(&format!("{name}_{variant_name}"), name.span());
                let (inner_field_names, inner_field_types) =
                    fields.fields.iter().unzip::<_, _, Vec<_>, Vec<_>>();

                (
                    quote! {
                        #[repr(C)]
                        #[derive(Debug, Clone)]
                        pub struct #struct_name <#t_type_ref, #t_method_ref, #t_field_ref> {
                            #(
                                #inner_field_names: #inner_field_types,
                            )*
                            __p: ::std::marker::PhantomData<(#t_type_ref, #t_method_ref, #t_field_ref)>,
                        }
                    },
                    struct_name,
                )
            })
            .unzip::<_, _, Vec<_>, Vec<_>>()
    };
    let union_field_types = union_idents.iter().map(
        |t| quote! {::std::mem::ManuallyDrop::<#t <#t_type_ref, #t_method_ref, #t_field_ref> >},
    );
    Ok(quote! {
        #[repr(C)]
        #[derive(Debug, Clone)]
        pub enum #name <#t_type_ref, #t_method_ref, #t_field_ref> {
            #(
                #field_names {
                    #(
                        #field_inner_names: #field_inner_types,
                    )*
                },
            )*
        }

        #(#union_structs)*

        #[repr(C)]
        #[allow(nonstandard_style)]
        pub union #union_name <#t_type_ref, #t_method_ref, #t_field_ref> {
            #(
                #field_names: #union_field_types,
            )*
        }

        #[repr(u64)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
        pub enum #type_name {
            #(
                #field_names,
            )*
        }

        impl <#t_type_ref, #t_method_ref, #t_field_ref> #name <#t_type_ref, #t_method_ref, #t_field_ref> {
            pub fn to_type(&self) -> #type_name {
                match self {
                    #(
                        Self::#field_names { .. } => #type_name::#field_names,
                    )*
                }
            }

            pub fn from_raw_parts(ty: #type_name, r#union: #union_name <#t_type_ref, #t_method_ref, #t_field_ref>) -> Self {
                match ty {
                    #(
                        #type_name::#field_names => unsafe {
                            let var = ::std::mem::ManuallyDrop::into_inner(r#union.#field_names);
                            Self::#field_names {
                                #(
                                    #field_inner_names: var.#field_inner_names,
                                )*
                            }
                        },
                    )*
                }
            }
        }

        impl <#t_type_ref: ::std::clone::Clone, #t_method_ref: ::std::clone::Clone, #t_field_ref: ::std::clone::Clone> #name <#t_type_ref, #t_method_ref, #t_field_ref> {
            pub fn to_union(&self) -> #union_name <#t_type_ref, #t_method_ref, #t_field_ref> {
                match self {
                    #(
                        Self::#field_names {#( #field_inner_names ),*} => #union_name {
                            #field_names: ::std::mem::ManuallyDrop::new(#union_idents {
                                #( #field_inner_names: #field_inner_names.clone(), )*

                                __p: ::std::marker::PhantomData,
                            }),
                        },
                    )*
                }
            }
        }
    })
}
