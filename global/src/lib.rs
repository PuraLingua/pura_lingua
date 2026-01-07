#![feature(marker_trait_attr)]
#![feature(macro_metavar_expr)]
#![feature(can_vector)]
#![feature(write_all_vectored)]
#![feature(read_buf)]
#![feature(core_io_borrowed_buf)]
#![feature(pattern)]
#![feature(decl_macro)]
#![feature(panic_internals)]
#![feature(format_args_nl)]
#![feature(iterator_try_collect)]
#![feature(trait_alias)]
#![feature(const_trait_impl)]
#![feature(const_default)]
#![feature(if_let_guard)]
#![feature(const_option_ops)]
#![feature(super_let)]
#![feature(const_format_args)]
#![feature(derive_const)]
#![feature(const_clone)]
#![feature(closure_track_caller)]
#![feature(const_convert)]
#![feature(const_range)]
#![feature(const_try)]
#![feature(ptr_as_ref_unchecked)]
#![feature(const_ops)]
#![feature(const_cmp)]
#![feature(const_destruct)]
#![feature(generic_const_exprs)]
#![feature(negative_impls)]
//
#![allow(internal_features)]
#![allow(static_mut_refs)]
#![allow(incomplete_features)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("unsupported");

extern crate proc_macros;

#[doc(hidden)]
pub extern crate iota;
#[doc(hidden)]
pub extern crate paste;

pub mod assertions;
pub mod attrs;
pub mod configs;
pub mod find_util;
pub mod freeze_lock;
pub mod instruction;
pub mod io_utils;
pub mod macros;
pub mod sync;
pub mod traits;

pub mod color;
pub mod path_searcher;

// Re-exports
pub use anyhow::{Error, Result};
pub use cfg_if::cfg_if;
pub use derive_ctor;
pub use faststr::FastStr;
pub use getset;
pub use global_errors as errors;
pub use indexmap::{
    IndexMap, IndexSet, indexmap, indexmap_with_default, indexset, indexset_with_default,
};
pub use macros::*;
pub use num_enum;
pub use proc_macros::*;
pub use string_name::StringName;

#[doc(hidden)]
pub mod __internal {
    pub use enumflags2;
}
