use proc_macro_crate::FoundCrate;
use proc_macro2::{Ident, Span};

pub fn get_crate_name_of(name: &str, span: Span) -> Ident {
    let Ok(crate_name) = proc_macro_crate::crate_name(name) else {
        return Ident::new(name, span);
    };
    match crate_name {
        FoundCrate::Itself => Ident::new("crate", Span::call_site()),
        FoundCrate::Name(name) => Ident::new(&name, span),
    }
}

#[derive(Clone, Copy)]
pub enum PredefinedCrateName {
    Global,

    Binary,
    BinaryProcMacros,
    BinaryTraits,
    BinaryTypes,

    Runtime,
}

impl PredefinedCrateName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "pura_lingua_global",
            Self::Binary => "pura_lingua_binary",
            Self::BinaryProcMacros => "pura_lingua_binary_proc_macros",
            Self::BinaryTraits => "pura_lingua_binary_traits",
            Self::BinaryTypes => "pura_lingua_binary_types",

            Self::Runtime => "pura_lingua_runtime",
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
