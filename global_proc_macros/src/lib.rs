#![feature(proc_macro_totokens)]
#![feature(extend_one)]
#![feature(lazy_get)]
#![feature(decl_macro)]
#![feature(macro_metavar_expr_concat)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(const_try)]
#![allow(static_mut_refs)]

mod all_variants;
mod attr;
mod custom_partial_eq;
mod define_instruction;
mod str_enum;
mod thread_safe;
mod unwrap_enum;
mod util;
mod with_type;

use proc_macro_utils::macro_definitions::{define_derive_macros, define_macros};
use with_type::*;

use crate::{
    all_variants::derive_all_variants_impl,
    attr::{CreateAttrAst, create_attr_impl},
    custom_partial_eq::derive_custom_partial_eq_impl,
    define_instruction::{DefineInstructionAst, define_instruction_impl},
    thread_safe::derive_thread_safe_impl,
};

define_derive_macros! {
    AllVariants[] => derive_all_variants_impl;
    WithType[with_type] => derive_with_type_impl;
    ThreadSafe[] => derive_thread_safe_impl;
    CustomPartialEq[custom_eq, fully_eq] => derive_custom_partial_eq_impl;
    StrEnum[str_val, global_crate] => str_enum::derive_str_enum_impl;
    CharEnum[char_val, global_crate] => str_enum::derive_char_enum_impl;
    UnwrapEnum[unwrap_enum] => unwrap_enum::derive_unwrap_enum_impl;
}

define_macros! {
    define_instruction => define_instruction_impl as DefineInstructionAst;
    attr => create_attr_impl as CreateAttrAst;
}
