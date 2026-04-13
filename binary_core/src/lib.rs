#![feature(ptr_metadata)]
#![feature(const_convert)]
#![feature(const_trait_impl)]
#![feature(io_const_error)]
#![feature(const_eval_select)]
#![feature(core_intrinsics)]
#![feature(offset_of_enum)]
#![feature(const_index)]
#![feature(decl_macro)]
#![feature(trivial_bounds)]
#![feature(read_array)]
#![feature(const_type_name)]
#![feature(derive_const)]
#![feature(const_cmp)]
#![feature(const_clone)]
#![feature(const_range)]
#![feature(const_ops)]
#![feature(optimize_attribute)]
#![feature(trusted_len)]
#![feature(const_try)]
#![feature(const_result_trait_fn)]
#![feature(specialization)]
#![feature(nonzero_internals)]
#![allow(internal_features, incomplete_features)]

pub mod error;
pub mod file;
pub mod section;
pub mod traits;

pub use error::{BinaryResult, Error};
pub use file::{File, FileParser};

mod integers;
pub use integers::*;
