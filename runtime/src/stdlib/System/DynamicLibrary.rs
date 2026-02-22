use std::ffi::c_void;
#[cfg(unix)]
use std::ptr::NonNull;

use crate::{
    stdlib::System_DynamicLibrary_FieldId,
    type_system::{class::Class, method::Method},
    value::managed_reference::{FieldAccessor, ManagedReference, StringAccessor},
    virtual_machine::cpu::CPU,
};

pub extern "system" fn Constructor_String(
    cpu: &CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
    file: ManagedReference<Class>,
) {
    let handle_out = this
        .const_access_mut::<FieldAccessor<Class>>()
        .typed_field_mut::<*mut c_void>(
            System_DynamicLibrary_FieldId::Handle as _,
            Default::default(),
        )
        .unwrap();
    let Some(handle) = LoadLibrary(cpu, file) else {
        return;
    };
    *handle_out = handle.as_ptr();
}

pub extern "system" fn GetSymbol(
    cpu: &CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
    name: ManagedReference<Class>,
) -> Option<NonNull<c_void>> {
    let handle = this
        .const_access_mut::<FieldAccessor<Class>>()
        .read_typed_field::<*mut c_void>(
            System_DynamicLibrary_FieldId::Handle as _,
            Default::default(),
        )
        .unwrap();
    GetSymbolImpl(cpu, handle, name)
}

pub extern "system" fn Destructor(
    cpu: &CPU,
    _: &Method<Class>,
    this: &mut ManagedReference<Class>,
) {
    let handle = this
        .const_access_mut::<FieldAccessor<Class>>()
        .read_typed_field::<*mut c_void>(
            System_DynamicLibrary_FieldId::Handle as _,
            Default::default(),
        )
        .unwrap();
    FreeLibrary(cpu, handle);
    ClearHandlePtr(this);
}

fn ClearHandlePtr(this: &mut ManagedReference<Class>) {
    assert!(
        this.const_access_mut::<FieldAccessor<Class>>()
            .write_typed_field::<*mut c_void>(
                System_DynamicLibrary_FieldId::Handle as _,
                Default::default(),
                std::ptr::null_mut()
            )
    );
}

// cSpell:disable
#[cfg(windows)]
fn LoadLibrary(cpu: &CPU, file: ManagedReference<Class>) -> Option<NonNull<c_void>> {
    match unsafe {
        windows::Win32::System::LibraryLoader::LoadLibraryW(windows::core::PCWSTR::from_raw(
            file.access::<StringAccessor>()
                .unwrap()
                .get_str()
                .unwrap()
                .as_ptr(),
        ))
    } {
        Ok(x) => {
            debug_assert!(!x.is_invalid());
            Some(x.0)
        }
        Err(e) => {
            println!("WIN32 API ERROR: {}", e.message());
            assert!(cpu.throw_helper().win32(e.code().0));
            None
        }
    }
}

#[cfg(unix)]
fn LoadLibrary(cpu: &CPU, file: ManagedReference<Class>) -> Option<NonNull<c_void>> {
    let file_path = std::ffi::CString::new(
        file.access::<StringAccessor>()
            .unwrap()
            .to_string()
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    let handle = unsafe { libc::dlopen(file_path.as_ptr(), libc::RTLD_LAZY) };
    match NonNull::new(handle) {
        Some(x) => Some(x),
        None => {
            assert!(cpu.throw_helper().current_dlerror());
            None
        }
    }
}

#[cfg(unix)]
fn GetSymbolImpl(
    cpu: &CPU,
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
            assert!(cpu.throw_helper().current_dlerror());
            None
        }
    }
}

#[cfg(unix)]
fn FreeLibrary(cpu: &CPU, handle: *mut c_void) {
    if handle.is_null() {
        return;
    }
    unsafe {
        if libc::dlclose(handle) != 0 {
            assert!(cpu.throw_helper().current_dlerror());
        }
    }
}

// cSpell:enable
