#![feature(iterator_try_collect)]
#![feature(once_cell_try)]

use proc_macro_utils::{
    crate_name_resolution::PredefinedCrateName, macro_definitions::define_macros,
};
use proc_macro2::{Span, TokenStream};
use shared::common::{GenericCount, Parameter};

mod define_core_class;
mod define_core_struct;

mod serializing;

define_macros! {
    define_core_class => define_core_class::_impl as shared::define_core_class::DefineCoreClassAst;
    define_core_struct => define_core_struct::_impl as shared::define_core_struct::DefineCoreStructAst;
}

fn parameter2token_stream(p: &Parameter) -> TokenStream {
    let global_crate = PredefinedCrateName::Global.as_ident(Span::call_site());
    let attr = &p.attr.inner;
    let ty = &p.ty;

    quote::quote!((#global_crate::attr!(parameter #attr), #ty))
}

fn make_generic_count(GenericCount { count, is_infinite }: &GenericCount) -> TokenStream {
    let is_infinite = is_infinite.is_some();
    let stdlib_header_serde_crate =
        PredefinedCrateName::RuntimeStdlibSerde.as_ident(Span::call_site());

    quote::quote! {
        #stdlib_header_serde_crate::GenericCount {
            count: #count,
            is_infinite: #is_infinite,
        }
    }
}
