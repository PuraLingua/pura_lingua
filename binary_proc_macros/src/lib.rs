#![feature(macro_metavar_expr_concat)]

use crate::read_from_section::{
    derive_read_from_section_impl, read_from_section_foreign_with_new_impl,
};
use crate::write_to_section::derive_write_to_section_impl;
use proc_macro_utils::macro_definitions::define_derive_macros;

mod read_from_section;
mod write_to_section;

define_derive_macros! {
    ReadFromSection[read_from_file_bounds] => derive_read_from_section_impl;
    WriteToSection[] => derive_write_to_section_impl;
}

#[proc_macro]
pub fn read_from_section_foreign_with_new(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    read_from_section_foreign_with_new_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
