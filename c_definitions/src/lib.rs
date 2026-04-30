#![feature(ptr_metadata)]
#![feature(derive_const)]
#![feature(const_clone)]

mod slice;
mod vtable;

pub use slice::*;
pub use vtable::*;
