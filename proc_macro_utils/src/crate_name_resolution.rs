use std::{collections::HashMap, str::FromStr};

use proc_macro2::{Ident, Span};

pub fn get_crate_name_of(name: &str, span: Span) -> Ident {
    let Ok(crate_name) = proc_macro_crate::crate_name(name) else {
        return Ident::new(name, span);
    };
    match crate_name {
        proc_macro_crate::FoundCrate::Itself => Ident::new("crate", Span::call_site()),
        proc_macro_crate::FoundCrate::Name(name) => Ident::new(&name, span),
    }
}

pub fn parse_attribute(attr: &syn::Attribute) -> Option<(PredefinedCrateName, syn::Path)> {
    let ident = attr.path().get_ident()?;
    let mut ident_s = ident.to_string();
    ident_s.make_ascii_lowercase();
    let path = attr.parse_args::<syn::Path>().ok()?;
    let e = PredefinedCrateName::from_str(&ident_s).ok()?;
    Some((e, path))
}

pub fn parse_attributes(attrs: &[syn::Attribute]) -> HashMap<PredefinedCrateName, syn::Path> {
    attrs.iter().filter_map(parse_attribute).collect()
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, derive_more::FromStr)]
#[from_str(rename_all = "snake_case")]
pub enum PredefinedCrateName {
    Global,
    GlobalErrors,

    Binary,
    BinaryCore,
    BinaryProcMacros,

    Runtime,
    RuntimeStdlib,
    RuntimeStdlibSerde,
}

impl PredefinedCrateName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "pura_lingua_global",
            Self::GlobalErrors => "pura_lingua_global_errors",

            Self::Binary => "pura_lingua_binary",
            Self::BinaryCore => "pura_lingua_binary_core",
            Self::BinaryProcMacros => "pura_lingua_binary_proc_macros",

            Self::Runtime => "pura_lingua_runtime",
            Self::RuntimeStdlib => "pura_lingua_runtime_stdlib",
            Self::RuntimeStdlibSerde => "pura_lingua_runtime_stdlib_serde",
        }
    }
    pub fn as_ident(&self, span: Span) -> Ident {
        get_crate_name_of(self.as_str(), span)
    }
    pub fn as_path(&self, span: Span) -> syn::Path {
        syn::Path::from(self.as_ident(span))
    }
}

pub fn get_predefined_crate_name(c: PredefinedCrateName, span: Span) -> Ident {
    get_crate_name_of(c.as_str(), span)
}
