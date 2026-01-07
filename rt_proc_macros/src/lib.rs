#![feature(iterator_try_collect)]

use proc_macro_utils::macro_definitions::define_macros;

mod common;
mod define_core_class;
mod define_core_struct;

define_macros! {
    define_core_class => define_core_class::_impl as define_core_class::DefineCoreClassAst;
    define_core_struct => define_core_struct::_impl as define_core_struct::DefineCoreClassAst;
}
