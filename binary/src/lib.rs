#![feature(iterator_try_collect)]
#![feature(try_find)]
#![feature(const_trait_impl)]
#![feature(decl_macro)]
#![feature(marker_trait_attr)]
#![feature(generic_const_exprs)]
#![feature(macro_metavar_expr)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(cold_path)]
#![feature(char_max_len)]
#![feature(super_let)]
#![feature(specialization)]
#![feature(derive_const)]
#![feature(const_clone)]
#![feature(panic_internals)]
#![feature(const_format_args)]
#![feature(ptr_as_ref_unchecked)]
#![feature(const_default)]
#![feature(const_try)]
#![allow(internal_features)]
#![allow(incomplete_features)]

pub mod assembly;
pub mod core;
mod custom_attribute;
mod implement;
// #[cfg(test)]
// mod tests;
mod ty;

pub(crate) type Error = global::errors::BinaryError;

pub use assembly::Assembly;
pub use custom_attribute::*;
pub use ty::*;
pub use types::item_token;
