#![feature(iterator_try_collect)]
#![feature(once_cell_try)]

use proc_macro_utils::macro_definitions::define_macros;

mod define_core_class;
mod define_core_struct;

mod serializing;

define_macros! {
    define_core_class => define_core_class::_impl as shared::define_core_class::DefineCoreClassAst;
    define_core_struct => define_core_struct::_impl as shared::define_core_struct::DefineCoreStructAst;
}
