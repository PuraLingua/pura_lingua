#![feature(c_size_t)]
#![feature(exact_div)]
#![feature(mapped_lock_guards)]
#![feature(box_vec_non_null)]
#![allow(nonstandard_style)]

use std::{
    ffi::{CString, c_char, c_size_t, c_void},
    ptr::NonNull,
    sync::MappedRwLockReadGuard,
};

use c_definitions::SlicePtr;
use string_name::StringName;

#[unsafe(no_mangle)]
pub extern "C" fn Slice_Create(ptr: *mut c_void, len: usize) -> SlicePtr<c_void> {
    SlicePtr { len, ptr }
}

/// Usually used by C++
#[unsafe(no_mangle)]
pub extern "C" fn Slice_CreateFromRange(
    start: *mut c_void,
    end: *mut c_void,
    size: c_size_t,
) -> SlicePtr<c_void> {
    let offset = unsafe { end.byte_offset_from(end) };
    assert!(offset > 0);
    let len = offset.cast_unsigned().div_exact(size).unwrap();
    SlicePtr { len, ptr: start }
}

#[unsafe(no_mangle)]
pub extern "C" fn CString_Drop(data: *mut c_char) {
    unsafe {
        drop(CString::from_raw(data));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn StringName_DropNoDealloc(name: NonNull<StringName>) {
    unsafe {
        name.drop_in_place();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn StringName_ToCString(name: &StringName) -> *mut c_char {
    CString::new(name.as_str()).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn MappedErrorGuard_Drop(guard: NonNull<MappedRwLockReadGuard<'_, anyhow::Error>>) {
    unsafe {
        drop(Box::from_non_null(guard));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn MappedErrorGuard_AsRef(
    guard: NonNull<MappedRwLockReadGuard<'_, anyhow::Error>>,
) -> &anyhow::Error {
    unsafe { &**guard.as_ref() }
}

/// Returns owned ptr
#[unsafe(no_mangle)]
pub extern "C" fn AnyhowError_ToString(err: &anyhow::Error) -> *mut c_char {
    CString::into_raw(CString::new(format!("{err}")).unwrap())
}
