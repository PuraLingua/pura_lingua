use std::{borrow::Cow, ffi::c_void, ptr::NonNull};

use widestring::U16CStr;

use crate::{
    type_system::class::Class,
    value::managed_reference::{ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

// cSpell:disable

pub(in super::super) type LibraryPointer = NonNull<c_void>;

pub(in super::super) fn LoadLibraryImpl(
    cpu: &mut CPU,
    file: ManagedReference<Class>,
) -> Option<LibraryPointer> {
    let handle = LoadLibraryInner(file.access::<StringAccessor>().unwrap().get_str().unwrap());
    match NonNull::new(handle) {
        Some(x) => Some(x),
        None => {
            assert!(cpu.throw_helper_mut().current_dlerror());
            None
        }
    }
}

pub(super) fn LoadLibraryInner(name: &U16CStr) -> *mut c_void {
    let name = match name {
        x if x == widestring::u16cstr!("$libc") => {
            cfg_select! {
                target_vendor = "apple" => {
                    Cow::Borrowed(widestring::u16cstr!("/usr/lib/libc.dylib"))
                }
                target_os = "freebsd" => {
                    Cow::Borrowed(widestring::u16cstr!("libc.so.7"))
                }
                target_os = "linux" => {
                    Cow::Borrowed(widestring::u16cstr!("libc.so.6"))
                }
                _ => {
                    Cow::Borrowed(widestring::u16cstr!("libc.so"))
                }
            }
        }
        x if x == widestring::u16cstr!("$RUNTIME") => {
            return unsafe { libc::dlopen(std::ptr::null(), libc::RTLD_LAZY) };
        }

        _ => Cow::Borrowed(name),
    };

    let file_path = std::ffi::CString::new(name.to_string().unwrap()).unwrap();
    return unsafe { libc::dlopen(file_path.as_ptr(), libc::RTLD_LAZY) };
}

pub(in super::super) fn GetSymbolImpl(
    cpu: &mut CPU,
    handle: *mut c_void,
    name: ManagedReference<Class>,
) -> Option<NonNull<c_void>> {
    let c_name = std::ffi::CString::new(
        name.access::<StringAccessor>()
            .unwrap()
            .to_string()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    match NonNull::new(unsafe { libc::dlsym(handle, c_name.as_ptr()) }) {
        Some(x) => Some(x),
        None => {
            assert!(cpu.throw_helper_mut().current_dlerror());
            None
        }
    }
}

#[must_use]
pub(in super::super) fn FreeLibraryImpl(cpu: &mut CPU, handle: *mut c_void) -> bool {
    if handle.is_null() {
        return true;
    }
    unsafe {
        if libc::dlclose(handle) != 0 {
            assert!(cpu.throw_helper_mut().current_dlerror());
            false
        } else {
            true
        }
    }
}
