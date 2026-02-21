#![feature(iterator_try_collect)]

use proc_macro_utils::macro_definitions::define_macros;

mod define_core_class;
mod define_core_struct;

define_macros! {
    define_core_class => define_core_class::_impl as shared::define_core_class::DefineCoreClassAst;
    define_core_struct => define_core_struct::_impl as shared::define_core_struct::DefineCoreStructAst;
}

#[proc_macro]
pub fn when_impl(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    ts
}

#[proc_macro]
pub fn when_not_impl(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}
