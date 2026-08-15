#![feature(ptr_metadata)]
#![feature(derive_const)]
#![feature(const_clone)]
#![feature(const_heap)]
#![feature(slice_ptr_get)]
#![feature(const_trait_impl)]
#![feature(specialization)]
#![cfg_attr(test, feature(drop_guard))]
#![feature(core_intrinsics)]
#![feature(allocator_api)]
#![allow(internal_features, incomplete_features)]

mod c_any;
mod slice;
mod vtable;

pub use c_any::*;
pub use slice::*;
pub use vtable::*;
