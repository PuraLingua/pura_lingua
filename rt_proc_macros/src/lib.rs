use proc_macro_utils::{
    crate_name_resolution::PredefinedCrateName, macro_definitions::define_macros,
};
use proc_macro2::{Span, TokenStream};
use shared::common::Parameter;

mod define_core_class;
mod define_core_struct;

define_macros! {
    define_core_class => define_core_class::_impl as shared::define_core_class::DefineCoreClassAst;
    define_core_struct => define_core_struct::_impl as shared::define_core_struct::DefineCoreStructAst;
}

fn parameter2token_stream(p: &Parameter) -> TokenStream {
    let runtime_crate = PredefinedCrateName::Runtime.as_ident(Span::call_site());
    let global_crate = PredefinedCrateName::Global.as_ident(Span::call_site());
    let attr = &p.attr.inner;
    let ty = &p.ty;

    quote::quote!(
        #runtime_crate::type_system::method::Parameter {
            ty: #runtime_crate::type_system::type_handle::MaybeUnloadedTypeHandle::from(#ty),
            attr: #global_crate::attr!(parameter #attr),
        }
    )
}
