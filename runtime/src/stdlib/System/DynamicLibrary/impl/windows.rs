use std::{borrow::Cow, ffi::c_void, ptr::NonNull};

use widestring::{U16CStr, u16cstr};
use windows::{
    Win32::{
        Foundation::{FreeLibrary, HMODULE},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
    },
    core::PCWSTR,
};

use crate::{
    type_system::class::Class,
    value::managed_reference::{ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

pub(in super::super) type LibraryPointer = HMODULE;

pub(in super::super) fn LoadLibraryImpl(
    cpu: &mut CPU,
    file: ManagedReference<Class>,
) -> Option<LibraryPointer> {
    match LoadLibraryInner(file.access::<StringAccessor>().unwrap().get_str().unwrap()) {
        Ok(x) => Some(x),
        Err(e) => {
            println!("WIN32 API ERROR: {}", e.message());
            assert!(cpu.throw_helper_mut().win32(e.code().0));
            None
        }
    }
}

pub(super) fn LoadLibraryInner(name: &U16CStr) -> windows::core::Result<HMODULE> {
    let name = match name {
        x if x == u16cstr!("$RUNTIME") => {
            return unsafe { LoadLibraryW(PCWSTR::null()) };
        }

        _ => Cow::Borrowed(name),
    };

    unsafe { LoadLibraryW(PCWSTR::from_raw(name.as_ptr())) }
}

pub(in super::super) fn GetSymbolImpl(
    cpu: &mut CPU,
    handle: *mut c_void,
    name: ManagedReference<Class>,
) -> Option<NonNull<c_void>> {
    let hModule = HMODULE(handle);

    unsafe {
        let (name_out, used_len) = name
            .access::<StringAccessor>()
            .unwrap()
            .to_multi_byte()
            .unwrap();

        if used_len.is_none() {
            assert!(cpu.throw_helper_mut().current_win32());
            return None;
        }
        match GetProcAddress(hModule, windows::core::PCSTR::from_raw(name_out.as_ptr())) {
            Some(x) => Some(NonNull::new_unchecked(x as _)),
            None => {
                assert!(cpu.throw_helper_mut().current_win32());
                None
            }
        }
    }
}

#[must_use]
pub(in super::super) fn FreeLibraryImpl(cpu: &mut CPU, handle: *mut c_void) -> bool {
    let hLibModule = HMODULE(handle);
    match unsafe { FreeLibrary(hLibModule) } {
        Ok(_) => true,
        Err(e) => {
            println!("WIN32 API ERROR: {}", e.message());
            assert!(cpu.throw_helper_mut().win32(e.code().0));
            false
        }
    }
}
