#![feature(allocator_api)]
#![feature(mapped_lock_guards)]
#![feature(box_vec_non_null)]
#![feature(iterator_try_collect)]
#![feature(lock_value_accessors)]
// lints
#![allow(nonstandard_style)]

use std::{
    ptr::NonNull,
    sync::{MappedRwLockReadGuard, RwLock, RwLockReadGuard},
};

use pura_lingua::global;

#[cfg(test)]
mod test_assertions;

pub mod type_system;
pub mod virtual_machine;

pub static PURALINGUA_RUNTIME_ERROR: RwLock<Option<global::Error>> = RwLock::new(None);

fn value_to_owned_ptr<T>(val: T) -> NonNull<T> {
    Box::into_non_null(Box::new(val))
}

#[unsafe(no_mangle)]
pub extern "C" fn GetLastPuralinguaRuntimeError()
-> Option<NonNull<MappedRwLockReadGuard<'static, global::Error>>> {
    RwLockReadGuard::filter_map(PURALINGUA_RUNTIME_ERROR.read().unwrap(), |x| x.as_ref())
        .ok()
        .map(value_to_owned_ptr)
}
