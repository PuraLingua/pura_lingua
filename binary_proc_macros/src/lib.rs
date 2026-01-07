use crate::read_from_file::{derive_read_from_file_impl, read_from_file_foreign_with_new_impl};
use crate::write_to_file::derive_write_to_file_impl;
use syn::DeriveInput;
use syn::parse_macro_input;

mod read_from_file;
mod write_to_file;

#[proc_macro_derive(ReadFromFile, attributes(read_from_file_bounds))]
pub fn derive_read_from_file(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_read_from_file_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
pub fn read_from_file_foreign_with_new(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    read_from_file_foreign_with_new_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(WriteToFile)]
pub fn derive_write_to_file(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_write_to_file_impl(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
